use crate::version::versions::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 300)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraInstructionPacket<V: ProtoVersion> {
    pub camera_instruction: V::CameraInstruction,
}
