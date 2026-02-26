use bedrockrs_macros::ProtoCodec;
    
use std::vec::Vec;

#[derive(ProtoCodec, Debug, Clone)]
pub struct BiomeSurfaceMaterialData {
    #[endianness(le)]
    pub top_block_runtime_ids: Vec<u32>,
    #[endianness(le)]
    pub mid_block_runtime_ids: Vec<u32>,
    #[endianness(le)]
    pub sea_floor_block_runtime_ids: Vec<u32>,
    #[endianness(le)]
    pub foundation_block_runtime_ids: Vec<u32>,
    #[endianness(le)]
    pub sea_block_runtime_ids: Vec<u32>,
    #[endianness(le)]
    pub sea_floor_depth_runtime_ids: Vec<u32>,
}