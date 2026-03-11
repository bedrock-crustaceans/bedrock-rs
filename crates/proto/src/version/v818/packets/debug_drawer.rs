use crate::version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 328)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DebugDrawerPacket<V: ProtoVersion> {
    pub shapes: Vec<V::DebugShape>,
}
