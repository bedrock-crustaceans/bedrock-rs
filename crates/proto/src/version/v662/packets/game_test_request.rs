use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 194)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct GameTestRequestPacket<V: ProtoVersion> {
    #[endianness(var)]
    pub max_tests_per_batch: i32,
    #[endianness(var)]
    pub repeat_count: i32,
    pub rotation: V::Rotation,
    pub stop_on_failure: bool,
    pub test_pos: V::BlockPos,
    #[endianness(var)]
    pub tests_per_row: i32,
    pub test_name: String,
}
