use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeNoiseGradientSurfaceData {
    #[endianness(le)]
    pub non_replaceable_block_runtime_ids: Vec<i32>,
    #[endianness(le)]
    pub gradient_block_runtime_ids: Vec<i32>,
    pub noise_seed_string: String,
    #[endianness(le)]
    pub first_octave: i32,
    #[endianness(le)]
    pub amplitudes: Vec<f32>,
}
