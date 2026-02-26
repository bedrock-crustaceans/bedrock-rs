use super::super::enums::{StructureBlockType, StructureRedstoneSaveMode};
use super::super::types::StructureSettings;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct StructureEditorData {
    pub structure_name: String,
    pub filtered_structure_name: String,
    pub data_field: String,
    pub include_players: bool,
    pub show_bounding_box: bool,
    pub structure_block_type: StructureBlockType,
    pub structure_settings: StructureSettings,
    pub redstone_save_mode: StructureRedstoneSaveMode,
}
