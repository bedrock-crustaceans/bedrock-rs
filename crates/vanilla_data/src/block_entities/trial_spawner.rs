use facet::Facet;

// #[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
// #[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
// pub struct SpawnPotentialEntity {
//     pub id: String,
// }

// #[derive(Debug, Clone, PartialEq, Facet)]
// #[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
// pub struct SpawnPotentialData {
//     pub entity: SpawnPotentialEntity,
//     pub equipment: Option<Equipment>,
// }

// #[derive(Debug, Clone, PartialEq, Facet)]
// #[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
// pub struct SpawnPotential {
//     pub weight: i32,
//     pub data: SpawnPotentialData,
// }

// #[derive(Debug, Clone, PartialEq, Facet)]
// #[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
// pub struct EjectableLootTable {
//     pub weight: i32,
//     /// Path to a loot table.
//     pub data: String,
// }

// #[derive(Debug, Clone, PartialEq, Facet)]
// #[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
// pub struct SpawnerConfig {
//     pub spawn_range: i32,
//     pub total_mobs: f32,
//     pub simultaneous_mobs: f32,
//     pub total_mobs_added_per_player: f32,
//     pub simultaneous_mobs_added_per_player: f32,
//     pub ticks_between_spawn: i32,
//     pub target_cooldown_length: i32,
//     pub spawn_potentials: Vec<SpawnPotential>,
//     pub loot_tables_to_eject: Vec<EjectableLootTable>,
//     /// A path to a loot table.
//     pub items_to_drop_when_ominuous: String,
// }

/// Chances for the equipment to drop upon death.
#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct DropChances {
    pub head: f32,
    pub chest: f32,
    pub legs: f32,
    pub feet: f32,
    pub mainhand: f32,
    pub offhand: f32,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Equipment {
    pub loot_table: String,
    pub slot_drop_chances: Option<DropChances>,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct SpawnData {
    pub type_id: String,
    pub weight: i32,
    #[facet(rename = "equipment")]
    pub equipment: Option<Equipment>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Facet)]
// Never denied unknown keys under serde; nbtx 4.0 denies by default.
#[facet(nbtx::allow_unknown_fields)]
pub struct RegisteredEntity {
    pub uuid: i64,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct TrialSpawner {
    pub ominous_config: String,
    pub spawn_data: SpawnData,
    pub total_mobs_spawned: i32,
    pub required_player_range: i32,
    pub current_mobs: Vec<RegisteredEntity>,
    pub next_mob_spawns_at: i64,
    pub registered_players: Vec<RegisteredEntity>,
    pub normal_config: String,
    pub cooldown_end_at: i64,
}
