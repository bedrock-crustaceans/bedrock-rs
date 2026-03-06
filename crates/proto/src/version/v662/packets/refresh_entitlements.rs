use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 305)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RefreshEntitlementsPacket {}
