use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 300, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraInstructionPacket<V: ProtoVersion> {
    pub camera_instruction: V::CameraInstruction,
}
