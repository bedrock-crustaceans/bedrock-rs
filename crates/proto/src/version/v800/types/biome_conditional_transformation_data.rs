use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeConditionalTransformationData<V: ProtoVersion> {
    pub weighted_biomes: Vec<V::BiomeWeightedData>,
    #[endianness(le)]
    pub condition_json: u16,
    #[endianness(le)]
    pub min_passing_neighbors: u32,
}
