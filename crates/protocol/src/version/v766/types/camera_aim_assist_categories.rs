use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistCategories<V: ProtoVersion> {
    pub identifier: String,

    pub categories: Vec<V::CameraAimAssistCategory>,
}
