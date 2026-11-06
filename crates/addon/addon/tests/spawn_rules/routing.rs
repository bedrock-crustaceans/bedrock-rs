//! The routing between the two spawn-rules shapes and the deliberate deviations of the loader:
//! the experiment, the version gates and the findings vanilla does not make.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::spawn_rules::SpawnRules;
use bedrock_addon::version::ENGINE_VERSION;

use super::common::{
    condition, context, document, experiment, legacy, load, loads, names, rejected, standard,
};

fn is_standard(text: &str, cx: &bedrock_addon::load::LoadContext) -> bool {
    matches!(loads(text, cx).value(), SpawnRules::Standard(_))
}

#[test]
fn experiment_off_reads_every_version() {
    for version in ["1.8.0", "1.21.60", &ENGINE_VERSION.to_string()] {
        assert!(is_standard(
            &document(
                version,
                r#"{"description": {"identifier": "x:mob", "population_control": "animal"}}"#
            ),
            &context()
        ));
    }
}

#[test]
fn new_shape_without_the_experiment() {
    let text = document(
        "1.21.60",
        r#"{"description": {"identifier": "x:mob"}, "placement": {}, "conditions": [{"weight": 1}]}"#,
    );
    let document = loads(&text, &context());
    let warnings = names(document.diagnostics(), Severity::Info);
    assert!(warnings.contains(&"UnrecognizedField") && warnings.contains(&"MissingRequiredField"));
}

#[test]
fn legacy_file_under_the_experiment() {
    let text = document(
        "1.21.50",
        r#"{"description": {"identifier": "x:mob", "population_control": "animal"}, "conditions": [{"minecraft:permute_type": [{"weight": 1, "entity_type": "x:a", "guaranteed_count": 1}]}]}"#,
    );
    let document = loads(&text, &experiment());
    let condition = &standard(&document).conditions.as_ref().unwrap()[0];
    assert_eq!(condition.to_spawn_rules().guaranteed.len(), 1);
}

#[test]
fn missing_format_version() {
    let text = r#"{"minecraft:spawn_rules": {"description": {"identifier": "x:mob", "population_control": "animal"}}}"#;
    for cx in [context(), experiment()] {
        let document = loads(text, &cx);
        assert_eq!(
            names(document.diagnostics(), Severity::Info),
            ["MissingFormatVersion"]
        );
    }
}

#[test]
fn new_shape_below_1_21_60() {
    let text = document(
        "1.21.0",
        r#"{"description": {"identifier": "x:mob", "population_control": "animal"}, "placement": {}, "conditions": [{"weight": 1}]}"#,
    );
    let document = loads(&text, &experiment());
    assert!(matches!(document.value(), SpawnRules::Standard(_)));
    assert_eq!(
        names(document.diagnostics(), Severity::Info),
        ["UnrecognizedField", "UnrecognizedField"]
    );
}

