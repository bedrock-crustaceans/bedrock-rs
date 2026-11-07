//! The deferred renderer's settings ("Vibrant Visuals"): the eight document kinds a resource
//! pack configures it with — lighting (`lighting/`), atmosphere (`atmospherics/`), water
//! (`water/`), colour grading (`color_grading/`) and cubemaps (`cubemaps/`), each named by an
//! identifier client biomes refer to; the shadow style (`shadows/global.json`) and the PBR
//! fallback values (`pbr/global.json`), one file each; and the local lights
//! (`local_lighting/local_lighting.json`), merged per block — loaded together as the
//! `DeferredSettings` group.
//!
//! The reader stays a hand walk over each payload's slices (a node and members of its own), with
//! the game's texts per value form and every finding at its own value's place; moving it onto the
//! reader kit is more than a swap of helpers.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
pub mod history;
pub use atmosphere::{AtmosphereSettings, HorizonBlendStops};
pub use color_grading::{
    ColorGrading, ColorGradingDescription, ColorGradingSettings, Highlights, Midtones, Shadows,
    Temperature, TemperatureType, ToneMapping, ToneMappingOperator,
};
pub use cubemap::{CubemapLighting, CubemapSettings};
pub use lighting::{
    AmbientLight, DirectionalLights, Emissive, Flash, LightingSettings, OrbitalLight,
    OrbitalLights, Sky,
};
pub use local_light::{LightType, LocalLight, LocalLightSettings};
pub use pbr::{Mer, Mers, PbrFallback, PbrFallbackSettings, PbrValue};
pub use shadows::{ShadowSettings, ShadowStyle};
pub use value::{Description, Keyframe, Keyframes, Rgb8, Rgba8, Vec3};
pub use water::{Caustics, DEFAULT_CAUSTICS_TEXTURE, ParticleConcentrations, WaterSettings, Waves};

/// The accessors of members that have a value of the game's own when the document leaves them
/// out: `field()` gives the member, or that value.
macro_rules! defaults {
    ($ty:ty { $($field:ident: $t:ty = $default:expr;)* }) => {
        impl $ty {
            $(
                #[doc = concat!(
                    "[`", stringify!($field), "`](Self::", stringify!($field),
                    "), or the value the game uses when the document leaves it out."
                )]
                #[must_use]
                pub fn $field(&self) -> $t {
                    self.$field.clone().unwrap_or_else(|| $default)
                }
            )*
        }
    };
}
pub(crate) use defaults;

mod atmosphere;
mod color_grading;
mod cubemap;
mod file;
mod lighting;
mod link;
mod local_light;

mod pbr;
mod read;
mod shadows;
mod value;
mod water;
mod write;

#[cfg(test)]
mod tests;

// The checker of the references the link pass resolves, for its rules.
#[doc(hidden)]
pub use link::{Targets, Texture, check as check_links};

