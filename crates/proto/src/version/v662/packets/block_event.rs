use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 26)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BlockEventPacket<V: ProtoVersion> {
    pub block_position: V::NetworkBlockPosition,
    #[endianness(var)]
    pub event_type: i32,
    #[endianness(var)]
    pub event_value: i32,
}
