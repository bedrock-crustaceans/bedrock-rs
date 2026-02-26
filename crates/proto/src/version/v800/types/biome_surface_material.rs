use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeSurfaceMaterialData {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    #[endianness(le)]
    pub top_block_runtime_ids: Vec<i32>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    #[endianness(le)]
    pub mid_block_runtime_ids: Vec<i32>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    #[endianness(le)]
    pub sea_floor_block_runtime_ids: Vec<i32>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    #[endianness(le)]
    pub foundation_block_runtime_ids: Vec<i32>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    #[endianness(le)]
    pub sea_block_runtime_ids: Vec<i32>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    #[endianness(le)]
    pub sea_floor_depth_runtime_ids: Vec<i32>,
}
