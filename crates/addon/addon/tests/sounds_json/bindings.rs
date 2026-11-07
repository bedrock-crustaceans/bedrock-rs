//! `sounds.json`: the rows `sj02`–`sj16` of the fixtures (the vanilla row `sj01` is the smoke
//! test's) and the rules of the file, both ends pinned.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Merged, RawLayer};
use bedrock_addon::families::sounds_json::{SoundBindings, SoundRange};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackStack, Resource};

use super::common::{
    BINDINGS, copies, describe, expect_clean, expect_none, expect_one, found, json, pack_bytes,
    pack_of, stack_of,
};

/// A range of two bounds.
fn range(min: f32, max: f32) -> SoundRange {
    SoundRange { min, max }
}

/// The names of the unknown events the group reports, from vanilla's text of each: the
/// content-log error of `LevelSoundEventMap::getIdOtherwiseContentError`, after which the file
/// loads, a kept error.
fn unknown_events(sounds: &Merged<SoundBindings, RawLayer>) -> Vec<String> {
    found(sounds, "UnknownSound")
        .iter()
        .filter(|d| d.severity == Severity::Error && d.kept)
        .map(|d| {
            let line = d.code.to_string();
            let name = line
                .strip_prefix("Event name '")
                .and_then(|rest| rest.strip_suffix("' is not a valid LevelSoundEvent"))
                .unwrap_or_else(|| panic!("{line}"));
            assert!(d.detail.is_none());
            name.to_owned()
        })
        .collect()
}

/// The paths of the findings whose line contains `text`.
fn paths_with(sounds: &Merged<SoundBindings, RawLayer>, text: &str) -> Vec<String> {
    sounds
        .diagnostics()
        .filter(|d| d.to_string().contains(text))
        .map(|d| d.location.json_path.to_string())
        .collect()
}

/// A stack of `sounds.json` copies, lowest first, each pack with its `min_engine_version`.
fn versioned(copies: &[(Option<&str>, &str)], options: LoadOptions) -> PackStack<Resource> {
    let packs = copies
        .iter()
        .enumerate()
        .map(|(i, (engine, text))| {
            pack_bytes(
                10 * (i as u128 + 1),
                *engine,
                &[(BINDINGS, text.as_bytes())],
            )
        })
        .collect();
    stack_of(packs, options)
}

#[test]
fn sj02_ranges() {
    let stack = copies(
        BINDINGS,
        &[
            r#"{"individual_event_sounds": {"pitch": [0.25, 0.75], "events": {
            "attack": {"sound": "a", "pitch": 0.8},
            "break": {"sound": "a", "pitch": [1.2, 0.8]},
            "burp": {"sound": "a", "pitch": [0.9]},
            "death": {"sound": "a", "pitch": {"min": 0.5}},
            "drink": {"sound": "a", "pitch": true},
            "eat": {"sound": "a", "pitch": []},
            "fall": {"sound": "a", "pitch": {"min": 0.9, "max": 0.1}},
            "fizz": {"sound": "a", "pitch": {"max": 2}},
            "hurt": {"sound": "a", "pitch": [3, 1, 2]},
            "step": {"sound": "a", "pitch": null},
            "thorns": {"sound": "a", "pitch": "[1.5, 0.5]"},
            "swim": {"sound": "a", "pitch": false}
        }}}"#,
        ],
    );
    let sounds = stack.sounds_json();
    let events = &sounds.individual_event_sounds.events;
    let pitch = |event: &str| events[event].pitch;
    assert_eq!(pitch("attack"), range(0.8, 0.8));
    assert_eq!(pitch("break"), range(0.8, 1.2));
    assert_eq!(pitch("burp"), range(0.9, 0.9));
    // A missing bound keeps the table's; `max` is never below `min`.
    assert_eq!(pitch("death"), range(0.5, 0.75));
    assert_eq!(pitch("drink"), range(1.0, 1.0));
    assert_eq!(pitch("eat"), range(0.25, 0.75));
    assert_eq!(pitch("fall"), range(0.9, 0.9));
    assert_eq!(pitch("fizz"), range(0.25, 2.0));
    // Elements past the second are ignored.
    assert_eq!(pitch("hurt"), range(1.0, 3.0));
    assert_eq!(pitch("step"), range(0.25, 0.75));
    assert_eq!(pitch("thorns"), range(0.5, 1.5));
    assert_eq!(pitch("swim"), range(0.0, 0.0));
    // The volume no one sets is the unset range.
    assert!(events["attack"].volume.is_unset());
    assert_eq!(events["attack"].volume, range(-1.0, -1.0));
    // Rule 29a: the two bools.
    assert_eq!(
        paths_with(sounds, "invalid value"),
        [
            "individual_event_sounds/events/drink/pitch",
            "individual_event_sounds/events/swim/pitch"
        ]
    );
    assert_eq!(sounds.diagnostics().count(), 2, "{:#?}", describe(sounds));
}

#[test]
fn a_sound_given_as_a_name_takes_the_table_ranges_and_other_types_coerce() {
    let stack = copies(
        BINDINGS,
        &[r#"{"individual_named_sounds": {"volume": 0.5, "sounds": {
            "a": "x", "b": 5, "c": true, "d": null, "e": ["x"], "f": {"volume": 2}}}}"#],
    );
    let sounds = stack.sounds_json();
    let named = &sounds.individual_named_sounds.sounds;
    let read: Vec<(&str, &str)> = named
        .iter()
        .map(|(k, v)| (k.as_str(), v.sound.as_str()))
        .collect();
    assert_eq!(
        read,
        [
            ("a", "x"),
            ("b", ""),
            ("c", "true"),
            ("d", ""),
            ("e", ""),
            ("f", "")
        ]
    );
    assert_eq!(named["a"].volume, range(0.5, 0.5));
    assert_eq!(named["f"].volume, range(2.0, 2.0));
    assert_eq!(
        paths_with(sounds, "invalid value"),
        [
            "individual_named_sounds/sounds/b",
            "individual_named_sounds/sounds/c",
            "individual_named_sounds/sounds/e"
        ]
    );
}

#[test]
fn sj03_an_unknown_event_is_dropped() {
    let stack = copies(
        BINDINGS,
        &[
            r#"{"individual_event_sounds": {"events": {"notanevent": "x", "break": "y", "undefined": "z"}}}"#,
        ],
    );
    let sounds = stack.sounds_json();
    // `undefined` names the enum's `Undefined` value, which the binding's lookup refuses with
    // the same line.
    assert_eq!(unknown_events(sounds), ["notanevent", "undefined"]);
    let events = &sounds.individual_event_sounds.events;
    assert_eq!(events.len(), 1);
    assert_eq!(events["break"].sound, "y");
}

#[test]
fn sj04_event_names_fold_case_and_the_later_sorted_key_wins() {
    let stack = copies(
        BINDINGS,
        &[r#"{"individual_event_sounds": {"events": {"break": "lower", "Break": "upper"}}}"#],
    );
    let sounds = stack.sounds_json();
    let events = &sounds.individual_event_sounds.events;
    assert_eq!(events.len(), 1);
    // `Break` sorts before `break`.
    assert_eq!(events["break"].sound, "lower");
    expect_clean(sounds);
}

#[test]
fn sj05_a_block_base_chain_fills_missing_events() {
    let stack = copies(
        BINDINGS,
        &[r#"{"block_sounds": {
            "a": {"base": "b", "events": {"break": "a.break"}},
            "b": {"base": "c", "events": {"break": "b.break", "hit": "b.hit"}},
            "c": {"events": {"place": "c.place", "hit": "c.hit"}}}}"#],
    );
    let sounds = stack.sounds_json();
    let a = &sounds.block_sounds["a"];
    assert_eq!(a.base, "b");
    // The table holds what each type writes; an event it lacks is its base chain's, the
    // nearer definition first.
    assert_eq!(a.sounds.events.len(), 1);
    let sound = |block: &str, event: &str| {
        sounds
            .block_sound(block, event)
            .map(|sound| sound.sound.as_str())
    };
    assert_eq!(sound("a", "break"), Some("a.break"));
    assert_eq!(sound("a", "hit"), Some("b.hit"));
    assert_eq!(sound("a", "place"), Some("c.place"));
    assert_eq!(sound("b", "place"), Some("c.place"));
    assert_eq!(sound("c", "break"), None);
    assert_eq!(sound("a", "step"), None);
    assert_eq!(sound("nope", "hit"), None);
    assert_eq!(sounds.block_sounds["c"].sounds.events.len(), 2);
    expect_clean(sounds);
}

#[test]
fn a_type_that_leads_into_a_cycle_inherits_from_all_of_it() {
    let stack = copies(
        BINDINGS,
        &[r#"{"block_sounds": {
            "a": {"base": "b", "events": {"place": "a.place"}},
            "b": {"base": "a", "events": {"hit": "b.hit"}},
            "tail": {"base": "a"}, "far": {"base": "tail"},
            "0tail": {"base": "a"}, "0far": {"base": "0tail"}}}"#],
    );
    let sounds = stack.sounds_json();
    let sound = |block: &str, event: &str| {
        sounds
            .block_sound(block, event)
            .map(|sound| sound.sound.as_str())
    };
    // The whole cycle is passed once before the chain closes.
    assert_eq!(sound("tail", "place"), Some("a.place"));
    assert_eq!(sound("tail", "hit"), Some("b.hit"));
    assert_eq!(sound("far", "hit"), Some("b.hit"));
    assert_eq!(sound("far", "step"), None);
    // The same when the types that lead in are resolved before the cycle is.
    assert_eq!(sound("0tail", "hit"), Some("b.hit"));
    assert_eq!(sound("0far", "hit"), Some("b.hit"));
    assert_eq!(sound("0far", "place"), Some("a.place"));
    assert_eq!(sound("a", "hit"), Some("b.hit"));
    assert_eq!(sound("b", "place"), Some("a.place"));
    // A member names itself; a type that leads in names the member it enters at.
    assert_eq!(
        paths_with(sounds, "Block base sound 'a' is recursively used"),
        [
            "block_sounds/0far/base",
            "block_sounds/0tail/base",
            "block_sounds/a/base",
            "block_sounds/far/base",
            "block_sounds/tail/base"
        ]
    );
    assert_eq!(
        paths_with(sounds, "Block base sound 'b' is recursively used"),
        ["block_sounds/b/base"]
    );
    assert_eq!(sounds.diagnostics().count(), 6);
}

#[test]
fn sj06_a_base_cycle_and_an_unknown_base() {
    let stack = copies(
        BINDINGS,
        &[r#"{"block_sounds": {
            "a": {"base": "b"}, "b": {"base": "a", "events": {"hit": "b.hit"}},
            "d": {"base": "nope"}, "self": {"base": "self"}}}"#],
    );
    let sounds = stack.sounds_json();
    assert_eq!(
        paths_with(sounds, "Block base sound 'a' is recursively used"),
        ["block_sounds/a/base"]
    );
    assert_eq!(
        paths_with(sounds, "Block base sound 'b' is recursively used"),
        ["block_sounds/b/base"]
    );
    assert_eq!(
        paths_with(sounds, "Block base sound 'self' is recursively used"),
        ["block_sounds/self/base"]
    );
    assert_eq!(
        paths_with(sounds, "Unknown block base sound 'nope'"),
        ["block_sounds/d/base"]
    );
    assert_eq!(found(sounds, "ReferenceCycle").len(), 3);
    assert_eq!(found(sounds, "UnresolvedReference").len(), 1);
    // the two lines are the game's content-log errors, the file loads: kept
    assert!(
        sounds
            .diagnostics()
            .all(|d| d.severity == Severity::Error && d.kept)
    );
    // What the chain reached before it closed is still inherited, and no lookup runs on.
    assert_eq!(sounds.block_sound("a", "hit").unwrap().sound, "b.hit");
    assert!(sounds.block_sound("b", "break").is_none());
    assert!(sounds.block_sound("self", "hit").is_none());
    assert!(sounds.block_sound("d", "hit").is_none());
}

#[test]
fn sj07_events_are_optional_only_with_a_base() {
    let stack = copies(
        BINDINGS,
        &[
            r#"{"block_sounds": {"based": {"base": "plain"}, "plain": {"events": {}}, "bare": {}},
              "entity_sounds": {"entities": {"pig": {"volume": 1.0}}}}"#,
        ],
    );
    let sounds = stack.sounds_json();
    assert_eq!(
        paths_with(sounds, "Expected \"events\" object"),
        ["block_sounds/bare", "entity_sounds/entities/pig"]
    );
    assert_eq!(found(sounds, "MissingRequiredField").len(), 2);
    assert!(
        found(sounds, "MissingRequiredField")
            .iter()
            .all(|d| d.severity == Severity::Error && d.kept)
    );
}

