use crate::version::versions::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraPresets<V: ProtoVersion> {
    pub presets: Vec<V::CameraPreset>,
}
