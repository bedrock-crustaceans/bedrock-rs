//! The components of a biome (`minecraft:biome/components`): one type per component, named
//! after the JSON key without its namespace, in `PascalCase`.
//!
//! [`BiomeComponent`] is the closed set a [`bedrock_addon_core::document::ComponentMap`] of a biome holds;
//! `biome.components.get::<Climate>()` reads one. A key the set does not know is reported and
//! dropped; vanilla rejects such a document.

mod climate;
mod generation;
mod surface;

pub use climate::{
    Climate, Color, ColorRgba, CreatureSpawnProbability, GrassTint, Humidity, MapTints,
    PartiallyFrozen, Tags, VillageKind, VillageType,
};
pub use generation::{
    BiomeReplacement, CategoryOrWeight, MountainParameters, MultinoiseGenerationRules,
    NameOrWeight, NoiseType, OverworldGenerationRules, OverworldHeight, ReplaceBiomes,
    SteepMaterialAdjustment, TemperatureCategory, TopSlide, WeightedBiome, WeightedBiomes,
    WeightedClimate,
};
pub use surface::{
    Adjustment, AdjustmentMaterials, Builder, CappedSurface, MesaSurface, NoiseBlockSpecifier,
    NoiseDescriptor, NoiseGradientSurface, OverworldSurface, SubsurfaceBuilder, SurfaceBuilder,
    SurfaceMaterialAdjustments, SwampSurface, TheEndSurface,
};

use crate::read::V1_26_30;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::FromJson;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_derive::Components;
use facet::Facet;

use super::history::BiomeFamily;
use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::molang::{
    Molang, MolangFieldSpec, MolangVersion, MolangVersionSource, MolangVisitor, ReaderKind,
};
use bedrock_addon_core::versioning::write_payload;
use bedrock_addon_shared_biome_env::read_row_of_struct;

// Every component is released with the first format version whose document declares it: most
// with 1.21.110, `village_type` with 1.26.0 and `subsurface_builder` with 1.26.30. No component
// has a requirement, a deprecation or a non-public area; `released` is the first version that
// binds it, and a document below it meets the unknown-component rule instead.

/// One component of a biome: the closed set of `minecraft:` components the 1.26.30 document
/// declares.
#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = BiomeFamily, write_with = written)]
#[non_exhaustive]
pub enum BiomeComponent {
    /// `minecraft:climate`.
    #[component(name = "minecraft:climate", released = "1.21.110", parse = read_row_of_struct::<Climate, BiomeComponent>)]
    Climate(Climate),
    /// `minecraft:creature_spawn_probability`.
    #[component(name = "minecraft:creature_spawn_probability", released = "1.21.110", parse = read_row_of_struct::<CreatureSpawnProbability, BiomeComponent>)]
    CreatureSpawnProbability(CreatureSpawnProbability),
    /// `minecraft:humidity`.
    #[component(name = "minecraft:humidity", released = "1.21.110", parse = read_row_of_struct::<Humidity, BiomeComponent>)]
    Humidity(Humidity),
    /// `minecraft:map_tints`.
    #[component(name = "minecraft:map_tints", released = "1.21.110", parse = read_row_of_struct::<MapTints, BiomeComponent>)]
    MapTints(MapTints),
    /// `minecraft:mountain_parameters`.
    #[component(name = "minecraft:mountain_parameters", released = "1.21.110", parse = read_row_of_struct::<MountainParameters, BiomeComponent>)]
    MountainParameters(MountainParameters),
    /// `minecraft:multinoise_generation_rules`.
    #[component(name = "minecraft:multinoise_generation_rules", released = "1.21.110", parse = read_row_of_struct::<MultinoiseGenerationRules, BiomeComponent>)]
    MultinoiseGenerationRules(MultinoiseGenerationRules),
    /// `minecraft:overworld_generation_rules`.
    #[component(name = "minecraft:overworld_generation_rules", released = "1.21.110", parse = read_row_of_struct::<OverworldGenerationRules, BiomeComponent>)]
    OverworldGenerationRules(OverworldGenerationRules),
    /// `minecraft:overworld_height`.
    #[component(name = "minecraft:overworld_height", released = "1.21.110", parse = read_row_of_struct::<OverworldHeight, BiomeComponent>)]
    OverworldHeight(OverworldHeight),
    /// `minecraft:partially_frozen`.
    #[component(name = "minecraft:partially_frozen", released = "1.21.110", parse = read_row_of_struct::<PartiallyFrozen, BiomeComponent>)]
    PartiallyFrozen(PartiallyFrozen),
    /// `minecraft:replace_biomes`.
    #[component(name = "minecraft:replace_biomes", released = "1.21.110", parse = read_row_of_struct::<ReplaceBiomes, BiomeComponent>)]
    ReplaceBiomes(ReplaceBiomes),
    /// `minecraft:subsurface_builder` (format 1.26.30).
    #[component(name = "minecraft:subsurface_builder", released = "1.26.30", parse = builder_row::<SubsurfaceBuilder>)]
    SubsurfaceBuilder(SubsurfaceBuilder),
    /// `minecraft:surface_builder`.
    #[component(name = "minecraft:surface_builder", released = "1.21.110", parse = builder_row::<SurfaceBuilder>)]
    SurfaceBuilder(SurfaceBuilder),
    /// `minecraft:surface_material_adjustments`.
    #[component(name = "minecraft:surface_material_adjustments", released = "1.21.110", molang, parse = read_row_of_struct::<SurfaceMaterialAdjustments, BiomeComponent>)]
    SurfaceMaterialAdjustments(SurfaceMaterialAdjustments),
    /// `minecraft:tags`.
    #[component(name = "minecraft:tags", released = "1.21.110", parse = read_row_of_struct::<Tags, BiomeComponent>)]
    Tags(Tags),
    /// `minecraft:village_type` (format 1.26.0).
    #[component(name = "minecraft:village_type", released = "1.26.0", parse = read_row_of_struct::<VillageType, BiomeComponent>)]
    VillageType(VillageType),
}

