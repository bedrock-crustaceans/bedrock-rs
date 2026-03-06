use crate::version::versions::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct StructureEditorData<V: ProtoVersion> {
    pub structure_name: String,
    pub filtered_structure_name: String,
    pub data_field: String,
    pub include_players: bool,
    pub show_bounding_box: bool,
    pub structure_block_type: V::StructureBlockType,
    pub structure_settings: V::StructureSettings,
    pub redstone_save_mode: V::StructureRedstoneSaveMode,
}
