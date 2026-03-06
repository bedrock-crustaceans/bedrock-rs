use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 1)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LoginPacket {
    #[endianness(be)]
    pub client_network_version: i32,
    pub connection_request: String,
}