use std::collections::HashSet;
use std::convert::Infallible;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::unversioned::origin_of;
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, Files, FilesBuilder, FilesRule,
    JsonDocument, JsonPlaced, Merge, Merged, NoBeta, Notice, Origin, Registry, RegistryBuilder,
    Rejected, default_path,
};
use bedrock_addon_core::id::{
    AtmosphereSettingsId, ColorGradingSettingsId, LightingSettingsId, WaterSettingsId,
};
use bedrock_addon_core::id::{BlockId, CubemapSettingsId, Id, Kind, kind};
use bedrock_addon_core::load::Reported;
use bedrock_addon_core::load::{
    Accept, CollectionLoader, FileCopy, LoadContext, StackCx, Walk, WalkFile,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{
    BelowFloor, DocumentRead, Envelope, Family, Loaded, MissingVersion, NoComponents, RangeModel,
    ValidateCx, Versioned,
};

/// The deferred renderer's settings of the resource packs (`rp.deferred_settings()`): eight
/// collections loaded and used together.
///
/// The game reads every one of them on every resource reload, whatever the graphics mode, the
/// pack's capabilities and the experiments.
///
/// - Lighting, atmosphere, water, colour grading and cubemap settings are keyed by their
///   `description.identifier`; per identifier the highest-priority pack's definition wins,
///   whole. Within one pack the first file of an identifier (by path) stays and a later one is
///   an error. A file that fails to load does not hide a lower pack's file of the same
///   identifier.
/// - The shadow style and the PBR fallback values are one file each: the highest-priority pack
///   whose file loads decides.
/// - The local lights of every pack are merged: per block the highest-priority pack's entry
///   wins.
///
/// A biome that names no settings of a kind uses the kind's default: see
/// [`DefaultSettings`].
#[derive(Debug)]
#[non_exhaustive]
#[doc(alias = "render_configs")]
#[doc(alias = "Vibrant Visuals")]
pub struct DeferredSettings {
    /// The lighting settings (`lighting/*.json`).
    pub lighting: Registry<LightingSettings>,
    /// The atmosphere settings (`atmospherics/*.json`).
    pub atmosphere: Registry<AtmosphereSettings>,
    /// The water settings (`water/*.json`).
    pub water: Registry<WaterSettings>,
    /// The colour-grading settings (`color_grading/*.json`).
    pub color_grading: Registry<ColorGradingSettings>,
    /// The cubemap settings (`cubemaps/*.json`).
    pub cubemaps: Registry<CubemapSettings>,
    /// The shadow style (`shadows/global.json`); see [`DeferredSettings::shadow_settings`].
    pub shadows: Files<ShadowSettings>,
    /// The PBR fallback values (`pbr/global.json`); see
    /// [`DeferredSettings::pbr_fallback_settings`].
    pub pbr_fallback: Files<PbrFallbackSettings>,
    /// The local lights of every pack's `local_lighting/local_lighting.json`, merged per block.
    pub local_lighting: Merged<LocalLightSettings>,
    /// The identifier each kind of settings uses for a biome that names none.
    pub defaults: DefaultSettings,
    /// The findings about the group as a whole.
    diagnostics: Diagnostics,
}

/// The settings a biome uses when it names none of a kind.
///
/// For lighting, atmosphere, water and colour grading it is the identifier of the kind's
/// default file (`lighting/global.json`, `atmospherics/atmospherics.json`, `water/water.json`,
/// `color_grading/color_grading.json`) in the highest-priority pack whose copy loads and has an
/// identifier; `None` when no pack has one, and the game then uses its own values. For
/// cubemaps the identifier is fixed.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct DefaultSettings {
    /// The default lighting settings.
    pub lighting: Option<LightingSettingsId>,
    /// The default atmosphere settings.
    pub atmosphere: Option<AtmosphereSettingsId>,
    /// The default water settings.
    pub water: Option<WaterSettingsId>,
    /// The default colour-grading settings.
    pub color_grading: Option<ColorGradingSettingsId>,
    /// The default cubemap settings: always `minecraft:default_cubemap`, whether or not a pack
    /// defines them.
    pub cubemap: CubemapSettingsId,
}

impl DeferredSettings {
    /// The path of the one shadow-style file.
    pub const SHADOWS_PATH: &'static str = "shadows/global.json";
    /// The path of the one PBR-fallback file.
    pub const PBR_FALLBACK_PATH: &'static str = "pbr/global.json";
    /// The path of a pack's local-light file.
    pub const LOCAL_LIGHTING_PATH: &'static str = "local_lighting/local_lighting.json";

    /// The shadow style in effect: the highest-priority `shadows/global.json` that loads;
    /// `None` when no pack has one (the game then draws soft shadows with texel size 16).
    #[must_use]
    pub fn shadow_settings(&self) -> Option<&Document<ShadowSettings>> {
        self.shadows
            .get(&bedrock_addon_core::pack_identity::PackPath::from_static(
                Self::SHADOWS_PATH,
            ))
    }

    /// The PBR fallback values in effect: the highest-priority `pbr/global.json` that loads;
    /// `None` when no pack has one (the game then uses [`Mers::DEFAULT`] for every kind).
    #[must_use]
    pub fn pbr_fallback_settings(&self) -> Option<&Document<PbrFallbackSettings>> {
        self.pbr_fallback
            .get(&bedrock_addon_core::pack_identity::PackPath::from_static(
                Self::PBR_FALLBACK_PATH,
            ))
    }

    /// The lighting settings of a biome that names none; `None` when the game's own apply.
    #[must_use]
    pub fn default_lighting(&self) -> Option<&Document<LightingSettings>> {
        self.lighting.get(self.defaults.lighting.as_ref()?)
    }

    /// The atmosphere settings of a biome that names none; `None` when the game's own apply.
    #[must_use]
    pub fn default_atmosphere(&self) -> Option<&Document<AtmosphereSettings>> {
        self.atmosphere.get(self.defaults.atmosphere.as_ref()?)
    }

