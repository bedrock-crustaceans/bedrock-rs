//! Biomes: `Biome`, `component::*`.
//!
//! Server biomes: `biomes/*.json` of a behaviour pack (`minecraft:biome`). `Biome` is the
//! newest document (format 1.26.30); older files are upgraded when they load, and `history` has
//! the older shapes; `Beta` is the `"format_version": "beta"` document.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod component;
mod environment;
pub mod history;
mod read;
mod texts;
mod validate;
mod write;

pub use environment::{
    BoolAttribute, BoolOperation, ColorAttribute, ColorOperation, EnvironmentAttributes,
    FloatAttribute, FloatOperation,
};

use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::id::BiomeId;

use component::BiomeComponent;

use facet_json::RawJson;
use indexmap::IndexMap;
// Every block a component names, for the link pass.
#[doc(hidden)]
pub use validate::blocks as block_references;

use crate::component::child;
use crate::history::{Any, BiomeFamily};
use crate::read::CEREAL;
use crate::texts::{DUPLICATE, WORLD_GATE, fixed, text as formatted};
use std::borrow::Cow;

use crate::component::{
    Adjustment, AdjustmentMaterials, BiomeReplacement, Builder, CappedSurface, ColorRgba,
    GrassTint, Humidity, MapTints, MesaSurface, NoiseBlockSpecifier, NoiseDescriptor,
    NoiseGradientSurface, OverworldSurface, ReplaceBiomes, SubsurfaceBuilder, SurfaceBuilder,
    SwampSurface, Tags, TopSlide,
};
use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, JsonDefinition,
    JsonDocument, JsonPlaced, NoBeta, Origin, Placed, Registry, RegistryBuilder, Rejected,
    default_path_flat_for_vanilla,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle, write_json};
use bedrock_addon_core::json::{PrepareError, Prepared};
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, StrictString};
use bedrock_addon_core::load::{self, CollectionLoader, LoadContext, StackCx, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::shared::BlockSpecifier;
use bedrock_addon_core::version::known;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, SemVersion};
use bedrock_addon_core::versioning::{
    BETA_VERSION, BelowFloor, ComponentSection, ComponentTable, DocumentRead, Envelope,
    EnvelopeCheck, Family, Identity, InputKind, Loaded, MissingVersion, OpenBeta, Parsed,
    ValidateCx, VersionStruct, finish, member_version, parse_as_written, write_envelope,
};

/// A server biome (`minecraft:biome`), at the newest format version.
///
/// JSON: `{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier"},
/// "components": {…}}}`. A document of an older version is upgraded to this shape when it
/// loads; the components a vanilla biome needs are inserted for overrides of vanilla biomes
/// older than 1.26.0.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:biome")]
pub struct Biome {
    /// The biome's name (JSON `description`).
    pub description: Description,
    /// The components (JSON `components`, may be empty).
    pub components: ComponentMap<BiomeComponent>,
    /// Unknown members of `minecraft:biome`.
    pub extra: Extra,
}

/// `minecraft:biome/description`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct Description {
    /// The biome's identifier, `namespace:name` in lower case (JSON `identifier`, matching
    /// `^[a-z0-9._%+-]+:[a-z0-9._%+-]+$`).
    pub identifier: BiomeId,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A beta biome (`"format_version": "beta"`): the newest components plus
/// `environment_attributes`. Loaded only from a base-game pack or with the Beta APIs toggle,
/// and never upgraded.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:biome")]
pub struct Beta {
    /// The biome's name (JSON `description`).
    pub description: Description,
    /// The components (JSON `components`, may be empty).
    pub components: ComponentMap<BiomeComponent>,
    /// Experimental environment attributes (JSON `environment_attributes`).
    pub environment_attributes: Option<EnvironmentAttributes>,
    /// Unknown members of `minecraft:biome`.
    pub extra: Extra,
}

/// The directory biome files live in.
const DIRECTORY: &str = "biomes";
/// The floor: the first format vanilla loads; every earlier format was experimental.
const FLOOR: FormatVersion = known::V1_21_110;
/// The world version a non-base-game pack's biomes need, as the world-gate message says.
const WORLD_GATE_VERSION: SemVersion = SemVersion::new(1, 21, 110);
/// The first experimental format the below-floor hint names.
const EXPERIMENTAL_FORMATS: FormatVersion = FormatVersion::new(1, 13, 0);

