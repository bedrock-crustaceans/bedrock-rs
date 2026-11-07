//! `scatter_feature`: the 1.21.10 move into `distribution`, the strict schema of both eras and
//! the runtime scatter checks (fixtures f01–f03).

use bedrock_addon::diagnostic::{Kind, Severity};
use bedrock_addon::document::Content;
use bedrock_addon::families::feature::{Feature, ScatterFeature};
use bedrock_addon::families::feature_rules::{
    CoordinateEvalOrder, CoordinateRange, FeatureRule, RandomDistribution, ScatterChance,
    ScatterFraction,
};
use bedrock_addon::load::LoadContext;
use bedrock_addon::molang::Molang;
use bedrock_addon::storage::PackPath;

use super::common::{above_info, cx, doc, error_at, fixture, kept_at, loads, rejects};

/// The scatter feature of a loaded document.
#[track_caller]
fn scatter(json: &str) -> ScatterFeature {
    match loads(json, &cx()).into_value() {
        Feature::Scatter(scatter) => scatter,
        other => panic!("not a scatter feature: {other:?}"),
    }
}

/// The fraction form of `scatter_chance`.
fn fraction(numerator: i32, denominator: i32) -> ScatterChance {
    ScatterChance::Fraction(ScatterFraction {
        numerator,
        denominator,
    })
}

/// `distribution` of a scatter feature and of a feature rule is one
/// type, so the same text gives equal values and a value moves from one to the other.
#[test]
fn a_scatter_feature_and_a_feature_rule_hold_the_same_scatter_params() {
    let distribution = r#"{"iterations": 3, "scatter_chance": {"numerator": 1, "denominator": 4}, "coordinate_eval_order": "zyx", "x": 2, "y": {"distribution": "fixed_grid", "extent": [0, 15], "step_size": 2, "grid_offset": 1}, "z": {"distribution": "triangle", "extent": [1, 8]}}"#;
    let feature = scatter(&doc(
        "1.21.10",
        "scatter_feature",
        &format!(r#""places_feature": "demo:y", "distribution": {distribution}"#),
    ));
    let rule = format!(
        r#"{{"format_version": "1.21.10", "minecraft:feature_rules": {{"description": {{"identifier": "demo:test", "places_feature": "demo:feature"}}, "conditions": {{"placement_pass": "surface_pass"}}, "distribution": {distribution}}}}}"#
    );
    let mut rule = FeatureRule::parse(
        &PackPath::from_static("feature_rules/test.json"),
        rule.as_bytes(),
        &LoadContext::detached(),
    )
    .unwrap_or_else(|rejected| panic!("the rule loads: {:#?}", rejected.diagnostics))
    .into_value();
    assert_eq!(rule.distribution.as_ref(), Some(&feature.distribution));
    assert_eq!(
        feature.distribution.z.as_ref().map(|z| match z {
            CoordinateRange::Object(z) => z.distribution.name(),
            CoordinateRange::Single(_) => "an expression",
        }),
        Some("triangle")
    );
    // A feature's parameters are a rule's. Those of a 1.13.0 feature may lack a coordinate
    // (f02e): the rule is written without it, nothing is invented in its place.
    let old = scatter(&doc(
        "1.13.0",
        "scatter_feature",
        r#""places_feature": "demo:y", "iterations": 1, "y": 0, "z": 4"#,
    ));
    assert_eq!(old.distribution.x, None);
    rule.distribution = Some(old.distribution);
    let written = rule.to_json().unwrap();
    let value: bedrock_addon::json::Value = facet_json::from_str(&written).unwrap();
    let keys: Vec<&str> = value.as_object().unwrap()["minecraft:feature_rules"]
        .as_object()
        .unwrap()["distribution"]
        .as_object()
        .unwrap()
        .keys()
        .map(facet_value::VString::as_str)
        .collect();
    assert_eq!(keys, ["iterations", "y", "z"], "{written}");
}

/// The text of a Molang expression value.
fn expression(value: &Molang) -> &str {
    match value {
        Molang::Expr(source) => source.as_str(),
        other => panic!("not an expression: {other:?}"),
    }
}

#[test]
fn f01_scatter_1_13_0_moves_the_root_parameters_into_distribution() {
    let document = loads(&fixture("f01_scatter_1_13_0_to_distribution.json"), &cx());
    assert!(document.version().upgraded);
    let Feature::Scatter(feature) = document.value() else {
        panic!("not a scatter feature");
    };
    assert_eq!(feature.places_feature.as_str(), "demo:flower");
    assert_eq!(feature.project_input_to_floor, Some(true));
    let distribution = &feature.distribution;
    assert_eq!(expression(&distribution.iterations), "math.random(1, 3)");
    assert_eq!(distribution.scatter_chance, Some(fraction(1, 4)));
    assert_eq!(
        distribution.coordinate_eval_order,
        Some(CoordinateEvalOrder::Zyx)
    );
    let Some(CoordinateRange::Single(x)) = &distribution.x else {
        panic!("x is not an expression");
    };
    assert_eq!(expression(x), "v.originx + 2");
    let Some(CoordinateRange::Object(y)) = &distribution.y else {
        panic!("y is not a range");
    };
    assert_eq!(y.distribution, RandomDistribution::Uniform);
    assert_eq!(y.extent, [Molang::Const(0.0), Molang::Const(15.0)]);
    let Some(CoordinateRange::Object(z)) = &distribution.z else {
        panic!("z is not a range");
    };
    assert_eq!(z.distribution, RandomDistribution::Gaussian);
}

#[test]
fn f01_scatter_chance_in_the_expression_form_moves_too() {
    let json = doc(
        "1.13.0",
        "scatter_feature",
        r#""places_feature": "demo:y", "iterations": 2, "scatter_chance": "50 * 0.5", "x": 0, "y": 0, "z": 0"#,
    );
    let feature = scatter(&json);
    let Some(ScatterChance::Percent(chance)) = &feature.distribution.scatter_chance else {
        panic!("not a percentage");
    };
    assert_eq!(expression(chance), "50 * 0.5");
    assert_eq!(feature.distribution.iterations, Molang::Const(2.0));
}

#[test]
fn f02_a_1_21_10_document_with_root_iterations_is_rejected() {
    let json = doc(
        "1.21.10",
        "scatter_feature",
        r#""places_feature": "demo:y", "iterations": 2, "distribution": {"iterations": 1, "x": 0, "y": 0, "z": 0}"#,
    );
    let d = rejects(&json, &cx());
    error_at(
        &d,
        "core/child_not_valid",
        "minecraft:scatter_feature/iterations",
    );
}

#[test]
fn f02_a_1_21_10_document_without_distribution_is_rejected() {
    let json = doc(
        "1.21.10",
        "scatter_feature",
        r#""places_feature": "demo:y""#,
    );
    let d = rejects(&json, &cx());
    error_at(
        &d,
        "core/required_child",
        "minecraft:scatter_feature/distribution",
    );
}

#[test]
fn f02b_a_1_13_0_document_with_distribution_is_rejected() {
    let json = doc(
        "1.13.0",
        "scatter_feature",
        r#""places_feature": "demo:y", "iterations": 1, "distribution": {"iterations": 1, "x": 0, "y": 0, "z": 0}"#,
    );
    let d = rejects(&json, &cx());
    error_at(
        &d,
        "core/child_not_valid",
        "minecraft:scatter_feature/distribution",
    );
    assert!(super::common::has_text(
        &d,
        "child 'distribution' not valid here."
    ));
}

#[test]
fn f02c_a_distribution_without_y_is_rejected() {
    let json = doc(
        "1.21.10",
        "scatter_feature",
        r#""places_feature": "demo:y", "distribution": {"iterations": 1, "x": 0, "z": 0}"#,
    );
    let d = rejects(&json, &cx());
    error_at(&d, "core/required_child", "distribution/y");
    assert!(super::common::has_text(&d, "Required child y not found"));
}

#[test]
fn f02c_an_unknown_member_of_distribution_is_rejected() {
    let json = doc(
        "1.21.10",
        "scatter_feature",
        r#""places_feature": "demo:y", "distribution": {"iterations": 1, "x": 0, "y": 0, "z": 0, "foo": 1}"#,
    );
    let d = rejects(&json, &cx());
    error_at(&d, "core/child_not_valid", "distribution/foo");
}

#[test]
fn f02d_runtime_values_are_replaced_and_warned_and_the_feature_loads() {
    let json = doc(
        "1.21.10",
        "scatter_feature",
        r#""places_feature": "demo:y", "distribution": {
            "iterations": -3,
            "scatter_chance": 150,
            "x": {"distribution": "uniform", "extent": [0, 15], "step_size": 2},
            "y": {"distribution": "uniform", "extent": [4, 2]},
            "z": 0
        }"#,
    );
    let document = loads(&json, &cx());
    let warnings: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|f| {
            f.severity == Severity::Error
                && matches!(
                    f.code.id(),
                    "core/scatter_param_value"
                        | "core/scatter_param_value_worded"
                        | "feature/iterations_negative"
                        | "feature/scatter_chance_percent"
                        | "feature/scatter_chance_fraction"
                        | "feature/must_be_provided"
                        | "feature/step_size_not_grid"
                        | "feature/grid_offset_not_grid"
                        | "feature/extent_order"
                )
        })
        .collect();
    assert_eq!(warnings.len(), 4, "{:#?}", document.diagnostics());
    let d = document.diagnostics();
    kept_at(d, "feature/iterations_negative", "distribution/iterations");
    kept_at(
        d,
        "feature/scatter_chance_percent",
        "distribution/scatter_chance",
    );
    kept_at(d, "feature/step_size_not_grid", "distribution/x/step_size");
    kept_at(d, "feature/extent_order", "distribution/y/extent");
    for text in [
        "Bad value for iterations - should be >= 0",
        "Bad value for scatter_chance - should be between 0 and 100 (exclusive)",
        "step_size is only valid when distribution is fixed_grid or jittered_grid",
        "Bad value for extent - maximum must be greater than minimum",
    ] {
        assert!(super::common::has_text(d, text), "{text}: {d:#?}");
    }
    let Feature::Scatter(feature) = document.value() else {
        panic!("not a scatter feature");
    };
    assert_eq!(feature.distribution.iterations, Molang::Const(1.0));
    assert_eq!(
        feature.distribution.scatter_chance,
        Some(ScatterChance::Percent(Molang::Const(100.0)))
    );
    // The fifth value of the row: a ratio whose denominator is not above the numerator.
    let json = doc(
        "1.21.10",
        "scatter_feature",
        r#""places_feature": "demo:y", "distribution": {
            "iterations": 1, "scatter_chance": {"numerator": 5, "denominator": 5},
            "x": 0, "y": 0, "z": 0
        }"#,
    );
    let document = loads(&json, &cx());
    kept_at(
        document.diagnostics(),
        "feature/scatter_chance_fraction",
        "distribution/scatter_chance",
    );
    assert!(super::common::has_text(
        document.diagnostics(),
        "Bad value for scatter_chance - denominator should be greater than the numerator"
    ));
    let Feature::Scatter(feature) = document.value() else {
        panic!("not a scatter feature");
    };
    assert_eq!(feature.distribution.scatter_chance, Some(fraction(5, 1)));
}

