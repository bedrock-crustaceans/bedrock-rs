use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug, Copy)]
pub struct BiomeClimateData {
    #[endianness(le)]
    pub temperature: f32,
    #[endianness(le)]
    pub downfall: f32,
    #[endianness(le)]
    pub snow_accumulation_min: f32,
    #[endianness(le)]
    pub snow_accumulation_max: f32,
}
