use bedrockrs_macros::ProtoCodec;
use crate::v800::types::{
    BiomeCappedSurfaceData, BiomeClimateData, BiomeConsolidatedFeatureList,
    BiomeLegacyWorldGenRulesData, BiomeMesaSurfaceData, BiomeMountainParamsData,
    BiomeMultinoiseGenRulesData, BiomeOverworldGenRulesData,
    BiomeSurfaceMaterialAdjustmentData, BiomeSurfaceMaterialData,
};

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeDefinitionChunkGenData {
    pub climate: Option<BiomeClimateData>,
    pub consolidated_features: Option<BiomeConsolidatedFeatureList>,
    pub mountain_params: Option<BiomeMountainParamsData>,
    pub surface_material_adjustment: Option<BiomeSurfaceMaterialAdjustmentData>,
    pub surface_material: Option<BiomeSurfaceMaterialData>,
    pub has_swamp_surface: bool,
    pub has_frozen_ocean_surface: bool,
    pub has_the_end_surface: bool,
    pub mesa_surface: Option<BiomeMesaSurfaceData>,
    pub capped_surface: Option<BiomeCappedSurfaceData>,
    pub overworld_gen_rules: Option<BiomeOverworldGenRulesData>,
    pub multinoise_gen_rules: Option<BiomeMultinoiseGenRulesData>,
    pub legacy_world_gen_rules: Option<BiomeLegacyWorldGenRulesData>,
}
