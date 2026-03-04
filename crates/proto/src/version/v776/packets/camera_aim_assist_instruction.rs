use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 321)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistInstructionPacket<V: ProtoVersion> {
    pub preset_id: String,
    pub action: V::AimAssistAction,
    pub allow_aim_assist: bool,
}
