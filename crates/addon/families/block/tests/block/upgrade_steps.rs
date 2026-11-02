//! Behaviours of single `Upgrade` steps of the block chain, run on a hand-built input through
//! `versioning::testing::step_output`.

use bedrock_addon_core::diagnostic::DiagnosticCode;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::testing::{Checks, Run, step_output};
use bedrock_addon_family_block::history::v1_18_0;
use facet_value::VNumber;

/// A JSON value from text.
fn json(text: &str) -> Value {
    facet_json::from_str(text).expect("test JSON")
}

/// The value at a `/`-separated path.
fn at<'v>(value: &'v Value, path: &str) -> Option<&'v Value> {
    let mut value = value;
    for step in path.split('/') {
        value = match step.parse::<usize>() {
            Ok(index) if value.is_array() => value.as_array()?.get(index)?,
            _ => value.as_object()?.get(step)?,
        };
    }
    Some(value)
}

/// The number at `path`.
fn number(value: &Value, path: &str) -> Option<f64> {
    at(value, path)?.as_number().map(VNumber::to_f64_lossy)
}

#[test]
fn light_emission_scales_to_fifteen_and_clamps() {
    let out = step_output::<v1_18_0::Block>(
        &json(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:block_light_emission": {"emission": 0.5}},
                "permutations": [{"condition": "1", "components": {"minecraft:block_light_emission": 2.0}}]}"#,
        ),
        Run::default(),
        &Checks::default(),
    )
    .expect("the step runs");
    let p = at(&out.payload, "minecraft:block").unwrap_or(&out.payload);
    assert_eq!(number(p, "components/minecraft:light_emission"), Some(7.0));
    assert_eq!(
        number(p, "permutations/0/components/minecraft:light_emission"),
        Some(15.0),
        "2.0 → 30 → 15"
    );
    assert!(
        out.diagnostics
            .iter()
            .any(|d| matches!(d.code, DiagnosticCode::ClampedValue { .. })),
        "the clamp is reported"
    );
}

#[test]
fn ticking_becomes_queued_ticking_in_ticks() {
    let out = step_output::<v1_18_0::Block>(
        &json(
            r#"{"description": {"identifier": "demo:x"}, "components": {
                "minecraft:ticking": {"on_tick": {"event": "e"}, "range": [2.5, 1.0], "looping": true}},
                "permutations": [{"condition": "1", "components": {"minecraft:ticking": {"range": [1, 2]}}}]}"#,
        ),
        Run::default(),
        &Checks::default(),
    )
    .expect("the step runs");
    let p = at(&out.payload, "minecraft:block").unwrap_or(&out.payload);
    let range: Vec<f64> = at(p, "components/minecraft:queued_ticking/interval_range")
        .and_then(Value::as_array)
        .expect("queued_ticking.interval_range")
        .iter()
        .filter_map(|v| v.as_number().map(VNumber::to_f64_lossy))
        .collect();
    assert_eq!(range, [20.0, 50.0], "[2.5, 1.0] → [20, 50]");
    assert!(
        at(p, "components/minecraft:queued_ticking/looping").is_some(),
        "looping kept"
    );
    assert!(
        at(p, "permutations/0/components/minecraft:queued_ticking").is_none(),
        "a ticking without on_tick is dropped"
    );
}
