use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 3)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerToClientHandshakePacket {
    pub handshake_web_token: String,
}

// TODO: more complex stuff
