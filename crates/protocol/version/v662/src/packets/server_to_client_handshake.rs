use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 3, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerToClientHandshakePacket {
    pub handshake_web_token: String,
}

// TODO: more complex stuff
