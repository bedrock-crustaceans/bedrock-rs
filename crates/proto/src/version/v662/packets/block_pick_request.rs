use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 34)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BlockPickRequestPacket<V: ProtoVersion> {
    pub position: V::BlockPos,
    pub with_data: bool,
    pub max_slots: i8,
}
