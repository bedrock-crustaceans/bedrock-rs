//! The 44 synthetic malformed manifests, one row per file: whether the manifest loads and the
//! finding it must produce (code by `matches!`, severity, JSON path). The rows that
//! differ from the corpus's own expectation table name the reason.

use bedrock_addon::diagnostic::{DiagnosticCode as C, Severity};
use bedrock_addon::manifest::ModuleType;
use bedrock_addon::version::SemVersion;

use super::{findings, fixture, fixtures, has, parse};

/// What one fixture must do.
struct Row {
    file: &'static str,
    /// The manifest loads (no error-level finding).
    loads: bool,
    severity: Severity,
    /// The JSON path of the expected finding.
    path: &'static str,
    code: fn(&C) -> bool,
}

const E: Severity = Severity::Error;
const W: Severity = Severity::Warning;
const I: Severity = Severity::Info;

#[rustfmt::skip]
const TABLE: &[Row] = &[
    // fv0 metadata is read from the header, strictly: error 9.
    // By legacy name: the variant and the definitions that were it.
    Row { file: "fv0_metadata_wrong_type.json", loads: false, severity: E, path: "header/metadata", code: |c| matches!(c.name(), "WrongTypeProperty") },
    // 256 is version 256.0.0 (a number read as its text), above the comprehensive slice
    // [3.0.0, 3.0.0]: no parser.
    // By legacy name: the variant and the definitions that were it.
    Row { file: "fv_256.json", loads: false, severity: E, path: "format_version", code: |c| matches!(c.name(), "UnsupportedVersion") },
    // 258 is 258.0.0: no parser.
    // By legacy name: the variant and the definitions that were it.
    Row { file: "fv_258.json", loads: false, severity: E, path: "format_version", code: |c| matches!(c.name(), "UnsupportedVersion") },
    // A number is read as its text: 3 is the comprehensive format, which refuses the
    // array version of this format-2 fixture.
    // By legacy name: the variant and the definitions that were it.
    Row { file: "fv_3.json", loads: false, severity: E, path: "header/version", code: |c| matches!(c.name(), "InvalidType") },
    // Absent → 0: the deprecated path looks for `pack_id`.
    // By legacy name: the variant and the definitions that were it.
    Row { file: "fv_absent.json", loads: false, severity: E, path: "header/pack_id", code: |c| matches!(c.name(), "MissingProperty") },
    // The game gives no version for these: "Malformed format version", nothing read.
    // By legacy name: the variant and the definitions that were it.
    Row { file: "fv_array.json", loads: false, severity: E, path: "format_version", code: |c| matches!(c.name(), "MalformedFormatVersion") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "fv_bool.json", loads: false, severity: E, path: "format_version", code: |c| matches!(c.name(), "MalformedFormatVersion") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "fv_neg1.json", loads: false, severity: E, path: "format_version", code: |c| matches!(c.name(), "MalformedFormatVersion") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "fv_null.json", loads: false, severity: E, path: "format_version", code: |c| matches!(c.name(), "MalformedFormatVersion") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "fv_object.json", loads: false, severity: E, path: "format_version", code: |c| matches!(c.name(), "MalformedFormatVersion") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "fv_string.json", loads: false, severity: I, path: "format_version", code: |c| matches!(c.name(), "WrongTypeProperty") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "header_absent.json", loads: false, severity: E, path: "header", code: |c| matches!(c.name(), "MissingProperty") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "header_wrong_type.json", loads: false, severity: E, path: "header", code: |c| matches!(c.name(), "WrongTypeProperty") },
    // Syntax errors: `MalformedInput` (vanilla 5), nothing read (the corpus expected
    // vanilla's partial-tree warning).
    // By legacy name: the variant and the definitions that were it.
    Row { file: "malformed_nan.json", loads: false, severity: E, path: "", code: |c| matches!(c.name(), "MalformedInput") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "malformed_single_quotes.json", loads: false, severity: E, path: "", code: |c| matches!(c.name(), "MalformedInput") },
    // Deviation: a trailing comma is accepted; the fixture then lacks
    // `min_engine_version`.
    // By legacy name: the variant and the definitions that were it.
    Row { file: "malformed_trailing_comma.json", loads: false, severity: E, path: "header/min_engine_version", code: |c| matches!(c.name(), "MissingProperty") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "malformed_unquoted_key.json", loads: false, severity: E, path: "", code: |c| matches!(c.name(), "MalformedInput") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "malformed_unterminated.json", loads: false, severity: E, path: "", code: |c| matches!(c.name(), "MalformedInput") },
    Row { file: "min_engine_above_current.json", loads: false, severity: E, path: "header/min_engine_version", code: |c| matches!(c, C::VersionTooHigh { version, .. } if version == "1.99.0") },
    Row { file: "min_engine_below_1_13.json", loads: false, severity: E, path: "header/min_engine_version", code: |c| matches!(c, C::VersionTooLow { version, limit } if version == "1.5.0" && limit == "1.13.0") },
    // Strict path: an absent `modules` is error 9, not warning 20.
    // By legacy name: the variant and the definitions that were it.
    Row { file: "modules_absent.json", loads: false, severity: E, path: "modules", code: |c| matches!(c.name(), "WrongTypeProperty") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "modules_missing_type.json", loads: false, severity: E, path: "modules/0/type", code: |c| matches!(c.name(), "InvalidProperty") },
    // Lenient path: warning 19 and `resources`.
    // By legacy name: the variant and the definitions that were it.
    Row { file: "modules_unknown_type_fv0.json", loads: true, severity: W, path: "header/modules/0/type", code: |c| matches!(c.name(), "InvalidProperty") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "modules_unknown_type_fv2.json", loads: false, severity: E, path: "modules/0/type", code: |c| matches!(c.name(), "InvalidProperty") },
    // fv0: warning 11, the UUID as the name.
    // By legacy name: the variant and the definitions that were it.
    Row { file: "name_empty_fv0.json", loads: true, severity: W, path: "header/name", code: |c| matches!(c.name(), "EmptyProperty") },
    // fv2: `manifestvalidation.noname` and no report entry, not the corpus's error 10:
    // an information, ours.
    // By legacy name: the variant and the definitions that were it.
    Row { file: "name_empty_fv2.json", loads: true, severity: I, path: "header/name", code: |c| matches!(c.name(), "EmptyProperty") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "root_not_object.json", loads: false, severity: E, path: "header", code: |c| matches!(c.name(), "MissingProperty") },
    // a missing UUID is `missing_property` and rejected (vanilla: random).
    // By legacy name: the variant and the definitions that were it.
    Row { file: "uuid_absent.json", loads: false, severity: E, path: "header/uuid", code: |c| matches!(c.name(), "MissingProperty") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "uuid_empty.json", loads: false, severity: E, path: "header/uuid", code: |c| matches!(c.name(), "EmptyProperty") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "uuid_malformed.json", loads: false, severity: E, path: "header/uuid", code: |c| matches!(c.name(), "MalformedPropertyUuid") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "uuid_malformed_fv0.json", loads: true, severity: W, path: "header/pack_id", code: |c| matches!(c.name(), "MalformedPropertyUuid") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "uuid_nil_explicit.json", loads: false, severity: E, path: "header/uuid", code: |c| matches!(c.name(), "MalformedPropertyUuid") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "uuid_wrong_type.json", loads: false, severity: E, path: "header/uuid", code: |c| matches!(c.name(), "WrongTypeProperty") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "version_absent.json", loads: false, severity: E, path: "header/version", code: |c| matches!(c.name(), "MissingProperty") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "version_long.json", loads: false, severity: E, path: "header/version", code: |c| matches!(c.name(), "WrongTypeProperty") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "version_out_of_range.json", loads: false, severity: E, path: "header/version", code: |c| matches!(c.name(), "WrongTypeProperty") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "version_short.json", loads: false, severity: E, path: "header/version", code: |c| matches!(c.name(), "WrongTypeProperty") },
    // By legacy name: the variant and the definitions that were it.
    Row { file: "version_string_component.json", loads: false, severity: E, path: "header/version", code: |c| matches!(c.name(), "WrongTypeProperty") },
];

