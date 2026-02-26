use super::super::enums::PlayStatus;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 2)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PlayStatusPacket {
    pub status: PlayStatus,
}