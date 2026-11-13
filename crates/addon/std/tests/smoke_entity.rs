//! Entity smoke over the game's own files: every behaviour-pack entity definition of the
//! sample packs, the server install and client install B loads through `Driver` and
//! `bp.entities()` with no error-level diagnostic (MCB documents, baked documents the
//! loaders do not decode, are counted apart), and the upgraded `sulfur_cube` presets equal the
//! shipped ones.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_entity`; without the variable every test skips.
//!
//! Components and goals are typed, so the zero-error gate covers them too; the warnings are printed by code.

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Phase, Severity};
use bedrock_addon::document::{Content, Registry};
use bedrock_addon::families::entity::Entity;
use bedrock_addon::id::EntityId;
use bedrock_addon::json::Value;

use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{Behavior, OpenOptions, PackOrigin, PackStack, StackSet};
use bedrock_addon_std::{Driver, Options};
use bedrock_addon_test_support::corpus;
use bedrock_addon_test_support::json::strict as json;

use common::game_files;

/// The behaviour stack of the base-game pack `dir` alone, its files read by the driver that
/// opened it.
fn entities(dir: &Path, options: LoadOptions) -> PackStack<Behavior> {
    let mut driver = Driver::new(Options::default().with_open(OpenOptions {
        origin: PackOrigin::Package,
        ..OpenOptions::default()
    }))
    .unwrap();
    let pack = driver
        .pack_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e:?}", dir.display()));
    let set = StackSet {
        behavior: PackStack::of([pack], options.clone()).unwrap(),
        resource: PackStack::of([], options).unwrap(),
    };
    driver
        .load_stacks(set, StackOptions::default())
        .unwrap()
        .behavior
}

/// What a registry reported: its error-level findings and the counts. An
/// `UnsupportedFileFormat` finding is an MCB document as long as the game-files helper counted
/// as many on disk, an error otherwise.
struct Tally {
    documents: usize,
    errors: Vec<String>,
    mcb: usize,
    warnings: BTreeMap<&'static str, usize>,
    /// The notes of the upgrade steps (severity INFO): what the game's own fixer replaces
    /// or drops without a word.
    notes: BTreeMap<&'static str, usize>,
    /// The references of an entity to itself that name nothing (an event trigger, a sensor
    /// event or an `animate` entry that names no key of its own file: `UnresolvedReference`, an
    /// information of the validation), and the component groups an event names that the file
    /// does not define.
    self_references: usize,
}

/// The entity MCB documents on disk below the pack `dir`: `entities/` and the `entities`
/// brarchive.
fn entity_mcb(dir: &Path) -> usize {
    [
        dir.join("entities"),
        dir.join("__brarchive").join("entities.brarchive"),
    ]
    .iter()
    .filter(|path| path.exists())
    .map(|path| game_files::mcb_on_disk(path).total())
    .sum()
}

fn tally(registry: &Registry<Entity>, dir: &Path) -> Tally {
    let on_disk = entity_mcb(dir);
    let mut errors = Vec::new();
    let mut unsupported = Vec::new();
    let mut warnings = BTreeMap::new();
    let mut notes = BTreeMap::new();
    let mut self_references = 0;
    for diagnostic in registry.diagnostics() {
        if diagnostic.severity == Severity::Info
            && diagnostic.phase == Phase::Validate
            && diagnostic.code.id() == "core/unresolved_reference"
        {
            self_references += 1;
        }
        if diagnostic.severity == Severity::Info && diagnostic.phase == Phase::Upgrade {
            *notes.entry(diagnostic.code.id()).or_insert(0) += 1;
        }
        if diagnostic.code.id() == "core/unsupported_file_format" {
            unsupported.push(diagnostic.to_string());
        } else if diagnostic.severity >= Severity::Error {
            errors.push(diagnostic.to_string());
        } else if diagnostic.severity == Severity::Warning {
            *warnings.entry(diagnostic.code.id()).or_insert(0) += 1;
        }
    }
    let mcb = unsupported.len();
    if mcb > on_disk {
        errors.extend(unsupported);
    }
    Tally {
        documents: registry.len(),
        errors,
        mcb,
        warnings,
        notes,
        self_references,
    }
}