/// The range rules run on the upgraded document; in a file below 1.21.10 the
/// parameters are at the root of the payload, and a rule names the member there. The value
/// is the rule's replacement, which the written (1.21.10) document holds.
#[test]
fn a_rule_of_a_1_13_0_scatter_feature_names_the_member_at_the_root() {
    let json = doc(
        "1.13.0",
        "scatter_feature",
        r#""places_feature": "demo:y", "iterations": -1, "scatter_chance": 400,
            "x": {"distribution": "uniform", "extent": [4, 2]}, "y": 0, "z": 0"#,
    );
    let document = loads(&json, &cx());
    let mut found: Vec<(&str, String)> = above_info(document.diagnostics())
        .iter()
        .map(|f| (f.code.id(), f.location.json_path.to_string()))
        .collect();
    found.sort();
    assert_eq!(
        found,
        [
            (
                "feature/extent_order",
                "minecraft:scatter_feature/x/extent".to_owned()
            ),
            (
                "feature/iterations_negative",
                "minecraft:scatter_feature/iterations".to_owned()
            ),
            (
                "feature/scatter_chance_percent",
                "minecraft:scatter_feature/scatter_chance".to_owned()
            ),
        ]
    );
    let Feature::Scatter(feature) = document.value() else {
        panic!("not a scatter feature");
    };
    assert_eq!(feature.distribution.iterations, Molang::Const(1.0));
}

