//! The reading axes the sound files' reader needs (a jsoncpp variant that coerces every member
//! and takes a `null` for the default): `Scalars::Fork`, `NullHandling::AbsentSilently`.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::json::kit::{Node, Present, Profile, Read};
use bedrock_addon_core::json::{Value, from_str_jsonc};
use bedrock_addon_core::schema::{self, FromJson, Schema};
use bedrock_addon_core::versioning::{NullHandling, Reading, Scalars};
use facet::Facet;

/// Reads `text` as `T` under `reading`, every failure told: the value and each finding as
/// `severity path code`.
fn read_as<T: FromJson>(text: &str, reading: Reading) -> (Option<T>, Vec<String>) {
    let json: Value = from_str_jsonc(text).expect("JSON");
    let node = Node::new(&json, JsonPath::of_keys(["root"]));
    let mut found = Diagnostics::default();
    let mut read = Read::new(reading, &mut found).profile(Profile::EVERY_FAILURE);
    let value = T::from_json(&node, &mut read);
    read.finish();
    let told = found
        .iter()
        .map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code.id()))
        .collect();
    (value, told)
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Coerced {
    #[facet(default)]
    size: f32,
    #[facet(default)]
    on: bool,
    #[facet(default)]
    count: i32,
    #[facet(default)]
    name: String,
}

/// The sound files' reader coerces every member and refuses none: a float of a string, array or
/// object is `0.0`, a bool of a number or string is whether it is non-zero or non-empty, an
/// integer of a real is its truncation, a text of a bool is its name and of anything else
/// empty; each is told as an information at the member, and a `null`, a bool read as a float
/// or a number as a float are no finding.
#[test]
fn the_fork_profile_coerces_every_member_and_tells_what_is_of_another_type() {
    let read = |text: &str| read_as::<Coerced>(text, Reading::OWN.with_scalars(Scalars::Fork));
    let (value, told) = read(r#"{"size": "1", "on": "false", "count": 2.9, "name": 5}"#);
    assert_eq!(
        value,
        Some(Coerced {
            size: 0.0,
            on: true,
            count: 2,
            name: String::new()
        })
    );
    assert_eq!(
        told,
        [
            "Info root/size core/invalid_value",
            "Info root/on core/invalid_value",
            "Info root/count core/invalid_value",
            "Info root/name core/invalid_value"
        ]
    );
    let (value, told) = read(r#"{"size": true, "on": 0, "count": true, "name": true}"#);
    assert_eq!(
        value,
        Some(Coerced {
            size: 1.0,
            on: false,
            count: 1,
            name: "true".to_owned()
        })
    );
    assert_eq!(
        told,
        [
            "Info root/on core/invalid_value",
            "Info root/count core/invalid_value",
            "Info root/name core/invalid_value"
        ]
    );
    let (value, told) = read(r#"{"size": [1], "on": {}, "count": null, "name": null}"#);
    assert_eq!(
        value,
        Some(Coerced {
            size: 0.0,
            on: false,
            count: 0,
            name: String::new()
        })
    );
    assert_eq!(
        told,
        [
            "Info root/size core/invalid_value",
            "Info root/on core/invalid_value"
        ]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Volume {
    #[facet(default = 1.0)]
    volume: f32,
    name: Option<String>,
}

/// A silent `null` is the member absent, told nothing: the default of a defaulted member, `None`
/// of an `Option`; the `Absent` rule tells it.
#[test]
fn a_silent_null_member_is_absent_and_told_nothing() {
    let text = r#"{"volume": null, "name": null}"#;
    let (value, told) =
        read_as::<Volume>(text, Reading::OWN.with_null(NullHandling::AbsentSilently));
    assert_eq!(
        value,
        Some(Volume {
            volume: 1.0,
            name: None
        })
    );
    assert!(told.is_empty(), "{told:?}");
    let (value, told) = read_as::<Volume>(text, Reading::OWN.with_null(NullHandling::Absent));
    assert_eq!(value.map(|v| v.volume), Some(1.0));
    assert_eq!(told.len(), 2, "{told:?}");
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Written3 {
    #[facet(default)]
    a: i32,
    #[facet(default, rename = "bee")]
    b: i32,
    c: Option<i32>,
    #[facet(skip)]
    present: Present,
}

/// A skipped `Present` field is filled with the names of the struct's fields the object wrote,
/// a `null` one included and an unknown one not, and is left out of equality.
#[test]
fn a_present_field_holds_the_members_the_object_wrote() {
    let reading = Reading::OWN.with_null(NullHandling::AbsentSilently);
    let (value, told) = read_as::<Written3>(r#"{"bee": null, "z": 1, "c": 2}"#, reading);
    let value = value.expect("read");
    assert_eq!(value.present.0, Some(vec!["bee", "c"]));
    assert_eq!(told.len(), 1, "{told:?}");
    assert_eq!(
        value,
        Written3 {
            a: 0,
            b: 0,
            c: Some(2),
            present: Present::default()
        }
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::after_read = follow)]
struct Following {
    #[facet(default)]
    kind: String,
    #[facet(default)]
    size: f32,
    extra: Option<Value>,
}

/// The hook of the struct runs after the members and sees the object as written: a default that
/// follows a sibling, a member kept beside the typed one.
fn follow(value: &mut Following, node: &Node<'_>, read: &mut Read<'_>) {
    if value.kind == "big"
        && !node
            .value()
            .as_object()
            .is_some_and(|o| o.contains_key("size"))
    {
        value.size = 9.0;
    }
    if value.size > 100.0 {
        let finding = Diagnostic::of(&core_codes::INVALID_VALUE).at(node.path().child("size"));
        read.push(finding);
    }
}

/// A struct's `after_read` runs once its members are read and tells its findings after theirs.
#[test]
fn an_after_read_hook_runs_over_the_value_read() {
    let reading = Reading::OWN.with_null(NullHandling::AbsentSilently);
    let (value, told) = read_as::<Following>(r#"{"kind": "big"}"#, reading);
    assert_eq!(value.map(|v| v.size), Some(9.0));
    assert!(told.is_empty(), "{told:?}");
    let (value, told) = read_as::<Following>(r#"{"size": 500, "kind": 1}"#, reading);
    assert_eq!(value.map(|v| v.size), Some(500.0));
    assert_eq!(
        told,
        [
            "Error root/kind core/invalid_type",
            "Error root/size core/invalid_value"
        ]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Raw {
    #[facet(schema::reads_null)]
    raw: Option<Value>,
    other: Option<Value>,
}

/// A field that reads a `null` as a value keeps it under a reading that makes a `null` absent,
/// silently or told: the member is there for it, and the other members are absent.
#[test]
fn a_reads_null_field_takes_a_null_as_a_value_whatever_makes_it_absent() {
    let text = r#"{"raw": null, "other": null}"#;
    let expected = Some(Raw {
        raw: Some(Value::NULL),
        other: None,
    });
    let (value, told) = read_as::<Raw>(text, Reading::OWN.with_null(NullHandling::AbsentSilently));
    assert_eq!(value, expected);
    assert!(told.is_empty(), "{told:?}");
    let (value, told) = read_as::<Raw>(text, Reading::OWN.with_null(NullHandling::Absent));
    assert_eq!(value, expected);
    assert_eq!(
        told,
        ["Info root/other core/value_rewritten"],
        "only the other is rewritten"
    );
}

/// A read told in a phase tells every finding of its own in it, and a fork of it does as well.
#[test]
fn a_read_with_a_phase_tells_its_findings_in_it() {
    let json: Value = from_str_jsonc(r#"{"size": "x"}"#).expect("JSON");
    let node = Node::new(&json, JsonPath::new());
    let mut found = Diagnostics::default();
    let mut read =
        Read::new(Reading::OWN.with_scalars(Scalars::Fork), &mut found).phase(Phase::Validate);
    let _ = Coerced::from_json(&node, &mut read);
    read.push(Diagnostic::of(&core_codes::INVALID_VALUE));
    let mut attempt = read.fork();
    attempt.push(Diagnostic::of(&core_codes::INVALID_VALUE));
    read.commit(attempt);
    read.finish();
    let phases: Vec<Phase> = found.iter().map(|finding| finding.phase).collect();
    assert_eq!(phases, [Phase::Validate; 3]);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::quiet = "volume pitch", schema::after_read = list_later)]
struct Table {
    #[facet(default)]
    name: String,
    #[facet(default, schema::read_with = schema::read_later)]
    sounds: Vec<String>,
    extra: bedrock_addon_core::document::UnknownMembers,
}

/// The rule of the table reads the member the derived read left to it.
fn list_later(table: &mut Table, node: &Node<'_>, _: &mut Read<'_>) {
    let sounds = node.value().as_object().and_then(|o| o.get("sounds"));
    if let Some(array) = sounds.and_then(Value::as_array) {
        table.sounds = vec![format!("{} members", array.len())];
    }
}

/// A quiet member is kept and told nothing, where another unknown one is told and kept; a member
/// `read_later` names is no unknown one and is left to the struct's rule.
#[test]
fn a_quiet_member_is_kept_untold_and_a_member_read_later_is_known() {
    let reading = Reading::OWN.with_null(NullHandling::AbsentSilently);
    let (value, told) = read_as::<Table>(
        r#"{"name": "n", "volume": 1, "odd": 2, "sounds": [1, 2]}"#,
        reading,
    );
    let value = value.expect("read");
    assert_eq!(value.sounds, ["2 members"]);
    let kept: Vec<&str> = value.extra.iter().map(|(key, _)| key).collect();
    assert_eq!(kept, ["volume", "odd"]);
    assert_eq!(told, ["Warning root/odd core/unrecognized_field"]);
}

/// The one read of the three sound documents: members coerced, a `null` the default, unknown
/// members after the rest told as an information, all of it in the validate phase.
#[test]
fn a_merged_sound_document_reads_as_the_fork_does() {
    use bedrock_addon_core::diagnostic::Severity;
    use bedrock_addon_core::load::sound::read_merged;
    let json: Value =
        from_str_jsonc(r#"{"zz": 1, "size": "2", "on": null, "name": true}"#).expect("JSON");
    let mut found = Diagnostics::default();
    let value: Coerced = read_merged(&json, &mut found);
    assert_eq!(
        value,
        Coerced {
            size: 0.0,
            on: false,
            count: 0,
            name: "true".to_owned()
        }
    );
    let told: Vec<(Severity, Phase, String)> = found
        .iter()
        .map(|f| (f.severity, f.phase, f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        told,
        [
            (Severity::Info, Phase::Validate, "size".to_owned()),
            (Severity::Info, Phase::Validate, "name".to_owned()),
            (Severity::Info, Phase::Validate, "zz".to_owned()),
        ]
    );
}
