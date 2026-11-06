//! The three pack-setting queries in
//! behaviour-pack Molang are one warning per expression. The finding is the compile check's
//! (core, over molang's `QueryClientOnly` info).

use bedrock_addon::diagnostic::{DiagnosticDetail, Diagnostics, JsonPath, Location, Severity};
use bedrock_addon::molang::{
    DiagCode, ExperimentMask, MolangCheck, MolangContext, MolangFieldSpec, MolangOutcome,
    MolangSource, Side,
};
use bedrock_addon::version::{FormatVersionInput, MinEngineVersion};

/// The three queries, each called as vanilla's unit test calls it.
const QUERIES: [&str; 3] = [
    "query.get_pack_setting('ns:some_slider')",
    "query.is_pack_setting_enabled('ns:some_toggle')",
    "query.is_pack_setting_selected('ns:some_dropdown', 'banana')",
];

/// The outcome and the lints (molang code, severity) of `text` compiled for `side` in a pack
/// at 1.21.130, the engine that introduced the queries.
fn lints(text: &str, side: Side) -> (MolangOutcome, Vec<(DiagCode, Severity)>, usize) {
    let cx = MolangContext::new(
        Some(&MinEngineVersion::new(1, 21, 130)),
        ExperimentMask::empty(),
        &FormatVersionInput::parse("1.21.130").unwrap(),
        None,
    );
    let mut diagnostics = Diagnostics::default();
    let outcome = MolangCheck::new().check_source(
        &MolangSource::string(text, cx.pack),
        &MolangFieldSpec::DEFAULT,
        side,
        &cx,
        &Location::new(None, None, JsonPath::new(), None),
        &mut diagnostics,
    );
    let found = diagnostics
        .iter()
        // By legacy name: the variant and the definitions that were it.
        .filter(|d| matches!(d.code.name(), "MolangLint"))
        .filter_map(|d| match d.detail.as_deref() {
            Some(DiagnosticDetail::Molang(m)) => Some((m.code(), d.severity)),
            _ => None,
        })
        .collect();
    (outcome, found, diagnostics.len())
}

#[test]
fn each_pack_setting_query_in_a_behavior_pack_is_one_client_only_warning() {
    for query in QUERIES {
        let (outcome, found, total) = lints(query, Side::Server);
        assert_eq!(outcome, MolangOutcome::Ok, "{query}");
        // An information of ours (molang's `QueryClientOnly` is one too).
        assert_eq!(
            found,
            [(DiagCode::QueryClientOnly, Severity::Info)],
            "{query}"
        );
        assert_eq!(total, 1, "{query}: one finding");
    }
}

#[test]
fn pack_setting_queries_in_one_expression_are_one_warning() {
    // "Once per expression", whichever of the three queries and however many.
    for text in [
        format!("{} + {}", QUERIES[0], QUERIES[1]),
        format!("{0} + {0}", QUERIES[2]),
        format!("{} ? {} : {}", QUERIES[1], QUERIES[0], QUERIES[2]),
    ] {
        let (outcome, found, total) = lints(&text, Side::Server);
        assert_eq!(outcome, MolangOutcome::Ok, "{text}");
        assert_eq!(
            found,
            [(DiagCode::QueryClientOnly, Severity::Info)],
            "{text}"
        );
        assert_eq!(total, 1, "{text}");
    }
}

#[test]
fn another_client_only_query_beside_a_pack_setting_query_keeps_its_own_info() {
    // The override is by name: `query.anim_time` is client-only too and stays an info per use.
    let text = format!("{} + query.anim_time + query.anim_time", QUERIES[0]);
    let (outcome, found, total) = lints(&text, Side::Server);
    assert_eq!(outcome, MolangOutcome::Ok);
    assert_eq!(
        found,
        [
            (DiagCode::QueryClientOnly, Severity::Info),
            (DiagCode::QueryClientOnly, Severity::Info),
            (DiagCode::QueryClientOnly, Severity::Info)
        ]
    );
    assert_eq!(total, 3);
}

#[test]
fn the_pack_setting_queries_in_a_resource_pack_are_silent() {
    for query in QUERIES {
        let (outcome, found, total) = lints(query, Side::Client);
        assert_eq!(outcome, MolangOutcome::Ok, "{query}");
        assert!(found.is_empty(), "{query}: {found:?}");
        assert_eq!(total, 0, "{query}");
    }
}
