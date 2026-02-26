use super::super::enums::SpawnPositionType;
use super::super::types::NetworkBlockPosition;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 43)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetSpawnPositionPacket {
    pub spawn_position_type: SpawnPositionType,
    pub block_position: NetworkBlockPosition,
    #[endianness(var)]
    pub dimension_type: i32,
    pub spawn_block_pos: NetworkBlockPosition,
}