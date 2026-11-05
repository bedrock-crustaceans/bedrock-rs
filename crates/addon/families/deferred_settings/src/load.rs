//! Loading the deferred renderer's settings: the eight payloads' chain parameters, `Content` /
//! `Definition` / `Placed`, the walk of their folders and the group's combine rules.

use std::convert::Infallible;

use super::file::{self, Payload};
use super::history;
use super::{
    AtmosphereSettings, ColorGradingSettings, CubemapSettings, DefaultSettings, DeferredSettings,
    LightingSettings, LocalLightSettings, PbrFallbackSettings, ShadowSettings, WaterSettings,
};
use std::collections::HashSet;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity,
    VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, FilesBuilder, FilesRule, JsonContent,
    Merge, Merged, NoBeta, Notice, Origin, Placed, Registry, RegistryBuilder, Rejected,
    default_path,
};
use bedrock_addon_core::id::{BlockId, CubemapSettingsId, Id, Kind, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle, write_json};
use bedrock_addon_core::load::Reported;
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    Family, Loaded, MissingVersion, NoComponents, RangeModel, ValidateCx, Versioned,
};

/// `lighting`: `minecraft:lighting_settings` documents.
pub(crate) const LIGHTING: &str = "lighting";
/// `atmospherics`: `minecraft:atmosphere_settings` documents.
pub(crate) const ATMOSPHERICS: &str = "atmospherics";
/// `water`: `minecraft:water_settings` documents.
pub(crate) const WATER: &str = "water";
/// `color_grading`: `minecraft:color_grading_settings` documents.
pub(crate) const COLOR_GRADING: &str = "color_grading";
/// `pbr`: the one `minecraft:pbr_fallback_settings` document.
pub(crate) const PBR: &str = "pbr";
/// `shadows`: the one `minecraft:shadow_settings` document.
pub(crate) const SHADOWS: &str = "shadows";
/// `cubemaps`: `minecraft:cubemap_settings` documents.
pub(crate) const CUBEMAPS: &str = "cubemaps";
/// `local_lighting`: the one `minecraft:local_light_settings` document of each pack.
pub(crate) const LOCAL_LIGHTING: &str = "local_lighting";

/// The file whose identifier is the default lighting.
const LIGHTING_DEFAULT: &str = "lighting/global.json";
/// The file whose identifier is the default atmosphere.
const ATMOSPHERICS_DEFAULT: &str = "atmospherics/atmospherics.json";
/// The file whose identifier is the default water.
const WATER_DEFAULT: &str = "water/water.json";
/// The file whose identifier is the default colour grading.
const COLOR_GRADING_DEFAULT: &str = "color_grading/color_grading.json";

/// The identifier of the default cubemap settings, fixed by the game.
pub(crate) const DEFAULT_CUBEMAP: &str = "minecraft:default_cubemap";

// The struct versions.
pub(crate) const V1_21_40: FormatVersion = FormatVersion::new(1, 21, 40);
pub(crate) const V1_21_60: FormatVersion = FormatVersion::new(1, 21, 60);
pub(crate) const V1_21_70: FormatVersion = FormatVersion::new(1, 21, 70);
pub(crate) const V1_21_80: FormatVersion = FormatVersion::new(1, 21, 80);
pub(crate) const V1_21_90: FormatVersion = FormatVersion::new(1, 21, 90);
pub(crate) const V1_21_120: FormatVersion = FormatVersion::new(1, 21, 120);
pub(crate) const V1_21_130: FormatVersion = FormatVersion::new(1, 21, 130);
pub(crate) const V1_26_0: FormatVersion = FormatVersion::new(1, 26, 0);
pub(crate) const V1_26_20: FormatVersion = FormatVersion::new(1, 26, 20);

/// The `Versioned` impls of the structs of one chain.
macro_rules! versions {
    ($($ty:ty = $version:expr;)*) => {
        $(impl Versioned for $ty {
            const VERSION: FormatVersion = $version;
        })*
    };
}

