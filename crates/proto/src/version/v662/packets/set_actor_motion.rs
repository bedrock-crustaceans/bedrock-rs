use bedrockrs_macros::{gamepacket, ProtoCodec};
use vek::Vec3;
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 40)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetActorMotionPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,
    #[endianness(le)]
    pub motion: Vec3<f32>,
    #[endianness(var)]
    pub server_tick: u64,
}