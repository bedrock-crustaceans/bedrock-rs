use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 337)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct VoxelShapesPacket {
    pub shapes: Vec<VoxelShape>,
    pub names: Vec<VoxelShapeName>,
    #[endianness(le)]
    pub custom_shape_count: u16,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct VoxelShape {
    pub cells: VoxelShapeCells,
    #[endianness(le)]
    pub x_coordinates: Vec<f32>,
    #[endianness(le)]
    pub y_coordinates: Vec<f32>,
    #[endianness(le)]
    pub z_coordinates: Vec<f32>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct VoxelShapeCells {
    pub size: (u8, u8, u8),
    pub storage: Vec<u8>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct VoxelShapeName {
    pub name: String,
    #[endianness(le)]
    pub index: u16,
}