/// One component's JSON, without a member written `null`: an absent `Option` has no JSON
/// (a tagged builder's alternative writes it as `null`), and a `null` member refuses the
/// biome on the next read.
fn written<'f, T: Facet<'f>>(component: &T) -> Result<Value, EncodeError> {
    let mut json = write_payload(component)?;
    bedrock_addon_core::json::remove_null_members(T::SHAPE, &mut json);
    Ok(json)
}

/// A component map as a JSON object, in map order.
pub(crate) fn write_components(map: &ComponentMap<BiomeComponent>) -> Result<Value, EncodeError> {
    bedrock_addon_shared_biome_env::write_components(map, BiomeComponent::write_component)
}

/// `surface_material_adjustments.adjustments[].height_range[0..1]`: a number, bool or string
/// (no object form), compiled at the constant Molang version 1 with only the `world_gen`
/// queries; a rejected expression fails the component and the biome is not registered.
// The game compiles these expressions at Molang version 1 with only the `world_gen` queries
// admitted, so a query outside that set rejects the component.
// An empty text is compiled: at version 1 it is rejected with only an info note.
pub(crate) const HEIGHT_RANGE: MolangFieldSpec = MolangFieldSpec::REJECTING
    .reader(ReaderKind::BiomeHeightRange)
    .version(MolangVersionSource::Constant(MolangVersion::V1))
    .world_gen()
    .compile_empty();

/// The walk of one adjustment's `height_range`: both ends under [`HEIGHT_RANGE`]
/// (`schema::molang_with` on [`Adjustment::height_range`]).
pub(crate) fn height_range(range: &mut Option<[Molang; 2]>, visitor: &mut MolangVisitor<'_>) {
    if let Some([low, high]) = range {
        visitor.field(0, &HEIGHT_RANGE, low);
        visitor.field(1, &HEIGHT_RANGE, high);
    }
}

pub(crate) use bedrock_addon_support::biome_json::child;

/// A builder row (`surface_builder`, `subsurface_builder`): below 1.26.30 the builder variant has
/// no `minecraft:noise_gradient` alternative, so such a builder is an unknown `type`, which the
/// strict slice rejects.
fn builder_row<T>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<BiomeComponent>
where
    T: FromJson + Into<BiomeComponent>,
{
    let value = read_row_of_struct::<T, BiomeComponent>(slice, mode, cx, d)?;
    let (BiomeComponent::SurfaceBuilder(SurfaceBuilder { builder, .. })
    | BiomeComponent::SubsurfaceBuilder(SubsurfaceBuilder { builder, .. })) = &value
    else {
        return Some(value);
    };
    if cx.document_format.is_some_and(|version| version < V1_26_30)
        && matches!(builder, Builder::NoiseGradient(_))
    {
        d.push(
            Diagnostic::of(&core_codes::INVALID_ENUM_VALUE)
                .at(child(&JsonPath::new(), &["builder", "type"])),
        );
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An absent `Option` is written as no member, never as `null`: the game refuses a `null`
    /// member, so a written `null` would refuse the biome on the next read.
    #[test]
    fn an_absent_option_is_written_as_no_member() {
        let text = r#"{"builder": {"type": "minecraft:noise_gradient", "noise_descriptor": {"name": "test:gradient", "first_octave": -4, "amplitudes": [1.0]}, "noise_block_specifiers": [{"range": {"min": 0.5, "max": 0.7}, "block": "minecraft:calcite"}]}}"#;
        let mut found = Diagnostics::default();
        let builder = read_row_of_struct::<SubsurfaceBuilder, BiomeComponent>(
            JsonSlice::new(text),
            ConvertMode::Strict,
            &LoadContext::detached(),
            &mut found,
        )
        .expect("parses");
        let written = builder.write_component().unwrap();
        let written = bedrock_addon_core::json::to_json(&written).unwrap();
        assert!(!written.contains("null"), "{written}");
    }
}