impl Family for BiomeFamily {
    const NAME: &'static str = "biome";
    const PAYLOAD_KEY: &'static str = "minecraft:biome";
    const FLOOR: FormatVersion = FLOOR;
    const ENVELOPE: Envelope = Envelope {
        // The game reads every file as a document: the version is the first textual
        // `format_version` (a number is read as its text) and the size limits apply.
        input: InputKind::Document,
        // What the document does not read is kept for write-back: a number no double holds is
        // the largest finite one, with a warning at it.
        clamps_beyond_double: true,
        // Below 1.21.120 the game reads the raw text of the whole document; from 1.21.120 the
        // envelope has exactly two members, `format_version` and `minecraft:biome`.
        check: EnvelopeCheck::From(CEREAL),
        // A file without a `format_version` is rejected: "no valid version passed by the caller or
        // found in the input".
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };
    type Latest = Biome;
    type Beta = Beta;
    type Component = BiomeComponent;
    type Any = Any;

    fn components() -> &'static ComponentTable<Self> {
        BiomeComponent::TABLE
    }

    /// The 1.12 shape (`{ "<name>": { "format_version": …, … } }`) holds its version inside
    /// its only root member; it is read so the document is rejected as below the floor.
    // The game reports "no parser available for version 1.12.0" for such a file.
    fn payload_version(payload: JsonSlice<'_>) -> Option<FormatVersionInput> {
        let only = only_member(payload)?;
        member_version(only, "format_version")
    }

    fn component_sections(v: &mut Biome) -> Vec<ComponentSection<'_, BiomeComponent>> {
        let at = JsonPath::of_keys(["components"]);
        vec![ComponentSection::new(at, false, &mut v.components)]
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        match v {
            Loaded::Stable(biome) => {
                validate::validate(&biome.description.identifier, &mut biome.components, cx);
            }
            Loaded::Beta(beta) => {
                validate::validate(&beta.description.identifier, &mut beta.components, cx);
            }
        }
    }
}

/// The only member of a root without `minecraft:biome`, as a slice.
fn only_member(document: JsonSlice<'_>) -> Option<JsonSlice<'_>> {
    let members =
        bedrock_addon_core::json::from_str_borrowed_jsonc::<IndexMap<String, RawJson<'_>>>(
            document.text,
        )
        .ok()?;
    if members.len() != 1 || members.contains_key("minecraft:biome") {
        return None;
    }
    let (_, raw) = members.first()?;
    document.nested(raw)
}

impl JsonDocument for Biome {
    type Family = BiomeFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// One biome file through the chain (`load`): the world gate and the text checks first,
    /// the payload's path and the below-floor hint between the parse and the upgrade. A beta
    /// document is the [`Beta`] (the stable type's own `parse` refuses it).
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<BiomeFamily> {
        match load(origin, bytes, cx) {
            (_, Ok((Loaded::Stable(biome), parts))) => DocumentRead::Stable(parts.with(biome)),
            (_, Ok((Loaded::Beta(beta), parts))) => {
                let identifier = beta.description.identifier.as_str().to_owned();
                DocumentRead::Beta(Some(identifier), OpenBeta::new(parts.with(beta)))
            }
            (identifier, Err(rejected)) => DocumentRead::Rejected(identifier, rejected),
        }
    }

    /// Parses one biome file without upgrading it: the struct its `format_version` selects
    /// with the findings of the parse.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file is not text, is not JSON, or its findings include an
    /// error (the findings are in the [`Rejected`]).
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        let origin = origin(PackRef::app_data(), PackSide::Behavior, path);
        let (_, parsed) = parse(&origin, bytes, cx)?;
        let mut diagnostics = parsed.diagnostics;
        diagnostics.locate(&origin);
        match parsed.value {
            Some(value) => Ok(Document::new(value, origin, parsed.version, diagnostics)),
            None => Err(Rejected::new(origin, diagnostics)),
        }
    }
}

impl JsonPlaced for Biome {
    /// `biomes/<name>.json` for a `minecraft:` identifier (the `/` of the name as `_`), which
    /// a base-game pack's flat walk reads back; `biomes/<namespace>/<name>.json` for any
    /// other.
    fn place(&self) -> PackPath {
        default_path_flat_for_vanilla(DIRECTORY, &self.description.identifier)
    }
}

