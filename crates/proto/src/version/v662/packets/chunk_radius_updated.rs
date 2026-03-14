use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 70)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ChunkRadiusUpdatedPacket {
    #[endianness(var)]
    pub chunk_radius: i32,
}
