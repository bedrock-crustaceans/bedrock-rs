//! The standard (1.8.0-shaped) spawn rules: the fixture rows `s01`, `s05`–`s09o`.

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::document::Content;
use bedrock_addon::families::spawn_rules::{Difficulty, OneOrMany, PopulationControl};
use bedrock_addon::id::EntityId;
use bedrock_addon::shared::BlockDescriptor;

use super::common::{
    closes_with, condition, context, document, fixture, has_text, legacy, load, loads, names,
    paths, rejected, round_trip, standard, with,
};

fn invalid_type(code: &DiagnosticCode) -> bool {
    matches!(code, DiagnosticCode::InvalidType { .. })
}

fn invalid_value(code: &DiagnosticCode) -> bool {
    matches!(code, DiagnosticCode::InvalidValue { .. })
}

fn missing_field(code: &DiagnosticCode) -> bool {
    matches!(code, DiagnosticCode::MissingRequiredField { .. })
}

fn invalid_enum(code: &DiagnosticCode) -> bool {
    matches!(code, DiagnosticCode::InvalidEnumValue { .. })
}

/// The object-form text of `key` for `x:mob`.
fn object_form(key: &str) -> String {
    format!(
        "Spawn rules for minecraft:{key} in x:mob need to be put inside a json object - it has to start with an \"{{\" and end with a \"}}\" - using default values"
    )
}

#[test]
fn s01_spawn_legacy_all_keys() {
    let text = fixture("spawn", "s01_spawn_legacy_all_keys.json");
    let document = loads(&text, &context());
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let rules = standard(&document);
    assert_eq!(
        rules.description.population_control(),
        Some(PopulationControl::Animal)
    );
    let conditions = rules.conditions.as_ref().unwrap();
    assert_eq!(conditions.len(), 2);
    let first = &conditions[0];
    assert_eq!(first.spawn_weight(), 8);
    assert!(first.spawn_event.is_some());
    assert!(first.biome_filter.is_some());
    let r = first.to_spawn_rules();
    assert_eq!(r.rarity, 3);
    assert_eq!((r.surface_cap, r.underground_cap), (4, 2));
    assert_eq!(r.spawns_on_blocks.len(), 1);
    assert_eq!(r.spawns_on_blocks_prevented.len(), 2);
    assert_eq!(
        (r.spawns_above_blocks.len(), r.above_block_distance),
        (1, 3)
    );
    assert_eq!(r.herds.len(), 1);
    let herd = &r.herds[0];
    assert_eq!((herd.min_size, herd.max_size), (2, 4));
    assert_eq!(
        (herd.event.as_str(), herd.event_skip_count),
        ("minecraft:entity_born", 1)
    );
    assert_eq!(
        (herd.initial_event.as_str(), herd.initial_event_count),
        ("minecraft:start", 2)
    );
    assert_eq!(
        r.permutations,
        vec![
            (95, EntityId::parse("x:mob")),
            (5, EntityId::parse("x:mob_baby"))
        ]
    );
    assert_eq!(r.guaranteed, vec![(1, EntityId::parse("x:mob_baby"))]);
    assert_eq!((r.brightness_min, r.brightness_max), (7, 15));
    assert!(r.adjust_for_weather);
    assert_eq!((r.height_min, r.height_max), (0, 64));
    assert!(r.surface_spawner && r.underground_spawner && r.underwater_spawner && r.lava_spawner);
    assert!(!r.bubble_spawner);
    assert_eq!(
        (r.difficulty_min, r.difficulty_max),
        (Difficulty::Easy, Difficulty::Normal)
    );
    assert_eq!((r.spawn_distance_min, r.spawn_distance_max), (12, 100));
    assert!(r.experimental && r.persistent);
    assert_eq!((r.world_age_min, r.world_age_max), (100, 2000));
    assert_eq!((r.delay_min, r.delay_max), (600, 660));
    assert_eq!(
        (r.delay_identifier.as_str(), r.delay_spawn_chance),
        ("x:mob", 25)
    );
    assert_eq!(r.mob_event, "minecraft:pillager_patrols_event");
    assert_eq!((r.village_distance, r.village_border_tolerance), (48, 32));
    // The herd array form and the permutation object form.
    let second = &conditions[1];
    assert!(matches!(second.herd, Some(OneOrMany::Many(ref h)) if h.len() == 2));
    assert!(matches!(second.permute_type, Some(OneOrMany::One(_))));
    let r = second.to_spawn_rules();
    assert_eq!(r.herds.len(), 2);
    assert_eq!(
        (r.herds[1].min_size, r.herds[1].event.as_str()),
        (3, "minecraft:make_white")
    );
    assert_eq!(second.spawn_weight(), 100);
    // The capture is written back byte for byte.
    assert_eq!(document.value().to_json().unwrap(), text);
    round_trip(document.value(), "spawn_rules/x.json", &context());
}

