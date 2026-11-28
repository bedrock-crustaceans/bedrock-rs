use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraPresets<V: ProtoVersion> {
    pub presets: Vec<V::CameraPreset>,
}
