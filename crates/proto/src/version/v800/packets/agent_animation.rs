use super::super::types::ActorRuntimeID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 304)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AgentAnimationPacket {
    pub agent_animation: i8,
    pub runtime_id: ActorRuntimeID,
}