#[test]
fn a_section_or_entry_that_is_no_object_asks_for_no_events() {
    let stack = copies(
        BINDINGS,
        &[
            r#"{"individual_event_sounds": 5, "entity_sounds": {"defaults": "x", "entities": {"pig": 3}},
              "block_sounds": {"stone": "x"}}"#,
        ],
    );
    let sounds = stack.sounds_json();
    expect_none(sounds, "MissingRequiredField");
    // A block entry that is no object registers an empty type.
    assert!(sounds.block_sounds["stone"].sounds.events.is_empty());
    assert!(
        sounds.entity_sounds.entities["pig"]
            .sounds
            .events
            .is_empty()
    );
    // An object without `events` is asked for them, in each place.
    let stack = copies(
        BINDINGS,
        &[r#"{"individual_event_sounds": {}, "entity_sounds": {"defaults": {}, "entities": {}}}"#],
    );
    assert_eq!(
        paths_with(stack.sounds_json(), "Expected \"events\" object"),
        ["individual_event_sounds", "entity_sounds/defaults"]
    );
    // `events` of another type is as bad as none.
    let stack = copies(
        BINDINGS,
        &[r#"{"individual_event_sounds": {"events": []}}"#],
    );
    expect_one(
        stack.sounds_json(),
        "MissingRequiredField",
        Severity::Error,
        "Expected \"events\" object",
    );
}

#[test]
fn named_sounds_need_their_sounds_object() {
    let stack = copies(BINDINGS, &[r#"{"individual_named_sounds": {"volume": 1}}"#]);
    expect_one(
        stack.sounds_json(),
        "MissingRequiredField",
        Severity::Error,
        "Expected \"sounds\" object",
    );
    let stack = copies(
        BINDINGS,
        &[r#"{"individual_named_sounds": {"sounds": "x"}}"#],
    );
    expect_one(
        stack.sounds_json(),
        "MissingRequiredField",
        Severity::Error,
        "Expected \"sounds\" object",
    );
    for silent in [
        r#"{"individual_named_sounds": 5}"#,
        "{}",
        r#"{"individual_named_sounds": {"sounds": {}}}"#,
    ] {
        expect_none(
            copies(BINDINGS, &[silent]).sounds_json(),
            "MissingRequiredField",
        );
    }
    // Names compare with case.
    let stack = copies(
        BINDINGS,
        &[r#"{"individual_named_sounds": {"sounds": {"A": "x", "a": "y"}}}"#],
    );
    assert_eq!(stack.sounds_json().individual_named_sounds.sounds.len(), 2);
}

#[test]
fn sj10_a_legacy_ambient_event_is_also_a_named_sound() {
    let stack = copies(
        BINDINGS,
        &[
            r#"{"individual_event_sounds": {"events": {"ambient.cave": "cave", "ambient.warped_forest.mood": "mood", "break": "b"}},
              "individual_named_sounds": {"sounds": {"ambient.warped_forest.mood": "own"}}}"#,
        ],
    );
    let sounds = stack.sounds_json();
    let named = &sounds.individual_named_sounds.sounds;
    assert_eq!(named["ambient.cave"].sound, "cave");
    // A named sound of that name stays.
    assert_eq!(named["ambient.warped_forest.mood"].sound, "own");
    // No other event becomes a named sound.
    assert!(!named.contains_key("break"));
    assert_eq!(named.len(), 2);
    // The fallback runs without the section too.
    let stack = copies(
        BINDINGS,
        &[r#"{"individual_event_sounds": {"events": {"ambient.cave": "cave"}}}"#],
    );
    assert_eq!(
        stack.sounds_json().individual_named_sounds.sounds["ambient.cave"].sound,
        "cave"
    );
}

// The client biome is read only with the `biome` feature; without it the link pass has no
// ambient sound to ask about.
#[cfg(feature = "biome")]
#[test]
fn sj10b_a_client_biome_ambient_sound_that_is_no_named_sound_is_an_unknown_sound() {
    use bedrock_addon::pack::{PackRepository, PackSelection, WorldPacks};
    // One resource pack: `sounds.json` names `known.sound`, and the client biome's ambient
    // sounds use it for `loop` and `missing.sound` for `mood`.
    let biome = r#"{"format_version": "1.21.120", "minecraft:client_biome": {
        "description": {"identifier": "demo:cave"},
        "components": {"minecraft:ambient_sounds": {"loop": "known.sound", "mood": "missing.sound"}}}}"#;
    let pack = pack_bytes(
        1,
        Some("[1, 21, 120]"),
        &[
            (
                BINDINGS,
                br#"{"individual_named_sounds": {"sounds": {"known.sound": "x"}}}"#,
            ),
            ("biomes/cave.client_biome.json", biome.as_bytes()),
        ],
    );
    let world = WorldPacks {
        behavior: vec![],
        resource: vec![PackSelection {
            pack_id: pack.id().uuid,
            version: pack.id().version.clone(),
            subpack: None,
        }],
    };
    let mut repository = PackRepository::new();
    repository.add_pack(pack, bedrock_addon::pack::PackOrigin::World);
    let set = repository.stacks(&world, LoadOptions::default());
    // The link pass: one `UnknownSound` error, at the member that names the missing sound
    // (the linked-asset validator's line, an error).
    let report = set.link();
    let unknown: Vec<_> = report
        .iter()
        .filter(|d| d.code.name() == "UnknownSound")
        .collect();
    assert_eq!(unknown.len(), 1, "{unknown:#?}");
    assert_eq!(unknown[0].severity, Severity::Error);
    assert_eq!(
        unknown[0]
            .location
            .path
            .as_ref()
            .map(bedrock_addon::storage::PackPath::as_str),
        Some("biomes/cave.client_biome.json")
    );
    assert_eq!(
        unknown[0].location.json_path.to_string(),
        "minecraft:client_biome/components/minecraft:ambient_sounds/mood"
    );
}

#[test]
fn sj11_variant_errors() {
    let variant = r#"{"events": {"ambient": "v"}}"#;
    let text = format!(
        r#"{{"entity_sounds": {{"entities": {{
            "no_key": {{"events": {{}}, "variants": {{"map": {{"baby": {variant}}}}}}},
            "no_map": {{"events": {{}}, "variants": {{"key": "'baby'"}}}},
            "empty_map": {{"events": {{}}, "variants": {{"key": "'baby'", "map": {{}}}}}},
            "scalar_map": {{"events": {{}}, "variants": {{"key": "'baby'", "map": 5}}}},
            "empty": {{"events": {{}}, "variants": {{}}}},
            "none": {{"events": {{}}}},
            "fine": {{"events": {{}}, "variants": {{"key": "'baby'", "map": {{"baby": {variant}}}}}}}
        }}}}}}"#
    );
    let stack = copies(BINDINGS, &[&text]);
    let sounds = stack.sounds_json();
    assert_eq!(
        paths_with(
            sounds,
            "Expected \"key\" expression for entity with sound variants"
        ),
        ["entity_sounds/entities/no_key/variants"]
    );
    assert_eq!(
        paths_with(
            sounds,
            "Expected \"map\" list for entity with sound variant key"
        ),
        [
            "entity_sounds/entities/empty_map/variants",
            "entity_sounds/entities/no_map/variants",
            "entity_sounds/entities/scalar_map/variants"
        ]
    );
    assert_eq!(found(sounds, "MissingRequiredField").len(), 4);
    let entities = &sounds.entity_sounds.entities;
    assert!(entities["none"].variants.is_none());
    let empty = entities["empty"].variants.as_ref().unwrap();
    assert!(empty.key.is_none() && empty.map.is_empty());
    // The entity and its map are kept whatever the key.
    assert_eq!(
        entities["no_key"].variants.as_ref().unwrap().map["baby"].events["ambient"].sound,
        "v"
    );
    let fine = entities["fine"].variants.as_ref().unwrap();
    assert_eq!(fine.key.as_ref().unwrap().as_str(), "'baby'");
    // The transform stamped the pack's version (1.21.100 is Molang 13) where a key is.
    assert_eq!(entities["fine"].molang_version, Some(13));
    assert_eq!(entities["no_key"].molang_version, None);
}

