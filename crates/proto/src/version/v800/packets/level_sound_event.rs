use super::super::enums::LevelSoundEventType;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;

#[gamepacket(id = 123)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelSoundEventPacket {
    pub event_id: LevelSoundEventType,
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(var)]
    pub data: i32,
    pub actor_identifier: String,
    pub is_baby_mob: bool,
    pub is_global: bool,
    #[endianness(le)]
    pub entity_unique_id: u64,
}