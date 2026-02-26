use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeCappedSurfaceData {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    #[endianness(le)]
    pub floor_block_runtime_ids: Vec<i32>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    #[endianness(le)]
    pub ceiling_block_runtime_ids: Vec<i32>,
    #[endianness(le)]
    pub sea_block_runtime_ids: Option<i32>,
    #[endianness(le)]
    pub foundation_block_runtime_ids: Option<i32>,
    #[endianness(le)]
    pub beach_block_runtime_ids: Option<i32>,
}