/// Loads every behaviour pack in `dirs` and requires no error, the warnings by code to be
/// `expected` exactly, the notes of the upgrade steps by code to be `noted` and the
/// references of the entities to themselves that name nothing to be `self_references`
/// (informations: the triggers of the game's own entities that fire an event their
/// definition lacks).
fn assert_loads(
    source: &str,
    dirs: &[PathBuf],
    expected: &[(&str, usize)],
    noted: &[(&str, usize)],
    self_references: usize,
) {
    let mut found_self_references = 0;
    let mut documents = 0;
    let mut mcb = 0;
    let mut warnings: BTreeMap<&str, usize> = BTreeMap::new();
    let mut found_notes: BTreeMap<&str, usize> = BTreeMap::new();
    let mut errors = Vec::new();
    for dir in dirs {
        if !dir.join("entities").is_dir() && !dir.join("manifest.json").is_file() {
            continue;
        }
        let stack = entities(dir, LoadOptions::default());
        let found = tally(stack.entities(), dir);
        documents += found.documents;
        mcb += found.mcb;
        found_self_references += found.self_references;
        for (code, count) in found.warnings {
            *warnings.entry(code).or_insert(0) += count;
        }
        for (code, count) in found.notes {
            *found_notes.entry(code).or_insert(0) += count;
        }
        errors.extend(
            found
                .errors
                .into_iter()
                .map(|e| format!("{}: {e}", dir.display())),
        );
    }
    eprintln!(
        "{source}: {documents} entities loaded, {} errors, {mcb} MCB documents, warnings {warnings:?}",
        errors.len()
    );
    // The errors a source is known to have, each by the end of its text, and the one file
    // they all belong to with their count and the first of them.
    let known: &[&str] = if source == "server install" {
        ERRORS_SERVER
    } else {
        &[]
    };
    for tail in known {
        let times = errors.iter().filter(|error| error.ends_with(tail)).count();
        assert_eq!(times, 1, "{source}: the known error {tail:?} occurs once");
    }
    if source == "server install" {
        let of_the_file: Vec<&String> = errors
            .iter()
            .filter(|error| error.contains(ERRORS_SERVER_FILE))
            .collect();
        assert_eq!(
            of_the_file.len(),
            ERRORS_SERVER_COUNT,
            "{source}: {of_the_file:#?}"
        );
        assert!(
            of_the_file
                .first()
                .is_some_and(|first| first.ends_with(ERRORS_SERVER_FIRST)),
            "{source}: {of_the_file:#?}"
        );
        errors.retain(|error| !error.contains(ERRORS_SERVER_FILE));
    }
    errors.retain(|error| !known.iter().any(|tail| error.ends_with(tail)));
    assert!(errors.is_empty(), "{source}: {errors:#?}");
    game_files::assert_warnings(source, &warnings, expected);
    eprintln!("{source}: notes of the upgrade steps {found_notes:?}");
    let noted: BTreeMap<&str, usize> = noted.iter().copied().collect();
    assert_eq!(
        found_notes, noted,
        "{source}: the notes of the upgrade steps"
    );
    assert_eq!(
        found_self_references, self_references,
        "{source}: the references of an entity to itself that name nothing"
    );
}

/// The `vanilla*` packs of a behaviour packs directory.
fn vanilla_packs(behavior_packs: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(behavior_packs)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("vanilla"))
        })
        .collect();
    dirs.sort();
    dirs
}

