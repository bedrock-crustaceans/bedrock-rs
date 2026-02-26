use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone, Copy)]
pub struct BiomeMultinoiseGenRulesData {
    #[endianness(le)]
    pub temperature: f32,
    #[endianness(le)]
    pub humidity: f32,
    #[endianness(le)]
    pub altitude: f32,
    #[endianness(le)]
    pub weirdness: f32,
    #[endianness(le)]
    pub weight: f32,
}