/// The fixtures that produce no finding at the member they exercise.
const QUIET: &[(&str, &str)] = &[
    // Two modules with one UUID: the game compares modules with the header only.
    ("modules_duplicate_uuid.json", ""),
    ("modules_empty.json", "modules"),
    ("version_as_string.json", "header/version"),
    ("version_bool_elem.json", "header/version"),
    ("version_float.json", "header/version"),
    ("version_null_elem.json", "header/version"),
];

#[test]
fn every_malformed_fixture_is_in_a_table() {
    let mut files: Vec<String> = std::fs::read_dir(fixtures().join("malformed"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    files.sort();
    let mut listed: Vec<&str> = TABLE
        .iter()
        .map(|row| row.file)
        .chain(QUIET.iter().map(|(file, _)| *file))
        .collect();
    listed.sort_unstable();
    assert_eq!(files.len(), 44);
    assert_eq!(files, listed);
}

#[test]
fn each_malformed_fixture_produces_its_code() {
    let mut failures = Vec::new();
    for row in TABLE {
        let result = parse(&fixture(&format!("malformed/{}", row.file)));
        if result.is_ok() != row.loads {
            failures.push(format!("{}: loads = {}", row.file, result.is_ok()));
        }
        if !has(findings(&result), row.severity, row.path, row.code) {
            let list: Vec<String> = findings(&result)
                .iter()
                .map(|d| {
                    format!(
                        "{} {} at {:?}",
                        d.severity,
                        d.code.name(),
                        d.location.json_path.to_string()
                    )
                })
                .collect();
            failures.push(format!(
                "{}: expected finding missing; got {list:?}",
                row.file
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// `bytes` with `"min_engine_version": [1, 26, 20]` added to the header, so a fixture that
/// exercises another member can load.
fn with_engine(file: &str) -> Vec<u8> {
    let text = String::from_utf8(fixture(&format!("malformed/{file}"))).unwrap();
    text.replacen(
        "\"header\": {",
        "\"header\": {\n    \"min_engine_version\": [1, 26, 20],",
        1,
    )
    .into_bytes()
}

#[test]
fn quiet_fixtures_produce_no_finding_at_their_member() {
    for (file, path) in QUIET {
        let result = parse(&with_engine(file));
        let at_member: Vec<_> = findings(&result)
            .iter()
            .filter(|d| path.is_empty() || d.location.json_path.to_string() == *path)
            .collect();
        assert!(at_member.is_empty(), "{file}: {at_member:?}");
        assert!(result.is_ok(), "{file}: {:?}", findings(&result));
    }
}

#[test]
fn version_arrays_coerce_reals_booleans_and_null() {
    let version = |file: &str| parse(&with_engine(file)).unwrap().header.version.clone();
    assert_eq!(version("version_float.json"), SemVersion::new(1, 2, 0));
    assert_eq!(version("version_bool_elem.json"), SemVersion::new(1, 0, 1));
    assert_eq!(version("version_null_elem.json"), SemVersion::new(0, 1, 0));
    assert_eq!(version("version_as_string.json"), SemVersion::new(1, 2, 3));
}

#[test]
fn two_modules_with_one_uuid_are_both_kept() {
    let document = parse(&with_engine("modules_duplicate_uuid.json")).unwrap();
    assert_eq!(document.modules.len(), 2);
    assert_eq!(document.modules[0].uuid, document.modules[1].uuid);
}

#[test]
fn the_lenient_path_substitutes_resources_for_an_unknown_module_type() {
    let document = parse(&fixture("malformed/modules_unknown_type_fv0.json")).unwrap();
    assert_eq!(document.modules[0].module_type, ModuleType::Resources);
}

#[test]
fn an_empty_format_0_name_is_the_uuid() {
    let document = parse(&fixture("malformed/name_empty_fv0.json")).unwrap();
    assert_eq!(document.header.name, "33333333-3333-3333-3333-333333333333");
}

#[test]
fn a_malformed_uuid_is_replaced_by_the_same_hash_every_time() {
    let first = parse(&fixture("malformed/uuid_malformed_fv0.json")).unwrap();
    let second = parse(&fixture("malformed/uuid_malformed_fv0.json")).unwrap();
    assert_eq!(first.header.uuid, second.header.uuid);
    assert_eq!(first.header.uuid.get_version_num(), 3);
}

#[test]
fn deviation_a_trailing_comma_is_accepted() {
    let result = parse(&fixture("malformed/malformed_trailing_comma.json"));
    assert!(
        !findings(&result)
            .iter()
            // By legacy name: the variant and the definitions that were it.
            .any(|d| matches!(d.code.name(), "MalformedInput"))
    );
    assert!(parse(&with_engine("malformed_trailing_comma.json")).is_ok());
}

#[test]
fn a_syntax_error_keeps_its_file_offset() {
    // the stray `@` sits at byte 11 of the text, 14 of the file after the byte-order mark
    let text = "{\"header\": @}";
    for (bytes, offset) in [
        (text.as_bytes().to_vec(), 11),
        ([b"\xEF\xBB\xBF".as_slice(), text.as_bytes()].concat(), 14),
    ] {
        let result = parse(&bytes);
        let finding = findings(&result)
            .iter()
            // By legacy name: the variant and the definitions that were it.
            .find(|d| matches!(d.code.name(), "MalformedInput"))
            .expect("a syntax error");
        let span = finding.location.span.expect("the syntax error's span");
        assert_eq!(span.offset, offset, "{finding:?}");
    }
}

/// The findings of `text` as `(code name, severity, path)`.
fn told(text: &str) -> Vec<(String, Severity, String)> {
    let document = parse(text.as_bytes()).unwrap_or_else(|r| panic!("{:#?}", r.diagnostics));
    document
        .diagnostics()
        .iter()
        .map(|d| {
            (
                d.code.name().to_owned(),
                d.severity,
                d.location.json_path.to_string(),
            )
        })
        .collect()
}

/// Format 1 reads like format 2 for an empty name. The header parser takes the
/// legacy branch only for format 0; an empty name takes the UUID branch with
/// its warning type 11 there, and `manifestvalidation.noname`
/// without a report entry otherwise.
#[test]
fn an_empty_format_1_name_is_the_noname_placeholder_without_a_finding() {
    let text = super::format_2("", "")
        .replacen("\"Ruby\"", "\"\"", 1)
        .replacen("\"format_version\": 2", "\"format_version\": 1", 1);
    let document = parse(text.as_bytes()).unwrap();
    // kept empty (reports show `manifestvalidation.noname`), not the UUID of format 0
    assert_eq!(document.header.name, "");
    let at_name: Vec<_> = told(&text)
        .into_iter()
        .filter(|(_, _, path)| path == "header/name")
        .collect();
    assert!(at_name.is_empty(), "{at_name:?}");
}

/// the comprehensive reader has no report site for an empty name, so its finding is
/// the crate's own information.
#[test]
fn an_empty_format_3_name_is_an_information() {
    let text = super::format_3("", "").replacen("\"Ruby\"", "\"\"", 1);
    let at_name: Vec<_> = told(&text)
        .into_iter()
        .filter(|(_, _, path)| path == "header/name")
        .collect();
    assert_eq!(
        at_name,
        [("EmptyProperty".to_owned(), I, "header/name".to_owned())]
    );
}

/// R1: a format-0 manifest logs "Pack Manifest version 0 is deprecated, please
/// include `format_version` 2 in your manifest.json." at content-log level 2.
#[test]
fn a_format_0_manifest_is_told_deprecated_at_level_2() {
    let document = parse(&fixture("malformed/name_empty_fv0.json")).unwrap();
    let found: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|d| d.code.name() == "DeprecatedMember")
        .collect();
    assert_eq!(found.len(), 1, "{:#?}", document.diagnostics());
    assert_eq!(found[0].severity, Severity::Warning);
    assert_eq!(
        found[0].code.to_string(),
        "Pack Manifest version 0 is deprecated, please include format_version 2 in your manifest.json."
    );
    // a format-1 and a format-2 manifest are not
    for format in ["1", "2"] {
        let text = super::format_2("", "").replacen(
            "\"format_version\": 2",
            &format!("\"format_version\": {format}"),
            1,
        );
        assert!(
            told(&text)
                .iter()
                .all(|(name, ..)| name != "DeprecatedMember"),
            "format {format}"
        );
    }
}
