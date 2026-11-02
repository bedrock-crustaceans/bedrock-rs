//! One test per row of the table of how the game's jsoncpp reader differs from facet-json, each
//! over its fixture in `tests/fixtures/json/`. A test asserts what facet-json / facet-value
//! `0.50.0-rc.7` do with the input and, for a recorded deviation, what vanilla does instead (in the
//! test's comment and name), so a facet release that changes a row fails here first.
//!
//! Row → fixture: comments → `comments.json`; BOM → `bom.json`; trailing comma →
//! `trailing_comma.json`; duplicate key → `duplicate_key.json`; content after the root →
//! `trailing_content.json`; scalar root / NUL → `scalar_root.json`, `nul_first_byte.json`;
//! `NaN` → `nan_literal.json`; number tag → `number_tag.json`; big integers →
//! `big_integer.json`; `-0` → `negative_zero.json`; `1e39` / `1e999` → `float_overflow.json`;
//! invalid UTF-8 → `invalid_utf8_string.json`, `invalid_utf8_comment.json`; escapes →
//! `escapes.json`; number forms → `number_forms.json`; nesting depth → `deep_129.json`;
//! source positions → `typed_error.json`; member order → `member_order.json`; syntax error →
//! `syntax_error.json`; typed coercions → `coercions.json`; serialising a `Value` →
//! `value_write.json`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon_core::json::{MAX_JSON_DEPTH, TooDeep, Value, check_depth, strip_bom};
use facet::Facet;
use facet_format::DeserializeErrorKind;
use indexmap::IndexMap;

