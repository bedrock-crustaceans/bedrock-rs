//! Round-trip and malformed-input property tests for `level.dat`
//! (`LevelSettings::write`/`LevelSettings::read`,
//! `crates/level/src/settings.rs`).
//!
//! # Why full identity holds here, not just semantic equality
//!
//! `LevelSettings::write`'s own doc comment documents a defaulted-field
//! carve-out: decoding a *real* level.dat that omits a `#[facet(default)]`
//! key (an old world saved before that field existed) and writing it back
//! out re-emits the field explicitly, which is why a decode-write-decode
//! round trip on a real file is only guaranteed `PartialEq`, not
//! byte-identical (see `tests/settings_write.rs`).
//!
//! That carve-out does not apply to the generator below: it builds a fully
//! populated `LevelSettings` value directly (every `#[facet(default)]`
//! field already has a concrete value, not an on-disk absence), so there is
//! nothing for `write` to default-fill that the value did not already
//! specify. For a value built this way, `read(write(x))` reproduces `x`
//! exactly -- checked with `PartialEq`, since NBT float fields and the
//! struct's own layout make an exact byte comparison brittle for reasons
//! unrelated to this property (float canonicalization is not something this
//! codec controls).

use bedrock_level::settings::{Abilities, Experiments, LevelSettings, Policies};
use bedrock_level::version::GameVersion;
use proptest::prelude::*;

fn game_version() -> impl Strategy<Value = GameVersion> {
    (
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
    )
        .prop_map(|(major, minor, patch, revision, beta)| {
            GameVersion::new(major, minor, patch, revision, beta)
        })
}

fn abilities() -> impl Strategy<Value = Abilities> {
    (
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        -1000f32..1000f32,
        -1000f32..1000f32,
        -1000f32..1000f32,
    )
        .prop_map(
            |(
                attack_mobs,
                attack_players,
                build,
                doors_and_switches,
                flying,
                instant_build,
                invulnerable,
                fly_speed,
                vertical_fly_speed,
                walk_speed,
            )| Abilities {
                attack_mobs,
                attack_players,
                build,
                doors_and_switches,
                flying,
                instant_build,
                invulnerable,
                lightning: false,
                mayfly: false,
                mine: true,
                op: false,
                open_containers: true,
                teleport: false,
                fly_speed,
                vertical_fly_speed,
                walk_speed,
                permissions_level: Some(0),
                player_permissions_level: Some(0),
            },
        )
}

fn experiments() -> impl Strategy<Value = Experiments> {
    (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(experiments_ever_used, saved_with_toggled_experiments, caves_and_cliffs, armadillo)| {
            Experiments {
                experiments_ever_used,
                saved_with_toggled_experiments,
                caves_and_cliffs,
                caves_and_cliffs_internal: false,
                armadillo,
                update_announced_live_2023: false,
            }
        },
    )
}

/// A fully populated `LevelSettings` value: every field is set to a
/// baseline, deterministic value, then a handful of representative fields
/// (spanning `bool`, `i32`, `i64`, `f32`, `String`, `Option<_>`, and the
/// nested `Abilities`/`Experiments`/`GameVersion` types) are drawn from the
/// given strategies. This is not every one of the struct's ~100 fields
/// varying independently -- the point of this property is the write/read
/// codec's fidelity for a value with real content in every field class the
/// struct has, not an exhaustive sweep of the field set (which
/// `tests/settings_write.rs`'s real-fixture coverage already exercises from
/// the read side).
#[expect(
    clippy::too_many_lines,
    reason = "sets every field of a struct with about 100 of them"
)]
fn level_settings() -> impl Strategy<Value = LevelSettings> {
    (
        any::<i64>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        "[a-zA-Z0-9 ]{0,24}",
        proptest::option::of("[a-zA-Z0-9.]{0,8}"),
        abilities(),
        proptest::option::of(experiments()),
        any::<bool>(),
        game_version(),
        proptest::option::of(game_version()),
    )
        .prop_map(
            |(
                random_seed,
                spawn_x,
                spawn_y,
                spawn_z,
                current_tick_extra,
                level_name,
                base_game_version,
                abilities,
                experiments,
                cheats_enabled,
                last_opened_with_version,
                minimum_compatible_client_version,
            )| {
                LevelSettings {
                    header_storage_version: 9,
                    editor_world_type: Some(0),
                    created_in_editor: false,
                    exported_from_editor: false,
                    random_seed_allowed: false,
                    sleeping_percentage: Some(50),
                    recipes_unlock: false,
                    cheats_enabled,
                    lightning_level: 0.0,
                    lightning_time: 0,
                    rain_level: 0.0,
                    rain_time: 0,
                    difficulty: 2,
                    game_mode: 0,
                    generator: 1,
                    limited_world_origin_x: 0,
                    limited_world_origin_y: 32767,
                    limited_world_origin_z: 0,
                    limited_world_depth: Some(256),
                    limited_world_width: Some(16),
                    minimum_compatible_client_version,
                    nether_scale: 8,
                    network_version: Some(constant_network_version(current_tick_extra)),
                    platform: 2,
                    platform_broadcast_intent: 3,
                    random_seed,
                    spawn_v1_villagers: false,
                    spawn_x,
                    spawn_y,
                    spawn_z,
                    storage_version: 9,
                    time: 100,
                    world_version: Some(1),
                    xbox_broadcast_intent: 3,
                    current_tick: i64::from(current_tick_extra),
                    experiments,
                    experimental_gameplay: false,
                    abilities,
                    edu_offer: Some(0),
                    edu_level: Some(0),
                    education_features_enabled: false,
                    last_opened_with_version,
                    bonus_chest_enabled: false,
                    bonus_chest_spawned: false,
                    command_block_output: true,
                    center_maps_to_origin: false,
                    command_blocks_enabled: true,
                    commands_enabled: true,
                    confirmed_platform_locked_content: false,
                    daylight_cycle: Some(1),
                    daylight_lock: true,
                    limited_crafting: false,
                    entity_drops: true,
                    fire_tick: true,
                    immediate_respawn: false,
                    insomnia: true,
                    mob_loot: true,
                    mob_spawning: true,
                    tile_drops: true,
                    weather_cycle: true,
                    projectiles_can_break_blocks: false,
                    drowning_damage: true,
                    fall_damage: true,
                    fire_damage: true,
                    freeze_damage: true,
                    keep_inventory: false,
                    max_command_chain_length: 65536,
                    mob_griefing: true,
                    natural_regeneration: true,
                    function_command_limit: 10000,
                    pvp: true,
                    random_tick_speed: 1,
                    respawn_blocks_explode: true,
                    send_command_feedback: true,
                    show_border_effect: true,
                    show_coordinates: false,
                    show_death_messages: true,
                    show_tags: true,
                    show_days_played: false,
                    show_recipe_messages: true,
                    locator_bar: true,
                    spawn_radius: 5,
                    tnt_explodes: true,
                    tnt_explosion_drop_decay: false,
                    force_game_mode: false,
                    has_been_loaded_in_creative: false,
                    has_locked_behavior_pack: false,
                    has_locked_resource_pack: false,
                    immutable_world: false,
                    is_from_locked_template: false,
                    is_from_world_template: false,
                    is_single_use_world: false,
                    is_world_template_option_locked: false,
                    requires_copied_pack_removal_check: false,
                    texture_packs_required: false,
                    is_hardcore: false,
                    player_has_died: false,
                    has_uncomplete_world_file_on_disk: false,
                    lan_broadcast: true,
                    lan_broadcast_intent: 3,
                    multiplayer_game: true,
                    multiplayer_game_intent: 0,
                    last_played: 1_700_000_000,
                    base_game_version,
                    biome_override: String::new(),
                    flat_world_layers: String::new(),
                    inventory_version: Some("1.21.0".to_string()),
                    level_name,
                    use_msa_gamertags_only: false,
                    world_start_count: 1,
                    start_with_map_enabled: false,
                    spawn_mobs: true,
                    server_chunk_tick_range: 4,
                    permissions_level: Some(0),
                    player_permissions_level: Some(0),
                    prid: String::new(),
                    world_policies: Some(Policies {}),
                    platform_broadcast: Some(3),
                    platform_broadcast_mode: Some(0),
                    xbl_broadcast: Some(3),
                    xbl_broadcast_mode: Some(0),
                }
            },
        )
}

/// A small helper so `network_version` still varies with the input without
/// adding a separate strategy leg -- any deterministic function of an
/// already-drawn field works for this property, since the point is field
/// coverage, not independence between fields.
fn constant_network_version(seed: i32) -> i32 {
    seed
}

proptest! {
    /// A fully populated `LevelSettings` value round-trips exactly through
    /// `write`/`read` -- see the module docs for why the defaulted-field
    /// carve-out documented on `LevelSettings::write` does not apply to a
    /// value built directly (as opposed to decoded from a real, possibly
    /// older, level.dat).
    #[test]
    fn level_settings_round_trips(settings in level_settings()) {
        let mut buf = Vec::new();
        settings.write(&mut buf).unwrap();

        let decoded = LevelSettings::read(std::io::Cursor::new(buf)).unwrap();
        prop_assert_eq!(decoded, settings);
    }

    /// Arbitrary bytes (of at least the 8-byte header) never panic
    /// `LevelSettings::read`.
    #[test]
    fn level_settings_read_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..200)) {
        let _ = LevelSettings::read(std::io::Cursor::new(bytes));
    }

    /// A truncated well-formed level.dat never panics either.
    #[test]
    fn level_settings_read_never_panics_on_truncated_input(
        settings in level_settings(),
        cut in 0usize..400,
    ) {
        let mut buf = Vec::new();
        settings.write(&mut buf).unwrap();
        let truncated = &buf[..cut.min(buf.len())];
        let _ = LevelSettings::read(std::io::Cursor::new(truncated.to_vec()));
    }
}
