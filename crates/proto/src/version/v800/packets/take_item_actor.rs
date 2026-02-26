use bedrockrs_macros::{gamepacket, ProtoCodec};
use bedrockrs_shared::actor_runtime_id::ActorRuntimeID;

#[gamepacket(id = 17)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct TakeItemActorPacket {
    pub item_runtime_id: ActorRuntimeID,
    pub actor_runtime_id: ActorRuntimeID,
}