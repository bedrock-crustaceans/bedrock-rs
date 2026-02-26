use super::super::types::BlockPos;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 34)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BlockPickRequestPacket {
    pub position: BlockPos,
    pub with_data: bool,
    pub max_slots: i8,
}