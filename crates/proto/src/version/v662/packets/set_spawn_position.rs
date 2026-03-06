use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 43)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetSpawnPositionPacket<V: ProtoVersion> {
    pub spawn_position_type: V::SpawnPositionType,
    pub block_position: V::NetworkBlockPosition,
    #[endianness(var)]
    pub dimension_type: i32,
    pub spawn_block_pos: V::NetworkBlockPosition,
}
