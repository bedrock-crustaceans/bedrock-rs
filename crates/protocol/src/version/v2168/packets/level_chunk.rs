use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 58)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LevelChunkPacket<V: ProtoVersion> {
    pub chunk_position: V::ChunkPos,
    #[endianness(var)]
    pub dimension_id: i32,
    #[endianness(var)]
    pub sub_chunk_count: u32,
    #[endianness(var)]
    pub client_request_sub_chunk_limit: Option<i32>,
    pub cache_enabled: bool,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    #[endianness(le)]
    pub cache_blobs: Vec<u64>,
    pub serialized_chunk_data: Vec<u8>,
}
