use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct BiomeReplacementData {
    #[endianness(le)]
    pub biome: i16,
    #[endianness(le)]
    pub dimension: i16,
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    #[endianness(le)]
    pub target_biomes: Vec<i16>,
    #[endianness(le)]
    pub amount: f32,
    #[endianness(le)]
    pub noise_frequency_scale: f32,
    #[endianness(le)]
    pub replacement_index: i32,
}
