#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum StructureBlockMode {
    Data = 0,
    Save = 1,
    Load = 2,
    Corner = 3,
    Inventory = 4,
    Export = 5,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructureBlock {
    pub animation_mode: i8,
    pub animation_seconds: f32,
    pub data: StructureBlockMode,
    pub data_field: String,
    pub ignore_entities: bool,
    pub integrity: f32,
    pub is_powered: bool,
    pub mirror: i8,
    pub redstone_save_mode: i32,
    pub remove_blocks: i8,
    pub rotation: i8,
    pub seed: i64,
    pub show_bounding_box: bool,
    pub structure_name: String,
    pub x_structure_offset: i32,
    pub y_structure_offset: i32,
    pub z_structure_offset: i32,
    pub x_structure_size: i32,
    pub y_structure_size: i32,
    pub z_structure_size: i32,
}
