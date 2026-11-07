//! Spawn-group documents: document parsing (`sg10`–`sg28e`) and versions and envelope
//! (`sg30`–`sg39a`). Rows marked [test] reproduce vanilla's `SpawnGroupDataLoaderUnitTests` with
//! their template.

use bedrock_addon::diagnostic::{Code, Diagnostics, Kind, Severity};
use bedrock_addon::document::{Content, Document};
use bedrock_addon::families::spawn_groups::SpawnGroup;
use bedrock_addon::families::spawn_rules::{Difficulty, MobSpawnRules};
use bedrock_addon::id::EntityId;

use crate::common::{closes_with, context, has_text, load_group, names, paths, round_trip, with};

/// Vanilla's unit-test template with one condition `{ body }`.
fn template(body: &str) -> String {
    format!(
        r#"{{ "format_version": "1.8.0", "minecraft:spawn_groups": {{ "description": {{ "identifier": "test:test_mob", "population_control": "monster" }}, "conditions": [ {{ {body} }} ] }} }}"#
    )
}

/// Loads `text` and requires it to load.
#[track_caller]
fn loads(text: &str) -> Document<SpawnGroup> {
    load_group(text).unwrap_or_else(|r| panic!("rejected: {:#?}", r.diagnostics))
}

/// Loads `text` and requires it to be rejected.
#[track_caller]
fn rejected(text: &str) -> Diagnostics {
    match load_group(text) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The group's one condition: its spawn weight, what the runtime keeps, the findings.
#[track_caller]
fn one(body: &str) -> (i32, MobSpawnRules, Diagnostics) {
    let document = loads(&template(body));
    let condition = &document.conditions.as_ref().unwrap()[0];
    let rules = document.spawn_rules().remove(0);
    (
        condition.spawn_weight(),
        rules,
        document.diagnostics().clone(),
    )
}

fn invalid_type(code: &Code) -> bool {
    code.kind() == Kind::WrongType
}

fn invalid_value(code: &Code) -> bool {
    code.kind() == Kind::InvalidValue
}

fn missing_field(code: &Code) -> bool {
    code.kind() == Kind::MissingRequired
}

#[test]
fn sg10_description() {
    let document = loads(&template(r#""minecraft:spawns_on_surface": {}"#));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.description.identifier.as_str(), "test:test_mob");
    assert_eq!(
        document.description.population_control.as_deref(),
        Some("monster")
    );
}

#[test]
fn sg11_missing_identifier() {
    let d = rejected(r#"{ "format_version": "1.8.0", "minecraft:spawn_groups": {} }"#);
    assert!(
        has_text(
            &d,
            missing_field,
            Severity::Error,
            "Missing identifier in spawn rule file"
        ),
        "{d:#?}"
    );
    assert!(closes_with(&d, "Failed to load spawn groups"), "{d:#?}");
    assert_eq!(d.len(), 2);
}

#[test]
fn sg12_weight() {
    assert_eq!(one(r#""minecraft:weight": { "default": 50 }"#).0, 50);
    assert_eq!(one(r#""minecraft:spawns_on_surface": {}"#).0, 100);
    assert_eq!(one(r#""minecraft:weight": { "default": -3 }"#).0, 0);
    let (weight, _, d) = one(r#""minecraft:weight": 5"#);
    assert_eq!(weight, 100);
    assert_eq!(names(&d, Severity::Info), ["spawn/weight_object"]);
}

#[test]
fn sg13_density() {
    let (_, r, _) = one(r#""minecraft:density_limit": { "surface": 50, "underground": 50 }"#);
    assert_eq!((r.surface_cap, r.underground_cap), (50, 50));
    let (_, r, _) = one(r#""minecraft:density_limit": {}"#);
    assert_eq!((r.surface_cap, r.underground_cap), (-1, -1));
}

#[test]
fn sg14_block_filters() {
    let (_, r, d) = one(
        r#""minecraft:spawns_on_block_filter": ["test:block"], "minecraft:spawns_on_block_prevented_filter": ["test:block"], "minecraft:spawns_above_block_filter": { "blocks": "test:block", "distance": 10 }"#,
    );
    assert!(d.is_empty(), "{d:#?}");
    assert_eq!(r.spawns_on_blocks.len(), 1);
    assert_eq!(r.spawns_on_blocks_prevented.len(), 1);
    assert_eq!(
        (r.spawns_above_blocks.len(), r.above_block_distance),
        (1, 10)
    );
    let (_, r, _) = one(r#""minecraft:spawns_above_block_filter": { "blocks": "test:block" }"#);
    assert_eq!(r.above_block_distance, 1);
}

#[test]
fn sg15_herd() {
    let fields = r#""min_size": 2, "max_size": 4, "event": "e", "event_skip_count": 1, "initial_event": "i", "initial_event_count": 3"#;
    let (_, object, _) = one(&format!(r#""minecraft:herd": {{ {fields} }}"#));
    let (_, array, _) = one(&format!(r#""minecraft:herd": [{{ {fields} }}]"#));
    assert_eq!(object.herds, array.herds);
    let (_, defaults, _) = one(r#""minecraft:herd": {}"#);
    let herd = &defaults.herds[0];
    assert_eq!(
        (herd.min_size, herd.max_size, herd.event.as_str()),
        (1, 1, "")
    );
    assert_eq!(
        (
            herd.event_skip_count,
            herd.initial_event.as_str(),
            herd.initial_event_count
        ),
        (0, "", 0)
    );
}

#[test]
fn sg16_permute() {
    let (_, r, d) = one(
        r#""minecraft:permute_type": [{ "weight": 30, "entity_type": "test:test_permute_1" }, { "weight": 70, "entity_type": "test:test_permute_2", "guaranteed_count": 1, "min_guaranteed": 2 }]"#,
    );
    assert_eq!(
        r.permutations,
        vec![
            (30, EntityId::parse("test:test_permute_1")),
            (70, EntityId::parse("test:test_permute_2"))
        ]
    );
    assert_eq!(
        r.guaranteed,
        vec![(1, EntityId::parse("test:test_permute_2"))]
    );
    let unknown = with(&d, |c| matches!(c.id(), "core/unrecognized_field"));
    assert_eq!(
        paths(&unknown),
        ["minecraft:spawn_groups/conditions/0/minecraft:permute_type/1/min_guaranteed"]
    );
}

#[test]
fn sg17_brightness() {
    let (_, r, _) = one(r#""minecraft:brightness_filter": { "min": 2, "max": 4 }"#);
    assert_eq!((r.brightness_min, r.brightness_max), (2, 4));
    let (_, r, _) = one(r#""minecraft:brightness_filter": { "min": -5, "max": 99 }"#);
    assert_eq!((r.brightness_min, r.brightness_max), (0, 15));
}

#[test]
fn sg18_height() {
    let (_, r, _) = one(r#""minecraft:height_filter": { "min": 2, "max": 4 }"#);
    assert_eq!((r.height_min, r.height_max), (2, 4));
    let (_, r, _) = one(r#""minecraft:height_filter": {}"#);
    assert_eq!((r.height_min, r.height_max), (24, 128));
    let (_, r, d) = one(r#""minecraft:height_filter": { "min": 9, "max": 3 }"#);
    assert!(has_text(
        &d,
        invalid_value,
        Severity::Warning,
        "Spawn rules for minecraft:distance_filter specified a min height > max."
    ));
    assert_eq!((r.height_min, r.height_max), (3, 9));
}

#[test]
fn sg19_bool_flags() {
    let (_, r, d) = one(
        r#""minecraft:spawns_on_surface": {}, "minecraft:spawns_underground": {}, "minecraft:spawns_underwater": {}, "minecraft:spawns_lava": {}, "minecraft:disallow_spawns_in_bubble": {}, "minecraft:is_persistent": {}, "minecraft:is_experimental": {}"#,
    );
    assert!(d.is_empty());
    assert!(r.surface_spawner && r.underground_spawner && r.underwater_spawner && r.lava_spawner);
    assert!(!r.bubble_spawner && r.persistent && r.experimental);
    let (_, r, _) = one(r#""minecraft:spawns_on_surface": false"#);
    assert!(r.surface_spawner);
}

#[test]
fn sg20_biome_filter() {
    let document = loads(&template(
        r#""minecraft:biome_filter": {"test": "has_biome_tag", "operator": "==", "value": "animal"}"#,
    ));
    assert!(document.diagnostics().is_empty());
    let condition = &document.conditions.as_ref().unwrap()[0];
    assert!(condition.biome_filter.is_some());
    // `MobSpawnRules` has no biome filter: the runtime group drops it.
    let _: Vec<MobSpawnRules> = document.spawn_rules();
}

#[test]
fn sg21_difficulty() {
    let (_, r, _) = one(r#""minecraft:difficulty_filter": { "min": "easy", "max": "hard" }"#);
    assert_eq!(
        (r.difficulty_min, r.difficulty_max),
        (Difficulty::Easy, Difficulty::Hard)
    );
    let (_, r, _) = one(r#""minecraft:difficulty_filter": { "min": "EASY" }"#);
    assert_eq!(r.difficulty_min, Difficulty::Easy);
    let (_, r, d) = one(r#""minecraft:difficulty_filter": { "min": "extreme" }"#);
    assert_eq!(r.difficulty_min, Difficulty::Peaceful);
    assert_eq!(names(&d, Severity::Info), ["core/invalid_enum_value"]);
}

#[test]
fn sg22_distance() {
    let (_, r, d) = one(r#""minecraft:distance_filter": { "min": 2, "max": 4 }"#);
    assert!(d.is_empty());
    assert_eq!((r.spawn_distance_min, r.spawn_distance_max), (2, 4));
}

#[test]
fn sg23_distance_above_cap() {
    let (_, r, d) = one(r#""minecraft:distance_filter": { "min": 2, "max": 1025 }"#);
    assert_eq!((r.spawn_distance_min, r.spawn_distance_max), (2, 1024));
    assert_eq!(r.spawn_distance_cap, 1024);
    // C7's clamp is an info.
    assert_eq!(names(&d, Severity::Info), ["core/clamped_value"]);
    // The spawn-rules parser caps the same condition at 128.
    let rules =
        crate::common::condition(r#""minecraft:distance_filter": { "min": 2, "max": 1025 }"#).1;
    assert_eq!(rules.spawn_distance_max, 128);
}

#[test]
fn sg24_world_age() {
    let (_, r, _) = one(r#""minecraft:world_age_filter": { "min": 2 }"#);
    assert_eq!((r.world_age_min, r.world_age_max), (2, u64::MAX));
    let (_, r, d) = one(r#""minecraft:world_age_filter": { "min": 5, "max": 1 }"#);
    assert!(has_text(
        &d,
        invalid_value,
        Severity::Warning,
        "Spawn rules for minecraft:world_age_filter specified a min world age > max."
    ));
    assert_eq!((r.world_age_min, r.world_age_max), (1, 5));
}

#[test]
fn sg25_delay() {
    let (_, r, d) = one(
        r#""minecraft:delay_filter": { "min": 2, "max": 4, "identifier": "test:test_permute_1", "spawn_chance": 20 }"#,
    );
    assert!(d.is_empty());
    assert_eq!(
        (
            r.delay_min,
            r.delay_max,
            r.delay_identifier.as_str(),
            r.delay_spawn_chance
        ),
        (2, 4, "test:test_permute_1", 20)
    );
    let (_, r, d) = one(r#""minecraft:delay_filter": { "min": 2, "max": 4 }"#);
    assert!(has_text(
        &d,
        missing_field,
        Severity::Warning,
        "Spawn rules for test:test_mob specified a delay without a target identifier."
    ));
    assert_eq!((r.delay_min, r.delay_max), (2, 4));
}

#[test]
fn sg26_mob_event() {
    let (_, r, _) = one(r#""minecraft:mob_event_filter": { "event": "minecraft:village_raid" }"#);
    assert_eq!(r.mob_event, "minecraft:village_raid");
    let (_, r, d) = one(r#""minecraft:mob_event_filter": "x""#);
    assert!(
        has_text(
            &d,
            invalid_type,
            Severity::Warning,
            "Spawn rules for minecraft:mob_event_filter in test:test_mob need to be put inside a json object - it has to start with an \"{\" and end with a \"}\" - using default values"
        ),
        "{d:#?}"
    );
    assert_eq!(r.mob_event, "");
}

#[test]
fn sg27_player_in_village() {
    let (_, r, d) = one(
        r#""minecraft:player_in_village_filter": { "distance": 10, "village_border_tolerance": 4 }"#,
    );
    assert!(d.is_empty());
    assert_eq!((r.village_distance, r.village_border_tolerance), (10, 4));
    let (_, r, d) = one(r#""minecraft:player_in_village_filter": {}"#);
    assert!(has_text(
        &d,
        missing_field,
        Severity::Warning,
        "Missing \"distance\" parameter for minecraft:player_in_village_filter"
    ));
    assert_eq!((r.village_distance, r.village_border_tolerance), (0, 10));
}

#[test]
fn sg28_spawn_event_ignored() {
    let document = loads(&template(r#""minecraft:spawn_event": {"event": "x"}"#));
    assert!(document.diagnostics().is_empty());
    assert!(
        document.conditions.as_ref().unwrap()[0]
            .spawn_event
            .is_some()
    );
}

#[test]
fn sg28a_value_coercion() {
    let (weight, _, d) = one(r#""minecraft:weight": { "default": "50" }"#);
    assert_eq!(weight, 0);
    assert_eq!(names(&d, Severity::Info), ["core/invalid_type"]);
    let (_, r, d) = one(r#""minecraft:herd": { "min_size": true, "max_size": 2.9 }"#);
    assert_eq!((r.herds[0].min_size, r.herds[0].max_size), (1, 2));
    assert_eq!(
        names(&d, Severity::Info),
        ["core/invalid_type", "core/invalid_type"]
    );
    let (_, r, d) = one(r#""minecraft:difficulty_filter": { "min": 1 }"#);
    assert_eq!(r.difficulty_min, Difficulty::Peaceful);
    assert_eq!(names(&d, Severity::Info), ["core/invalid_type"]);
}

#[test]
fn sg28b_untyped_elements() {
    let text = r#"{ "format_version": "1.8.0", "minecraft:spawn_groups": { "description": { "identifier": "test:test_mob" }, "conditions": [ 5, { "minecraft:herd": [ 3, { "min_size": 2, "max_size": 2 } ] } ] } }"#;
    let document = loads(text);
    assert_eq!(
        paths(&with(document.diagnostics(), invalid_type)),
        [
            "minecraft:spawn_groups/conditions/0",
            "minecraft:spawn_groups/conditions/1/minecraft:herd/0"
        ]
    );
    let rules = document.spawn_rules();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].herds.len(), 1);
    assert_eq!(
        (rules[0].herds[0].min_size, rules[0].herds[0].max_size),
        (2, 2)
    );
}

#[test]
fn sg28c_permute_weight_zero() {
    let (_, r, d) = one(
        r#""minecraft:permute_type": [ { "entity_type": "x:a", "weight": 0 }, { "entity_type": "x:b", "weight": 0, "guaranteed_count": 2 } ]"#,
    );
    assert!(r.permutations.is_empty());
    assert_eq!(r.guaranteed, vec![(2, EntityId::parse("x:b"))]);
    // ours, informations
    assert_eq!(
        names(&d, Severity::Info),
        ["core/invalid_value", "core/invalid_value"]
    );
}

#[test]
fn sg28d_delay_clamp() {
    let (_, r, _) =
        one(r#""minecraft:delay_filter": { "min": -5, "max": 3, "identifier": "x:y" }"#);
    assert_eq!((r.delay_min, r.delay_max), (0, 3));
    let (_, r, d) =
        one(r#""minecraft:delay_filter": { "min": 4, "max": -2, "identifier": "x:y" }"#);
    assert!(has_text(
        &d,
        invalid_value,
        Severity::Warning,
        "Spawn rules for test:test_mob specified a min delay > max."
    ));
    assert_eq!((r.delay_min, r.delay_max), (0, 4));
    let (_, r, d) = one(r#""minecraft:delay_filter": 7"#);
    assert_eq!(names(&d, Severity::Info), ["spawn/delay_object"]);
    assert_eq!(
        (
            r.delay_min,
            r.delay_max,
            r.delay_identifier.as_str(),
            r.delay_spawn_chance
        ),
        (0, 0, "", 100)
    );
    let (_, r, _) = one(r#""minecraft:spawns_on_surface": {}"#);
    assert_eq!(r.delay_spawn_chance, 0);
}

#[test]
fn sg28e_block_descriptor_kept() {
    let (_, r, d) = one(r#""minecraft:spawns_on_block_filter": [ 5, "minecraft:grass_block" ]"#);
    assert!(has_text(
        &d,
        invalid_value,
        Severity::Error,
        "BlockDescriptor requires a string, block object, or any_tag object. Cannot parse 5"
    ));
    assert_eq!(r.spawns_on_blocks.len(), 1);
    let (_, r, d) = one(r#""minecraft:spawns_on_block_filter": []"#);
    assert!(d.is_empty() && r.spawns_on_blocks.is_empty());
}

// ---- versions and envelope ----

/// A group document of `version` (a JSON value) for `x:g` with one empty condition.
fn versioned(version: &str) -> String {
    format!(
        r#"{{"format_version": {version}, "minecraft:spawn_groups": {{"description": {{"identifier": "x:g"}}, "conditions": [{{}}]}}}}"#
    )
}

#[test]
fn sg30_below_floor() {
    let d = rejected(&versioned(r#""1.7.0""#));
    assert!(
        has_text(
            &d,
            |c| matches!(c.id(), "core/no_parser_for_version"),
            Severity::Error,
            "unsupported version: no parser available for version 1.7.0"
        ),
        "{d:#?}"
    );
    assert!(closes_with(&d, "Failed to load spawn groups"));
}

#[test]
fn sg31_above_engine() {
    let d = rejected(&versioned(r#""99.0.0""#));
    assert_eq!(names(&d, Severity::Error)[0], "core/no_parser_for_version");
}

#[test]
fn sg32_missing_version() {
    let d = rejected(
        r#"{"minecraft:spawn_groups": {"description": {"identifier": "x:g"}, "conditions": [{}]}}"#,
    );
    assert!(
        has_text(
            &d,
            |c| matches!(c.id(), "core/no_valid_version"),
            Severity::Error,
            "no valid version passed by the caller or found in the input"
        ),
        "{d:#?}"
    );
    assert!(closes_with(&d, "Failed to load spawn groups"));
}

#[test]
fn sg33_malformed_version() {
    for version in [r#""abc""#, "1.11", "true"] {
        let d = rejected(&versioned(version));
        assert!(
            has_text(
                &d,
                |c| matches!(c.id(), "core/malformed_format_version"),
                Severity::Error,
                "Malformed format version"
            ),
            "{version}: {d:#?}"
        );
    }
}

#[test]
fn sg34_array_version() {
    let document = loads(&versioned("[1, 11, 0]"));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.format_version.to_string(), "1.11.0");
}

#[test]
fn sg35_raw_envelope() {
    let text = r#"{"format_version": "1.11.0", "minecraft:spawn_groups": {"description": {"identifier": "x:g", "note": 1}, "conditions": [{}], "other": 2}, "extra": 1}"#;
    let document = loads(text);
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    // Kept for write-back.
    assert!(document.root_extra.get("extra").is_some());
    let written = round_trip(
        document.value(),
        "definitions/spawn_groups/x.json",
        &context(),
    );
    assert!(written.contains("\"extra\"") && written.contains("\"other\""));
}

#[test]
fn sg36_wrong_payload_key() {
    let text = r#"{"format_version": "1.11.0", "minecraft:spawn_rules": {"description": {"identifier": "x:g"}}}"#;
    let d = rejected(text);
    // `MissingPayload` with vanilla's text (V7).
    assert_eq!(
        names(&d, Severity::Error)[0],
        "spawn/missing_identifier_payload",
        "{d:#?}"
    );
    assert!(
        has_text(
            &d,
            |c| matches!(c.id(), "spawn/missing_identifier_payload"),
            Severity::Error,
            "Missing identifier in spawn rule file"
        ),
        "{d:#?}"
    );
    assert!(closes_with(&d, "Failed to load spawn groups"));
}

#[test]
fn sg37_root_not_object() {
    let d = rejected(r#"[ { "format_version": "1.11.0" } ]"#);
    assert!(
        has_text(
            &d,
            |c| matches!(c.id(), "spawn/not_an_object"),
            Severity::Error,
            "Spawn rules json was not formatted as an object"
        ),
        "{d:#?}"
    );
    let d = rejected("[]");
    assert!(
        has_text(
            &d,
            |c| matches!(c.id(), "core/no_valid_version"),
            Severity::Error,
            "no valid version passed by the caller or found in the input"
        ),
        "{d:#?}"
    );
}

#[test]
fn sg39_json_syntax_error() {
    let d = rejected(
        r#"{ "format_version": "1.11.0", "minecraft:spawn_groups": { "description": { "identifier": "x:g" }, } "#,
    );
    assert_eq!(names(&d, Severity::Error), ["core/syntax"], "{d:#?}");
}

#[test]
fn sg39_json_syntax_error_deviation() {
    // Vanilla: no message, an empty document under ""; ours rejects the file without vanilla's closing line.
    let d = rejected(r#"{ "format_version": "1.11.0", "#);
    assert!(!closes_with(&d, "Failed to load spawn groups"));
}

#[test]
fn sg39a_nested_version_only() {
    let text = r#"{"minecraft:spawn_groups": { "format_version": "1.11.0", "description": { "identifier": "x:g" }, "conditions": [ {} ] }}"#;
    let d = rejected(text);
    assert_eq!(
        names(&d, Severity::Error)[0],
        "core/no_valid_version",
        "{d:#?}"
    );
}

#[test]
fn sg39a_nested_version_only_deviation() {
    // Vanilla routes on the nested version and then fails with "Spawn rules format for x:g is
    // invalid"; ours reads only the
    // root member and reports the missing version.
    let d = rejected(
        r#"{"minecraft:spawn_groups": { "format_version": "1.11.0", "description": { "identifier": "x:g" } }}"#,
    );
    assert!(!d.iter().any(|f| f.code.to_string().contains("is invalid")));
}

#[test]
fn sg38_no_conditions() {
    for text in [
        r#"{"format_version": "1.11.0", "minecraft:spawn_groups": {"description": {"identifier": "x:g"}}}"#,
        r#"{"format_version": "1.11.0", "minecraft:spawn_groups": {"description": {"identifier": "x:g"}, "conditions": {}}}"#,
    ] {
        let document = loads(text);
        // ours, an information
        assert_eq!(
            names(document.diagnostics(), Severity::Info),
            ["core/empty_document"],
            "{text}"
        );
        assert!(document.spawn_rules().is_empty());
    }
}

#[test]
fn sg38_no_conditions_deviation() {
    // Vanilla loads it silently and never registers it; ours is an information. The registry side is in
    // `discovery.rs`.
    let document = loads(
        r#"{"format_version": "1.11.0", "minecraft:spawn_groups": {"description": {"identifier": "x:g"}, "conditions": []}}"#,
    );
    let empty = with(document.diagnostics(), |c| {
        matches!(c.id(), "core/empty_document")
    });
    assert_eq!(empty[0].severity, Severity::Info);
}

#[test]
fn a_group_round_trips() {
    let document = loads(&template(
        r#""minecraft:herd": {"min_size": 3, "max_size": 3, "event": "minecraft:spawn_for_raid"}, "minecraft:permute_type": [{"weight": 100, "entity_type": "minecraft:pillager"}]"#,
    ));
    let text = round_trip(
        document.value(),
        "definitions/spawn_groups/x.json",
        &context(),
    );
    let again = SpawnGroup::parse(
        &bedrock_addon::storage::PackPath::from_static("definitions/spawn_groups/x.json"),
        text.as_bytes(),
        &context(),
    )
    .unwrap();
    assert_eq!(again.spawn_rules(), document.spawn_rules());
}

/// A 1.11.0 group `x:g` whose one condition is `{ body }`.
fn group(body: &str) -> String {
    format!(
        r#"{{"format_version": "1.11.0", "minecraft:spawn_groups": {{"description": {{"identifier": "x:g"}}, "conditions": [{{ {body} }}]}}}}"#
    )
}

#[test]
fn an_unknown_condition_key_is_an_info() {
    let document = loads(&group(r#""minecraft:spawns_on_moon": {}"#));
    assert_eq!(
        names(document.diagnostics(), Severity::Info),
        ["core/unrecognized_field"]
    );
}

#[test]
fn the_identifier_is_kept_verbatim() {
    let text = r#"{"format_version": "1.11.0", "minecraft:spawn_groups": {"description": {"identifier": "pig"}, "conditions": [{}]}}"#;
    assert_eq!(loads(text).description.identifier.as_str(), "pig");
}

#[test]
fn the_engine_version_loads() {
    let engine = bedrock_addon::version::ENGINE_VERSION;
    let text = format!(
        r#"{{"format_version": "{engine}", "minecraft:spawn_groups": {{"description": {{"identifier": "x:g"}}, "conditions": [{{}}]}}}}"#
    );
    assert!(loads(&text).diagnostics().is_empty());
}
