use super::super::enums::Rotation;
use super::super::types::BlockPos;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 194)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct GameTestRequestPacket {
    #[endianness(var)]
    pub max_tests_per_batch: i32,
    #[endianness(var)]
    pub repeat_count: i32,
    pub rotation: Rotation,
    pub stop_on_failure: bool,
    pub test_pos: BlockPos,
    #[endianness(var)]
    pub tests_per_row: i32,
    pub test_name: String,
}