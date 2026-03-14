use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistCategories<V: ProtoVersion> {
    pub identifier: String,

    pub categories: Vec<V::CameraAimAssistCategory>,
}
