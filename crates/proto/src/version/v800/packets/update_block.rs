use super::super::types::NetworkBlockPosition;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 21)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateBlockPacket {
    pub block_position: NetworkBlockPosition,
    #[endianness(var)]
    pub block_runtime_id: u32,
    #[endianness(var)]
    pub flags: u32,
    #[endianness(var)]
    pub layer: u32,
}