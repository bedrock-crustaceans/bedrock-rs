use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 69)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RequestChunkRadiusPacket {
    #[endianness(var)]
    pub chunk_radius: i32,
    pub max_chunk_radius: i8,
}