#[test]
fn s05_below_floor() {
    let d = rejected(
        &document("1.7.0", r#"{"description": {"identifier": "x:mob"}}"#),
        &context(),
    );
    assert!(
        has_text(
            &d,
            |c| matches!(c, DiagnosticCode::UnsupportedVersion { .. }),
            Severity::Error,
            "unsupported version: no parser available for version 1.7.0"
        ),
        "{d:#?}"
    );
    assert!(closes_with(&d, "Failed to load spawn rules"), "{d:#?}");
}

#[test]
fn s06_permute_guaranteed_count() {
    let (_, rules, d) = condition(
        r#""minecraft:permute_type": [{"weight": 1, "entity_type": "x:a", "guaranteed_count": 2, "min_guaranteed": 3}]"#,
    );
    assert_eq!(rules.guaranteed, vec![(2, EntityId::parse("x:a"))]);
    let unknown = with(&d, |c| {
        matches!(c, DiagnosticCode::UnrecognizedField { .. })
    });
    assert_eq!(
        paths(&unknown),
        ["minecraft:spawn_rules/conditions/0/minecraft:permute_type/0/min_guaranteed"]
    );
    assert_eq!(unknown[0].severity, Severity::Info);
}

#[test]
fn s07_bad_format_version() {
    let d = rejected(
        &document("abc", r#"{"description": {"identifier": "x:mob"}}"#),
        &context(),
    );
    assert!(
        has_text(
            &d,
            |c| matches!(c, DiagnosticCode::MalformedFormatVersion { .. }),
            Severity::Error,
            "Malformed format version"
        ),
        "{d:#?}"
    );
    assert!(closes_with(&d, "Failed to load spawn rules"));
}

#[test]
fn s07a_array_format_version() {
    let text = r#"{"format_version": [1, 8, 0], "minecraft:spawn_rules": {"description": {"identifier": "x:mob", "population_control": "animal"}, "conditions": []}}"#;
    let document = loads(text, &context());
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(standard(&document).effective_version().to_string(), "1.8.0");
}

#[test]
fn s07b_json_syntax_error() {
    let text = r#"{ "format_version": "1.8.0", "minecraft:spawn_rules": { "description": { "identifier": "x:mob" }, } "#;
    let d = rejected(text, &context());
    assert_eq!(names(&d, Severity::Error), ["MalformedInput"], "{d:#?}");
    // Vanilla logs nothing, not even "Failed to load spawn rules" (a deviation).
    assert!(!closes_with(&d, "Failed to load spawn rules"));
}

#[test]
fn s07b_json_syntax_error_deviation() {
    // Vanilla stores an empty document under "" silently; ours rejects the file.
    assert!(load(r#"{ "format_version": "1.8.0", "#, &context()).is_err());
}

#[test]
fn s08_spawn_event_silent() {
    let (condition, _, d) =
        condition(r#""minecraft:spawn_event": {"event": "minecraft:entity_spawned"}"#);
    assert!(d.is_empty(), "{d:#?}");
    assert!(condition.spawn_event.is_some());
}

#[test]
fn s09_descriptor_errors() {
    let (_, rules, d) = condition(
        r#""minecraft:spawns_on_block_filter": [7, {"name": 5}, {"name": "minecraft:stone", "states": {"x": 1.5}}]"#,
    );
    assert!(rules.spawns_on_blocks.is_empty());
    for text in [
        "BlockDescriptor requires a string, block object, or any_tag object. Cannot parse 7",
        // vanilla's literal ends in a newline (spawn_groups C15, pe)
        "Only strings are valid for name. The invalid name value: 5\n",
        "Invalid state type during block parsing: block name minecraft:stone, state name x, state type 3. State type can only be bool, int, or string.",
    ] {
        assert!(
            has_text(&d, invalid_value, Severity::Error, text),
            "{text}: {d:#?}"
        );
    }
    assert!(!d.has_errors());
}

#[test]
fn s09e_permute_weight_default() {
    for body in [
        r#""minecraft:permute_type": [{"entity_type": "minecraft:zombie"}]"#,
        r#""minecraft:permute_type": {"entity_type": "minecraft:zombie"}"#,
    ] {
        let (_, rules, d) = condition(body);
        assert!(d.is_empty(), "{d:#?}");
        assert_eq!(
            rules.permutations,
            vec![(1, EntityId::parse("minecraft:zombie"))]
        );
    }
}

#[test]
fn s09f_village_distance_missing() {
    let text = "Missing \"distance\" parameter for minecraft:player_in_village_filter";
    let (_, rules, d) =
        condition(r#""minecraft:player_in_village_filter": {"village_border_tolerance": 4}"#);
    assert!(
        has_text(&d, missing_field, Severity::Warning, text),
        "{d:#?}"
    );
    assert_eq!(
        (rules.village_distance, rules.village_border_tolerance),
        (0, 4)
    );
    let (_, rules, d) = condition(r#""minecraft:player_in_village_filter": {}"#);
    assert!(
        has_text(&d, missing_field, Severity::Warning, text),
        "{d:#?}"
    );
    assert_eq!(
        (rules.village_distance, rules.village_border_tolerance),
        (0, 10)
    );
}

#[test]
fn s09d_unrecorded_defaults() {
    let (_, r, d) = condition(r#""minecraft:height_filter": {}"#);
    assert!(d.is_empty());
    assert_eq!((r.height_min, r.height_max), (24, 128));
    let (_, r, _) = condition("");
    assert_eq!((r.height_min, r.height_max), (i32::MIN, i32::MAX));
    let (_, r, _) = condition(r#""minecraft:brightness_filter": {"min": -5, "max": 99}"#);
    assert_eq!((r.brightness_min, r.brightness_max), (0, 15));
    // min > max: swapped, with a warning.
    let (_, r, d) = condition(r#""minecraft:distance_filter": {"min": 9, "max": 3}"#);
    assert_eq!((r.spawn_distance_min, r.spawn_distance_max), (3, 9));
    assert_eq!(with(&d, invalid_value).len(), 1, "{d:#?}");
    let (_, r, d) = condition(r#""minecraft:world_age_filter": {"min": 5, "max": 1}"#);
    assert_eq!((r.world_age_min, r.world_age_max), (1, 5));
    assert_eq!(with(&d, invalid_value).len(), 1, "{d:#?}");
    let (_, r, d) =
        condition(r#""minecraft:delay_filter": {"min": 4, "max": 2, "identifier": "x"}"#);
    assert_eq!((r.delay_min, r.delay_max), (2, 4));
    assert_eq!(with(&d, invalid_value).len(), 1, "{d:#?}");
    // Difficulty names.
    let (_, r, d) = condition(r#""minecraft:difficulty_filter": {"min": "EASY"}"#);
    assert!(d.is_empty(), "{d:#?}");
    assert_eq!(
        (r.difficulty_min, r.difficulty_max),
        (Difficulty::Easy, Difficulty::Hard)
    );
    let (_, r, d) =
        condition(r#""minecraft:difficulty_filter": {"min": "extreme", "max": "nope"}"#);
    assert_eq!(
        (r.difficulty_min, r.difficulty_max),
        (Difficulty::Peaceful, Difficulty::Hard)
    );
    assert_eq!(with(&d, invalid_enum).len(), 2, "{d:#?}");
    assert!(
        with(&d, invalid_enum)
            .iter()
            .all(|f| f.severity == Severity::Info)
    );
    let (c, r, d) = condition(r#""minecraft:difficulty_filter": 1"#);
    assert_eq!(
        (r.difficulty_min, r.difficulty_max),
        (Difficulty::Peaceful, Difficulty::Hard)
    );
    assert_eq!(with(&d, invalid_type).len(), 1, "{d:#?}");
    assert!(c.difficulty_filter.is_none());
    // Ignored shapes, each an `InvalidType` warning (ours; vanilla silent).
    for (body, key) in [
        (
            r#""minecraft:spawns_above_block_filter": 3"#,
            "minecraft:spawns_above_block_filter",
        ),
        (
            r#""minecraft:player_in_village_filter": 3"#,
            "minecraft:player_in_village_filter",
        ),
        (r#""minecraft:herd": 3"#, "minecraft:herd"),
        (r#""minecraft:permute_type": 3"#, "minecraft:permute_type"),
    ] {
        let (c, r, d) = condition(body);
        let found = with(&d, invalid_type);
        assert_eq!(
            paths(&found),
            [format!("minecraft:spawn_rules/conditions/0/{key}")],
            "{body}"
        );
        assert_eq!(found[0].severity, Severity::Info);
        assert!(!d.has_errors());
        assert_eq!(c.spawns_above_block_filter, None);
        assert_eq!(c.player_in_village_filter, None);
        assert_eq!(c.herd, None);
        assert_eq!(c.permute_type, None);
        assert_eq!((r.village_distance, r.village_border_tolerance), (0, 0));
        assert!(r.herds.is_empty() && r.permutations.is_empty());
        assert_eq!(r.above_block_distance, 0);
    }
}

#[test]
fn s09d_ignored_shapes_deviation() {
    // Vanilla ignores these shapes silently; ours warns.
    let (_, _, d) = condition(r#""minecraft:herd": "x""#);
    assert_eq!(names(&d, Severity::Info), ["InvalidType"]);
    let (_, _, d) = condition(r#""minecraft:difficulty_filter": {"min": "extreme"}"#);
    assert_eq!(names(&d, Severity::Info), ["InvalidEnumValue"]);
}

#[test]
fn s09c_object_form_messages() {
    let (_, r, d) = condition(r#""minecraft:brightness_filter": 7"#);
    assert!(
        has_text(
            &d,
            invalid_type,
            Severity::Warning,
            &object_form("brightness_filter")
        ),
        "{d:#?}"
    );
    assert_eq!(
        (r.brightness_min, r.brightness_max, r.adjust_for_weather),
        (0, 15, false)
    );
    let (_, r, d) = condition(r#""minecraft:height_filter": 7"#);
    assert!(
        has_text(
            &d,
            invalid_type,
            Severity::Warning,
            &object_form("height_filter")
        ),
        "{d:#?}"
    );
    assert_eq!((r.height_min, r.height_max), (24, 128));
    let (_, r, d) = condition(r#""minecraft:height_filter": {"min": 9, "max": 3}"#);
    assert!(has_text(
        &d,
        invalid_value,
        Severity::Warning,
        "Spawn rules for minecraft:distance_filter specified a min height > max."
    ));
    assert_eq!((r.height_min, r.height_max), (3, 9));
    // The other object-form keys.
    for key in [
        "weight",
        "density_limit",
        "distance_filter",
        "delay_filter",
        "mob_event_filter",
    ] {
        let (_, _, d) = condition(&format!(r#""minecraft:{key}": 7"#));
        assert!(
            has_text(&d, invalid_type, Severity::Warning, &object_form(key)),
            "{key}: {d:#?}"
        );
    }
}

#[test]
fn s09b_unknown_condition_key() {
    let (c, _, d) = condition(r#""minecraft:spawns_on_moon": {}"#);
    let found = with(&d, |c| {
        matches!(c, DiagnosticCode::UnrecognizedField { .. })
    });
    assert_eq!(
        paths(&found),
        ["minecraft:spawn_rules/conditions/0/minecraft:spawns_on_moon"]
    );
    assert_eq!(found[0].severity, Severity::Info);
    assert!(c.extra.get("minecraft:spawns_on_moon").is_some());
}

#[test]
fn s09b_unknown_condition_key_deviation() {
    // Vanilla never looks at the key; ours warns and keeps it.
    let (c, _, d) = condition(r#""minecraft:spawns_on_moon": {}"#);
    assert_eq!(names(&d, Severity::Info), ["UnrecognizedField"]);
    let written = c.extra.get("minecraft:spawns_on_moon").unwrap();
    assert!(written.as_object().is_some_and(|o| o.is_empty()));
}

#[test]
fn s09g_distance_world_age_delay() {
    let (_, r, d) = condition("");
    assert!(d.is_empty());
    assert_eq!((r.spawn_distance_min, r.spawn_distance_max), (24, 128));
    assert_eq!((r.world_age_min, r.world_age_max), (0, u64::MAX));
    let (_, r, _) = condition(r#""minecraft:distance_filter": {}"#);
    assert_eq!((r.spawn_distance_min, r.spawn_distance_max), (24, 128));
    let (_, r, d) = condition(r#""minecraft:distance_filter": {"min": -4, "max": 500}"#);
    assert_eq!((r.spawn_distance_min, r.spawn_distance_max), (0, 128));
    assert!(!d.has_errors());
    let (_, r, d) = condition(r#""minecraft:distance_filter": 1"#);
    assert!(has_text(
        &d,
        invalid_type,
        Severity::Warning,
        &object_form("distance_filter")
    ));
    assert_eq!((r.spawn_distance_min, r.spawn_distance_max), (24, 128));
    let (_, r, d) = condition(r#""minecraft:world_age_filter": 1"#);
    assert!(
        has_text(
            &d,
            invalid_type,
            Severity::Warning,
            "Spawn rules for minecraft:world_age_filter need to be put inside a json object - it has to start with an \"{\" and end with a \"}\" - using default values"
        ),
        "{d:#?}"
    );
    assert_eq!((r.world_age_min, r.world_age_max), (0, u64::MAX));
    let (_, r, _) =
        condition(r#""minecraft:delay_filter": {"min": -3, "max": 2, "identifier": "x"}"#);
    assert_eq!(
        (r.delay_min, r.delay_max, r.delay_spawn_chance),
        (0, 2, 100)
    );
    let (_, r, d) = condition(r#""minecraft:delay_filter": {"min": 1, "max": 2}"#);
    assert!(
        has_text(
            &d,
            missing_field,
            Severity::Warning,
            "Spawn rules for x:mob specified a delay without a target identifier."
        ),
        "{d:#?}"
    );
    assert_eq!((r.delay_min, r.delay_max), (1, 2));
    let (_, r, d) = condition(r#""minecraft:delay_filter": 1"#);
    assert!(has_text(
        &d,
        invalid_type,
        Severity::Warning,
        &object_form("delay_filter")
    ));
    assert_eq!((r.delay_min, r.delay_max), (0, 0));
}

#[test]
fn s09h_population_control() {
    let pool = |pool: &str| {
        loads(
            &document(
                "1.8.0",
                &format!(r#"{{"description": {{"identifier": "x:mob"{pool}}}, "conditions": []}}"#),
            ),
            &context(),
        )
    };
    let monster = pool(r#", "population_control": "Monster""#);
    assert!(monster.diagnostics().is_empty());
    assert_eq!(
        standard(&monster).description.population_control(),
        Some(PopulationControl::Monster)
    );
    let boss = pool(r#", "population_control": "boss""#);
    assert!(
        has_text(
            boss.diagnostics(),
            invalid_enum,
            Severity::Info,
            "No population control specified for 'x:mob'"
        ),
        "{:#?}",
        boss.diagnostics()
    );
    assert_eq!(standard(&boss).description.population_control(), None);
    for absent in [pool(""), pool(r#", "population_control": """#)] {
        let found = with(absent.diagnostics(), missing_field);
        assert_eq!(
            paths(&found),
            ["minecraft:spawn_rules/description/population_control"]
        );
        assert_eq!(found[0].severity, Severity::Info);
    }
}

#[test]
fn s09h_population_control_deviation() {
    // Vanilla leaves the rules unregistered without a content-log line; ours warns. The
    // registry side is in `stack.rs`.
    let text = document("1.8.0", r#"{"description": {"identifier": "x:mob"}}"#);
    assert_eq!(
        names(loads(&text, &context()).diagnostics(), Severity::Info),
        ["MissingRequiredField"]
    );
}

#[test]
fn s09i_missing_identifier() {
    for text in [
        document("1.8.0", r#"{"description": {}}"#),
        document("1.8.0", r#"{"description": {"identifier": 5}}"#),
        r#"{"format_version": "1.8.0", "minecraft:spawn_groups": {"description": {"identifier": "x:mob"}}}"#.to_owned(),
    ] {
        let d = rejected(&text, &context());
        assert!(
            has_text(&d, missing_field, Severity::Error, "Missing identifier in spawn rule file"),
            "{text}: {d:#?}"
        );
        assert!(closes_with(&d, "Failed to load spawn rules"), "{d:#?}");
    }
}

#[test]
fn s09j_flag_presence() {
    let (_, r, d) =
        condition(r#""minecraft:spawns_on_surface": false, "minecraft:is_persistent": 0"#);
    assert!(d.is_empty(), "{d:#?}");
    assert!(r.surface_spawner && r.persistent);
    let (_, r, _) = condition(r#""minecraft:disallow_spawns_in_bubble": {}"#);
    assert!(!r.bubble_spawner);
    let (_, r, _) = condition("");
    assert!(r.bubble_spawner);
    assert_eq!((r.village_distance, r.village_border_tolerance), (0, 0));
}

#[test]
fn s09k_value_coercion() {
    let (c, r, d) = condition(r#""minecraft:weight": {"default": "50"}"#);
    assert_eq!(c.spawn_weight(), 0);
    assert_eq!(
        paths(&with(&d, invalid_type)),
        ["minecraft:spawn_rules/conditions/0/minecraft:weight/default"]
    );
    let _ = r;
    let (_, r, d) = condition(r#""minecraft:herd": {"min_size": true, "max_size": 2.9}"#);
    assert_eq!((r.herds[0].min_size, r.herds[0].max_size), (1, 2));
    assert_eq!(with(&d, invalid_type).len(), 2, "{d:#?}");
    let (_, r, d) = condition(r#""minecraft:difficulty_filter": {"min": 1}"#);
    assert_eq!(r.difficulty_min, Difficulty::Peaceful);
    assert_eq!(with(&d, invalid_type).len(), 1, "{d:#?}");
    assert!(with(&d, invalid_enum).is_empty());
    assert!(d.iter().all(|f| f.severity == Severity::Info));
}

/// the low 32 bits of an integer no `int` holds, and the 5 of a `null` rarity, are
/// vanilla's values; ours tells each at the member, since the written document
/// holds the value read and not the one in the file.
#[test]
fn an_integer_no_int_holds_and_a_null_rarity_are_told_deviation() {
    let (c, r, d) = condition(r#""minecraft:weight": {"default": 4294967304, "rarity": null}"#);
    assert_eq!((c.spawn_weight(), r.rarity), (8, 5));
    assert_eq!(
        paths(&with(&d, invalid_value)),
        ["minecraft:spawn_rules/conditions/0/minecraft:weight/default"]
    );
    assert_eq!(
        paths(&with(&d, invalid_type)),
        ["minecraft:spawn_rules/conditions/0/minecraft:weight/rarity"]
    );
    assert_eq!(names(&d, Severity::Info).len(), d.len(), "{d:#?}");
    // In range, and an absent rarity: nothing is said.
    let (c, r, d) = condition(r#""minecraft:weight": {"default": 2147483647}"#);
    assert_eq!((c.spawn_weight(), r.rarity), (i32::MAX, 0));
    assert!(d.is_empty(), "{d:#?}");
    let (_, r, d) = condition(r#""minecraft:herd": {"min_size": -2147483649, "max_size": 2}"#);
    assert_eq!(r.herds[0].max_size, 2);
    assert_eq!(
        paths(&with(&d, invalid_value)),
        ["minecraft:spawn_rules/conditions/0/minecraft:herd/min_size"]
    );
}

#[test]
fn s09k_value_coercion_deviation() {
    // Vanilla coerces silently; ours warns, same value.
    let (c, _, d) = condition(r#""minecraft:weight": {"default": [1]}"#);
    assert_eq!(c.spawn_weight(), 0);
    assert_eq!(names(&d, Severity::Info), ["InvalidType"]);
}

#[test]
fn s09l_untyped_elements() {
    let document = loads(
        &legacy(r#"5, {"minecraft:herd": [3, {"min_size": 2, "max_size": 2}]}"#),
        &context(),
    );
    let d = document.diagnostics();
    assert_eq!(
        paths(&with(d, invalid_type)),
        [
            "minecraft:spawn_rules/conditions/0",
            "minecraft:spawn_rules/conditions/1/minecraft:herd/0"
        ]
    );
    let conditions = standard(&document).conditions.as_ref().unwrap();
    assert_eq!(conditions.len(), 1);
    let herds = conditions[0].to_spawn_rules().herds;
    assert_eq!(herds.len(), 1);
    assert_eq!((herds[0].min_size, herds[0].max_size), (2, 2));
}

#[test]
fn s09l_untyped_elements_deviation() {
    // Vanilla adds a default condition / herd for a non-object element; ours skips it.
    let document = loads(&legacy("[]"), &context());
    assert!(standard(&document).conditions.as_ref().unwrap().is_empty());
}

#[test]
fn s09m_permute_weight_zero() {
    let (_, r, d) = condition(
        r#""minecraft:permute_type": [{"entity_type": "x:a", "weight": 0}, {"entity_type": "x:b", "weight": 0, "guaranteed_count": 2}]"#,
    );
    assert!(r.permutations.is_empty());
    assert_eq!(r.guaranteed, vec![(2, EntityId::parse("x:b"))]);
    let found = with(&d, invalid_value);
    assert_eq!(found.len(), 2, "{d:#?}");
    // ours, informations
    assert!(found.iter().all(|f| f.severity == Severity::Info));
}

#[test]
fn s09m_permute_weight_zero_deviation() {
    // Vanilla leaves the entity out of the pick silently; ours is an information.
    let (_, _, d) = condition(r#""minecraft:permute_type": {"entity_type": "x:a", "weight": -1}"#);
    assert_eq!(names(&d, Severity::Info), ["InvalidValue"]);
}

#[test]
fn s09n_delay_clamp() {
    let (_, r, _) =
        condition(r#""minecraft:delay_filter": {"min": -5, "max": 3, "identifier": "x:y"}"#);
    assert_eq!((r.delay_min, r.delay_max), (0, 3));
    let (_, r, d) =
        condition(r#""minecraft:delay_filter": {"min": 4, "max": -2, "identifier": "x:y"}"#);
    assert!(
        has_text(
            &d,
            invalid_value,
            Severity::Warning,
            "Spawn rules for x:mob specified a min delay > max."
        ),
        "{d:#?}"
    );
    assert_eq!((r.delay_min, r.delay_max), (0, 4));
    let (_, r, d) = condition(r#""minecraft:delay_filter": 7"#);
    assert!(has_text(
        &d,
        invalid_type,
        Severity::Warning,
        &object_form("delay_filter")
    ));
    assert_eq!(
        (
            r.delay_min,
            r.delay_max,
            r.delay_identifier.as_str(),
            r.delay_spawn_chance
        ),
        (0, 0, "", 100)
    );
    let (_, r, _) = condition("");
    assert_eq!(r.delay_spawn_chance, 0);
}

#[test]
fn s09o_block_descriptor_kept() {
    let (c, r, d) =
        condition(r#""minecraft:spawns_on_block_filter": [5, "minecraft:grass_block"]"#);
    assert!(
        has_text(
            &d,
            invalid_value,
            Severity::Error,
            "BlockDescriptor requires a string, block object, or any_tag object. Cannot parse 5"
        ),
        "{d:#?}"
    );
    assert_eq!(
        r.spawns_on_blocks,
        vec![BlockDescriptor::block(bedrock_addon::id::BlockId::parse(
            "minecraft:grass_block"
        ))]
    );
    assert!(c.spawns_on_block_filter.is_some());
    let (_, r, d) = condition(r#""minecraft:spawns_on_block_filter": []"#);
    assert!(d.is_empty() && r.spawns_on_blocks.is_empty());
}

#[test]
fn s09o_block_descriptor_kept_deviation() {
    // Vanilla keeps an empty descriptor in the list; ours drops the entry after the warning.
    let (_, r, _) = condition(r#""minecraft:spawns_on_block_filter": [5]"#);
    assert!(r.spawns_on_blocks.is_empty());
}

#[test]
fn s09a_weight_defaults() {
    let weight = |body: &str| {
        let (c, r, d) = condition(body);
        assert!(with(&d, missing_field).is_empty(), "{d:#?}");
        (c.spawn_weight(), r.rarity, d)
    };
    assert_eq!(weight("").0, 100);
    assert_eq!(weight(r#""minecraft:weight": {"default": -3}"#).0, 0);
    assert_eq!(weight(r#""minecraft:weight": {}"#).0, 100);
    assert_eq!(weight(r#""minecraft:weight": {"default": 8}"#).1, 0);
    assert_eq!(
        weight(r#""minecraft:weight": {"default": 8, "rarity": 3}"#).1,
        3
    );
    let (w, _, d) = weight(r#""minecraft:weight": 5"#);
    assert_eq!(w, 100);
    assert!(has_text(
        &d,
        invalid_type,
        Severity::Warning,
        &object_form("weight")
    ));
}

#[test]
fn a_missing_format_version_is_1_8_0_with_a_warning_deviation() {
    // vanilla rejects it; ours parses it at the
    // floor with the warning
    let text = r#"{"minecraft:spawn_rules": {"description": {"identifier": "x:mob", "population_control": "animal"}}}"#;
    let document = loads(text, &context());
    assert_eq!(
        names(document.diagnostics(), Severity::Info),
        ["MissingFormatVersion"]
    );
    assert_eq!(standard(&document).effective_version().to_string(), "1.8.0");
}

#[test]
fn a_root_that_is_not_an_object_is_vanillas_message() {
    let d = rejected(r#"[{"format_version": "1.8.0"}]"#, &context());
    assert!(
        has_text(
            &d,
            |c| matches!(c, DiagnosticCode::MalformedInput { .. }),
            Severity::Error,
            "Spawn rules json was not formatted as an object"
        ),
        "{d:#?}"
    );
}

#[test]
fn write_back_keeps_what_the_author_wrote() {
    // a member of the wrong shape is kept as written (and still read as vanilla reads
    // it); an absent `format_version` is not invented.
    let text = r#"{"minecraft:spawn_rules": {"description": {"identifier": "x:mob", "population_control": "animal"}, "conditions": [{"minecraft:height_filter": 7, "minecraft:delay_filter": "x", "minecraft:herd": 3, "minecraft:spawns_on_block_filter": null}]}}"#;
    let document = loads(text, &context());
    let rules = standard(&document);
    assert_eq!(rules.format_version, None);
    let condition = &rules.conditions.as_ref().unwrap()[0];
    for key in [
        "minecraft:height_filter",
        "minecraft:delay_filter",
        "minecraft:herd",
        "minecraft:spawns_on_block_filter",
    ] {
        assert!(condition.extra.get(key).is_some(), "{key}");
    }
    // Kept members are not reported again as unknown.
    let unknown = with(document.diagnostics(), |c| {
        matches!(c, DiagnosticCode::UnrecognizedField { .. })
    });
    assert!(unknown.is_empty(), "{unknown:#?}");
    let r = condition.to_spawn_rules();
    assert_eq!((r.height_min, r.height_max), (24, 128));
    assert_eq!(r.delay_spawn_chance, 100);
    let written = round_trip(document.value(), "spawn_rules/x.json", &context());
    assert!(!written.contains("format_version"), "{written}");
    assert!(
        written.contains("\"minecraft:height_filter\": 7"),
        "{written}"
    );
    assert!(
        written.contains("\"minecraft:spawns_on_block_filter\": null"),
        "{written}"
    );
}

#[test]
fn unknown_root_description_and_in_object_members_deviation() {
    // at any unknown member. Ours warns about the root, the description and members inside a
    // condition's objects too, and keeps them.
    let text = r#"{"format_version": "1.8.0", "extra": 1, "minecraft:spawn_rules": {"description": {"identifier": "x:mob", "population_control": "animal", "note": "x"}, "conditions": [{"minecraft:weight": {"default": 2, "bonus": 1}}]}}"#;
    let document = loads(text, &context());
    let unknown = with(document.diagnostics(), |c| {
        matches!(c, DiagnosticCode::UnrecognizedField { .. })
    });
    assert_eq!(
        paths(&unknown),
        [
            "extra",
            "minecraft:spawn_rules/description/note",
            "minecraft:spawn_rules/conditions/0/minecraft:weight/bonus",
        ]
    );
    assert!(unknown.iter().all(|d| d.severity == Severity::Info));
    let rules = standard(&document);
    assert!(rules.root_extra.get("extra").is_some());
    assert!(rules.description.extra.get("note").is_some());
}

#[test]
fn a_member_written_as_null_reads_as_absent_and_is_not_written_back() {
    // jsoncpp's accessors treat `null` as missing, so the member takes its default; the
    // document keeps no trace of the `null` and write-back omits it (an authored `null`
    // is not preserved, nothing is invented).
    let document = loads(
        &legacy(r#"{"minecraft:herd": {"min_size": null, "max_size": 3}}"#),
        &context(),
    );
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let condition = &standard(&document).conditions.as_ref().unwrap()[0];
    let OneOrMany::One(herd) = condition.herd.as_ref().unwrap() else {
        panic!("one herd");
    };
    assert_eq!((herd.min_size, herd.max_size), (None, Some(3)));
    assert_eq!(condition.to_spawn_rules().herds[0].min_size, 1);
    let written = round_trip(document.value(), "spawn_rules/x.json", &context());
    assert!(!written.contains("min_size"), "{written}");
}
