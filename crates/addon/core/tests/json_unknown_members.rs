//! `UnknownMembers`, the unknown members of one object a hand reader keeps for write-back: a
//! `#[facet(transparent)]` newtype over `IndexMap<String, json::Value>` in document order; and
//! `UnknownMembers`, the zero-size mark of a struct whose unknown members the document's table keeps.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests read fixtures and call facet-json directly"
)]
use bedrock_addon_core::document::{Extra, UnknownMembers};
use bedrock_addon_core::json::Value;
use facet::Facet;

fn number(n: i64) -> Value {
    Value::from(n)
}

#[test]
fn a_new_extra_is_empty() {
    let extra = UnknownMembers::default();
    assert!(extra.is_empty());
    assert_eq!(extra.iter().count(), 0);
    assert_eq!(extra.get("a"), None);
}

#[test]
fn extra_keeps_insertion_order() {
    let mut extra = UnknownMembers::default();
    assert_eq!(extra.insert("b".into(), number(1)), None);
    assert_eq!(extra.insert("a".into(), number(2)), None);
    assert!(!extra.is_empty());
    let keys: Vec<&str> = extra.iter().map(|(k, _)| k).collect();
    assert_eq!(keys, ["b", "a"]);
}

#[test]
fn inserting_a_present_key_replaces_it_in_place() {
    let mut extra = UnknownMembers::default();
    extra.insert("b".into(), number(1));
    extra.insert("a".into(), number(2));
    assert_eq!(extra.insert("b".into(), number(3)), Some(number(1)));
    let entries: Vec<(&str, &Value)> = extra.iter().collect();
    assert_eq!(entries, [("b", &number(3)), ("a", &number(2))]);
}

#[test]
fn removing_a_member_keeps_the_order_of_the_rest() {
    let mut extra = UnknownMembers::default();
    for (i, key) in ["a", "b", "c"].into_iter().enumerate() {
        extra.insert(key.into(), number(i64::try_from(i).unwrap()));
    }
    assert_eq!(extra.remove("a"), Some(number(0)));
    assert_eq!(extra.remove("a"), None);
    let keys: Vec<&str> = extra.iter().map(|(k, _)| k).collect();
    assert_eq!(keys, ["b", "c"]);
    assert_eq!(extra.get("c"), Some(&number(2)));
}

#[test]
fn extra_is_a_transparent_map_for_facet() {
    assert!(UnknownMembers::SHAPE.is_transparent());
    let extra: UnknownMembers = facet_json::from_str(r#"{ "z": 1, "a": [true] }"#).unwrap();
    let keys: Vec<&str> = extra.iter().map(|(k, _)| k).collect();
    assert_eq!(keys, ["z", "a"]);
    assert_eq!(
        facet_json::to_string(&extra).unwrap(),
        r#"{"z":1,"a":[true]}"#
    );
}

#[derive(Debug, Default, Facet)]
struct Keeps {
    known: u32,
    #[facet(skip)]
    extra: UnknownMembers,
}

#[test]
fn a_skipped_extra_field_is_left_empty_by_the_deserialiser() {
    let parsed: Keeps = facet_json::from_str(r#"{ "known": 1 }"#).unwrap();
    assert_eq!(parsed.known, 1);
    assert!(parsed.extra.is_empty());
}

#[test]
fn the_extra_mark_holds_nothing() {
    assert_eq!(std::mem::size_of::<Extra>(), 0);
    assert_eq!(Extra::default(), Extra {});
}
