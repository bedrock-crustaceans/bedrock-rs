use bedrockrs_macros::ProtoCodec;

use crate::version::ProtoVersion;

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeDefinitionChunkGenData<V: ProtoVersion> {
    pub climate: Option<V::BiomeClimateData>,
    pub consolidated_features: Option<V::BiomeConsolidatedFeatureList>,
    pub mountain_params: Option<V::BiomeMountainParamsData>,
    pub surface_material_adjustment: Option<V::BiomeSurfaceMaterialAdjustmentData>,
    pub overworld_gen_rules: Option<V::BiomeOverworldGenRulesData>,
    pub multinoise_gen_rules: Option<V::BiomeMultinoiseGenRulesData>,
    pub legacy_world_gen_rules: Option<V::BiomeLegacyWorldGenRulesData>,
    pub replacement_biomes: Option<Vec<V::BiomeReplacementData>>,
    pub village_type: Option<u8>,
    pub surface_builder_data: Option<V::BiomeSurfaceBuilderData>,
    pub sub_surface_builder_data: Option<V::BiomeSurfaceBuilderData>,
}
