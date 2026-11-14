use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 84, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ResourcePackChunkRequestPacket {
    pub resource_name: String,
    #[endianness(le)]
    pub chunk: u32,
}
