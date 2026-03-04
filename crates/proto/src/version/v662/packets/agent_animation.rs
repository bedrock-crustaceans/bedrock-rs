use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 304)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AgentAnimationPacket<V: ProtoVersion> {
    pub agent_animation: i8,
    pub runtime_id: V::ActorRuntimeID,
}
