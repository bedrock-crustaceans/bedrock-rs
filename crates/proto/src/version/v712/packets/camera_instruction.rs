use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 300)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraInstructionPacket<V: ProtoVersion> {
    pub camera_instruction: V::CameraInstruction,
    pub remove_target: Option<bool>,
}