impl JsonDefinition for Biome {
    type Kind = kind::Biome;

    fn identifier(&self) -> &BiomeId {
        &self.description.identifier
    }
}

// `Beta` keeps a hand quartet: its family's `Latest` is `Biome`, so it is no `JsonDocument`.
impl Sealed for Beta {}

impl Content for Beta {
    const NAME: &'static str = "beta biome";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Parses one beta biome file (`"format_version": "beta"`); any other version is rejected.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = origin(PackRef::app_data(), PackSide::Behavior, path);
        match load(origin, bytes, cx).1? {
            (Loaded::Beta(beta), document) => Ok(document.with(beta)),
            (Loaded::Stable(_), document) => {
                let Parts {
                    origin,
                    mut diagnostics,
                    ..
                } = document;
                diagnostics.push(Diagnostic::error(DiagnosticCode::UnsupportedVersion {
                    text: None,
                }));
                Err(Rejected::located(origin, diagnostics))
            }
        }
    }

    /// The document, at `"format_version": "beta"`.
    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        let root = write_envelope::<BiomeFamily>(
            BETA_VERSION,
            VersionStruct::write(self)?,
            &Extra::default(),
        );
        write_json(&root, out, style)
    }
}

impl JsonContent for Beta {}

impl Definition for Beta {
    type Kind = kind::Biome;
    type File = Beta;
    type Beta = NoBeta<kind::Biome>;
}

impl Placed for Beta {
    /// `biomes/<name>.json` for a `minecraft:` identifier (the `/` of the name as `_`), which
    /// a base-game pack's flat walk reads back; `biomes/<namespace>/<name>.json` for any
    /// other.
    fn default_path(&self) -> PackPath {
        default_path_flat_for_vanilla(DIRECTORY, &self.description.identifier)
    }
}

/// `history::upgrade`: rebuilds the parse stage from the document and finishes it.
pub(crate) fn upgrade_document(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<Biome>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let found = document.diagnostics().clone();
    let value = document.into_value();
    let identity = Identity {
        identifier: Some(identifier_of(&value).to_owned()),
        runtime_identifier: None,
    };
    let at = child(&JsonPath::new(), &["minecraft:biome"]);
    let written = FormatVersionInput {
        version: (!version.beta).then_some(version.written.unwrap_or(version.effective)),
        beta_literal: version.beta,
        prerelease: false,
    };
    let parsed = Parsed::<BiomeFamily>::from_value(
        Some(value),
        version,
        written,
        at,
        Diagnostics::default(),
    )
    .with_identity(identity);
    let outcome = finish(parsed, cx);
    let mut diagnostics = found;
    let mut more = outcome.diagnostics;
    more.locate(&origin);
    diagnostics.extend(more);
    match outcome.value {
        Some(Loaded::Stable(biome)) => {
            Ok(Document::new(biome, origin, outcome.version, diagnostics))
        }
        Some(Loaded::Beta(_)) => {
            let finding = Diagnostic::error(DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion {
                    version: "beta".to_owned(),
                }),
            })
            .in_file(&origin);
            diagnostics.push(finding);
            Err(Rejected::new(origin, diagnostics))
        }
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The identifier of a parsed document, as written.
fn identifier_of(value: &Any) -> &str {
    match value {
        Any::V1_21_110(b) => b.description.identifier.as_str(),
        Any::V1_21_120(b) => b.description.identifier.as_str(),
        Any::V1_26_0(b) => b.description.identifier.as_str(),
        Any::V1_26_30(b) => b.description.identifier.as_str(),
        Any::Beta(b) => b.description.identifier.as_str(),
    }
}

/// A loaded document's parts, before the value goes into a [`Document`].
struct Parts {
    origin: Origin,
    version: bedrock_addon_core::document::VersionInfo,
    diagnostics: Diagnostics,
}

impl Parts {
    fn with<T>(self, value: T) -> Document<T> {
        Document::new(value, self.origin, self.version, self.diagnostics)
    }
}

