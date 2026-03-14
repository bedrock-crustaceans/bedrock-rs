use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 335)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDataDrivenUIReloadPacket {}
