use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 27)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ActorEventPacket<V: ProtoVersion> {
    pub target_runtime_id: V::ActorRuntimeID,
    pub event_id: V::ActorEvent,
    #[endianness(var)]
    pub data: i32,
}
