use bedrockrs_macros::ProtoCodec;
use crate::v800::types::{
    BiomeConditionalTransformationData, BiomeWeightedData, BiomeWeightedTemperatureData,
};

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeOverworldGenRulesData {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub hills_transformations: Vec<BiomeWeightedData>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub mutate_transformations: Vec<BiomeWeightedData>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub river_transformations: Vec<BiomeWeightedData>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub shore_transformations: Vec<BiomeWeightedData>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub pre_hills_edge_transformations: Vec<BiomeConditionalTransformationData>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub post_shore_transformations: Vec<BiomeConditionalTransformationData>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub climate_transformations: Vec<BiomeWeightedTemperatureData>,
}
