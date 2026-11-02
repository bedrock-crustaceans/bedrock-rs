//! Documents nested to `MAX_JSON_DEPTH`, and a level past it, load through every strategy of
//! the driver without a stack that overflows: on the calling thread with a megabyte of stack
//! (`Strategy::Sequential`), on the driver's own pool (`Strategy::Rayon`) and on a pool the
//! caller built with `RECOMMENDED_JOB_STACK` (`Strategy::RayonPool`), in whatever profile the
//! test is built with.
//!
//! The files are the ones whose nesting a reader follows by a call for each level, a file
//! or more for each such reader of a family:
//!
//! - read by facet: a block's legacy events (`DeprecatedDefinitionEvent`, the one document
//!   type of a pack that holds itself), an entity's filter (an untyped member, then
//!   `Filter`'s own reader);
//! - written as text and read again: an entity's goal with an unknown member and with a
//!   filter, a component of an old format, an item's deprecated side-car (facet-json's
//!   writer, behind `json::to_facet_json`);
//! - walked by hand: a loot table's pools in entries, a behaviour tree's nodes, a
//!   dialogue's raw text, a particle's events, a material's variants, a UI screen's controls
//!   and the global variables, a data-driven UI's children;
//! - merged across two resource packs that hold the same file: materials, UI, sounds and
//!   sound definitions, flipbook textures, the client biomes.
//!
//! Its own binary: an overflow aborts the process.
//!
//! Needs no game files.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode};
use bedrock_addon::document::Content;
use bedrock_addon::json::MAX_JSON_DEPTH;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet, WorldPacks};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

/// The stack of the thread that calls the driver.
const SMALL_STACK: usize = 1024 * 1024;

const BEHAVIOR: &str = "5e1f0a3c-3a61-4b1e-9e0e-3c1f5b0a6d22";
/// Two resource packs with the same files: what one pack has is merged with the other's.
const RESOURCES: [&str; 2] = [
    "5e1f0a3c-3a61-4b1e-9e0e-3c1f5b0a6d23",
    "5e1f0a3c-3a61-4b1e-9e0e-3c1f5b0a6d24",
];

/// The manifest of the pack `uuid` with one module of `module_type`.
fn manifest(uuid: &str, module_type: &str) -> String {
    let module = uuid.replace("5e1f0a3c", "9a0d3c1e");
    format!(
        r#"{{
    "format_version": 2,
    "header": {{
        "name": "deep {uuid}",
        "description": "",
        "uuid": "{uuid}",
        "version": [1, 0, 0],
        "min_engine_version": [1, 26, 0]
    }},
    "modules": [{{ "type": "{module_type}", "uuid": "{module}", "version": [1, 0, 0] }}]
}}"#
    )
}

fn write(dir: &Path, path: &str, text: &str) {
    let path = dir.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// A text nested exactly `levels` deep: `before`, then `turn` as often as the depth lets it,
/// then `bottom`, then the brackets that close them. `bottom` has the member [`PAD`], no
/// member of the format: what the turns leave of the depth it makes up as an array in an
/// array.
fn nested(levels: usize, before: &str, turn: &str, bottom: &str) -> String {
    let opens = |text: &str| open_in(text).len();
    let mut text = before.to_owned();
    let mut open = opens(before);
    while !turn.is_empty() && open + opens(turn) + opens(bottom) <= levels {
        text.push_str(turn);
        open += opens(turn);
    }
    let left = levels - open - opens(bottom);
    let pad = format!("{}0{}", "[".repeat(left), "]".repeat(left));
    text.push_str(&bottom.replace(PAD, &format!("\"zz\": {pad}")));
    for byte in open_in(&text).iter().rev() {
        text.push(if *byte == b'{' { '}' } else { ']' });
    }
    assert_eq!(deepest_of(&text), levels, "{text}");
    text
}

/// Where the bottom of a text of [`nested`] takes the levels the turns leave.
const PAD: &str = "\"zz\": 0";

/// The arrays and objects `text` leaves open.
fn open_in(text: &str) -> Vec<u8> {
    let (mut open, mut in_string) = (Vec::new(), false);
    for byte in text.bytes() {
        match byte {
            b'"' => in_string = !in_string,
            b'{' | b'[' if !in_string => open.push(byte),
            b'}' | b']' if !in_string => {
                open.pop();
            }
            _ => {}
        }
    }
    open
}

/// The most arrays and objects `text` has open at once.
fn deepest_of(text: &str) -> usize {
    let (mut open, mut deepest, mut in_string) = (0usize, 0usize, false);
    for byte in text.bytes() {
        match byte {
            b'"' => in_string = !in_string,
            b'{' | b'[' if !in_string => {
                open += 1;
                deepest = deepest.max(open);
            }
            b'}' | b']' if !in_string => open = open.saturating_sub(1),
            _ => {}
        }
    }
    deepest
}

/// The head of an entity `identifier` of format `version`, up to its components.
fn entity_head(version: &str, identifier: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:entity": {{"description": {{"identifier": "demo:{identifier}", "is_spawnable": false, "is_summonable": true, "is_experimental": false}}, "components": {{"#
    )
}

