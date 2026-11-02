//! The game's schema readers of entity components, goals and event responses: what the load
//! tells and keeps. The table of readers is checked by an xtask test (`entity_schemas::tests`);
//! the per-reader cases are the tests here.

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::Content;
use bedrock_addon_core::json::Value;

use super::common::{cx, doc, json, outcome, stable};

/// A document at `version` whose `components` hold `key` with `value` (JSON text).
fn wrapped(version: &str, key: &str, value: &str) -> String {
    doc(
        version,
        &format!(
            r#"{{"description": {{"identifier": "test:mob"}}, "components": {{"{key}": {value}}}}}"#
        ),
    )
}

/// The written component `key` of `text` (which must load), with the findings.
fn read(text: &str, key: &str) -> (Option<Value>, Diagnostics) {
    let loaded = outcome(text, &cx());
    assert!(
        matches!(
            loaded.value,
            Some(bedrock_addon_core::versioning::Loaded::Stable(_))
        ),
        "the entity is refused: {:#?}",
        loaded.diagnostics
    );
    let (entity, found) = stable(text, &cx());
    let written = json(&entity.to_json().expect("the entity writes"));
    let component = written
        .as_object()
        .and_then(|o| o.get("minecraft:entity"))
        .and_then(Value::as_object)
        .and_then(|e| e.get("components"))
        .and_then(Value::as_object)
        .and_then(|c| c.get(key))
        .cloned();
    (component, found)
}

/// The kept errors of `found` with their code, path tail and text.
fn kept(found: &Diagnostics) -> Vec<(String, String, String)> {
    found
        .iter()
        .filter(|f| f.kept)
        .map(|f: &Diagnostic| {
            assert_eq!(f.severity, Severity::Error, "{f:#?}");
            (
                f.code.name().to_owned(),
                f.location.json_path.to_string(),
                f.code.to_string(),
            )
        })
        .collect()
}

/// Whether `found` has a warning or an error that is not kept.
fn loud(found: &Diagnostics) -> Vec<&Diagnostic> {
    found
        .iter()
        .filter(|f| f.severity >= Severity::Warning && !f.kept)
        .collect()
}

/// From 1.15.0 a member a schema reader does not know fails its validation: the
/// game logs "child '%s' not valid here." at error level, parses nothing (the component keeps
/// its defaults) and loads the entity.
#[test]
fn an_unknown_member_of_a_schema_read_component_is_a_kept_error_and_defaults_the_component() {
    let (ageable, found) = read(
        &wrapped(
            "1.26.60",
            "minecraft:ageable",
            r#"{"duration": 30, "bogus": 1}"#,
        ),
        "minecraft:ageable",
    );
    assert_eq!(
        kept(&found),
        [(
            "UnrecognizedField".to_owned(),
            "minecraft:entity/components/minecraft:ageable/bogus".to_owned(),
            "child 'bogus' not valid here.".to_owned()
        )],
        "{found:#?}"
    );
    assert!(loud(&found).is_empty(), "{found:#?}");
    assert_eq!(ageable, Some(json("{}")));
}