    /// The water settings of a biome that names none; `None` when the game's own apply.
    #[must_use]
    pub fn default_water(&self) -> Option<&Document<WaterSettings>> {
        self.water.get(self.defaults.water.as_ref()?)
    }

    /// The colour-grading settings of a biome that names none; `None` when the game's own
    /// apply.
    #[must_use]
    pub fn default_color_grading(&self) -> Option<&Document<ColorGradingSettings>> {
        self.color_grading
            .get(self.defaults.color_grading.as_ref()?)
    }

    /// The cubemap settings of a biome that names none; `None` when the game's own apply.
    #[must_use]
    pub fn default_cubemap(&self) -> Option<&Document<CubemapSettings>> {
        self.cubemaps.get(&self.defaults.cubemap)
    }

    /// Every finding of the eight collections, in the order the game loads them (lighting,
    /// atmosphere, water, colour grading, PBR fallback, shadows, cubemaps, local lights), then
    /// the findings about the group as a whole.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.lighting
            .diagnostics()
            .chain(self.atmosphere.diagnostics())
            .chain(self.water.diagnostics())
            .chain(self.color_grading.diagnostics())
            .chain(self.pbr_fallback.diagnostics())
            .chain(self.shadows.diagnostics())
            .chain(self.cubemaps.diagnostics())
            .chain(self.local_lighting.diagnostics())
            .chain(self.diagnostics.iter())
    }
}

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
pub(crate) const V1_21_90: FormatVersion = known::V1_21_90;
pub(crate) const V1_21_120: FormatVersion = known::V1_21_120;
pub(crate) const V1_21_130: FormatVersion = known::V1_21_130;
pub(crate) const V1_26_0: FormatVersion = known::V1_26_0;
pub(crate) const V1_26_20: FormatVersion = known::V1_26_20;

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

/// The envelope of every payload: a missing `format_version` rejected, and below the first
/// slice no parser at all.
const PAYLOAD_ENVELOPE: Envelope = Envelope {
    missing: MissingVersion::REJECT,
    below_floor: BelowFloor::NoParser,
    ..Envelope::DEFAULT
};

impl Family for history::lighting::Family {
    const NAME: &'static str = "lighting settings";
    const PAYLOAD_KEY: &'static str = "minecraft:lighting_settings";
    const FLOOR: FormatVersion = V1_21_40;
    const ENVELOPE: Envelope = PAYLOAD_ENVELOPE;
    // Every struct's parser takes exactly its own version, so its range is
    // `[VERSION, VERSION]`; no struct for the game's experimental builds.
    const RANGE_MODEL: RangeModel = RangeModel::ExactOnly;
    type Latest = LightingSettings;
    type Beta = Infallible;
    type Component = NoComponents;
    type History = history::lighting::LightingHistory;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(value) = v {
            file::unnamed(identifier_of_lighting(value), cx);
        }
    }
}

impl JsonDocument for LightingSettings {
    type Family = history::lighting::Family;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// The envelope rules of the game's document reader, then the version chain.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<Self::Family> {
        file::read(origin, bytes, cx, None)
    }

    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<history::lighting::LightingHistory>, Rejected> {
        file::as_written::<Self::Family>(path, bytes, cx, None)
    }
}

/// The identifier a settings description names, if there is one.
fn identifier_in<K: Kind>(description: Option<&Description<K>>) -> Option<&str> {
    description.map(|d| d.identifier.as_str())
}

/// The `Definition` of each payload named by an identifier, written out: the identifier is
/// optional, so none of them is a `JsonDefinition`.
macro_rules! definition_written_out {
    ($($settings:ident),* $(,)?) => {
        $(
            impl Definition for $settings {
                type Kind = kind::$settings;
                type File = $settings;
                type Beta = NoBeta<kind::$settings>;
            }
        )*
    };
}

definition_written_out! {
    LightingSettings,
    AtmosphereSettings,
    WaterSettings,
    ColorGradingSettings,
    CubemapSettings,
}

/// The identifier a lighting settings value names, if any.
fn identifier_of_lighting(value: &LightingSettings) -> Option<&str> {
    identifier_in(value.description.as_ref())
}