/// The text of a file, after the BOM, the UTF-8 and the depth check.
fn text<'b>(bytes: &'b [u8], origin: &Origin) -> Result<Cow<'b, str>, Rejected> {
    let malformed = |message: &str| {
        reject(
            origin.clone(),
            Diagnostic::error(DiagnosticCode::MalformedInput {
                message: message.to_owned(),
                text: None,
            }),
        )
    };
    // `new` never refuses an empty text: that is the chain's to report.
    let text = Prepared::new(bytes)
        .map_err(|error| match error {
            PrepareError::NotUtf8(_) => malformed("the file is not UTF-8"),
            _ => malformed("the JSON nests too deeply"),
        })?
        .into_text();
    Ok(text)
}

/// The world gate: outside a base-game pack, biomes need a world of base-game version
/// 1.21.110 or later. A context without the world's version passes.
fn world_gate(cx: &LoadContext) -> Option<Diagnostic> {
    (!cx.is_base_game_pack && !cx.world_base_game_version.includes(&WORLD_GATE_VERSION)).then(
        || {
            Diagnostic::error(DiagnosticCode::IgnoredFile {
                text: fixed(WORLD_GATE),
            })
        },
    )
}

/// The parse stage of one file: the world gate, then the chain's `parse_as_written`, with the
/// below-floor findings in vanilla's words. Also the identifier the payload names.
fn parse(
    origin: &Origin,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<(Option<String>, Parsed<BiomeFamily>), Rejected> {
    let text = text(bytes, origin)?;
    if let Some(gate) = world_gate(cx) {
        return Err(reject(origin.clone(), gate));
    }
    let mut parsed = parse_as_written::<BiomeFamily>(JsonSlice::new(&text), cx);
    // The payload is under `minecraft:biome` on the raw-text path too: the checks after the
    // parse report below it.
    parsed.at = child(&JsonPath::new(), &["minecraft:biome"]);
    floor_hint(&mut parsed, &text);
    let identifier = parsed.identity.identifier.clone();
    Ok((identifier, parsed))
}

/// One file through the chain: the identifier it names, and the loaded value with its parts
/// or the rejection.
#[allow(clippy::type_complexity)]
fn load(
    origin: Origin,
    bytes: &[u8],
    cx: &LoadContext,
) -> (
    Option<String>,
    Result<(Loaded<BiomeFamily>, Parts), Rejected>,
) {
    let (identifier, parsed) = match parse(&origin, bytes, cx) {
        Ok(parsed) => parsed,
        Err(rejected) => return (None, Err(rejected)),
    };
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    let result = match outcome.value {
        Some(value) => Ok((
            value,
            Parts {
                origin,
                version: outcome.version,
                diagnostics,
            },
        )),
        None => Err(Rejected::new(origin, diagnostics)),
    };
    (identifier, result)
}

/// Below the floor vanilla has no parser ("unsupported version: no parser available for
/// version %s", `ENVELOPE.below_floor`): the crate's own hint for the old formats follows that
/// finding.
fn floor_hint(parsed: &mut Parsed<BiomeFamily>, text: &str) {
    let Some(written) = parsed.version.written.filter(|v| *v < FLOOR) else {
        return;
    };
    let mut found = Diagnostics::default();
    for diagnostic in std::mem::take(&mut parsed.diagnostics) {
        let floor = matches!(
            diagnostic.code,
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion { .. })
            }
        );
        found.push(diagnostic);
        if floor && let Some(hint) = hint(text, written) {
            found.push(Diagnostic::info(hint));
        }
    }
    parsed.diagnostics = found;
}

/// The crate's own hint for a document of an old format: the 1.12 name-keyed shape, or an
/// experimental format from 1.13.0 (with the surface sentence when the 1.20.60-era surface
/// components are used).
fn hint(text: &str, version: FormatVersion) -> Option<DiagnosticCode> {
    let root = bedrock_addon_core::json::from_str_jsonc::<Value>(
        &bedrock_addon_core::json::last_wins(text),
    )
    .ok()?;
    let root = root.as_object()?;
    let biome = root.get("minecraft:biome");
    if biome.is_none() && root.len() == 1 {
        let only = root.iter().next().map(|(_, v)| v)?;
        if only
            .as_object()
            .is_some_and(|o| o.get("format_version").is_some())
        {
            return Some(DiagnosticCode::BiomeFormatHint {
                version: None,
                surface_components: false,
            });
        }
    }
    if version < EXPERIMENTAL_FORMATS {
        return None;
    }
    // `minecraft:surface_parameters` and `minecraft:*_surface`.
    let surface_components = biome
        .and_then(Value::as_object)
        .and_then(|b| b.get("components"))
        .and_then(Value::as_object)
        .is_some_and(|components| {
            components.iter().any(|(key, _)| {
                let key = key.as_str();
                key == "minecraft:surface_parameters"
                    || (key.starts_with("minecraft:") && key.ends_with("_surface"))
            })
        });
    Some(DiagnosticCode::BiomeFormatHint {
        version: Some(version),
        surface_components,
    })
}

