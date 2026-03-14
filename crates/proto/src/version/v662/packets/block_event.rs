use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 26)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BlockEventPacket<V: ProtoVersion> {
    pub block_position: V::NetworkBlockPosition,
    #[endianness(var)]
    pub event_type: i32,
    #[endianness(var)]
    pub event_value: i32,
}
