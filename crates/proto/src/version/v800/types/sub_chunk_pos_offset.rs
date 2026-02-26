use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct SubChunkPosOffset {
    pub offset_x: i8,
    pub offset_y: i8,
    pub offset_z: i8,
}