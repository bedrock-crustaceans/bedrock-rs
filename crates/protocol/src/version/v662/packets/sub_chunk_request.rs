use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 175)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SubChunkRequestPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub dimension_type: i32,
    pub center_pos: V::SubChunkPos,
    pub sub_chunk_pos_offsets: Vec<V::SubChunkPosOffset>,
}
