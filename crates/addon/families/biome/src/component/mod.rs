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

use bedrock_addon_derive::Components;
use facet::Facet;

use super::history::BiomeFamily;
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::document::{ComponentMap, ComponentRef};
use bedrock_addon_core::json::NullRule;
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, Misreading, Value, keep_members, parse_at, unknown_members,
};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{
    MolangFieldSpec, MolangFields, MolangVersion, MolangVersionSource, MolangVisitor, ReaderKind,
    Step,
};
use bedrock_addon_core::versioning::{ConvertMode, write_payload};

// Every component is released with the first format version whose document declares it: most
// with 1.21.110, `village_type` with 1.26.0 and `subsurface_builder` with 1.26.30. No component
// has a requirement, a deprecation or a non-public area; `released` is the first version that
// binds it, and a document below it meets the unknown-component rule instead.

/// One component of a biome: the closed set of `minecraft:` components the 1.26.30 document
/// declares.
#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = BiomeFamily)]
#[non_exhaustive]
pub enum BiomeComponent {
    /// `minecraft:climate`.
    #[component(name = "minecraft:climate", released = "1.21.110", parse = parse_row::<Climate>)]
    Climate(Climate),
    /// `minecraft:creature_spawn_probability`.
    #[component(name = "minecraft:creature_spawn_probability", released = "1.21.110", parse = parse_row::<CreatureSpawnProbability>)]
    CreatureSpawnProbability(CreatureSpawnProbability),
    /// `minecraft:humidity`.
    #[component(name = "minecraft:humidity", released = "1.21.110", parse = parse_row::<Humidity>)]
    Humidity(Humidity),
    /// `minecraft:map_tints`.
    #[component(name = "minecraft:map_tints", released = "1.21.110", parse = parse_row::<MapTints>)]
    MapTints(MapTints),
    /// `minecraft:mountain_parameters`.
    #[component(name = "minecraft:mountain_parameters", released = "1.21.110", parse = parse_row::<MountainParameters>)]
    MountainParameters(MountainParameters),
    /// `minecraft:multinoise_generation_rules`.
    #[component(name = "minecraft:multinoise_generation_rules", released = "1.21.110", parse = parse_row::<MultinoiseGenerationRules>)]
    MultinoiseGenerationRules(MultinoiseGenerationRules),
    /// `minecraft:overworld_generation_rules`.
    #[component(name = "minecraft:overworld_generation_rules", released = "1.21.110", parse = parse_row::<OverworldGenerationRules>)]
    OverworldGenerationRules(OverworldGenerationRules),
    /// `minecraft:overworld_height`.
    #[component(name = "minecraft:overworld_height", released = "1.21.110", parse = parse_row::<OverworldHeight>)]
    OverworldHeight(OverworldHeight),
    /// `minecraft:partially_frozen`.
    #[component(name = "minecraft:partially_frozen", released = "1.21.110", parse = parse_row::<PartiallyFrozen>)]
    PartiallyFrozen(PartiallyFrozen),
    /// `minecraft:replace_biomes`.
    #[component(name = "minecraft:replace_biomes", released = "1.21.110", parse = parse_row::<ReplaceBiomes>)]
    ReplaceBiomes(ReplaceBiomes),
    /// `minecraft:subsurface_builder` (format 1.26.30).
    #[component(name = "minecraft:subsurface_builder", released = "1.26.30", parse = parse_row::<SubsurfaceBuilder>)]
    SubsurfaceBuilder(SubsurfaceBuilder),
    /// `minecraft:surface_builder`.
    #[component(name = "minecraft:surface_builder", released = "1.21.110", parse = parse_row::<SurfaceBuilder>)]
    SurfaceBuilder(SurfaceBuilder),
    /// `minecraft:surface_material_adjustments`.
    #[component(name = "minecraft:surface_material_adjustments", released = "1.21.110", molang, parse = parse_row::<SurfaceMaterialAdjustments>)]
    SurfaceMaterialAdjustments(SurfaceMaterialAdjustments),
    /// `minecraft:tags`.
    #[component(name = "minecraft:tags", released = "1.21.110", parse = parse_row::<Tags>)]
    Tags(Tags),
    /// `minecraft:village_type` (format 1.26.0).
    #[component(name = "minecraft:village_type", released = "1.26.0", parse = parse_row::<VillageType>)]
    VillageType(VillageType),
}

