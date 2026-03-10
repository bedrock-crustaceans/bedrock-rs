use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 94)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SubClientLoginPacket {
    pub connection_request: String, // TODO: parse auth jwt here? (changed in v818)
}
