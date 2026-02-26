use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 102)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerSettingsRequestPacket {}