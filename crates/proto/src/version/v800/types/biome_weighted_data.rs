use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone, Copy)]
pub struct BiomeWeightedData {
    #[endianness(le)]
    pub biome: u16,
    #[endianness(le)]
    pub weight: i32,
}
