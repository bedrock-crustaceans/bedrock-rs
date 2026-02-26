use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 3)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerToClientHandshakePacket {
    pub handshake_web_token: String,
}

// TODO: more complex stuff