#[test]
fn a_variant_starts_from_fresh_table_ranges() {
    let stack = copies(
        BINDINGS,
        &[
            r#"{"entity_sounds": {"entities": {"cat": {"volume": 0.5, "events": {"ambient": "a"},
            "variants": {"key": "'x'", "map": {"x": {"events": {"ambient": "b"}}}}}}}}"#,
        ],
    );
    let cat = &stack.sounds_json().entity_sounds.entities["cat"];
    assert_eq!(cat.sounds.events["ambient"].volume, range(0.5, 0.5));
    assert!(
        cat.variants.as_ref().unwrap().map["x"].events["ambient"]
            .volume
            .is_unset()
    );
}

#[test]
fn sj08_the_variant_key_keeps_the_supplying_packs_molang_version() {
    let cat = r#"{"entity_sounds": {"entities": {"cat": {"events": {"ambient": "a"},
        "variants": {"key": "'x'", "map": {"x": {"events": {"ambient": "b"}}}}}}}}"#;
    let over = r#"{"entity_sounds": {"entities": {"cat": {"events": {"ambient": "c"}}}}}"#;
    let stack = versioned(
        &[(Some("[1, 16, 0]"), cat), (Some("[1, 21, 100]"), over)],
        LoadOptions::default(),
    );
    let sounds = stack.sounds_json();
    let entry = &sounds.entity_sounds.entities["cat"];
    assert_eq!(entry.sounds.events["ambient"].sound, "c");
    assert_eq!(
        entry.molang_version,
        Some(0),
        "p0's version, not the top pack's 13"
    );
    let merged = sounds.json();
    let cat_json = merged
        .as_object()
        .unwrap()
        .get("entity_sounds")
        .unwrap()
        .as_object()
        .unwrap()
        .get("entities")
        .unwrap()
        .as_object()
        .unwrap()
        .get("cat")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(cat_json.get("__molang_version"), Some(&json("0")));
    expect_none(sounds, "SoundsJsonTableCleared");
}

#[test]
fn the_transform_overwrites_an_author_written_molang_version() {
    let text = r#"{"entity_sounds": {"entities": {"cat": {"__molang_version": 4, "events": {},
        "variants": {"key": "'x'", "map": {"x": {"events": {}}}}}}}}"#;
    let stack = versioned(&[(Some("[1, 21, 100]"), text)], LoadOptions::default());
    assert_eq!(
        stack.sounds_json().entity_sounds.entities["cat"].molang_version,
        Some(13)
    );
}

const SJ09_BASE: &str = r#"{"entity_sounds": {"defaults": {"events": {"ambient": "d"}},
        "entities": {"pig": {"events": {"ambient": "p"}}}},
    "interactive_sounds": {"entity_sounds": {"entities": {"cow": {"events": {"step": {"default": "s"}}}}}}}"#;

#[test]
fn sj09_an_overlay_without_entity_sounds_leaves_the_entity_table() {
    // Inferred from vanilla's data: twelve retail vanilla slices ship a
    // `sounds.json` without `entity_sounds`, one of them `{}`, and the game keeps its entity
    // sounds. A copy without the member inserts nothing.
    for overlay in [
        r#"{ "block_sounds": { "x": { "events": { "break": "dig.stone" } } } }"#,
        "{}",
    ] {
        let stack = copies(BINDINGS, &[SJ09_BASE, overlay]);
        let sounds = stack.sounds_json();
        let bindings = sounds;
        assert_eq!(
            bindings.entity_sounds.entities["pig"].sounds.events["ambient"].sound,
            "p"
        );
        assert_eq!(bindings.entity_sounds.defaults.events["ambient"].sound, "d");
        assert_eq!(bindings.interactive_sounds.entity_sounds.entities.len(), 1);
        expect_none(sounds, "SoundsJsonTableCleared");
        assert_eq!(sounds.diagnostics().count(), 0, "{:#?}", describe(sounds));
        // Nothing the copy did not write reaches the merged document.
        let merged = bindings.json().as_object().unwrap();
        let entities = merged
            .get("entity_sounds")
            .unwrap()
            .as_object()
            .unwrap()
            .get("entities")
            .unwrap();
        assert!(entities.is_object());
    }
}

