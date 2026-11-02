//! Components that name a definition of another file: `minecraft:fog_appearance` and the
//! render-configuration identifiers.

use facet::Facet;

use super::Checked;
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::{
    AtmosphereSettingsId, ColorGradingSettingsId, CubemapSettingsId, FogId, LightingSettingsId,
    WaterSettingsId,
};

/// One component with a single required identifier member.
macro_rules! identifier_component {
    ($(#[$doc:meta])* $name:ident, $field:ident, $id:ty, $what:literal) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Facet)]
        pub struct $name {
            #[doc = concat!("JSON `", stringify!($field), "`: ", $what, ".")]
            pub $field: $id,
            /// Unknown members.
            #[facet(skip, rename = "\u{0}extra")]
            pub extra: Extra,
        }

        impl $name {
            #[doc = concat!("The component naming `", stringify!($field), "`.")]
            pub fn new($field: $id) -> Self {
                Self {
                    $field,
                    extra: Extra::default(),
                }
            }
        }

        impl Checked for $name {}
    };
}

identifier_component! {
    /// `minecraft:fog_appearance`: the fog of the biome.
    FogAppearance, fog_identifier, FogId, "a fog definition of `fogs/`"
}

identifier_component! {
    /// `minecraft:atmosphere_identifier` (format 1.21.70): the atmosphere settings the
    /// deferred renderer uses in the biome.
    AtmosphereIdentifier, atmosphere_identifier, AtmosphereSettingsId,
    "an atmosphere definition of `atmospherics/`"
}

identifier_component! {
    /// `minecraft:color_grading_identifier` (format 1.21.70): the colour grading the deferred
    /// renderer uses in the biome.
    ColorGradingIdentifier, color_grading_identifier, ColorGradingSettingsId,
    "a colour-grading definition of `color_grading/`"
}

identifier_component! {
    /// `minecraft:lighting_identifier` (format 1.21.70): the lighting the deferred renderer
    /// uses in the biome.
    LightingIdentifier, lighting_identifier, LightingSettingsId,
    "a lighting definition of `lighting/`"
}

identifier_component! {
    /// `minecraft:water_identifier` (format 1.21.70): the water settings the deferred renderer
    /// uses in the biome.
    WaterIdentifier, water_identifier, WaterSettingsId, "a water definition of `water/`"
}

identifier_component! {
    /// `minecraft:cubemap_identifier` (format 1.21.130): the sky cubemap of the biome.
    CubemapIdentifier, cubemap_identifier, CubemapSettingsId,
    "a cubemap definition of `cubemaps/`"
}
