use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 186)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ToastRequestPacket {
    pub title: String,
    pub content: String,
}
