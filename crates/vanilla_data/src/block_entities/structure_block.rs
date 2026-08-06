use facet::Facet;


/// Stored as an `Int` tag holding the numeric mode.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(nbtx::variant_as(i32))]
#[repr(i32)]
pub enum StructureBlockMode {
    Data = 0,
    Save = 1,
    Load = 2,
    Corner = 3,
    Inventory = 4,
    Export = 5,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct StructureBlock {
    pub redstone_save_mode: i32,
    pub structure_name: String,
    pub data_field: String,
    pub integrity: f32,
    pub is_powered: bool,
    pub seed: i64,
    pub show_bounding_box: bool,
    pub include_players: bool,

    #[facet(rename = "lastTouchedPlayerId")]
    pub last_touched_player_id: i64,

    pub mirror: bool,
    pub remove_blocks: bool,
    pub ignore_entities: bool,

    pub rotation: i8,
    #[facet(rename = "data")]
    pub mode: StructureBlockMode,

    pub animation_seconds: f32,
    pub animation_mode: i8,

    pub x_structure_offset: i32,
    pub y_structure_offset: i32,
    pub z_structure_offset: i32,

    pub x_structure_size: i32,
    pub y_structure_size: i32,
    pub z_structure_size: i32,
}
