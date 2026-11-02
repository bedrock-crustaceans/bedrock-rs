//! Writing a catalogue back: `parse(write(parse(x))) ==
//! parse(x)`, unknown members kept at their level, nothing written the author did not write.

use bedrock_addon_core::document::Content;
use bedrock_addon_core::json::{Indent, Value, WriteStyle};
use bedrock_addon_family_crafting_catalog::CraftingCatalog;

use super::common::*;

/// The JSON `text` holds.
fn json(text: &str) -> Value {
    bedrock_addon_core::json::from_str(text).unwrap_or_else(|error| panic!("{error}: {text}"))
}

/// Writes `catalog`, re-reads the text strictly (no comments, no finding) and requires the
/// re-read value to equal it and to write the same text again.
#[track_caller]
fn round_trip(catalog: &CraftingCatalog) -> String {
    let written = catalog.to_json().unwrap();
    json(&written);
    let again = loads(&written);
    assert!(
        !again
            .diagnostics()
            .iter()
            .any(|f| f.severity > bedrock_addon_core::diagnostic::Severity::Warning),
        "{:#?}",
        again.diagnostics()
    );
    assert_eq!(again.value(), catalog);
    assert_eq!(again.to_json().unwrap(), written, "the rewrite differs");
    written
}

#[test]
fn cat01_is_written_as_its_expected_document() {
    let document = loads(&fixture("cat01_all_categories.json"));
    let written = round_trip(document.value());
    assert_eq!(
        json(&written),
        json(&fixture("cat01_all_categories.expected.json"))
    );
}

/// `vanilla_1.26.40`'s file: the strongest capture.
#[test]
fn vanilla_s_catalogue_round_trips() {
    let document = loads(&fixture("cat14_line_comments.json"));
    let written = round_trip(document.value());
    // The same members as the capture, comments apart, under the newest struct's version.
    let stripped: String = fixture("cat14_line_comments.json")
        .lines()
        .map(|line| line.split("//").next().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
        .replace("\"1.26.40\"", "\"1.26.50\"");
    assert_eq!(json(&written), json(&stripped));
}

#[test]
fn the_written_format_version_is_the_newest_struct_s_whatever_the_file_said() {
    let text = fixture("cat01_all_categories.json").replace("1.21.60", "1.26.40");
    let written = loads(&text).to_json().unwrap();
    let root = json(&written);
    let root = root.as_object().unwrap();
    assert_eq!(root.len(), 2);
    assert_eq!(
        root.get("format_version")
            .and_then(|v| v.as_string())
            .map(|s| s.as_str()),
        Some("1.26.50")
    );
    assert!(root.get("minecraft:crafting_items_catalog").is_some());
    // `format_version` first, as vanilla's own files have it.
    assert!(
        written.trim_start().starts_with("{\n  \"format_version\""),
        "{written}"
    );
}

/// A read keeps no unknown member; an `extra` filled in code is written after the
/// known members of its level, at 1.26.50, where the strict slice refuses it (C2).
#[test]
fn a_hand_built_extra_is_written_after_the_known_members_and_refused_on_read() {
    let mut value = loads(&fixture("cat01_all_categories.json")).value().clone();
    value.extra.insert("note".to_owned(), json("[1, 2]"));
    value.categories[0]
        .extra
        .insert("hidden".to_owned(), json("true"));
    let written = value.to_json().unwrap();
    assert!(
        written.contains("\"format_version\": \"1.26.50\""),
        "{written}"
    );
    let position = |needle: &str| written.find(needle).unwrap_or_else(|| panic!("{needle}"));
    assert!(position("\"categories\"") < position("\"note\""));
    assert!(position("\"groups\"") < position("\"hidden\""));
    let found = rejects(&written);
    let unknown: Vec<String> = found
        .iter()
        .filter(|f| f.code.name() == "UnrecognizedField")
        .map(|f| {
            assert_eq!(f.severity, bedrock_addon_core::diagnostic::Severity::Error);
            f.location.json_path.to_string()
        })
        .collect();
    assert_eq!(
        unknown,
        [
            "minecraft:crafting_items_catalog/note",
            "minecraft:crafting_items_catalog/categories/0/hidden"
        ]
    );
}

#[test]
fn an_object_item_without_unknown_members_is_written_as_the_string() {
    let text = r#"{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {"categories": [
        {"category_name": "items", "groups": [{"items": [{"name": "custom:a:3"}]}]}]}}"#;
    let written = round_trip(loads(text).value());
    assert!(written.contains("\"custom:a:3\""));
    assert!(!written.contains("\"name\""), "{written}");
}

#[test]
fn nothing_the_author_did_not_write_is_written() {
    let text = fixture("cat10_empty_arrays/groups_and_items.json");
    let written = round_trip(loads(&text).value());
    assert_eq!(json(&written), json(&text.replace("1.21.60", "1.26.50")));
    assert!(!written.contains("group_identifier"));
    let named = r#"{"format_version": "1.26.50", "minecraft:crafting_items_catalog": {"categories": [
        {"category_name": "nature", "groups": [{"group_identifier": {"name": "custom:g"}, "items": []}]}]}}"#;
    let written = round_trip(loads(named).value());
    assert_eq!(json(&written), json(named));
    assert!(!written.contains("icon"));
}

#[test]
fn write_json_honours_the_style() {
    let document = loads(&fixture("cat10_empty_arrays/categories.json"));
    let mut compact = Vec::new();
    document
        .write_json(
            &mut compact,
            WriteStyle {
                indent: Indent::Compact,
                trailing_newline: false,
            },
        )
        .unwrap();
    assert_eq!(
        String::from_utf8(compact).unwrap(),
        r#"{"format_version":"1.26.50","minecraft:crafting_items_catalog":{"categories":[]}}"#
    );
}