#[test]
fn f02e_a_1_13_0_document_without_x_loads_and_is_warned() {
    let json = doc(
        "1.13.0",
        "scatter_feature",
        r#""places_feature": "demo:y", "iterations": 1, "y": 0, "z": 0"#,
    );
    let document = loads(&json, &cx());
    let Feature::Scatter(feature) = document.value() else {
        panic!("not a scatter feature");
    };
    assert_eq!(feature.distribution.x, None);
    // The rule names the member where a 1.13.0 file has it, at the root.
    kept_at(
        document.diagnostics(),
        "feature/must_be_provided",
        "minecraft:scatter_feature/x",
    );
    assert!(super::common::has_text(
        document.diagnostics(),
        "x must be provided, either as a Molang expression string or as a sub-object"
    ));
}

#[test]
fn f03_the_root_layout_up_to_1_21_0_and_distribution_from_1_21_10() {
    let root = r#""places_feature": "demo:y", "iterations": 1, "x": 0, "y": 0, "z": 0"#;
    let nested =
        r#""places_feature": "demo:y", "distribution": {"iterations": 1, "x": 0, "y": 0, "z": 0}"#;
    let old = loads(&doc("1.21.0", "scatter_feature", root), &cx());
    assert!(old.version().upgraded);
    rejects(&doc("1.21.10", "scatter_feature", root), &cx());
    let new = loads(&doc("1.21.10", "scatter_feature", nested), &cx());
    assert!(!new.version().upgraded);
    rejects(&doc("1.21.0", "scatter_feature", nested), &cx());
    assert_eq!(old.value(), new.value());
}

#[test]
fn a_cereal_minimum_in_a_coordinate_rejects_the_feature() {
    let json = doc(
        "1.21.10",
        "scatter_feature",
        r#""places_feature": "demo:y", "distribution": {"iterations": 1, "x": {"distribution": "fixed_grid", "extent": [0, 4], "step_size": 0}, "y": 0, "z": 0}"#,
    );
    let d = rejects(&json, &cx());
    assert!(d.iter().any(|f| f.code.id() == "core/outside_range"
        && f.location.json_path.to_string().ends_with("x/step_size")));
}

/// A scatter document of `version` with the scatter parameters `members` (after `iterations`),
/// in the shape of its era: below 1.21.10 at the root, from it in `distribution`.
fn scatter_doc(version: &str, members: &str) -> String {
    if version == "1.21.10" {
        doc(
            version,
            "scatter_feature",
            &format!(
                r#""places_feature": "demo:y", "distribution": {{"iterations": 1, {members}}}"#
            ),
        )
    } else {
        doc(
            version,
            "scatter_feature",
            &format!(r#""places_feature": "demo:y", "iterations": 1, {members}"#),
        )
    }
}

/// A scatter document whose `members` hold one number beyond a float: the one warning at
/// `tail`, on `token`, the clamped value, and the written file reading back without it.
fn check_clamped_coordinate(
    version: &str,
    base: &str,
    members: &str,
    tail: &str,
    token: &str,
    expected: f32,
) {
    use bedrock_addon::diagnostic::Phase;
    let text = scatter_doc(version, members);
    let document = loads(&text, &cx());
    let found: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.id() == "core/invalid_value")
        .collect();
    assert_eq!(
        found.len(),
        1,
        "{version} {members}: {:#?}",
        document.diagnostics()
    );
    let finding = found[0];
    assert_eq!(finding.severity, Severity::Info, "{version} {members}");
    assert_eq!(finding.phase, Phase::Parse, "{version} {members}");
    assert_eq!(
        finding.location.json_path.to_string(),
        format!("{base}/{tail}"),
        "{version} {members}"
    );
    let span = finding.location.span.expect("the number's token");
    assert_eq!(
        &text[span.offset as usize..(span.offset + span.len) as usize],
        token,
        "{version} {members}"
    );
    assert_eq!(
        above_info(document.diagnostics()).len(),
        0,
        "{version} {members}"
    );
    // The value, and what is written of it.
    let Feature::Scatter(scatter) = document.value() else {
        panic!("not a scatter feature");
    };
    let coordinate = match tail.as_bytes()[0] {
        b'x' => &scatter.distribution.x,
        b'y' => &scatter.distribution.y,
        _ => &scatter.distribution.z,
    };
    let held = match coordinate.as_ref().unwrap() {
        CoordinateRange::Single(value) => value.clone(),
        CoordinateRange::Object(range) => range.extent[usize::from(tail.ends_with('1'))].clone(),
    };
    assert_eq!(held, Molang::Const(expected), "{version} {members}");
    let written = document.to_json().unwrap();
    let again = loads(&written, &cx());
    assert_eq!(again.value(), document.value(), "{version} {members}");
    assert!(
        above_info(again.diagnostics()).is_empty(),
        "{version} {members}"
    );
}