/// A file of the deep pack: where it lies, the collection that loads it, and its text nested
/// to a depth.
struct File {
    path: &'static str,
    collection: &'static str,
    text: fn(usize) -> String,
}

/// The files of the behaviour pack.
const BEHAVIOR_FILES: &[File] = &[
    // A block of the last format with events, its event a `sequence` in a `sequence`.
    File {
        path: "blocks/deep.json",
        collection: "blocks",
        text: |levels| {
            nested(
                levels,
                r#"{"format_version": "1.19.70", "minecraft:block": {"description": {"identifier": "demo:deep"}, "components": {}, "events": {"e": "#,
                r#"{"sequence": ["#,
                r#"{"probability": 0.5, "zz": 0"#,
            )
        },
    },
    // An entity whose sensor has a filter of `all_of` in `all_of`.
    File {
        path: "entities/filter.json",
        collection: "entities",
        text: |levels| {
            nested(
                levels,
                &(entity_head("1.21.0", "filter")
                    + r#""minecraft:environment_sensor": {"triggers": {"event": "demo:e", "filters": "#),
                r#"{"all_of": ["#,
                r#"{"test": "is_daytime", "value": true, "zz": 0"#,
            )
        },
    },
    // A goal with an unknown member of arrays: the goal's lenient read writes the component
    // as text and reads it again.
    File {
        path: "entities/goal.json",
        collection: "entities",
        text: |levels| {
            nested(
                levels,
                &(entity_head("1.21.0", "goal")
                    + r#""minecraft:behavior.float": {"priority": 0, "#),
                "",
                r#""zz": 0"#,
            )
        },
    },
    // The same goal in the oldest format.
    File {
        path: "entities/goal_old.json",
        collection: "entities",
        text: |levels| {
            nested(
                levels,
                &(entity_head("1.8.0", "goal_old")
                    + r#""minecraft:behavior.float": {"priority": 0, "#),
                "",
                r#""zz": 0"#,
            )
        },
    },
    // A goal whose target has a filter of `all_of` in `all_of`.
    File {
        path: "entities/goal_filter.json",
        collection: "entities",
        text: |levels| {
            nested(
                levels,
                &(entity_head("1.21.0", "goal_filter")
                    + r#""minecraft:behavior.nearest_attackable_target": {"priority": 2, "entity_types": [{"max_dist": 8, "filters": "#),
                r#"{"all_of": ["#,
                r#"{"test": "is_daytime", "value": true, "zz": 0"#,
            )
        },
    },
    // A component of an old format with an unknown member: read by the lenient conversion.
    File {
        path: "entities/health_old.json",
        collection: "entities",
        text: |levels| {
            nested(
                levels,
                &(entity_head("1.8.0", "health_old")
                    + r#""minecraft:health": {"value": 5, "max": 5, "#),
                "",
                r#""zz": 0"#,
            )
        },
    },
    // An item with an unknown member in its description (a hand-written deprecated side-car;
    // the game refuses the item below 1.26.30, so it never reaches the depth).
    File {
        path: "items/dig.json",
        collection: "items",
        text: |levels| {
            nested(
                levels,
                r#"{"format_version": "1.21.0", "minecraft:item": {"components": {"minecraft:max_stack_size": 16}, "description": {"identifier": "demo:dig", "#,
                "",
                r#""zz": 0"#,
            )
        },
    },
    // A loot table whose entries hold pools.
    File {
        path: "loot_tables/deep.json",
        collection: "loot_tables",
        text: |levels| {
            nested(
                levels,
                r#"{"pools": ["#,
                r#"{"rolls": 1, "entries": [{"type": "empty", "pools": ["#,
                r#"{"rolls": 1, "zz": 0"#,
            )
        },
    },
    // A behaviour tree of an inverter in an inverter.
    File {
        path: "behavior_trees/deep.json",
        collection: "behavior_trees",
        text: |levels| {
            nested(
                levels,
                r#"{"minecraft:behavior_tree": {"minecraft:name": "deep", "minecraft:root": "#,
                r#"{"minecraft:inverter": {"child": "#,
                r#"{"minecraft:wait_ticks": {"num_of_ticks": 1, "zz": 0"#,
            )
        },
    },
    // A dialogue whose text is raw text with raw text for its argument.
    File {
        path: "dialogue/deep.json",
        collection: "dialogue",
        text: |levels| {
            nested(
                levels,
                r#"{"format_version": "1.17.0", "minecraft:npc_dialogue": {"scenes": [{"scene_tag": "deep", "text": "#,
                r#"{"rawtext": [{"translate": "a", "with": "#,
                r#"{"rawtext": [{"text": "b", "zz": 0"#,
            )
        },
    },
];

/// The files each of the two resource packs has.
const RESOURCE_FILES: &[File] = &[
    // A particle whose event is a `sequence` in a `sequence`.
    File {
        path: "particles/deep.particle.json",
        collection: "particles",
        text: |levels| {
            nested(
                levels,
                r#"{"format_version": "1.10.0", "particle_effect": {"description": {"identifier": "demo:deep", "basic_render_parameters": {"material": "particles_alpha", "texture": "textures/particle/particles"}}, "components": {}, "events": {"e": "#,
                r#"{"sequence": ["#,
                r#"{"log": "x", "zz": 0"#,
            )
        },
    },
    // A material with a variant that has a variant.
    File {
        path: "materials/deep.material",
        collection: "materials",
        text: |levels| {
            nested(
                levels,
                r#"{"materials": {"version": "1.0.0", "deep": "#,
                r#"{"variants": [{"v": "#,
                r#"{"vertexShader": "x", "zz": 0"#,
            )
        },
    },
    // A UI screen of a panel in a panel, and the list that names its file.
    File {
        path: "ui/_ui_defs.json",
        collection: "",
        text: |_| r#"{"ui_defs": ["ui/deep.json"]}"#.to_owned(),
    },
    File {
        path: "ui/deep.json",
        collection: "ui",
        text: |levels| {
            nested(
                levels,
                r#"{"namespace": "deep", "root": "#,
                r#"{"type": "panel", "controls": [{"c": "#,
                r#"{"type": "label", "text": "x", "zz": 0"#,
            )
        },
    },
    // The UI's global variables, one an object in an object.
    File {
        path: "ui/_global_variables.json",
        collection: "ui",
        text: |levels| nested(levels, r#"{"$deep": "#, r#"{"a": "#, r#"{"zz": 0"#),
    },
    // A data-driven UI whose panel has a panel for its child. A panel is no child of a
    // panel: the reader says so at each level it walks, and walks them all.
    File {
        path: "ddui/root/deep.json",
        collection: "data_driven_ui",
        text: |levels| {
            nested(
                levels,
                r#"{"format_version": "1.21.130", "minecraft:ui-root": {"description": {"identifier": "demo:deep"}, "attribs": {"root_point": "minecraft:screen"}, "layout": {"markup": ["#,
                r#"{"component": "Panel", "children": ["#,
                r#"{"component": "Panel.Text", "attribs": {"text": "x", "zz": 0"#,
            )
        },
    },
    // The sounds of a block, with an object in an object beside them.
    File {
        path: "sounds.json",
        collection: "sounds_json",
        text: |levels| {
            nested(
                levels,
                r#"{"block_sounds": {"deep": {"events": {"break": {"sound": "x", "deep": "#,
                r#"{"a": "#,
                r#"{"zz": 0"#,
            )
        },
    },
    File {
        path: "sounds/sound_definitions.json",
        collection: "sound_definitions",
        text: |levels| {
            nested(
                levels,
                r#"{"format_version": "1.14.0", "sound_definitions": {"deep.click": {"category": "ui", "sounds": ["sounds/x"], "deep": "#,
                r#"{"a": "#,
                r#"{"zz": 0"#,
            )
        },
    },
    File {
        path: "textures/flipbook_textures.json",
        collection: "flipbooks",
        text: |levels| {
            nested(
                levels,
                r#"[{"flipbook_texture": "textures/blocks/x", "atlas_tile": "x", "deep": "#,
                r#"{"a": "#,
                r#"{"zz": 0"#,
            )
        },
    },
    File {
        path: "biomes_client.json",
        collection: "legacy_client_biomes",
        text: |levels| {
            nested(
                levels,
                r#"{"biomes": {"default": {"deep": "#,
                r#"{"a": "#,
                r#"{"zz": 0"#,
            )
        },
    },
];

/// The three packs with every file nested `levels` deep.
fn deep_packs(dir: &Path, levels: usize) {
    write(dir, "deep_bp/manifest.json", &manifest(BEHAVIOR, "data"));
    for file in BEHAVIOR_FILES {
        write(dir, &format!("deep_bp/{}", file.path), &(file.text)(levels));
    }
    for (index, uuid) in RESOURCES.iter().enumerate() {
        write(
            dir,
            &format!("deep_rp{index}/manifest.json"),
            &manifest(uuid, "resources"),
        );
        for file in RESOURCE_FILES {
            write(
                dir,
                &format!("deep_rp{index}/{}", file.path),
                &(file.text)(levels),
            );
        }
    }
}

/// The stacks of the packs in `dir`, loaded with `strategy`.
fn load(dir: &Path, strategy: Strategy) -> StackSet {
    let mut driver = Driver::new(Options::default().with_strategy(strategy)).unwrap();
    let repository = driver
        .repository([DirSource::packs(dir.to_path_buf(), PackOrigin::World)])
        .unwrap();
    let listed = |uuids: &[&str]| {
        let entries: Vec<String> = uuids
            .iter()
            .map(|uuid| format!(r#"{{"pack_id": "{uuid}", "version": [1, 0, 0]}}"#))
            .collect();
        format!("[{}]", entries.join(", "))
    };
    let world = WorldPacks::parse(
        Some(listed(&[BEHAVIOR]).as_bytes()),
        Some(listed(&RESOURCES).as_bytes()),
    )
    .unwrap();
    driver
        .load_stacks(
            repository.stacks(&world, LoadOptions::default()),
            StackOptions::default(),
        )
        .unwrap()
}

/// What a load made of a collection's files: how many it rejected (where the collection
/// counts them), and how many of its findings say the text is no JSON it reads (the
/// refusal of a text past the limit, in each family's words).
type Made = (Option<usize>, usize);

/// What a load made of the files, by collection.
type Loaded = BTreeMap<&'static str, Made>;

/// How many of `findings` refuse a text.
fn malformed<'a>(findings: impl Iterator<Item = &'a Diagnostic>) -> usize {
    findings
        .filter(|finding| matches!(finding.code, DiagnosticCode::MalformedInput { .. }))
        .count()
}

/// What a load of files past the limit must make of each collection: every file refused.
/// The collections that merge the two packs' copies tell it once for each copy or once for
/// the name, as each does for any file that is no JSON.
const REFUSED: &[(&str, Made)] = &[
    ("blocks", (Some(1), 1)),
    ("entities", (Some(5), 5)),
    ("items", (Some(1), 1)),
    ("loot_tables", (Some(1), 1)),
    ("behavior_trees", (Some(1), 1)),
    ("dialogue", (Some(1), 1)),
    ("particles", (Some(2), 2)),
    ("materials", (None, 2)),
    ("ui", (None, 2)),
    ("data_driven_ui", (None, 2)),
    ("sounds_json", (None, 1)),
    ("sound_definitions", (None, 1)),
    ("flipbooks", (None, 2)),
    ("legacy_client_biomes", (None, 2)),
];

/// What a load must make of the files: nothing refused, or every one.
fn expected(refused: bool) -> Loaded {
    REFUSED
        .iter()
        .map(|&(collection, made)| {
            let (rejected, _) = made;
            let made = if refused {
                made
            } else {
                (rejected.map(|_| 0), 0)
            };
            (collection, made)
        })
        .collect()
}

/// What the load `set` made of each collection.
fn made_of(set: &StackSet) -> Loaded {
    let (behavior, resource) = (&set.behavior, &set.resource);
    let ui = resource.data_driven_ui();
    Loaded::from([
        (
            "blocks",
            (
                Some(behavior.blocks().rejected().len()),
                malformed(behavior.blocks().diagnostics()),
            ),
        ),
        (
            "entities",
            (
                Some(behavior.entities().rejected().len()),
                malformed(behavior.entities().diagnostics()),
            ),
        ),
        (
            "items",
            (
                Some(behavior.items().rejected().len()),
                malformed(behavior.items().diagnostics()),
            ),
        ),
        (
            "loot_tables",
            (
                Some(behavior.loot_tables().rejected().len()),
                malformed(behavior.loot_tables().diagnostics()),
            ),
        ),
        (
            "behavior_trees",
            (
                Some(behavior.behavior_trees().rejected().len()),
                malformed(behavior.behavior_trees().diagnostics()),
            ),
        ),
        (
            "dialogue",
            (
                Some(behavior.dialogue().rejected().len()),
                malformed(behavior.dialogue().diagnostics()),
            ),
        ),
        (
            "particles",
            (
                Some(resource.particles().rejected().len()),
                malformed(resource.particles().diagnostics()),
            ),
        ),
        (
            "materials",
            (None, malformed(resource.materials().diagnostics())),
        ),
        ("ui", (None, malformed(resource.ui().diagnostics()))),
        ("data_driven_ui", (None, malformed(ui.diagnostics()))),
        (
            "sounds_json",
            (None, malformed(resource.sounds_json().diagnostics())),
        ),
        (
            "sound_definitions",
            (None, malformed(resource.sound_definitions().diagnostics())),
        ),
        (
            "flipbooks",
            (None, malformed(resource.flipbooks().diagnostics())),
        ),
        (
            "legacy_client_biomes",
            (
                None,
                malformed(resource.legacy_client_biomes().diagnostics()),
            ),
        ),
    ])
}

/// What the read of the files nested `levels` deep found in them, loaded with `strategy`
/// from a thread with a megabyte of stack: what each collection made of its files, how many
/// documents the collections that count them hold, and how long the text of the deep
/// material is when its value is shown.
fn loaded(levels: usize, strategy: Strategy) -> (Loaded, [usize; 4], usize) {
    let (made, held, material, _) = loaded_with_entities(levels, strategy);
    (made, held, material)
}

/// The entities a load holds, each as it is written, by the file name of [`BEHAVIOR_FILES`]
/// its identifier is (`demo:goal` is `goal`).
type Entities = BTreeMap<String, String>;

/// [`loaded`], and the entities the load holds.
fn loaded_with_entities(
    levels: usize,
    strategy: Strategy,
) -> (Loaded, [usize; 4], usize, Entities) {
    let dir = tempfile::tempdir().unwrap();
    deep_packs(dir.path(), levels);
    let root = dir.path().to_path_buf();
    std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(move || {
            let set = load(&root, strategy);
            let (behavior, resource) = (&set.behavior, &set.resource);
            let held = [
                behavior.blocks().len(),
                behavior.entities().len(),
                behavior.items().len(),
                resource.particles().iter().count(),
            ];
            let material = resource
                .materials()
                .files
                .values()
                .map(|file| format!("{file:?}").len())
                .sum();
            let entities = behavior
                .entities()
                .iter()
                .map(|(id, document)| {
                    let name = id.as_str().trim_start_matches("demo:").to_owned();
                    (
                        name,
                        Content::to_json(document.value()).expect("the entity writes"),
                    )
                })
                .collect();
            (made_of(&set), held, material, entities)
        })
        .unwrap()
        .join()
        .expect("the load does not panic")
}

/// Every strategy the build has, by name.
fn strategies() -> Vec<(&'static str, Strategy)> {
    let strategies = vec![("Sequential", Strategy::Sequential)];
    #[cfg(feature = "rayon")]
    let strategies = {
        let mut strategies = strategies;
        strategies.push(("Rayon", Strategy::Rayon { threads: None }));
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .stack_size(bedrock_addon::load::RECOMMENDED_JOB_STACK)
            .build()
            .unwrap();
        strategies.push(("RayonPool", Strategy::RayonPool(std::sync::Arc::new(pool))));
        strategies
    };
    strategies
}

#[test]
fn the_files_nest_as_deep_as_asked() {
    for levels in [
        40,
        MAX_JSON_DEPTH - 1,
        MAX_JSON_DEPTH,
        MAX_JSON_DEPTH + 1,
        1000,
    ] {
        for file in BEHAVIOR_FILES.iter().chain(RESOURCE_FILES) {
            if !file.collection.is_empty() {
                assert_eq!(deepest_of(&(file.text)(levels)), levels, "{}", file.path);
            }
        }
    }
}

#[test]
fn documents_nested_to_the_limit_load_through_every_strategy() {
    for (name, strategy) in strategies() {
        for levels in [MAX_JSON_DEPTH - 1, MAX_JSON_DEPTH] {
            let (made, held, material) = loaded(levels, strategy.clone());
            assert_eq!(made, expected(false), "{name}, {levels} levels");
            // One block, five entities, one item, one particle (the two packs' is one).
            assert_eq!(held, [1, 5, 1, 1], "{name}, {levels} levels");
            // The material was read to its innermost variant: 42 of them, one in the other.
            assert!(material > 20_000, "{name}, {levels} levels: {material}");
        }
    }
}

#[test]
fn documents_nested_past_the_limit_are_rejected_through_every_strategy() {
    for (name, strategy) in strategies() {
        for levels in [MAX_JSON_DEPTH + 1, 1000, 100_000] {
            let (made, held, material) = loaded(levels, strategy.clone());
            assert_eq!(made, expected(true), "{name}, {levels} levels");
            assert_eq!(held, [0, 0, 0, 0], "{name}, {levels} levels");
            assert!(material < 5_000, "{name}, {levels} levels: {material}");
        }
    }
}

/// A load at the limit keeps what the deep member is in.
/// The goal with the unknown member, the goal and the sensor with the filter and the
/// component of the old format are held with every level of the file, not dropped for
/// their depth with the entity left standing.
#[test]
fn an_entity_nested_to_the_limit_keeps_the_component_that_nests() {
    let count = |text: &str, what: &str| text.matches(what).count();
    for (name, strategy) in strategies() {
        for levels in [MAX_JSON_DEPTH - 1, MAX_JSON_DEPTH] {
            let (.., entities) = loaded_with_entities(levels, strategy.clone());
            for file in BEHAVIOR_FILES
                .iter()
                .filter(|file| file.collection == "entities")
            {
                let stem = file
                    .path
                    .trim_start_matches("entities/")
                    .trim_end_matches(".json");
                let source = (file.text)(levels);
                let written = entities
                    .get(stem)
                    .unwrap_or_else(|| panic!("{name}, {levels} levels: no entity {stem}"));
                let component = match stem {
                    "filter" => "minecraft:environment_sensor",
                    "goal" | "goal_old" => "minecraft:behavior.float",
                    "goal_filter" => "minecraft:behavior.nearest_attackable_target",
                    "health_old" => "minecraft:health",
                    other => panic!("an entity file without an expectation: {other}"),
                };
                let at = format!("{name}, {levels} levels, {stem}");
                assert_eq!(count(written, &format!("\"{component}\"")), 1, "{at}");
                // The filter with every `all_of` of the file, the unknown member the levels
                // are padded with, and the depth of the file.
                assert_eq!(
                    count(written, "\"all_of\""),
                    count(&source, "\"all_of\""),
                    "{at}"
                );
                // In a goal the padding is an unknown member — of the goal itself, or of a filter
                // test inside it — which the legacy goal reader does not store (it reads only
                // the keys it knows) and the crate tells and does not keep: the goal stands, written
                // without it, so its depth is less than the file's. The sensor's filter (a
                // component kept as written) keeps it.
                if stem.starts_with("goal") {
                    assert_eq!(count(written, "\"zz\""), 0, "{at}");
                    // Without the padding the goal is at most as deep as the file.
                    assert!(deepest_of(written) <= levels, "{at}");
                } else {
                    assert_eq!(count(written, "\"zz\""), 1, "{at}");
                    assert_eq!(deepest_of(written), levels, "{at}");
                }
            }
        }
    }
}
