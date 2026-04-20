use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeNoiseGradientSurfaceData {
    #[endianness(le)]
    pub non_replaceable_block_runtime_ids: Vec<i32>,
    #[endianness(le)]
    pub gradient_block_runtime_ids: Vec<i32>,
}