#[test]
fn a_version_below_1_8_0_is_rejected() {
    let d = rejected(
        &document("1.7.0", r#"{"description": {"identifier": "x:mob"}}"#),
        &context(),
    );
    assert_eq!(names(&d, Severity::Error)[0], "UnsupportedVersion");
}

#[test]
fn a_json_syntax_error_is_malformed_input() {
    let d = rejected(r#"{"format_version": "1.8.0", "#, &context());
    assert_eq!(names(&d, Severity::Error), ["MalformedInput"]);
}

#[test]
fn an_unknown_population_control_is_an_info() {
    let text = document(
        "1.8.0",
        r#"{"description": {"identifier": "x:mob", "population_control": "boss"}}"#,
    );
    assert_eq!(
        names(loads(&text, &context()).diagnostics(), Severity::Info),
        ["InvalidEnumValue"]
    );
}

#[test]
fn a_wrong_typed_weight_is_coerced_with_an_info() {
    let (c, _, d) = condition(r#""minecraft:weight": {"default": "50"}"#);
    assert_eq!(
        (c.spawn_weight(), names(&d, Severity::Info)),
        (0, vec!["InvalidType"])
    );
}

#[test]
fn a_condition_that_is_no_object_is_left_out() {
    let document = loads(&legacy("5"), &context());
    assert!(standard(&document).conditions.as_ref().unwrap().is_empty());
    assert_eq!(
        names(document.diagnostics(), Severity::Info),
        ["InvalidType"]
    );
}

#[test]
fn untyped_herd_or_permutation() {
    let (_, r, d) = condition(r#""minecraft:herd": [3], "minecraft:permute_type": ["x"]"#);
    assert!(r.herds.is_empty() && r.permutations.is_empty());
    assert_eq!(names(&d, Severity::Info), ["InvalidType", "InvalidType"]);
}

#[test]
fn a_permutation_weight_below_1_is_left_out() {
    let (_, r, d) = condition(r#""minecraft:permute_type": {"weight": 0, "entity_type": "x:a"}"#);
    assert!(r.permutations.is_empty());
    // ours, an information
    assert_eq!(names(&d, Severity::Warning), [] as [&str; 0]);
    assert_eq!(names(&d, Severity::Info), ["InvalidValue"]);
}

#[test]
fn unknown_condition_key() {
    let (_, _, d) = condition(r#""weight": 1"#);
    assert_eq!(names(&d, Severity::Info), ["UnrecognizedField"]);
}

#[test]
fn ignored_shapes() {
    for body in [
        r#""minecraft:spawns_above_block_filter": 1"#,
        r#""minecraft:player_in_village_filter": 1"#,
        r#""minecraft:difficulty_filter": 1"#,
        r#""minecraft:herd": 1"#,
        r#""minecraft:permute_type": 1"#,
    ] {
        let (_, _, d) = condition(body);
        assert_eq!(names(&d, Severity::Info), ["InvalidType"], "{body}");
    }
    let (_, _, d) = condition(r#""minecraft:difficulty_filter": {"max": "x"}"#);
    assert_eq!(names(&d, Severity::Info), ["InvalidEnumValue"]);
}

#[test]
fn failed_block_descriptor() {
    let (_, r, d) = condition(r#""minecraft:spawns_on_block_prevented_filter": [{"name": 1}]"#);
    assert!(r.spawns_on_blocks_prevented.is_empty());
    assert_eq!(names(&d, Severity::Error), ["InvalidValue"]);
}

#[test]
fn conditions_not_an_array() {
    let text = document(
        "1.8.0",
        r#"{"description": {"identifier": "x:mob", "population_control": "animal"}, "conditions": {}}"#,
    );
    let document = loads(&text, &context());
    assert!(standard(&document).conditions.is_none());
    assert_eq!(
        names(document.diagnostics(), Severity::Info),
        ["InvalidType"]
    );
}

#[test]
fn experiment_on_from_1_21_60() {
    let text = document("1.21.60", r#"{"description": {"identifier": "x:mob"}}"#);
    assert!(matches!(
        loads(&text, &experiment()).value(),
        SpawnRules::Simplified(_)
    ));
}

#[test]
fn legacy_shape_at_1_21_60() {
    let text = document(
        "1.21.60",
        r#"{"description": {"identifier": "x:mob"}, "conditions": [{"minecraft:weight": {}}]}"#,
    );
    let d = rejected(&text, &experiment());
    assert!(
        d.iter()
            // By legacy name: the variant and the definitions that were it.
            .any(|f| matches!(f.code.name(), "MissingRequiredField"))
    );
}

#[test]
fn engine_version_unsupported() {
    let text = document(
        &ENGINE_VERSION.to_string(),
        r#"{"description": {"identifier": "x:mob"}}"#,
    );
    assert_eq!(
        names(&rejected(&text, &experiment()), Severity::Error)[0],
        "UnsupportedVersion"
    );
}

#[test]
fn a_simplified_file_with_a_third_root_member_is_malformed() {
    let text = r#"{"format_version": "1.21.60", "minecraft:spawn_rules": {"description": {"identifier": "x:mob"}}, "x": 1}"#;
    assert_eq!(
        names(&rejected(text, &experiment()), Severity::Error)[0],
        "MalformedInput"
    );
}

#[test]
fn a_1_21_50_file_under_the_experiment_is_standard() {
    let text = document(
        "1.21.50",
        r#"{"description": {"identifier": "x:mob", "population_control": "animal"}}"#,
    );
    assert!(is_standard(&text, &experiment()));
}

#[test]
fn placement_names_built_in_only() {
    let text = document(
        "1.21.60",
        r#"{"description": {"identifier": "x:mob"}, "placement": {"spawn_predicate": "x:script_predicate"}}"#,
    );
    assert!(
        load(&text, &experiment())
            .is_ok_and(|d| names(d.diagnostics(), Severity::Warning) == ["UnresolvedReference"])
    );
}
