//! The scatter parameters' forms,
//! their bind constraints (errors) and `ScatterParams::fillFromData`'s runtime checks (warnings
//! with vanilla's text, the value replaced or ignored).

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::Document;
use bedrock_addon_core::molang::Molang;
use bedrock_addon_family_feature_rules::{
    CoordinateRange, FeatureRule, RandomDistribution, ScatterChance, ScatterParams,
};

use super::common::*;

/// A rule with `distribution` (a JSON object's text).
fn with_distribution(distribution: &str) -> String {
    rule("1.13.0", "surface_pass", Some(distribution))
}

/// A rule whose `distribution` has `scatter_chance` and constant coordinates.
fn with_chance(chance: &str) -> String {
    with_distribution(&format!(
        r#"{{"iterations": 1, "scatter_chance": {chance}, "x": 0, "y": 0, "z": 0}}"#
    ))
}

/// A rule whose `x` is `x` (a JSON text), the other coordinates 0.
fn with_x(x: &str) -> String {
    with_distribution(&distribution("1", x, "0", "0"))
}

/// The distribution of a loaded rule.
fn params(document: &Document<FeatureRule>) -> &ScatterParams {
    document.distribution.as_ref().expect("a distribution")
}

/// The messages of the `InvalidValue` kept errors of `document` (the game's content-log lines,
/// the rule kept), with their paths.
fn warnings(document: &Document<FeatureRule>) -> Vec<(String, String)> {
    document
        .diagnostics()
        .iter()
        .filter(|f| f.severity == Severity::Error && f.kept && f.code.name() == "InvalidValue")
        .map(|f| (f.location.json_path.to_string(), f.code.to_string()))
        .collect()
}

