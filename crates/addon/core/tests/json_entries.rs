//! Per-entry isolation on the reader: the members of a map whose entries fail one by one are the
//! slices of the text the parse gives (`json::parse::members_of`), each entry is read from its
//! own slice, and a failure is told at file offsets with its JSON path. (What `json_slice.rs`
//! pinned of the old door's `parse_at` over `RawJson` entries; it told a failure at the
//! member's key, the reader tells it at the member's value.)

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity, Span};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::parse::{Options, members_of, parse_slice};
use bedrock_addon_core::schema::{FromJson, Schema, read_document};
use bedrock_addon_core::versioning::Reading;
use facet::Facet;
use indexmap::IndexMap;

#[derive(Debug, Facet, Schema)]
struct Friction {
    value: f32,
}

#[derive(Debug, Facet, Schema)]
struct Light {
    level: u8,
}

const DOCUMENT: &str = r#"{
  "components": {
    "minecraft:friction": { /* kept */ "value": 0.4 },
    "minecraft:light": { "level": "bright" },
    "minecraft:dim": { "level": 300 }
  }
}"#;

/// The components of [`DOCUMENT`], each the slice of its value.
fn components(document: JsonSlice<'_>) -> IndexMap<String, JsonSlice<'_>> {
    let root = members_of(document).expect("an object");
    members_of(root["components"]).expect("an object")
}

/// The first error of the strict read of `slice` as `T`: its path and span; `None` when it reads.
fn failure<T: FromJson>(slice: JsonSlice<'_>) -> Option<(String, Span)> {
    let parsed = parse_slice(slice, Options::default()).expect("JSON");
    let mut d = Diagnostics::default();
    let value = {
        let mut read = Read::new(Reading::OWN.strict(), &mut d).profile(Profile::FIRST_FAILURE);
        read_document::<T>(&Node::parsed(&parsed, JsonPath::new()), &mut read)
    };
    let error = d.iter().find(|f| f.severity >= Severity::Error);
    match (value, error) {
        (Some(_), None) => None,
        (_, Some(error)) => Some((
            error.location.json_path.to_string(),
            error.location.span.expect("a span"),
        )),
        (None, None) => panic!("`{}` failed and told nothing", slice.text),
    }
}

#[test]
fn the_other_entries_survive_a_failing_one() {
    let entries = components(JsonSlice::new(DOCUMENT));
    let failed: Vec<bool> = entries
        .values()
        .map(|slice| failure::<Light>(*slice).is_some())
        .collect();
    assert_eq!(failed, [true, true, true]);
    assert!(failure::<Friction>(entries["minecraft:friction"]).is_none());
}

#[test]
fn a_failing_entry_is_reported_at_its_file_offset() {
    let entries = components(JsonSlice::new(DOCUMENT));
    let slice = entries["minecraft:light"];
    let (path, span) = failure::<Light>(slice).expect("a failure");
    // The span covers the member's value, rebased into the file.
    let value = slice.offset + slice.text.find("\"bright\"").unwrap();
    assert_eq!(span.offset as usize, value);
    assert_eq!(span.len as usize, "\"bright\"".len());
    // The path is relative to the slice; the caller prefixes the component key.
    assert_eq!(path, "level");
}

#[derive(Debug, Facet, Schema)]
struct Layers {
    layers: Vec<IndexMap<String, Light>>,
    renamed: Option<Renamed>,
}

#[derive(Debug, Facet, Schema)]
struct Renamed {
    #[facet(rename = "minecraft:level")]
    level: u8,
}

#[test]
fn a_failing_member_is_named_by_its_json_path() {
    let text = r#"{ "layers": [{ "a": { "level": 1 } }, { "b": { "level": "x" } }] }"#;
    let (path, _) = failure::<Layers>(JsonSlice::new(text)).expect("a failure");
    assert_eq!(path, "layers/1/b/level");
    let text = r#"{ "layers": [], "renamed": {} }"#;
    let (path, _) = failure::<Layers>(JsonSlice::new(text)).expect("a failure");
    assert_eq!(path, "renamed/minecraft:level");
}

#[test]
fn a_whole_file_slice_reports_its_offsets_unchanged() {
    let text = r#"{ "level": true }"#;
    let (_, span) = failure::<Light>(JsonSlice::new(text)).expect("a failure");
    assert_eq!(span.offset as usize, text.find("true").unwrap());
}
