//! The engine's own feature-rule inputs of 1.26.60 (main-1, case-11 and the template case-18, filed with the features; copied
//! under `fixtures/worldgen/feature_rules/engine/`). The rules are unchanged in
//! 1.26.60; each input loads without a warning or an error and round-trips
//! (value equality, a stable second write).

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::Content;
use bedrock_addon_core::load::LoadContext;

use super::common::{fixture, load};

/// Loads, has no warning or error, writes, reads back equal, writes the same text again.
#[track_caller]
fn round_trips(name: &str, text: &str) {
    let cx = LoadContext::detached();
    let first = load(text, &cx).unwrap_or_else(|r| panic!("{name}: {:#?}", r.diagnostics));
    assert!(
        first
            .diagnostics()
            .iter()
            .all(|d| d.severity < Severity::Warning),
        "{name}: {:#?}",
        first.diagnostics()
    );
    let written = first.to_json().unwrap();
    let second = load(&written, &cx).unwrap_or_else(|r| panic!("{written}: {:#?}", r.diagnostics));
    assert_eq!(first.value(), second.value(), "{name}: {written}");
    assert_eq!(second.to_json().unwrap(), written, "{name}");
}

/// The two 1.13.0 rules with a `distribution` whose `iterations` is a text (an expression):
/// main-1 (a static initialiser's string) and case-11 (three distribution kinds).
#[test]
fn the_engine_s_feature_rules_load_and_round_trip() {
    for name in ["main-1.json", "case-11.json"] {
        round_trips(name, &fixture(&format!("engine/{name}")));
    }
}

/// case-18 is a template (`{}` = the format version) of a rule without a
/// `distribution`; instantiated at the first format and at 1.21.20 it loads and round-trips.
#[test]
fn the_engine_s_rule_template_without_distribution_loads() {
    let template = fixture("engine/case-18.fmt.txt");
    for version in ["1.13.0", "1.21.20"] {
        let text = template
            .replacen("{}", version, 1)
            .replace("{{", "{")
            .replace("}}", "}");
        round_trips(version, &text);
    }
}
