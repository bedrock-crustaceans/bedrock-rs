use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone, Copy)]
pub struct BiomeWeightedData {
    #[endianness(le)]
    pub biome: i32,
    #[endianness(le)]
    pub weight: i64,
}