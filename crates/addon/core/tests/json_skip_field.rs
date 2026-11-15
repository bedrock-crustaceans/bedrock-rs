//! The skip-field convention: a `#[facet(skip)]` field is never read, so a member named like it
//! is an unknown member like any other: told (the family's `UnrecognizedField`), kept in the
//! document's table for a struct marked with `extra: Extra` and written back.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::parse::{Options, parse};
use bedrock_addon_core::json::{Value, from_str_jsonc, write_json_with};
use bedrock_addon_core::schema::{Schema, read_document};
use bedrock_addon_core::versioning::Reading;
use facet::Facet;
use indexmap::IndexMap;

/// A struct whose skipped fields follow the convention.
#[derive(Debug, PartialEq, Facet, Schema)]
struct Marked {
    #[facet(default)]
    level: u8,
    #[facet(skip, rename = "\u{0}converted")]
    converted: bool,
    #[facet(skip, rename = "\u{0}side")]
    side: IndexMap<String, Value>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

fn value(text: &str) -> Value {
    from_str_jsonc(text).unwrap()
}

#[test]
fn a_member_named_like_a_skipped_field_is_an_unknown_member_kept_and_written_back() {
    const TEXT: &str = r#"{"level": 1, "converted": true, "extra": {"a": 2}}"#;
    let parsed = parse(TEXT, Options::default()).unwrap();
    let mut d = Diagnostics::default();
    let marked: Marked = {
        let mut read = Read::new(Reading::DOCUMENT, &mut d);
        read_document(&Node::parsed(&parsed, JsonPath::new()), &mut read).unwrap()
    };
    let unknown: Vec<String> = d
        .iter()
        .filter(|f| f.is(&core_codes::UNRECOGNIZED_FIELD))
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(unknown, ["converted", "extra"]);
    let table = d.unknown();
    assert_eq!(table.len(), 2);
    assert!(!marked.converted, "the skipped field keeps its own value");
    let root = JsonPath::new();
    assert_eq!(table.get(&root, "converted"), Some(&Value::from(true)));
    assert_eq!(table.get(&root, "extra"), Some(&value(r#"{"a": 2}"#)));
    let mut written = Vec::new();
    write_json_with(
        &marked,
        table,
        &mut written,
        bedrock_addon_core::json::WriteStyle::default(),
    )
    .unwrap();
    let written = String::from_utf8(written).unwrap();
    assert_eq!(value(&written), value(TEXT), "{written}");
}
