use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 110)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateBlockSyncedPacket<V: ProtoVersion> {
    pub block_position: V::NetworkBlockPosition,
    #[endianness(var)]
    pub block_runtime_id: u32,
    #[endianness(var)]
    pub flags: u32,
    #[endianness(var)]
    pub later: u32,
    #[endianness(var)]
    pub unique_actor_id: u64,
    pub actor_sync_message: V::ActorBlockSyncMessageID,
}
