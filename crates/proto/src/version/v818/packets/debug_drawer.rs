use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 328)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DebugDrawerPacket<V: ProtoVersion> {
    pub shapes: Vec<V::DebugShape>,
}
