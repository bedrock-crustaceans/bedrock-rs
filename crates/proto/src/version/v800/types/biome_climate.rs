use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug, Copy)]
pub struct BiomeClimateData {
    #[endianness(le)]
    pub temperature: f32,
    #[endianness(le)]
    pub downfall: f32,
    #[endianness(le)]
    pub red_spore_density: f32,
    #[endianness(le)]
    pub blue_spore_density: f32,
    #[endianness(le)]
    pub ash_density: f32,
    #[endianness(le)]
    pub white_ash_density: f32,
    #[endianness(le)]
    pub snow_accumulation_min: f32,
    #[endianness(le)]
    pub snow_accumulation_max: f32,
}