versions! {
    history::lighting::v1_21_40::LightingSettings = V1_21_40;
    history::lighting::v1_21_60::LightingSettings = V1_21_60;
    history::lighting::v1_21_70::LightingSettings = V1_21_70;
    history::lighting::v1_21_80::LightingSettings = V1_21_80;
    LightingSettings = V1_26_0;
    history::water::v1_21_40::WaterSettings = V1_21_40;
    history::water::v1_21_80::WaterSettings = V1_21_80;
    history::water::v1_21_120::WaterSettings = V1_21_120;
    history::water::v1_26_0::WaterSettings = V1_26_0;
    WaterSettings = V1_26_20;
    history::atmosphere::v1_21_40::AtmosphereSettings = V1_21_40;
    AtmosphereSettings = V1_26_20;
    history::color_grading::v1_21_40::ColorGradingSettings = V1_21_40;
    ColorGradingSettings = V1_21_90;
    CubemapSettings = V1_21_130;
    ShadowSettings = V1_21_80;
    PbrFallbackSettings = V1_21_40;
    LocalLightSettings = V1_21_120;
}

/// A payload: its `Family` (exact versions only, no struct for the game's experimental
/// builds, a missing `format_version` rejected), `Payload`, `Content` and
/// `parse_as_written`.
macro_rules! payload {
    (
        $latest:ident, history::$module:ident,
        name = $name:literal, key = $key:literal, directory = $directory:expr,
        floor = $floor:expr $(, payload_required = $required:expr)?;
        identifier = |$value:ident| $identifier:expr $(, named = $named:expr)?
    ) => {
        impl Family for history::$module::Family {
            const NAME: &'static str = $name;
            const PAYLOAD_KEY: &'static str = $key;
            const DIRECTORIES: &'static [&'static str] = &[$directory];
            const FLOOR: FormatVersion = $floor;
            // Every struct's parser takes exactly its own version, so its range is
            // `[VERSION, VERSION]`.
            const RANGE_MODEL: RangeModel = RangeModel::ExactOnly;
            type Latest = $latest;
            type Beta = Infallible;
            type Component = NoComponents;
            type Any = history::$module::Any;
            const MISSING_VERSION: MissingVersion = MissingVersion::Reject;

            fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
                if let Loaded::Stable(value) = v {
                    file::unnamed::<Self>(value, cx);
                }
            }
        }

        impl Payload for history::$module::Family {
            $(const PAYLOAD_REQUIRED: Option<FormatVersion> = Some($required);)?
            $(const NAMED: bool = $named;)?

            fn identifier($value: &$latest) -> Option<&str> {
                $identifier
            }
        }

        impl bedrock_addon_core::__private::Sealed for $latest {}

        impl JsonContent for $latest {}

        impl Content for $latest {
            const NAME: &'static str = $name;
            const SIDES: &'static [PackSide] = &[PackSide::Resource];

            fn parse(
                path: &PackPath,
                bytes: &[u8],
                cx: &LoadContext,
            ) -> Result<Document<Self>, Rejected> {
                file::parse_file::<history::$module::Family>(PackRef::app_data(), path, bytes, cx)
            }

            fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
                write_json(&file::document::<history::$module::Family>(self)?, out, style)
            }
        }

        impl $latest {
            /// Parses one file without upgrading it: the struct its `format_version` selects.
            ///
            /// # Errors
            ///
            /// Returns [`Rejected`](bedrock_addon_core::document::Rejected) with the findings the game's content
            /// log would print when the bytes cannot be read as this file at all: not JSON, a
            /// `format_version` with no struct here, or a document that does not fit the struct it selects.
            pub fn parse_as_written(
                path: &PackPath,
                bytes: &[u8],
                cx: &LoadContext,
            ) -> Result<Document<history::$module::Any>, Rejected> {
                file::as_written::<history::$module::Family>(path, bytes, cx)
            }
        }
    };
}

