use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 321)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistInstructionPacket<V: ProtoVersion> {
    pub preset_id: String,
    pub action: V::AimAssistAction,
    pub allow_aim_assist: bool,
}
