//! The scatter parameters'
//! Molang members take the pack's `min_engine_version` version (an object form's own version is
//! kept for writing and ignored by the compile), compile with the `world_gen` query set, and a
//! rejection is a warning; an empty coordinate expression is "must be provided".

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::molang::{Molang, RawVersion, SourceForm};
use bedrock_addon_family_feature_rules::{CoordinateRange, ScatterChance};

use super::common::*;

/// The raw version of `value`'s source.
fn version(value: &Molang) -> Option<RawVersion> {
    value.source().and_then(|source| source.raw_version())
}

/// The "%s must be provided" kept errors of `d`, with their paths.
fn must_be_provided(d: &bedrock_addon_core::diagnostic::Diagnostics) -> Vec<(String, String)> {
    d.iter()
        .filter(|f| {
            f.code.to_string().ends_with(
                "must be provided, either as a Molang expression string or as a sub-object",
            )
        })
        .map(|f| {
            assert_eq!((f.severity, f.kept), (Severity::Error, true));
            (f.location.json_path.to_string(), f.code.to_string())
        })
        .collect()
}

#[test]
fn r06_the_molang_version_is_the_pack_s_not_the_document_s() {
    let text = rule(
        "1.21.100",
        "surface_pass",
        Some(&distribution(
            r#""math.random(1, 2)""#,
            r#""v.x""#,
            r#"{"distribution": "uniform", "extent": ["v.a", "v.b"]}"#,
            "0",
        )),
    )
    .replace(
        r#""iterations""#,
        r#""scatter_chance": "v.c", "iterations""#,
    );
    for (pack_version, molang) in [([1, 16, 0], 0), ([1, 21, 100], 13)] {
        let [major, minor, patch] = pack_version;
        let document = loads_with(&text, &pack(major, minor, patch));
        let params = document.distribution.as_ref().unwrap();
        let expected = Some(RawVersion(molang));
        assert_eq!(version(&params.iterations), expected, "{pack_version:?}");
        let Some(CoordinateRange::Single(x)) = &params.x else {
            panic!()
        };
        assert_eq!(version(x), expected);
        let Some(CoordinateRange::Object(y)) = &params.y else {
            panic!()
        };
        assert_eq!(version(&y.extent[0]), expected);
        assert_eq!(version(&y.extent[1]), expected);
        let Some(ScatterChance::Percent(chance)) = &params.scatter_chance else {
            panic!()
        };
        assert_eq!(version(chance), expected);
    }
}

#[test]
fn r20_an_empty_coordinate_expression_must_be_provided_and_the_rule_loads() {
    let document = loads(&rule(
        "1.13.0",
        "surface_pass",
        Some(&distribution("1", r#""""#, "0", "0")),
    ));
    assert_eq!(
        must_be_provided(document.diagnostics()),
        [(
            "minecraft:feature_rules/distribution/x".to_owned(),
            "x must be provided, either as a Molang expression string or as a sub-object"
                .to_owned()
        )]
    );
    let document = loads(&rule(
        "1.13.0",
        "surface_pass",
        Some(&distribution("1", UNIFORM, "0", "0")),
    ));
    assert!(must_be_provided(document.diagnostics()).is_empty());
}

#[test]
fn r22_iterations_forms_without_the_compile_check() {
    for (iterations, check) in [
        (
            "2",
            (|m: &Molang| *m == Molang::Const(2.0)) as fn(&Molang) -> bool,
        ),
        ("true", |m| *m == Molang::Bool(true)),
        (r#""Math.Random(1, 2)""#, |m| {
            matches!(
                m.source().map(|s| s.form()),
                Some(SourceForm::String {
                    context_version: Some(RawVersion(12))
                })
            )
        }),
        (r#"{"expression": "2", "version": 0}"#, |m| {
            matches!(
                m.source().map(|s| s.form()),
                Some(SourceForm::Object {
                    version: RawVersion(0)
                })
            )
        }),
    ] {
        let document = loads_with(
            &rule(
                "1.13.0",
                "surface_pass",
                Some(&distribution(iterations, "0", "0", "0")),
            ),
            &pack(1, 21, 80),
        );
        let params = document.distribution.as_ref().unwrap();
        assert!(
            check(&params.iterations),
            "{iterations}: {:?}",
            params.iterations
        );
        assert!(
            above_info(document.diagnostics()).is_empty(),
            "{iterations}"
        );
    }
}

