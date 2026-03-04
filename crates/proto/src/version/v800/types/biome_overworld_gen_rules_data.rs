use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeOverworldGenRulesData<V: ProtoVersion> {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub hills_transformations: Vec<V::BiomeWeightedData>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub mutate_transformations: Vec<V::BiomeWeightedData>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub river_transformations: Vec<V::BiomeWeightedData>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub shore_transformations: Vec<V::BiomeWeightedData>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub pre_hills_edge_transformations: Vec<V::BiomeConditionalTransformationData>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub post_shore_transformations: Vec<V::BiomeConditionalTransformationData>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub climate_transformations: Vec<V::BiomeWeightedTemperatureData>,
}