fn fixture(name: &str) -> Vec<u8> {
    let path = format!("{}/tests/fixtures/json/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn text(name: &str) -> String {
    String::from_utf8(fixture(name)).unwrap()
}

fn value(text: &str) -> Result<Value, facet_json::DeserializeError> {
    facet_json::from_str_borrowed_jsonc::<Value>(text)
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

#[derive(Debug, Facet)]
struct One {
    a: u32,
}

// --- `//`, `/* */` comments ---------------------------------------------------------------

#[test]
fn comments_are_accepted_by_the_jsonc_entry_points() {
    let text = text("comments.json");
    let v = value(&text).unwrap();
    assert_eq!(keys(&v), ["format_version", "header"]);
    assert!(facet_json::from_slice_borrowed_jsonc::<Value>(text.as_bytes()).is_ok());
}

#[test]
fn comments_are_rejected_by_the_plain_entry_points() {
    let text = text("comments.json");
    assert!(facet_json::from_str_borrowed::<Value>(&text).is_err());
    assert!(facet_json::from_slice_borrowed::<Value>(text.as_bytes()).is_err());
}

// --- leading UTF-8 BOM ----------------------------------------------------------------------

#[test]
fn a_bom_is_rejected_by_facet_at_offset_0() {
    let bytes = fixture("bom.json");
    let error = facet_json::from_slice_borrowed_jsonc::<Value>(&bytes).unwrap_err();
    assert!(matches!(
        error.kind,
        DeserializeErrorKind::UnexpectedChar { ch: 'ï', .. }
    ));
    assert_eq!(error.span.map(|s| s.offset), Some(0));
}

#[test]
fn a_bom_is_skipped_through_strip_bom_as_vanilla_does() {
    let bytes = fixture("bom.json");
    let v = facet_json::from_slice_borrowed_jsonc::<Value>(strip_bom(&bytes)).unwrap();
    assert_eq!(keys(&v), ["a"]);
}

// --- trailing comma -------------------------------------------------------------------------

/// Deviation: jsoncpp (the game's reader) rejects a trailing comma and keeps
/// the partial value; facet accepts it silently at every entry point.
#[test]
fn trailing_commas_are_accepted_silently_deviation() {
    let text = text("trailing_comma.json");
    let v = value(&text).unwrap();
    assert_eq!(keys(&v), ["a", "b"]);
    assert_eq!(array(v.as_object().unwrap().get("a").unwrap()).len(), 2);
    assert!(facet_json::from_str_borrowed::<Value>(&text).is_ok());
    assert!(facet_json::from_str::<Value>(&text).is_ok());
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
        facet_json::from_str_borrowed_jsonc(&text("duplicate_key.json")).unwrap();
    assert_eq!(
        map.into_iter().collect::<Vec<_>>(),
        [("a".to_owned(), 3), ("b".to_owned(), 2)]
    );
}

#[derive(Debug, Facet)]
struct TwoFields {
    a: u32,
    b: u32,
}

/// Deviation: the game takes the last value of a duplicate member, but a duplicate member of a
/// struct without a flatten field is facet's `duplicate field` error with a span; the crate
/// reports it as `MalformedInput`.
#[test]
fn a_duplicate_typed_member_is_an_error_with_a_span_deviation() {
    let error =
        facet_json::from_str_borrowed_jsonc::<TwoFields>(&text("duplicate_key.json")).unwrap_err();
    assert!(
        matches!(&error.kind, DeserializeErrorKind::DuplicateField { field, .. } if field == "a")
    );
    assert!(error.span.is_some());
}

#[derive(Debug, Facet)]
struct WithFlatten {
    a: u32,
    #[facet(flatten)]
    rest: IndexMap<String, Value>,
}

#[test]
fn a_duplicate_member_of_a_struct_with_a_flatten_field_keeps_the_last_value() {
    let parsed: WithFlatten =
        facet_json::from_str_borrowed_jsonc(&text("duplicate_key.json")).unwrap();
    assert_eq!(parsed.a, 3);
    assert_eq!(parsed.rest.keys().collect::<Vec<_>>(), ["b"]);
}

// --- content after the root -----------------------------------------------------------------

#[test]
fn content_after_the_root_is_ignored_silently() {
    let v = value(&text("trailing_content.json")).unwrap();
    assert_eq!(keys(&v), ["a"]);
    let typed: One = facet_json::from_str_borrowed_jsonc(&text("trailing_content.json")).unwrap();
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
    let error = facet_json::from_slice_borrowed_jsonc::<Value>(&fixture("nul_first_byte.json"))
        .unwrap_err();
    assert!(matches!(
        error.kind,
        DeserializeErrorKind::UnexpectedChar { ch: '\0', .. }
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

/// Accepted loss: jsoncpp decodes such a number as a real; a
/// `Value` holds it as a string of its digits, indistinguishable from an authored string.
#[test]
fn an_integer_outside_64_bits_is_a_string_in_a_value_deviation() {
    let v = value(&text("big_integer.json")).unwrap();
    let items = array(&v);
    assert_eq!(
        items[0].as_string().map(facet_value::VString::as_str),
        Some("18446744073709551616")
    );
    assert_eq!(
        items[1].as_string().map(facet_value::VString::as_str),
        Some("-9223372036854775809")
    );
}

#[test]
fn an_integer_outside_64_bits_is_a_float_in_a_typed_f64() {
    let parsed: Vec<f64> = facet_json::from_str_borrowed_jsonc(&text("big_integer.json")).unwrap();
    assert_eq!(
        parsed,
        [18_446_744_073_709_551_616.0, -9_223_372_036_854_775_809.0]
    );
}

#[test]
fn an_integer_of_2_pow_128_or_more_is_a_syntax_error() {
    let error = value("[340282366920938463463374607431768211456]").unwrap_err();
    assert!(matches!(
        error.kind,
        DeserializeErrorKind::UnexpectedChar { .. }
    ));
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

#[derive(Debug, Facet)]
struct Floats {
    big: f32,
    huge: f32,
}

#[test]
fn an_out_of_range_real_into_f32_is_infinity_without_an_error() {
    let parsed: Floats = facet_json::from_str_borrowed_jsonc(&text("float_overflow.json")).unwrap();
    assert_eq!(parsed.big, f32::INFINITY);
    assert_eq!(parsed.huge, f32::INFINITY);
}

// --- invalid UTF-8 ---------------------------------------------------------------------------

#[test]
fn invalid_utf8_in_a_string_is_an_error_with_a_span() {
    let error =
        facet_json::from_slice_borrowed_jsonc::<Value>(&fixture("invalid_utf8_string.json"))
            .unwrap_err();
    assert!(matches!(
        error.kind,
        DeserializeErrorKind::InvalidUtf8 { .. }
    ));
    assert!(error.span.is_some());
}

#[test]
fn invalid_utf8_in_a_comment_is_accepted() {
    let v = facet_json::from_slice_borrowed_jsonc::<Value>(&fixture("invalid_utf8_comment.json"))
        .unwrap();
    assert_eq!(keys(&v), ["a"]);
}

// --- escapes, control characters ------------------------------------------------------------

#[test]
fn an_unknown_escape_reads_as_the_letter_and_raw_control_characters_are_kept() {
    let v = value(&text("escapes.json")).unwrap();
    let strings: Vec<&str> = array(&v)
        .iter()
        .map(|s| s.as_string().unwrap().as_str())
        .collect();
    assert_eq!(strings, ["q", "line\nbreak", "tab\there"]);
}

#[test]
fn a_lone_surrogate_escape_is_invalid_utf8() {
    let error = value(r#"["\ud800"]"#).unwrap_err();
    assert!(
        matches!(error.kind, DeserializeErrorKind::InvalidUtf8 { .. }),
        "{error:?}"
    );
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

#[derive(Debug, Facet)]
struct Counted {
    name: String,
    count: u32,
}

/// The span of a member's typed error covers the member's key (`"count"`), not its value
/// [probe in this test].
#[test]
fn a_typed_error_from_text_has_a_byte_span_on_the_member_key() {
    let text = text("typed_error.json");
    let error = facet_json::from_str_borrowed_jsonc::<Counted>(&text).unwrap_err();
    let span = error.span.unwrap();
    let key = text.find("\"count\"").unwrap();
    assert_eq!(
        (span.offset as usize, span.len as usize),
        (key, "\"count\"".len())
    );
    assert!(error.path.is_some());
}

#[test]
fn an_integer_range_error_has_no_span() {
    let error = facet_json::from_str_borrowed_jsonc::<Vec<u8>>("[300]").unwrap_err();
    assert!(
        matches!(error.kind, DeserializeErrorKind::NumberOutOfRange { .. }),
        "{error:?}"
    );
    assert_eq!(error.span, None);
}

// --- object member order ---------------------------------------------------------------------

#[test]
fn object_members_keep_document_order() {
    let text = text("member_order.json");
    assert_eq!(keys(&value(&text).unwrap()), ["b", "a", "c"]);
    let map: IndexMap<String, u32> = facet_json::from_str_borrowed_jsonc(&text).unwrap();
    assert_eq!(map.keys().collect::<Vec<_>>(), ["b", "a", "c"]);
}

// --- syntax error mid-document ---------------------------------------------------------------

/// Accepted loss: jsoncpp keeps the partial value before a syntax error; facet
/// returns only the error, a parser-level `UnexpectedToken` with a span and no path.
#[test]
fn a_syntax_error_mid_document_yields_no_partial_value_deviation() {
    let text = text("syntax_error.json");
    let error = value(&text).unwrap_err();
    assert!(
        matches!(error.kind, DeserializeErrorKind::UnexpectedToken { .. }),
        "{error:?}"
    );
    assert_eq!(error.span.map(|s| s.offset as usize), text.rfind('}'));
    assert!(error.path.is_none());
}

// --- typed coercions (from text) -------------------------------------------------------------

#[derive(Debug, Facet)]
struct Coerced {
    string: String,
    unsigned: u32,
    negative: i32,
    signed: i32,
    exponent: u32,
    real: f32,
    flag: bool,
    nothing: String,
}

#[test]
fn scalars_are_coerced_into_typed_fields() {
    let parsed: Coerced = facet_json::from_str_borrowed_jsonc(&text("coercions.json")).unwrap();
    assert_eq!(parsed.string, "5");
    assert_eq!(parsed.unsigned, 7);
    assert_eq!(parsed.negative, -64);
    assert_eq!(parsed.signed, 1);
    assert_eq!(parsed.exponent, 100);
    assert_eq!(parsed.real, 1.0);
    assert!(parsed.flag);
    assert_eq!(parsed.nothing, "");
}

#[test]
fn null_sets_the_default_of_a_number_and_a_bool() {
    #[derive(Debug, Facet)]
    struct Nulls {
        n: u32,
        b: bool,
    }
    let parsed: Nulls = facet_json::from_str_borrowed_jsonc(r#"{ "n": null, "b": null }"#).unwrap();
    assert_eq!((parsed.n, parsed.b), (0, false));
}

#[test]
fn a_fraction_into_an_integer_a_bool_into_a_string_and_a_scalar_into_a_list_are_rejected() {
    #[derive(Debug, Facet)]
    struct S {
        s: String,
    }
    #[derive(Debug, Facet)]
    struct B {
        b: bool,
    }
    #[derive(Debug, Facet)]
    struct L {
        l: Vec<u32>,
    }
    assert!(facet_json::from_str_borrowed_jsonc::<One>(r#"{ "a": 1.5 }"#).is_err());
    assert!(facet_json::from_str_borrowed_jsonc::<S>(r#"{ "s": true }"#).is_err());
    assert!(facet_json::from_str_borrowed_jsonc::<B>(r#"{ "b": 1 }"#).is_err());
    assert!(facet_json::from_str_borrowed_jsonc::<L>(r#"{ "l": 1 }"#).is_err());
}

// --- serialising a Value ---------------------------------------------------------------------

/// Accepted loss: facet-json writes a `Value`'s `1.0` as `1`, `-0.0` as `0` and a string-ified big
/// integer quoted; the crate's own writer keeps the real tag (`tests/json_write.rs`).
#[test]
fn facet_json_writes_a_value_without_the_real_tag_deviation() {
    let v = value(&text("value_write.json")).unwrap();
    assert_eq!(
        facet_json::to_string(&v).unwrap(),
        r#"[1,0,"18446744073709551616"]"#
    );
}

#[test]
fn facet_json_writes_a_nan_value_as_null() {
    let v = Value::from(f64::NAN);
    assert_eq!(facet_json::to_string(&v).unwrap(), "null");
}
