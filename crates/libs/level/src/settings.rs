use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use facet::Facet;
use nbtx::Error;
use std::io::{Read, Write};

#[derive(Facet, Debug, PartialEq)]
#[facet(deny_unknown_fields)]
pub struct Abilities {
    #[facet(rename = "attackmobs")]
    pub attack_mobs: bool,
    #[facet(rename = "attackplayers")]
    pub attack_players: bool,
    pub build: bool,
    #[facet(rename = "doorsandswitches")]
    pub doors_and_switches: bool,
    pub flying: bool,
    #[facet(rename = "instabuild")]
    pub instant_build: bool,
    pub invulnerable: bool,
    pub lightning: bool,
    pub mayfly: bool,
    pub mine: bool,
    pub op: bool,
    #[facet(rename = "opencontainers")]
    pub open_containers: bool,
    pub teleport: bool,
    #[facet(rename = "flySpeed")]
    pub fly_speed: f32,
    #[facet(rename = "verticalFlySpeed")]
    pub vertical_fly_speed: f32,
    #[facet(rename = "walkSpeed")]
    pub walk_speed: f32,
}

#[derive(Facet, Debug, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct Experiments {
    pub experiments_ever_used: bool,
    pub saved_with_toggled_experiments: bool,
}

#[derive(Facet, Debug, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct Policies {
    // Not sure what is supposed to be in here
}