/// One component read strictly from its slice: every slice of the biome loader parses with
/// the strict mask, so a component that does not deserialise rejects the document (an error).
/// Unknown members are kept and reported as Parse-phase `UnrecognizedField` warnings, which
/// core's `unknown_members_are_errors` (`versioning/pipeline.rs`) turns into the refusing error the
/// game gives for an unknown member; the `Warning` is that step's input, not a grade. Findings
/// are relative to the component.
fn parse_row<T>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<BiomeComponent>
where
    T: for<'a> Facet<'a> + Into<BiomeComponent>,
{
    let _ = mode;
    let json = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text).ok();
    let unrecognized = |path: JsonPath| {
        Diagnostic::warn(DiagnosticCode::UnrecognizedField { text: None }).at(path)
    };
    let mut value = match parse_at::<T>(slice) {
        Ok(value) => value,
        Err(error) => {
            let code = error.code_with_path(error.has_path());
            d.push(
                Diagnostic::error(code)
                    .at(without_variant_index(&error.path))
                    .span(error.span),
            );
            // The members no field takes are reported also when the component fails, so a
            // document of another era names what it has that the struct does not.
            for (path, _) in json.iter().flat_map(|json| unknown_members(T::SHAPE, json)) {
                d.push(unrecognized(path));
            }
            return None;
        }
    };
    // A `null` where the component reads a value is a type error, which refuses the biome.
    if cx.null_rule == Some(NullRule::Rejected)
        && bedrock_addon_core::json::reject_nulls(&value, slice, &JsonPath::new(), d)
    {
        return None;
    }
    if let Some(json) = &json {
        let (unknown, misread) = keep_members(&mut value, json);
        for (path, _) in unknown {
            d.push(unrecognized(path));
        }
        // Without a null rule, a `null` read as a struct of defaults (the component itself, a
        // struct inside it) is kept so and told where it is. A number beyond a float is the
        // clamp's.
        for misread in misread {
            if matches!(misread.kind, Misreading::WrongType) {
                d.push(misread.finding(&JsonPath::new()));
            }
        }
    }
    Some(value.into())
}

/// facet's path of a failing member inside an internally tagged builder holds the newtype
/// variant's field index (`builder/0/noise_descriptor`); the JSON has no such level.
fn without_variant_index(path: &JsonPath) -> JsonPath {
    let mut out = JsonPath::new();
    let mut below_builder = false;
    for segment in path.segments() {
        match segment {
            bedrock_addon_core::diagnostic::JsonPathSegment::Index(0) if below_builder => {}
            bedrock_addon_core::diagnostic::JsonPathSegment::Index(index) => out.push_index(index),
            bedrock_addon_core::diagnostic::JsonPathSegment::Key(key) => out.push(key),
        }
        below_builder = segment == bedrock_addon_core::diagnostic::JsonPathSegment::Key("builder");
    }
    out
}

/// One typed component as JSON.
pub(crate) fn write_component(component: &BiomeComponent) -> Result<Value, EncodeError> {
    use BiomeComponent as C;
    match component {
        C::Climate(c) => written(c),
        C::CreatureSpawnProbability(c) => written(c),
        C::Humidity(c) => written(c),
        C::MapTints(c) => written(c),
        C::MountainParameters(c) => written(c),
        C::MultinoiseGenerationRules(c) => written(c),
        C::OverworldGenerationRules(c) => written(c),
        C::OverworldHeight(c) => written(c),
        C::PartiallyFrozen(c) => written(c),
        C::ReplaceBiomes(c) => written(c),
        C::SubsurfaceBuilder(c) => written(c),
        C::SurfaceBuilder(c) => written(c),
        C::SurfaceMaterialAdjustments(c) => written(c),
        C::Tags(c) => written(c),
        C::VillageType(c) => written(c),
    }
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
    let mut object = facet_value::VObject::new();
    for entry in map.iter() {
        let (name, json) = match entry {
            ComponentRef::Typed { name, value, .. } => (name.to_owned(), write_component(value)?),
            ComponentRef::Custom(c) => (c.name.clone(), c.parameters.clone()),
            ComponentRef::Dynamic { name, json } | ComponentRef::Unknown { name, json } => {
                (name.to_owned(), json.clone())
            }
            _ => continue,
        };
        object.insert(name, json);
    }
    Ok(object.into())
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

impl MolangFields for SurfaceMaterialAdjustments {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let Some(adjustments) = &mut self.adjustments else {
            return;
        };
        visitor.enter(Step::from("adjustments"));
        for (index, adjustment) in adjustments.iter_mut().enumerate() {
            if let Some([low, high]) = &mut adjustment.height_range {
                visitor.enter(Step::Index(index));
                visitor.enter(Step::from("height_range"));
                visitor.field(0, &HEIGHT_RANGE, low);
                visitor.field(1, &HEIGHT_RANGE, high);
                visitor.leave();
                visitor.leave();
            }
        }
        visitor.leave();
    }
}

pub(crate) use bedrock_addon_support::biome_json::child;

#[cfg(test)]
mod tests {
    use super::*;

    /// An absent `Option` is written as no member, never as `null`: the game refuses a `null`
    /// member, so a written `null` would refuse the biome on the next read.
    #[test]
    fn an_absent_option_is_written_as_no_member() {
        let text = r#"{"builder": {"type": "minecraft:noise_gradient", "noise_descriptor": {"name": "test:gradient", "first_octave": -4, "amplitudes": [1.0]}, "noise_block_specifiers": [{"range": {"min": 0.5, "max": 0.7}, "block": "minecraft:calcite"}]}}"#;
        let builder = parse_at::<SubsurfaceBuilder>(JsonSlice::new(text)).expect("parses");
        let written = write_component(&BiomeComponent::SubsurfaceBuilder(builder)).unwrap();
        let written = bedrock_addon_core::json::to_json(&written).unwrap();
        assert!(!written.contains("null"), "{written}");
    }
}