#[test]
fn r26_an_empty_string_is_not_compiled_without_the_feature_either() {
    let document = loads(&rule(
        "1.13.0",
        "surface_pass",
        Some(&distribution("1", r#""""#, "0", "0")),
    ));
    assert!(with_code(document.diagnostics(), "InvalidMolang").is_empty());
    assert_eq!(must_be_provided(document.diagnostics()).len(), 1);
}

#[cfg(not(feature = "molang"))]
#[test]
fn rule_5_without_the_molang_feature_a_rule_with_expressions_is_not_checked() {
    let document = loads(&rule(
        "1.13.0",
        "surface_pass",
        Some(&distribution(r#""v.x""#, "0", "0", "0")),
    ));
    let found = with_code(document.diagnostics(), "MolangNotChecked");
    assert_eq!(found.len(), 1, "{:#?}", document.diagnostics());
    assert_eq!(found[0].severity, Severity::Info);
}

#[cfg(feature = "molang")]
mod compiled {
    use std::sync::Arc;

    use bedrock_addon_core::diagnostic::{DiagnosticDetail, Diagnostics, Severity};
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::molang::{DiagCode, MolangCheck};

    use super::*;

    /// A pack at `min_engine_version` `[major, minor, patch]` that compiles Molang.
    fn checking(major: u16, minor: u16, patch: u16) -> LoadContext {
        let mut cx = pack(major, minor, patch);
        cx.molang = Some(Arc::new(MolangCheck::new()));
        cx
    }

    /// The compiler codes of the Molang findings, with their paths and severities.
    fn molang(d: &Diagnostics) -> Vec<(String, DiagCode, Severity)> {
        d.iter()
            .filter_map(|f| match f.detail.as_deref() {
                Some(DiagnosticDetail::Molang(m)) => {
                    Some((f.location.json_path.to_string(), m.code(), f.severity))
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn r22_iterations_forms_compile_at_the_pack_version_without_a_diagnostic() {
        for iterations in [
            "2",
            "true",
            r#""Math.Random(1, 2)""#,
            r#"{"expression": "2", "version": 0}"#,
        ] {
            let document = loads_with(
                &rule(
                    "1.13.0",
                    "surface_pass",
                    Some(&distribution(iterations, "0", "0", "0")),
                ),
                &checking(1, 21, 80),
            );
            assert!(
                document.diagnostics().is_empty(),
                "{iterations}: {:#?}",
                document.diagnostics()
            );
        }
    }

    #[test]
    fn r23_an_object_form_s_version_is_ignored_by_the_compile() {
        let document = loads_with(
            &rule(
                "1.13.0",
                "surface_pass",
                Some(&distribution(
                    "1",
                    "0",
                    r#"{"expression": "'a' + 1", "version": 2}"#,
                    "0",
                )),
            ),
            &checking(1, 21, 100),
        );
        let found = molang(document.diagnostics());
        assert!(
            found.iter().any(|(path, code, severity)| path
                == "minecraft:feature_rules/distribution/y"
                && *code == DiagCode::StringMisuse
                && *severity == Severity::Error),
            "{:#?}",
            document.diagnostics()
        );
        assert!(!document.diagnostics().has_errors());
        assert_eq!(
            must_be_provided(document.diagnostics()),
            [(
                "minecraft:feature_rules/distribution/y".to_owned(),
                "y must be provided, either as a Molang expression string or as a sub-object"
                    .to_owned()
            )]
        );
        // The object form keeps its own version for writing back.
        let Some(CoordinateRange::Single(y)) = &document.distribution.as_ref().unwrap().y else {
            panic!()
        };
        assert_eq!(version(y), Some(RawVersion(2)));
    }

    #[test]
    fn r24_only_the_world_gen_query_set_resolves() {
        let document = loads_with(
            &rule(
                "1.13.0",
                "surface_pass",
                Some(&distribution(
                    "1",
                    r#""q.is_baby""#,
                    r#""query.heightmap(variable.worldx, variable.worldz)""#,
                    "0",
                )),
            ),
            &checking(1, 21, 100),
        );
        let found = molang(document.diagnostics());
        assert!(
            found.iter().all(|(path, _, severity)| path
                == "minecraft:feature_rules/distribution/x"
                && *severity == Severity::Error),
            "{:#?}",
            document.diagnostics()
        );
        assert!(
            found
                .iter()
                .any(|(_, code, _)| *code == DiagCode::UnknownQuery),
            "{found:?}"
        );
        assert!(document.diagnostics().iter().any(|f| f.code.to_string().contains(
            "Failed to resolve query query.is_baby.  Either the query does not exist or it is not supported in this context."
        ) || format!("{f}").contains("Failed to resolve query query.is_baby.")), "{:#?}", document.diagnostics());
        assert_eq!(
            must_be_provided(document.diagnostics()),
            [(
                "minecraft:feature_rules/distribution/x".to_owned(),
                "x must be provided, either as a Molang expression string or as a sub-object"
                    .to_owned()
            )]
        );
        // The rule-7 text follows the compile's findings.
        let last = document.diagnostics().iter().last().unwrap();
        assert!(last.code.to_string().starts_with("x must be provided"));
    }

    #[test]
    fn r25_unparsable_iterations_warn_and_are_not_a_bad_value() {
        let document = loads_with(
            &rule(
                "1.13.0",
                "surface_pass",
                Some(&distribution(r#""1 +""#, "0", "0", "0")),
            ),
            &checking(1, 21, 100),
        );
        let found = molang(document.diagnostics());
        assert!(
            found.iter().any(|(path, _, severity)| path
                == "minecraft:feature_rules/distribution/iterations"
                && *severity == Severity::Error),
            "{:#?}",
            document.diagnostics()
        );
        assert!(
            !document
                .diagnostics()
                .iter()
                .any(|f| f.code.to_string().starts_with("Bad value for iterations")),
        );
    }

    #[test]
    fn r26_an_empty_string_is_not_compiled() {
        let document = loads_with(
            &rule(
                "1.13.0",
                "surface_pass",
                Some(&distribution("1", r#""""#, "0", "0")),
            ),
            &checking(1, 21, 100),
        );
        assert!(molang(document.diagnostics()).is_empty());
        assert_eq!(must_be_provided(document.diagnostics()).len(), 1);
    }
}
