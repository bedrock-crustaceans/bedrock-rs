use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 320, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPresetsPacket<V: ProtoVersion> {
    pub category_definitions: Vec<V::CameraAimAssistCategory>,
    pub presets: Vec<V::CameraAimAssistPresetDefinition>,
    pub operation: V::CameraAimAssistOperation,
}
