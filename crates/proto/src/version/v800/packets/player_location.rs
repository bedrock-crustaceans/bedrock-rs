use bedrockrs_macros::{ProtoCodec, packet};
use vek::Vec3;

#[packet(id = 326)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerLocationPacket {
    pub update: PlayerLocationType,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum PlayerLocationType {
    Coordinates {
        #[endianness(var)]
        target_entity_id: i64,
        #[endianness(le)]
        position: Vec3<f32>,
    } = 0,
    Hide {
        #[endianness(var)]
        target_entity_id: i64,
    } = 1,
}
