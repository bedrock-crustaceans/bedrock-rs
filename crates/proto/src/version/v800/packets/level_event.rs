use super::super::enums::LevelEvent;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;

#[gamepacket(id = 25)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelEventPacket {
    #[endianness(var)]
    pub event_id: i32,
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(var)]
    pub data: i32,
}