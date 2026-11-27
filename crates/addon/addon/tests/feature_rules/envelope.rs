//! The envelope, and the deliberate deviations from vanilla
//! (each pinned by a test whose name says `deviation`).

use bedrock_addon::diagnostic::{Kind, Severity};
use bedrock_addon::version::FormatVersion;

use super::common::*;

/// `_parseAndInsertUnsorted` passes a zeroed `SemVersion` as the caller's version to
/// `loadWithArea`: no key and no caller version is 0x8 "no valid
/// version passed by the caller or found in the input", the rule skipped.
#[test]
fn rule_1_a_missing_format_version_skips_the_file() {
    let text = vanilla_input("without_distribution", "1.13.0")
        .replace(r#""format_version": "1.13.0","#, "");
    let d = rejects(&text);
    let found = with_code(&d, "core/no_valid_version");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(found[0].severity, Severity::Error);
    assert!(has_text(
        &d,
        "no valid version passed by the caller or found in the input"
    ));
    // The member is absent: the finding is at the document root.
    assert_eq!(found[0].location.json_path.to_string(), "", "{d:#?}");
}

#[test]
fn rule_1_a_malformed_format_version_skips_the_file() {
    for version in [r#""abc""#, "true", "{}", "[1, 21]"] {
        let text = vanilla_input("without_distribution", "X").replace(r#""X""#, version);
        let d = rejects(&text);
        assert_eq!(
            with_code(&d, "core/malformed_format_version").len(),
            1,
            "{version}: {d:#?}"
        );
    }
}

/// `MinecraftDocumentInput::initAsJson` reads a number as its text: `1.21` is
/// 1.21.0.
#[test]
fn rule_1_a_number_is_read_as_its_text() {
    let text = vanilla_input("without_distribution", "X").replace(r#""X""#, "1.21");
    let document = loads(&text);
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 21, 0))
    );
}

#[test]
fn rule_1_three_integers_are_a_version() {
    let text = vanilla_input("without_distribution", "X").replace(r#""X""#, "[1, 21, 20]");
    let document = loads(&text);
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 21, 20))
    );
}

#[test]
fn rule_1_a_third_root_member_skips_the_file() {
    let text = vanilla_input("without_distribution", "1.13.0").replacen('{', r#"{"extra": 1, "#, 1);
    let d = rejects(&text);
    assert!(
        d.iter().any(|f| f.code.to_string()
            == "A Minecraft Document must have exactly two members: \"format_version\" and a payload member"),
        "{d:#?}"
    );
}

#[test]
fn rule_1_beta_is_unsupported() {
    let d = rejects(&vanilla_input("without_distribution", "beta"));
    assert!(
        !with_code(&d, "core/no_parser_for_version").is_empty()
            || d.iter().any(|f| f.kind() == Kind::Toggle),
        "{d:#?}"
    );
}

#[test]
fn a_duplicate_rule_is_reported_as_shadowed_deviation() {
    // Vanilla drops it silently (TOML `vanilla = "silently skipped"`); ours: `ShadowedDefinition`
    // info, tested in `stack.rs` (r13, r18, rule 10). This pins the severity.
    use bedrock_addon::load::LoadOptions;
    use bedrock_addon::pack::{Behavior, PackStack};
    let text: &'static [u8] = vanilla_input("without_distribution", "1.13.0")
        .leak()
        .as_bytes();
    let top = super::pack_common::bp(1)
        .file("feature_rules/custom_feature_rule.json", text)
        .open();
    let bottom = super::pack_common::bp(2)
        .file("feature_rules/custom_feature_rule.json", text)
        .open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    let shadowed: Vec<_> = stack
        .feature_rules()
        .diagnostics()
        .filter(|f| f.code.id() == "core/shadowed_definition")
        .collect();
    assert_eq!(shadowed.len(), 1);
    assert_eq!(shadowed[0].severity, Severity::Info);
}

#[test]
fn a_logged_and_kept_check_is_a_kept_error() {
    // Vanilla logs these at error level and keeps the rule: kept errors (the
    // document loads).
    let document = loads(&rule(
        "1.13.0",
        "nowhere_pass",
        Some(&distribution("-1", "0", "0", "0")),
    ));
    let found: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|f| {
            matches!(
                f.code.id(),
                "feature_rules/unknown_pass" | "feature_rules/iterations_negative"
            )
        })
        .collect();
    assert_eq!(found.len(), 2);
    assert!(
        found
            .iter()
            .all(|f| f.severity == Severity::Error && f.kept)
    );
}