/// A rejection with one finding.
fn reject(origin: Origin, finding: Diagnostic) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding);
    Rejected::located(origin, diagnostics)
}

/// The origin of a file of `pack`.
fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    Origin::file(pack, side, path.clone())
}

// ---- the collection ----------------------------------------------------------------------

/// A biome file: a `.json` file below `biomes/`.
#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "vanilla's directory walk matches `.json` as written"
)]
/// Whether vanilla's walk reaches `path` in the pack of `cx`: the game lists `biomes/` of a
/// pack shipped in the app package without descending into sub-folders, and every other pack
/// with its sub-folders.
fn reached(path: &PackPath, cx: &LoadContext) -> bool {
    let nested = path.segments().count() > 2;
    !(cx.is_base_game_pack && nested)
}

/// `biomes/**/*.json`, packs bottom-up; `biomes/*.json` in a base-game pack.
const WALK: Walk = Walk::json_tree_in(&[DIRECTORY]).in_pack(reached);

/// The registry. Vanilla keeps the first document of an identifier and drops later ones;
/// with the stack walked top first that is the highest-priority pack's, so the rule is
/// `TopWins`. A dropped document from a pack other than a base-game pack is vanilla's error
/// "Multiple biome documents with same identifier: %s".
fn combine(parsed: Vec<Contribution<Biome>>, stack: &StackCx<'_>) -> Registry<Biome> {
    let mut duplicates = Vec::new();
    {
        // The highest priority (lowest rank) loaded per identifier, stable and beta alike.
        let mut top: IndexMap<&BiomeId, usize> = IndexMap::new();
        let loaded = parsed.iter().filter_map(|c| match c {
            Contribution::Stable(id, d) => Some((id, d.origin())),
            Contribution::Beta(id, d) => Some((id, d.origin())),
            Contribution::Rejected(..) => None,
        });
        let loaded: Vec<_> = loaded.collect();
        for (id, origin) in &loaded {
            let rank = rank(&origin.pack);
            top.entry(id)
                .and_modify(|r| *r = (*r).min(rank))
                .or_insert(rank);
        }
        for (id, origin) in &loaded {
            let dropped = top.get(id).is_some_and(|&r| rank(&origin.pack) > r);
            if dropped && !is_base_game(stack, &origin.pack) {
                duplicates.push(
                    Diagnostic::error(DiagnosticCode::DuplicateIdentifier {
                        text: formatted(DUPLICATE, &[id.as_str()]),
                    })
                    .at(child(
                        &JsonPath::new(),
                        &["minecraft:biome", "description", "identifier"],
                    ))
                    .phase(Phase::Validate)
                    .in_file(origin),
                );
            }
        }
    }
    let mut builder = RegistryBuilder::new(
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        stack.options,
    );
    for contribution in parsed {
        builder.push(contribution);
    }
    for finding in duplicates {
        builder.push_diagnostic(finding);
    }
    builder.finish()
}

/// The priority rank of a pack: 0 is the top; the app-data layer is below every pack.
fn rank(pack: &PackRef) -> usize {
    pack.index().unwrap_or(usize::MAX)
}

/// Whether `pack` is a base-game pack of the stack.
fn is_base_game(stack: &StackCx<'_>, pack: &PackRef) -> bool {
    pack.index()
        .and_then(|i| stack.packs.get(i))
        .is_some_and(|p| p.context.is_base_game_pack)
}

/// The loader of `bp.biomes()`: `biomes/**/*.json` of every behaviour pack, combined top pack
/// wins.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<Biome>, Contribution<Biome>> = CollectionLoader {
    combine,
    ..load::registry::<Biome>(WALK)
};

impl Description {
    /// The description of `identifier`.
    pub fn new(identifier: BiomeId) -> Self {
        Self {
            identifier,
            extra: Extra::default(),
        }
    }
}

