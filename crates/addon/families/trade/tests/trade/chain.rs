//! The chain `v0 → v1_21_30`: contiguity, the range model's selection, the floor,
//! the ceiling and the beta literal, and the envelope of each side.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    EnvelopeCheck, Family, MissingVersion, RangeModel, Selection, VersionChain, check_chain, select,
};
use bedrock_addon_family_trade::history::{Any, TradeFamily};

use super::common::{above_info, doc, loads, rejects, with};

/// One tier a document of either side accepts.
const TIERS: &str = r#"[{"total_exp_required": 0, "groups": [{"num_to_select": 1, "trades": [
    {"wants": [{"item": "minecraft:wheat"}], "gives": [{"item": "minecraft:emerald"}]}
]}]}]"#;

/// What `version` selects.
fn selects(version: &str) -> Selection {
    select::<TradeFamily>(&FormatVersionInput::parse(version).expect("a version"))
}

#[test]
fn the_chain_is_v0_then_v1_21_30_with_no_beta() {
    check_chain::<TradeFamily>().expect("the chain is well formed");
    assert_eq!(
        <Any as VersionChain<TradeFamily>>::VERSIONS,
        [FormatVersion::new(0, 0, 0), FormatVersion::new(1, 21, 30)]
    );
    const HAS_BETA: bool = <Any as VersionChain<TradeFamily>>::HAS_BETA;
    let has_beta = HAS_BETA;
    assert!(!has_beta, "Beta = Infallible");
}

#[test]
fn the_family_parameters_are_the_plans() {
    assert_eq!(TradeFamily::FLOOR, FormatVersion::new(0, 0, 0));
    assert_eq!(TradeFamily::RANGE_MODEL, RangeModel::ClosedToEngine);
    assert_eq!(TradeFamily::PAYLOAD_KEY, "tiers");
    assert_eq!(TradeFamily::DIRECTORIES, ["trading"]);
    const WARN: bool = TradeFamily::ENVELOPE_WARN_MISSING_VERSION;
    let warn = WARN;
    assert!(!warn, "a missing version is 0.0.0 without a message");
    assert_eq!(TradeFamily::MISSING_VERSION, MissingVersion::Floor);
    assert_eq!(
        TradeFamily::ENVELOPE,
        EnvelopeCheck::From(FormatVersion::new(1, 21, 30))
    );
}

#[test]
fn each_version_selects_the_struct_whose_range_holds_it() {
    assert_eq!(selects("0.0.0"), Selection::Struct(0));
    assert_eq!(selects("1.8.0"), Selection::Struct(0));
    assert_eq!(selects("1.21.20"), Selection::Struct(0));
    // A pre-release sorts below its release.
    assert_eq!(selects("1.21.30-beta.1"), Selection::Struct(0));
    assert_eq!(selects("1.21.30"), Selection::Struct(1));
    // Vanilla's 1.26.10 stamp is a version of the newest struct's range, as every one up to
    // the engine's.
    assert_eq!(selects("1.26.10"), Selection::Struct(1));
    assert_eq!(selects(&ENGINE_VERSION.to_string()), Selection::Struct(1));
    let above = FormatVersion::new(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 1,
    );
    assert_eq!(selects(&above.to_string()), Selection::Unsupported);
}

#[test]
fn the_selected_struct_parses_the_document() {
    for (version, parsed_as, upgraded) in [
        ("0.0.0", (0, 0, 0), true),
        ("1.18.10", (0, 0, 0), true),
        ("1.21.20", (0, 0, 0), true),
        ("1.21.30", (1, 21, 30), false),
        ("1.26.10", (1, 21, 30), false),
    ] {
        let document = loads(&doc(Some(version), TIERS));
        assert!(above_info(document.diagnostics()).is_empty(), "{version}");
        let info = document.version();
        let (a, b, c) = parsed_as;
        assert_eq!(info.parsed_as, FormatVersion::new(a, b, c), "{version}");
        assert_eq!(info.upgraded, upgraded, "{version}");
        assert_eq!(
            info.written,
            FormatVersionInput::parse(version).unwrap().version
        );
        assert_eq!(info.effective, info.written.unwrap());
    }
    let engine = loads(&doc(Some(&ENGINE_VERSION.to_string()), TIERS));
    assert_eq!(engine.version().parsed_as, FormatVersion::new(1, 21, 30));
}