impl JsonPlaced for LightingSettings {
    /// The file directly in the payload's folder, named after the identifier with `:` as `_`;
    /// a value without an identifier is named after the payload.
    fn place(&self) -> PackPath {
        let id = identifier_of_lighting(self).unwrap_or("lighting");
        default_path(LIGHTING, &Id::<kind::LightingSettings>::parse(id))
    }
}

impl Family for history::atmosphere::Family {
    const NAME: &'static str = "atmosphere settings";
    const PAYLOAD_KEY: &'static str = "minecraft:atmosphere_settings";
    const FLOOR: FormatVersion = V1_21_40;
    const ENVELOPE: Envelope = PAYLOAD_ENVELOPE;
    // Every struct's parser takes exactly its own version, so its range is
    // `[VERSION, VERSION]`; no struct for the game's experimental builds.
    const RANGE_MODEL: RangeModel = RangeModel::ExactOnly;
    type Latest = AtmosphereSettings;
    type Beta = Infallible;
    type Component = NoComponents;
    type History = history::atmosphere::AtmosphereHistory;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(value) = v {
            file::unnamed(identifier_of_atmosphere(value), cx);
        }
    }
}

impl JsonDocument for AtmosphereSettings {
    type Family = history::atmosphere::Family;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// The envelope rules of the game's document reader, then the version chain.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<Self::Family> {
        file::read(origin, bytes, cx, Some(V1_26_20))
    }

    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<history::atmosphere::AtmosphereHistory>, Rejected> {
        file::as_written::<Self::Family>(path, bytes, cx, Some(V1_26_20))
    }
}

/// The identifier an atmosphere settings value names, if any.
fn identifier_of_atmosphere(value: &AtmosphereSettings) -> Option<&str> {
    identifier_in(value.description.as_ref())
}

impl JsonPlaced for AtmosphereSettings {
    /// The file directly in the payload's folder, named after the identifier with `:` as `_`;
    /// a value without an identifier is named after the payload.
    fn place(&self) -> PackPath {
        let id = identifier_of_atmosphere(self).unwrap_or("atmospherics");
        default_path(ATMOSPHERICS, &Id::<kind::AtmosphereSettings>::parse(id))
    }
}

impl Family for history::water::Family {
    const NAME: &'static str = "water settings";
    const PAYLOAD_KEY: &'static str = "minecraft:water_settings";
    const FLOOR: FormatVersion = V1_21_40;
    const ENVELOPE: Envelope = PAYLOAD_ENVELOPE;
    // Every struct's parser takes exactly its own version, so its range is
    // `[VERSION, VERSION]`; no struct for the game's experimental builds.
    const RANGE_MODEL: RangeModel = RangeModel::ExactOnly;
    type Latest = WaterSettings;
    type Beta = Infallible;
    type Component = NoComponents;
    type History = history::water::WaterHistory;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(value) = v {
            file::unnamed(identifier_of_water(value), cx);
        }
    }
}

impl JsonDocument for WaterSettings {
    type Family = history::water::Family;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// The envelope rules of the game's document reader, then the version chain.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<Self::Family> {
        file::read(origin, bytes, cx, Some(V1_26_20))
    }

    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<history::water::WaterHistory>, Rejected> {
        file::as_written::<Self::Family>(path, bytes, cx, Some(V1_26_20))
    }
}

/// The identifier a water settings value names, if any.
fn identifier_of_water(value: &WaterSettings) -> Option<&str> {
    identifier_in(value.description.as_ref())
}

impl JsonPlaced for WaterSettings {
    /// The file directly in the payload's folder, named after the identifier with `:` as `_`;
    /// a value without an identifier is named after the payload.
    fn place(&self) -> PackPath {
        let id = identifier_of_water(self).unwrap_or("water");
        default_path(WATER, &Id::<kind::WaterSettings>::parse(id))
    }
}

impl Family for history::color_grading::Family {
    const NAME: &'static str = "color grading settings";
    const PAYLOAD_KEY: &'static str = "minecraft:color_grading_settings";
    const FLOOR: FormatVersion = V1_21_40;
    const ENVELOPE: Envelope = PAYLOAD_ENVELOPE;
    // Every struct's parser takes exactly its own version, so its range is
    // `[VERSION, VERSION]`; no struct for the game's experimental builds.
    const RANGE_MODEL: RangeModel = RangeModel::ExactOnly;
    type Latest = ColorGradingSettings;
    type Beta = Infallible;
    type Component = NoComponents;
    type History = history::color_grading::ColorGradingHistory;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(value) = v {
            file::unnamed(identifier_of_color_grading(value), cx);
        }
    }
}

impl JsonDocument for ColorGradingSettings {
    type Family = history::color_grading::Family;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// The envelope rules of the game's document reader, then the version chain.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<Self::Family> {
        file::read(origin, bytes, cx, None)
    }

    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<history::color_grading::ColorGradingHistory>, Rejected> {
        file::as_written::<Self::Family>(path, bytes, cx, None)
    }
}

/// The identifier a color grading settings value names, if any.
fn identifier_of_color_grading(value: &ColorGradingSettings) -> Option<&str> {
    value
        .description
        .as_ref()
        .and_then(|d| d.identifier.as_ref())
        .map(Id::as_str)
}

impl JsonPlaced for ColorGradingSettings {
    /// The file directly in the payload's folder, named after the identifier with `:` as `_`;
    /// a value without an identifier is named after the payload.
    fn place(&self) -> PackPath {
        let id = identifier_of_color_grading(self).unwrap_or("color_grading");
        default_path(COLOR_GRADING, &Id::<kind::ColorGradingSettings>::parse(id))
    }
}

impl Family for history::cubemap::Family {
    const NAME: &'static str = "cubemap settings";
    const PAYLOAD_KEY: &'static str = "minecraft:cubemap_settings";
    const FLOOR: FormatVersion = V1_21_130;
    const ENVELOPE: Envelope = PAYLOAD_ENVELOPE;
    // Every struct's parser takes exactly its own version, so its range is
    // `[VERSION, VERSION]`; no struct for the game's experimental builds.
    const RANGE_MODEL: RangeModel = RangeModel::ExactOnly;
    type Latest = CubemapSettings;
    type Beta = Infallible;
    type Component = NoComponents;
    type History = history::cubemap::CubemapHistory;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(value) = v {
            file::unnamed(identifier_of_cubemap(value), cx);
        }
    }
}

impl JsonDocument for CubemapSettings {
    type Family = history::cubemap::Family;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// The envelope rules of the game's document reader, then the version chain.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<Self::Family> {
        file::read(origin, bytes, cx, None)
    }

    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<history::cubemap::CubemapHistory>, Rejected> {
        file::as_written::<Self::Family>(path, bytes, cx, None)
    }
}

/// The identifier a cubemap settings value names, if any.
fn identifier_of_cubemap(value: &CubemapSettings) -> Option<&str> {
    identifier_in(value.description.as_ref())
}

impl JsonPlaced for CubemapSettings {
    /// The file directly in the payload's folder, named after the identifier with `:` as `_`;
    /// a value without an identifier is named after the payload.
    fn place(&self) -> PackPath {
        let id = identifier_of_cubemap(self).unwrap_or("cubemap");
        default_path(CUBEMAPS, &Id::<kind::CubemapSettings>::parse(id))
    }
}

impl Family for history::shadows::Family {
    const NAME: &'static str = "shadow settings";
    const PAYLOAD_KEY: &'static str = "minecraft:shadow_settings";
    const FLOOR: FormatVersion = V1_21_80;
    const ENVELOPE: Envelope = PAYLOAD_ENVELOPE;
    // Every struct's parser takes exactly its own version, so its range is
    // `[VERSION, VERSION]`; no struct for the game's experimental builds.
    const RANGE_MODEL: RangeModel = RangeModel::ExactOnly;
    type Latest = ShadowSettings;
    type Beta = Infallible;
    type Component = NoComponents;
    type History = history::shadows::ShadowsHistory;
}

impl JsonDocument for ShadowSettings {
    type Family = history::shadows::Family;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// The envelope rules of the game's document reader, then the version chain.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<Self::Family> {
        file::read(origin, bytes, cx, None)
    }

    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<history::shadows::ShadowsHistory>, Rejected> {
        file::as_written::<Self::Family>(path, bytes, cx, None)
    }
}

impl JsonPlaced for ShadowSettings {
    /// `shadows/global.json`: the one path the game reads.
    fn place(&self) -> PackPath {
        PackPath::from_static(DeferredSettings::SHADOWS_PATH)
    }
}

