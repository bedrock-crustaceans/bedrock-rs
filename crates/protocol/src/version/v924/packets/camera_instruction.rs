use crate::ProtoVersion;
use crate::version::v924::types::{SetInstruction, FadeInstruction, TargetInstruction, FieldOfViewInstruction, AttachInstruction};
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 300, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraInstructionPacket<V: ProtoVersion> {
    pub set: Option<SetInstruction<V>>,
    pub clear: Option<bool>,
    pub fade: Option<FadeInstruction>,
    pub target: Option<TargetInstruction<V>>,
    pub remove_target: Option<bool>,
    pub field_of_view: Option<FieldOfViewInstruction>,
    pub spline: Option<V::CameraSplineInstruction>,
    pub attach: Option<AttachInstruction>,
    pub detach: Option<bool>,
}
