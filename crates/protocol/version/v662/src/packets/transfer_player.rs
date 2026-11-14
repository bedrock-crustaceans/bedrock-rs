use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 85, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct TransferPlayerPacket {
    pub server_address: String,
    #[endianness(le)]
    pub server_port: u16,
}
