//! `testing::step_output` against the toy family: one step is run on a payload of its struct,
//! under the run settings and the context it is given, and gives the next struct's payload with
//! what the step reported.

use bedrock_addon_core::diagnostic::DiagnosticCode;
use bedrock_addon_core::json::{Value, from_str};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::testing::{Checks, FailureKind, Run, step_output};

use crate::toy::{v1_0_0, v1_1_0};

fn json(text: &str) -> Value {
    from_str(text).expect("JSON")
}

/// A payload of the 1.1.0 struct.
fn second_base() -> Value {
    json(r#"{"description": {"identifier": "demo:x"}, "color": "blue", "speed": 2.0}"#)
}

fn checks() -> Checks<'static> {
    Checks {
        identifier: "demo:x",
        ..Checks::default()
    }
}

fn member<'v>(value: &'v Value, name: &str) -> Option<&'v Value> {
    value.as_object().and_then(|object| object.get(name))
}

#[test]
fn one_step_is_run_on_a_payload_and_gives_the_next_struct_s() {
    let output = step_output::<v1_1_0::Toy>(&second_base(), Run::default(), &checks()).unwrap();
    let color = member(&output.payload, "color");
    assert_eq!(
        color
            .and_then(Value::as_string)
            .map(facet_value::VString::as_str),
        Some("BLUE")
    );
    assert!(member(&output.payload, "movement").is_some());
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn a_payload_the_struct_does_not_take_is_refused() {
    let refused = step_output::<v1_1_0::Toy>(&Value::from(7), Run::default(), &checks());
    assert!(
        matches!(refused, Err(FailureKind::InputRejected { .. })),
        "{refused:?}"
    );
    // `old_speed` is a number in the 1.0.0 struct.
    let input = json(r#"{"description": {"identifier": "demo:x"}, "old_speed": "fast"}"#);
    let refused = step_output::<v1_0_0::Toy>(&input, Run::default(), &checks());
    assert!(
        matches!(refused, Err(FailureKind::InputRejected { .. })),
        "{refused:?}"
    );
}

#[test]
fn what_the_step_reports_comes_back_with_the_payload() {
    let input = json(r#"{"description": {"identifier": "demo:x"}, "legacy_flag": true}"#);
    let output = step_output::<v1_0_0::Toy>(&input, Run::default(), &checks()).unwrap();
    let codes: Vec<&DiagnosticCode> = output.diagnostics.iter().map(|d| &d.code).collect();
    assert!(
        matches!(codes[..], [DiagnosticCode::DroppedField { .. }]),
        "{codes:?}"
    );
    assert!(member(&output.payload, "legacy_flag").is_none());
}

#[test]
fn skip_section_a_leaves_out_what_section_a_adds() {
    let with = step_output::<v1_1_0::Toy>(&second_base(), Run::default(), &checks()).unwrap();
    assert_eq!(member(&with.payload, "marker"), Some(&Value::from(true)));
    let skipped = Run {
        skip_section_a: true,
        ..Run::default()
    };
    let without = step_output::<v1_1_0::Toy>(&second_base(), skipped, &checks()).unwrap();
    assert!(member(&without.payload, "marker").is_none());
}

#[test]
fn the_original_version_reaches_the_step() {
    // The 1.1.0 step marks a document written before 1.0.5; the default is the step's own.
    let own = step_output::<v1_1_0::Toy>(&second_base(), Run::default(), &checks()).unwrap();
    assert!(member(&own.payload, "legacy").is_none());
    let old = Run {
        original_version: Some(FormatVersion::new(1, 0, 4)),
        ..Run::default()
    };
    let output = step_output::<v1_1_0::Toy>(&second_base(), old, &checks()).unwrap();
    assert_eq!(member(&output.payload, "legacy"), Some(&Value::from(true)));
}
