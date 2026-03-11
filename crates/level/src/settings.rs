use crate::error::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Write};

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
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
    #[serde(rename = "walkSpeed")]
    pub walk_speed: f32,
    #[serde(rename = "playerPermissionLevel")]
    pub permission_level: i32,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Experiments {
    pub experiments_ever_used: bool,
    pub saved_with_toggled_experiments: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Policies {
    // Not sure what is supposed to be in here
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct LevelSettings {
    #[serde(default)]
    pub file_version: u32,
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
    #[serde(rename = "MinimumCompatibleClientVersion")]
    pub minimum_compatible_client_version: [i32; 5],
    // pub minimum_compatible_client_version: f32,
    #[serde(rename = "NetherScale")]
    pub nether_scale: i32,
    #[serde(rename = "NetworkVersion")]
    pub network_version: i32,
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
    #[serde(rename = "StorageVersion")]
    pub storage_version: i32,
    #[serde(rename = "Time")]
    pub time: i64,
    #[serde(rename = "WorldVersion")]
    pub world_version: i32,
    #[serde(rename = "XBLBroadcastIntent")]
    pub xbox_broadcast_intent: i32,
    pub current_tick: i64,
    pub experiments: Experiments,
    pub abilities: Abilities,
    pub edu_offer: i32,
    pub education_features_enabled: bool,
    #[serde(rename = "lastOpenedWithVersion")]
    pub last_opened_with_version: [i32; 5],
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
    #[serde(rename = "spawnradius")]
    pub spawn_radius: i32,
    #[serde(rename = "tntexplodes")]
    pub tnt_explodes: bool,
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
    pub base_game_version: String,
    #[serde(rename = "BiomeOverride")]
    pub biome_override: String,
    #[serde(rename = "FlatWorldLayers")]
    pub flat_world_layers: String,
    #[serde(rename = "InventoryVersion")]
    pub inventory_version: String,
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
    #[serde(rename = "PlatformBroadcast")]
    pub platform_broadcast: i8,
    #[serde(rename = "PlatformBroadcastMode")]
    pub platform_broadcast_mode: i32,
    #[serde(rename = "XBLBroadcast")]
    pub xbl_broadcast: i8,
    #[serde(rename = "XBLBroadcastMode")]
    pub xbl_broadcast_mode: i32,
}

impl LevelSettings {
    pub fn write<W: Write>(&self, _writer: W) -> Result<()> {
        todo!()
    }

    pub fn read<R: Read>(mut data: R) -> Result<Self> {
        let file_version = data.read_u32::<LittleEndian>()?;
        let _file_size = data.read_u32::<LittleEndian>()?;

        println!("version: {file_version}, size: {_file_size}");

        // let mut settings: nbtx::Value = nbtx::from_le_bytes(&mut data)?;
        // println!("{settings:?}");

        let mut settings: Self = nbtx::from_le_bytes(&mut data)?;
        settings.file_version = file_version;

        Ok(settings)
    }
}
