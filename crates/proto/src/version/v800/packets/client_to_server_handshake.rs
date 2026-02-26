use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 4)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientToServerHandshakePacket {}