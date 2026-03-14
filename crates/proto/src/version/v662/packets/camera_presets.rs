use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 198)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraPresetsPacket<V: ProtoVersion> {
    pub camera_presets: V::CameraPresets,
}
