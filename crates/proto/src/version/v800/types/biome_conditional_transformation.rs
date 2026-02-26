use crate::v800::types::BiomeWeightedData;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeConditionalTransformationData {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub weighted_biomes: Vec<BiomeWeightedData>,
    #[endianness(le)]
    pub condition_json: u16,
    #[endianness(le)]
    pub min_passing_neighbors: u32,
}
