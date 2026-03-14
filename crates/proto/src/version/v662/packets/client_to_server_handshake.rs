use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 4)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientToServerHandshakePacket {}
