use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 27)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ActorEventPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,
    pub event_id: V::ActorEvent,
    #[endianness(var)]
    pub data: i32,
}
