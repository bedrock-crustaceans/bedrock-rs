use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 310)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientBoundCloseFormPacket {}
