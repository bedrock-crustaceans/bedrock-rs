use bedrockrs_macros::{gamepacket, ProtoCodec};
use crate::v800::types::CameraAimAssistCategory;
use super::super::enums::CameraAimAssistOperation;
use super::super::types::{CameraAimAssistCategories, CameraAimAssistPresetDefinition};

#[gamepacket(id = 320)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct CameraAimAssistPresetsPacket {
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub categories: Vec<CameraAimAssistCategories>,
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub category_definitions: Vec<CameraAimAssistCategory>,
    #[vec_repr(i32)]
    #[vec_endianness(var)]
    pub presets: Vec<CameraAimAssistPresetDefinition>,
    pub operation: CameraAimAssistOperation,
}