impl Biome {
    /// A biome named `identifier` without components.
    pub fn new(identifier: BiomeId) -> Self {
        Self {
            description: Description::new(identifier),
            components: ComponentMap::new(),
            extra: Extra::default(),
        }
    }
}

impl Beta {
    /// A beta biome named `identifier` without components or environment attributes.
    pub fn new(identifier: BiomeId) -> Self {
        Self {
            description: Description::new(identifier),
            components: ComponentMap::new(),
            environment_attributes: None,
            extra: Extra::default(),
        }
    }
}

// ---- components and members with required members ------------------------

impl Humidity {
    /// `minecraft:humidity` with `is_humid`.
    pub fn new(is_humid: bool) -> Self {
        Self {
            is_humid: StrictBool(is_humid),
            extra: Extra::default(),
        }
    }
}

impl Tags {
    /// `minecraft:tags` with `tags`, in order.
    pub fn new<S: Into<String>>(tags: impl IntoIterator<Item = S>) -> Self {
        Self {
            tags: tags.into_iter().map(|t| StrictString(t.into())).collect(),
            extra: Extra::default(),
        }
    }
}

impl TopSlide {
    /// `top_slide` with `enabled`.
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled: StrictBool(enabled),
            extra: Extra::default(),
        }
    }
}

impl ReplaceBiomes {
    /// `minecraft:replace_biomes` with `replacements`.
    pub fn new(replacements: Vec<BiomeReplacement>) -> Self {
        Self {
            replacements,
            extra: Extra::default(),
        }
    }
}

impl BiomeReplacement {
    /// A replacement of `targets` in `dimension`, by `amount` with noise of
    /// `noise_frequency_scale`.
    pub fn new<S: Into<String>>(
        dimension: impl Into<String>,
        targets: impl IntoIterator<Item = S>,
        amount: f32,
        noise_frequency_scale: f32,
    ) -> Self {
        Self {
            dimension: StrictString(dimension.into()),
            targets: targets
                .into_iter()
                .map(|t| StrictString(t.into()))
                .collect(),
            amount: StrictFloat(amount),
            noise_frequency_scale: StrictFloat(noise_frequency_scale),
            extra: Extra::default(),
        }
    }
}

impl MapTints {
    /// `minecraft:map_tints` with `grass` and no foliage colour.
    pub fn new(grass: GrassTint) -> Self {
        Self {
            grass,
            foliage: None,
            extra: Extra::default(),
        }
    }
}

impl SurfaceBuilder {
    /// `minecraft:surface_builder` with `builder`.
    pub fn new(builder: Builder) -> Self {
        Self {
            builder,
            extra: Extra::default(),
        }
    }
}

impl SubsurfaceBuilder {
    /// `minecraft:subsurface_builder` with `builder`.
    pub fn new(builder: Builder) -> Self {
        Self {
            builder,
            extra: Extra::default(),
        }
    }
}

impl OverworldSurface {
    /// The layered surface: sea floor depth, then the sea floor, foundation, middle, top and
    /// sea blocks.
    pub fn new(
        sea_floor_depth: i32,
        sea_floor_material: BlockSpecifier,
        foundation_material: BlockSpecifier,
        mid_material: BlockSpecifier,
        top_material: BlockSpecifier,
        sea_material: BlockSpecifier,
    ) -> Self {
        Self {
            sea_floor_depth: StrictInt(sea_floor_depth),
            sea_floor_material,
            foundation_material,
            mid_material,
            top_material,
            sea_material,
            extra: Extra::default(),
        }
    }
}

impl SwampSurface {
    /// The swamp surface: `surface`'s members and the puddle depth below sea level.
    pub fn new(surface: OverworldSurface, max_puddle_depth_below_sea_level: i32) -> Self {
        Self {
            sea_floor_depth: surface.sea_floor_depth,
            sea_floor_material: surface.sea_floor_material,
            foundation_material: surface.foundation_material,
            mid_material: surface.mid_material,
            top_material: surface.top_material,
            sea_material: surface.sea_material,
            max_puddle_depth_below_sea_level: StrictInt(max_puddle_depth_below_sea_level),
            extra: surface.extra,
        }
    }
}

