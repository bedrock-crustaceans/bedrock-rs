use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 1)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LoginPacket {
    #[endianness(be)]
    pub client_network_version: i32,
    pub connection_request: Vec<u8>, // TODO: parse auth jwt here? (changed in v818)
}