/// The one error of the server packs: `vanilla_1.26.30/entities/player.json` is a `"beta"`
/// document read with the newest typed shape, and its `minecraft:add_raid_omen` group writes
/// `"remove_effects": "bad_omen"`, a string where the typed reader takes only an array, so the
/// game logs a wrong-type error for it.
const ERRORS_SERVER: &[&str] = &[
    "entities/player.json | minecraft:entity/component_groups/minecraft:add_raid_omen/minecraft:spell_effects/remove_effects | wrong JSON type",
];

/// The beta slice is read strictly, so the same `player.json` is refused for more: its
/// `apply_knockback_rules` presets 0 to 11 write four members no type binds
/// (`scale_previous_velocity`, `horizontal_hit_angle_scale`, `vertical_hit_angle_scale`,
/// `vertical_position_angle_scale`; 48 errors). With the wrong-type error above and the closing
/// line that makes 50 errors of the one file: a newer game refusing an older server's beta
/// file.
const ERRORS_SERVER_FILE: &str =
    "vanilla_1.26.30: [error] resourcePack.vanilla_server.name | entities/player.json |";
/// The number of errors of [`ERRORS_SERVER_FILE`].
const ERRORS_SERVER_COUNT: usize = 50;
/// The end of the first error of [`ERRORS_SERVER_FILE`].
const ERRORS_SERVER_FIRST: &str = "minecraft:entity/components/minecraft:apply_knockback_rules/presets/0/scale_previous_velocity | unrecognized field";

/// The notes of the upgrade steps over the server packs. The eight `DroppedField` are one
/// member in eight packs (`vanilla_1.17.0` to `vanilla_1.21.90`): `axolotl.json`,
/// `component_groups/axolotl_baby/minecraft:ageable`, `"feed_items": "tropical_fish_bucket"`
/// (one string, no list) beside `"transform_to_item": "water_bucket:0"`, which the 1.21.130
/// step gives to the entries of a list alone and then removes.
/// The other notes are findings of components converted from below their typed version, which
/// the game accepts silently: 38 informations of the legacy-reader rule, one `MemberNotKept`
/// per older `behavior.follow_mob` (19; the fixer writes `abort_navigation_on_stop: false`,
/// which the goal holds and never writes), 9 `target_distance`, 21 legacy bounds, 199 unknown
/// members, and 6 `DroppedField` for legacy goal ranges written `{min, max}` that the member's
/// default replaces. The 10 spawn chances and 3 `*_spawn_percent_chance` members are read, not
/// noted.
const NOTES_SERVER: &[(&str, usize)] = &[
    ("core/dropped_field", 14),
    ("entity/member_removed_by_upgrade", 9),
    ("core/invalid_value", 21),
    ("core/member_not_kept", 19),
    ("core/unrecognized_field", 199),
    ("core/value_rewritten", 38),
];
/// The same over the packs of client install B (the same, with 6 more unknown members).
const NOTES_CLIENT_INSTALL_B: &[(&str, usize)] = &[
    ("core/dropped_field", 14),
    ("entity/member_removed_by_upgrade", 9),
    ("core/invalid_value", 21),
    ("core/member_not_kept", 19),
    ("core/unrecognized_field", 205),
    ("core/value_rewritten", 38),
];

#[test]
fn sample_packs_entities_load() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    assert_loads(
        "sample packs",
        &[samples.join("behavior_pack")],
        // The 4 unknown members and the 16 undefined groups are informations of the crate's
        // own: the game accepts them without a word.
        &[],
        // Two older `behavior.follow_mob` goals' `abort_navigation_on_stop`; the 4 unknown
        // members of the conversions.
        &[("core/member_not_kept", 2), ("core/unrecognized_field", 4)],
        // Plus the 16 undefined groups an event names.
        29,
    );
}

