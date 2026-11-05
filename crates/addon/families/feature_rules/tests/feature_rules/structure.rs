//! The cereal binds with mask 0xfbf8
//! (unknown members warn; a missing member, a wrong type or an unknown enum name skips the file).

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_family_feature_rules::CoordinateEvalOrder;

use super::common::*;

#[test]
fn r07_an_unknown_member_of_conditions_warns_and_the_rule_loads() {
    let text = rule("1.13.0", "surface_pass", None).replace(
        r#""placement_pass": "surface_pass""#,
        r#""placement_pass": "surface_pass", "foo": 1"#,
    );
    let document = loads(&text);
    let found = with_code(document.diagnostics(), "UnrecognizedField");
    assert_eq!(found.len(), 1, "{:#?}", document.diagnostics());
    assert_eq!(found[0].severity, Severity::Warning);
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:feature_rules/conditions/foo"
    );
    let conditions = bedrock_addon_core::diagnostic::JsonPath::of_keys([
        "minecraft:feature_rules",
        "conditions",
    ]);
    assert!(document.unknown().get(&conditions, "foo").is_some());
}

#[test]
fn rule_6_unknown_members_warn_at_every_level() {
    let text = rule(
        "1.13.0",
        "surface_pass",
        Some(
            r#"{"iterations": 1, "bar": 2, "scatter_chance": {"numerator": 1, "denominator": 2, "baz": 3}, "x": {"distribution": "uniform", "extent": [0, 15], "qux": 4}, "y": 0, "z": 0}"#,
        ),
    )
    .replace(
        r#""places_feature": "demo:feature""#,
        r#""places_feature": "demo:feature", "foo": 1"#,
    );
    let document = loads(&text);
    let mut paths: Vec<String> = with_code(document.diagnostics(), "UnrecognizedField")
        .iter()
        .map(|f| f.location.json_path.to_string())
        .collect();
    paths.sort();
    assert_eq!(
        paths,
        [
            "minecraft:feature_rules/description/foo",
            "minecraft:feature_rules/distribution/bar",
            "minecraft:feature_rules/distribution/scatter_chance/baz",
            "minecraft:feature_rules/distribution/x/qux",
        ]
    );
}

