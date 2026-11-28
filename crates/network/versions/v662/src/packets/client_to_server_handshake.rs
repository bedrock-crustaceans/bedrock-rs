use bedrock_network_macros::{packet, ProtoCodec};

#[packet(id = 4, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientToServerHandshakePacket {}
