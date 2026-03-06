use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 318)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct MovementEffectPacket<V: ProtoVersion> {
    pub entity_runtime_id: V::ActorRuntimeID,
    pub effect_type: V::MovementEffectType,
    #[endianness(var)]
    pub duration: u32,
    #[endianness(var)]
    pub tick: u64,
}
