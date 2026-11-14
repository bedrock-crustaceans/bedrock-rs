use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 83, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePackChunkDataPacket {
    pub resource_name: String,
    #[endianness(le)]
    pub chunk_id: u32,
    #[endianness(le)]
    pub byte_offset: u64,
    pub chunk_data: Vec<u8>,
}
