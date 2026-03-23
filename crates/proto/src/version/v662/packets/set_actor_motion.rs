use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 40)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetActorMotionPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,
    #[endianness(le)]
    pub motion: (f32, f32, f32),
    #[endianness(var)]
    pub server_tick: u64,
}