#[test]
fn a_missing_version_is_0_0_0_without_a_message() {
    let document = loads(&doc(None, TIERS));
    assert!(document.diagnostics().is_empty());
    let info = document.version();
    assert_eq!(info.written, None);
    assert_eq!(info.effective, FormatVersion::new(0, 0, 0));
    assert_eq!(info.parsed_as, FormatVersion::new(0, 0, 0));
}

#[test]
fn a_version_above_the_engine_is_unsupported() {
    let above = FormatVersion::new(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 1,
    );
    let d = rejects(&doc(Some(&above.to_string()), TIERS));
    let found = with(&d, "UnsupportedVersion", Severity::Error);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].location.json_path.to_string(), "format_version");
    assert_eq!(above_info(&d).len(), 1);
}

#[test]
fn a_prerelease_of_1_21_30_is_read_by_the_legacy_parser_with_an_info() {
    let document = loads(&doc(Some("1.21.30-beta.1"), TIERS));
    assert_eq!(document.version().parsed_as, FormatVersion::new(0, 0, 0));
    assert!(document.diagnostics().iter().any(|f| matches!(
        f.code,
        DiagnosticCode::PrereleaseFormatVersion { .. }
    ) && f.severity == Severity::Info));
}

#[test]
fn only_a_prerelease_of_1_21_30_itself_changes_sides() {
    // A pre-release of a later version is still at or above 1.21.30: the strict slice.
    let later = loads(&doc(Some("1.26.10-beta.1"), TIERS));
    assert_eq!(later.version().parsed_as, FormatVersion::new(1, 21, 30));
    // One of an earlier version is the legacy parser's anyway.
    let earlier = loads(&doc(Some("1.21.20-beta.1"), TIERS));
    assert_eq!(earlier.version().parsed_as, FormatVersion::new(0, 0, 0));
    assert_eq!(earlier.version().effective, FormatVersion::new(1, 21, 20));
    // The release itself is strict: a flat tier is an unknown member there.
    let flat = r#"[{"trades": []}]"#;
    assert!(rejects(&doc(Some("1.21.30"), flat)).has_errors());
    let prerelease = loads(&doc(Some("1.21.30-beta.1"), flat));
    assert_eq!(
        prerelease.version().written,
        Some(FormatVersion::new(1, 21, 30))
    );
    assert_eq!(
        prerelease.version().effective,
        FormatVersion::new(1, 21, 30)
    );
    assert!(prerelease.version().upgraded);
}

#[test]
fn the_beta_literal_has_no_struct() {
    // No beta struct: no parser for `beta`, with or without the Beta APIs.
    let text = doc(Some("beta"), TIERS);
    let d = rejects(&text);
    assert_eq!(with(&d, "UnsupportedVersion", Severity::Error).len(), 1);
    let mut cx = super::common::cx();
    cx.beta_apis = true;
    let rejected = {
        use bedrock_addon_core::document::Content;
        bedrock_addon_family_trade::TradeTable::parse(
            &bedrock_addon_core::pack_identity::PackPath::from_static("trading/test_trades.json"),
            text.as_bytes(),
            &cx,
        )
        .unwrap_err()
    };
    assert_eq!(
        with(&rejected.diagnostics, "UnsupportedVersion", Severity::Error).len(),
        1
    );
}

#[test]
fn a_version_string_that_is_no_version_is_malformed() {
    let d = rejects(&doc(Some("latest"), TIERS));
    assert_eq!(with(&d, "MalformedFormatVersion", Severity::Error).len(), 1);
}

#[test]
fn an_array_version_is_picked_by_the_document_input() {
    // `initAsJson` reads an array of three integers as the version:
    // below 1.21.30 it selects the legacy slice, whose parser reads its own version only from
    // a string (0.0.0, said at the member); from 1.21.30 the cereal slice reads it
    // (`input.rs`); above the engine no slice does.
    let text = format!(r#"{{"format_version": [1, 18, 10], "tiers": {TIERS}}}"#);
    let document = loads(&text);
    let told: Vec<_> = document
        .diagnostics()
        .iter()
        .map(|f| (f.code.name(), f.severity, f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        told,
        [("InvalidType", Severity::Info, "format_version".to_owned())]
    );
    let info = document.version();
    assert_eq!(info.written, None);
    assert_eq!(info.effective, FormatVersion::new(0, 0, 0));
    assert_eq!(info.parsed_as, FormatVersion::new(0, 0, 0));
    let text = format!(r#"{{"format_version": [1, 99, 0], "tiers": {TIERS}}}"#);
    let d = rejects(&text);
    assert_eq!(with(&d, "UnsupportedVersion", Severity::Error).len(), 1);
}
