use crate::error::Result;
use crate::version::GameVersion;
use byteorder::{LittleEndian, ReadBytesExt};
use facet::Facet;
use nbtx::{Compound, Value};
use std::io::{Read, Write};

#[derive(Facet, Debug, PartialEq)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
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
    #[facet(rename = "verticalFlySpeed", default)]
    pub vertical_fly_speed: f32,
    #[facet(rename = "walkSpeed")]
    pub walk_speed: f32,
    // Absent on some worlds; permission level is tracked at the root level
    // via `permissions_level` / `player_permissions_level` in that case.
    #[facet(rename = "playerPermissionLevel")]
    pub permission_level: Option<i32>,
}

#[derive(Facet, Debug, PartialEq, Eq)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Experiments {
    pub experiments_ever_used: bool,
    pub saved_with_toggled_experiments: bool,
}

#[derive(Facet, Debug, PartialEq, Eq)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Policies {
    // Not sure what is supposed to be in here
}

#[derive(Facet, Debug, PartialEq)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct LevelSettings {
    // The i32 storage version from the 8-byte level.dat header, not part of
    // the NBT payload itself. Kept alongside `storage_version` (the NBT
    // `StorageVersion` field) since the two are set independently and can in
    // principle disagree.
    //
    // `#[facet(default)]` because the key is never present in the payload; it is
    // patched in by `read` afterwards. Typed `i32` rather than `u32` because NBT
    // has no unsigned tag and nbtx refuses to encode a `u32` field at all.
    #[facet(default)]
    pub header_storage_version: i32,
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
    // Absent on worlds saved before this field was introduced.
    #[facet(rename = "MinimumCompatibleClientVersion")]
    pub minimum_compatible_client_version: Option<GameVersion>,
    #[facet(rename = "NetherScale")]
    pub nether_scale: i32,
    // Protocol version of the network codec the world was last saved with.
    // Absent on worlds saved before this field was introduced.
    #[facet(rename = "NetworkVersion")]
    pub network_version: Option<i32>,
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
    // The NBT `StorageVersion` field, distinct from the header's storage
    // version stored in `header_storage_version`.
    #[facet(rename = "StorageVersion")]
    pub storage_version: i32,
    #[facet(rename = "Time")]
    pub time: i64,
    // Absent on worlds saved before this field was introduced.
    #[facet(rename = "WorldVersion")]
    pub world_version: Option<i32>,
    #[facet(rename = "XBLBroadcastIntent")]
    pub xbox_broadcast_intent: i32,
    pub current_tick: i64,
    pub experiments: Experiments,
    pub abilities: Abilities,
    pub edu_offer: i32,
    pub education_features_enabled: bool,
    #[facet(rename = "lastOpenedWithVersion")]
    pub last_opened_with_version: GameVersion,
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
    // Absent on worlds saved before this rule was introduced.
    #[facet(rename = "projectilescanbreakblocks", default)]
    pub projectiles_can_break_blocks: bool,
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
    // Absent on worlds saved before these settings were introduced.
    #[facet(rename = "showdaysplayed", default)]
    pub show_days_played: bool,
    #[facet(rename = "showrecipemessages", default)]
    pub show_recipe_messages: bool,
    #[facet(rename = "locatorbar", default)]
    pub locator_bar: bool,
    #[facet(rename = "spawnradius")]
    pub spawn_radius: i32,
    #[facet(rename = "tntexplodes")]
    pub tnt_explodes: bool,
    #[facet(rename = "tntexplosiondropdecay", default)]
    pub tnt_explosion_drop_decay: bool,
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
    // Absent on worlds saved before these fields were introduced.
    #[facet(rename = "IsHardcore", default)]
    pub is_hardcore: bool,
    #[facet(rename = "PlayerHasDied", default)]
    pub player_has_died: bool,
    #[facet(rename = "HasUncompleteWorldFileOnDisk", default)]
    pub has_uncomplete_world_file_on_disk: bool,
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
    // Absent on worlds saved before this field was introduced; holds "*" or
    // a version string like "1.17" once present.
    pub base_game_version: Option<String>,
    #[facet(rename = "BiomeOverride")]
    pub biome_override: String,
    #[facet(rename = "FlatWorldLayers")]
    pub flat_world_layers: String,
    // Absent on worlds saved before this field was introduced.
    #[facet(rename = "InventoryVersion")]
    pub inventory_version: Option<String>,
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
    // Superseded by `PlatformBroadcastIntent` / `XBLBroadcastIntent` on
    // newer worlds, which no longer write these.
    #[facet(rename = "PlatformBroadcast")]
    pub platform_broadcast: Option<i8>,
    #[facet(rename = "PlatformBroadcastMode")]
    pub platform_broadcast_mode: Option<i32>,
    #[facet(rename = "XBLBroadcast")]
    pub xbl_broadcast: Option<i8>,
    #[facet(rename = "XBLBroadcastMode")]
    pub xbl_broadcast_mode: Option<i32>,
}

