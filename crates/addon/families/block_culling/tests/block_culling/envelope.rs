//! Rows v01 to v12: the envelope and the version range
//! [1.20.60, 1.21.80]. A file that never reaches a parser is "Missing object"; a version above
//! the ceiling is "no parser available", then "Failed to parse".

use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Family, RangeModel, VersionChain};
use bedrock_addon_family_block_culling::CullingRules;
use bedrock_addon_family_block_culling::history::{Any, CullingFamily};

use super::common::{
    base_game, document, error, errors, failed_to_parse, fixture, loads, missing_object, rejects,
    rejects_in,
};

const PAYLOAD: &str = r#"{"description": {"identifier": "a:b"}, "rules": []}"#;

/// The closing "Missing object" line, after a finding that carries a text of its own.
fn closing_missing_object() -> (&'static str, String, String) {
    missing_object("DocumentRejected", "")
}

/// "no parser available" for `version`.
fn no_parser(version: &str) -> (&'static str, String, String) {
    error(
        "UnsupportedVersion",
        &format!("unsupported version: no parser available for version {version}"),
        "format_version",
    )
}

#[test]
fn v01_text_that_is_not_json() {
    // Vanilla `BlockCullingBadJson_ParseWithCereal_ExpectFailedParsing` asserts "Missing
    // object".
    let found = errors(&rejects(&fixture("v01_bad_json.json")));
    assert_eq!(found.len(), 2, "{found:#?}");
    assert_eq!((found[0].0, found[0].2.as_str()), ("MalformedInput", ""));
    assert_eq!(found[1], closing_missing_object());
    for text in ["", "[]", "5", "\"x\"", "{", "\u{0}"] {
        let found = errors(&rejects(text));
        assert_eq!(found.last(), Some(&closing_missing_object()), "{text:?}");
    }
    // Bytes that are not UTF-8, and a document nested beyond the limit.
    for bytes in [&b"\xff\xfe"[..], "[".repeat(200).as_bytes()] {
        let rejected = CullingRules_parse(bytes).unwrap_err();
        assert_eq!(
            errors(&rejected).last(),
            Some(&closing_missing_object()),
            "{bytes:?}"
        );
    }
}

/// `CullingRules::parse` of raw bytes from a user pack: the findings of a rejection.
#[allow(non_snake_case)]
fn CullingRules_parse(bytes: &[u8]) -> Result<(), bedrock_addon_core::diagnostic::Diagnostics> {
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::pack_identity::PackPath;
    CullingRules::parse(
        &PackPath::from_static(super::common::PATH),
        bytes,
        &super::common::user_pack(),
    )
    .map(|_| ())
    .map_err(|rejected| rejected.diagnostics)
}

#[test]
fn a_byte_order_mark_is_skipped() {
    let text = format!("\u{feff}{}", document("1.21.80", "a:b", &[]));
    assert!(CullingRules_parse(text.as_bytes()).is_ok());
}

