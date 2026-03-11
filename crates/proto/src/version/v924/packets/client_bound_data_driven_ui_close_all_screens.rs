use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 334)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDataDrivenUICloseAllScreensPacket {}
