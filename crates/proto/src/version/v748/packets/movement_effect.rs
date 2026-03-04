use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 318)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MovementEffectPacket<V: ProtoVersion> {
    pub entity_runtime_id: V::ActorRuntimeID,
    pub effect_type: V::MovementEffectType,
    #[endianness(var)]
    pub duration: u32,
    #[endianness(var)]
    pub tick: u64,
}
