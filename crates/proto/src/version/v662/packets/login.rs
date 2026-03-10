use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 1)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct LoginPacket {
    #[endianness(be)]
    pub client_network_version: i32,
    pub connection_request: String, // TODO: parse auth jwt here? (changed in v818)
}
