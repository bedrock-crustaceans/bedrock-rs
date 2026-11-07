//! The checks vanilla's loader runs before it parses the payload, in
//! vanilla's order: a valid version, the floor, the payload key, the engine.

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity, codes};
use bedrock_addon_family_crafting_catalog::codes as crafting_catalog;

use super::common::*;

const FORMAT_VERSION_TEXT: &str = "The file cannot be processed without a valid format_version";
const PAYLOAD_TEXT: &str =
    "The crafting item catalog needs to have an object minecraft:crafting_items_catalog";

/// A document with the root members `members` (JSON text).
fn root(members: &str) -> String {
    format!("{{{members}}}")
}

const PAYLOAD: &str = r#""minecraft:crafting_items_catalog": {"categories": []}"#;
const OTHER_PAYLOAD: &str = r#""minecraft:crafting_item_catalog": {"categories": []}"#;

/// The findings below error level: the notes of a rejection.
fn notes(d: &Diagnostics) -> Vec<&Diagnostic> {
    d.iter().filter(|f| f.severity < Severity::Error).collect()
}

#[track_caller]
fn missing_version(json: &str) -> Diagnostics {
    let found = rejects(json);
    let error = one_error(&found);
    assert!(
        // By legacy name: the variant and the definitions that were it.
        matches!(error.code.name(), "MissingFormatVersion"),
        "{json}: {found:#?}"
    );
    assert_eq!(error.code.to_string(), FORMAT_VERSION_TEXT, "{json}");
    found
}