#[test]
fn v02_a_missing_format_version_is_not_floored() {
    let json = format!(r#"{{"minecraft:block_culling_rules": {PAYLOAD}}}"#);
    assert_eq!(
        errors(&rejects(&json)),
        [missing_object("MissingFormatVersion", "")]
    );
    // In a base-game pack too.
    assert_eq!(
        errors(&rejects_in(&json, &base_game())),
        [missing_object("MissingFormatVersion", "")]
    );
}

#[test]
fn v03_a_third_root_member() {
    let json = format!(
        r#"{{"format_version": "1.21.80", "minecraft:block_culling_rules": {PAYLOAD}, "extra": 1}}"#
    );
    let found = errors(&rejects(&json));
    assert_eq!(found.len(), 2, "{found:#?}");
    assert_eq!(found[0].0, "MalformedInput");
    assert_eq!(found[1], closing_missing_object());
    // One member is not two either.
    let found = errors(&rejects(r#"{"format_version": "1.21.80"}"#));
    assert_eq!(found[1], closing_missing_object());
}

#[test]
fn v04_another_payload_key() {
    for key in [
        "minecraft:block_culling",
        "minecraft:block_culling_rule",
        "minecraft:block_culling_ruless",
        "Minecraft:block_culling_rules",
        "block_culling_rules",
    ] {
        let json = format!(r#"{{"format_version": "1.21.80", "{key}": {PAYLOAD}}}"#);
        assert_eq!(
            errors(&rejects(&json)),
            [missing_object("MissingPayload", "")],
            "{key}"
        );
    }
}

#[test]
fn v05_below_the_floor_is_missing_object_only() {
    for version in ["1.20.50", "1.20.59", "1.19.0", "0.0.0", "1.2.3"] {
        assert_eq!(
            errors(&rejects(&document(version, "a:b", &[]))),
            [missing_object("UnsupportedVersion", "format_version")],
            "{version}"
        );
    }
    // The envelope is never looked at below the floor: a third member changes nothing.
    let json = format!(
        r#"{{"format_version": "1.20.50", "minecraft:block_culling_rules": {PAYLOAD}, "extra": 1}}"#
    );
    assert_eq!(
        errors(&rejects(&json)),
        [missing_object("UnsupportedVersion", "format_version")]
    );
}

#[test]
fn v06_above_the_ceiling_has_no_parser() {
    for version in ["1.21.90", "1.21.81", "2.0.0"] {
        assert_eq!(
            errors(&rejects(&document(version, "a:b", &[]))),
            [no_parser(version), failed_to_parse()],
            "{version}"
        );
    }
}

/// The text prints the version as the game parsed it (`SemVersion::asString`): missing
/// numbers are 0 and build metadata is dropped; a pre-release and the beta literal stay as
/// written.
#[test]
fn v06b_the_no_parser_text_prints_the_version_as_parsed() {
    for (written, printed) in [
        ("1.26", "1.26.0"),
        ("2", "2.0.0"),
        ("1.21.90+abc", "1.21.90"),
    ] {
        assert_eq!(
            errors(&rejects(&document(written, "a:b", &[]))),
            [no_parser(printed), failed_to_parse()],
            "{written}"
        );
    }
}

#[test]
fn v07_the_engine_s_own_version_is_above_the_ceiling() {
    for version in ["1.26.20", "1.26.50"] {
        assert_eq!(
            errors(&rejects(&document(version, "a:b", &[]))),
            [no_parser(version), failed_to_parse()],
            "{version}"
        );
    }
}

#[test]
fn v08_a_pre_release_of_1_21_80_is_not_loaded() {
    // The folder has the 1.20.60 struct parse it and refuse `condition` as unknown; the chain
    //  has no parser for it.
    // Either way the file is skipped.
    let json = document(
        "1.21.80-beta.1",
        "a:b",
        &[r#"{"condition": "same_block", "geometry_part": {"bone": "b"}, "direction": "up"}"#],
    );
    assert_eq!(
        errors(&rejects(&json)),
        [no_parser("1.21.80-beta.1"), failed_to_parse()]
    );
}

#[test]
fn v09_the_beta_literal_has_no_parser() {
    assert_eq!(
        errors(&rejects(&document("beta", "a:b", &[]))),
        [no_parser("beta"), failed_to_parse()]
    );
    // Not a matter of the pack's trust: a base-game pack has no beta parser either.
    assert_eq!(
        errors(&rejects_in(&document("beta", "a:b", &[]), &base_game())),
        [no_parser("beta"), failed_to_parse()]
    );
}

#[test]
fn v10_the_floor_itself_loads_through_the_1_20_60_struct() {
    let document_ = loads(&document("1.20.60", "a:b", &[]));
    assert_eq!(document_.version().parsed_as, FormatVersion::new(1, 20, 60));
    assert!(document_.version().upgraded);
    // The newest version loads without a step; one patch below it is still the old struct.
    let newest = loads(&document("1.21.80", "a:b", &[]));
    assert_eq!(newest.version().parsed_as, FormatVersion::new(1, 21, 80));
    assert!(!newest.version().upgraded);
    let below = loads(&document("1.21.79", "a:b", &[]));
    assert_eq!(below.version().parsed_as, FormatVersion::new(1, 20, 60));
    assert_eq!(below.source_format_version, FormatVersion::new(1, 21, 79));
}

#[test]
fn v11_a_pre_release_of_the_floor_is_below_it() {
    assert_eq!(
        errors(&rejects(&document("1.20.60-beta.1", "a:b", &[]))),
        [missing_object("UnsupportedVersion", "format_version")]
    );
}

#[test]
fn v12_a_version_that_is_not_one() {
    for version in [
        r#""abc""#,
        "5",
        "true",
        "null",
        r#""1.21.x""#,
        "[1, 21]",
        "{}",
    ] {
        let json = format!(
            r#"{{"format_version": {version}, "minecraft:block_culling_rules": {PAYLOAD}}}"#
        );
        assert_eq!(
            errors(&rejects(&json)),
            [missing_object("MalformedFormatVersion", "format_version")],
            "{version}"
        );
    }
}

#[test]
fn the_chain_is_two_structs_closed_at_the_newest() {
    assert_eq!(
        <Any as VersionChain<CullingFamily>>::VERSIONS,
        [FormatVersion::new(1, 20, 60), FormatVersion::new(1, 21, 80)]
    );
    assert_eq!(CullingFamily::FLOOR, FormatVersion::new(1, 20, 60));
    assert_eq!(CullingFamily::RANGE_MODEL, RangeModel::ClosedToLatest);
    assert_eq!(
        CullingFamily::CEILING,
        None,
        "derived: the newest struct's version"
    );
    assert_eq!(CullingFamily::PAYLOAD_KEY, "minecraft:block_culling_rules");
    assert_eq!(CullingFamily::PAYLOAD_KEY.len(), 29);
    assert_eq!(CullingFamily::DIRECTORIES, ["block_culling"]);
    assert_eq!(CullingRules::FORMAT_VERSION, FormatVersion::new(1, 21, 80));
    bedrock_addon_core::versioning::check_chain::<CullingFamily>().expect("a contiguous chain");
}

#[test]
fn content_after_the_document_is_ignored_with_one_warning_at_its_span() {
    // Core's versioned load reads the file's root and reports the content; the
    // payload reader, which reads a slice, adds nothing.
    let valid = super::common::latest(&[r#"{"geometry_part": {"bone": "b"}, "direction": "up"}"#]);
    let text = format!("{valid}\n garbage\n");
    let document = super::common::load_in(&text, &super::common::user_pack())
        .unwrap_or_else(|r| panic!("{:#?}", r.diagnostics));
    assert_eq!(document.rules.len(), 1);
    let found: Vec<_> = document.diagnostics().iter().collect();
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].code.name(), "MalformedInput");
    assert_eq!(
        found[0].severity,
        bedrock_addon_core::diagnostic::Severity::Info
    );
    let span = found[0].location.span.expect("the content's span");
    let at = span.offset as usize;
    assert_eq!(&text[at..at + span.len as usize], "garbage");
}