payload! {
    LightingSettings, history::lighting,
    name = "lighting settings", key = "minecraft:lighting_settings", directory = LIGHTING,
    floor = V1_21_40;
    identifier = |value| value.description.as_ref().map(|d| d.identifier.as_str())
}
payload! {
    AtmosphereSettings, history::atmosphere,
    name = "atmosphere settings", key = "minecraft:atmosphere_settings",
    directory = ATMOSPHERICS, floor = V1_21_40, payload_required = V1_26_20;
    identifier = |value| value.description.as_ref().map(|d| d.identifier.as_str())
}
payload! {
    WaterSettings, history::water,
    name = "water settings", key = "minecraft:water_settings", directory = WATER,
    floor = V1_21_40, payload_required = V1_26_20;
    identifier = |value| value.description.as_ref().map(|d| d.identifier.as_str())
}
payload! {
    ColorGradingSettings, history::color_grading,
    name = "color grading settings", key = "minecraft:color_grading_settings",
    directory = COLOR_GRADING, floor = V1_21_40;
    identifier = |value| value
        .description
        .as_ref()
        .and_then(|d| d.identifier.as_ref())
        .map(Id::as_str)
}
payload! {
    CubemapSettings, history::cubemap,
    name = "cubemap settings", key = "minecraft:cubemap_settings", directory = CUBEMAPS,
    floor = V1_21_130;
    identifier = |value| value.description.as_ref().map(|d| d.identifier.as_str())
}
payload! {
    ShadowSettings, history::shadows,
    name = "shadow settings", key = "minecraft:shadow_settings", directory = SHADOWS,
    floor = V1_21_80;
    identifier = |_value| None, named = false
}
payload! {
    PbrFallbackSettings, history::pbr_fallback,
    name = "PBR fallback settings", key = "minecraft:pbr_fallback_settings", directory = PBR,
    floor = V1_21_40;
    identifier = |_value| None, named = false
}
payload! {
    LocalLightSettings, history::local_lighting,
    name = "local light settings", key = "minecraft:local_light_settings",
    directory = LOCAL_LIGHTING, floor = V1_21_120;
    identifier = |_value| None, named = false
}

/// `Definition` and `Placed` of a payload keyed by its identifier: one definition per file,
/// `<directory>/<identifier with ':' as '_'>.json`.
macro_rules! definition {
    ($latest:ident, $kind:ident, $directory:expr, $family:ty, $fallback:literal) => {
        impl Definition for $latest {
            type Kind = kind::$kind;
            type File = $latest;
            type Beta = NoBeta<kind::$kind>;
        }

        impl Placed for $latest {
            /// The file directly in the payload's folder, named after the identifier with
            /// `:` as `_`; a value without an identifier is named after the payload.
            fn default_path(&self) -> PackPath {
                let id = <$family as Payload>::identifier(self).unwrap_or($fallback);
                flat_path($directory, &Id::<kind::$kind>::parse(id))
            }
        }
    };
}

definition!(
    LightingSettings,
    LightingSettings,
    LIGHTING,
    history::lighting::Family,
    "lighting"
);
definition!(
    AtmosphereSettings,
    AtmosphereSettings,
    ATMOSPHERICS,
    history::atmosphere::Family,
    "atmospherics"
);
definition!(
    WaterSettings,
    WaterSettings,
    WATER,
    history::water::Family,
    "water"
);
definition!(
    ColorGradingSettings,
    ColorGradingSettings,
    COLOR_GRADING,
    history::color_grading::Family,
    "color_grading"
);
definition!(
    CubemapSettings,
    CubemapSettings,
    CUBEMAPS,
    history::cubemap::Family,
    "cubemap"
);

/// `<directory>/<identifier with ':' as '_'>.json`: the default path of an identifier that
/// has no namespace rules.
fn flat_path<K: Kind>(directory: &str, id: &Id<K>) -> PackPath {
    default_path(directory, id)
}

impl Placed for ShadowSettings {
    /// `shadows/global.json`: the one path the game reads.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(DeferredSettings::SHADOWS_PATH)
    }
}

impl Placed for PbrFallbackSettings {
    /// `pbr/global.json`: the one path the game reads.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(DeferredSettings::PBR_FALLBACK_PATH)
    }
}