impl CappedSurface {
    /// The capped surface: floor and ceiling blocks, the sea and foundation blocks, no beach.
    pub fn new(
        floor_materials: Vec<BlockSpecifier>,
        ceiling_materials: Vec<BlockSpecifier>,
        sea_material: BlockSpecifier,
        foundation_material: BlockSpecifier,
    ) -> Self {
        Self {
            floor_materials,
            ceiling_materials,
            sea_material,
            foundation_material,
            beach_material: None,
            extra: Extra::default(),
        }
    }
}

impl MesaSurface {
    /// The mesa surface: `surface`'s members, the clay and hardened clay, and whether bryce
    /// pillars and a forest generate.
    pub fn new(
        surface: OverworldSurface,
        clay_material: BlockSpecifier,
        hard_clay_material: BlockSpecifier,
        bryce_pillars: bool,
        has_forest: bool,
    ) -> Self {
        Self {
            sea_floor_depth: surface.sea_floor_depth,
            sea_floor_material: surface.sea_floor_material,
            foundation_material: surface.foundation_material,
            mid_material: surface.mid_material,
            top_material: surface.top_material,
            sea_material: surface.sea_material,
            clay_material,
            hard_clay_material,
            bryce_pillars: StrictBool(bryce_pillars),
            has_forest: StrictBool(has_forest),
            extra: surface.extra,
        }
    }
}

impl NoiseGradientSurface {
    /// The noise gradient of `noise_descriptor` over `noise_block_specifiers`, replacing every
    /// block.
    pub fn new(
        noise_descriptor: NoiseDescriptor,
        noise_block_specifiers: Vec<NoiseBlockSpecifier>,
    ) -> Self {
        Self {
            noise_descriptor,
            non_replaceable_blocks: None,
            noise_block_specifiers,
            extra: Extra::default(),
        }
    }
}

impl NoiseDescriptor {
    /// The noise `name` from `first_octave` with one amplitude per octave.
    pub fn new(name: impl Into<String>, first_octave: i32, amplitudes: Vec<f32>) -> Self {
        Self {
            name: StrictString(name.into()),
            first_octave: StrictInt(first_octave),
            amplitudes: amplitudes.into_iter().map(StrictFloat).collect(),
            extra: Extra::default(),
        }
    }
}

impl NoiseBlockSpecifier {
    /// `block` with no noise, threshold or range written (they read as `""`, `0`, `{0, 0}`).
    pub fn new(block: BlockSpecifier) -> Self {
        Self {
            noise: None,
            threshold: None,
            range: None,
            block,
            extra: Extra::default(),
        }
    }
}

impl Adjustment {
    /// An adjustment setting `materials`, everywhere (no range, no scale).
    pub fn new(materials: AdjustmentMaterials) -> Self {
        Self {
            materials,
            noise_range: None,
            height_range: None,
            noise_frequency_scale: None,
            extra: Extra::default(),
        }
    }
}

impl BoolAttribute {
    /// `value`, combined by `operation`.
    pub fn new(value: bool, operation: BoolOperation) -> Self {
        Self {
            value: StrictBool(value),
            operation,
            extra: Extra::default(),
        }
    }
}

impl ColorAttribute {
    /// `value`, combined by `operation`.
    pub fn new(value: ColorRgba, operation: ColorOperation) -> Self {
        Self {
            value,
            operation,
            extra: Extra::default(),
        }
    }
}

