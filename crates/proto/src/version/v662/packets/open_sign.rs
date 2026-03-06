use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 303)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct OpenSignPacket<V: ProtoVersion> {
    pub pos: V::NetworkBlockPosition,
    pub is_front: bool,
}
