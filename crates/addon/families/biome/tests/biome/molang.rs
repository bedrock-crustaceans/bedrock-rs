//! The Molang of `surface_material_adjustments.adjustments[].height_range`: the reader and the constant version 1 in both builds, the compile
//! pass and the glue text with the `molang` feature, `MolangNotChecked` without it. Rows
//! `b21`–`b26`.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::molang::{Molang, RawVersion};
use bedrock_addon_family_biome::component::SurfaceMaterialAdjustments;

use super::common::{biome, context, errors, rejected, with};

/// A 1.21.110 biome with one adjustment of `height_range`.
fn height_range(range: &str) -> String {
    biome(
        "1.21.110",
        "test:heights",
        &format!(
            r#""minecraft:surface_material_adjustments": {{"adjustments": [{{"materials": {{"top_material": "minecraft:sand"}}, "height_range": {range}}}]}}"#
        ),
    )
}

/// The height range of the first adjustment.
fn range_of(document: &bedrock_addon_family_biome::Biome) -> [Molang; 2] {
    document
        .components
        .get::<SurfaceMaterialAdjustments>()
        .and_then(|c| c.adjustments.as_ref())
        .and_then(|a| a[0].height_range.clone())
        .expect("a height range")
}

/// The raw version of an expression.
fn version(value: &Molang) -> Option<RawVersion> {
    value
        .source()
        .and_then(bedrock_addon_core::molang::MolangSource::raw_version)
}

