//! the fixture rows 20–28 (molang): how each Molang member is read, which
//! version it compiles at and what a failure does. Without the `molang` feature a document with
//! expressions gets one `MolangNotChecked` info.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::molang::{Molang, RawVersion};
use bedrock_addon::version::MinEngineVersion;

use super::common::{animation, bp, expect_prefix, lines, loaded, rejected};

fn anim(body: &str) -> String {
    format!(r#"{{"format_version": "1.8.0", "animations": {{"animation.x": {body}}}}}"#)
}

#[cfg(feature = "molang")]
fn ctrl(version: &str, state: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "animation_controllers": {{"controller.animation.x": {{"states": {{"default": {state}, "b": {{}}}}}}}}}}"#
    )
}

#[test]
fn r20_molang_forms_anim() {
    let cx = bp(1, 16, 0);
    for (json, check) in [
        (
            "0.5",
            (|m: &Molang| matches!(m, Molang::Const(x) if (*x - 0.5).abs() < f32::EPSILON))
                as fn(&Molang) -> bool,
        ),
        ("true", |m| matches!(m, Molang::Bool(true))),
        (
            r#""q.anim_time * 2""#,
            |m| matches!(m, Molang::Expr(s) if s.raw_version() == Some(RawVersion::from(MinEngineVersion::new(1, 16, 0).molang_version()))),
        ),
    ] {
        let file = loaded(animation(
            &anim(&format!(r#"{{"blend_weight": {json}}}"#)),
            &cx,
        ));
        let weight = file.animations["animation.x"]
            .blend_weight
            .as_ref()
            .unwrap();
        assert!(check(weight), "{json}: {weight:?}");
        assert!(
            file.diagnostics()
                .at_least(Severity::Info)
                .all(|d| d.code.name() != "InvalidMolang" && d.code.name() != "MolangLint"),
            "{json}: {:?}",
            lines(file.diagnostics())
        );
    }
    for json in [r#"{"expression": "1", "version": 13}"#, "null"] {
        let d = rejected(animation(
            &anim(&format!(r#"{{"blend_weight": {json}}}"#)),
            &cx,
        ));
        expect_prefix(
            &d,
            "InvalidType",
            Severity::Error,
            "unknown child schema option type.  Allowed types: ",
        );
        assert!(d.iter().all(|f| f.code.name() != "InvalidMolang"), "{json}");
    }
}

#[cfg(not(feature = "molang"))]
#[test]
fn without_the_feature_a_document_with_expressions_is_not_checked() {
    let file = loaded(animation(
        &anim(r#"{"blend_weight": "1 +"}"#),
        &bp(1, 16, 0),
    ));
    assert!(
        file.diagnostics()
            .iter()
            .any(|d| d.code.name() == "MolangNotChecked" && d.severity == Severity::Info),
        "{:?}",
        lines(file.diagnostics())
    );
}

#[cfg(feature = "molang")]
mod compiled {
    use std::sync::Arc;

    use bedrock_addon::diagnostic::{DiagnosticDetail, Diagnostics, Severity};
    use bedrock_addon::molang::{DiagCode, MolangCheck};

    use super::super::common::{animation, bp, controller, lines, loaded, rejected};
    use bedrock_addon::families::animation::OneOrMany;

    use super::{anim, ctrl};

    /// The `InvalidMolang` findings: the compiler's code and the severity.
    fn invalid(d: &Diagnostics) -> Vec<(DiagCode, Severity)> {
        d.iter()
            .filter(|f| f.code.name() == "InvalidMolang")
            .map(|f| match f.detail.as_deref() {
                Some(DiagnosticDetail::Molang(m)) => (m.code(), f.severity),
                _ => (DiagCode::Syntax, f.severity),
            })
            .collect()
    }

    #[test]
    fn r21_molang_bad_typed_member_drops_the_file() {
        let d = rejected(animation(&anim(r#"{"loop_delay": "1 +"}"#), &bp(1, 16, 0)));
        assert_eq!(
            invalid(&d),
            [(DiagCode::StatementForm, Severity::Error)],
            "{:?}",
            lines(&d)
        );
        let d = rejected(controller(
            &ctrl("1.10.0", r#"{"variables": {"v": {"input": "1 +"}}}"#),
            &bp(1, 16, 0),
        ));
        assert_eq!(invalid(&d), [(DiagCode::StatementForm, Severity::Error)]);
    }

    #[test]
    fn r22_molang_wrong_query_set_drops_the_file() {
        for text in ["q.noise(1, 2)", "q.is_on_screen"] {
            let d = rejected(animation(
                &anim(&format!(r#"{{"blend_weight": "{text}"}}"#)),
                &bp(1, 16, 0),
            ));
            assert_eq!(
                invalid(&d),
                [
                    (DiagCode::UnknownQuery, Severity::Error),
                    (DiagCode::Syntax, Severity::Error)
                ],
                "{text}: {:?}",
                lines(&d)
            );
            let messages: Vec<String> = d
                .iter()
                .filter_map(|f| match f.detail.as_deref() {
                    Some(DiagnosticDetail::Molang(m)) => Some(m.message().to_string()),
                    _ => None,
                })
                .collect();
            let query = text.split('(').next().unwrap().replacen("q.", "query.", 1);
            assert_eq!(
                messages,
                [
                    format!(
                        "Failed to resolve query {query}.  Either the query does not exist or it is not supported in this context."
                    ),
                    format!("unrecognized token: {text}"),
                ],
                "{text}"
            );
        }
    }

    #[test]
    fn r23_molang_transition_unchecked_in_1_10_but_typed_in_1_8() {
        let file = loaded(controller(
            &ctrl("1.10.0", r#"{"transitions": [{"b": "v.x = 1"}]}"#),
            &bp(1, 16, 0),
        ));
        assert_eq!(
            invalid(file.diagnostics()),
            [(DiagCode::StatementForm, Severity::Error)]
        );
        // The Molang parser's content-log error, and the file loads with the constant 0: a
        // kept error.
        assert!(file.diagnostics().iter().all(|d| !d.rejects()));
        let d = rejected(controller(
            &ctrl("1.8.0", r#"{"transitions": [{"b": "v.x = 1"}]}"#),
            &bp(1, 16, 0),
        ));
        assert_eq!(invalid(&d), [(DiagCode::StatementForm, Severity::Error)]);
    }

    #[test]
    fn r24_molang_event_unchecked() {
        let file = loaded(animation(
            &anim(r#"{"timeline": {"0.0": "1 +"}}"#),
            &bp(1, 16, 0),
        ));
        assert_eq!(
            invalid(file.diagnostics()),
            [(DiagCode::StatementForm, Severity::Error)]
        );
        assert!(file.diagnostics().iter().all(|d| !d.rejects()));
        let timeline = file.animations["animation.x"].timeline.as_ref().unwrap();
        assert!(matches!(&timeline["0.0"], OneOrMany::One(_)));
        let file = loaded(controller(
            &ctrl("1.10.0", r#"{"on_entry": ["1 +"]}"#),
            &bp(1, 16, 0),
        ));
        assert_eq!(
            invalid(file.diagnostics()),
            [(DiagCode::StatementForm, Severity::Error)]
        );
    }

    #[test]
    fn r25_molang_effective_version_compile() {
        let text = ctrl("1.10.0", r#"{"transitions": [{"b": "'a' + 1"}]}"#);
        let file = loaded(controller(&text, &bp(1, 16, 0)));
        assert!(
            invalid(file.diagnostics()).is_empty(),
            "{:?}",
            lines(file.diagnostics())
        );
        let file = loaded(controller(&text, &bp(1, 20, 50)));
        assert_eq!(
            invalid(file.diagnostics()),
            [(DiagCode::StringMisuse, Severity::Error)]
        );
    }

    #[test]
    fn r26_molang_empty_below_v4_takes_the_member_severity() {
        let text = ctrl("1.10.0", r#"{"variables": {"v": {"input": ""}}}"#);
        let d = rejected(controller(&text, &bp(1, 16, 0)));
        assert!(
            d.iter()
                .any(|f| f.code.name() == "InvalidMolang" && f.severity == Severity::Error),
            "{:?}",
            lines(&d)
        );
        let d = rejected(controller(&text, &bp(1, 20, 50)));
        assert!(
            d.iter().any(|f| f.code.name() == "InvalidMolang"
                && f.severity == Severity::Error
                && matches!(f.detail.as_deref(), Some(DiagnosticDetail::Molang(m)) if m.code() == DiagCode::Syntax && m.message().contains("No tokens found in expression"))),
            "{:?}",
            lines(&d)
        );
    }

    #[test]
    fn r27_molang_default_anim_time_update_is_one_cache_entry() {
        let mut cx = bp(1, 16, 0);
        let check = Arc::new(MolangCheck::new());
        cx.molang = Some(Arc::clone(&check));
        let text = r#"{"format_version": "1.8.0", "animations": {"animation.a": {}, "animation.b": {"loop": true}}}"#;
        let file = loaded(animation(text, &cx));
        assert!(invalid(file.diagnostics()).is_empty());
        assert_eq!(check.cache().misses(), 1);
        assert_eq!(check.cache().hits(), 1);
    }

    #[test]
    fn r28_molang_resource_names_are_a_warning_on_the_server() {
        let file = loaded(animation(
            &anim(r#"{"blend_weight": "geometry.default"}"#),
            &bp(1, 16, 0),
        ));
        assert!(
            file.diagnostics()
                .iter()
                .any(|f| f.code.name() == "InvalidMolang" && f.severity == Severity::Info),
            "{:?}",
            lines(file.diagnostics())
        );
    }
}

/// The Molang lints of `d` as `<severity> <molang code> <path>`.
#[cfg(feature = "molang")]
fn lints(d: &bedrock_addon::diagnostic::Diagnostics) -> Vec<String> {
    use bedrock_addon::diagnostic::{DiagnosticCode, DiagnosticDetail};
    d.iter()
        .filter(|f| matches!(f.code, DiagnosticCode::MolangLint { .. }))
        .map(|f| {
            let code = match f.detail.as_deref() {
                Some(DiagnosticDetail::Molang(m)) => format!("{:?}", m.code()),
                _ => "-".to_owned(),
            };
            format!("{:?} {code} {}", f.severity, f.location.json_path)
        })
        .collect()
}

/// A pack-setting query in a behaviour pack's animation
/// controller or animation is one warning of the expression, as it is in any other
/// behaviour-pack Molang. The family's own rule stays: no other client-only note (the
/// game's default `anim_time_update` reads `query.anim_time` on the server).
#[cfg(feature = "molang")]
#[test]
fn s13_a_pack_setting_query_in_a_behavior_pack_is_one_warning_of_the_expression() {
    use super::common::{controller, rp};
    // The queries exist from 1.21.130.
    let cx = bp(1, 21, 130);
    let states = "animation_controllers/controller.animation.x/states/default";
    // A transition that calls two of them, and an event that calls the third.
    let file = loaded(controller(
        &ctrl(
            "1.10.0",
            r#"{"transitions": [{"b": "query.get_pack_setting('demo:slider') > 0 && query.is_pack_setting_enabled('demo:toggle')"}], "on_entry": ["v.x = query.is_pack_setting_selected('demo:list', 'a');"]}"#,
        ),
        &cx,
    ));
    assert_eq!(
        lints(file.diagnostics()),
        [
            format!("Info QueryClientOnly {states}/transitions/0/b"),
            format!("Info QueryClientOnly {states}/on_entry/0"),
        ],
        "{:?}",
        lines(file.diagnostics())
    );
    // A typed member of an animation.
    let file = loaded(animation(
        &anim(r#"{"blend_weight": "query.get_pack_setting('demo:slider')"}"#),
        &cx,
    ));
    assert_eq!(
        lints(file.diagnostics()),
        ["Info QueryClientOnly animations/animation.x/blend_weight"]
    );
    // Another client-only query says nothing, alone or beside a pack-setting query.
    let file = loaded(controller(
        &ctrl(
            "1.10.0",
            r#"{"transitions": [{"b": "query.anim_time > 1"}, {"b": "query.anim_time > query.get_pack_setting('demo:slider')"}]}"#,
        ),
        &cx,
    ));
    assert_eq!(
        lints(file.diagnostics()),
        [format!("Info QueryClientOnly {states}/transitions/1/b")]
    );
    // In a resource pack the queries are the client's.
    let file = loaded(controller(
        &ctrl(
            "1.10.0",
            r#"{"transitions": [{"b": "query.get_pack_setting('demo:slider') > 0"}]}"#,
        ),
        &rp(1, 21, 130),
    ));
    assert_eq!(lints(file.diagnostics()), [] as [&str; 0]);
}

/// The family's own rule beside the load's: an animation keeps molang's argument-count lint for
/// the calls the load passes over in vanilla's files (`query.anger_level(this)`).
#[cfg(feature = "molang")]
#[test]
fn an_animation_keeps_the_argument_count_lint_the_load_passes_over() {
    let file = loaded(animation(
        &anim(r#"{"blend_weight": "query.anger_level(this)"}"#),
        &bp(1, 21, 130),
    ));
    assert_eq!(
        lints(file.diagnostics()),
        ["Info QueryArity animations/animation.x/blend_weight"],
        "{:?}",
        lines(file.diagnostics())
    );
}
