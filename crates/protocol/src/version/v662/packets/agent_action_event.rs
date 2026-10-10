use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 181, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AgentActionEventPacket<V: ProtoVersion> {
    pub request_id: String,
    pub action_type: V::AgentActionType,
    pub response: String,
}
