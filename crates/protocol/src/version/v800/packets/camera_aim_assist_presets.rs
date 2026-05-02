use crate::version::versions::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 320)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPresetsPacket<V: ProtoVersion> {
    pub category_definitions: Vec<V::CameraAimAssistCategory>,
    pub presets: Vec<V::CameraAimAssistPresetDefinition>,
    pub operation: V::CameraAimAssistOperation,
}
