use super::super::enums::ActorEvent;
use super::super::types::ActorRuntimeID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 27)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ActorEventPacket {
    pub target_runtime_id: ActorRuntimeID,
    pub event_id: ActorEvent,
    #[endianness(var)]
    pub data: i32,
}
