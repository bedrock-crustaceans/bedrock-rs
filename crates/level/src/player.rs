use crate::settings::Abilities;
use serde::{Deserialize, Serialize};
use vek::{Vec2, Vec3};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct InventoryItemWithSlot {
    pub damage: i16,
    pub slot: i8,
    #[serde(rename = "id")]
    pub id: i16,
    pub count: i8,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct InventoryItem {
    pub damage: i16,
    #[serde(rename = "id")]
    pub id: i16,
    pub count: i8,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct StatusEffect {
    pub id: i8,
    pub duration: i32,
    pub duration_easy: i32,
    pub duration_normal: i32,
    pub duration_hard: i32,
    pub ambient: bool,
    pub amplifier: i8,
    pub show_particles: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Attribute {
    pub current: f32,
    pub max: f32,
    pub name: String,
    pub base: f32,
    /// TODO: Figure out what this is supposed to be. It seems to be linked to `minecraft:attack_damage`
    pub modifiers: Option<Vec<usize>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerData {
    pub inventory_version: String,
    pub is_swimming: bool,
    pub enchantment_seed: i32,
    pub fall_distance: f32,
    pub mark_variant: i32,
    #[serde(rename = "LeasherID")]
    pub leasher_id: i64,
    #[serde(rename = "DimensionID")]
    pub dimension_id: i32,
    pub sheared: bool,
    pub sleep_timer: i16,
    pub spawn_y: i32,
    pub chested: bool,
    pub spawn_forced: bool,
    pub is_global: bool,
    pub strength: i32,
    pub cursor_selected_items: InventoryItem,
    pub surface: bool,
    pub strength_max: i32,
    pub color2: i8,
    #[serde(rename = "boundY")]
    pub bound_y: i32,
    #[serde(rename = "boundZ")]
    pub bound_z: i32,
    #[serde(rename = "limitedLife")]
    pub limited_life: i32,
    pub armor: [InventoryItem; 4],
    pub hurt_time: i16,
    pub is_gliding: bool,
    pub player_game_mode: i32,
    pub color: i8,
    pub rotation: Vec2<f32>,
    pub invulnerable: bool,
    pub is_angry: bool,
    pub active_effects: Vec<StatusEffect>,
    // pub mainhand: i16,
    pub natural_spawn: bool,
    pub death_time: i16,
    pub is_baby: bool,
    pub variant: i32,
    pub spawn_z: i32,
    pub loot_dropped: bool,
    pub selected_inventory_slot: i32,
    pub on_ground: bool,
    pub sneaking: bool,
    pub sleeping: bool,
    pub attributes: Vec<Attribute>,
    pub bed_position_z: i32,
    pub abilities: Abilities,
    #[serde(rename = "UniqueID")]
    pub unique_id: i64,
    pub definitions: Vec<String>,
    pub saddled: bool,
    pub show_bottom: bool,
    #[serde(rename = "TargetID")]
    pub target_id: i64,
    pub bed_position_y: i32,
    pub bed_position_x: i32,
    #[serde(rename = "hasBoundOrigin")]
    pub has_bound_origin: bool,
    pub is_tamed: bool,
    pub pos: Vec3<f32>,
    pub map_index: i32,
    #[serde(rename = "boundX")]
    pub bound_x: i32,
    pub fire: i16,
    pub ender_chest_inventory: [InventoryItemWithSlot; 27],
    pub is_autonomous: bool,
    pub persistent: bool,
    pub offhand: Vec<InventoryItem>,
    #[serde(rename = "SelectedContainerID")]
    pub selected_contain_id: i32,
    pub owner_new: i64,
    pub player_level: i32,
    /// Thank you Mojang.
    pub is_pregnant: bool,
    #[serde(rename = "id")]
    pub id: i32,
    pub air: i16,
    pub attack_time: i16,
    pub portal_cooldown: i32,
    pub spawn_x: i32,
    pub sitting: bool,
    pub player_level_progress: f32,
}
