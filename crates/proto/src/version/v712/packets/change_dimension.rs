use bedrockrs_macros::{ProtoCodec, packet};
use vek::Vec3;

#[packet(id = 61)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ChangeDimensionPacket {
    #[endianness(var)]
    pub dimension_id: i32,
    #[endianness(le)]
    pub position: Vec3<f32>,
    pub respawn: bool,
    #[endianness(le)]
    pub loading_screen_id: Option<i32>,
}