#[test]
fn r10_scatter_chance_forms_load_clean() {
    for (chance, expected) in [
        ("50", ScatterChance::Percent(Molang::Const(50.0))),
        ("100", ScatterChance::Percent(Molang::Const(100.0))),
    ] {
        let document = loads(&with_chance(chance));
        assert!(above_info(document.diagnostics()).is_empty(), "{chance}");
        assert_eq!(params(&document).scatter_chance, Some(expected));
    }
    let document = loads(&with_chance(r#""v.x""#));
    assert!(above_info(document.diagnostics()).is_empty());
    assert!(matches!(
        &params(&document).scatter_chance,
        Some(ScatterChance::Percent(Molang::Expr(s))) if s.as_str() == "v.x"
    ));
    let document = loads(&with_chance(r#"{"numerator": 1, "denominator": 3}"#));
    assert!(above_info(document.diagnostics()).is_empty());
    let Some(ScatterChance::Fraction(fraction)) = &params(&document).scatter_chance else {
        panic!("a fraction")
    };
    assert_eq!((fraction.numerator, fraction.denominator), (1, 3));
}

#[test]
fn r10_a_denominator_not_above_the_numerator_is_logged_and_becomes_1() {
    let document = loads(&with_chance(r#"{"numerator": 1, "denominator": 1}"#));
    assert_eq!(
        warnings(&document),
        [(
            "minecraft:feature_rules/distribution/scatter_chance".to_owned(),
            "Bad value for scatter_chance - denominator should be greater than the numerator"
                .to_owned()
        )]
    );
    let Some(ScatterChance::Fraction(fraction)) = &params(&document).scatter_chance else {
        panic!("a fraction")
    };
    assert_eq!(
        fraction.denominator, 1,
        "the document keeps what was written"
    );
    let ScatterChance::Fraction(effective) = params(&document).effective_chance() else {
        panic!("a fraction")
    };
    assert_eq!((effective.numerator, effective.denominator), (1, 1));
    let document = loads(&with_chance(r#"{"numerator": 3, "denominator": 2}"#));
    let ScatterChance::Fraction(effective) = params(&document).effective_chance() else {
        panic!("a fraction")
    };
    assert_eq!((effective.numerator, effective.denominator), (3, 1));
}

#[test]
fn r10_a_numerator_below_1_is_a_constraint_error_and_the_file_is_skipped() {
    for chance in [
        r#"{"numerator": 0, "denominator": 3}"#,
        r#"{"numerator": 1, "denominator": 0}"#,
    ] {
        let d = rejects(&with_chance(chance));
        let found = with_code(&d, "InvalidValue");
        assert_eq!(found.len(), 1, "{chance}: {d:#?}");
        assert_eq!(found[0].severity, Severity::Error);
    }
}

#[test]
fn rule_7_a_percentage_outside_0_to_100_is_logged_and_becomes_100() {
    for chance in ["0", "-5", "100.5"] {
        let document = loads(&with_chance(chance));
        assert_eq!(
            warnings(&document),
            [(
                "minecraft:feature_rules/distribution/scatter_chance".to_owned(),
                "Bad value for scatter_chance - should be between 0 and 100 (exclusive)".to_owned()
            )],
            "{chance}"
        );
        assert_eq!(
            params(&document).effective_chance(),
            ScatterChance::Percent(Molang::Const(100.0))
        );
    }
}

#[test]
fn r11_coordinate_forms_load() {
    let document = loads(&with_x("3"));
    assert_eq!(
        params(&document).x,
        Some(CoordinateRange::Single(Molang::Const(3.0)))
    );
    let document = loads(&with_x(r#""math.random(0, 15)""#));
    assert!(
        matches!(&params(&document).x, Some(CoordinateRange::Single(Molang::Expr(s))) if s.as_str() == "math.random(0, 15)")
    );
    let document = loads(&with_x(
        r#"{"distribution": "fixed_grid", "extent": [0, 15], "step_size": 4, "grid_offset": 1}"#,
    ));
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let Some(CoordinateRange::Object(x)) = &params(&document).x else {
        panic!("an object")
    };
    assert_eq!(x.distribution, RandomDistribution::FixedGrid);
    assert_eq!((x.effective_step_size(), x.effective_grid_offset()), (4, 1));
    let document = loads(&with_x(
        r#"{"distribution": "jittered_grid", "extent": [0, 15]}"#,
    ));
    let Some(CoordinateRange::Object(x)) = &params(&document).x else {
        panic!("an object")
    };
    assert_eq!((x.effective_step_size(), x.effective_grid_offset()), (1, 0));
}

#[test]
fn r11_a_grid_member_with_another_distribution_is_logged_and_ignored() {
    let document = loads(&with_x(
        r#"{"distribution": "uniform", "extent": [0, 15], "step_size": 4, "grid_offset": 2}"#,
    ));
    assert_eq!(
        warnings(&document),
        [
            (
                "minecraft:feature_rules/distribution/x/step_size".to_owned(),
                "step_size is only valid when distribution is fixed_grid or jittered_grid"
                    .to_owned()
            ),
            (
                "minecraft:feature_rules/distribution/x/grid_offset".to_owned(),
                "grid_offset is only valid when distribution is fixed_grid or jittered_grid"
                    .to_owned()
            ),
        ]
    );
    let Some(CoordinateRange::Object(x)) = &params(&document).x else {
        panic!("an object")
    };
    assert_eq!(x.step_size, Some(4), "kept as written");
    assert_eq!((x.effective_step_size(), x.effective_grid_offset()), (1, 0));
}

#[test]
fn r11_an_empty_extent_is_logged_and_kept() {
    for extent in ["[5, 5]", "[5, 4]", "[4.6, 5.4]"] {
        let document = loads(&with_x(&format!(
            r#"{{"distribution": "uniform", "extent": {extent}}}"#
        )));
        assert_eq!(
            warnings(&document),
            [(
                "minecraft:feature_rules/distribution/x/extent".to_owned(),
                "Bad value for extent - maximum must be greater than minimum".to_owned()
            )],
            "{extent}"
        );
    }
    for extent in ["[4.4, 5.4]", r#"["v.a", 0]"#] {
        let document = loads(&with_x(&format!(
            r#"{{"distribution": "uniform", "extent": {extent}}}"#
        )));
        assert!(warnings(&document).is_empty(), "{extent}");
    }
}

#[test]
fn rule_6_a_grid_member_below_its_minimum_is_a_constraint_error() {
    for x in [
        r#"{"distribution": "fixed_grid", "extent": [0, 15], "step_size": 0}"#,
        r#"{"distribution": "fixed_grid", "extent": [0, 15], "grid_offset": -1}"#,
    ] {
        let d = rejects(&with_x(x));
        let found = with_code(&d, "InvalidValue");
        assert_eq!(found.len(), 1, "{x}: {d:#?}");
        assert_eq!(found[0].severity, Severity::Error);
    }
}

#[test]
fn r14_negative_iterations_are_logged_and_become_1() {
    let document = loads(&with_distribution(&distribution("-2", "0", "0", "0")));
    assert_eq!(
        warnings(&document),
        [(
            "minecraft:feature_rules/distribution/iterations".to_owned(),
            "Bad value for iterations - should be >= 0".to_owned()
        )]
    );
    assert_eq!(params(&document).iterations, Molang::Const(-2.0));
    assert_eq!(params(&document).effective_iterations(), Molang::Const(1.0));
    let document = loads(&with_distribution(&distribution("0", "0", "0", "0")));
    assert!(warnings(&document).is_empty());
    assert_eq!(params(&document).effective_iterations(), Molang::Const(0.0));
}

#[test]
fn r21_without_scatter_chance_the_chance_is_100_percent_and_the_order_xzy() {
    let document = loads(&with_distribution(&distribution("1", "0", "0", "0")));
    let params = params(&document);
    assert_eq!(params.scatter_chance, None);
    assert_eq!(
        params.effective_chance(),
        ScatterChance::Percent(Molang::Const(100.0))
    );
    assert_eq!(
        params.eval_order(),
        bedrock_addon_family_feature_rules::CoordinateEvalOrder::Xzy
    );
}

/// Every Molang member of a rule's `distribution` is
/// read as a typed member from its own slice, so a number beyond `f32` is refused where it
/// stands (`InvalidType`, the file not loaded) and never clamped: there is nothing to warn
/// about and nothing written that the author did not write.
#[test]
fn a_distribution_number_beyond_f32_is_refused_where_it_stands() {
    for (text, at) in [
        (
            with_distribution(&distribution("1e39", "0", "0", "0")),
            "minecraft:feature_rules/distribution/iterations",
        ),
        (
            with_chance("1e39"),
            "minecraft:feature_rules/distribution/scatter_chance",
        ),
        (with_x("1e39"), "minecraft:feature_rules/distribution/x"),
        (
            with_x(r#"{"distribution": "uniform", "extent": [0, 1e39]}"#),
            "minecraft:feature_rules/distribution/x/extent/1",
        ),
    ] {
        let found = rejects(&text);
        let errors: Vec<(&str, String)> = found
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .map(|f| (f.code.name(), f.location.json_path.to_string()))
            .collect();
        assert_eq!(errors, [("InvalidType", at.to_owned())], "{text}");
    }
    // A number no double holds refuses the document, as the game's JSON reader does.
    let found = rejects(&with_x(
        r#"{"distribution": "uniform", "extent": [-1e999, 4]}"#,
    ));
    let errors: Vec<(&str, String)> = found
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| (f.code.name(), f.location.json_path.to_string()))
        .collect();
    assert_eq!(errors, [("MalformedInput", String::new())]);
    // The largest float itself is a number like any other.
    let document = loads(&with_x(
        r#"{"distribution": "uniform", "extent": [0, 3.4028235e38]}"#,
    ));
    assert!(above_info(document.diagnostics()).is_empty());
}
