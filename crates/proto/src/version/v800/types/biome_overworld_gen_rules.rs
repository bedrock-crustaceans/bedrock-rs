use bedrockrs_macros::ProtoCodec;

use crate::v800::types::{BiomeConditionalTransformationData, BiomeWeightedData, BiomeWeightedTemperatureData};

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeOverworldGenRulesData {
    #[endianness(le)]
    pub hills_transformations: Vec<BiomeWeightedData>,
    #[endianness(le)]
    pub mutate_transformations: Vec<BiomeWeightedData>,
    #[endianness(le)]
    pub river_transformations: Vec<BiomeWeightedData>,
    #[endianness(le)]
    pub shore_transformations: Vec<BiomeWeightedData>,
    #[endianness(le)]
    pub pre_hills_edge_transformations: Vec<BiomeConditionalTransformationData>,
    #[endianness(le)]
    pub post_shore_transformations: Vec<BiomeConditionalTransformationData>,
    #[endianness(le)]
    pub climate_transformations: Vec<BiomeWeightedTemperatureData>,
}