impl FloatAttribute {
    /// `value`, combined by `operation`.
    pub fn new(value: f32, operation: FloatOperation) -> Self {
        Self {
            operation,
            value: StrictFloat(value),
            extra: Extra::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{
        AdjustmentMaterials, BiomeComponent, Builder, CappedSurface, GrassTint, MesaSurface,
        NoiseBlockSpecifier, NoiseDescriptor, NoiseGradientSurface, OverworldSurface,
        ReplaceBiomes, SwampSurface, TheEndSurface,
    };
    use bedrock_addon_core::document::Extra;
    use bedrock_addon_core::id::BlockId;
    use bedrock_addon_core::json::JsonSlice;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::versioning::ConvertMode;

    /// Writes `component` and reads it back with its row parser: the same value, no finding.
    #[track_caller]
    fn reads_back(name: &str, component: BiomeComponent) {
        let json = crate::component::write_component(&component).expect("writes");
        let text = bedrock_addon_core::json::to_facet_json(&json).expect("text");
        let row = BiomeComponent::TABLE.get(name).expect("a row");
        let mut found = bedrock_addon_core::diagnostic::Diagnostics::default();
        let read = (row.parse)(
            JsonSlice::new(&text),
            ConvertMode::Strict,
            &LoadContext::detached(),
            &mut found,
        );
        assert!(found.is_empty(), "{name}: {found:#?}");
        assert_eq!(read, Some(component), "{name}");
    }

    fn block(name: &'static str) -> BlockSpecifier {
        BlockSpecifier::new(BlockId::from_static(name))
    }

    fn overworld() -> OverworldSurface {
        OverworldSurface::new(
            7,
            block("minecraft:gravel"),
            block("minecraft:stone"),
            block("minecraft:dirt"),
            block("minecraft:grass_block"),
            block("minecraft:water"),
        )
    }

    #[test]
    fn every_constructor_builds_a_component_that_reads_back() {
        reads_back("minecraft:humidity", Humidity::new(true).into());
        reads_back("minecraft:tags", Tags::new(["animal", "plains"]).into());
        reads_back(
            "minecraft:mountain_parameters",
            crate::component::MountainParameters {
                top_slide: Some(TopSlide::new(true)),
                ..Default::default()
            }
            .into(),
        );
        reads_back(
            "minecraft:replace_biomes",
            ReplaceBiomes::new(vec![BiomeReplacement::new(
                "minecraft:overworld",
                ["plains"],
                0.5,
                1.0,
            )])
            .into(),
        );
        reads_back(
            "minecraft:map_tints",
            MapTints::new(GrassTint::Noise {
                extra: Extra::default(),
            })
            .into(),
        );
        for builder in [
            Builder::Overworld(overworld()),
            Builder::Swamp(SwampSurface::new(overworld(), 3)),
            Builder::Capped(CappedSurface::new(
                vec![block("minecraft:netherrack")],
                vec![block("minecraft:netherrack")],
                block("minecraft:lava"),
                block("minecraft:netherrack"),
            )),
            Builder::TheEnd(TheEndSurface::default()),
            Builder::Mesa(MesaSurface::new(
                overworld(),
                block("minecraft:white_terracotta"),
                block("minecraft:hardened_clay"),
                false,
                true,
            )),
            Builder::NoiseGradient(NoiseGradientSurface::new(
                NoiseDescriptor::new("test:noise", -5, vec![1.0, 0.0]),
                vec![NoiseBlockSpecifier::new(block("minecraft:stone"))],
            )),
        ] {
            reads_back(
                "minecraft:surface_builder",
                SurfaceBuilder::new(builder.clone()).into(),
            );
            reads_back(
                "minecraft:subsurface_builder",
                SubsurfaceBuilder::new(builder).into(),
            );
        }
        reads_back(
            "minecraft:surface_material_adjustments",
            crate::component::SurfaceMaterialAdjustments {
                adjustments: Some(vec![Adjustment::new(AdjustmentMaterials {
                    top_material: Some(block("minecraft:sand")),
                    ..Default::default()
                })]),
                ..Default::default()
            }
            .into(),
        );
    }

    #[test]
    fn the_attribute_constructors_read_back() {
        let attributes = crate::EnvironmentAttributes {
            experimental_bool_attr: Some(BoolAttribute::new(true, BoolOperation::Xor)),
            experimental_color_attr: Some(ColorAttribute::new(
                ColorRgba::Rgba([1, 2, 3, 4]),
                ColorOperation::Multiply,
            )),
            experimental_float_attr_1: Some(FloatAttribute::new(0.5, FloatOperation::Maximum)),
            ..Default::default()
        };
        let text = bedrock_addon_core::json::to_json(&attributes).expect("writes");
        let read: crate::EnvironmentAttributes =
            bedrock_addon_core::json::from_str(&text).expect("reads");
        assert_eq!(read, attributes);
    }
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn biome_reads_as_pinned() {
        assert_reading_table(
            "BiomeFamily",
            &reading_table::<super::BiomeFamily>(),
            r"
                text: Document, clamps beyond a double: true, duplicates kept in: []
                envelope: From(FormatVersion { major: 1, minor: 21, patch: 120 })
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.110..=1.26.30 (0..=3): json Rejected/Strict/Error/fatal/Refuse, mcb Own/Strict/Error/fatal/Refuse
                beta: json Rejected/Strict/Error/fatal/Refuse, mcb Own/Strict/Error/fatal/Refuse
            ",
        );
    }
}
