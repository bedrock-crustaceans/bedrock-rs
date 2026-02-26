use super::super::types::NetworkBlockPosition;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 26)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BlockEventPacket {
    pub block_position: NetworkBlockPosition,
    #[endianness(var)]
    pub event_type: i32,
    #[endianness(var)]
    pub event_value: i32,
}