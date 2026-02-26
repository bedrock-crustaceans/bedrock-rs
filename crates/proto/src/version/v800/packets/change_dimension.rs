use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;

#[gamepacket(id = 61)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ChangeDimensionPacket {
    #[endianness(var)]
    pub dimension_id: i32,
    #[endianness(le)]
    pub position: Vec3<f32>,
    pub respawn: bool,
    #[endianness(le)]
    pub loading_screen_id: Option<i32>
}