/// How a world was written, gathered from level.dat's header and NBT
/// version fields. Does not decide anything on its own; version-dispatch
/// and upgrade logic build on top of this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionInfo<'a> {
    pub header_storage_version: i32,
    pub storage_version: i32,
    pub last_opened_with_version: GameVersion,
    pub minimum_compatible_client_version: Option<GameVersion>,
    pub base_game_version: Option<&'a str>,
    pub network_version: Option<i32>,
    pub inventory_version: Option<&'a str>,
    pub world_version: Option<i32>,
}

/// The two NBT keys that hold a [`GameVersion`] as an int list rather than a
/// compound. Both are read with the key spelling level.dat uses on disk.
const GAME_VERSION_KEYS: [&str; 2] = ["lastOpenedWithVersion", "MinimumCompatibleClientVersion"];

/// Reshapes the two game-version keys from the int list they are on disk into
/// the compound [`GameVersion`]'s five fields decode from.
///
/// nbtx reflects any struct as a compound and has no hook for converting one
/// on the way in, so the reshaping is done here on the decoded tree. It stays
/// confined to this function because these are the only two places in the
/// format where a game version appears; nothing else nests one.
///
/// A key that is absent, or already a compound, or holds anything other than a
/// five-element int list is left exactly as it is — `from_value` then reports
/// the mismatch as it would have anyway.
fn unpack_game_versions(value: &mut Value) {
    let Value::Compound(entries) = value else {
        return;
    };

    for key in GAME_VERSION_KEYS {
        let Some(entry) = entries.get_mut(bstr::BStr::new(key)) else {
            continue;
        };
        // `IntArray` is what Bedrock writes; a `List` of `Int` decoded into the
        // same five ints, so accept it too.
        let parts: Vec<i32> = match entry {
            Value::IntArray(parts) => parts.clone(),
            Value::List(items) => {
                let ints: Option<Vec<i32>> = items.iter().map(|i| i.as_int().copied()).collect();
                match ints {
                    Some(ints) => ints,
                    None => continue,
                }
            }
            _ => continue,
        };
        let [major, minor, patch, revision, beta] = parts[..] else {
            continue;
        };

        *entry = Value::Compound(Compound::from_iter([
            ("major".into(), Value::Int(major)),
            ("minor".into(), Value::Int(minor)),
            ("patch".into(), Value::Int(patch)),
            ("revision".into(), Value::Int(revision)),
            ("beta".into(), Value::Int(beta)),
        ]));
    }
}

impl LevelSettings {
    /// # Note
    ///
    /// Unimplemented. A writer has to undo what [`unpack_game_versions`] does —
    /// fold the two [`GameVersion`] compounds back into `IntArray`s — or the
    /// file it produces will not read back in Bedrock.
    pub fn write<W: Write>(&self, _writer: W) -> Result<()> {
        todo!()
    }

    pub fn read<R: Read>(mut data: R) -> Result<Self> {
        let header_storage_version = data.read_u32::<LittleEndian>()?;
        let _file_size = data.read_u32::<LittleEndian>()?;

        // Decoded through a `Value` rather than straight into `Self`: the two
        // game-version keys need reshaping (below) before the struct can be
        // built from them.
        let mut value: Value = nbtx::from_le_bytes(&mut data)?;
        unpack_game_versions(&mut value);

        let mut settings: Self = nbtx::from_value(value)?;
        settings.header_storage_version = header_storage_version as i32;

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
