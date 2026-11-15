//! One test per row of the table of how the game's jsoncpp reader differs from the crate's text
//! reader (`json::parse`, facet-json's rules kept) and typed reader, each over its
//! fixture in `tests/fixtures/json/`. A test asserts what the crate does with the input and, for
//! a recorded deviation, what vanilla does instead (in the test's comment and name).
//!
//! Row → fixture: comments → `comments.json`; BOM → `bom.json`; trailing comma →
//! `trailing_comma.json`; duplicate key → `duplicate_key.json`; content after the root →
//! `trailing_content.json`; scalar root / NUL → `scalar_root.json`, `nul_first_byte.json`;
//! `NaN` → `nan_literal.json`; number tag → `number_tag.json`; big integers →
//! `big_integer.json`; `-0` → `negative_zero.json`; `1e39` / `1e999` → `float_overflow.json`;
//! invalid UTF-8 → `invalid_utf8_string.json`, `invalid_utf8_comment.json`; number forms →
//! `number_forms.json`; nesting depth → `deep_129.json`; member order → `member_order.json`;
//! syntax error → `syntax_error.json`.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::parse::{Fault, FaultKind, Options, parse, parse_bytes};
use bedrock_addon_core::json::{MAX_JSON_DEPTH, TooDeep, Value, check_depth, strip_bom};
use bedrock_addon_core::schema::{FromJson, Schema, read_document, read_value};
use bedrock_addon_core::versioning::Reading;
use facet::Facet;
use indexmap::IndexMap;

