use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 320, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPresetsPacket<V: ProtoVersion> {
    pub categories: Vec<V::CameraAimAssistCategories>,
    pub presets: Vec<V::CameraAimAssistPresetDefinition>,
}
