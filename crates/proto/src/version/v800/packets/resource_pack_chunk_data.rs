use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 83)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePackChunkDataPacket {
    pub resource_name: String,
    #[endianness(le)]
    pub chunk_id: u32,
    #[endianness(le)]
    pub byte_offset: u64,
    pub chunk_data: String,
}