#[test]
fn sj09_an_overlay_with_entity_sounds_but_no_entities_clears_the_table() {
    // The shape the folder's rule 21 describes: the section is there, its `entities` is not.
    let stack = copies(
        BINDINGS,
        &[
            SJ09_BASE,
            r#"{ "entity_sounds": { "defaults": { "events": { "ambient": "d2" } } } }"#,
        ],
    );
    let sounds = stack.sounds_json();
    let bindings = sounds;
    assert!(bindings.entity_sounds.entities.is_empty());
    assert_eq!(
        bindings.entity_sounds.defaults.events["ambient"].sound,
        "d2"
    );
    // The interactive table the loader reads is untouched.
    assert_eq!(bindings.interactive_sounds.entity_sounds.entities.len(), 1);
    let merged = bindings.json().as_object().unwrap();
    assert!(
        merged
            .get("entity_sounds")
            .unwrap()
            .as_object()
            .unwrap()
            .get("entities")
            .unwrap()
            .is_null()
    );
    expect_one(
        sounds,
        "SoundsJsonTableCleared",
        Severity::Info,
        "entity_sounds/entities",
    );
    assert_eq!(pack_of(found(sounds, "SoundsJsonTableCleared")[0]), Some(0));
    assert_eq!(sounds.diagnostics().count(), 1, "{:#?}", describe(sounds));
    // An `entity_sounds` that is no object is one without `entities` too.
    let stack = copies(BINDINGS, &[SJ09_BASE, r#"{ "entity_sounds": 5 }"#]);
    assert!(stack.sounds_json().entity_sounds.entities.is_empty());
    expect_one(
        stack.sounds_json(),
        "SoundsJsonTableCleared",
        Severity::Info,
        "entity_sounds/entities",
    );
}

#[test]
fn the_interactive_section_is_made_only_where_the_copy_has_it() {
    // The transform's second, inert insertion follows the same rule.
    let stack = copies(
        BINDINGS,
        &[r#"{"interactive_sounds": {"block_sounds": {}}, "block_sounds": {}}"#],
    );
    let merged = stack.sounds_json().json().clone();
    assert_eq!(
        merged,
        json(
            r#"{"interactive_sounds": {"block_sounds": {}, "entities": null}, "block_sounds": {}}"#
        )
    );
}

#[test]
fn an_overlay_with_its_own_table_or_over_no_table_clears_nothing() {
    let base = r#"{"entity_sounds": {"entities": {"pig": {"events": {"ambient": "p"}}}}}"#;
    let stack = copies(
        BINDINGS,
        &[
            base,
            r#"{"entity_sounds": {"entities": {"cow": {"events": {}}}}}"#,
        ],
    );
    expect_none(stack.sounds_json(), "SoundsJsonTableCleared");
    assert_eq!(stack.sounds_json().entity_sounds.entities.len(), 2);
    let stack = copies(
        BINDINGS,
        &[r#"{"block_sounds": {}}"#, r#"{"block_sounds": {}}"#],
    );
    expect_none(stack.sounds_json(), "SoundsJsonTableCleared");
    // The lowest copy has nothing below it to clear.
    expect_none(
        copies(BINDINGS, &[r#"{"block_sounds": {}}"#]).sounds_json(),
        "SoundsJsonTableCleared",
    );
}

#[test]
fn sj13_an_overlay_with_a_map_only_clears_the_key() {
    let base = r#"{"entity_sounds": {"entities": {"cat": {"events": {"ambient": "a"},
        "variants": {"key": "'x'", "map": {"x": {"events": {"ambient": "b"}}}}}}}}"#;
    let over = r#"{"entity_sounds": {"entities": {"cat": {"variants": {"map": {"y": {"events": {"ambient": "c"}}}}}}}}"#;
    let stack = copies(BINDINGS, &[base, over]);
    let sounds = stack.sounds_json();
    let variants = sounds.entity_sounds.entities["cat"]
        .variants
        .as_ref()
        .unwrap();
    assert!(variants.key.is_none());
    assert_eq!(variants.map.len(), 2);
    expect_one(
        sounds,
        "MissingRequiredField",
        Severity::Error,
        "Expected \"key\" expression for entity with sound variants",
    );
    expect_one(
        sounds,
        "SoundsJsonTableCleared",
        Severity::Info,
        "entity_sounds/entities/cat/variants/key",
    );
    // An overlay with its own key, or over an entity without one, clears nothing.
    let keyed = r#"{"entity_sounds": {"entities": {"cat": {"variants": {"key": "'y'", "map": {"y": {"events": {}}}}}}}}"#;
    expect_none(
        copies(BINDINGS, &[base, keyed]).sounds_json(),
        "SoundsJsonTableCleared",
    );
    let plain = r#"{"entity_sounds": {"entities": {"cat": {"events": {}}}}}"#;
    expect_none(
        copies(BINDINGS, &[plain, over]).sounds_json(),
        "SoundsJsonTableCleared",
    );
}

#[test]
fn sj12_an_interactive_variant_key_gets_no_injected_version() {
    let text = r#"{"interactive_sounds": {"entity_sounds": {"entities": {
        "x": {"events": {}, "variants": {"key": "'a'", "map": {"a": {"events": {"step": {"default": "s"}}}}}},
        "y": {"__molang_version": 9, "variants": {"key": "'a'", "map": {"a": {}}}}}}}}"#;
    let stack = versioned(&[(Some("[1, 21, 100]"), text)], LoadOptions::default());
    let sounds = stack.sounds_json();
    let entities = &sounds.interactive_sounds.entity_sounds.entities;
    assert_eq!(entities["x"].molang_version, None);
    assert_eq!(entities["y"].molang_version, Some(9));
    assert_eq!(
        entities["x"].variants.as_ref().unwrap().map["a"].events["step"]["default"].sound,
        "s"
    );
    // Ours: the info of rule 19, for the entry without a written version only.
    let infos: Vec<String> = found(sounds, "InvalidValue")
        .iter()
        .filter(|d| d.severity == Severity::Info)
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        infos,
        ["interactive_sounds/entity_sounds/entities/x/variants"]
    );
    expect_none(sounds, "MissingRequiredField");
}