#[derive(Facet, Debug, PartialEq)]
#[facet(rename_all = "camelCase")]
#[facet(deny_unknown_fields)]
pub struct LevelSettings {
    #[facet(default)]
    #[facet(skip_serializing)]
    pub file_version: u32,
    pub editor_world_type: i32,
    #[facet(rename = "isCreatedInEditor")]
    pub created_in_editor: bool,
    #[facet(rename = "isExportedFromEditor")]
    pub exported_from_editor: bool,
    #[facet(rename = "isRandomSeedAllowed")]
    pub random_seed_allowed: bool,
    #[facet(rename = "playerssleepingpercentage")]
    pub sleeping_percentage: i32,
    #[facet(rename = "recipesunlock")]
    pub recipes_unlock: bool,
    pub cheats_enabled: bool,
    pub lightning_level: f32,
    pub lightning_time: i32,
    pub rain_level: f32,
    pub rain_time: i32,
    #[facet(rename = "Difficulty")]
    pub difficulty: i32,
    #[facet(rename = "GameType")]
    pub game_mode: i32,
    #[facet(rename = "Generator")]
    pub generator: i32,
    #[facet(rename = "LimitedWorldOriginX")]
    pub limited_world_origin_x: i32,
    #[facet(rename = "LimitedWorldOriginY")]
    pub limited_world_origin_y: i32,
    #[facet(rename = "LimitedWorldOriginZ")]
    pub limited_world_origin_z: i32,
    pub limited_world_depth: i32,
    pub limited_world_width: i32,
    #[facet(rename = "MinimumCompatibleClientVersion")]
    pub minimum_compatible_client_version: [i32; 5],
    // pub minimum_compatible_client_version: f32,
    #[facet(rename = "NetherScale")]
    pub nether_scale: i32,
    #[facet(rename = "NetworkVersion")]
    pub network_version: i32,
    #[facet(rename = "Platform")]
    pub platform: i32,
    #[facet(rename = "PlatformBroadcastIntent")]
    pub platform_broadcast_intent: i32,
    #[facet(rename = "RandomSeed")]
    pub random_seed: i64,
    #[facet(rename = "SpawnV1Villagers")]
    pub spawn_v1_villagers: bool,
    #[facet(rename = "SpawnX")]
    pub spawn_x: i32,
    #[facet(rename = "SpawnY")]
    pub spawn_y: i32,
    #[facet(rename = "SpawnZ")]
    pub spawn_z: i32,
    #[facet(rename = "StorageVersion")]
    pub storage_version: i32,
    #[facet(rename = "Time")]
    pub time: i64,
    #[facet(rename = "WorldVersion")]
    pub world_version: i32,
    #[facet(rename = "XBLBroadcastIntent")]
    pub xbox_broadcast_intent: i32,
    pub current_tick: i64,
    pub experiments: Experiments,
    pub abilities: Abilities,
    pub edu_offer: i32,
    pub education_features_enabled: bool,
    #[facet(rename = "lastOpenedWithVersion")]
    pub last_opened_with_version: [i32; 5],
    pub bonus_chest_enabled: bool,
    pub bonus_chest_spawned: bool,
    #[facet(rename = "commandblockoutput")]
    pub command_block_output: bool,
    #[facet(rename = "CenterMapsToOrigin")]
    pub center_maps_to_origin: bool,
    #[facet(rename = "commandblocksenabled")]
    pub command_blocks_enabled: bool,
    pub commands_enabled: bool,
    #[facet(rename = "ConfirmedPlatformLockedContent")]
    pub confirmed_platform_locked_content: bool,
    pub daylight_cycle: i32,
    #[facet(rename = "dodaylightcycle")]
    pub daylight_lock: bool,
    #[facet(rename = "dolimitedcrafting")]
    pub limited_crafting: bool,
    #[facet(rename = "doentitydrops")]
    pub entity_drops: bool,
    #[facet(rename = "dofiretick")]
    pub fire_tick: bool,
    #[facet(rename = "doimmediaterespawn")]
    pub immediate_respawn: bool,
    #[facet(rename = "doinsomnia")]
    pub insomnia: bool,
    #[facet(rename = "domobloot")]
    pub mob_loot: bool,
    #[facet(rename = "domobspawning")]
    pub mob_spawning: bool,
    #[facet(rename = "dotiledrops")]
    pub tile_drops: bool,
    #[facet(rename = "doweathercycle")]
    pub weather_cycle: bool,
    #[facet(rename = "drowningdamage")]
    pub drowning_damage: bool,
    #[facet(rename = "falldamage")]
    pub fall_damage: bool,
    #[facet(rename = "firedamage")]
    pub fire_damage: bool,
    #[facet(rename = "freezedamage")]
    pub freeze_damage: bool,
    #[facet(rename = "keepinventory")]
    pub keep_inventory: bool,
    #[facet(rename = "maxcommandchainlength")]
    pub max_command_chain_length: i32,
    #[facet(rename = "mobgriefing")]
    pub mob_griefing: bool,
    #[facet(rename = "naturalregeneration")]
    pub natural_regeneration: bool,
    #[facet(rename = "functioncommandlimit")]
    pub function_command_limit: i32,
    pub pvp: bool,
    #[facet(rename = "randomtickspeed")]
    pub random_tick_speed: i32,
    #[facet(rename = "respawnblocksexplode")]
    pub respawn_blocks_explode: bool,
    #[facet(rename = "sendcommandfeedback")]
    pub send_command_feedback: bool,
    #[facet(rename = "showbordereffect")]
    pub show_border_effect: bool,
    #[facet(rename = "showcoordinates")]
    pub show_coordinates: bool,
    #[facet(rename = "showdeathmessages")]
    pub show_death_messages: bool,
    #[facet(rename = "showtags")]
    pub show_tags: bool,
    #[facet(rename = "spawnradius")]
    pub spawn_radius: i32,
    #[facet(rename = "tntexplodes")]
    pub tnt_explodes: bool,
    #[facet(rename = "ForceGameType")]
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
    #[facet(rename = "LANBroadcast")]
    pub lan_broadcast: bool,
    #[facet(rename = "LANBroadcastIntent")]
    pub lan_broadcast_intent: i8,
    #[facet(rename = "MultiplayerGame")]
    pub multiplayer_game: bool,
    #[facet(rename = "MultiplayerGameIntent")]
    pub multiplayer_game_intent: i8,
    #[facet(rename = "LastPlayed")]
    pub last_played: i64,
    pub base_game_version: String,
    #[facet(rename = "BiomeOverride")]
    pub biome_override: String,
    #[facet(rename = "FlatWorldLayers")]
    pub flat_world_layers: String,
    #[facet(rename = "InventoryVersion")]
    pub inventory_version: String,
    #[facet(rename = "LevelName")]
    pub level_name: String,
    pub use_msa_gamertags_only: bool,
    pub world_start_count: i64,
    pub start_with_map_enabled: bool,
    pub spawn_mobs: bool,
    pub server_chunk_tick_range: i32,
    pub permissions_level: i32,
    pub player_permissions_level: i32,
    pub prid: String,
    #[facet(rename = "world_policies")]
    pub world_policies: Policies,
    #[facet(rename = "HasUncompleteWorldFileOnDisk")]
    pub has_uncomplete_world_file_on_disk: bool,
    #[facet(rename = "IsHardcore")]
    pub is_hardcore: bool,
    #[facet(rename = "PlayerHasDied")]
    pub player_has_died: bool,
    #[facet(rename = "allowAnonymousBlockDropsInEditorWorlds")]
    pub allow_anonymous_block_drops_in_editor_worlds: bool,
    #[facet(rename = "playerwaypoints")]
    pub player_waypoints: i32,
    #[facet(rename = "projectilescanbreakblocks")]
    pub projectiles_can_break_blocks: bool,
    #[facet(rename = "serverEditorConnectionPolicy")]
    pub server_editor_connection_policy: i32,
    #[facet(rename = "showdaysplayed")]
    pub show_days_played: bool,
    #[facet(rename = "showrecipemessages")]
    pub show_recipe_messages: bool,
    #[facet(rename = "tntexplosiondropdecay")]
    pub tnt_explosion_drop_decay: bool,
}

impl LevelSettings {
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        let settings = nbtx::to_le_bytes(self)?;

        writer.write_u32::<LittleEndian>(self.file_version)?;
        writer.write_u32::<LittleEndian>(settings.len() as u32)?;
        writer.write_all(&settings)?;

        Ok(())
    }

    pub fn read<R: Read>(mut data: R) -> Result<Self, Error> {
        let file_version = data.read_u32::<LittleEndian>()?;
        let _file_size = data.read_u32::<LittleEndian>()?;

        let mut settings: Self = nbtx::from_le_bytes::<LevelSettings>(&mut data)?;

        settings.file_version = file_version;

        Ok(settings)
    }
}
