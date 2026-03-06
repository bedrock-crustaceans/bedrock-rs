use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 102)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerSettingsRequestPacket {}