impl Family for history::pbr_fallback::Family {
    const NAME: &'static str = "PBR fallback settings";
    const PAYLOAD_KEY: &'static str = "minecraft:pbr_fallback_settings";
    const FLOOR: FormatVersion = V1_21_40;
    const ENVELOPE: Envelope = PAYLOAD_ENVELOPE;
    // Every struct's parser takes exactly its own version, so its range is
    // `[VERSION, VERSION]`; no struct for the game's experimental builds.
    const RANGE_MODEL: RangeModel = RangeModel::ExactOnly;
    type Latest = PbrFallbackSettings;
    type Beta = Infallible;
    type Component = NoComponents;
    type History = history::pbr_fallback::PbrFallbackHistory;
}

impl JsonDocument for PbrFallbackSettings {
    type Family = history::pbr_fallback::Family;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// The envelope rules of the game's document reader, then the version chain.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<Self::Family> {
        file::read(origin, bytes, cx, None)
    }

    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<history::pbr_fallback::PbrFallbackHistory>, Rejected> {
        file::as_written::<Self::Family>(path, bytes, cx, None)
    }
}

impl JsonPlaced for PbrFallbackSettings {
    /// `pbr/global.json`: the one path the game reads.
    fn place(&self) -> PackPath {
        PackPath::from_static(DeferredSettings::PBR_FALLBACK_PATH)
    }
}

impl Family for history::local_lighting::Family {
    const NAME: &'static str = "local light settings";
    const PAYLOAD_KEY: &'static str = "minecraft:local_light_settings";
    const FLOOR: FormatVersion = V1_21_120;
    const ENVELOPE: Envelope = PAYLOAD_ENVELOPE;
    // Every struct's parser takes exactly its own version, so its range is
    // `[VERSION, VERSION]`; no struct for the game's experimental builds.
    const RANGE_MODEL: RangeModel = RangeModel::ExactOnly;
    type Latest = LocalLightSettings;
    type Beta = Infallible;
    type Component = NoComponents;
    type History = history::local_lighting::LocalLightingHistory;
}

impl JsonDocument for LocalLightSettings {
    type Family = history::local_lighting::Family;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// The envelope rules of the game's document reader, then the version chain.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<Self::Family> {
        file::read(origin, bytes, cx, None)
    }

    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<history::local_lighting::LocalLightingHistory>, Rejected> {
        file::as_written::<Self::Family>(path, bytes, cx, None)
    }
}

impl JsonPlaced for LocalLightSettings {
    /// `local_lighting/local_lighting.json`: the one path the game reads.
    fn place(&self) -> PackPath {
        PackPath::from_static(DeferredSettings::LOCAL_LIGHTING_PATH)
    }
}

impl Merge for LocalLightSettings {
    /// One entry per block.
    type Key = BlockId;
}

// ---- the group -----------------------------------------------------------------------------

/// What one walked file parses to.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum SettingsFile {
    /// A `minecraft:lighting_settings` file.
    Lighting(Box<FileCopy<LightingSettings>>),
    /// A `minecraft:atmosphere_settings` file.
    Atmosphere(Box<FileCopy<AtmosphereSettings>>),
    /// A `minecraft:water_settings` file.
    Water(Box<FileCopy<WaterSettings>>),
    /// A `minecraft:color_grading_settings` file.
    ColorGrading(Box<FileCopy<ColorGradingSettings>>),
    /// `pbr/global.json`.
    PbrFallback(Box<FileCopy<PbrFallbackSettings>>),
    /// `shadows/global.json`.
    Shadows(Box<FileCopy<ShadowSettings>>),
    /// A `minecraft:cubemap_settings` file.
    Cubemap(Box<FileCopy<CubemapSettings>>),
    /// A pack's `local_lighting/local_lighting.json`.
    LocalLighting(Box<FileCopy<LocalLightSettings>>),
}

/// The pack a walked file is in.
fn pack_of<T>((_, outcome): &FileCopy<T>) -> &PackRef {
    match outcome {
        Ok(document) => &document.origin().pack,
        Err(rejected) => &rejected.origin.pack,
    }
}