impl Placed for LocalLightSettings {
    /// `local_lighting/local_lighting.json`: the one path the game reads.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(DeferredSettings::LOCAL_LIGHTING_PATH)
    }
}

impl Merge for LocalLightSettings {
    /// One entry per block.
    type Key = BlockId;
}

// ---- the group -----------------------------------------------------------------------------

/// One walked file of payload `T`: the path the walk keys it by (without a subpack folder)
/// and what it parsed to.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct Walked<T> {
    key: PackPath,
    outcome: Result<Document<T>, Rejected>,
}

/// What one walked file parses to.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum SettingsFile {
    /// A `minecraft:lighting_settings` file.
    Lighting(Box<Walked<LightingSettings>>),
    /// A `minecraft:atmosphere_settings` file.
    Atmosphere(Box<Walked<AtmosphereSettings>>),
    /// A `minecraft:water_settings` file.
    Water(Box<Walked<WaterSettings>>),
    /// A `minecraft:color_grading_settings` file.
    ColorGrading(Box<Walked<ColorGradingSettings>>),
    /// `pbr/global.json`.
    PbrFallback(Box<Walked<PbrFallbackSettings>>),
    /// `shadows/global.json`.
    Shadows(Box<Walked<ShadowSettings>>),
    /// A `minecraft:cubemap_settings` file.
    Cubemap(Box<Walked<CubemapSettings>>),
    /// A pack's `local_lighting/local_lighting.json`.
    LocalLighting(Box<Walked<LocalLightSettings>>),
}

impl<T> Walked<T> {
    /// The pack the file is in.
    fn pack(&self) -> &PackRef {
        match &self.outcome {
            Ok(document) => &document.origin().pack,
            Err(rejected) => &rejected.origin.pack,
        }
    }
}

impl SettingsFile {
    /// The pack the file is in.
    fn pack(&self) -> &PackRef {
        match self {
            Self::Lighting(walked) => walked.pack(),
            Self::Atmosphere(walked) => walked.pack(),
            Self::Water(walked) => walked.pack(),
            Self::ColorGrading(walked) => walked.pack(),
            Self::PbrFallback(walked) => walked.pack(),
            Self::Shadows(walked) => walked.pack(),
            Self::Cubemap(walked) => walked.pack(),
            Self::LocalLighting(walked) => walked.pack(),
        }
    }
}

/// The folders in the order the game loads them on a resource reload.
const DIRECTORIES: &[&str] = &[
    LIGHTING,
    ATMOSPHERICS,
    WATER,
    COLOR_GRADING,
    PBR,
    SHADOWS,
    CUBEMAPS,
    LOCAL_LIGHTING,
];

/// Whether the game reads the file at `path`.
///
/// - In the five identifier folders: a path of five or more bytes that ends in `.json` (a
///   case-sensitive compare; `x.JSON` is skipped without a message).
/// - `pbr/global.json` and `shadows/global.json` exactly.
/// - Below `local_lighting/`: a path that ends with `local_lighting/local_lighting.json`.
#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "the game's compare is case-sensitive"
)]
pub(crate) fn is_read(path: &PackPath) -> bool {
    let path = path.as_str();
    let Some((directory, _)) = path.split_once('/') else {
        return false;
    };
    match directory {
        PBR => path == DeferredSettings::PBR_FALLBACK_PATH,
        SHADOWS => path == DeferredSettings::SHADOWS_PATH,
        LOCAL_LIGHTING => path.ends_with(DeferredSettings::LOCAL_LIGHTING_PATH),
        _ => path.len() >= 5 && path.ends_with(".json"),
    }
}

