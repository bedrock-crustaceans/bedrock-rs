use super::super::types::NetworkBlockPosition;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 303)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct OpenSignPacket {
    pub pos: NetworkBlockPosition,
    pub is_front: bool,
}