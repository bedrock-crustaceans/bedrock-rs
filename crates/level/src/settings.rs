use crate::error::Result;
use crate::version::GameVersion;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Write};

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Abilities {
    #[serde(rename = "attackmobs")]
    pub attack_mobs: bool,
    #[serde(rename = "attackplayers")]
    pub attack_players: bool,
    pub build: bool,
    #[serde(rename = "doorsandswitches")]
    pub doors_and_switches: bool,
    pub flying: bool,
    #[serde(rename = "instabuild")]
    pub instant_build: bool,
    pub invulnerable: bool,
    pub lightning: bool,
    pub mayfly: bool,
    pub mine: bool,
    pub op: bool,
    #[serde(rename = "opencontainers")]
    pub open_containers: bool,
    pub teleport: bool,
    #[serde(rename = "flySpeed")]
    pub fly_speed: f32,
    #[serde(rename = "verticalFlySpeed", default)]
    pub vertical_fly_speed: f32,
    #[serde(rename = "walkSpeed")]
    pub walk_speed: f32,
    // Absent on some worlds; permission level is tracked at the root level
    // via `permissions_level` / `player_permissions_level` in that case.
    #[serde(rename = "playerPermissionLevel", default)]
    pub permission_level: Option<i32>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Experiments {
    pub experiments_ever_used: bool,
    pub saved_with_toggled_experiments: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Policies {
    // Not sure what is supposed to be in here
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct LevelSettings {
    // The i32 storage version from the 8-byte level.dat header, not part of
    // the NBT payload itself. Kept alongside `storage_version` (the NBT
    // `StorageVersion` field) since the two are set independently and can in
    // principle disagree.
    #[serde(default)]
    pub header_storage_version: u32,
    pub editor_world_type: i32,
    #[serde(rename = "isCreatedInEditor")]
    pub created_in_editor: bool,
    #[serde(rename = "isExportedFromEditor")]
    pub exported_from_editor: bool,
    #[serde(rename = "isRandomSeedAllowed")]
    pub random_seed_allowed: bool,
    #[serde(rename = "playerssleepingpercentage")]
    pub sleeping_percentage: i32,
    #[serde(rename = "recipesunlock")]
    pub recipes_unlock: bool,
    pub cheats_enabled: bool,
    pub lightning_level: f32,
    pub lightning_time: i32,
    pub rain_level: f32,
    pub rain_time: i32,
    #[serde(rename = "Difficulty")]
    pub difficulty: i32,
    #[serde(rename = "GameType")]
    pub game_mode: i32,
    #[serde(rename = "Generator")]
    pub generator: i32,
    #[serde(rename = "LimitedWorldOriginX")]
    pub limited_world_origin_x: i32,
    #[serde(rename = "LimitedWorldOriginY")]
    pub limited_world_origin_y: i32,
    #[serde(rename = "LimitedWorldOriginZ")]
    pub limited_world_origin_z: i32,
    pub limited_world_depth: i32,
    pub limited_world_width: i32,
    // Absent on worlds saved before this field was introduced.
    #[serde(rename = "MinimumCompatibleClientVersion", default)]
    pub minimum_compatible_client_version: Option<GameVersion>,
    #[serde(rename = "NetherScale")]
    pub nether_scale: i32,
    // Protocol version of the network codec the world was last saved with.
    // Absent on worlds saved before this field was introduced.
    #[serde(rename = "NetworkVersion", default)]
    pub network_version: Option<i32>,
    #[serde(rename = "Platform")]
    pub platform: i32,
    #[serde(rename = "PlatformBroadcastIntent")]
    pub platform_broadcast_intent: i32,
    #[serde(rename = "RandomSeed")]
    pub random_seed: i64,
    #[serde(rename = "SpawnV1Villagers")]
    pub spawn_v1_villagers: bool,
    #[serde(rename = "SpawnX")]
    pub spawn_x: i32,
    #[serde(rename = "SpawnY")]
    pub spawn_y: i32,
    #[serde(rename = "SpawnZ")]
    pub spawn_z: i32,
    // The NBT `StorageVersion` field, distinct from the header's storage
    // version stored in `header_storage_version`.
    #[serde(rename = "StorageVersion")]
    pub storage_version: i32,
    #[serde(rename = "Time")]
    pub time: i64,
    // Absent on worlds saved before this field was introduced.
    #[serde(rename = "WorldVersion", default)]
    pub world_version: Option<i32>,
    #[serde(rename = "XBLBroadcastIntent")]
    pub xbox_broadcast_intent: i32,
    pub current_tick: i64,
    pub experiments: Experiments,
    pub abilities: Abilities,
    pub edu_offer: i32,
    pub education_features_enabled: bool,
    #[serde(rename = "lastOpenedWithVersion")]
    pub last_opened_with_version: GameVersion,
    pub bonus_chest_enabled: bool,
    pub bonus_chest_spawned: bool,
    #[serde(rename = "commandblockoutput")]
    pub command_block_output: bool,
    #[serde(rename = "CenterMapsToOrigin")]
    pub center_maps_to_origin: bool,
    #[serde(rename = "commandblocksenabled")]
    pub command_blocks_enabled: bool,
    pub commands_enabled: bool,
    #[serde(rename = "ConfirmedPlatformLockedContent")]
    pub confirmed_platform_locked_content: bool,
    pub daylight_cycle: i32,
    #[serde(rename = "dodaylightcycle")]
    pub daylight_lock: bool,
    #[serde(rename = "dolimitedcrafting")]
    pub limited_crafting: bool,
    #[serde(rename = "doentitydrops")]
    pub entity_drops: bool,
    #[serde(rename = "dofiretick")]
    pub fire_tick: bool,
    #[serde(rename = "doimmediaterespawn")]
    pub immediate_respawn: bool,
    #[serde(rename = "doinsomnia")]
    pub insomnia: bool,
    #[serde(rename = "domobloot")]
    pub mob_loot: bool,
    #[serde(rename = "domobspawning")]
    pub mob_spawning: bool,
    #[serde(rename = "dotiledrops")]
    pub tile_drops: bool,
    #[serde(rename = "doweathercycle")]
    pub weather_cycle: bool,
    // Absent on worlds saved before this rule was introduced.
    #[serde(rename = "projectilescanbreakblocks", default)]
    pub projectiles_can_break_blocks: bool,
    #[serde(rename = "drowningdamage")]
    pub drowning_damage: bool,
    #[serde(rename = "falldamage")]
    pub fall_damage: bool,
    #[serde(rename = "firedamage")]
    pub fire_damage: bool,
    #[serde(rename = "freezedamage")]
    pub freeze_damage: bool,
    #[serde(rename = "keepinventory")]
    pub keep_inventory: bool,
    #[serde(rename = "maxcommandchainlength")]
    pub max_command_chain_length: i32,
    #[serde(rename = "mobgriefing")]
    pub mob_griefing: bool,
    #[serde(rename = "naturalregeneration")]
    pub natural_regeneration: bool,
    #[serde(rename = "functioncommandlimit")]
    pub function_command_limit: i32,
    pub pvp: bool,
    #[serde(rename = "randomtickspeed")]
    pub random_tick_speed: i32,
    #[serde(rename = "respawnblocksexplode")]
    pub respawn_blocks_explode: bool,
    #[serde(rename = "sendcommandfeedback")]
    pub send_command_feedback: bool,
    #[serde(rename = "showbordereffect")]
    pub show_border_effect: bool,
    #[serde(rename = "showcoordinates")]
    pub show_coordinates: bool,
    #[serde(rename = "showdeathmessages")]
    pub show_death_messages: bool,
    #[serde(rename = "showtags")]
    pub show_tags: bool,
    // Absent on worlds saved before these settings were introduced.
    #[serde(rename = "showdaysplayed", default)]
    pub show_days_played: bool,
    #[serde(rename = "showrecipemessages", default)]
    pub show_recipe_messages: bool,
    #[serde(rename = "locatorbar", default)]
    pub locator_bar: bool,
    #[serde(rename = "spawnradius")]
    pub spawn_radius: i32,
    #[serde(rename = "tntexplodes")]
    pub tnt_explodes: bool,
    #[serde(rename = "tntexplosiondropdecay", default)]
    pub tnt_explosion_drop_decay: bool,
    #[serde(rename = "ForceGameType")]
    pub force_game_mode: bool,
    pub has_been_loaded_in_creative: bool,
    pub has_locked_behavior_pack: bool,
    pub has_locked_resource_pack: bool,
    pub immutable_world: bool,
    pub is_from_locked_template: bool,
    pub is_from_world_template: bool,
    pub is_single_use_world: bool,
    pub is_world_template_option_locked: bool,
    pub requires_copied_pack_removal_check: bool,
    pub texture_packs_required: bool,
    // Absent on worlds saved before these fields were introduced.
    #[serde(rename = "IsHardcore", default)]
    pub is_hardcore: bool,
    #[serde(rename = "PlayerHasDied", default)]
    pub player_has_died: bool,
    #[serde(rename = "HasUncompleteWorldFileOnDisk", default)]
    pub has_uncomplete_world_file_on_disk: bool,
    #[serde(rename = "LANBroadcast")]
    pub lan_broadcast: bool,
    #[serde(rename = "LANBroadcastIntent")]
    pub lan_broadcast_intent: i8,
    #[serde(rename = "MultiplayerGame")]
    pub multiplayer_game: bool,
    #[serde(rename = "MultiplayerGameIntent")]
    pub multiplayer_game_intent: i8,
    #[serde(rename = "LastPlayed")]
    pub last_played: i64,
    // Absent on worlds saved before this field was introduced; holds "*" or
    // a version string like "1.17" once present.
    #[serde(default)]
    pub base_game_version: Option<String>,
    #[serde(rename = "BiomeOverride")]
    pub biome_override: String,
    #[serde(rename = "FlatWorldLayers")]
    pub flat_world_layers: String,
    // Absent on worlds saved before this field was introduced.
    #[serde(rename = "InventoryVersion", default)]
    pub inventory_version: Option<String>,
    #[serde(rename = "LevelName")]
    pub level_name: String,
    pub use_msa_gamertags_only: bool,
    pub world_start_count: i64,
    pub start_with_map_enabled: bool,
    pub spawn_mobs: bool,
    pub server_chunk_tick_range: i32,
    pub permissions_level: i32,
    pub player_permissions_level: i32,
    pub prid: String,
    #[serde(rename = "world_policies")]
    pub world_policies: Policies,
    // Superseded by `PlatformBroadcastIntent` / `XBLBroadcastIntent` on
    // newer worlds, which no longer write these.
    #[serde(rename = "PlatformBroadcast", default)]
    pub platform_broadcast: Option<i8>,
    #[serde(rename = "PlatformBroadcastMode", default)]
    pub platform_broadcast_mode: Option<i32>,
    #[serde(rename = "XBLBroadcast", default)]
    pub xbl_broadcast: Option<i8>,
    #[serde(rename = "XBLBroadcastMode", default)]
    pub xbl_broadcast_mode: Option<i32>,
}

/// How a world was written, gathered from level.dat's header and NBT
/// version fields. Does not decide anything on its own; version-dispatch
/// and upgrade logic build on top of this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionInfo<'a> {
    pub header_storage_version: u32,
    pub storage_version: i32,
    pub last_opened_with_version: GameVersion,
    pub minimum_compatible_client_version: Option<GameVersion>,
    pub base_game_version: Option<&'a str>,
    pub network_version: Option<i32>,
    pub inventory_version: Option<&'a str>,
    pub world_version: Option<i32>,
}

impl LevelSettings {
    pub fn write<W: Write>(&self, _writer: W) -> Result<()> {
        todo!()
    }

    pub fn read<R: Read>(mut data: R) -> Result<Self> {
        let header_storage_version = data.read_u32::<LittleEndian>()?;
        let _file_size = data.read_u32::<LittleEndian>()?;

        let mut settings: Self = nbtx::from_le_bytes(&mut data)?;
        settings.header_storage_version = header_storage_version;

        Ok(settings)
    }

    pub fn version_info(&self) -> VersionInfo<'_> {
        VersionInfo {
            header_storage_version: self.header_storage_version,
            storage_version: self.storage_version,
            last_opened_with_version: self.last_opened_with_version,
            minimum_compatible_client_version: self.minimum_compatible_client_version,
            base_game_version: self.base_game_version.as_deref(),
            network_version: self.network_version,
            inventory_version: self.inventory_version.as_deref(),
            world_version: self.world_version,
        }
    }
}
