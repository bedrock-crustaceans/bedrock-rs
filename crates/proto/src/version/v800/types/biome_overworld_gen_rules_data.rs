use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeOverworldGenRulesData<V: ProtoVersion> {
    pub hills_transformations: Vec<V::BiomeWeightedData>,

    pub mutate_transformations: Vec<V::BiomeWeightedData>,

    pub river_transformations: Vec<V::BiomeWeightedData>,

    pub shore_transformations: Vec<V::BiomeWeightedData>,

    pub pre_hills_edge_transformations: Vec<V::BiomeConditionalTransformationData>,

    pub post_shore_transformations: Vec<V::BiomeConditionalTransformationData>,

    pub climate_transformations: Vec<V::BiomeWeightedTemperatureData>,
}
