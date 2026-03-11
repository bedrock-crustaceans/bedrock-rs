use bedrockrs_macros::{ProtoCodec, gamepacket};
use vek::Vec3;

#[gamepacket(id = 337)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct VoxelShapesPacket {
    pub shapes: Vec<VoxelShape>,
    pub names: Vec<VoxelShapeName>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct VoxelShape {
    pub cells: Vec<VoxelShapeCell>,
    #[endianness(le)]
    pub x_coordinates: Vec<f32>,
    #[endianness(le)]
    pub y_coordinates: Vec<f32>,
    #[endianness(le)]
    pub z_coordinates: Vec<f32>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct VoxelShapeCell {
    pub size: Vec3<u8>,
    pub storage: Vec<u8>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct VoxelShapeName {
    pub name: String,
    #[endianness(le)]
    pub index: u16,
}
