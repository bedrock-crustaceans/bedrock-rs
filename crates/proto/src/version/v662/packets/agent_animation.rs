use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 304)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AgentAnimationPacket<V: ProtoVersion> {
    pub agent_animation: i8,
    pub runtime_id: V::ActorRuntimeID,
}
