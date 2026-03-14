use bedrockrs_macros::ProtoCodec;

use crate::version::ProtoVersion;

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeDefinitionChunkGenData<V: ProtoVersion> {
    pub climate: Option<V::BiomeClimateData>,
    pub consolidated_features: Option<V::BiomeConsolidatedFeatureList>,
    pub mountain_params: Option<V::BiomeMountainParamsData>,
    pub surface_material_adjustment: Option<V::BiomeSurfaceMaterialAdjustmentData>,
    pub surface_material: Option<V::BiomeSurfaceMaterialData>,
    pub has_default_overworld_surface: bool,
    pub has_swamp_surface: bool,
    pub has_frozen_ocean_surface: bool,
    pub has_the_end_surface: bool,
    pub mesa_surface: Option<V::BiomeMesaSurfaceData>,
    pub capped_surface: Option<V::BiomeCappedSurfaceData>,
    pub overworld_gen_rules: Option<V::BiomeOverworldGenRulesData>,
    pub multinoise_gen_rules: Option<V::BiomeMultinoiseGenRulesData>,
    pub legacy_world_gen_rules: Option<V::BiomeLegacyWorldGenRulesData>,
}
