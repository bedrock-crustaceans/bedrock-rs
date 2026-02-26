use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeCappedSurfaceData {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    #[endianness(le)]
    pub floor_blocks: Vec<i32>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    #[endianness(le)]
    pub ceiling_blocks: Vec<i32>,
    #[endianness(le)]
    pub sea_block: Option<i32>,
    #[endianness(le)]
    pub foundation_block: Option<i32>,
    #[endianness(le)]
    pub beach_block: Option<i32>,
}
