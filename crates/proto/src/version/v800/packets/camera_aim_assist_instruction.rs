use bedrockrs_macros::{gamepacket, ProtoCodec};
use super::super::enums::AimAssistAction;

#[gamepacket(id = 321)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistInstructionPacket {
    pub preset_id: String,
    pub action: AimAssistAction,
    pub allow_aim_assist: bool,
}