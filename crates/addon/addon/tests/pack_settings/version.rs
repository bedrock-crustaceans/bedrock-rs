//! `families/pack_settings` fixtures.md "Document" rows about the version: `ps05`–`ps08b` and
//! `ps16` (validation.md D2–D4b).

use bedrock_addon::diagnostic::codes;
use bedrock_addon::diagnostic::{Code as C, Diagnostics, Severity};
use bedrock_addon::version::{ENGINE_VERSION, FormatVersion};

use super::common::{fixture, loads, names, parse_text, refused};
use super::document::closes;

/// "unsupported version: no parser available for version <version>".
fn no_parser(version: &str) -> C {
    codes::NO_PARSER_FOR_VERSION.with([version.to_owned()])
}

/// The errors of a rejected file: `first`, then the closing finding.
fn rejected_with(found: &Diagnostics, name: &str, first: &C) {
    let errors: Vec<&C> = found
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| &d.code)
        .collect();
    assert_eq!(errors.len(), 2, "{found:#?}");
    assert_eq!(errors[0], first);
    closes(found, name);
}

/// The fixture `name` with its `format_version` value's text replaced by `version`.
fn with_version(name: &str, from: &str, version: &str) -> String {
    let text = String::from_utf8(fixture(name)).unwrap();
    assert!(text.contains(from));
    text.replacen(from, version, 1)
}

#[test]
fn ps05_below_floor_has_no_parser() {
    let name = "ps05_below_floor.json";
    rejected_with(&refused(name), name, &no_parser("1.21.90"));
}

#[test]
fn ps06_above_engine_has_no_parser() {
    let name = "ps06_above_engine.json";
    assert!(FormatVersion::new(1, 27, 50) > ENGINE_VERSION);
    rejected_with(&refused(name), name, &no_parser("1.27.50"));
}

#[test]
pub fn ps06b_between_structs_loads_with_an_info_that_vanilla_rejects_it_deviation() {
    let name = "ps06b_between_structs.json";
    for version in ["1.21.110", "1.26.20"] {
        let text = with_version(name, "\"1.21.110\"", &format!("\"{version}\""));
        let loaded = parse_text(name, &text).unwrap_or_else(|r| panic!("{:#?}", r.diagnostics));
        assert_eq!(loaded.version().parsed_as, FormatVersion::new(1, 21, 100));
        assert_eq!(
            names(loaded.diagnostics(), Severity::Info),
            ["VanillaRejectsVersion"],
            "{version}"
        );
        assert!(names(loaded.diagnostics(), Severity::Warning).is_empty());
    }
    // Exactly the versions vanilla 1.26.20 reads, and our own newest, carry no info.
    for version in ["1.21.100", "1.26.40"] {
        let text = with_version(name, "\"1.21.110\"", &format!("\"{version}\""));
        let loaded = parse_text(name, &text).unwrap();
        assert!(loaded.diagnostics().is_empty(), "{version}");
    }
}

#[test]
fn ps06b_a_prerelease_of_the_floor_is_below_it() {
    let name = "ps06b_between_structs.json";
    let text = with_version(name, "\"1.21.110\"", "\"1.21.100-beta.1\"");
    let found = match parse_text(name, &text) {
        Ok(_) => panic!("loaded"),
        Err(rejected) => rejected.diagnostics,
    };
    rejected_with(&found, name, &no_parser("1.21.100-beta.1"));
}

#[test]
fn ps07_missing_version_has_no_valid_version() {
    let name = "ps07_missing_version.json";
    rejected_with(&refused(name), name, &C::from(&codes::NO_VALID_VERSION));
}

#[test]
pub fn ps08_array_version_loads_with_an_info() {
    let loaded = loads("ps08_array_version.json");
    assert_eq!(
        loaded.version().written,
        Some(FormatVersion::new(1, 21, 100))
    );
    assert_eq!(
        names(loaded.diagnostics(), Severity::Info),
        ["FormatVersionArray"]
    );
    assert_eq!(loaded.diagnostics().len(), 1);
}

#[test]
fn ps08b_malformed_version_is_malformed() {
    let name = "ps08b_malformed_version.json";
    let malformed = C::from(&codes::MALFORMED_FORMAT_VERSION);
    rejected_with(&refused(name), name, &malformed);
    assert_eq!(malformed.to_string(), "Malformed format version");
    let text = with_version(name, "\"1.x\"", "1.21");
    let found = match parse_text(name, &text) {
        Ok(_) => panic!("loaded"),
        Err(rejected) => rejected.diagnostics,
    };
    rejected_with(&found, name, &malformed);
}

#[test]
fn the_beta_literal_has_no_parser_whoever_asks() {
    // The family has no beta struct, and the file belongs to no pack that could be permitted.
    let name = "ps06b_between_structs.json";
    let text = with_version(name, "\"1.21.110\"", "\"beta\"");
    let found = match parse_text(name, &text) {
        Ok(_) => panic!("loaded"),
        Err(rejected) => rejected.diagnostics,
    };
    rejected_with(&found, name, &no_parser("beta"));
}

#[test]
fn ps16_old_version_three_members_reports_only_the_version() {
    let name = "ps16_old_version_three_members.json";
    rejected_with(&refused(name), name, &no_parser("1.21.90"));
}