fn fixture(name: &str) -> Vec<u8> {
    let path = format!("{}/tests/fixtures/json/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn text(name: &str) -> String {
    String::from_utf8(fixture(name)).unwrap()
}

fn value(text: &str) -> Result<Value, Fault> {
    parse(text, Options::default()).map(|parsed| parsed.value)
}

/// `text` as plain JSON (no comments).
fn plain(text: &str) -> Result<Value, Fault> {
    parse(text, Options::default().without_comments()).map(|parsed| parsed.value)
}

/// `text` read as `T` under `reading`: the value when no error was told.
fn typed<T: FromJson>(text: &str, reading: Reading) -> Option<T> {
    let parsed = parse(text, Options::default()).ok()?;
    let mut d = Diagnostics::default();
    let value = {
        let mut read = Read::new(reading, &mut d);
        read_document::<T>(&Node::parsed(&parsed, JsonPath::new()), &mut read)
    };
    value.filter(|_| !d.iter().any(|f| f.severity >= Severity::Error))
}

fn number(v: &Value) -> &facet_value::VNumber {
    v.as_number().unwrap()
}

fn array(v: &Value) -> &[Value] {
    v.as_array().unwrap().as_slice()
}

fn keys(v: &Value) -> Vec<&str> {
    v.as_object()
        .unwrap()
        .iter()
        .map(|(k, _)| k.as_str())
        .collect()
}

#[derive(Debug, Facet, Schema)]
struct One {
    a: u32,
}

// --- `//`, `/* */` comments ---------------------------------------------------------------

#[test]
fn comments_are_accepted_by_the_jsonc_entry_points() {
    let text = text("comments.json");
    let v = value(&text).unwrap();
    assert_eq!(keys(&v), ["format_version", "header"]);
    assert!(parse_bytes(text.as_bytes(), Options::default()).is_ok());
}

#[test]
fn comments_are_rejected_by_the_plain_entry_points() {
    let text = text("comments.json");
    assert!(plain(&text).is_err());
    assert!(parse_bytes(text.as_bytes(), Options::default().without_comments()).is_err());
}

// --- leading UTF-8 BOM ----------------------------------------------------------------------

#[test]
fn a_bom_is_rejected_by_facet_at_offset_0() {
    let bytes = fixture("bom.json");
    let error = parse_bytes(&bytes, Options::default()).unwrap_err();
    assert!(matches!(
        error.kind,
        FaultKind::UnexpectedChar { ch: 'ï', .. }
    ));
    assert_eq!(error.span.offset, 0);
}

#[test]
fn a_bom_is_skipped_through_strip_bom_as_vanilla_does() {
    let bytes = fixture("bom.json");
    let v = parse_bytes(strip_bom(&bytes), Options::default()).unwrap();
    assert_eq!(keys(&v), ["a"]);
}

// --- trailing comma -------------------------------------------------------------------------

/// Deviation: jsoncpp (the game's reader) rejects a trailing comma and keeps
/// the partial value; the crate accepts it silently at every entry point.
#[test]
fn trailing_commas_are_accepted_silently_deviation() {
    let text = text("trailing_comma.json");
    let v = value(&text).unwrap();
    assert_eq!(keys(&v), ["a", "b"]);
    assert_eq!(array(v.as_object().unwrap().get("a").unwrap()).len(), 2);
    assert!(plain(&text).is_ok());
}

#[test]
fn a_doubled_or_leading_comma_is_rejected() {
    assert!(value("[1,,2]").is_err());
    assert!(value("[,1]").is_err());
}

// --- duplicate key --------------------------------------------------------------------------

#[test]
fn a_duplicate_key_in_a_value_keeps_the_last_value_at_the_first_position() {
    let v = value(&text("duplicate_key.json")).unwrap();
    assert_eq!(keys(&v), ["a", "b"]);
    assert_eq!(
        number(v.as_object().unwrap().get("a").unwrap()).to_i64(),
        Some(3)
    );
}

#[test]
fn a_duplicate_key_in_a_map_keeps_the_last_value_at_the_first_position() {
    let map: IndexMap<String, u32> =
        read_value(&value(&text("duplicate_key.json")).unwrap()).unwrap();
    assert_eq!(
        map.into_iter().collect::<Vec<_>>(),
        [("a".to_owned(), 3), ("b".to_owned(), 2)]
    );
}

// --- content after the root -----------------------------------------------------------------

#[test]
fn content_after_the_root_is_ignored_silently() {
    let v = value(&text("trailing_content.json")).unwrap();
    assert_eq!(keys(&v), ["a"]);
    let typed: One = typed(&text("trailing_content.json"), Reading::DOCUMENT).unwrap();
    assert_eq!(typed.a, 1);
}

// --- scalar root; NUL first byte ------------------------------------------------------------

#[test]
fn a_scalar_root_is_accepted() {
    let v = value(&text("scalar_root.json")).unwrap();
    assert_eq!(number(&v).to_i64(), Some(42));
}

#[test]
fn a_nul_first_byte_is_a_syntax_error() {
    let error = parse_bytes(&fixture("nul_first_byte.json"), Options::default()).unwrap_err();
    assert!(matches!(
        error.kind,
        FaultKind::UnexpectedChar { ch: '\0', .. }
    ));
}

// --- NaN, Infinity literals -----------------------------------------------------------------

#[test]
fn nan_and_infinity_literals_are_rejected() {
    assert!(value(&text("nan_literal.json")).is_err());
    assert!(value(r#"{ "a": Infinity }"#).is_err());
    assert!(value(r#"{ "a": -Infinity }"#).is_err());
}

// --- number tag -----------------------------------------------------------------------------

#[test]
fn an_integer_token_is_tagged_integer_and_a_real_token_float() {
    let v = value(&text("number_tag.json")).unwrap();
    let items = array(&v);
    assert!(number(&items[0]).is_integer());
    for real in &items[1..] {
        assert!(number(real).is_float(), "{real:?}");
    }
    assert_eq!(number(&items[3]).to_f64(), Some(1000.0));
}

// --- integer above u64::MAX or below i64::MIN -----------------------------------------------

#[test]
fn an_integer_outside_64_bits_is_a_float_in_a_typed_f64() {
    let parsed: Vec<f64> = read_value(&value(&text("big_integer.json")).unwrap()).unwrap();
    assert_eq!(
        parsed,
        [18_446_744_073_709_551_616.0, -9_223_372_036_854_775_809.0]
    );
}

// --- -0, -0.0 -------------------------------------------------------------------------------

#[test]
fn negative_zero_integer_is_zero_and_negative_zero_real_keeps_its_sign() {
    let v = value(&text("negative_zero.json")).unwrap();
    let items = array(&v);
    assert!(number(&items[0]).is_integer());
    assert_eq!(number(&items[0]).to_i64(), Some(0));
    assert!(number(&items[1]).is_float());
    assert!(number(&items[1]).to_f64().unwrap().is_sign_negative());
}

// --- 1e999, 1e39 into f32 -------------------------------------------------------------------

#[derive(Debug, Facet, Schema)]
struct Floats {
    big: f32,
    huge: f32,
}

#[test]
fn an_out_of_range_real_into_f32_is_infinity_without_an_error() {
    // The plain reading reads it as the infinity it is (an information, no error).
    let parsed: Floats = typed(&text("float_overflow.json"), Reading::OWN).unwrap();
    assert_eq!(parsed.big, f32::INFINITY);
    assert_eq!(parsed.huge, f32::INFINITY);
}

// --- invalid UTF-8 ---------------------------------------------------------------------------

#[test]
fn invalid_utf8_in_a_string_is_an_error_with_a_span() {
    let error = parse_bytes(&fixture("invalid_utf8_string.json"), Options::default()).unwrap_err();
    assert!(matches!(error.kind, FaultKind::InvalidUtf8));
    assert!(error.span.len > 0);
}

#[test]
fn invalid_utf8_in_a_comment_is_accepted() {
    let v = parse_bytes(&fixture("invalid_utf8_comment.json"), Options::default()).unwrap();
    assert_eq!(keys(&v), ["a"]);
}

// --- escapes, control characters ------------------------------------------------------------

#[test]
fn a_lone_surrogate_escape_is_invalid_utf8() {
    let error = value(r#"["\ud800"]"#).unwrap_err();
    assert!(matches!(error.kind, FaultKind::InvalidUtf8), "{error:?}");
}

// --- 01, +1, .5, 0x10, JSON5 quotes / bare keys ---------------------------------------------

#[test]
fn a_leading_zero_reads_as_the_number() {
    let v = value(&text("number_forms.json")).unwrap();
    assert_eq!(number(&array(&v)[0]).to_i64(), Some(1));
}

#[test]
fn json5_number_forms_quotes_and_bare_keys_are_rejected() {
    for input in ["[+1]", "[.5]", "[0x10]", "['a']", "{a:1}"] {
        assert!(value(input).is_err(), "{input}");
    }
}

#[test]
fn a_root_hex_number_reads_zero_and_ignores_the_rest() {
    assert_eq!(number(&value("0x10").unwrap()).to_i64(), Some(0));
}

// --- nesting depth ---------------------------------------------------------------------------

#[test]
fn a_document_deeper_than_the_limit_is_rejected_by_check_depth() {
    assert_eq!(
        check_depth(&fixture("deep_129.json"), MAX_JSON_DEPTH),
        Err(TooDeep { offset: 128 })
    );
}

// --- source positions ------------------------------------------------------------------------

// --- object member order ---------------------------------------------------------------------

#[test]
fn object_members_keep_document_order() {
    let text = text("member_order.json");
    assert_eq!(keys(&value(&text).unwrap()), ["b", "a", "c"]);
    let map: IndexMap<String, u32> = read_value(&value(&text).unwrap()).unwrap();
    assert_eq!(map.keys().collect::<Vec<_>>(), ["b", "a", "c"]);
}

// --- syntax error mid-document ---------------------------------------------------------------

/// Accepted loss: jsoncpp keeps the partial value before a syntax error; the crate returns
/// only the fault, an `UnexpectedToken` with a span.
#[test]
fn a_syntax_error_mid_document_yields_no_partial_value_deviation() {
    let text = text("syntax_error.json");
    let error = value(&text).unwrap_err();
    assert!(
        matches!(error.kind, FaultKind::UnexpectedToken { .. }),
        "{error:?}"
    );
    assert_eq!(Some(error.span.offset as usize), text.rfind('}'));
}

// --- typed coercions (from text) -------------------------------------------------------------

#[test]
fn a_fraction_into_an_integer_a_bool_into_a_string_and_a_scalar_into_a_list_are_rejected() {
    #[derive(Debug, Facet, Schema)]
    struct S {
        s: String,
    }
    #[derive(Debug, Facet, Schema)]
    struct B {
        b: bool,
    }
    #[derive(Debug, Facet, Schema)]
    struct L {
        l: Vec<u32>,
    }
    let strict = Reading::DOCUMENT.strict();
    assert!(typed::<One>(r#"{ "a": 1.5 }"#, strict).is_none());
    assert!(typed::<S>(r#"{ "s": true }"#, strict).is_none());
    assert!(typed::<B>(r#"{ "b": 1 }"#, strict).is_none());
    assert!(typed::<L>(r#"{ "l": 1 }"#, strict).is_none());
}

// --- serialising a Value ---------------------------------------------------------------------