#[test]
fn sj14_section_guards_and_interactive_defaults() {
    let stack = copies(
        BINDINGS,
        &[r#"{"interactive_sounds": {"entity_sounds": {
            "defaults": {"events": {"step": {"default": "x"}}},
            "entities": {"cow": {"events": {"step": {"": "skipped", "stone": {"sound": "y", "volume": 0.5}}, "notanevent": {"default": "z"}}},
                         "bare": {}}}}}"#],
    );
    let sounds = stack.sounds_json();
    expect_none(sounds, "MissingRequiredField");
    let interactive = &sounds.interactive_sounds.entity_sounds;
    let step = &interactive.defaults.events["step"]["default"];
    assert_eq!(
        (step.volume, step.pitch),
        (range(1.0, 1.0), range(1.0, 1.0))
    );
    let cow = &interactive.entities["cow"].sounds.events;
    assert_eq!(cow["step"].len(), 1, "the empty inner key is skipped");
    assert_eq!(cow["step"]["stone"].volume, range(0.5, 0.5));
    assert_eq!(cow["step"]["stone"].pitch, SoundRange::ONE);
    // `events` is never required here.
    assert!(interactive.entities["bare"].sounds.events.is_empty());
    // Ours (vanilla: dropped silently): the unknown event, an information.
    let unknown = found(sounds, "UnknownSound");
    assert_eq!(unknown.len(), 1, "{:#?}", describe(sounds));
    assert_eq!(unknown[0].severity, Severity::Info);
    assert_eq!(sounds.diagnostics().count(), 1, "{:#?}", describe(sounds));
}

#[test]
fn unknown_members_of_the_bindings_are_reported_and_the_known_ones_not() {
    let stack = copies(
        BINDINGS,
        &[r#"{"format_version": "1.0.0", "extra_section": 1,
              "individual_event_sounds": {"events": {"break": {"sound": "a", "loud": 1}}, "odd": 1},
              "interactive_sounds": {"entities": null, "odd": 1}}"#],
    );
    let paths: Vec<String> = found(stack.sounds_json(), "UnrecognizedField")
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        paths,
        [
            "extra_section",
            "individual_event_sounds/odd",
            "individual_event_sounds/events/break/loud",
            "interactive_sounds/odd"
        ]
    );
}

#[cfg(feature = "molang")]
mod molang {
    use std::sync::Arc;

    use bedrock_addon::diagnostic::Severity;
    use bedrock_addon::load::LoadOptions;
    use bedrock_addon::molang::MolangCheck;

    use super::super::common::{describe, expect_none, found};
    use super::versioned;

    fn checked() -> LoadOptions {
        LoadOptions {
            molang: Some(Arc::new(MolangCheck::new())),
            ..LoadOptions::default()
        }
    }

    fn entity(name: &str, key: &str) -> String {
        format!(
            r#""{name}": {{"events": {{}}, "variants": {{"key": "{key}", "map": {{"v": {{"events": {{}}}}}}}}}}"#
        )
    }

    #[test]
    fn sj15_variant_keys_compile_against_the_client_catalogue() {
        let text = format!(
            r#"{{"entity_sounds": {{"entities": {{{}, {}, {}, {}}}}}}}"#,
            entity(
                "a",
                "query.is_baby ? 'baby' : query.property('minecraft:sound_variant')"
            ),
            entity("b", "query.is_on_screen ? 'near' : 'far'"),
            entity("c", "query.distance_from_camera > 8 ? 'far' : 'near'"),
            entity("d", "1 +"),
        );
        let stack = versioned(&[(Some("[1, 21, 100]"), &text)], checked());
        let sounds = stack.sounds_json();
        let invalid = found(sounds, "InvalidMolang");
        assert!(!invalid.is_empty(), "{:#?}", describe(sounds));
        for finding in &invalid {
            assert_eq!(
                finding.location.json_path.to_string(),
                "entity_sounds/entities/d/variants/key"
            );
        }
        // The parser's lines are content-log errors and the file loads: kept errors.
        assert!(
            invalid
                .iter()
                .any(|d| d.severity == Severity::Error && d.kept),
            "{:#?}",
            describe(sounds)
        );
        assert!(invalid.iter().all(|d| !d.rejects()));
        expect_none(sounds, "MolangLint");
        assert_eq!(
            sounds.diagnostics().count(),
            invalid.len(),
            "{:#?}",
            describe(sounds)
        );
        // The entity, its map and its key are kept.
        let d = &sounds.entity_sounds.entities["d"];
        let variants = d.variants.as_ref().unwrap();
        assert_eq!(variants.key.as_ref().unwrap().as_str(), "1 +");
        assert_eq!(variants.map.len(), 1);
    }

    #[test]
    fn sj16_a_pack_without_a_min_engine_version_stamps_minus_one() {
        // A stack never holds such a pack (its manifest is refused), so the copy is read as
        // the file of a pack whose context has no `min_engine_version`.
        use bedrock_addon::document::Content;
        use bedrock_addon::families::sounds_json::SoundBindings;
        use bedrock_addon::load::LoadContext;
        use bedrock_addon::pack::PackType;
        use bedrock_addon::storage::PackPath;
        let text = format!(
            r#"{{"entity_sounds": {{"entities": {{{}}}}}}}"#,
            entity("e", "query.is_baby ? 'a' : 'b'")
        );
        let mut cx = LoadContext::for_pack(
            &checked(),
            uuid::Uuid::from_u128(7),
            PackType::Resource,
            None,
            false,
        );
        cx.molang_version = bedrock_addon::molang::pack_molang_version(None);
        assert_eq!(cx.molang_version.0, -1);
        let document =
            SoundBindings::parse(&PackPath::new("sounds.json").unwrap(), text.as_bytes(), &cx)
                .expect("the file loads");
        let e = &document.entity_sounds.entities["e"];
        assert_eq!(e.molang_version, Some(-1));
        let lines: Vec<String> = document
            .diagnostics()
            .iter()
            // a kept error
            .filter(|d| d.code.name() == "InvalidMolang" && d.severity == Severity::Error && d.kept)
            .map(ToString::to_string)
            .collect();
        assert!(
            lines.iter().any(|line| line.contains(
                "Failed to resolve query query.is_baby.  Either the query does not exist or it is not supported in this context."
            )),
            "{lines:#?}"
        );
        assert!(e.variants.as_ref().unwrap().key.is_some());
    }

