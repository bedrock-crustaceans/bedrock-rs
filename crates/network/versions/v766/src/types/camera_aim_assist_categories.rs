use bedrock_network_version_traits::ProtoVersion;
use bedrock_network_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistCategories<V: ProtoVersion> {
    pub identifier: String,

    pub categories: Vec<V::CameraAimAssistCategory>,
}
