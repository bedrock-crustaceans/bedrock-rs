use super::super::enums::AgentActionType;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 181)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AgentActionEventPacket {
    pub request_id: String,
    pub action_type: AgentActionType,
    pub response: String,
}