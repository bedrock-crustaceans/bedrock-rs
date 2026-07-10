use bedrock_level::serde_helpers::deserialize_bool;

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Serialize_repr,
    serde_repr::Deserialize_repr,
)]
#[repr(i32)]
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
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct StructureBlock {
    pub redstone_save_mode: i32,
    pub structure_name: String,
    pub data_field: String,
    pub integrity: f32,
    #[serde(deserialize_with = "deserialize_bool")]
    pub is_powered: bool,
    pub seed: i64,
    #[serde(deserialize_with = "deserialize_bool")]
    pub show_bounding_box: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub include_players: bool,

    #[serde(rename = "lastTouchedPlayerId")]
    pub last_touched_player_id: i64,

    #[serde(deserialize_with = "deserialize_bool")]
    pub mirror: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub remove_blocks: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub ignore_entities: bool,

    pub rotation: i8,
    #[serde(rename = "data")]
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