/// Each folder over every pack from the highest priority down, sub-folders included (a
/// directory pack outside the app package lists them).
const WALK: Walk = Walk {
    directories: DIRECTORIES,
    depth: ListDepth::Recursive,
    accept: Accept::Fn(is_read),
    order: WalkOrder::TopDown,
    directory_major: true,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

/// Parses one walked file by its folder.
fn parse_walk_file(walked: &WalkFile, cx: &LoadContext) -> SettingsFile {
    let (pack, path, bytes) = (walked.pack.clone(), &walked.path, &walked.bytes);
    macro_rules! parse {
        ($variant:ident, $module:ident) => {
            SettingsFile::$variant(Box::new(Walked {
                key: walked.key.clone(),
                outcome: file::parse_file::<history::$module::Family>(pack, path, bytes, cx),
            }))
        };
    }
    match WALK.matches(&walked.key).map(|index| DIRECTORIES[index]) {
        Some(ATMOSPHERICS) => parse!(Atmosphere, atmosphere),
        Some(WATER) => parse!(Water, water),
        Some(COLOR_GRADING) => parse!(ColorGrading, color_grading),
        Some(PBR) => parse!(PbrFallback, pbr_fallback),
        Some(SHADOWS) => parse!(Shadows, shadows),
        Some(CUBEMAPS) => parse!(Cubemap, cubemap),
        Some(LOCAL_LIGHTING) => parse!(LocalLighting, local_lighting),
        _ => parse!(Lighting, lighting),
    }
}

/// A finding about the document at `origin`, raised while the stack is combined.
fn combined(code: DiagnosticCode, severity: Severity, origin: &Origin, at: JsonPath) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Validate,
        Location::new(
            Some(origin.pack.clone()),
            Some(origin.path.clone()),
            at,
            None,
        ),
    )
}

/// `ShadowedDefinition` at the information level: a definition a higher-priority pack hides.
/// The game says nothing about it, so the finding is the crate's own.
fn shadowed(origin: &Origin, at: JsonPath) -> Diagnostic {
    combined(
        DiagnosticCode::ShadowedDefinition { text: None },
        Severity::Info,
        origin,
        at,
    )
}

/// The identifier map of one payload while it is built: per identifier the first pack of the
/// walk, which is the highest, wins.
struct IdentifierMap<T: Definition, F> {
    builder: RegistryBuilder<T>,
    /// The pack whose files are being added, and the identifiers it has defined so far.
    pack: Option<PackRef>,
    in_pack: HashSet<Id<T::Kind>>,
    /// The identifier of the first loaded default file that has one.
    default: Option<Id<T::Kind>>,
    family: std::marker::PhantomData<F>,
}

impl<T, F> IdentifierMap<T, F>
where
    T: Definition + Clone,
    F: Payload<Latest = T>,
{
    fn new(stack: &StackCx<'_>) -> Self {
        Self {
            builder: RegistryBuilder::new(
                CombineRule::TopWins {
                    shadowed: Some(Notice::shadowed()),
                },
                CombineRule::top_wins(),
                stack.options,
            ),
            pack: None,
            in_pack: HashSet::new(),
            default: None,
            family: std::marker::PhantomData,
        }
    }

    /// Adds the next file of the walk; `default_file` is the payload's default file, when it
    /// has one.
    ///
    /// - A file that failed is listed with its findings and reserves nothing.
    /// - A file without an identifier loaded, but is not indexed (the game keys it by the
    ///   empty string).
    /// - A second file of an identifier in one pack is skipped with the game's error; the
    ///   first one, in path order, stays.
    /// - The first file at `default_file` that passes all of that names the payload's
    ///   default identifier.
    fn push(&mut self, walked: Walked<T>, default_file: Option<&str>) {
        let document = match walked.outcome {
            Ok(document) => document,
            Err(rejected) => {
                self.builder.push(Contribution::Rejected(None, rejected));
                return;
            }
        };
        if self.pack.as_ref() != Some(&document.origin().pack) {
            self.pack = Some(document.origin().pack.clone());
            self.in_pack.clear();
        }
        let Some(id) = file::named::<F>(document.value()).map(Id::<T::Kind>::parse) else {
            for finding in document.diagnostics().iter().cloned() {
                self.builder.push_diagnostic(finding);
            }
            return;
        };
        if !self.in_pack.insert(id.clone()) {
            let origin = document.origin().clone();
            let mut diagnostics = document.diagnostics().clone();
            diagnostics.push(combined(
                DiagnosticCode::DuplicateIdentifier {
                    text: Some(VanillaText::Formatted {
                        template: "Identifier {} already exists in resource pack",
                        args: vec![id.as_str().to_owned()],
                    }),
                },
                Severity::Error,
                &origin,
                JsonPath::new(),
            ));
            self.builder.push(Contribution::Rejected(
                None,
                Rejected::new(origin, diagnostics),
            ));
            return;
        }
        if self.default.is_none() && default_file == Some(walked.key.as_str()) {
            self.default = Some(id.clone());
        }
        self.builder.push(Contribution::Stable(id, document));
    }

    /// The registry and the default identifier.
    fn finish(self) -> (Registry<T>, Option<Id<T::Kind>>) {
        (self.builder.finish(), self.default)
    }
}

