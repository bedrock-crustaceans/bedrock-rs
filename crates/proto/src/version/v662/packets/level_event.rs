use bedrockrs_macros::{packet, ProtoCodec};
use vek::Vec3;

#[packet(id = 25)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelEventPacket {
    #[endianness(var)]
    pub event_id: i32,
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(var)]
    pub data: i32,
}