#[test]
fn server_install_entities_load() {
    let server = skip!(game_files::SERVER_INSTALL);
    assert_loads(
        "server install",
        &vanilla_packs(&server.join("behavior_packs")),
        // The 9 `target_distance`, the 31 legacy bounds, the 2 npc partials, the 202 unknown
        // members and the 151 undefined groups are informations of the crate's own; none of
        // them is in the game's content log.
        &[],
        NOTES_SERVER,
        // Plus the 151 undefined groups an event names.
        287,
    );
}

#[test]
fn client_install_b_entities_load() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    assert_loads(
        "client install B",
        &vanilla_packs(&client.join("behavior_packs")),
        // As for the server install, with the 1.26.40 arrows'
        // `projectile.owner_launch_immunity_ticks` among the unknown members.
        &[],
        NOTES_CLIENT_INSTALL_B,
        // Plus the 151 undefined groups an event names.
        287,
    );
}

/// The 1.26.30 and 1.26.40 entity files of client install B and the in-place 1.26.40
/// `creaking.json` rewrites load with no error; `sulfur_cube` 1.26.30 carries
/// `require_collision_overlap: false` into every preset, equal to the shipped 1.26.40 copy.
#[test]
fn upgraded_sulfur_cube_presets_equal_the_shipped_ones() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let packs = client.join("behavior_packs");
    let dirs: Vec<PathBuf> = [
        "vanilla_1.26.30",
        "vanilla_1.26.40",
        "vanilla_1.21.50",
        "vanilla_1.21.60",
    ]
    .iter()
    .map(|name| packs.join(name))
    .filter(|dir| dir.is_dir())
    .collect();
    assert_loads(
        "client install B 1.26.3x/4x",
        &dirs,
        // Unknown members, legacy bounds and undefined groups are informations of the crate's
        // own.
        &[],
        // One older `behavior.follow_mob` goal's `abort_navigation_on_stop`; the conversions'
        // 3 bounds and 11 unknown members.
        &[
            // 2 of the 3 bounds were spawn chances, which are read, not noted.
            ("core/invalid_value", 1),
            ("core/member_not_kept", 1),
            ("core/unrecognized_field", 11),
        ],
        // Plus the 6 undefined groups an event names.
        21,
    );
    let at = |pack: &str| -> Option<Entity> {
        let stack = entities(&packs.join(pack), LoadOptions::default());
        stack
            .entities()
            .get(&EntityId::parse("minecraft:sulfur_cube"))
            .map(|document| document.value().clone())
    };
    let (Some(upgraded), Some(install_b)) = (at("vanilla_1.26.30"), at("vanilla_1.26.40")) else {
        corpus::skipped(
            "the sulfur_cube presets of client install B",
            "sulfur_cube is not in both packs",
        );
        return;
    };
    let presets = |entity: &Entity| -> Vec<String> {
        let text = entity.to_json().unwrap();
        overlap_flags(&text)
    };
    let upgraded_flags = presets(&upgraded);
    assert!(!upgraded_flags.is_empty(), "sulfur_cube has presets");
    assert_eq!(upgraded_flags, presets(&install_b));

    // The 1.26.30 skeleton and stray carry `ranged_attack.attack_interval_min` / `_max`: they
    // convert to `attack_interval {min, max}` in `components` and every group.
    let stack = entities(&packs.join("vanilla_1.26.30"), LoadOptions::default());
    for id in ["minecraft:skeleton", "minecraft:stray"] {
        let written = written(&stack, id);
        let intervals = members(
            &written,
            "minecraft:behavior.ranged_attack",
            "attack_interval",
        );
        assert!(!intervals.is_empty(), "{id} has a ranged_attack");
        for interval in intervals {
            let range = interval
                .as_object()
                .unwrap_or_else(|| panic!("{id}: {interval:?}"));
            assert!(
                range.contains_key("min") && range.contains_key("max"),
                "{id}: {interval:?}"
            );
        }
        assert!(
            members(
                &written,
                "minecraft:behavior.ranged_attack",
                "attack_interval_min"
            )
            .is_empty(),
            "{id}"
        );
    }
    // Older packs write `beg.look_time` and `float_wander.float_duration` as arrays (the
    // 1.20.80 wolf and bat): they convert to ranges.
    let stack = entities(&packs.join("vanilla_1.20.80"), LoadOptions::default());
    for (id, goal, member) in [
        ("minecraft:wolf", "minecraft:behavior.beg", "look_time"),
        (
            "minecraft:bat",
            "minecraft:behavior.float_wander",
            "float_duration",
        ),
    ] {
        let values = members(&written(&stack, id), goal, member);
        assert!(!values.is_empty(), "{id} has {goal}.{member}");
        for value in values {
            let range = value
                .as_object()
                .unwrap_or_else(|| panic!("{id}: {value:?}"));
            assert!(
                range.contains_key("min") && range.contains_key("max"),
                "{id}: {value:?}"
            );
        }
    }
}