/// Below 1.15.0 the reader is lenient: an unknown member is taken out without a word, the rest is read. The crate tells it as an
/// information.
#[test]
fn below_1_15_0_an_unknown_member_is_taken_out_and_the_rest_read() {
    let (ageable, found) = read(
        &wrapped(
            "1.13.0",
            "minecraft:ageable",
            r#"{"duration": 30, "bogus": 1}"#,
        ),
        "minecraft:ageable",
    );
    assert!(kept(&found).is_empty(), "{found:#?}");
    assert!(loud(&found).is_empty(), "{found:#?}");
    assert!(
        found.iter().any(|f| f.code.name() == "UnrecognizedField"
            && f.severity == Severity::Info
            && f.location
                .json_path
                .to_string()
                .ends_with("minecraft:ageable/bogus")),
        "{found:#?}"
    );
    assert_eq!(ageable, Some(json(r#"{"duration": 30}"#)));
}

/// A value of a type no option takes is logged in both modes ("unknown child schema
/// option type.  Allowed types: %s"); strict, the component keeps
/// its defaults; lenient, the member is taken out and the rest read. A float takes a bool
/// (`!isNull && isNumeric`).
#[test]
fn a_value_of_another_type_is_a_kept_error_in_both_modes() {
    let value = r#"{"duration": "long", "grow_up": {"event": "test:grow"}}"#;
    for (version, written) in [
        ("1.26.60", "{}"),
        ("1.13.0", r#"{"grow_up": {"event": "test:grow"}}"#),
    ] {
        let (ageable, found) = read(
            &wrapped(version, "minecraft:ageable", value),
            "minecraft:ageable",
        );
        assert_eq!(
            kept(&found),
            [(
                "InvalidType".to_owned(),
                "minecraft:entity/components/minecraft:ageable/duration".to_owned(),
                "unknown child schema option type.  Allowed types:  'float'".to_owned()
            )],
            "{version}: {found:#?}"
        );
        assert!(loud(&found).is_empty(), "{version}: {found:#?}");
        assert_eq!(ageable, Some(json(written)), "{version}");
    }
    let (ageable, found) = read(
        &wrapped("1.26.60", "minecraft:ageable", r#"{"duration": true}"#),
        "minecraft:ageable",
    );
    assert!(kept(&found).is_empty(), "{found:#?}");
    assert_eq!(ageable, Some(json(r#"{"duration": true}"#)));
}

/// An entity with event `demo:e` whose node is `node` (JSON text), at 1.26.60.
fn with_event(node: &str) -> String {
    doc(
        "1.26.60",
        &format!(
            r#"{{"description": {{"identifier": "test:mob"}}, "components": {{}}, "events": {{"demo:e": {node}}}}}"#
        ),
    )
}

/// The members of an event node left once the loader took `weight`, `filters`,
/// `trigger`, `add` and `remove` out are the response collection's, validated strictly at every
/// version (the event loader is never lenient): an unknown response or an unknown member of a response is the game's line at
/// error level; the event loses its responses and the entity loads.
#[test]
fn an_unknown_member_of_an_event_s_responses_is_a_kept_error() {
    for (node, at, name) in [
        (
            r#"{"queue_command": {"command": "say hi", "bogus": 1}}"#,
            "events/demo:e/queue_command/bogus",
            "bogus",
        ),
        (
            r#"{"bogus_response": {}}"#,
            "events/demo:e/bogus_response",
            "bogus_response",
        ),
    ] {
        let loaded = outcome(&with_event(node), &cx());
        assert!(
            matches!(
                loaded.value,
                Some(bedrock_addon_core::versioning::Loaded::Stable(_))
            ),
            "{node}: {:#?}",
            loaded.diagnostics
        );
        assert_eq!(
            kept(&loaded.diagnostics),
            [(
                "UnrecognizedField".to_owned(),
                format!("minecraft:entity/{at}"),
                format!("child '{name}' not valid here.")
            )],
            "{node}: {:#?}",
            loaded.diagnostics
        );
        assert!(
            loud(&loaded.diagnostics).is_empty(),
            "{node}: {:#?}",
            loaded.diagnostics
        );
    }
}

/// When the response collection fails, the event loader parses no
/// response: the node's responses
/// are dropped, each with an information at its member, the kept error stays where it is, and
/// what the loader read before the collection (`add`, `remove`, `filters`, `trigger`, `weight`)
/// stays.
#[test]
fn a_failed_response_collection_drops_the_node_s_responses() {
    let text = with_event(
        r#"{"add": {"component_groups": []}, "queue_command": {"command": "say hi"}, "bogus_response": {}}"#,
    );
    let (entity, found) = stable(&text, &cx());
    assert_eq!(kept(&found).len(), 1, "{found:#?}");
    for member in ["queue_command", "bogus_response"] {
        assert!(
            found.iter().any(|f| f.code.name() == "DroppedField"
                && f.severity == Severity::Info
                && f.location.json_path.to_string()
                    == format!("minecraft:entity/events/demo:e/{member}")),
            "{member}: {found:#?}"
        );
    }
    let written = json(&entity.to_json().expect("the entity writes"));
    let node = written
        .as_object()
        .and_then(|o| o.get("minecraft:entity"))
        .and_then(Value::as_object)
        .and_then(|e| e.get("events"))
        .and_then(Value::as_object)
        .and_then(|e| e.get("demo:e"))
        .cloned();
    assert_eq!(node, Some(json(r#"{"add": {"component_groups": []}}"#)));
    // A collection that passes keeps its responses.
    let (entity, _) = stable(
        &with_event(r#"{"queue_command": {"command": "say hi"}}"#),
        &cx(),
    );
    let written = json(&entity.to_json().expect("the entity writes"));
    assert!(
        format!("{written:?}").contains("queue_command"),
        "{written:?}"
    );
}

/// A node with `randomize`, `sequence` or `first_valid` returns before the validation: its other members are passed over without a word;
/// the crate's information.
#[test]
fn a_member_beside_a_sequence_is_passed_over_without_a_word() {
    let loaded = outcome(
        &with_event(r#"{"sequence": [{"queue_command": {"command": "say"}}], "bogus": 1}"#),
        &cx(),
    );
    assert!(
        kept(&loaded.diagnostics).is_empty(),
        "{:#?}",
        loaded.diagnostics
    );
    assert!(
        loaded
            .diagnostics
            .iter()
            .any(|f| f.code.name() == "UnrecognizedField"
                && f.severity == Severity::Info
                && f.location.json_path.to_string().ends_with("demo:e/bogus")),
        "{:#?}",
        loaded.diagnostics
    );
}

/// The legacy goal readers' numeric checks:
/// a value out of its bound is logged at the content log's error level with the game's text,
/// the member takes its default and the goal
/// is kept. One row per text shape: int `less_equal` / `less`, float `less_equal` /
/// `less` (`%.4f`), the unsigned range, `buildBase`, and the post-parse dependency check.
#[test]
fn a_legacy_goal_value_out_of_its_bound_is_the_game_s_kept_error_and_takes_the_default() {
    let c = "minecraft:entity/components";
    let rows: [(&str, &str, &str, &str, &str); 7] = [
        (
            "minecraft:behavior.work",
            r#"{"priority": 1, "sound_delay_min": -5}"#,
            "sound_delay_min",
            "\"sound_delay_min\" on minecraft:behavior.work cannot be less than 0, value was -5, defaulting to DefinitionType::DEFAULT_SOUND_DELAY_MIN",
            "0",
        ),
        (
            "minecraft:behavior.dragonflaming",
            r#"{"priority": 1, "ground_flame_count": 0}"#,
            "ground_flame_count",
            "\"ground_flame_count\" on minecraft:behavior.dragonflaming must be greater than 0, value was 0, defaulting to Definition::DEFAULT_GROUND_FLAME_COUNT",
            "4",
        ),
        (
            "minecraft:behavior.charge_attack",
            r#"{"priority": 1, "speed_multiplier": -1.5}"#,
            "speed_multiplier",
            "\"speed_multiplier\" on minecraft:behavior.charge_attack cannot be less than 0, value was -1.5000, defaulting to Definition::DEFAULT_SPEED_MULTIPLIER",
            "1",
        ),
        (
            "minecraft:behavior.charge_attack",
            r#"{"priority": 1, "min_distance": 0}"#,
            "min_distance",
            "\"min_distance\" on minecraft:behavior.charge_attack must be greater than 0, value was 0.0000, defaulting to Definition::DEFAULT_MIN_DISTANCE",
            "2",
        ),
        (
            "minecraft:behavior.controlled_by_player",
            r#"{"priority": 1, "fractional_rotation": 2}"#,
            "fractional_rotation",
            "\"fractional_rotation\" on minecraft:behavior.controlled_by_player must be between 0 and Definition::DEFAULT_FRACTIONAL_ROTATION_MAX, value was 2.0000, defaulting to Definition::DEFAULT_FRACTIONAL_ROTATION_MIN",
            "0.5",
        ),
        (
            "minecraft:behavior.nearest_attackable_target",
            r#"{"priority": 1, "scan_interval": 0}"#,
            "scan_interval",
            "\"scan_interval\" on minecraft:behavior.nearest_attackable_target cannot be less than Definition::DEFAULT_SCAN_INTERVAL_BASE, value was 0, defaulting to Definition::DEFAULT_SCAN_INTERVAL",
            "10",
        ),
        (
            "minecraft:behavior.melee_attack",
            r#"{"priority": 1, "min_path_time": 2, "max_path_time": 1}"#,
            "max_path_time",
            "\"max_path_time\" was less than \"\", defaulting both to \"\" value of 2.0000",
            "2",
        ),
    ];
    let mut wrong = Vec::new();
    for (key, value, member, text, default) in rows {
        let (written, found) = read(&wrapped("1.20.0", key, value), key);
        let expected = vec![(
            "InvalidValue".to_owned(),
            format!("{c}/{key}/{member}"),
            text.to_owned(),
        )];
        let told = kept(&found);
        if told != expected {
            wrong.push(format!("{key}: told {told:#?}"));
        }
        if let Some(f) = found
            .iter()
            .find(|f| f.severity >= Severity::Warning && !f.kept)
        {
            wrong.push(format!("{key}: also {f:#?}"));
        }
        let held = written
            .as_ref()
            .and_then(Value::as_object)
            .and_then(|o| o.get(member))
            .map(|v| format!("{v:?}"));
        if held
            .as_deref()
            .is_none_or(|v| v.trim_end_matches(".0") != default)
        {
            wrong.push(format!(
                "{key}: written {written:?}, not {member} {default}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The setters run as the parse meets the members, in the
/// order the file writes them; two failing members of one goal are told in that order, not in
/// the table's (where `sound_delay_min` comes before `active_time`).
#[test]
fn legacy_goal_numeric_lines_follow_the_file_s_member_order() {
    let key = "minecraft:behavior.work";
    let (_, found) = read(
        &wrapped(
            "1.20.0",
            key,
            r#"{"priority": 1, "active_time": -1, "sound_delay_min": -2}"#,
        ),
        key,
    );
    let told: Vec<String> = kept(&found).into_iter().map(|(_, at, _)| at).collect();
    assert_eq!(
        told,
        [
            format!("minecraft:entity/components/{key}/active_time"),
            format!("minecraft:entity/components/{key}/sound_delay_min"),
        ]
    );
}
