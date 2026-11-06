//! Per-entry isolation without a tree: a map whose entries fail one by one is held as
//! `IndexMap<String, RawJson>`, each entry is parsed from its own slice with `parse_at`, and a
//! failure is reported at file offsets.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

// The crate's one protocol for cost tests, and this binary's one lock.
use bedrock_addon_core::test_cost as cost;

use bedrock_addon_core::json::{JsonSlice, parse_at};
use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;
use std::fmt::Write as _;

/// A document shell holding its components as raw slices of the text.
#[derive(Debug, Facet)]
struct Shell<'a> {
    components: IndexMap<String, RawJson<'a>>,
}

#[derive(Debug, Facet)]
struct Friction {
    value: f32,
}

#[derive(Debug, Facet)]
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

fn shell(document: JsonSlice<'_>) -> Shell<'_> {
    parse_at::<Shell<'_>>(document).unwrap()
}

#[test]
fn a_raw_entry_borrows_its_text_and_knows_its_file_offset() {
    let document = JsonSlice::new(DOCUMENT);
    let shell = shell(document);
    for (key, raw) in &shell.components {
        let slice = document.nested(raw).unwrap();
        assert_eq!(
            &DOCUMENT[slice.offset..slice.offset + slice.text.len()],
            raw.as_str(),
            "{key}"
        );
    }
    let friction = document
        .nested(&shell.components["minecraft:friction"])
        .unwrap();
    assert_eq!(friction.text, r#"{ /* kept */ "value": 0.4 }"#);
    assert_eq!(friction.offset, DOCUMENT.find("{ /* kept */").unwrap());
}

#[test]
fn a_nested_slice_of_a_nested_slice_adds_the_offsets() {
    // The document sits at offset 8 of a larger file.
    let file = format!(r#"{{"doc": {DOCUMENT}}}"#);
    let outer = JsonSlice {
        text: &file[8..file.len() - 1],
        offset: 8,
    };
    let shell = shell(outer);
    let light = outer.nested(&shell.components["minecraft:light"]).unwrap();
    assert_eq!(light.offset, file.find(r#"{ "level": "bright" }"#).unwrap());
}

#[test]
fn an_owned_or_foreign_raw_json_has_no_offset() {
    let document = JsonSlice::new(DOCUMENT);
    assert!(
        document
            .nested(&RawJson::from_owned("{}".to_owned()))
            .is_none()
    );
    let other = String::from(r#"{"a": 1}"#);
    assert!(document.nested(&RawJson::new(&other)).is_none());
}

#[test]
fn a_good_entry_parses_from_its_slice() {
    let document = JsonSlice::new(DOCUMENT);
    let shell = shell(document);
    let slice = document
        .nested(&shell.components["minecraft:friction"])
        .unwrap();
    assert_eq!(parse_at::<Friction>(slice).unwrap().value, 0.4);
}

#[test]
fn a_failing_entry_is_reported_at_its_file_offset() {
    let document = JsonSlice::new(DOCUMENT);
    let shell = shell(document);
    let slice = document
        .nested(&shell.components["minecraft:light"])
        .unwrap();
    let error = parse_at::<Light>(slice).unwrap_err();
    // facet's span covers the member key; rebased, it points into the file.
    let key = slice.offset + slice.text.find("\"level\"").unwrap();
    assert_eq!(error.span.offset as usize, key);
    assert_eq!(error.span.len as usize, "\"level\"".len());
    // The path is relative to the slice; the caller prefixes the component key.
    assert_eq!(error.path.to_string(), "level");
}

#[derive(Debug, Facet)]
struct Layers {
    layers: Vec<IndexMap<String, Light>>,
    #[facet(default)]
    renamed: Option<Renamed>,
}

#[derive(Debug, Facet)]
struct Renamed {
    #[facet(rename = "minecraft:level")]
    level: u8,
}

#[test]
fn a_failing_member_is_named_by_its_json_path() {
    let text = r#"{ "layers": [{ "a": { "level": 1 } }, { "b": { "level": "x" } }] }"#;
    let error = parse_at::<Layers>(JsonSlice::new(text)).unwrap_err();
    assert_eq!(error.path.to_string(), "layers/1/b/level");
    let error =
        parse_at::<Layers>(JsonSlice::new(r#"{ "layers": [], "renamed": {} }"#)).unwrap_err();
    assert_eq!(error.path.to_string(), "renamed/minecraft:level");
}

#[test]
fn an_error_without_a_span_falls_back_to_the_slice() {
    let document = JsonSlice::new(DOCUMENT);
    let shell = shell(document);
    let slice = document.nested(&shell.components["minecraft:dim"]).unwrap();
    let error = parse_at::<Light>(slice).unwrap_err();
    assert!(matches!(
        error.error.kind,
        facet_format::DeserializeErrorKind::NumberOutOfRange { .. }
    ));
    assert_eq!(error.span.offset as usize, slice.offset);
    assert_eq!(error.span.len as usize, slice.text.len());
}

#[test]
fn the_other_entries_survive_a_failing_one() {
    let document = JsonSlice::new(DOCUMENT);
    let shell = shell(document);
    let parsed: Vec<bool> = shell
        .components
        .values()
        .map(|raw| parse_at::<Light>(document.nested(raw).unwrap()).is_ok())
        .collect();
    assert_eq!(parsed, [false, false, false]);
    let friction = document
        .nested(&shell.components["minecraft:friction"])
        .unwrap();
    assert!(parse_at::<Friction>(friction).is_ok());
}

#[test]
fn a_whole_file_slice_reports_facets_offsets_unchanged() {
    let text = r#"{ "level": true }"#;
    let error = parse_at::<Light>(JsonSlice::new(text)).unwrap_err();
    assert_eq!(error.span.offset, 2);
}

// ---- a bad element of an array is at its own index ----

#[derive(Debug, Facet)]
struct Offsets {
    view_offset: [bedrock_addon_core::json::StrictFloat<f32>; 2],
    list: Option<Vec<bedrock_addon_core::json::StrictFloat<f32>>>,
    #[facet(default)]
    nested: Option<Nested>,
}

#[derive(Debug, Facet)]
struct Nested {
    pair: [bedrock_addon_core::json::StrictInt<u8>; 2],
}

#[test]
fn a_bad_element_of_a_fixed_array_is_reported_at_its_index() {
    for (text, at) in [
        (r#"{"view_offset": [1, 1e40]}"#, "view_offset/1"),
        (r#"{"view_offset": ["a", 1]}"#, "view_offset/0"),
        (r#"{"view_offset": [1, 2], "list": [1, 2, "x"]}"#, "list/2"),
        (
            r#"{"view_offset": [1, 2], "nested": {"pair": [1, 300]}}"#,
            "nested/pair/1",
        ),
    ] {
        let error = parse_at::<Offsets>(JsonSlice::new(text)).unwrap_err();
        assert_eq!(error.path.to_string(), at, "{text}: {}", error.error);
    }
}

// ---- the same below an untagged enum (a block's `selection_box`) ----

#[derive(Debug, Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(dead_code, reason = "only the failures are read")]
enum BoxOrFlag {
    Flag(bool),
    Box(Corner),
}

#[derive(Debug, Facet)]
struct Corner {
    #[facet(default)]
    origin: Option<[bedrock_addon_core::json::StrictFloat<f32>; 3]>,
    #[facet(default)]
    size: Option<[bedrock_addon_core::json::StrictFloat<f32>; 3]>,
}

#[test]
fn a_bad_element_of_a_fixed_array_below_an_untagged_enum_is_reported_at_its_index() {
    for (text, at) in [
        (r#"{"origin": ["7", 0, -8]}"#, "origin/0"),
        (r#"{"origin": [-8, "7", -8]}"#, "origin/1"),
        (r#"{"origin": [-8, 0, "7"]}"#, "origin/2"),
        (
            r#"{"origin": [-8, 0, -8], "size": [16, 1e39, 16]}"#,
            "size/1",
        ),
        (
            r#"{"origin": [-8, 0, -8], "size": [16, 16, true]}"#,
            "size/2",
        ),
        (r#"{"size": [16, null, 16]}"#, "size/1"),
    ] {
        let error = parse_at::<BoxOrFlag>(JsonSlice::new(text)).unwrap_err();
        assert_eq!(error.path.to_string(), at, "{text}: {:?}", error.error);
    }
}

// ---- a map level given an array is a type error ----

#[derive(Debug, Facet)]
struct Maps<'a> {
    #[facet(default)]
    map: Option<IndexMap<String, u8>>,
    #[facet(default)]
    raw: Option<IndexMap<String, RawJson<'a>>>,
    #[facet(default)]
    list: Option<Vec<IndexMap<String, u8>>>,
    #[facet(default)]
    nested: Option<IndexMap<String, IndexMap<String, u8>>>,
    #[facet(default)]
    plain: Option<Vec<u8>>,
}

#[test]
fn an_empty_array_for_a_map_level_is_a_type_error_at_that_level() {
    for (text, at) in [
        (r#"{"map": []}"#, "map"),
        (r#"{"raw": [ ]}"#, "raw"),
        (r#"{"list": [{}, []]}"#, "list/1"),
        (r#"{"nested": {"a": {}, "b": []}}"#, "nested/b"),
    ] {
        let error = parse_at::<Maps<'_>>(JsonSlice::new(text)).unwrap_err();
        assert_eq!(error.path.to_string(), at, "{text}");
        assert!(
            matches!(
                bedrock_addon_core::diagnostic::code_for_deserialize_error_kind(
                    &error.error.kind,
                    !error.path.is_empty(),
                    false
                ),
                bedrock_addon_core::diagnostic::DiagnosticCode::InvalidType { .. }
            ),
            "{text}: {}",
            error.error
        );
        // The span is the array's own text.
        let start = error.span.offset as usize;
        let spanned = &text[start..start + error.span.len as usize];
        assert!(
            spanned.starts_with('[') && spanned.ends_with(']'),
            "{text}: {spanned}"
        );
    }
    // At the root too, with an empty path.
    let error = parse_at::<IndexMap<String, u8>>(JsonSlice::new("[]")).unwrap_err();
    assert!(error.path.is_empty());
}

#[test]
fn an_object_or_an_array_where_one_belongs_is_fine() {
    for text in [
        r#"{"map": {}, "raw": {"a": []}, "list": [], "nested": {"a": {}}, "plain": []}"#,
        r#"{"map": {"a": 1}}"#,
        r#"{"plain": [], "map": {"[]": 1}}"#,
    ] {
        assert!(parse_at::<Maps<'_>>(JsonSlice::new(text)).is_ok(), "{text}");
    }
    // `null` for an optional map is `None`: an absent member, as jsoncpp's `isNull` reads it.
    let maps = parse_at::<Maps<'_>>(JsonSlice::new(r#"{"map": null}"#)).unwrap();
    assert!(maps.map.is_none());
}

// ---- the text is read a second time only for a map below the root ----

/// A map of 4000 raw members, each an object of some depth; the last one holds `element`.
fn raw_members(element: &str) -> String {
    let mut text = String::from("{");
    for index in 0..4000 {
        write!(
            text,
            r#""minecraft:member_{index}": {{"value": [1, 2.5, "three"], "nested": {{"flag": true, "name": "text", "list": [{{"a": 1}}, {{"b": 2}}]}}}}, "#
        )
        .unwrap();
    }
    write!(text, r#""last": {{"list": {element}}}}}"#).unwrap();
    text
}

/// What one read of `text` as a map of raw members and of `list` as a list of raw elements
/// takes.
fn raw_read(text: &str, list: &str) -> std::time::Duration {
    cost::time_of(|| {
        let map = parse_at::<IndexMap<String, RawJson<'_>>>(JsonSlice::new(text)).unwrap();
        let list = parse_at::<Vec<RawJson<'_>>>(JsonSlice::new(list)).unwrap();
        (map.len(), list.len())
    })
}

/// An empty array somewhere in a member is no reason to read the whole text as a value: a
/// map of raw members holds no map below itself, and a list of raw elements none at all, so
/// an array for a map can only be the root. Two times of one read compared under the lock of
/// the cost tests, not a bound on growth: the text with `[]` in its last member against the
/// same text with `[0]` there, each the best of seven taken in turn so that both see the
/// same neighbours, required to stay under twice it (reading the text a second time as a
/// `Value` would take about three and a half times the first), and timed once more before a
/// time over the bound is believed.
#[test]
fn raw_members_with_an_empty_array_are_read_once() {
    let (with, without) = (raw_members("[]"), raw_members("[0]"));
    let (with_list, without_list) = (format!("[{with}, []]"), format!("[{without}, [0]]"));
    let mut ratios = Vec::new();
    for _attempt in 0..2 {
        let (empty, filled) = cost::alone_for(|| {
            let mut best = (std::time::Duration::MAX, std::time::Duration::MAX);
            for _ in 0..7 {
                best.0 = best.0.min(raw_read(&with, &with_list));
                best.1 = best.1.min(raw_read(&without, &without_list));
            }
            best
        });
        let ratio = empty.as_secs_f64() / filled.as_secs_f64().max(1e-9);
        eprintln!(
            "cost: raw members with an empty array {empty:?}, without {filled:?}, ratio {ratio:.2}"
        );
        if ratio < 2.0 {
            return;
        }
        ratios.push(ratio);
    }
    panic!("a text with an empty array took {ratios:?} times the same text without one, twice");
}

/// What the shape decides without the second read is what the second read decided.
#[test]
fn an_array_for_a_map_is_found_whatever_the_shape_holds_below_it() {
    // The root of a map of raw members: an array there is the type error, an object is fine
    // whatever its members hold, behind white space and comments too.
    for text in ["[]", " [ ]", "/* c */ []", "// c\n[]"] {
        let error = parse_at::<IndexMap<String, RawJson<'_>>>(JsonSlice::new(text)).unwrap_err();
        assert!(error.path.is_empty(), "{text}");
        // The root's span is the slice.
        assert_eq!(
            (error.span.offset, error.span.len as usize),
            (0, text.len())
        );
    }
    for text in [
        r#"{"a": []}"#,
        r#" {"a": {"b": []}, "c": [[]]}"#,
        r#"/* [] */ {"a": 1}"#,
    ] {
        assert!(
            parse_at::<IndexMap<String, RawJson<'_>>>(JsonSlice::new(text)).is_ok(),
            "{text}"
        );
    }
    // A shape without a map: no array is one for a map.
    assert!(parse_at::<Vec<RawJson<'_>>>(JsonSlice::new("[[], {}]")).is_ok());
    assert!(parse_at::<Vec<Vec<u8>>>(JsonSlice::new("[[], []]")).is_ok());
    // A map below a map of maps, and a struct that holds itself through a list.
    let error = parse_at::<IndexMap<String, IndexMap<String, u8>>>(JsonSlice::new(r#"{"a": []}"#))
        .unwrap_err();
    assert_eq!(error.path.to_string(), "a");
    let error = parse_at::<Tree>(JsonSlice::new(
        r#"{"children": [{"children": [], "names": []}]}"#,
    ))
    .unwrap_err();
    assert_eq!(error.path.to_string(), "children/0/names");
    assert!(parse_at::<Twigs>(JsonSlice::new(r#"{"twigs": [{"twigs": []}]}"#)).is_ok());
}

/// A struct that holds itself, with a map.
#[derive(Debug, Facet)]
struct Tree {
    #[facet(default)]
    children: Vec<Tree>,
    #[facet(default)]
    names: IndexMap<String, u8>,
}

/// A struct that holds itself, without one.
#[derive(Debug, Facet)]
struct Twigs {
    #[facet(default)]
    twigs: Vec<Twigs>,
}

/// An array that holds nothing but white space and comments is an empty array too: for a
/// map it is the same type error.
#[test]
fn an_array_of_comments_alone_for_a_map_level_is_a_type_error_at_that_level() {
    for text in [
        "[ /* c */ ]",
        "[/**/]",
        "[ // c\n ]",
        "[//\n]",
        "[ \t\r\n /* a */ // b\n /* c */ \n]",
        "/* before */ [ /* c */ ] // after",
    ] {
        for error in [
            parse_at::<IndexMap<String, u8>>(JsonSlice::new(text)).map(|_| ()),
            parse_at::<IndexMap<String, RawJson<'_>>>(JsonSlice::new(text)).map(|_| ()),
        ] {
            let error = error.expect_err(text);
            assert!(error.path.is_empty(), "{text}");
            assert!(
                matches!(
                    error.error.kind,
                    facet_format::DeserializeErrorKind::TypeMismatch { .. }
                ),
                "{text}: {}",
                error.error
            );
        }
    }
    for (text, at) in [
        (r#"{"map": [ /* c */ ]}"#, "map"),
        ("{\"raw\": [// c\n]}", "raw"),
        (r#"{"list": [{}, [/* c */]]}"#, "list/1"),
        (
            "{\"nested\": {\"a\": {}, \"b\": [ // c\n /* d */ ]}}",
            "nested/b",
        ),
    ] {
        let error = parse_at::<Maps<'_>>(JsonSlice::new(text)).expect_err(text);
        assert_eq!(error.path.to_string(), at, "{text}");
        let start = error.span.offset as usize;
        let spanned = &text[start..start + error.span.len as usize];
        assert!(
            spanned.starts_with('[') && spanned.ends_with(']'),
            "{text}: {spanned}"
        );
    }
    // Where a list belongs, and in a key or a string, such an array is no fault.
    for text in [
        r#"{"plain": [ /* c */ ], "list": [// c
        ]}"#,
        r#"{"map": {"[ /* c */ ]": 1}}"#,
        r#"{"raw": {"a": [ /* c */ ], "b": "[//]"}}"#,
    ] {
        assert!(parse_at::<Maps<'_>>(JsonSlice::new(text)).is_ok(), "{text}");
    }
}

/// A list of two numbers with one comment between them that holds `[/*` (in a block comment)
/// or `[//` (in a line comment) `units` times: no empty array, and a `[` before a comment
/// at every third byte.
fn brackets_in_a_comment(units: usize) -> [String; 2] {
    [
        format!("[1, /*{}*/ 2]", "[/*".repeat(units)),
        format!("[1, //{}\n 2]", "[//".repeat(units)),
    ]
}

/// The search for an empty array goes over the text once: a comment is not searched for its
/// end again for every `[` that stands before it. By the crate's one protocol, for a block
/// comment and for a line comment: four times the text in under eight times the time.
#[test]
fn the_search_for_an_empty_array_is_linear_in_a_comment_of_brackets() {
    let read = |text: &String| {
        assert_eq!(
            parse_at::<Vec<u8>>(JsonSlice::new(text)).unwrap(),
            [1, 2],
            "the comment is no element"
        );
    };
    cost::assert_each_linear_at(
        UNITS_OF_BRACKETS,
        brackets_in_a_comment,
        &[
            ("`[/*` in a block comment, times", &|texts: &[String; 2]| {
                read(&texts[0]);
            }),
            ("`[//` in a line comment, times", &|texts: &[String; 2]| {
                read(&texts[1]);
            }),
        ],
    );
}

/// How many `[/*` and `[//` the smaller text of the cost test holds.
const UNITS_OF_BRACKETS: usize = 500_000;

#[derive(Debug, Facet)]
struct Texts {
    a: String,
    #[facet(default)]
    map: IndexMap<String, u8>,
    b: String,
}

/// A `[` and a comment's marks inside a string are text: the empty array that stands between
/// two such strings is found, and what only looks like one inside a string is none.
#[test]
fn an_empty_array_is_found_between_strings_that_hold_brackets_and_comment_marks() {
    for text in [
        r#"{"a": "[/*", "map": [], "b": "*/"}"#,
        r#"{"a": "[/*", "map": [ /* c */ ], "b": "*/"}"#,
        "{\"a\": \"[//\", \"map\": [ // c\n ], \"b\": \"//\"}",
        r#"{"a": "\"[/*", "map": [/**/], "b": "\\"}"#,
        r#"{"a": "/*", "map": [ /* " */ ], "b": "[ // ]"}"#,
        r#"/* " [ */ {"a": "x", "map": [ /* ] */ ], "b": "y"}"#,
    ] {
        let error = parse_at::<Texts>(JsonSlice::new(text)).expect_err(text);
        assert_eq!(error.path.to_string(), "map", "{text}");
        assert!(
            matches!(
                error.error.kind,
                facet_format::DeserializeErrorKind::TypeMismatch { .. }
            ),
            "{text}: {}",
            error.error
        );
    }
    for text in [
        r#"{"a": "[ /* c */ ]", "map": {}, "b": "[]"}"#,
        r#"{"a": "[/*", "map": {"[//": 1}, "b": "*/ ]"}"#,
        "{\"a\": \"[\", /* [ */ \"map\": {}, // [\n \"b\": \"]\"}",
    ] {
        let read = parse_at::<Texts>(JsonSlice::new(text)).unwrap();
        assert!(read.map.len() <= 1, "{text}");
    }
    // The same texts for a map of raw members hold no array for a map.
    let raw = r#"{"a": "[/*", "b": [ /* c */ ], "c": "*/"}"#;
    assert_eq!(
        parse_at::<IndexMap<String, RawJson<'_>>>(JsonSlice::new(raw))
            .unwrap()
            .len(),
        3
    );
}

// ---- the span follows the path ----

#[derive(Debug, Facet)]
struct Pairs {
    pairs: Vec<[bedrock_addon_core::json::StrictInt<u8>; 2]>,
}

#[test]
fn the_span_of_a_bad_element_is_that_element() {
    for (text, at) in [
        (r#"{"pairs": [[1,2],[7,300]]}"#, "pairs/1/1"),
        (r#"{"pairs": [[1,2],[300,7]]}"#, "pairs/1/0"),
        (r#"{"pairs": [[300,2],[7,7]]}"#, "pairs/0/0"),
    ] {
        let error = parse_at::<Pairs>(JsonSlice::new(text)).unwrap_err();
        assert_eq!(error.path.to_string(), at, "{text}");
        let start = error.span.offset as usize;
        assert_eq!(
            &text[start..start + error.span.len as usize],
            "300",
            "{text}"
        );
    }
    // Within a slice of a larger file the span is the file's.
    let file = r#"{"x": {"pairs": [[1,2],[7,300]]}}"#;
    let start = file.find("{\"pairs").unwrap();
    let end = file.rfind('}').unwrap();
    let slice = JsonSlice {
        text: &file[start..end],
        offset: start,
    };
    let error = parse_at::<Pairs>(slice).unwrap_err();
    let at = error.span.offset as usize;
    assert_eq!(&file[at..at + error.span.len as usize], "300");
}

// ---- a span of no length marks no text ----

#[derive(Debug, Facet)]
#[facet(transparent)]
struct Bare(bedrock_addon_core::json::StrictFloat<f32>);

#[derive(Debug, Facet)]
struct Holder {
    #[facet(default)]
    level: Option<bedrock_addon_core::json::StrictInt<u8>>,
}

/// A strict wrapper's refusal of a value that is the slice itself comes from facet without
/// the token's extent. The finding is on the value the path names: the whole slice.
#[test]
fn a_refusal_without_an_extent_has_the_span_of_the_value_it_names() {
    let document = r#"{"a": 1e39, "b": "x", "c": {"level": "x"}}"#;
    let shell: IndexMap<String, RawJson<'_>> =
        bedrock_addon_core::json::from_str_borrowed_jsonc(document).unwrap();
    let whole = JsonSlice::new(document);
    let under = |span: bedrock_addon_core::diagnostic::Span| {
        &document[span.offset as usize..(span.offset + span.len) as usize]
    };
    for (key, token) in [("a", "1e39"), ("b", r#""x""#)] {
        let slice = whole.nested(&shell[key]).unwrap();
        let error = parse_at::<Bare>(slice).unwrap_err();
        assert_eq!(under(error.span), token, "{key}: {:?}", error.error);
    }
    // A member's refusal has facet's span, which has a length: the member's key.
    let slice = whole.nested(&shell["c"]).unwrap();
    let error = parse_at::<Holder>(slice).unwrap_err();
    assert_eq!(error.path.to_string(), "level");
    assert_eq!(under(error.span), r#""level""#);
}
