use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 175)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SubChunkRequestPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub dimension_type: i32,
    #[vec_repr(u32)]
    #[vec_endianness(le)]
    pub sub_chunk_pos_offsets: Vec<V::SubChunkPosOffset>,
    #[endianness(le)]
    pub center_pos: (i32, i32, i32),
}