/// The finding of a default identifier `id` that `registry` does not define: an
/// `UnresolvedReference` kept error with the game's text; the game then uses its own values.
/// For each of the four kinds the game logs the line at the error level and leaves its
/// default as it was, so the finding is an error that rejects nothing.
pub(crate) fn unresolved_default<T: Definition>(
    registry: &Registry<T>,
    id: &Id<T::Kind>,
    noun: &'static str,
) -> Option<Diagnostic> {
    if registry.contains(id) {
        return None;
    }
    let mut finding = Diagnostic::new(
        DiagnosticCode::UnresolvedReference {
            text: Some(VanillaText::Formatted {
                template: "Could not find any matching settings for {} identifier '{}'",
                args: vec![noun.to_owned(), id.as_str().to_owned()],
            }),
        },
        Severity::Error,
        Phase::Validate,
        Location::default(),
    )
    .into_kept();
    finding.detail = Some(Box::new(DiagnosticDetail::Reference {
        kind: <T::Kind as Kind>::NAME,
        id: id.as_str().to_owned(),
        hint: None,
    }));
    Some(finding)
}

/// A payload's map with the default identifier its default file gave, if one did.
type WithDefault<'a, T> = (&'a Registry<T>, Option<&'a Id<<T as Definition>::Kind>>);

/// [`unresolved_default`] for the four payloads that have a default identifier, each with the
/// noun the game's message names it by, in the order lighting, atmosphere, water, colour
/// grading.
pub(crate) fn unresolved_defaults(
    lighting: WithDefault<'_, LightingSettings>,
    atmosphere: WithDefault<'_, AtmosphereSettings>,
    water: WithDefault<'_, WaterSettings>,
    color_grading: WithDefault<'_, ColorGradingSettings>,
) -> impl Iterator<Item = Diagnostic> {
    fn one<T: Definition>(map: WithDefault<'_, T>, noun: &'static str) -> Option<Diagnostic> {
        let (registry, id) = map;
        unresolved_default(registry, id?, noun)
    }
    [
        one(lighting, "lighting"),
        one(atmosphere, "atmosphere"),
        one(water, "water"),
        one(color_grading, "color grading"),
    ]
    .into_iter()
    .flatten()
}

/// One of the two single files while it is built: the first copy of the walk that loads, which
/// is the highest, wins; a copy that fails falls through to the next pack.
struct SingleFile<T: Content> {
    builder: FilesBuilder<T>,
    loaded: bool,
}

impl<T: Content> SingleFile<T> {
    fn new(stack: &StackCx<'_>) -> Self {
        Self {
            builder: FilesBuilder::new(
                FilesRule::TopWinsFallback { fallback: None },
                stack.options,
            ),
            loaded: false,
        }
    }

    /// Adds the next copy of the walk. The game stops at the first pack whose file loads; a
    /// lower copy that loads is a `ShadowedDefinition` info, and one that fails keeps its
    /// findings (the crate's own: the game never reads it).
    fn push(&mut self, walked: Walked<T>) {
        let hidden = match &walked.outcome {
            Ok(document) if self.loaded => Some(shadowed(document.origin(), JsonPath::new())),
            _ => None,
        };
        self.loaded |= walked.outcome.is_ok();
        self.builder.push(walked.key, walked.outcome);
        if let Some(finding) = hidden {
            self.builder.push_diagnostic(finding);
        }
    }
}

