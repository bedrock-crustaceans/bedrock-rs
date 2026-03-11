use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 58)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelChunkPacket<V: ProtoVersion> {
    pub chunk_position: V::ChunkPos,
    #[endianness(var)]
    pub dimension_id: i32,
    // TODO: sub-chunk count stuff
    pub cache_enabled: bool,
    pub cache_blobs: Vec<CacheBlobEntry>,
    pub serialized_chunk_data: String,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct CacheBlobEntry {
    #[endianness(var)]
    blob: u64,
}

// TODO: this whole thing is terrible
