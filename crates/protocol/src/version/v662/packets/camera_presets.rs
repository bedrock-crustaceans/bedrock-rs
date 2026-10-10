use crate::ProtoVersion;
use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 198, direction = "server_to_client")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraPresetsPacket<V: ProtoVersion> {
    pub camera_presets: V::CameraPresets,
}