    #[test]
    fn sj12_an_interactive_key_compiles_at_version_zero_or_the_authors() {
        // The key of an entry without `__molang_version` is a version-0 source; the author's
        // version is taken as written.
        let text = r#"{"interactive_sounds": {"entity_sounds": {"entities": {
                "old": {"variants": {"key": "query.is_baby ? 'a' : 'b'", "map": {"a": {}}}},
                "new": {"__molang_version": 13, "variants": {"key": "query.is_baby ? 'a' : 'b'", "map": {"a": {}}}}}}}}"#;
        let stack = versioned(&[(Some("[1, 21, 100]"), text)], checked());
        let sounds = stack.sounds_json();
        let keys = &sounds.interactive_sounds.entity_sounds.entities;
        assert_eq!(
            keys["old"]
                .variants
                .as_ref()
                .unwrap()
                .key
                .as_ref()
                .unwrap()
                .raw_version()
                .map(|v| v.0),
            Some(0)
        );
        assert_eq!(
            keys["new"]
                .variants
                .as_ref()
                .unwrap()
                .key
                .as_ref()
                .unwrap()
                .raw_version()
                .map(|v| v.0),
            Some(13)
        );
        expect_none(sounds, "MolangNotChecked");
    }
}

#[test]
fn without_the_compile_check_the_keys_are_counted_once() {
    let text = r#"{"entity_sounds": {"entities": {
        "a": {"events": {}, "variants": {"key": "1 +", "map": {"v": {"events": {}}}}},
        "b": {"events": {}, "variants": {"key": "'v'", "map": {"v": {"events": {}}}}},
        "c": {"events": {}}}}}"#;
    // `LoadOptions::default()` carries no compile check, with or without the feature.
    let stack = copies(BINDINGS, &[text]);
    let sounds = stack.sounds_json();
    expect_one(sounds, "MolangNotChecked", Severity::Info, "");
    assert_eq!(
        found(sounds, "MolangNotChecked")[0]
            .code
            .args_of(&bedrock_addon::diagnostic::codes::MOLANG_NOT_CHECKED),
        Some(&["2".to_owned()][..])
    );
    expect_none(sounds, "InvalidMolang");
    // No key, no note.
    let stack = copies(
        BINDINGS,
        &[r#"{"entity_sounds": {"entities": {"c": {"events": {}}}}}"#],
    );
    expect_clean(stack.sounds_json());
}

// The reach of each `base` chain is worked out at the read; a table edited afterwards is
// resolved again by the caller.
#[test]
fn editing_a_base_and_resolving_the_bases_again_changes_the_inherited_sound() {
    fn name(sound: Option<&bedrock_addon::families::sounds_json::SoundRef>) -> Option<&str> {
        sound.map(|sound| sound.sound.as_str())
    }
    let text = r#"{
        "block_sounds": {
            "stone": {"events": {"break": "dig.stone"}},
            "wood": {"base": "stone", "events": {"hit": "hit.wood"}},
            "plain": {"events": {"place": "use.plain"}}},
        "interactive_sounds": {"block_sounds": {
            "stone": {"events": {"fall": "fall.stone"}},
            "plain": {"events": {"jump": "jump.plain"}}}}}"#;
    let mut bindings = SoundBindings::from_json(json(text));
    assert_eq!(
        name(bindings.block_sound("wood", "break")),
        Some("dig.stone")
    );
    assert_eq!(name(bindings.block_sound("plain", "break")), None);
    assert_eq!(
        name(bindings.interactive_block_sound("plain", "fall")),
        None
    );

    bindings.block_sounds["plain"].base = "wood".to_owned();
    bindings.block_sounds["wood"].base = String::new();
    bindings.interactive_sounds.block_sounds["plain"].base = "stone".to_owned();
    bindings.resolve_block_bases();

    assert_eq!(
        name(bindings.block_sound("plain", "hit")),
        Some("hit.wood"),
        "`plain` takes the events of its new base"
    );
    assert_eq!(
        name(bindings.block_sound("plain", "break")),
        None,
        "`wood` has no base any more, so neither it nor `plain` reaches `stone`"
    );
    assert_eq!(name(bindings.block_sound("wood", "break")), None);
    assert_eq!(
        name(bindings.block_sound("plain", "place")),
        Some("use.plain")
    );
    assert_eq!(
        name(bindings.interactive_block_sound("plain", "fall")),
        Some("fall.stone"),
        "the interactive table is resolved too"
    );
}

#[test]
fn an_unparsable_bindings_base_names_its_path() {
    let stack = copies(BINDINGS, &["nope", "{}"]);
    expect_one(
        stack.sounds_json(),
        "MalformedInput",
        Severity::Error,
        "Could not parse file at the top of the stack: sounds.json.",
    );
}

#[test]
fn no_file_in_the_stack_is_an_empty_document_without_findings() {
    let stack = copies("textures/readme.txt", &["x"]);
    let sounds = stack.sounds_json();
    expect_clean(sounds);
    assert!(sounds.block_sounds.is_empty());
}
