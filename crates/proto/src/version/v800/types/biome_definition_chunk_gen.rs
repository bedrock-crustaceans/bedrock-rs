use std::option::Option;
use bedrockrs_macros::ProtoCodec;
use crate::v800::types::{BiomeCappedSurfaceData, BiomeClimateData, BiomeConsolidatedFeatureData, BiomeLegacyWorldGenRulesData, BiomeMesaSurfaceData, BiomeMountainParamsData, BiomeMultinoiseGenRulesData, BiomeOverworldGenRulesData, BiomeSurfaceMaterialAdjustmentData, BiomeSurfaceMaterialData};

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeDefinitionChunkGenData {
    #[endianness(le)]
    pub climate: Option<BiomeClimateData>,
    #[endianness(le)]
    pub consolidated_features: Option<Vec<BiomeConsolidatedFeatureData>>,
    #[endianness(le)]
    pub mountain_params: Option<BiomeMountainParamsData>,
    #[endianness(le)]
    pub surface_material_adjustment: Option<BiomeSurfaceMaterialAdjustmentData>,
    #[endianness(le)]
    pub surface_material: Option<BiomeSurfaceMaterialData>,
    pub has_swamp_surface: bool,
    pub has_frozen_ocean_surface: bool,
    pub has_the_end_surface: bool,
    #[endianness(le)]
    pub mesa_surface: Option<BiomeMesaSurfaceData>,
    #[endianness(le)]
    pub capped_surface: Option<BiomeCappedSurfaceData>,
    #[endianness(le)]
    pub overworld_gen_rules: Option<BiomeOverworldGenRulesData>,
    #[endianness(le)]
    pub multinoise_gen_rules: Option<BiomeMultinoiseGenRulesData>,
    #[endianness(le)]
    pub legacy_world_gen_rules: Option<BiomeLegacyWorldGenRulesData>
}