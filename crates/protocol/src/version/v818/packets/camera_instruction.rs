use crate::ProtoVersion;
use crate::version::v818::types::{SetInstruction, FadeInstruction, TargetInstruction};
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 300, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraInstructionPacket<V: ProtoVersion> {
    pub set: Option<SetInstruction<V>>,
    pub clear: Option<bool>,
    pub fade: Option<FadeInstruction>,
    pub target: Option<TargetInstruction<V>>,
    pub remove_target: Option<bool>,
}
