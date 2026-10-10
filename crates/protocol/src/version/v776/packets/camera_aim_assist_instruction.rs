use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 321, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistInstructionPacket<V: ProtoVersion> {
    pub preset_id: String,
    pub action: V::AimAssistAction,
    pub allow_aim_assist: bool,
}
