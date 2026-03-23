use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 172)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateSubChunkBlocksPacket<V: ProtoVersion> {
    pub sub_chunk_block_position: V::NetworkBlockPosition,
    pub standard_blocks_changed: Vec<BlocksChangedEntry<V>>,
    pub extra_blocks_changed: Vec<BlocksChangedEntry<V>>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct BlocksChangedEntry<V: ProtoVersion> {
    pub pos: V::NetworkBlockPosition,
    #[endianness(var)]
    pub runtime_id: u32,
    #[endianness(var)]
    pub update_flags: u32,
    #[endianness(var)]
    pub sync_message_entity_unique_id: u64,
    pub sync_message: V::ActorBlockSyncMessageID,
}
