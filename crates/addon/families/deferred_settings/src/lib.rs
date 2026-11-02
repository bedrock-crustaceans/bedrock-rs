//! The deferred renderer's settings ("Vibrant Visuals"): the eight document kinds a resource
//! pack configures it with — lighting (`lighting/`), atmosphere (`atmospherics/`), water
//! (`water/`), colour grading (`color_grading/`) and cubemaps (`cubemaps/`), each named by an
//! identifier client biomes refer to; the shadow style (`shadows/global.json`) and the PBR
//! fallback values (`pbr/global.json`), one file each; and the local lights
//! (`local_lighting/local_lighting.json`), merged per block — loaded together as the
//! `DeferredSettings` group.

#![cfg_attr(docsrs, feature(doc_cfg))]

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
mod load;
mod local_light;
mod new;
mod pbr;
mod read;
mod shadows;
mod value;
mod water;
mod write;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub use load::load_group;
// The checker of the references the link pass resolves, for its rules.
#[doc(hidden)]
pub use link::{Targets, Texture, check as check_links};

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::{Document, Files, Merged, Registry};
use bedrock_addon_core::id::{
    AtmosphereSettingsId, ColorGradingSettingsId, CubemapSettingsId, LightingSettingsId,
    WaterSettingsId,
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
