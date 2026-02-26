use bedrockrs_macros::ProtoCodec;
use super::super::types::CameraAimAssistCategory;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistCategories {
    pub identifier: String,
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub categories: Vec<CameraAimAssistCategory>
}