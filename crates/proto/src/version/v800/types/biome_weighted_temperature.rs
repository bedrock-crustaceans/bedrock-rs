use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone, Copy)]
pub struct BiomeWeightedTemperatureData {
    #[endianness(le)]
    pub temperature: i32,
    #[endianness(le)]
    pub weight: i64,
}