/// A scatter number beyond `f32` in a member read
/// through a conversion (`scatter_chance`, a coordinate, a bound of its `extent`) is read as
/// the largest finite float of its sign, and core's warning says so at the number, on its
/// token, in both eras. The value is written clamped and reads back without the finding.
#[test]
fn a_scatter_number_beyond_f32_is_the_largest_finite_float_with_a_warning_deviation() {
    for (version, base) in [
        ("1.21.10", "minecraft:scatter_feature/distribution"),
        ("1.13.0", "minecraft:scatter_feature"),
    ] {
        for (members, tail, token, expected) in [
            (r#""x": 1e39, "y": 0, "z": 0"#, "x", "1e39", f32::MAX),
            (r#""x": 0, "y": -1e999, "z": 0"#, "y", "-1e999", f32::MIN),
            (
                r#""x": 0, "y": 0, "z": {"distribution": "uniform", "extent": [0, 1e39]}"#,
                "z/extent/1",
                "1e39",
                f32::MAX,
            ),
            (
                r#""x": {"distribution": "uniform", "extent": [-4e38, 4]}, "y": 0, "z": 0"#,
                "x/extent/0",
                "-4e38",
                f32::MIN,
            ),
        ] {
            check_clamped_coordinate(version, base, members, tail, token, expected);
        }
        // A percentage beyond a float is clamped with the same warning, and is then a
        // chance above 100 like any other.
        let document = loads(
            &scatter_doc(version, r#""scatter_chance": 1e39, "x": 0, "y": 0, "z": 0"#),
            &cx(),
        );
        let mut found: Vec<(&str, String)> = above_info(document.diagnostics())
            .iter()
            .map(|f| (f.code.id(), f.location.json_path.to_string()))
            .collect();
        found.sort();
        assert_eq!(
            found,
            [
                // The clamp's information is not above info.
                // The range rule runs on the upgraded document and names the member
                // where the file has it.
                (
                    "feature/scatter_chance_percent",
                    format!("{base}/scatter_chance")
                ),
            ],
            "{version}"
        );
        // `iterations` is a typed Molang member: the number is refused there.
        let text = scatter_doc(version, r#""x": 0, "y": 0, "z": 0"#)
            .replace(r#""iterations": 1"#, r#""iterations": 1e39"#);
        let refused = rejects(&text, &cx());
        error_at(&refused, "core/invalid_type", "/iterations");
        // A number a float holds raises nothing.
        let fits = loads(
            &scatter_doc(
                version,
                r#""x": 3e38, "y": -3e38, "z": {"distribution": "uniform", "extent": [-3e38, 3e38]}"#,
            ),
            &cx(),
        );
        assert!(
            fits.diagnostics()
                .iter()
                .all(|f| f.kind() != Kind::InvalidValue),
            "{version}: {:#?}",
            fits.diagnostics()
        );
    }
}
