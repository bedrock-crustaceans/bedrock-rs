//! The skip-field convention: facet rc.7 still matches a JSON
//! member against a `#[facet(skip)]` field's name (`FieldLookup` takes every field, skipped or
//! not), so every skipped field is renamed to its name behind a NUL,
//! `#[facet(skip, rename = "\u{0}extra")]`, a key no document writes. A member named like the
//! field is then an unknown member like any other: reported by `unknown_members` (the family's
//! `UnrecognizedField`), kept by `keep_unknown_members` (in the document's table, for a struct
//! marked with `extra: Extra`) and written back.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests read fixtures and call facet-json directly"
)]
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::{Extra, Unknown};
use bedrock_addon_core::json::{Value, keep_unknown_members, unknown_members, write_json_with};
use facet::Facet;
use indexmap::IndexMap;

/// A struct whose skipped fields follow the convention.
#[derive(Debug, Default, Facet)]
struct Marked {
    level: u8,
    #[facet(skip, rename = "\u{0}converted")]
    converted: bool,
    #[facet(skip, rename = "\u{0}side")]
    side: IndexMap<String, Value>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

/// A struct with a skipped side-car without the convention: facet binds it by name.
#[derive(Debug, Default, Facet)]
struct Plain {
    level: u8,
    #[facet(skip)]
    side: IndexMap<String, Value>,
}

fn value(text: &str) -> Value {
    facet_json::from_str_borrowed_jsonc(text).unwrap()
}

/// The unknown members' paths as written.
fn unknown_paths<'a, T: Facet<'a>>(text: &str) -> Vec<String> {
    unknown_members(T::SHAPE, &value(text))
        .into_iter()
        .map(|(path, _)| path.to_string())
        .collect()
}

#[test]
fn facet_binds_a_member_to_a_skipped_field_of_its_name() {
    // The reason for the convention. When facet stops doing this, this test fails and the
    // convention may go.
    let scalar = facet_json::from_str_borrowed_jsonc::<Plain>(r#"{"level": 1, "side": 1}"#);
    assert!(scalar.is_err(), "{scalar:?}");
    // An object is read INTO the side-car.
    let object: Plain =
        facet_json::from_str_borrowed_jsonc(r#"{"level": 1, "side": {"a": 2}}"#).unwrap();
    assert_eq!(object.side.get("a"), Some(&Value::from(2i64)));
}

#[test]
fn a_skipped_field_renamed_behind_a_nul_never_binds_a_member_of_its_name() {
    for text in [
        r#"{"level": 1, "extra": 1}"#,
        r#"{"level": 1, "extra": {"a": 2}}"#,
        r#"{"level": 1, "converted": true}"#,
        r#"{"level": 1, "side": {"a": 2}}"#,
    ] {
        let marked: Marked = facet_json::from_str_borrowed_jsonc(text)
            .unwrap_or_else(|error| panic!("{text}: {error:?}"));
        assert_eq!(marked.level, 1, "{text}");
        assert!(!marked.converted, "{text}");
        assert!(marked.side.is_empty(), "{text}");
    }
}

#[test]
fn a_member_named_like_a_skipped_field_is_an_unknown_member_kept_and_written_back() {
    const TEXT: &str = r#"{"level": 1, "converted": true, "extra": {"a": 2}}"#;
    assert_eq!(unknown_paths::<Marked>(TEXT), ["converted", "extra"]);
    let marked: Marked = facet_json::from_str_borrowed_jsonc(TEXT).unwrap();
    let mut table = Unknown::new();
    let kept = keep_unknown_members(&marked, &value(TEXT), &mut table);
    assert_eq!(kept.len(), 2);
    assert!(!marked.converted, "the skipped field keeps its own value");
    let root = JsonPath::new();
    assert_eq!(table.get(&root, "converted"), Some(&Value::from(true)));
    assert_eq!(table.get(&root, "extra"), Some(&value(r#"{"a": 2}"#)));
    let mut written = Vec::new();
    write_json_with(
        &marked,
        &table,
        &mut written,
        bedrock_addon_core::json::WriteStyle::default(),
    )
    .unwrap();
    let written = String::from_utf8(written).unwrap();
    assert_eq!(value(&written), value(TEXT), "{written}");
}
