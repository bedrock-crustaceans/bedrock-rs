use super::super::enums::ActorBlockSyncMessageID;
use super::super::types::NetworkBlockPosition;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 110)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateBlockSyncedPacket {
    pub block_position: NetworkBlockPosition,
    #[endianness(var)]
    pub block_runtime_id: u32,
    #[endianness(var)]
    pub flags: u32,
    #[endianness(var)]
    pub later: u32,
    #[endianness(var)]
    pub unique_actor_id: u64,
    pub actor_sync_message: ActorBlockSyncMessageID,
}