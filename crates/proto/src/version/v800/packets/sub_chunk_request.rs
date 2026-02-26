use super::super::types::SubChunkPos;
use super::super::types::SubChunkPosOffset;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 175)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SubChunkRequestPacket {
    #[endianness(var)]
    pub dimension_type: i32,
    pub center_pos: SubChunkPos,
    #[vec_repr(u32)]
    #[vec_endianness(le)]
    pub sub_chunk_pos_offsets: Vec<SubChunkPosOffset>,
}