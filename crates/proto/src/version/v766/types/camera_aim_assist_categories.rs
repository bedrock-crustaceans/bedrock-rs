use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistCategories<V: ProtoVersion> {
    pub identifier: String,
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub categories: Vec<V::CameraAimAssistCategory>,
}
