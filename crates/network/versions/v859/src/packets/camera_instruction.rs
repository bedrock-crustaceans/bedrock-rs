use bedrock_network_version_traits::ProtoVersion;
use crate::types::{SetInstruction, FadeInstruction, TargetInstruction, FieldOfViewInstruction, SplineInstruction, AttachInstruction};
use bedrock_network_macros::{ProtoCodec, packet};

#[packet(id = 300, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraInstructionPacket<V: ProtoVersion> {
    pub set: Option<SetInstruction<V>>,
    pub clear: Option<bool>,
    pub fade: Option<FadeInstruction>,
    pub target: Option<TargetInstruction<V>>,
    pub remove_target: Option<bool>,
    pub field_of_view: Option<FieldOfViewInstruction>,
    pub spline: Option<SplineInstruction<V>>,
    pub attach: Option<AttachInstruction>,
    pub detach: Option<bool>,
}