/// The forms of `b21`, as read: the variants and the version 1 of every expression.
fn b21_forms_read(cx: &bedrock_addon_core::load::LoadContext) {
    let cases = [
        (r#"[0, "v.sea_level"]"#, ["Const", "Expr"]),
        ("[false, 64]", ["Bool", "Const"]),
        (
            r#"["q.heightmap(v.worldx, v.worldz) - 4", 320]"#,
            ["Expr", "Const"],
        ),
    ];
    for (range, kinds) in cases {
        let document = super::common::loads(&height_range(range), cx);
        let values = range_of(document.value());
        for (value, kind) in values.iter().zip(kinds) {
            let found = match value {
                Molang::Const(_) => "Const",
                Molang::Bool(_) => "Bool",
                Molang::Expr(_) => "Expr",
                _ => "other",
            };
            assert_eq!(found, kind, "{range}");
            if kind == "Expr" {
                assert_eq!(version(value), Some(RawVersion(1)), "{range}: version 1");
            }
        }
        assert!(
            with(document.diagnostics(), |c| matches!(
                c.id(),
                "core/invalid_molang" | "core/molang_lint"
            ))
            .is_empty(),
            "{range}: {:#?}",
            document.diagnostics()
        );
    }
}

/// `b22_height_range_object_form`: the object form is no height-range value in either build.
#[test]
fn b22_height_range_object_form() {
    let found = rejected(
        &height_range(r#"[{"expression": "1", "version": 13}, 64]"#),
        &context(),
    );
    let invalid = with(&found, |c| matches!(c.id(), "core/invalid_type"));
    assert_eq!(invalid.len(), 1, "{found:#?}");
    assert_eq!(invalid[0].severity, Severity::Error);
    assert_eq!(
        invalid[0].location.json_path.to_string(),
        "minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/height_range/0"
    );
}

/// Without the `molang` feature (or a check in the context) nothing is compiled: one
/// `MolangNotChecked` info, and the biome loads.
#[test]
fn b21_height_range_forms_unchecked() {
    b21_forms_read(&context());
    let document = super::common::loads(&height_range(r#"["1 +", 64]"#), &context());
    let unchecked = with(document.diagnostics(), |c| {
        c.args_of(&bedrock_addon_core::diagnostic::codes::MOLANG_NOT_CHECKED)
            .is_some_and(|a| a[0] == "1")
    });
    assert_eq!(unchecked.len(), 1, "{:#?}", document.diagnostics());
    assert_eq!(unchecked[0].severity, Severity::Info);
    assert!(errors(document.diagnostics()).is_empty());
}

/// Constant bounds are checked in both builds.
#[test]
fn constant_height_range_bounds_are_checked_without_compiling() {
    let found = rejected(&height_range("[64, 64]"), &context());
    assert_eq!(
        errors(&found)[0].code.to_string(),
        "Incorrect values for height_range bounds - upper bound (second value) should be strictly greater than lower bound (first value)"
    );
}

#[cfg(feature = "molang")]
mod compiled {
    use bedrock_addon_core::diagnostic::{DiagnosticDetail, Diagnostics, Severity};
    use bedrock_addon_core::molang::DiagCode;

    use super::super::common::{checked, context, errors, loads, rejected, with};
    use super::{b21_forms_read, height_range};

    /// The compiler codes of the `InvalidMolang` / `MolangLint` findings, with their severity.
    fn molang(found: &Diagnostics) -> Vec<(DiagCode, Severity)> {
        found
            .iter()
            .filter_map(|d| match d.detail.as_deref() {
                Some(DiagnosticDetail::Molang(m)) => Some((m.code(), d.severity)),
                _ => None,
            })
            .collect()
    }

    /// The glue's error is the last error: vanilla's text with the source.
    #[track_caller]
    fn glue_last(found: &Diagnostics, source: &str) {
        let errors = errors(found);
        let last = errors.last().expect("an error");
        assert!(matches!(last.code.id(), "biome/expression"));
        assert_eq!(
            last.code.to_string(),
            format!("Expression Text: '{source}', failed to parse.")
        );
        assert_eq!(
            last.location.json_path.to_string(),
            "minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/height_range/0"
        );
    }

    /// `b21_height_range_forms`: every form compiles at version 1 with the `world_gen`
    /// queries; no finding; the biome loads.
    #[test]
    fn b21_height_range_forms() {
        b21_forms_read(&checked(context()));
    }

    /// `b23_height_range_bad_expression`: a parse error, then the glue's text; not loaded.
    #[test]
    fn b23_height_range_bad_expression() {
        let found = rejected(&height_range(r#"["1 +", 64]"#), &checked(context()));
        let codes = molang(&found);
        assert!(
            codes.contains(&(DiagCode::StatementForm, Severity::Error)),
            "{codes:?}"
        );
        glue_last(&found, "1 +");
    }

    /// `b24_height_range_default_query`: a `default`-set query is unknown in `world_gen`.
    #[test]
    fn b24_height_range_default_query() {
        let found = rejected(&height_range(r#"["q.is_baby", 64]"#), &checked(context()));
        let codes: Vec<DiagCode> = molang(&found).into_iter().map(|(c, _)| c).collect();
        assert!(codes.contains(&DiagCode::UnknownQuery), "{codes:?}");
        assert!(codes.contains(&DiagCode::Syntax), "{codes:?}");
        glue_last(&found, "q.is_baby");
    }

    /// `b25_height_range_empty_string`: at version 1 an empty text is rejected with only an
    /// info note; the glue's error rejects the biome.
    #[test]
    fn b25_height_range_empty_string() {
        let found = rejected(&height_range(r#"["", 64]"#), &checked(context()));
        let codes = molang(&found);
        assert!(
            codes.contains(&(DiagCode::Syntax, Severity::Info)),
            "{codes:?}: {found:#?}"
        );
        assert_eq!(errors(&found).len(), 1, "{found:#?}");
        glue_last(&found, "");
    }

    /// `b26_height_range_version_1_rules`: version 1 has no string-arithmetic error, so the
    /// biome loads; the same text in a version-13 field is `StringMisuse`.
    #[test]
    fn b26_height_range_version_1_rules() {
        use bedrock_addon_core::molang::{
            CompileOptions, MolangSource, MolangVersion, QueryAdmission, QuerySetMask,
            compile_source,
        };
        let text = "'a' + 1";
        let document = loads(
            &height_range(&format!(r#"["{text}", 64]"#)),
            &checked(context()),
        );
        assert!(
            with(document.diagnostics(), |c| matches!(
                c.id(),
                "core/invalid_molang"
            ))
            .is_empty(),
            "{:#?}",
            document.diagnostics()
        );
        // The field's options (server catalogue, `world_gen`) at each version.
        let options = CompileOptions::server(MolangVersion::LATEST)
            .with_admission(QueryAdmission::Sets(QuerySetMask::WORLD_GEN));
        let codes = |version: MolangVersion| -> Vec<DiagCode> {
            compile_source(&MolangSource::string(text, version), &options)
                .diagnostics()
                .iter()
                .map(bedrock_addon_core::molang::MolangDiagnostic::code)
                .collect()
        };
        assert!(!codes(MolangVersion::V1).contains(&DiagCode::StringMisuse));
        assert!(
            codes(MolangVersion::V13).contains(&DiagCode::StringMisuse),
            "{:?}",
            codes(MolangVersion::V13)
        );
    }

    /// An expression the server cannot evaluate (a render-controller array, a resource) parses,
    /// but gets the glue's text all the same and the biome is not loaded.
    #[test]
    fn an_array_or_resource_height_range_is_the_glue_error() {
        for source in ["array.skins[0]", "geometry.default"] {
            let found = rejected(
                &height_range(&format!(r#"["{source}", 64]"#)),
                &checked(context()),
            );
            assert_eq!(errors(&found).len(), 1, "{source}: {found:#?}");
            glue_last(&found, source);
        }
    }

    /// A folded constant expression takes part in the bounds check (rule 9a "Bounds check").
    #[test]
    fn a_folded_height_range_is_bounds_checked() {
        let found = rejected(&height_range(r#"["64", 32]"#), &checked(context()));
        assert_eq!(
            errors(&found)[0].code.to_string(),
            "Incorrect values for height_range bounds - upper bound (second value) should be strictly greater than lower bound (first value)"
        );
    }
}
