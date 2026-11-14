use bedrock_protocol_version::ProtoVersion;
use crate::types::{SetInstruction, FadeInstruction};
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 300, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraInstructionPacket<V: ProtoVersion> {
    pub set: Option<SetInstruction<V>>,
    pub clear: Option<bool>,
    pub fade: Option<FadeInstruction>,
}
