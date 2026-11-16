//! Unit tests of the generic writer: one per rule of the module's list, each the JSON a hand
//! writer of the families builds for the same value.

use facet::Facet;
use indexmap::IndexMap;

use super::{Writing, write_value, write_value_with};
use crate::diagnostic::JsonPath;
use crate::document::{Extra, UnknownMembers};
use crate::id::{ItemId, kind};
use crate::json::{EncodeError, Unrepresentable, Value, from_str_jsonc, to_facet_json};
use crate::molang::Molang;
use crate::version::FormatVersion;

/// `value` written, on one line.
fn written<'f, T: Facet<'f>>(value: &T) -> String {
    to_facet_json(&write_value(value).unwrap()).unwrap()
}

/// The text of `value` in the crate's document style: the number tokens as a file has them.
fn text<'f, T: Facet<'f>>(value: &T, writing: Writing) -> String {
    let value = write_value_with(value, writing).unwrap();
    let mut out = Vec::new();
    crate::json::write_json(&value, &mut out, crate::json::WriteStyle::default()).unwrap();
    String::from_utf8(out).unwrap().split_whitespace().collect()
}

#[derive(Facet, Default)]
#[facet(rename_all = "snake_case")]
struct Members {
    first_name: Option<String>,
    #[facet(rename = "Count")]
    count: i32,
    absent: Option<bool>,
    #[facet(skip)]
    hidden: bool,
    #[facet(skip)]
    marker: Extra,
    extra: UnknownMembers,
}

