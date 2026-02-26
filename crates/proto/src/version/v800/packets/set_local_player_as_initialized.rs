use super::super::types::ActorRuntimeID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 113)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetLocalPlayerAsInitializedPacket {
    pub player_id: ActorRuntimeID,
}