use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeCappedSurfaceData {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub floor_block_runtime_ids: Vec<u32>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub ceiling_block_runtime_ids: Vec<u32>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub sea_block_runtime_ids: Vec<u32>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub foundation_block_runtime_ids: Vec<u32>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub beach_block_runtime_ids: Vec<u32>,
}