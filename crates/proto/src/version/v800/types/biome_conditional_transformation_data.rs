use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeConditionalTransformationData<V: ProtoVersion> {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub weighted_biomes: Vec<V::BiomeWeightedData>,
    #[endianness(le)]
    pub condition_json: u16,
    #[endianness(le)]
    pub min_passing_neighbors: u32,
}
