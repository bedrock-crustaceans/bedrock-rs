use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 305)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct RefreshEntitlementsPacket {}