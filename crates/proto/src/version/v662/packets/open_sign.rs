use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 303)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct OpenSignPacket<V: ProtoVersion> {
    pub pos: V::NetworkBlockPosition,
    pub is_front: bool,
}