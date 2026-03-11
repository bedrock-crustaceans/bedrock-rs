use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 334)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDataDrivenUICloseAllScreensPacket {}
