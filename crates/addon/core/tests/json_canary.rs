//! The `from_value` canary: typed structs are deserialised from text, never
//! from a `facet_value::Value`, because in rc.7 `facet_value::from_value` fills a missing
//! required field with its `Default`, drops the keys bound for a `#[facet(flatten)]` map and
//! breaks molangx's proxies. These tests pin that behaviour: when one of
//! them fails, facet has changed and the rule that every typed read goes through the text is
//! open for review again.

#![allow(clippy::disallowed_methods, clippy::disallowed_types, reason = "tests read fixtures and call facet-json directly")]
use bedrock_addon_core::json::Value;
use facet::Facet;
use indexmap::IndexMap;

#[derive(Debug, Facet)]
struct Required {
    name: String,
    count: u32,
}

fn value(text: &str) -> Value {
    facet_json::from_str_borrowed_jsonc(text).unwrap()
}

#[test]
fn from_value_still_defaults_a_missing_required_field_canary() {
    let parsed: Required = facet_value::from_value(value(r#"{ "name": "x" }"#)).unwrap();
    assert_eq!(
        parsed.count, 0,
        "from_value fills no missing field with its default: revisit the text-only reading rule"
    );
}

#[test]
fn from_text_a_missing_required_field_is_an_error() {
    let error = facet_json::from_str_borrowed_jsonc::<Required>(r#"{ "name": "x" }"#).unwrap_err();
    assert!(matches!(
        error.kind,
        facet_format::DeserializeErrorKind::MissingField { field: "count", .. }
    ));
}

#[derive(Debug, Facet)]
struct Flattened {
    name: String,
    #[facet(flatten)]
    rest: IndexMap<String, Value>,
}

#[test]
fn from_value_still_drops_the_keys_of_a_flatten_map_canary() {
    let parsed: Flattened = facet_value::from_value(value(r#"{ "name": "x", "a": 1 }"#)).unwrap();
    assert!(
        parsed.rest.is_empty(),
        "from_value now fills flatten maps: revisit the text-only reading rule"
    );
    let from_text: Flattened =
        facet_json::from_str_borrowed_jsonc(r#"{ "name": "x", "a": 1 }"#).unwrap();
    assert_eq!(from_text.rest.keys().collect::<Vec<_>>(), ["a"]);
}

#[derive(Debug, Facet)]
struct WithMolang {
    m: molangx::json::MolangValueRepr,
}

#[test]
fn from_value_still_breaks_the_molang_proxy_canary() {
    let parsed: WithMolang = facet_value::from_value(value(r#"{ "m": 1.5 }"#)).unwrap();
    assert!(
        matches!(parsed.m, molangx::json::MolangValueRepr::Expr(_)),
        "from_value now reads 1.5 as a constant: revisit the text-only reading rule"
    );
    let from_text: WithMolang = facet_json::from_str_borrowed_jsonc(r#"{ "m": 1.5 }"#).unwrap();
    assert!(matches!(from_text.m, molangx::json::MolangValueRepr::Const(x) if x == 1.5));
}
