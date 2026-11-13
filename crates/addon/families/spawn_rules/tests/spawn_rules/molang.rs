//! Block descriptor `tags` in legacy conditions:
//! the fixture rows `s04`–`s04d`. The Molang version is the document's
//! `format_version`, mapped; only the tag queries resolve; a rejection is a warning and the
//! descriptor stays.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::molang::{MolangVersion, RawVersion};
use bedrock_addon_shared_shapes::BlockDescriptor;

use super::common::{context, document, loads, standard};

/// A document of `version` whose one condition has `spawns_on_block_filter: {"tags": tags}`.
fn tagged(version: &str, tags: &str) -> String {
    document(
        version,
        &format!(
            r#"{{"description": {{"identifier": "x:mob", "population_control": "animal"}}, "conditions": [{{"minecraft:spawns_on_block_filter": {{"tags": "{tags}"}}}}]}}"#
        ),
    )
}

/// The version the descriptor's expression carries after loading.
fn version_of(text: &str, cx: &bedrock_addon_core::load::LoadContext) -> Option<RawVersion> {
    let document = loads(text, cx);
    let condition = &standard(&document).conditions.as_ref().unwrap()[0];
    let list = condition.spawns_on_block_filter.as_ref().unwrap();
    match &list.as_slice()[0] {
        BlockDescriptor::Tags { tags, .. } => tags.raw_version(),
        other => panic!("not tags: {other:?}"),
    }
}

#[test]
fn s04_molang_version_from_format() {
    for (version, molang) in [
        ("1.8.0", MolangVersion::V0),
        ("1.17.40", MolangVersion::V4),
        ("1.20.70", MolangVersion::V12),
        ("1.21.100", MolangVersion::V13),
    ] {
        assert_eq!(
            version_of(&tagged(version, "q.any_tag('dirt')"), &context()),
            Some(RawVersion::from(molang)),
            "{version}"
        );
    }
}

#[cfg(not(feature = "molang"))]
#[test]
fn s04_without_the_molang_feature_the_tags_are_not_checked() {
    let document = loads(&tagged("1.8.0", "q.any_tag('dirt')"), &context());
    let found: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|d| {
            d.code
                .args_of(&bedrock_addon_core::diagnostic::codes::MOLANG_NOT_CHECKED)
                .is_some_and(|a| a[0] == "1")
        })
        .collect();
    assert_eq!(found.len(), 1, "{:#?}", document.diagnostics());
    assert_eq!(found[0].severity, Severity::Info);
}

#[cfg(feature = "molang")]
mod compiled {
    use std::sync::Arc;

    use bedrock_addon_core::diagnostic::{DiagnosticDetail, Diagnostics};
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::molang::{DiagCode, MolangCheck};

    use super::*;

