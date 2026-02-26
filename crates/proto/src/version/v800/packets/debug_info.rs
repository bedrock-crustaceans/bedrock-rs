use super::super::types::ActorUniqueID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 155)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DebugInfoPacket {
    pub actor_id: ActorUniqueID,
    pub data: String,
}