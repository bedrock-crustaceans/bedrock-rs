use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 34)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BlockPickRequestPacket<V: ProtoVersion> {
    pub position: V::BlockPos,
    pub with_data: bool,
    pub max_slots: i8,
}
