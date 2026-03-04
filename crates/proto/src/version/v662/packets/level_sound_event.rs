use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 123)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelSoundEventPacket<V: ProtoVersion> {
    pub event_id: V::LevelSoundEventType,
    #[endianness(le)]
    pub position: Vec3<f32>,
    #[endianness(var)]
    pub data: i32,
    pub actor_identifier: String,
    pub is_baby_mob: bool,
    pub is_global: bool,
}