#[test]
fn r08_a_distribution_without_z_is_a_missing_member_and_the_file_is_skipped() {
    let text = rule(
        "1.13.0",
        "surface_pass",
        Some(r#"{"iterations": 1, "x": 0, "y": 0}"#),
    );
    let d = rejects(&text);
    let found = with_code(&d, "MissingRequiredField");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(found[0].severity, Severity::Error);
}

#[test]
fn rule_6_every_required_member_is_required() {
    let full = rule(
        "1.13.0",
        "surface_pass",
        Some(&distribution("1", UNIFORM, "0", "0")),
    );
    for (what, from, to) in [
        (
            "description",
            r#""description": {"identifier": "demo:test", "places_feature": "demo:feature"}, "#,
            "",
        ),
        ("identifier", r#""identifier": "demo:test", "#, ""),
        (
            "places_feature",
            r#", "places_feature": "demo:feature""#,
            "",
        ),
        ("placement_pass", r#""placement_pass": "surface_pass""#, ""),
        ("iterations", r#""iterations": 1, "#, ""),
        (
            "x",
            r#""x": {"distribution": "uniform", "extent": [0, 15]}, "#,
            "",
        ),
        ("y", r#""y": 0, "#, ""),
        (
            "coordinate distribution",
            r#""distribution": "uniform", "#,
            "",
        ),
        ("extent", r#", "extent": [0, 15]"#, ""),
    ] {
        assert!(full.contains(from), "{what}: {full}");
        let d = rejects(&full.replacen(from, to, 1));
        assert!(
            !with_code(&d, "MissingRequiredField").is_empty(),
            "{what}: {d:#?}"
        );
    }
    let without_conditions =
        full.replace(r#", "conditions": {"placement_pass": "surface_pass"}"#, "");
    assert!(!with_code(&rejects(&without_conditions), "MissingRequiredField").is_empty());
    for (what, chance) in [
        ("numerator", r#"{"denominator": 2}"#),
        ("denominator", r#"{"numerator": 1}"#),
    ] {
        let text = rule(
            "1.13.0",
            "surface_pass",
            Some(&format!(
                r#"{{"iterations": 1, "scatter_chance": {chance}, "x": 0, "y": 0, "z": 0}}"#
            )),
        );
        let d = rejects(&text);
        assert!(
            !with_code(&d, "MissingRequiredField").is_empty(),
            "{what}: {d:#?}"
        );
    }
}

#[test]
fn rule_6_a_wrong_json_type_skips_the_file() {
    for distribution in [
        r#"{"iterations": [1], "x": 0, "y": 0, "z": 0}"#,
        r#"{"iterations": 1, "x": [0], "y": 0, "z": 0}"#,
        r#"{"iterations": 1, "x": {"distribution": "uniform", "extent": [0, 15], "step_size": "4"}, "y": 0, "z": 0}"#,
        r#"{"iterations": 1, "scatter_chance": {"numerator": "1", "denominator": 2}, "x": 0, "y": 0, "z": 0}"#,
    ] {
        let d = rejects(&rule("1.13.0", "surface_pass", Some(distribution)));
        assert!(
            !with_code(&d, "InvalidType").is_empty(),
            "{distribution}: {d:#?}"
        );
    }
    let d = rejects(&rule("1.13.0", "surface_pass", None).replace(
        r#""placement_pass": "surface_pass""#,
        r#""placement_pass": 6"#,
    ));
    assert!(!with_code(&d, "InvalidType").is_empty(), "{d:#?}");
}

#[test]
fn rule_6_an_extent_needs_exactly_two_bounds() {
    for extent in ["[0]", "[0, 1, 2]"] {
        let x = format!(r#"{{"distribution": "uniform", "extent": {extent}}}"#);
        let d = rejects(&rule(
            "1.13.0",
            "surface_pass",
            Some(&distribution("1", &x, "0", "0")),
        ));
        assert!(d.has_errors(), "{extent}: {d:#?}");
    }
}

#[test]
fn r09_the_vanilla_test_shape_without_distribution_loads_at_1_21_20() {
    let document = loads(&vanilla_input("without_distribution", "1.21.20"));
    assert!(document.distribution.is_none());
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
}

#[test]
fn r12_coordinate_eval_order_defaults_to_xzy_and_an_unknown_name_skips_the_file() {
    let document = loads(&rule(
        "1.13.0",
        "surface_pass",
        Some(&distribution("1", "0", "0", "0")),
    ));
    let params = document.distribution.as_ref().unwrap();
    assert_eq!(params.coordinate_eval_order, None);
    assert_eq!(params.eval_order(), CoordinateEvalOrder::Xzy);
    for (name, order) in [
        ("xyz", CoordinateEvalOrder::Xyz),
        ("xzy", CoordinateEvalOrder::Xzy),
        ("yxz", CoordinateEvalOrder::Yxz),
        ("yzx", CoordinateEvalOrder::Yzx),
        ("zxy", CoordinateEvalOrder::Zxy),
        ("zyx", CoordinateEvalOrder::Zyx),
    ] {
        let text = rule(
            "1.13.0",
            "surface_pass",
            Some(&format!(
                r#"{{"iterations": 1, "coordinate_eval_order": "{name}", "x": 0, "y": 0, "z": 0}}"#
            )),
        );
        assert_eq!(
            loads(&text).distribution.as_ref().unwrap().eval_order(),
            order
        );
    }
    let d = rejects(&rule(
        "1.13.0",
        "surface_pass",
        Some(r#"{"iterations": 1, "coordinate_eval_order": "abc", "x": 0, "y": 0, "z": 0}"#),
    ));
    let found = with_code(&d, "InvalidEnumValue");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(found[0].severity, Severity::Error);
}

#[test]
fn r19_the_network_binder_s_pascal_case_names_are_invalid_enum_values() {
    for (x, order) in [
        (r#"{"distribution": "Uniform", "extent": [0, 15]}"#, "xzy"),
        (
            r#"{"distribution": "SingleValued", "extent": [0, 15]}"#,
            "xzy",
        ),
        (UNIFORM, "XYZ"),
    ] {
        let text = rule(
            "1.13.0",
            "surface_pass",
            Some(&format!(
                r#"{{"iterations": 1, "coordinate_eval_order": "{order}", "x": {x}, "y": 0, "z": 0}}"#
            )),
        );
        let d = rejects(&text);
        assert_eq!(
            with_code(&d, "InvalidEnumValue").len(),
            1,
            "{x} {order}: {d:#?}"
        );
    }
}

#[test]
fn r11_an_unknown_distribution_name_skips_the_file() {
    let x = r#"{"distribution": "normal", "extent": [0, 15]}"#;
    let d = rejects(&rule(
        "1.13.0",
        "surface_pass",
        Some(&distribution("1", x, "0", "0")),
    ));
    let found = with_code(&d, "InvalidEnumValue");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:feature_rules/distribution/x/distribution"
    );
}

#[test]
fn a_member_named_like_a_kept_field_is_an_unknown_member() {
    // facet rc.7 matches a skipped field by its name.
    let text = rule(
        "1.13.0",
        "surface_pass",
        Some(&distribution("1", "0", "0", "0")),
    )
    .replace(
        r#""identifier": "demo:test""#,
        r#""identifier": "demo:test", "extra": 1"#,
    )
    .replace(r#""iterations": 1"#, r#""iterations": 1, "extra": 2"#);
    let document = loads(&text);
    assert_eq!(
        with_code(document.diagnostics(), "UnrecognizedField").len(),
        2
    );
    let description = bedrock_addon_core::diagnostic::JsonPath::of_keys([
        "minecraft:feature_rules",
        "description",
    ]);
    assert!(document.unknown().get(&description, "extra").is_some());
}

#[test]
fn rule_6_a_non_string_enum_member_is_an_invalid_enum_value() {
    // facet reads a non-string where a unit enum's name goes as an unknown name: the
    // file is skipped either way.
    for (x, order) in [
        (r#"{"distribution": 5, "extent": [0, 15]}"#, r#""xzy""#),
        (r#"{"distribution": true, "extent": [0, 15]}"#, r#""xzy""#),
        (UNIFORM, "1"),
    ] {
        let text = rule(
            "1.13.0",
            "surface_pass",
            Some(&format!(
                r#"{{"iterations": 1, "coordinate_eval_order": {order}, "x": {x}, "y": 0, "z": 0}}"#
            )),
        );
        let d = rejects(&text);
        let found = with_code(&d, "InvalidEnumValue");
        assert_eq!(found.len(), 1, "{x} {order}: {d:#?}");
        assert_eq!(found[0].severity, Severity::Error);
    }
}