    /// A context that compiles Molang.
    fn checking() -> LoadContext {
        let mut cx = context();
        cx.molang = Some(Arc::new(MolangCheck::new()));
        cx
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

    /// The `InvalidMolang` findings at warning or above.
    fn rejections(d: &Diagnostics) -> usize {
        d.iter()
            .filter(|f| {
                matches!(f.code.id(), "core/invalid_molang") && f.severity >= Severity::Warning
            })
            .count()
    }

    #[test]
    fn s04a_tags_compile_forms() {
        for tags in ["q.any_tag('dirt')", "q.all_tags('dirt', 'grass')"] {
            let text = tagged("1.8.0", tags);
            let document = loads(&text, &checking());
            assert!(
                document.diagnostics().is_empty(),
                "{tags}: {:#?}",
                document.diagnostics()
            );
            assert_eq!(
                version_of(&text, &checking()),
                Some(RawVersion::from(MolangVersion::V0))
            );
        }
    }

    /// A rejected `tags` behind a condition and a descriptor the reader leaves out is at
    /// the index the file has it at.
    #[test]
    fn s04b_rejected_tags_behind_an_element_left_out_are_at_the_index_of_the_file() {
        let text = document(
            "1.8.0",
            r#"{"description": {"identifier": "x:mob", "population_control": "animal"}, "conditions": [7, {
                "minecraft:spawns_on_block_filter": [7, {"tags": "q.is_baby"}],
                "minecraft:spawns_above_block_filter": {"blocks": [{"tags": 3}, 7, {"tags": "q.is_baby"}]}}]}"#,
        );
        let loaded = loads(&text, &checking());
        let mut at: Vec<String> = loaded
            .diagnostics()
            .iter()
            .filter(|f| matches!(f.code.id(), "core/invalid_molang"))
            .map(|f| f.location.json_path.to_string())
            .collect();
        at.dedup();
        let c = "minecraft:spawn_rules/conditions/1/minecraft:";
        assert_eq!(
            at,
            [
                format!("{c}spawns_on_block_filter/1/tags"),
                format!("{c}spawns_above_block_filter/blocks/2/tags"),
            ],
            "{:#?}",
            loaded.diagnostics()
        );
    }

    #[test]
    fn s04b_tags_default_query() {
        let document = loads(
            &tagged("1.8.0", "q.any_tag('dirt') && q.is_baby"),
            &checking(),
        );
        let d = document.diagnostics();
        let found = molang(d);
        assert!(
            found.contains(&(DiagCode::UnknownQuery, Severity::Error)),
            "{found:?}"
        );
        assert!(
            found.contains(&(DiagCode::Syntax, Severity::Error)),
            "{found:?}"
        );
        // The parser's own text, in the content-log line (`<expression> | <message>`).
        assert!(d.iter().any(|f| format!("{f}").contains(
            "Failed to resolve query query.is_baby.  Either the query does not exist or it is not supported in this context."
        )), "{d:#?}");
        assert!(!d.has_errors());
        // The descriptor stays.
        let condition = &standard(&document).conditions.as_ref().unwrap()[0];
        assert_eq!(condition.to_spawn_rules().spawns_on_blocks.len(), 1);
    }

    #[test]
    fn s04c_tags_empty() {
        // Molang 0: the compiler's note is an info, the rejection a warning of ours.
        let low = loads(&tagged("1.8.0", ""), &checking());
        assert!(!low.diagnostics().has_errors());
        assert!(
            molang(low.diagnostics())
                .iter()
                .all(|(c, _)| *c == DiagCode::Syntax),
            "{:#?}",
            low.diagnostics()
        );
        // Below Molang 4 the empty text has no parser line: the rejection is an information of
        // ours.
        assert_eq!(rejections(low.diagnostics()), 0, "{:#?}", low.diagnostics());
        assert!(
            low.diagnostics()
                .iter()
                .any(|f| matches!(f.code.id(), "core/invalid_molang")
                    && f.severity == Severity::Info),
            "{:#?}",
            low.diagnostics()
        );
        // Molang 13: "No tokens found in expression", a warning.
        let high = loads(&tagged("1.21.100", ""), &checking());
        assert!(!high.diagnostics().has_errors());
        assert!(
            molang(high.diagnostics()).contains(&(DiagCode::Syntax, Severity::Error)),
            "{:#?}",
            high.diagnostics()
        );
        assert!(
            high.diagnostics()
                .iter()
                .any(|f| format!("{f}").contains("No tokens found in expression"))
        );
    }

    #[test]
    fn s04d_tags_version_from_partial_format() {
        let partial = tagged("1.20", "'a' + 1 == 1");
        assert_eq!(
            version_of(&partial, &checking()),
            Some(RawVersion::from(MolangVersion::V8))
        );
        let document = loads(&partial, &checking());
        assert!(
            molang(document.diagnostics())
                .iter()
                .any(|(c, s)| *c == DiagCode::StringMisuse && *s == Severity::Error),
            "{:#?}",
            document.diagnostics()
        );
        let old = loads(&tagged("1.16", "'a' + 1 == 1"), &checking());
        assert!(old.diagnostics().is_empty(), "{:#?}", old.diagnostics());
    }
}
