use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 186, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ToastRequestPacket {
    pub title: String,
    pub content: String,
}