/// The local lights of every pack's file, from the highest pack down: per block the first
/// entry stays; a later one is a `ShadowedDefinition` info (the game is silent, so the
/// finding is the crate's own). A file that failed contributes nothing.
fn merge_local_lights(files: Vec<Walked<LocalLightSettings>>) -> Merged<LocalLightSettings> {
    let walk = files.into_iter().map(|walked| walked.outcome).collect();
    Merged::merge(
        walk,
        LocalLightSettings::default(),
        |merged, layer: &Document<LocalLightSettings>, cx| {
            for (block, light) in &layer.lights {
                if merged.lights.contains_key(block) {
                    let mut at = JsonPath::new();
                    at.push(history::local_lighting::Family::PAYLOAD_KEY);
                    at.push(block.as_str());
                    let finding = shadowed(cx.origin(), at);
                    cx.push_diagnostic(finding);
                } else {
                    merged.lights.insert(block.clone(), light.clone());
                    cx.claim(block.clone());
                }
            }
        },
    )
}

/// From this `min_engine_version` a third-party pack needs the `pbr` capability for Vibrant
/// Visuals to be available.
const CAPABILITY_FLOOR: (u32, u32, u32) = (1, 21, 110);
/// The capabilities either of which makes the deferred renderer available to a pack.
const DEFERRED_CAPABILITIES: [&str; 2] = ["pbr", "raytraced"];

/// One `CapabilityHint` info per pack of `shipping` (the packs with a settings file) that is
/// no base-game pack, has a `min_engine_version` of 1.21.110 or above and declares neither
/// `pbr` nor `raytraced`, in stack order: its settings load, and are used only when Vibrant
/// Visuals is available. The game has no message for this because no loader reads the
/// capabilities, so the hint is the crate's own.
fn capability_hints(shipping: &HashSet<PackRef>, stack: &StackCx<'_>) -> Diagnostics {
    let mut hints = Diagnostics::default();
    for entry in stack.packs {
        let recent = entry
            .context
            .min_engine_version
            .as_ref()
            .is_some_and(|min| {
                let version = &min.version;
                (
                    u32::from(version.major),
                    u32::from(version.minor),
                    u32::from(version.patch),
                ) >= CAPABILITY_FLOOR
            });
        let capable = entry
            .capabilities
            .iter()
            .any(|name| DEFERRED_CAPABILITIES.contains(&&**name));
        if recent && !capable && !entry.context.is_base_game_pack && shipping.contains(&entry.pack)
        {
            hints.push(Diagnostic::new(
                DiagnosticCode::CapabilityHint {
                    text: Some(VanillaText::Fixed(
                        "render configuration files are only used when Vibrant Visuals is available; add the \"pbr\" capability",
                    )),
                },
                Severity::Info,
                Phase::Validate,
                Location::new(Some(entry.pack.clone()), None, JsonPath::new(), None),
            ));
        }
    }
    hints
}

/// `point_lights`: the folder of `local_lighting`'s predecessor, which the game has no reader
/// for.
const POINT_LIGHTS: &str = "point_lights";

/// One `IgnoredFile` info per file below a settings folder that the game does not read
/// (the game is silent, so the finding is the crate's own): a file the walk refuses
/// ([`is_read`]) and every file below `point_lights/`. Per pack from the highest down, in the
/// order of the pack's listing, which names the files without their bytes.
fn ignored_files(stack: &StackCx<'_>) -> Diagnostics {
    let mut infos = Diagnostics::default();
    for entry in stack.packs {
        for file in entry.files() {
            let Some((directory, _)) = file.key.as_str().split_once('/') else {
                continue;
            };
            let ignored = directory == POINT_LIGHTS
                || (DIRECTORIES.contains(&directory) && !is_read(&file.key));
            if ignored {
                infos.push(Diagnostic::new(
                    DiagnosticCode::IgnoredFile { text: None },
                    Severity::Info,
                    Phase::Validate,
                    Location::new(
                        Some(entry.pack.clone()),
                        Some(file.path.clone()),
                        JsonPath::new(),
                        None,
                    ),
                ));
            }
        }
    }
    infos
}

