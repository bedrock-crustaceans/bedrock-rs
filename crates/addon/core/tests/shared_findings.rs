//! Locations across nesting levels: a read error names the member inside the value
//! where it happened (`ShapeError.path`), and each type's `check` reports every member kept in
//! an `extra`, at any depth, as an `UnrecognizedField` warning at its full path.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests read fixtures and call facet-json directly"
)]
use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::JsonType;
use bedrock_addon_core::shared::{
    BlockDescriptor, BlockSpecifier, EventTrigger, Filter, ItemDescriptor, ShapeError,
    ShapeErrorKind,
};

/// A JSON path from its segments: a number is an index, anything else a key.
fn path(segments: &[&str]) -> JsonPath {
    let mut path = JsonPath::new();
    for segment in segments {
        match segment.parse::<usize>() {
            Ok(index) => path.push_index(index),
            Err(_) => path.push(segment),
        }
    }
    path
}

/// The JSON text as a `Value`.
fn value(json: &str) -> Value {
    facet_json::from_str_jsonc(json).unwrap()
}

/// The read error of `T` for `json`.
fn error<T: TryFrom<Value, Error = ShapeError> + std::fmt::Debug>(json: &str) -> ShapeError {
    T::try_from(value(json)).expect_err("must be rejected")
}

/// The paths of the `UnrecognizedField` findings among `found`, each at `severity`.
fn unrecognized_at(found: &Diagnostics, severity: Severity) -> Vec<JsonPath> {
    found
        .iter()
        .filter(|d| matches!(d.code, DiagnosticCode::UnrecognizedField { text: None }))
        .inspect(|d| assert_eq!(d.severity, severity))
        .map(|d| d.location.json_path.clone())
        .collect()
}

/// The paths of the `UnrecognizedField` warnings among `found` (a type the game reads with
/// cereal: code `0x4`, level 2 under the unstrict mask).
fn unrecognized(found: &Diagnostics) -> Vec<JsonPath> {
    unrecognized_at(found, Severity::Warning)
}

#[test]
fn a_read_error_in_a_nested_filter_names_its_member() {
    let e = error::<Filter>(r#"{"all_of": [{"test": "x"}, {"test": "y", "subject": "nobody"}]}"#);
    assert_eq!(e.path, path(&["all_of", "1", "subject"]));
    assert_eq!(e.kind, ShapeErrorKind::UnknownName);
    let e = error::<Filter>(r#"[{"any_of": {"NOT": [{"test": "a", "value": [1]}]}}]"#);
    assert_eq!(e.path, path(&["0", "any_of", "NOT", "0", "value"]));
    assert!(matches!(
        e.kind,
        ShapeErrorKind::WrongType {
            found: JsonType::Array,
            ..
        }
    ));
    let e = error::<Filter>(r#"{"OR": [{"value": 1}]}"#);
    assert_eq!(e.path, path(&["OR", "0"]));
    assert_eq!(e.kind, ShapeErrorKind::MissingMember { member: "test" });
    let e = error::<Filter>("5");
    assert!(e.path.is_empty());
}

#[test]
fn a_read_error_in_a_triggers_filter_names_its_member() {
    let e = error::<EventTrigger>(r#"{"event": "e", "filters": {"any_of": [5]}}"#);
    assert_eq!(e.path, path(&["filters", "any_of", "0"]));
    let e = error::<EventTrigger>(r#"{"event": "e", "target": 12}"#);
    assert_eq!(e.path, path(&["target"]));
    // An absent `event` is not an error: a trigger may be condition-only.
    let e = error::<EventTrigger>(r#"{"event": 5}"#);
    assert_eq!(e.path, path(&["event"]));
}

#[test]
fn a_read_error_in_a_block_specifier_state_names_the_state() {
    let e = error::<BlockSpecifier>(r#"{"name": "a:b", "states": {"age": 1.5}}"#);
    assert_eq!(e.path, path(&["states", "age"]));
}

#[test]
fn a_filter_reports_unknown_members_at_every_level() {
    let f = Filter::try_from(value(
        r#"{"any_of": [{"test": "is_family", "value": "pig", "note": 1}, {"all_of": [], "x": 2}], "y": true}"#,
    ))
    .unwrap();
    let found = f.check(&path(&["filters"]));
    assert_eq!(
        unrecognized(&found),
        [
            path(&["filters", "any_of", "0", "note"]),
            path(&["filters", "any_of", "1", "x"]),
            path(&["filters", "y"]),
        ]
    );
}

#[test]
fn a_trigger_reports_its_unknown_members_and_its_filters() {
    let t = EventTrigger::try_from(value(
        r#"{"event": "e", "condition": "q.is_baby", "filters": {"test": "is_daytime", "z": 0}}"#,
    ))
    .unwrap();
    assert_eq!(
        unrecognized(&t.check(&path(&["on_hurt"]))),
        [
            path(&["on_hurt", "filters", "z"]),
            path(&["on_hurt", "condition"]),
        ]
    );
}

#[test]
fn descriptors_report_their_unknown_members() {
    // The item descriptor is read from a `Json::Value`, which ignores the rest in silence: an
    // information, because the game says nothing about the unknown members.
    let d = ItemDescriptor::from(value(r#"{"item": "a:b", "count": 2}"#));
    assert_eq!(
        unrecognized_at(&d.check(&path(&["i"])), Severity::Info),
        [path(&["i", "count"])]
    );
    // The keys O4 passed over are reported by its own information, not as unknown.
    let d = ItemDescriptor::from(value(r#"{"item": "a:b", "tags": "x"}"#));
    assert!(unrecognized_at(&d.check(&JsonPath::new()), Severity::Info).is_empty());
    let d = BlockDescriptor::from(value(r#"{"name": "a:b", "weight": 2}"#));
    assert_eq!(
        unrecognized(&d.check(&JsonPath::new())),
        [path(&["weight"])]
    );
    let d = BlockDescriptor::from(value(r#"{"tags": "1", "name": "a:b", "w": 2}"#));
    let found = d.check(&JsonPath::new());
    assert_eq!(found.len(), 2);
    assert_eq!(unrecognized(&found), [path(&["w"])]);
    let s = BlockSpecifier::try_from(value(r#"{"name": "a:b", "x": 1}"#)).unwrap();
    assert_eq!(unrecognized(&s.check(&JsonPath::new())), [path(&["x"])]);
}
