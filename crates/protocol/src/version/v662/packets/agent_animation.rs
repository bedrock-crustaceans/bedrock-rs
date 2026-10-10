use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 304, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AgentAnimationPacket<V: ProtoVersion> {
    pub agent_animation: i8,
    pub runtime_id: V::ActorRuntimeID,
}
