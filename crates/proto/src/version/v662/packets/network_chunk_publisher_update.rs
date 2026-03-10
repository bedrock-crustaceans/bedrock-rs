use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 121)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct NetworkChunkPublisherUpdatePacket<V: ProtoVersion> {
    pub new_view_position: V::BlockPos,
    #[endianness(var)]
    pub new_view_radius: u32,
    #[endianness(le)]
    pub server_built_chunks_size: u32,

    pub server_built_chunks_list: Vec<V::ChunkPos>,
}
