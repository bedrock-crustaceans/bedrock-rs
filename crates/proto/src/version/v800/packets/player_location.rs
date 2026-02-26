use super::super::enums::PlayerLocationType;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;

#[gamepacket(id = 326)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayerLocationPacket {
    pub location_type: PlayerLocationType,
    #[endianness(le)]
    pub target_entity_id: i64,
    #[endianness(le)]
    pub position: Vec3<f32>,
}