use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone, Copy)]
pub struct BiomeSurfaceMaterialData {
    #[endianness(le)]
    pub top_block: i32,
    #[endianness(le)]
    pub mid_block: i32,
    #[endianness(le)]
    pub sea_floor_block: i32,
    #[endianness(le)]
    pub foundation_block: i32,
    #[endianness(le)]
    pub sea_block: i32,
    #[endianness(le)]
    pub sea_floor_depth: i32,
}
