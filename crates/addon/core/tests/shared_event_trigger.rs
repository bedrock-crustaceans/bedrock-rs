//! `EventTrigger`: `"ns:event"` or `{event, target?, filters?}`; a legacy
//! `condition` and other members are kept in `extra`.

#[path = "shared/common.rs"]
mod common;

use bedrock_addon_core::diagnostic::{JsonPath, Severity, codes};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::shared::{EventTrigger, Filter, FilterSubject};
use common::{compact, read, round_trip};

#[test]
fn a_string_is_the_event_on_self() {
    let t: EventTrigger = round_trip(r#""minecraft:become_angry""#);
    assert_eq!(t.event.as_deref(), Some("minecraft:become_angry"));
    assert!(t.bare);
    assert_eq!(t.target, None);
    assert_eq!(t.target.unwrap_or_default(), FilterSubject::Self_);
    assert_eq!(t.filters, None);
}

#[test]
fn the_object_form_reads_event_target_and_filters() {
    let t: EventTrigger = round_trip(
        r#"{"event": "minecraft:on_hurt", "target": "other", "filters": {"test": "is_family", "subject": "other", "value": "player"}}"#,
    );
    assert!(!t.bare);
    assert_eq!(t.event.as_deref(), Some("minecraft:on_hurt"));
    assert_eq!(t.target, Some(FilterSubject::Other));
    assert!(matches!(t.filters, Some(Filter::Test(_))));
}

#[test]
fn an_absent_or_explicit_self_target_is_written_back_as_read() {
    let t: EventTrigger = round_trip(r#"{"event": "e"}"#);
    assert_eq!(t.target, None);
    let t: EventTrigger = round_trip(r#"{"event": "e", "target": "self"}"#);
    assert_eq!(t.target, Some(FilterSubject::Self_));
}

#[test]
fn d6_a_legacy_condition_is_kept_in_extra() {
    let t: EventTrigger =
        round_trip(r#"{"event": "e", "target": "self", "condition": "query.is_baby"}"#);
    assert_eq!(
        t.extra.get("condition"),
        Some(&Value::from("query.is_baby"))
    );
    let t: EventTrigger = round_trip(
        r#"{"event": "e", "condition": {"expression": "q.x", "version": 12}, "other": 1}"#,
    );
    assert!(t.extra.get("condition").is_some() && t.extra.get("other").is_some());
}

#[test]
fn the_check_covers_the_filters() {
    let t: EventTrigger =
        read(r#"{"event": "e", "filters": {"all_of": [{"test": "is_nothing"}]}}"#).unwrap();
    let found = t.check(&JsonPath::new(), Severity::Warning);
    assert_eq!(found.len(), 1);
    let finding = found.iter().next().unwrap();
    assert!(finding.is(&codes::FILTER_TEST_NOT_FOUND));
    let mut expected = JsonPath::new();
    expected.push("filters");
    expected.push("all_of");
    expected.push_index(0);
    assert_eq!(finding.location.json_path, expected);
}

#[test]
fn malformed_triggers_are_read_errors() {
    // `{}` is a condition-only trigger, not an error.
    for json in [
        "5",
        "[]",
        r#"{"event": 1}"#,
        r#"{"event": "e", "target": "nobody"}"#,
        r#"{"event": "e", "filters": 5}"#,
    ] {
        assert!(
            read::<EventTrigger>(json).is_err(),
            "{json} must be rejected"
        );
    }
}

#[test]
fn a_constructed_trigger_is_a_string() {
    assert_eq!(compact(&EventTrigger::new("x:y")), r#""x:y""#);
    let mut t = EventTrigger::new("x:y");
    t.target = Some(FilterSubject::Target);
    assert_eq!(compact(&t), r#"{"event":"x:y","target":"target"}"#);
}

// A trigger may omit `event` and carry only conditions.
#[test]
fn d6_an_object_without_event_is_a_condition_only() {
    // Vanilla `horse.json`, `wolf.json`, `nautilus.json`, `sulfur_cube.json`: an
    // interaction's `on_interact` with `filters` and `target` only.
    let t: EventTrigger = round_trip(
        r#"{"target": "self", "filters": {"test": "is_sneak_held", "subject": "other", "value": false}}"#,
    );
    assert_eq!(t.event, None);
    assert!(!t.bare);
    assert!(t.filters.is_some());
    let t: EventTrigger = round_trip("{}");
    assert_eq!(t.event, None);
    assert_eq!(compact(&t), "{}");
}