/// `initAsJson` reads a number as its text: `1` is 1.0.0, a version
/// below 1.21.60 (L2's text), and `1.26` is 1.26.0, which loads.
#[test]
fn a_number_format_version_is_read_as_its_text() {
    let found = rejects(&root(&format!(r#""format_version": 1, {PAYLOAD}"#)));
    let error = one_error(&found);
    assert!(
        // By legacy name: the variant and the definitions that were it.
        matches!(error.code.name(), "UnsupportedVersion"),
        "{found:#?}"
    );
    assert_eq!(
        error.code.to_string(),
        "The file cannot be processed without a valid format_version"
    );
    let document = loads(&root(&format!(r#""format_version": 1.26, {PAYLOAD}"#)));
    assert_eq!(
        document.version().written,
        Some(bedrock_addon_core::version::FormatVersion::new(1, 26, 0))
    );
}

#[test]
fn l1a_every_unreadable_format_version_is_the_missing_version_error() {
    for version in [
        "true",
        "null",
        "{}",
        "[1, 21]",
        "[1, 21, 60, 0]",
        "[1, -21, 60]",
        r#"["1", "21", "60"]"#,
        r#""""#,
        r#""*""#,
    ] {
        let found = missing_version(&root(&format!(r#""format_version": {version}, {PAYLOAD}"#)));
        // Vanilla's "Malformed format version" goes to the document's own logger only: a note.
        let notes = notes(&found);
        assert_eq!(notes.len(), 1, "{version}: {found:#?}");
        // By legacy name: the variant and the definitions that were it.
        assert!(matches!(notes[0].code.name(), "MalformedFormatVersion"));
        assert_eq!(notes[0].severity, Severity::Info);
    }
}

#[test]
fn l1a_a_third_root_member_is_the_missing_version_error_with_the_envelope_text_as_a_note() {
    let found = missing_version(&fixture("cat04b_envelope_edges/third_root_member.json"));
    let notes = notes(&found);
    assert_eq!(notes.len(), 1, "{found:#?}");
    // By legacy name: the variant and the definitions that were it.
    assert!(matches!(notes[0].code.name(), "MalformedInput"));
    assert_eq!(notes[0].severity, Severity::Info);
    assert_eq!(
        notes[0].code.to_string(),
        "A Minecraft Document must have exactly two members: \"format_version\" and a payload member"
    );
}

#[test]
fn l1a_a_json_syntax_error_is_the_missing_version_error_with_the_reader_s_text_as_a_note() {
    let found = missing_version(r#"{"format_version": "1.21.60", "#);
    let notes = notes(&found);
    assert_eq!(notes.len(), 1, "{found:#?}");
    assert!(notes[0].code.is(&codes::DOCUMENT_JSON));
    assert_eq!(notes[0].severity, Severity::Info);
    assert!(
        notes[0]
            .code
            .to_string()
            .starts_with("Failed to parse the Minecraft Document JSON: ")
    );
}

#[test]
fn l1a_a_root_that_is_no_object_or_no_text_is_the_missing_version_error() {
    for json in ["[]", "\"1.21.60\"", "", "null", "  // nothing\n"] {
        missing_version(json);
    }
    let found = CraftingCatalogBytes(b"{\"format_version\": \"1.21.60\", \xff}").rejected();
    let error = one_error(&found);
    // By legacy name: the variant and the definitions that were it.
    assert!(matches!(error.code.name(), "MissingFormatVersion"));
    assert_eq!(error.code.to_string(), FORMAT_VERSION_TEXT);
}

/// A file given as bytes (not UTF-8).
struct CraftingCatalogBytes(&'static [u8]);

impl CraftingCatalogBytes {
    fn rejected(&self) -> Diagnostics {
        use bedrock_addon_core::document::Content;
        bedrock_addon_family_crafting_catalog::CraftingCatalog::parse(
            &bedrock_addon_core::pack_identity::PackPath::from_static(PATH),
            self.0,
            &bedrock_addon_core::load::LoadContext::detached(),
        )
        .expect_err("rejected")
        .diagnostics
    }
}

#[test]
fn l1_the_version_is_checked_before_the_payload_key() {
    // No version and no payload: vanilla stops at the version.
    missing_version(&root(OTHER_PAYLOAD));
    missing_version(&root(""));
    // No version, and more than one other member.
    missing_version(&root(&format!(r#"{PAYLOAD}, "extra": 1"#)));
}

#[test]
fn l2_the_floor_is_checked_before_the_shape_and_the_payload_key() {
    for members in [
        format!(r#""format_version": "1.21.50", {OTHER_PAYLOAD}"#),
        // Below its minimum version vanilla does not count the root members.
        format!(r#""format_version": "1.21.50", {PAYLOAD}, "extra": 1"#),
        r#""format_version": "1.21.50""#.to_owned(),
        format!(r#""format_version": [1, 21, 59], {PAYLOAD}"#),
        format!(r#""format_version": "0.0.0", {PAYLOAD}"#),
    ] {
        let found = rejects(&root(&members));
        let error = one_error(&found);
        assert!(
            // By legacy name: the variant and the definitions that were it.
            matches!(error.code.name(), "UnsupportedVersion"),
            "{members}: {found:#?}"
        );
        assert_eq!(error.code.to_string(), FORMAT_VERSION_TEXT, "{members}");
    }
}

#[test]
fn l4_the_payload_key_is_checked_before_the_engine_version() {
    for members in [
        format!(r#""format_version": "1.21.60", {OTHER_PAYLOAD}"#),
        format!(r#""format_version": "1.27.0", {OTHER_PAYLOAD}"#),
        r#""format_version": "1.21.60", "Minecraft:Crafting_Items_Catalog": {"categories": []}"#
            .to_owned(),
    ] {
        let found = rejects(&root(&members));
        let error = one_error(&found);
        assert!(
            // By legacy name: the variant and the definitions that were it.
            matches!(error.code.name(), "MissingRequiredField"),
            "{members}: {found:#?}"
        );
        assert_eq!(error.code.to_string(), PAYLOAD_TEXT, "{members}");
    }
    // A lone `format_version` at or above the floor has one member, not two: the shape.
    missing_version(&root(r#""format_version": "1.21.60""#));
}

#[test]
fn l3_above_the_engine_is_an_error_naming_the_version() {
    let found = rejects(&root(&format!(r#""format_version": "1.27.0", {PAYLOAD}"#)));
    let error = one_error(&found);
    assert!(
        error
            .code
            .args_of(&codes::FORMAT_VERSION_ABOVE_ENGINE)
            .is_some_and(|args| args[0] == "1.27.0")
    );
    assert_eq!(found.iter().count(), 1, "{found:#?}");
}

#[test]
fn a_prerelease_version_is_an_ordinary_version() {
    // 1.21.60-beta.1 sorts below 1.21.60: under the floor.
    let below = rejects(&root(&format!(
        r#""format_version": "1.21.60-beta.1", {PAYLOAD}"#
    )));
    assert!(one_error(&below).is(&crafting_catalog::BELOW_FLOOR));
    let document = loads(&root(&format!(
        r#""format_version": "1.21.70-beta.1", {PAYLOAD}"#
    )));
    assert!(!document.diagnostics().has_errors());
}

/// Where beta documents are allowed, `"beta"` still selects nothing: the catalogue has no beta
/// struct. It is the parser's "no parser available", not a version above the engine.
#[test]
fn the_beta_literal_is_unsupported_where_beta_documents_are_allowed() {
    use bedrock_addon_core::document::Content;
    let mut cx = bedrock_addon_core::load::LoadContext::detached();
    cx.beta_apis = true;
    let text = root(&format!(r#""format_version": "beta", {PAYLOAD}"#));
    let found = bedrock_addon_family_crafting_catalog::CraftingCatalog::parse(
        &bedrock_addon_core::pack_identity::PackPath::from_static(PATH),
        text.as_bytes(),
        &cx,
    )
    .expect_err("rejected")
    .diagnostics;
    let error = one_error(&found);
    assert!(
        // By legacy name: the variant and the definitions that were it.
        matches!(error.code.name(), "UnsupportedVersion"),
        "{found:#?}"
    );
    assert_eq!(
        error.code.to_string(),
        "unsupported version: no parser available for version beta"
    );
}

#[test]
fn the_beta_literal_has_no_struct() {
    let found = rejects(&root(&format!(r#""format_version": "beta", {PAYLOAD}"#)));
    assert_eq!(errors(&found).len(), 1, "{found:#?}");
}