/// The group, from every parsed file in walk order.
fn combine(parsed: Vec<SettingsFile>, stack: &StackCx<'_>) -> DeferredSettings {
    let mut lighting = IdentifierMap::<LightingSettings, history::lighting::Family>::new(stack);
    let mut atmosphere =
        IdentifierMap::<AtmosphereSettings, history::atmosphere::Family>::new(stack);
    let mut water = IdentifierMap::<WaterSettings, history::water::Family>::new(stack);
    let mut color_grading =
        IdentifierMap::<ColorGradingSettings, history::color_grading::Family>::new(stack);
    let mut cubemaps = IdentifierMap::<CubemapSettings, history::cubemap::Family>::new(stack);
    let mut shadows = SingleFile::<ShadowSettings>::new(stack);
    let mut pbr_fallback = SingleFile::<PbrFallbackSettings>::new(stack);
    let mut local = Vec::new();
    // The packs that ship a settings file, loaded or not.
    let mut shipping: HashSet<PackRef> = HashSet::new();
    for file in parsed {
        shipping.insert(file.pack().clone());
        match file {
            SettingsFile::Lighting(walked) => lighting.push(*walked, Some(LIGHTING_DEFAULT)),
            SettingsFile::Atmosphere(walked) => {
                atmosphere.push(*walked, Some(ATMOSPHERICS_DEFAULT));
            }
            SettingsFile::Water(walked) => water.push(*walked, Some(WATER_DEFAULT)),
            SettingsFile::ColorGrading(walked) => {
                color_grading.push(*walked, Some(COLOR_GRADING_DEFAULT));
            }
            // The default cubemap identifier is fixed: no file decides it.
            SettingsFile::Cubemap(walked) => cubemaps.push(*walked, None),
            SettingsFile::PbrFallback(walked) => pbr_fallback.push(*walked),
            SettingsFile::Shadows(walked) => shadows.push(*walked),
            SettingsFile::LocalLighting(walked) => local.push(*walked),
        }
    }
    let (lighting, default_lighting) = lighting.finish();
    let (atmosphere, default_atmosphere) = atmosphere.finish();
    let (water, default_water) = water.finish();
    let (color_grading, default_color_grading) = color_grading.finish();
    let (cubemaps, _) = cubemaps.finish();
    let mut diagnostics = capability_hints(&shipping, stack);
    diagnostics.extend(ignored_files(stack));
    // R50: a default identifier its map does not define. Each one is the identifier of a
    // definition just added, so the game's message has no case of its own here; the check
    // holds the two together.
    diagnostics.extend(unresolved_defaults(
        (&lighting, default_lighting.as_ref()),
        (&atmosphere, default_atmosphere.as_ref()),
        (&water, default_water.as_ref()),
        (&color_grading, default_color_grading.as_ref()),
    ));
    DeferredSettings {
        lighting,
        atmosphere,
        water,
        color_grading,
        cubemaps,
        shadows: shadows.builder.finish(),
        pbr_fallback: pbr_fallback.builder.finish(),
        local_lighting: merge_local_lights(local),
        defaults: DefaultSettings {
            lighting: default_lighting,
            atmosphere: default_atmosphere,
            water: default_water,
            color_grading: default_color_grading,
            cubemap: CubemapSettingsId::from_static(DEFAULT_CUBEMAP),
        },
        diagnostics,
    }
}

/// The loader of `rp.deferred_settings()`: the eight folders of every resource pack, combined
/// into the group.
pub const fn load_group() -> CollectionLoader<DeferredSettings, SettingsFile> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}

impl Reported for DeferredSettings {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A walked file and what it parsed to are printable and clonable, outside `cfg(test)` too.
    #[test]
    fn a_walked_settings_file_is_debug_and_clone() {
        fn assert_traits<T: std::fmt::Debug + Clone>() {}
        assert_traits::<Walked<LightingSettings>>();
        assert_traits::<SettingsFile>();
    }
}