impl SettingsFile {
    /// The pack the file is in.
    fn pack(&self) -> &PackRef {
        match self {
            Self::Lighting(walked) => pack_of(walked),
            Self::Atmosphere(walked) => pack_of(walked),
            Self::Water(walked) => pack_of(walked),
            Self::ColorGrading(walked) => pack_of(walked),
            Self::PbrFallback(walked) => pack_of(walked),
            Self::Shadows(walked) => pack_of(walked),
            Self::Cubemap(walked) => pack_of(walked),
            Self::LocalLighting(walked) => pack_of(walked),
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
const WALK: Walk = Walk::json_tree_in(DIRECTORIES)
    .accept(Accept::Fn(is_read))
    .top_down()
    .directory_major();

/// One walked file of payload `T`: the path the walk keys it by (without a subpack folder) and
/// what `JsonDocument::read` gives for it.
fn walked<T: JsonDocument>(walked: &WalkFile, cx: &LoadContext) -> Box<FileCopy<T>> {
    let read = T::read(origin_of(walked), &walked.bytes, cx);
    Box::new((walked.key.clone(), read.stable()))
}

/// Parses one walked file by its folder.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> SettingsFile {
    match WALK.matches(&file.key).map(|index| DIRECTORIES[index]) {
        Some(ATMOSPHERICS) => SettingsFile::Atmosphere(walked(file, cx)),
        Some(WATER) => SettingsFile::Water(walked(file, cx)),
        Some(COLOR_GRADING) => SettingsFile::ColorGrading(walked(file, cx)),
        Some(PBR) => SettingsFile::PbrFallback(walked(file, cx)),
        Some(SHADOWS) => SettingsFile::Shadows(walked(file, cx)),
        Some(CUBEMAPS) => SettingsFile::Cubemap(walked(file, cx)),
        Some(LOCAL_LIGHTING) => SettingsFile::LocalLighting(walked(file, cx)),
        _ => SettingsFile::Lighting(walked(file, cx)),
    }
}

/// `ShadowedDefinition` at the information level: a definition a higher-priority pack hides.
/// The game says nothing about it, so the finding is the crate's own.
fn shadowed(origin: &Origin, at: JsonPath) -> Diagnostic {
    Diagnostic::of(&core_codes::SHADOWED_DEFINITION)
        .at(at)
        .phase(Phase::Validate)
        .in_file(origin)
}

/// The identifier map of one payload while it is built: per identifier the first pack of the
/// walk, which is the highest, wins.
struct IdentifierMap<T: Definition> {
    builder: RegistryBuilder<T>,
    /// The identifier a value names, if any.
    identifier: fn(&T) -> Option<&str>,
    /// The pack whose files are being added, and the identifiers it has defined so far.
    pack: Option<PackRef>,
    in_pack: HashSet<Id<T::Kind>>,
    /// The identifier of the first loaded default file that has one.
    default: Option<Id<T::Kind>>,
}

impl<T: Definition + Clone> IdentifierMap<T> {
    fn new(stack: &StackCx<'_>, identifier: fn(&T) -> Option<&str>) -> Self {
        Self {
            builder: RegistryBuilder::new(
                CombineRule::TopWins {
                    shadowed: Some(Notice::shadowed()),
                },
                CombineRule::top_wins(),
                stack.options,
            ),
            identifier,
            pack: None,
            in_pack: HashSet::new(),
            default: None,
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
    fn push(&mut self, (key, outcome): FileCopy<T>, default_file: Option<&str>) {
        let document = match outcome {
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
        let Some(id) = file::named((self.identifier)(document.value())).map(Id::<T::Kind>::parse)
        else {
            for finding in document.diagnostics().iter().cloned() {
                self.builder.push_diagnostic(finding);
            }
            return;
        };
        if !self.in_pack.insert(id.clone()) {
            let origin = document.origin().clone();
            let mut diagnostics = document.diagnostics().clone();
            diagnostics.push(
                Diagnostic::of(codes::IDENTIFIER_IN_PACK.with([id.as_str()]))
                    .at(JsonPath::new())
                    .phase(Phase::Validate)
                    .in_file(&origin),
            );
            self.builder.push(Contribution::Rejected(
                None,
                Rejected::new(origin, diagnostics),
            ));
            return;
        }
        if self.default.is_none() && default_file == Some(key.as_str()) {
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
    let finding = Diagnostic::of(codes::NO_MATCHING_SETTINGS.with([noun, id.as_str()]))
        .phase(Phase::Validate)
        .detail(DiagnosticDetail::Reference {
            kind: <T::Kind as Kind>::NAME,
            id: id.as_str().to_owned(),
            hint: None,
        });
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
    fn push(&mut self, (key, outcome): FileCopy<T>) {
        let hidden = match &outcome {
            Ok(document) if self.loaded => Some(shadowed(document.origin(), JsonPath::new())),
            _ => None,
        };
        self.loaded |= outcome.is_ok();
        self.builder.push(key, outcome);
        if let Some(finding) = hidden {
            self.builder.push_diagnostic(finding);
        }
    }
}

/// The local lights of every pack's file, from the highest pack down: per block the first
/// entry stays; a later one is a `ShadowedDefinition` info (the game is silent, so the
/// finding is the crate's own). A file that failed contributes nothing.
fn merge_local_lights(files: Vec<FileCopy<LocalLightSettings>>) -> Merged<LocalLightSettings> {
    let walk = files.into_iter().map(|(_, outcome)| outcome).collect();
    Merged::merge(
        walk,
        LocalLightSettings::default(),
        |merged, layer: &Document<LocalLightSettings>, cx| {
            for (block, light) in &layer.lights {
                if merged.lights.contains_key(block) {
                    let at = JsonPath::new()
                        .child(history::local_lighting::Family::PAYLOAD_KEY)
                        .child(block.as_str());
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
                Code::from(&codes::PBR_CAPABILITY_HINT),
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
                    Code::from(&core_codes::IGNORED_FILE),
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
    let mut lighting = IdentifierMap::new(stack, identifier_of_lighting);
    let mut atmosphere = IdentifierMap::new(stack, identifier_of_atmosphere);
    let mut water = IdentifierMap::new(stack, identifier_of_water);
    let mut color_grading = IdentifierMap::new(stack, identifier_of_color_grading);
    let mut cubemaps = IdentifierMap::new(stack, identifier_of_cubemap);
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

#[doc(hidden)]
/// The loader of `rp.deferred_settings()`: the eight folders of every resource pack, combined
/// into the group.
pub const LOADER: CollectionLoader<DeferredSettings, SettingsFile> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};

impl Reported for DeferredSettings {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

#[cfg(test)]
mod walked_tests {
    use super::*;

    /// A walked file and what it parsed to are printable and clonable, outside `cfg(test)` too.
    #[test]
    fn a_walked_settings_file_is_debug_and_clone() {
        fn assert_traits<T: std::fmt::Debug + Clone>() {}
        assert_traits::<FileCopy<LightingSettings>>();
        assert_traits::<SettingsFile>();
    }
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn lighting_reads_as_pinned() {
        assert_reading_table(
            "history::lighting::Family",
            &reading_table::<super::history::lighting::Family>(),
            r"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.40..=1.26.0 (0..=4): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
            ",
        );
    }

    #[test]
    fn atmosphere_reads_as_pinned() {
        assert_reading_table(
            "history::atmosphere::Family",
            &reading_table::<super::history::atmosphere::Family>(),
            r"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.40..=1.26.20 (0..=1): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
            ",
        );
    }

    #[test]
    fn water_reads_as_pinned() {
        assert_reading_table(
            "history::water::Family",
            &reading_table::<super::history::water::Family>(),
            r"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.40..=1.26.20 (0..=4): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
            ",
        );
    }

    #[test]
    fn color_grading_reads_as_pinned() {
        assert_reading_table(
            "history::color_grading::Family",
            &reading_table::<super::history::color_grading::Family>(),
            r"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.40..=1.21.90 (0..=1): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
            ",
        );
    }

    #[test]
    fn cubemap_reads_as_pinned() {
        assert_reading_table(
            "history::cubemap::Family",
            &reading_table::<super::history::cubemap::Family>(),
            r"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.130..=1.21.130 (0..=0): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
            ",
        );
    }

    #[test]
    fn shadows_reads_as_pinned() {
        assert_reading_table(
            "history::shadows::Family",
            &reading_table::<super::history::shadows::Family>(),
            r"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.80..=1.21.80 (0..=0): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
            ",
        );
    }

    #[test]
    fn pbr_fallback_reads_as_pinned() {
        assert_reading_table(
            "history::pbr_fallback::Family",
            &reading_table::<super::history::pbr_fallback::Family>(),
            r"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.40..=1.21.40 (0..=0): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
            ",
        );
    }

    #[test]
    fn local_lighting_reads_as_pinned() {
        assert_reading_table(
            "history::local_lighting::Family",
            &reading_table::<super::history::local_lighting::Family>(),
            r"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.120..=1.21.120 (0..=0): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
            ",
        );
    }
}
