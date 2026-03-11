use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 335)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundDataDrivenUIReloadPacket {}
