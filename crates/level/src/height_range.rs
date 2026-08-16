//! Resolving a dimension's block-Y bounds.
//!
//! Bedrock has never written an explicit "height range" record for the two
//! dimensions whose bounds have stayed fixed since LevelDB saves began
//! (Nether, End); it only started recording one for the Overworld, and only
//! after the Overworld's own bounds stopped being a compile-time constant.
//! Resolving a dimension's range is therefore a precedence chain: prefer
//! what a world actually recorded, and fall back to the engine default for
//! the world's era when it recorded nothing.
//!
//! # Precedence
//!
//! 1. [`dictionary_height_range`] -- an entry in the world's
//!    `LevelChunkMetaDataDictionary` naming this dimension, if one exists.
//! 2. [`default_height_range`] -- the dimension's fixed default, with the
//!    Overworld's own default chosen from the world's declared game version
//!    and its `caves_and_cliffs` experiment toggle.
//!
//! [`resolve`] runs the full chain. Both steps are also exposed on their
//! own: a caller that already knows a world carries no dictionary, or wants
//! the era default regardless of what a dictionary says, can call either
//! independently.
//!
//! # Which per-entry field is authoritative
//!
//! A dictionary entry can carry both `OriginalDimensionHeightRange` (the
//! range in effect when the entry was first created) and
//! `LastSavedDimensionHeightRange` (the range as of the entry's most recent
//! save), and the two disagree exactly when a chunk was created before the
//! Overworld's height extended and has been saved again since. Resolving
//! *the world as it stands now* means preferring the more recent figure, so
//! [`dictionary_height_range`] reads `LastSavedDimensionHeightRange` and
//! only falls back to `OriginalDimensionHeightRange` where the entry has no
//! last-saved figure at all -- which happens on a chunk that has only ever
//! been saved once, where the two fields would agree anyway.
//!
//! # Multiple entries naming the same dimension
//!
//! A dictionary is not one entry per dimension; it is one entry per
//! generation context a chunk was ever saved under, and several of those
//! can share a `DimensionName`. Most of them are stale -- a chunk saved
//! before and after a height change leaves both its old and new generation
//! context in the table, and the game does not delete the old one. Nothing
//! available here (a dictionary and a dimension, with no chunk data to
//! cross-reference against `0x3f` links) can tell a live entry from an
//! orphaned one, so [`dictionary_height_range`] does not try to pick a
//! single "the" entry among several. It unions them instead -- the lowest
//! `min` and the highest `max` across every entry naming the dimension --
//! which is the only policy that cannot make the resolved range too narrow
//! for data a live chunk actually saved under one of those entries. A
//! majority or most-referenced entry would read equally well when every
//! entry agrees, which is what every dictionary seen so far does, but it
//! has no way to fail safely the day one does not: undercounting a
//! genuinely wider entry silently drops the chunks saved outside the
//! narrower figure from whatever iterates the dimension by its resolved
//! bounds.
//!
//! # The Overworld's own default: version and experiment toggle
//!
//! The Overworld's height extended from `0..256` to `-64..320` as part of
//! Caves & Cliffs. Bedrock 1.17 shipped the extension behind the
//! `caves_and_cliffs` experiment toggle (and, in some builds, the related
//! `caves_and_cliffs_internal` toggle -- see [`crate::settings::Experiments`]);
//! 1.18 made it the unconditional default and retired the toggles' effect on
//! this decision (a field can still appear, set, in a level.dat that was
//! never part of the 1.17 preview, since a flag is carried forward once
//! recorded rather than cleared). [`default_height_range`] therefore
//! extends the Overworld when *either* toggle is on *or* the world's own
//! `lastOpenedWithVersion` is at least 1.18.0 -- the toggles are what a
//! 1.17 preview world needs, and the version is what covers every
//! 1.18-and-later world outright, since most of them never carry either
//! toggle at all.
//!
//! # A deliberate divergence: override, not union
//!
//! Where a dictionary names the dimension, [`resolve`] uses the dictionary
//! range in place of the era default rather than folding the two together.
//! That is a choice, not an oversight -- what the world's own dictionary
//! recorded about a chunk it actually saved is better evidence of the
//! dimension's real bounds than a default inferred from the level's
//! declared version, so it is trusted outright rather than only widening
//! the default. The alternative -- union the dictionary range into the era
//! default instead of replacing it -- is strictly more conservative (it can
//! only widen, never narrow, what [`resolve`] returns) and would be an
//! equally defensible choice; it is not what this module does. The two
//! policies are indistinguishable on every dictionary this crate has seen:
//! every real entry's range already equals or exceeds its dimension's era
//! default, so unioning it in would change nothing observed in the corpus.
//! The divergence is therefore a live design choice, not a corpus finding,
//! and would only produce a different answer from a world whose dictionary
//! somehow recorded a range narrower than what the version-and-toggle
//! default would otherwise give it.

use crate::metadata_dictionary::{HeightRange as DictionaryHeightRange, MetadataDictionary};
use crate::settings::LevelSettings;
use crate::version::GameVersion;
use bedrock_shared::world::dimension::Dimension;

/// A dimension's block-Y bounds, `min` inclusive and `max` exclusive -- the
/// same convention the fixture corpus's own subchunk scan uses, and the one
/// a subchunk-index range naturally produces (`index * 16` through
/// `(index + 1) * 16`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightRange {
    pub min: i32,
    pub max: i32,
}

impl HeightRange {
    pub const fn new(min: i32, max: i32) -> Self {
        Self { min, max }
    }

    /// Whether `other` falls entirely within this range.
    pub fn contains_range(&self, other: HeightRange) -> bool {
        self.min <= other.min && other.max <= self.max
    }

    /// The smallest range containing both `self` and `other`.
    pub fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

/// Widens a dictionary entry's stored `i16` bounds (see
/// [`crate::metadata_dictionary::HeightRange`]) to this module's `i32`. The
/// entry's own storage is narrower because it is exactly what the two NBT
/// shorts on disk hold; this module's is `i32` to match the block-Y
/// coordinates a caller resolving bounds actually works in.
impl From<DictionaryHeightRange> for HeightRange {
    fn from(range: DictionaryHeightRange) -> Self {
        Self {
            min: i32::from(range.min),
            max: i32::from(range.max),
        }
    }
}

/// The Overworld's height range before Caves & Cliffs: `0..256`.
pub const OVERWORLD_BEFORE_CAVES_AND_CLIFFS: HeightRange = HeightRange::new(0, 256);
/// The Overworld's height range from Caves & Cliffs onward: `-64..320`.
pub const OVERWORLD_AFTER_CAVES_AND_CLIFFS: HeightRange = HeightRange::new(-64, 320);
/// The Nether's height range. Unlike the Overworld, this has never changed,
/// so no dictionary entry has ever been observed recording one -- see the
/// module docs.
pub const NETHER: HeightRange = HeightRange::new(0, 128);
/// The End's height range. Also unchanged since LevelDB saves began, but
/// unlike the Nether, worlds that carry a dictionary at all have been seen
/// recording a `TheEnd` entry for it -- the dictionary records a generation
/// context per dimension a chunk was saved in, not only ones whose bounds
/// have ever moved.
pub const END: HeightRange = HeightRange::new(0, 256);

/// The `lastOpenedWithVersion` at and after which the Overworld's extended
/// height range is the unconditional default, independent of the
/// `caves_and_cliffs` experiment toggle. See the module docs.
pub const OVERWORLD_EXTENDED_HEIGHT_SINCE: GameVersion = GameVersion::new(1, 18, 0, 0, 0);

/// The `DimensionName` string a dictionary entry carries for `dimension`,
/// if this crate has one to look for. `None` for a dimension the dictionary
/// format has no naming convention for -- every dictionary entry seen
/// (across the four worlds carrying one) names one of the three built-in
/// dimensions, never `Dimension::Undefined` (on-disk id 3) or an add-on
/// dimension.
fn dictionary_dimension_name(dimension: Dimension) -> Option<&'static str> {
    match dimension {
        Dimension::Overworld => Some("Overworld"),
        Dimension::Nether => Some("Nether"),
        Dimension::End => Some("TheEnd"),
        Dimension::Undefined | Dimension::Other(_) => None,
    }
}

/// Resolves `dimension`'s height range from `dictionary`, per the module
/// docs' precedence-within-the-dictionary and multiple-entry policies.
/// `None` if the dictionary has no entry naming `dimension` with a usable
/// height-range field, or if `dimension` is not one the dictionary format
/// names at all.
pub fn dictionary_height_range(
    dimension: Dimension,
    dictionary: &MetadataDictionary,
) -> Option<HeightRange> {
    let name = dictionary_dimension_name(dimension)?;

    dictionary
        .iter()
        .filter(|(_, entry)| entry.dimension_name() == Some(name))
        .filter_map(|(_, entry)| {
            entry
                .last_saved_dimension_height_range()
                .or_else(|| entry.original_dimension_height_range())
        })
        .map(HeightRange::from)
        .reduce(HeightRange::union)
}

/// Whether the Overworld's extended height range (`-64..320`) applies,
/// per the module docs.
fn overworld_height_extended(settings: &LevelSettings) -> bool {
    settings.last_opened_with_version >= OVERWORLD_EXTENDED_HEIGHT_SINCE
        || settings.experiments.as_ref().is_some_and(|experiments| {
            experiments.caves_and_cliffs || experiments.caves_and_cliffs_internal
        })
}

/// Resolves `dimension`'s era default height range from `settings` alone,
/// with no dictionary involved. `None` for a dimension this task does not
/// define a default for (`Dimension::Undefined` and any add-on dimension --
/// see the module docs).
pub fn default_height_range(dimension: Dimension, settings: &LevelSettings) -> Option<HeightRange> {
    match dimension {
        Dimension::Overworld => Some(if overworld_height_extended(settings) {
            OVERWORLD_AFTER_CAVES_AND_CLIFFS
        } else {
            OVERWORLD_BEFORE_CAVES_AND_CLIFFS
        }),
        Dimension::Nether => Some(NETHER),
        Dimension::End => Some(END),
        Dimension::Undefined | Dimension::Other(_) => None,
    }
}

/// Resolves `dimension`'s height range: the dictionary entry (or entries)
/// naming it if `dictionary` has one, otherwise the era default. `None`
/// only where [`default_height_range`] would also return `None` -- a
/// dimension outside the vanilla three, which no dictionary entry names
/// either.
pub fn resolve(
    dimension: Dimension,
    settings: &LevelSettings,
    dictionary: Option<&MetadataDictionary>,
) -> Option<HeightRange> {
    dictionary
        .and_then(|dictionary| dictionary_height_range(dimension, dictionary))
        .or_else(|| default_height_range(dimension, settings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata_dictionary::MetadataDictionaryEntry;
    use crate::settings::Experiments;

    fn base_settings(
        last_opened_with_version: GameVersion,
        caves_and_cliffs: bool,
    ) -> LevelSettings {
        LevelSettings {
            header_storage_version: 0,
            editor_world_type: None,
            created_in_editor: false,
            exported_from_editor: false,
            random_seed_allowed: false,
            sleeping_percentage: None,
            recipes_unlock: false,
            cheats_enabled: false,
            lightning_level: 0.0,
            lightning_time: 0,
            rain_level: 0.0,
            rain_time: 0,
            difficulty: 0,
            game_mode: 0,
            generator: 0,
            limited_world_origin_x: 0,
            limited_world_origin_y: 0,
            limited_world_origin_z: 0,
            limited_world_depth: None,
            limited_world_width: None,
            minimum_compatible_client_version: None,
            nether_scale: 0,
            network_version: None,
            platform: 0,
            platform_broadcast_intent: 0,
            random_seed: 0,
            spawn_v1_villagers: false,
            spawn_x: 0,
            spawn_y: 0,
            spawn_z: 0,
            storage_version: 0,
            time: 0,
            world_version: None,
            xbox_broadcast_intent: 0,
            current_tick: 0,
            experiments: Some(Experiments {
                experiments_ever_used: caves_and_cliffs,
                saved_with_toggled_experiments: caves_and_cliffs,
                caves_and_cliffs,
                caves_and_cliffs_internal: false,
                armadillo: false,
                update_announced_live_2023: false,
            }),
            experimental_gameplay: false,
            abilities: crate::settings::Abilities {
                attack_mobs: false,
                attack_players: false,
                build: false,
                doors_and_switches: false,
                flying: false,
                instant_build: false,
                invulnerable: false,
                lightning: false,
                mayfly: false,
                mine: false,
                op: false,
                open_containers: false,
                teleport: false,
                fly_speed: 0.0,
                vertical_fly_speed: 0.0,
                walk_speed: 0.0,
                permissions_level: None,
                player_permissions_level: None,
            },
            edu_offer: None,
            edu_level: None,
            education_features_enabled: false,
            last_opened_with_version,
            bonus_chest_enabled: false,
            bonus_chest_spawned: false,
            command_block_output: false,
            center_maps_to_origin: false,
            command_blocks_enabled: false,
            commands_enabled: false,
            confirmed_platform_locked_content: false,
            daylight_cycle: None,
            daylight_lock: false,
            limited_crafting: false,
            entity_drops: false,
            fire_tick: false,
            immediate_respawn: false,
            insomnia: false,
            mob_loot: false,
            mob_spawning: false,
            tile_drops: false,
            weather_cycle: false,
            projectiles_can_break_blocks: false,
            drowning_damage: false,
            fall_damage: false,
            fire_damage: false,
            freeze_damage: false,
            keep_inventory: false,
            max_command_chain_length: 0,
            mob_griefing: false,
            natural_regeneration: false,
            function_command_limit: 0,
            pvp: false,
            random_tick_speed: 0,
            respawn_blocks_explode: false,
            send_command_feedback: false,
            show_border_effect: false,
            show_coordinates: false,
            show_death_messages: false,
            show_tags: false,
            show_days_played: false,
            show_recipe_messages: false,
            locator_bar: false,
            spawn_radius: 0,
            tnt_explodes: false,
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
            lan_broadcast: false,
            lan_broadcast_intent: 0,
            multiplayer_game: false,
            multiplayer_game_intent: 0,
            last_played: 0,
            base_game_version: None,
            biome_override: String::new(),
            flat_world_layers: String::new(),
            inventory_version: None,
            level_name: String::new(),
            use_msa_gamertags_only: false,
            world_start_count: 0,
            start_with_map_enabled: false,
            spawn_mobs: false,
            server_chunk_tick_range: 0,
            permissions_level: None,
            player_permissions_level: None,
            prid: String::new(),
            world_policies: None,
            platform_broadcast: None,
            platform_broadcast_mode: None,
            xbl_broadcast: None,
            xbl_broadcast_mode: None,
        }
    }

    fn entry_with_range(dimension_name: &str, min: i16, max: i16) -> MetadataDictionaryEntry {
        let mut compound = nbtx::Compound::new();
        compound.insert(
            "DimensionName".into(),
            nbtx::Value::String(dimension_name.into()),
        );
        let mut range = nbtx::Compound::new();
        range.insert("min".into(), nbtx::Value::Short(min));
        range.insert("max".into(), nbtx::Value::Short(max));
        compound.insert(
            "OriginalDimensionHeightRange".into(),
            nbtx::Value::Compound(range),
        );
        MetadataDictionaryEntry::new(nbtx::Value::Compound(compound))
    }

    #[test]
    fn overworld_before_1_18_without_experiment_is_legacy() {
        let settings = base_settings(GameVersion::new(1, 17, 40, 20, 0), false);
        assert_eq!(
            default_height_range(Dimension::Overworld, &settings),
            Some(OVERWORLD_BEFORE_CAVES_AND_CLIFFS)
        );
    }

    #[test]
    fn overworld_before_1_18_with_experiment_is_extended() {
        let settings = base_settings(GameVersion::new(1, 17, 20, 22, 0), true);
        assert_eq!(
            default_height_range(Dimension::Overworld, &settings),
            Some(OVERWORLD_AFTER_CAVES_AND_CLIFFS)
        );
    }

    #[test]
    fn overworld_at_1_18_is_extended_regardless_of_experiment() {
        let settings = base_settings(GameVersion::new(1, 18, 0, 0, 0), false);
        assert_eq!(
            default_height_range(Dimension::Overworld, &settings),
            Some(OVERWORLD_AFTER_CAVES_AND_CLIFFS)
        );
    }

    #[test]
    fn overworld_past_1_18_is_extended_with_experiment_absent_entirely() {
        let mut settings = base_settings(GameVersion::new(1, 19, 30, 4, 0), false);
        settings.experiments = None;
        assert_eq!(
            default_height_range(Dimension::Overworld, &settings),
            Some(OVERWORLD_AFTER_CAVES_AND_CLIFFS)
        );
    }

    #[test]
    fn nether_and_end_defaults_do_not_depend_on_version() {
        let settings = base_settings(GameVersion::new(1, 12, 0, 28, 1), false);
        assert_eq!(
            default_height_range(Dimension::Nether, &settings),
            Some(NETHER)
        );
        assert_eq!(default_height_range(Dimension::End, &settings), Some(END));
    }

    #[test]
    fn undefined_and_other_dimensions_have_no_default() {
        let settings = base_settings(GameVersion::new(1, 20, 0, 0, 0), false);
        assert_eq!(default_height_range(Dimension::Undefined, &settings), None);
        assert_eq!(default_height_range(Dimension::Other(4), &settings), None);
    }

    #[test]
    fn dictionary_entry_takes_precedence_over_default() {
        let mut dictionary = MetadataDictionary::new();
        dictionary.insert([1; 8], entry_with_range("Overworld", -64, 320));

        // A world declared as pre-1.18 with no experiment would otherwise
        // resolve to the legacy default; the dictionary entry overrides it.
        let settings = base_settings(GameVersion::new(1, 17, 0, 0, 0), false);
        assert_eq!(
            resolve(Dimension::Overworld, &settings, Some(&dictionary)),
            Some(HeightRange::new(-64, 320))
        );
    }

    #[test]
    fn dictionary_without_a_matching_entry_falls_back_to_default() {
        let mut dictionary = MetadataDictionary::new();
        dictionary.insert([1; 8], entry_with_range("TheEnd", 0, 256));

        let settings = base_settings(GameVersion::new(1, 17, 0, 0, 0), false);
        assert_eq!(
            resolve(Dimension::Overworld, &settings, Some(&dictionary)),
            Some(OVERWORLD_BEFORE_CAVES_AND_CLIFFS)
        );
    }

    #[test]
    fn no_dictionary_falls_back_to_default() {
        let settings = base_settings(GameVersion::new(1, 20, 0, 0, 0), false);
        assert_eq!(
            resolve(Dimension::Overworld, &settings, None),
            Some(OVERWORLD_AFTER_CAVES_AND_CLIFFS)
        );
    }

    /// Two entries naming the same dimension with disagreeing ranges: the
    /// corpus has never shown this for a real world (every real entry
    /// found agrees with every other), but the format allows it and the
    /// multiple-entry policy needs to be pinned regardless. See the module
    /// docs for why the resolved range is their union rather than either
    /// one alone.
    #[test]
    fn multiple_entries_naming_one_dimension_resolve_to_their_union() {
        let mut dictionary = MetadataDictionary::new();
        dictionary.insert([1; 8], entry_with_range("Overworld", 0, 256));
        dictionary.insert([2; 8], entry_with_range("Overworld", -64, 320));

        assert_eq!(
            dictionary_height_range(Dimension::Overworld, &dictionary),
            Some(HeightRange::new(-64, 320))
        );
    }

    /// The union picks up a wider `min` from one entry and a wider `max`
    /// from a different entry -- neither entry alone spans the union.
    #[test]
    fn multiple_entries_union_combines_the_widest_min_and_widest_max_independently() {
        let mut dictionary = MetadataDictionary::new();
        dictionary.insert([1; 8], entry_with_range("Overworld", -64, 100));
        dictionary.insert([2; 8], entry_with_range("Overworld", 0, 320));

        assert_eq!(
            dictionary_height_range(Dimension::Overworld, &dictionary),
            Some(HeightRange::new(-64, 320))
        );
    }

    #[test]
    fn entries_naming_a_different_dimension_are_ignored() {
        let mut dictionary = MetadataDictionary::new();
        dictionary.insert([1; 8], entry_with_range("Overworld", -64, 320));
        dictionary.insert([2; 8], entry_with_range("TheEnd", 0, 256));

        assert_eq!(
            dictionary_height_range(Dimension::Nether, &dictionary),
            None
        );
    }

    #[test]
    fn last_saved_range_wins_over_original_within_one_entry() {
        let mut compound = nbtx::Compound::new();
        compound.insert(
            "DimensionName".into(),
            nbtx::Value::String("Overworld".into()),
        );
        let mut original = nbtx::Compound::new();
        original.insert("min".into(), nbtx::Value::Short(0));
        original.insert("max".into(), nbtx::Value::Short(256));
        compound.insert(
            "OriginalDimensionHeightRange".into(),
            nbtx::Value::Compound(original),
        );
        let mut last_saved = nbtx::Compound::new();
        last_saved.insert("min".into(), nbtx::Value::Short(-64));
        last_saved.insert("max".into(), nbtx::Value::Short(320));
        compound.insert(
            "LastSavedDimensionHeightRange".into(),
            nbtx::Value::Compound(last_saved),
        );

        let mut dictionary = MetadataDictionary::new();
        dictionary.insert(
            [1; 8],
            MetadataDictionaryEntry::new(nbtx::Value::Compound(compound)),
        );

        assert_eq!(
            dictionary_height_range(Dimension::Overworld, &dictionary),
            Some(HeightRange::new(-64, 320))
        );
    }

    #[test]
    fn contains_range_is_inclusive_of_equal_bounds() {
        let range = HeightRange::new(-64, 320);
        assert!(range.contains_range(range));
        assert!(range.contains_range(HeightRange::new(-32, 160)));
        assert!(!range.contains_range(HeightRange::new(-80, 320)));
        assert!(!range.contains_range(HeightRange::new(-64, 336)));
    }

    #[test]
    fn overworld_before_1_18_with_only_the_internal_experiment_is_extended() {
        let mut settings = base_settings(GameVersion::new(1, 17, 20, 22, 0), false);
        settings
            .experiments
            .as_mut()
            .unwrap()
            .caves_and_cliffs_internal = true;
        assert_eq!(
            default_height_range(Dimension::Overworld, &settings),
            Some(OVERWORLD_AFTER_CAVES_AND_CLIFFS)
        );
    }

    /// An entry names the dimension but carries neither height-range field
    /// (a shape no real record has, but nothing on disk rules it out) --
    /// the entry contributes nothing to the union, so with no other entry
    /// to fall back on, [`dictionary_height_range`] finds nothing to
    /// reduce and [`resolve`] falls through to the era default.
    #[test]
    fn entry_with_neither_height_range_field_falls_through_to_default() {
        let mut compound = nbtx::Compound::new();
        compound.insert(
            "DimensionName".into(),
            nbtx::Value::String("Overworld".into()),
        );
        let mut dictionary = MetadataDictionary::new();
        dictionary.insert(
            [1; 8],
            MetadataDictionaryEntry::new(nbtx::Value::Compound(compound)),
        );

        assert_eq!(
            dictionary_height_range(Dimension::Overworld, &dictionary),
            None
        );

        let settings = base_settings(GameVersion::new(1, 20, 0, 0, 0), false);
        assert_eq!(
            resolve(Dimension::Overworld, &settings, Some(&dictionary)),
            Some(OVERWORLD_AFTER_CAVES_AND_CLIFFS)
        );
    }

    #[test]
    fn undefined_and_other_dimensions_stay_none_even_with_a_populated_dictionary() {
        let mut dictionary = MetadataDictionary::new();
        dictionary.insert([1; 8], entry_with_range("Overworld", -64, 320));
        dictionary.insert([2; 8], entry_with_range("Nether", 0, 128));
        dictionary.insert([3; 8], entry_with_range("TheEnd", 0, 256));

        let settings = base_settings(GameVersion::new(1, 20, 0, 0, 0), false);
        assert_eq!(
            resolve(Dimension::Undefined, &settings, Some(&dictionary)),
            None
        );
        assert_eq!(
            resolve(Dimension::Other(4), &settings, Some(&dictionary)),
            None
        );
    }
}
