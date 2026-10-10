use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 328, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct DebugDrawerPacket<V: ProtoVersion> {
    pub shapes: Vec<V::DebugShape>,
}
