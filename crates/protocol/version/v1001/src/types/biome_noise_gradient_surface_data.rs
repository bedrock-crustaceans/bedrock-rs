use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeNoiseGradientSurfaceData {
    #[endianness(le)]
    pub non_replaceable_block_runtime_ids: Vec<i32>,
    pub gradient_blocks: Vec<NoiseBlockSpecifier>,
    pub noise: String,
    #[endianness(le)]
    pub first_octave: i32,
    #[endianness(le)]
    pub amplitudes: Vec<f32>,
}

#[derive(ProtoCodec, Debug, Clone)]
pub struct NoiseBlockSpecifier {
    pub noise: String,
    #[endianness(le)]
    pub threshold: f32,
    #[endianness(le)]
    pub range_min: f32,
    #[endianness(le)]
    pub range_max: f32,
    #[endianness(le)]
    pub block_runtime_id: i32,
}
