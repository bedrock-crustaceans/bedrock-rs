use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 123)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelSoundEventPacket<V: ProtoVersion> {
    pub event_id: V::LevelSoundEventType,
    #[endianness(le)]
    pub position: (f32, f32, f32),
    #[endianness(var)]
    pub data: i32,
    pub actor_identifier: String,
    pub is_baby_mob: bool,
    pub is_global: bool,
    #[endianness(le)]
    pub entity_unique_id: u64,
    #[endianness(le)]
    pub fire_at_position: Option<(f32, f32, f32)>,
}