#[test]
fn struct_members_are_written_in_declaration_order_by_their_json_names() {
    let value = Members {
        first_name: Some("a".to_owned()),
        count: 2,
        hidden: true,
        ..Members::default()
    };
    assert_eq!(written(&value), r#"{"first_name":"a","Count":2}"#);
}

#[test]
fn kept_unknown_members_come_after_the_members_and_a_typed_member_stands() {
    let mut extra = UnknownMembers::default();
    extra.insert("z".to_owned(), Value::from(1_i64));
    extra.insert("first_name".to_owned(), Value::from("kept"));
    extra.insert("absent".to_owned(), Value::from("as written"));
    let value = Members {
        first_name: Some("typed".to_owned()),
        extra,
        ..Members::default()
    };
    assert_eq!(
        written(&value),
        r#"{"first_name":"typed","Count":0,"z":1,"absent":"as written"}"#
    );
}

#[derive(Facet)]
struct Floats {
    whole: f32,
    tenth: f32,
    negative_zero: f32,
    wide: f64,
}

const FLOATS: Floats = Floats {
    whole: 1.0,
    tenth: 0.1,
    negative_zero: -0.0,
    wide: 0.1,
};

#[test]
fn a_float_is_its_shortest_real_and_whole_numbers_keep_their_fraction() {
    assert_eq!(
        text(&FLOATS, Writing::REAL),
        r#"{"whole":1.0,"tenth":0.1,"negative_zero":-0.0,"wide":0.1}"#
    );
}

#[test]
fn under_whole_a_whole_float_is_an_integer_and_the_negative_zero_a_real() {
    assert_eq!(
        text(&FLOATS, Writing::WHOLE),
        r#"{"whole":1,"tenth":0.1,"negative_zero":-0.0,"wide":0.1}"#
    );
}

#[derive(Facet)]
struct Deep {
    list: Vec<Floats>,
}

#[test]
fn a_float_that_is_not_finite_is_refused_at_its_path() {
    let value = Deep {
        list: vec![Floats {
            tenth: f32::NAN,
            ..FLOATS
        }],
    };
    assert!(matches!(
        write_value(&value),
        Err(EncodeError::NotRepresentable { at, what: Unrepresentable::NaN })
            if at == JsonPath::of_keys(["list"]).index(0).child("tenth")
    ));
}

#[allow(dead_code, reason = "the fields are written, never read")]
#[derive(Facet)]
#[facet(tag = "type")]
#[repr(u8)]
enum Kind {
    #[facet(rename = "item")]
    Item {
        name: Option<String>,
        functions: Option<Vec<i32>>,
    },
    #[facet(rename = "empty")]
    Empty,
}

#[derive(Facet)]
struct Entry {
    weight: Option<i32>,
    #[facet(flatten)]
    kind: Kind,
    pools: Option<Vec<i32>>,
}

#[test]
fn a_flattened_tagged_enum_writes_its_tag_and_members_in_the_place_of_the_field() {
    let item = Entry {
        weight: Some(1),
        kind: Kind::Item {
            name: Some("ns:a".to_owned()),
            functions: None,
        },
        pools: Some(Vec::new()),
    };
    assert_eq!(
        written(&item),
        r#"{"weight":1,"type":"item","name":"ns:a","pools":[]}"#
    );
    let empty = Entry {
        weight: None,
        kind: Kind::Empty,
        pools: None,
    };
    assert_eq!(written(&empty), r#"{"type":"empty"}"#);
}

#[test]
fn a_tagged_enum_is_an_object_with_its_tag_first() {
    assert_eq!(written(&Kind::Empty), r#"{"type":"empty"}"#);
}

#[allow(dead_code, reason = "the fields are written, never read")]
#[derive(Facet)]
#[facet(untagged)]
#[repr(u8)]
enum OneOrList {
    One(String),
    List(Vec<String>),
}

#[derive(Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
enum Name {
    FirstOne,
    #[allow(dead_code, reason = "a name not written")]
    Second,
}

#[allow(dead_code, reason = "the fields are written, never read")]
#[derive(Facet)]
#[repr(u8)]
enum External {
    Plain,
    Wrapped(i32),
}

#[test]
fn an_untagged_enum_is_its_value_and_a_unit_one_its_name() {
    assert_eq!(written(&OneOrList::One("a".to_owned())), r#""a""#);
    assert_eq!(written(&OneOrList::List(vec!["b".to_owned()])), r#"["b"]"#);
    assert_eq!(written(&Name::FirstOne), r#""first_one""#);
    assert_eq!(written(&External::Plain), r#""Plain""#);
    assert_eq!(written(&External::Wrapped(3)), r#"{"Wrapped":3}"#);
}

#[derive(Facet)]
#[facet(transparent)]
struct Text(String);

#[derive(Facet)]
struct Pair(i32, bool);

#[derive(Facet)]
struct Leaves {
    text: Text,
    pair: Pair,
    version: FormatVersion,
    item: ItemId,
    raw: Value,
    molang: Vec<Molang>,
    names: IndexMap<String, u8>,
    keyed: IndexMap<crate::id::Id<kind::Item>, bool>,
}

#[test]
fn leaves_are_written_through_their_proxies_and_their_own_forms() {
    let mut names = IndexMap::new();
    names.insert("b".to_owned(), 2);
    names.insert("a".to_owned(), 1);
    let mut keyed = IndexMap::new();
    keyed.insert(ItemId::parse("ns:x"), true);
    let value = Leaves {
        text: Text("t".to_owned()),
        pair: Pair(1, false),
        version: FormatVersion::new(1, 21, 0),
        item: ItemId::parse("ns:item"),
        raw: from_str_jsonc(r#"{"k": [null, 1.5]}"#).unwrap(),
        molang: vec![Molang::Const(1.0), Molang::Const(0.5), Molang::Bool(true)],
        names,
        keyed,
    };
    assert_eq!(
        text(&value, Writing::REAL),
        r#"{"text":"t","pair":[1,false],"version":"1.21.0","item":"ns:item","raw":{"k":[null,1.5]},"molang":[1,0.5,true],"names":{"b":2,"a":1},"keyed":{"ns:x":true}}"#
    );
}

#[derive(Facet)]
struct Inner {
    b: i32,
    extra: UnknownMembers,
}

#[derive(Facet)]
struct Outer {
    a: i32,
    #[facet(flatten)]
    inner: Inner,
    #[facet(skip_serializing_if = Vec::is_empty)]
    empty: Vec<i32>,
    #[facet(skip_serializing_if = Vec::is_empty)]
    full: Vec<i32>,
    c: Option<Box<Outer>>,
}

#[test]
fn a_flattened_struct_writes_in_place_and_skip_serializing_if_is_honoured() {
    let mut extra = UnknownMembers::default();
    extra.insert("u".to_owned(), Value::from(true));
    let value = Outer {
        a: 1,
        inner: Inner { b: 2, extra },
        empty: Vec::new(),
        full: vec![3],
        c: Some(Box::new(Outer {
            a: 4,
            inner: Inner {
                b: 5,
                extra: UnknownMembers::default(),
            },
            empty: vec![6],
            full: Vec::new(),
            c: None,
        })),
    };
    assert_eq!(
        written(&value),
        r#"{"a":1,"b":2,"full":[3],"c":{"a":4,"b":5,"empty":[6]},"u":true}"#
    );
}
