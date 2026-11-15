//! The components of a client biome (`minecraft:client_biome/components`): one type per
//! component, named after the JSON key without its namespace, in `PascalCase`.
//!
//! [`ClientBiomeComponent`] is the closed set a [`bedrock_addon_core::document::ComponentMap`] of a client
//! biome holds; `biome.components.get::<SkyColor>()` reads one. A key the set does not know is
//! reported and kept as written; vanilla rejects such a document.

mod appearance;
mod identifier;
mod sound;

pub use appearance::{
    ColorMap, ColorOrMap, DryFoliageColor, FoliageAppearance, FoliageColor, FoliageColorMap,
    GrassAppearance, GrassColor, GrassColorMap, Precipitation, SkyColor, WaterAppearance,
};
pub use identifier::{
    AtmosphereIdentifier, ColorGradingIdentifier, CubemapIdentifier, FogAppearance,
    LightingIdentifier, WaterIdentifier,
};
pub use sound::{AmbientSounds, BiomeMusic, SoundAddition};

use bedrock_addon_derive::Components;

use super::history::ClientBiomeFamily;
use crate::codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::FromJson;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_shared_biome_env::read_row_with;

// Every component is released with the first format version whose document declares it:
// 1.21.40 declares seven, 1.21.70 adds five (the atmosphere, colour-grading, lighting and
// water identifiers and `minecraft:dry_foliage_color`), 1.21.110 adds `minecraft:precipitation`
// and 1.21.130 adds `minecraft:cubemap_identifier`. The 1.21.130 document holds all fourteen,
// each with the type of the version that last changed it. No component has a requirement, a
// deprecation or a non-public area; `released` is the first version that binds it.

/// One component of a client biome: the closed set of `minecraft:` components the 1.21.130
/// document declares.
#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = ClientBiomeFamily, write)]
#[non_exhaustive]
#[allow(
    clippy::large_enum_variant,
    reason = "a component map holds few, and boxing every variant would cost each access"
)]
pub enum ClientBiomeComponent {
    /// `minecraft:ambient_sounds`.
    #[component(name = "minecraft:ambient_sounds", released = "1.21.40", parse = typed::<AmbientSounds>)]
    AmbientSounds(AmbientSounds),
    /// `minecraft:atmosphere_identifier` (format 1.21.70).
    #[component(name = "minecraft:atmosphere_identifier", released = "1.21.70", parse = typed::<AtmosphereIdentifier>)]
    AtmosphereIdentifier(AtmosphereIdentifier),
    /// `minecraft:biome_music`.
    #[component(name = "minecraft:biome_music", released = "1.21.40", parse = typed::<BiomeMusic>)]
    BiomeMusic(BiomeMusic),
    /// `minecraft:color_grading_identifier` (format 1.21.70).
    #[component(name = "minecraft:color_grading_identifier", released = "1.21.70", parse = typed::<ColorGradingIdentifier>)]
    ColorGradingIdentifier(ColorGradingIdentifier),
    /// `minecraft:cubemap_identifier` (format 1.21.130).
    #[component(name = "minecraft:cubemap_identifier", released = "1.21.130", parse = typed::<CubemapIdentifier>)]
    CubemapIdentifier(CubemapIdentifier),
    /// `minecraft:dry_foliage_color` (format 1.21.70).
    #[component(name = "minecraft:dry_foliage_color", released = "1.21.70", parse = typed::<DryFoliageColor>)]
    DryFoliageColor(DryFoliageColor),
    /// `minecraft:fog_appearance`.
    #[component(name = "minecraft:fog_appearance", released = "1.21.40", parse = typed::<FogAppearance>)]
    FogAppearance(FogAppearance),
    /// `minecraft:foliage_appearance`.
    #[component(name = "minecraft:foliage_appearance", released = "1.21.40", parse = typed::<FoliageAppearance>)]
    FoliageAppearance(FoliageAppearance),
    /// `minecraft:grass_appearance`.
    #[component(name = "minecraft:grass_appearance", released = "1.21.40", parse = typed::<GrassAppearance>)]
    GrassAppearance(GrassAppearance),
    /// `minecraft:lighting_identifier` (format 1.21.70).
    #[component(name = "minecraft:lighting_identifier", released = "1.21.70", parse = typed::<LightingIdentifier>)]
    LightingIdentifier(LightingIdentifier),
    /// `minecraft:precipitation` (format 1.21.110).
    #[component(name = "minecraft:precipitation", released = "1.21.110", parse = typed::<Precipitation>)]
    Precipitation(Precipitation),
    /// `minecraft:sky_color`.
    #[component(name = "minecraft:sky_color", released = "1.21.40", parse = typed::<SkyColor>)]
    SkyColor(SkyColor),
    /// `minecraft:water_appearance`.
    #[component(name = "minecraft:water_appearance", released = "1.21.40", parse = water_appearance)]
    WaterAppearance(WaterAppearance),
    /// `minecraft:water_identifier` (format 1.21.70).
    #[component(name = "minecraft:water_identifier", released = "1.21.70", parse = typed::<WaterIdentifier>)]
    WaterIdentifier(WaterIdentifier),
}

/// The parser of a typed component's table row: the component from its slice, its unknown
/// members kept ([`crate::typed_read`]).
fn typed<T>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<ClientBiomeComponent>
where
    T: FromJson + Into<ClientBiomeComponent>,
{
    read_row_with::<T, ClientBiomeComponent>(slice, mode, cx, d, crate::typed_read)
}

/// `minecraft:water_appearance`, which needs at least one member: a component with neither is
/// vanilla's error at the component and nothing else (a rule across the two members, after the
/// read; what the read told of a component that is empty is its unknown members, which the game
/// does not get to).
fn water_appearance(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<ClientBiomeComponent> {
    let mut told = Diagnostics::default();
    let water = read_row_with::<WaterAppearance, WaterAppearance>(
        slice,
        mode,
        cx,
        &mut told,
        crate::typed_read,
    );
    if water
        .as_ref()
        .is_some_and(|water| water.surface_color.is_none() && water.surface_opacity.is_none())
    {
        d.push(Diagnostic::of(&codes::WATER_EMPTY));
        return None;
    }
    d.extend_with_unknown(told);
    water.map(Into::into)
}

pub(crate) use bedrock_addon_support::biome_json::child;
