//! Molang in features: scatter members read leniently
//! and compiled at the pack's version, typed world-generation members that reject the feature,
//! and descriptor `tags` that are logged and kept.

use bedrock_addon_core::molang::{JsonScalar, Molang, SourceForm};
use bedrock_addon_family_feature::{Feature, ScatterFeature};

use super::common::{cx, doc, loads};

/// A 1.21.10 scatter feature whose `distribution` has `iterations` and three coordinates.
fn scatter(iterations: &str, x: &str) -> String {
    doc(
        "1.21.10",
        "scatter_feature",
        &format!(
            r#""places_feature": "demo:y", "distribution": {{"iterations": {iterations}, "x": {x}, "y": 0, "z": 0}}"#
        ),
    )
}

/// The scatter feature of a loaded document.
fn scatter_of(feature: Feature) -> ScatterFeature {
    match feature {
        Feature::Scatter(scatter) => scatter,
        other => panic!("not a scatter feature: {other:?}"),
    }
}

#[test]
fn f26_the_four_forms_of_iterations_are_read() {
    let constant = scatter_of(loads(&scatter("4", "0"), &cx()).into_value());
    assert_eq!(constant.distribution.iterations, Molang::Const(4.0));
    let boolean = scatter_of(loads(&scatter("true", "0"), &cx()).into_value());
    assert_eq!(boolean.distribution.iterations, Molang::Bool(true));
    let text = scatter_of(loads(&scatter(r#""math.random(1, 2)""#, "0"), &cx()).into_value());
    assert!(matches!(text.distribution.iterations, Molang::Expr(_)));
    let object = loads(
        &scatter(
            r#"{"expression": "q.noise(v.worldx, v.worldz)", "version": 2}"#,
            "0",
        ),
        &cx(),
    );
    let feature = scatter_of(object.value().clone());
    let Molang::Expr(source) = &feature.distribution.iterations else {
        panic!("not an expression");
    };
    assert!(matches!(source.form(), SourceForm::Object { version } if version.0 == 2));
    // Writing back keeps the object form with its own version.
    assert!(matches!(
        feature.distribution.iterations.write(),
        JsonScalar::Object { version, .. } if version.0 == 2
    ));
    let written = bedrock_addon_core::document::Content::to_json(object.value()).unwrap();
    assert!(written.contains(r#""version": 2"#), "{written}");
}

#[cfg(feature = "molang")]
mod compiled {
    use std::sync::Arc;

    use bedrock_addon_core::diagnostic::{DiagnosticDetail, Diagnostics, Severity};
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::molang::molang::stdlib;
    use bedrock_addon_core::molang::{
        CompileCache, DiagCode, MolangCheck, MolangVersion, RawVersion, Side,
    };
    use semver::Version;

    use super::super::common::{error_at, kept_at, rejects};
    use super::*;

    /// A third-party pack at engine 1.21.100 (Molang 13) that compiles Molang with `check`.
    fn checking_with(check: MolangCheck) -> LoadContext {
        let mut cx = cx();
        cx.molang = Some(Arc::new(check));
        cx.molang_version = RawVersion::from(MolangVersion::V13);
        cx
    }

    fn checking() -> LoadContext {
        checking_with(MolangCheck::new())
    }

    /// The compiler codes of the Molang findings, with their severities.
    fn molang(d: &Diagnostics) -> Vec<(DiagCode, Severity)> {
        d.iter()
            .filter_map(|f| match f.detail.as_deref() {
                Some(DiagnosticDetail::Molang(m)) => Some((m.code(), f.severity)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn f26_every_form_loads_without_a_diagnostic() {
        for iterations in [
            "4",
            "true",
            r#""math.random(1, 2)""#,
            r#"{"expression": "q.noise(v.worldx, v.worldz)", "version": 2}"#,
        ] {
            let document = loads(&scatter(iterations, "0"), &checking());
            assert!(
                document.diagnostics().is_empty(),
                "{iterations}: {:#?}",
                document.diagnostics()
            );
        }
    }

    #[test]
    fn f27_a_scatter_object_forms_version_is_ignored_when_compiling() {
        let cx = checking();
        loads(&scatter(r#""q.heightmap(v.worldx, v.worldz)""#, "0"), &cx);
        loads(
            &scatter(
                r#"{"expression": "q.heightmap(v.worldx, v.worldz)", "version": 0}"#,
                "0",
            ),
            &cx,
        );
        let cache = cx.molang.as_ref().unwrap().cache();
        assert_eq!(cache.misses(), 1);
        assert_eq!(cache.hits(), 1);
    }

    #[test]
    fn f28_a_default_query_in_a_scatter_member_is_logged_and_the_feature_loads() {
        let document = loads(&scatter(r#""q.is_baby""#, "0"), &checking());
        let found = molang(document.diagnostics());
        assert!(
            found.contains(&(DiagCode::UnknownQuery, Severity::Error)),
            "{found:?}"
        );
        assert!(
            found.contains(&(DiagCode::Syntax, Severity::Error)),
            "{found:?}"
        );
        assert!(!document.diagnostics().has_errors());
        assert!(matches!(
            scatter_of(document.value().clone()).distribution.iterations,
            Molang::Expr(_)
        ));
    }

    #[test]
    fn f29_an_empty_coordinate_is_not_compiled_and_is_warned() {
        let document = loads(&scatter("1", r#""""#), &checking());
        assert!(molang(document.diagnostics()).is_empty());
        kept_at(
            document.diagnostics(),
            "feature/must_be_provided",
            "distribution/x",
        );
        assert!(super::super::common::has_text(
            document.diagnostics(),
            "x must be provided, either as a Molang expression string or as a sub-object"
        ));
    }

    fn carver(width: &str) -> String {
        doc(
            "1.13.0",
            "cave_carver_feature",
            &format!(r#""fill_with": "minecraft:air", "width_modifier": {width}"#),
        )
    }

    #[test]
    fn f30_carver_width_modifier() {
        loads(&carver(r#""q.noise(v.worldx, v.worldz) * 2""#), &checking());
        let d = rejects(&carver(r#""1 +""#), &checking());
        assert!(
            molang(&d).contains(&(DiagCode::StatementForm, Severity::Error)),
            "{d:#?}"
        );
        let d = rejects(
            &carver(r#"{"expression": "1", "version": 13}"#),
            &checking(),
        );
        error_at(&d, "core/invalid_type", "width_modifier");
    }

    fn conditional(condition: &str) -> String {
        doc(
            "1.13.0",
            "conditional_list",
            &format!(
                r#""conditional_features": [{{"places_feature": "demo:y", "condition": "{condition}"}}]"#
            ),
        )
    }

    /// A base-game pack that compiles with `check`.
    fn base_checking(check: MolangCheck) -> LoadContext {
        let mut cx = checking_with(check);
        cx.is_base_game_pack = true;
        cx
    }

    #[test]
    fn f31_conditional_list_condition() {
        let d = rejects(
            &conditional("q.is_baby"),
            &base_checking(MolangCheck::new()),
        );
        let found = molang(&d);
        assert!(
            found.contains(&(DiagCode::UnknownQuery, Severity::Error)),
            "{found:?}"
        );
        loads(
            &conditional("q.has_biome_tag('ocean')"),
            &base_checking(MolangCheck::new()),
        );
    }

    #[test]
    fn f32_descriptor_tags_compile_under_the_tags_set_and_are_kept() {
        let ore = |tags: &str| {
            doc(
                "1.13.0",
                "ore_feature",
                &format!(
                    r#""count": 1, "replace_rules": [{{"places_block": {{"tags": "{tags}"}}}}]"#
                ),
            )
        };
        let document = loads(&ore("q.any_tag('stone')"), &checking());
        assert!(molang(document.diagnostics()).is_empty());
        let document = loads(&ore("q.is_baby"), &checking());
        let found = molang(document.diagnostics());
        assert!(
            found.contains(&(DiagCode::UnknownQuery, Severity::Error)),
            "{found:?}"
        );
        assert!(!document.diagnostics().has_errors());
    }

    #[test]
    fn f33_world_gen_queries_follow_the_catalogue() {
        let condition = conditional("q.has_any_biome_tags('a')");
        loads(&condition, &base_checking(MolangCheck::new()));
        let old = MolangCheck::with_catalogs(
            stdlib::queries_at(Side::Server, &Version::new(1, 26, 20)),
            stdlib::queries(Side::Client).clone(),
            Arc::new(CompileCache::new()),
        )
        .unwrap();
        let d = rejects(&condition, &base_checking(old));
        assert!(
            d.iter()
                // By legacy name: the variant and the definitions that were it.
                .any(|f| matches!(f.code.id(), "core/invalid_molang")
                    && f.severity == Severity::Error)
        );
        assert!(molang(&d).contains(&(DiagCode::UnknownQuery, Severity::Error)));
    }
}

#[cfg(not(feature = "molang"))]
#[test]
fn without_the_molang_feature_expressions_are_not_checked() {
    let document = loads(&scatter(r#""q.is_baby""#, "0"), &cx());
    assert!(document.diagnostics().iter().any(|f| {
        f.code
            .args_of(&bedrock_addon_core::diagnostic::codes::MOLANG_NOT_CHECKED)
            .is_some_and(|a| a[0] == "1")
    }));
}
