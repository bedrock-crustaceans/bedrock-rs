use super::super::types::{BlockPos, ChunkPos};
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 121)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct NetworkChunkPublisherUpdatePacket {
    pub new_view_position: BlockPos,
    #[endianness(var)]
    pub new_view_radius: u32,
    #[endianness(le)]
    pub server_built_chunks_size: u32,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub server_built_chunks_list: Vec<ChunkPos>,
}