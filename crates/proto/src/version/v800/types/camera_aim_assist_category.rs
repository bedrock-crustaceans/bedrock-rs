use bedrockrs_macros::ProtoCodec;
use super::super::types::CameraAimAssistPriority;

#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistCategory {
    pub name: String,
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub entity_priorities: Vec<CameraAimAssistPriority>,
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub block_priorities: Vec<CameraAimAssistPriority>,
    #[endianness(le)]
    pub entity_default_priorities: Option<i32>,
    #[endianness(le)]
    pub block_default_priorities: Option<i32>,
}