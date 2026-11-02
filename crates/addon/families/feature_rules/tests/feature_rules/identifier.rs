//! The identifier's namespace syntax and file name
//! are checked against the file, logged and the rule kept.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::load::LoadContext;

use super::common::*;

/// Loads the r05 fixture `name` at its own path.
fn r05(
    name: &'static str,
    path: &'static str,
) -> bedrock_addon_core::document::Document<bedrock_addon_family_feature_rules::FeatureRule> {
    let text = fixture(&format!("r05_identifier_checks/feature_rules/{name}"));
    load_at(path, &text, &LoadContext::detached())
        .unwrap_or_else(|rejected| panic!("{name}: {:#?}", rejected.diagnostics))
}

#[test]
fn r05_an_identifier_without_a_namespace_is_logged_and_the_rule_loads() {
    let document = r05("nonamespace.json", "feature_rules/nonamespace.json");
    let found = with_code(document.diagnostics(), "InvalidValue");
    assert_eq!(found.len(), 1, "{:#?}", document.diagnostics());
    // the game's content-log error, the rule kept: a kept error
    assert_eq!((found[0].severity, found[0].kept), (Severity::Error, true));
    assert_eq!(
        found[0].code.to_string(),
        "Feature rule identifier 'other' does not use the appropriate namespace syntax"
    );
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:feature_rules/description/identifier"
    );
}

#[test]
fn r05_an_identifier_that_does_not_match_the_file_name_is_logged_and_the_rule_loads() {
    let document = r05("a.json", "feature_rules/a.json");
    let found = with_code(document.diagnostics(), "InvalidValue");
    assert_eq!(found.len(), 1, "{:#?}", document.diagnostics());
    // the game's content-log error, the rule kept: a kept error
    assert_eq!((found[0].severity, found[0].kept), (Severity::Error, true));
    assert_eq!(
        found[0].code.to_string(),
        "Feature rule identifier 'b' does not match filename 'a'"
    );
}

#[test]
fn r05_an_identifier_equal_to_the_file_stem_needs_no_namespace() {
    let document = r05("plain.json", "feature_rules/plain.json");
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
}

#[test]
fn rule_2_an_empty_namespace_fails_the_syntax_check() {
    let text = rule("1.13.0", "surface_pass", None).replace("demo:test", ":test");
    let document = load_at("feature_rules/test.json", &text, &LoadContext::detached()).unwrap();
    assert!(has(document.diagnostics(), |c| c.to_string()
        == "Feature rule identifier ':test' does not use the appropriate namespace syntax"));
}

#[test]
fn rule_3_the_name_after_the_last_colon_is_compared_case_sensitively() {
    let text = rule("1.13.0", "surface_pass", None).replace("demo:test", "a:b:Test");
    let document = load_at("feature_rules/test.json", &text, &LoadContext::detached()).unwrap();
    assert!(has(document.diagnostics(), |c| c.to_string()
        == "Feature rule identifier 'Test' does not match filename 'test'"));
    let text = rule("1.13.0", "surface_pass", None).replace("demo:test", "a:b:test");
    let document = load_at(
        "feature_rules/sub/test.json",
        &text,
        &LoadContext::detached(),
    )
    .unwrap();
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
}

#[test]
fn rule_11_a_minecraft_identifier_outside_the_base_game_is_not_rejected() {
    let text = rule("1.13.0", "surface_pass", None).replace("demo:test", "minecraft:test");
    let document = load_at("feature_rules/test.json", &text, &LoadContext::detached()).unwrap();
    assert!(!has(document.diagnostics(), |c| matches!(
        c,
        DiagnosticCode::InvalidValue { .. }
    )));
}

// An object given to an identifier is a wrong type at the member, and reading it does not panic.
#[test]
fn an_identifier_that_is_an_object_is_a_type_error_and_no_panic() {
    let json = rule("1.26.50", "first_pass", None).replace(r#""demo:test""#, "{}");
    assert!(json.contains(r#""identifier": {}"#));
    let d = rejects(&json);
    let found = with_code(&d, "InvalidType");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:feature_rules/description/identifier"
    );
}
