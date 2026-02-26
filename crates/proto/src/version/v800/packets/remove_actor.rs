use super::super::types::ActorUniqueID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 14)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RemoveActorPacket {
    pub target_actor_id: ActorUniqueID,
}
