use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 121)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct NetworkChunkPublisherUpdatePacket<V: ProtoVersion> {
    pub new_view_position: V::BlockPos,
    #[endianness(var)]
    pub new_view_radius: u32,
    #[vec_repr(i32)]
    #[vec_endianness(le)]
    pub server_built_chunks: Vec<V::ChunkPos>,
}