/// The entity `id` of `stack` as it writes back.
fn written(stack: &PackStack<Behavior>, id: &str) -> Value {
    let entity = stack
        .entities()
        .get(&EntityId::parse(id))
        .unwrap_or_else(|| panic!("{id} loads"));
    json(&entity.to_json().unwrap())
}

/// Every value of `goal.member` in the `components` and the groups of a written document.
fn members(document: &Value, goal: &str, member: &str) -> Vec<Value> {
    let Some(payload) = document
        .as_object()
        .and_then(|d| d.get("minecraft:entity"))
        .and_then(Value::as_object)
    else {
        return Vec::new();
    };
    let groups = payload
        .get("component_groups")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|groups| groups.values());
    payload
        .get("components")
        .into_iter()
        .chain(groups)
        .filter_map(|section| {
            section
                .as_object()?
                .get(goal)?
                .as_object()?
                .get(member)
                .cloned()
        })
        .collect()
}

/// Every `require_collision_overlap` value of a written document, in document order.
fn overlap_flags(text: &str) -> Vec<String> {
    const KEY: &str = "\"require_collision_overlap\": ";
    text.match_indices(KEY)
        .map(|(at, _)| {
            text[at + KEY.len()..]
                .chars()
                .take_while(char::is_ascii_alphabetic)
                .collect()
        })
        .collect()
}

/// Two creaking files (`vanilla_1.21.50` and `vanilla_1.21.60`) have a root
/// `use_beta_features: true`. The game reads no such member and counts no root members, so the
/// member is passed over, told where it is (`UnrecognizedField`, an information: a root member
/// of an uncounted envelope gets no line from the game), and both creakings load.
#[test]
fn a_creaking_with_a_third_root_member_loads() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let packs = client.join("behavior_packs");
    let creaking = EntityId::parse("minecraft:creaking");
    for pack in ["vanilla_1.21.50", "vanilla_1.21.60"] {
        let dir = packs.join(pack);
        let text = std::fs::read_to_string(dir.join("entities/creaking.json")).unwrap();
        assert!(text.contains(r#""use_beta_features": true"#), "{pack}");
        let stack = entities(&dir, LoadOptions::default());
        let registry = stack.entities();
        assert!(registry.get(&creaking).is_some(), "{pack}");
        assert!(
            !registry.rejected().iter().any(|rejected| rejected
                .origin
                .path
                .as_str()
                .ends_with("creaking.json")),
            "{pack}"
        );
        let at_the_root: Vec<String> = registry
            .diagnostics()
            .filter(|f| {
                f.location
                    .path
                    .as_ref()
                    .is_some_and(|path| path.as_str().ends_with("creaking.json"))
                    && f.location.json_path.segments().len() == 1
                    && f.location.json_path.to_string() != "minecraft:entity"
            })
            .map(|f| format!("{} {:?} {}", f.code.id(), f.severity, f.location.json_path))
            .collect();
        assert_eq!(
            at_the_root,
            ["core/unrecognized_field Info use_beta_features"],
            "{pack}"
        );
    }
}
