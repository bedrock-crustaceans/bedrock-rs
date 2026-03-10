use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraPresets<V: ProtoVersion> {
    pub presets: Vec<V::CameraPreset>,
}
