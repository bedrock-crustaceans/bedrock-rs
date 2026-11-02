//! The writer: fields in declaration order, `extra` members after the known
//! members of their object, `IndexMap` order kept, `f32` in the shortest form that reads back to
//! the same `f32` with a `.0` on integral values, a non-finite number refused as
//! `EncodeError::NotRepresentable`, and `WriteStyle` defaulting to two spaces and a trailing
//! newline. `NotRepresentable` names the JSON path of the value it refused.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests read fixtures and call facet-json directly"
)]
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{
    EncodeError, Indent, StrictFloat, StrictInt, Unrepresentable, Value, WriteStyle,
    keep_unknown_members, to_json, write_json,
};
use facet::Facet;
use indexmap::IndexMap;

const COMPACT: WriteStyle = WriteStyle {
    indent: Indent::Compact,
    trailing_newline: false,
};

fn compact<'f, T: Facet<'f>>(value: &T) -> String {
    let mut out = Vec::new();
    write_json(value, &mut out, COMPACT).unwrap();
    String::from_utf8(out).unwrap()
}

fn not_representable<'f, T: Facet<'f>>(value: &T) -> Unrepresentable {
    match to_json(value) {
        Err(EncodeError::NotRepresentable { what, .. }) => what,
        other => panic!("{other:?}"),
    }
}

/// The path of the first value `value` cannot write.
fn not_representable_at<'f, T: Facet<'f>>(value: &T) -> String {
    match to_json(value) {
        Err(EncodeError::NotRepresentable { at, .. }) => at.to_string(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn not_representable_names_the_member_by_its_json_path() {
    let mut nested = Nested::default();
    nested.points.push(Point::default());
    nested.points.push(Point::default());
    nested.points[1]
        .extra
        .insert("bad".into(), Value::from(f64::NAN));
    assert_eq!(not_representable_at(&nested), "points/1/bad");
    let mut map: IndexMap<String, Vec<f32>> = IndexMap::new();
    map.insert("ok".into(), vec![1.0]);
    map.insert("scale".into(), vec![0.5, f32::INFINITY]);
    assert_eq!(not_representable_at(&map), "scale/1");
}

#[test]
fn write_style_defaults_to_two_spaces_and_a_trailing_newline() {
    assert_eq!(
        WriteStyle::default(),
        WriteStyle {
            indent: Indent::Spaces(2),
            trailing_newline: true,
        }
    );
}

#[derive(Debug, Default, Facet)]
struct Point {
    z: i32,
    #[facet(rename = "minecraft:x")]
    x: i32,
    #[facet(default)]
    label: Option<String>,
    #[facet(skip)]
    extra: Extra,
}

#[test]
fn the_default_style_indents_two_spaces_and_ends_with_a_newline() {
    let point = Point {
        z: 1,
        x: 2,
        ..Point::default()
    };
    assert_eq!(
        to_json(&point).unwrap(),
        "{\n  \"z\": 1,\n  \"minecraft:x\": 2\n}\n"
    );
}

#[test]
fn other_indents_and_no_trailing_newline() {
    let list = vec![1u8, 2];
    let mut out = Vec::new();
    let tabs = WriteStyle {
        indent: Indent::Tab,
        trailing_newline: false,
    };
    write_json(&list, &mut out, tabs).unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), "[\n\t1,\n\t2\n]");
    let mut out = Vec::new();
    let four = WriteStyle {
        indent: Indent::Spaces(4),
        ..WriteStyle::default()
    };
    write_json(&list, &mut out, four).unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), "[\n    1,\n    2\n]\n");
    assert_eq!(compact(&list), "[1,2]");
}

#[test]
fn fields_are_written_in_declaration_order_and_none_is_omitted() {
    let point = Point {
        z: 3,
        x: -1,
        label: Some("a".into()),
        ..Point::default()
    };
    assert_eq!(compact(&point), r#"{"z":3,"minecraft:x":-1,"label":"a"}"#);
    let point = Point::default();
    assert_eq!(compact(&point), r#"{"z":0,"minecraft:x":0}"#);
}

#[test]
fn extra_members_follow_the_known_members_of_their_object() {
    let mut point = Point::default();
    point.extra.insert("b_unknown".into(), Value::from(1i64));
    point.extra.insert("a_unknown".into(), Value::from("s"));
    assert_eq!(
        compact(&point),
        r#"{"z":0,"minecraft:x":0,"b_unknown":1,"a_unknown":"s"}"#
    );
}

#[derive(Debug, Default, Facet)]
struct Nested {
    points: Vec<Point>,
    first: Option<Point>,
    map: IndexMap<String, u8>,
    #[facet(skip)]
    extra: Extra,
}

#[test]
fn extra_members_are_written_at_their_level() {
    let mut inner = Point::default();
    inner.extra.insert("deep".into(), Value::TRUE);
    let mut nested = Nested {
        points: vec![Point::default(), inner],
        ..Nested::default()
    };
    nested.extra.insert("top".into(), Value::NULL);
    assert_eq!(
        compact(&nested),
        r#"{"points":[{"z":0,"minecraft:x":0},{"z":0,"minecraft:x":0,"deep":true}],"map":{},"top":null}"#
    );
}

#[test]
fn map_order_is_kept() {
    let map: IndexMap<String, u8> = [("b", 1), ("a", 2), ("c", 3)]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
    assert_eq!(compact(&map), r#"{"b":1,"a":2,"c":3}"#);
}

#[test]
fn f32_is_written_shortest_with_a_real_tag() {
    assert_eq!(
        compact(&vec![1.0f32, 0.1, -2.5, 100.0, 3e9, 1e-7, 1e20, -0.0]),
        "[1.0,0.1,-2.5,100.0,3000000000.0,1e-7,1e20,-0.0]"
    );
}

#[test]
fn f32_values_read_back_to_the_same_value() {
    let values: Vec<f32> = vec![
        0.1,
        1.0 / 3.0,
        f32::MAX,
        f32::MIN_POSITIVE,
        16_777_217.0,
        0.3,
    ];
    let text = compact(&values);
    let back: Vec<f32> = facet_json::from_str(&text).unwrap();
    assert_eq!(back, values);
}

#[test]
fn f64_and_integers_are_written_as_their_json_type() {
    assert_eq!(compact(&vec![1.0f64, 0.1]), "[1.0,0.1]");
    assert_eq!(compact(&vec![u64::MAX]), "[18446744073709551615]");
    assert_eq!(compact(&vec![i64::MIN]), "[-9223372036854775808]");
}

#[test]
fn strict_wrappers_write_their_value() {
    assert_eq!(compact(&StrictFloat(0.1f32)), "0.1");
    assert_eq!(compact(&StrictFloat(2.0f32)), "2.0");
    assert_eq!(compact(&StrictInt(-64i32)), "-64");
}

/// A strict float behind a transparent newtype, as a component payload holds it.
#[derive(Debug, Facet)]
#[facet(transparent)]
struct Transparent(StrictFloat<f32>);

/// The newtype as a member, where it is not the root of the walk.
#[derive(Debug, Facet)]
struct HoldsTransparent {
    friction: Transparent,
    #[facet(default)]
    optional: Option<Transparent>,
}

#[test]
fn a_strict_f32_behind_a_transparent_newtype_is_written_shortest() {
    // Written as its `f32`, as a struct field is, not widened to `0.4000000059604645`.
    assert_eq!(compact(&Transparent(StrictFloat(0.4))), "0.4");
    assert_eq!(compact(&Transparent(StrictFloat(2.0))), "2.0");
    assert_eq!(
        compact(&HoldsTransparent {
            friction: Transparent(StrictFloat(0.1)),
            optional: Some(Transparent(StrictFloat(0.3))),
        }),
        r#"{"friction":0.1,"optional":0.3}"#
    );
    assert_eq!(compact(&vec![Transparent(StrictFloat(0.6))]), "[0.6]");
}

#[test]
fn a_value_keeps_its_number_tags_and_member_order() {
    let value: Value =
        facet_json::from_str(r#"{"b": 1.0, "a": [1, -0.0, 2.5e3], "c": "18446744073709551616"}"#)
            .unwrap();
    assert_eq!(
        compact(&value),
        r#"{"b":1.0,"a":[1,-0.0,2500.0],"c":"18446744073709551616"}"#
    );
}

#[test]
fn strings_are_escaped() {
    assert_eq!(
        compact(&"a\"b\\c\nd\u{1}é".to_owned()),
        r#""a\"b\\c\nd\u0001é""#
    );
}

#[test]
fn empty_containers_stay_on_one_line() {
    let nested = Nested::default();
    assert_eq!(
        to_json(&nested).unwrap(),
        "{\n  \"points\": [],\n  \"map\": {}\n}\n"
    );
}

#[test]
fn nan_and_infinities_are_not_representable() {
    assert_eq!(not_representable(&vec![f32::NAN]), Unrepresentable::NaN);
    assert_eq!(
        not_representable(&vec![f32::INFINITY]),
        Unrepresentable::Infinity
    );
    assert_eq!(
        not_representable(&vec![f64::NEG_INFINITY]),
        Unrepresentable::NegativeInfinity
    );
    assert_eq!(
        not_representable(&Value::from(f64::NAN)),
        Unrepresentable::NaN
    );
    assert_eq!(
        not_representable(&StrictFloat(f32::INFINITY)),
        Unrepresentable::Infinity
    );
}

// A constant is written as the token an author writes, as a member, in a list and
// inside an `Option`; the widened text still reads as the same constant.
#[test]
#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
fn a_molang_constant_is_written_as_its_shortest_token() {
    use molangx::json::MolangValueRepr as Molang;
    #[derive(Facet)]
    struct Fields {
        member: Molang,
        list: Vec<Molang>,
        maybe: Option<Molang>,
        flag: Molang,
    }
    assert_eq!(compact(&Molang::Const(0.1)), "0.1");
    assert_eq!(compact(&Molang::Const(1.0)), "1");
    assert_eq!(compact(&Molang::Const(-0.0)), "-0.0");
    let fields = Fields {
        member: Molang::Const(16.0),
        list: vec![Molang::Const(0.0), Molang::Const(2.5)],
        maybe: Some(Molang::Const(-1.0)),
        flag: Molang::Bool(true),
    };
    assert_eq!(
        compact(&fields),
        r#"{"member":16,"list":[0,2.5],"maybe":-1,"flag":true}"#
    );
    let back: Molang = facet_json::from_str("0.10000000149011612").unwrap();
    assert!(matches!(back, Molang::Const(x) if x == 0.1));
}

#[test]
fn a_non_finite_molang_constant_is_refused() {
    let constant = molangx::json::MolangValueRepr::Const(f32::NAN);
    assert_eq!(not_representable(&constant), Unrepresentable::NaN);
    let constant = molangx::json::MolangValueRepr::Const(f32::NEG_INFINITY);
    assert_eq!(
        not_representable(&constant),
        Unrepresentable::NegativeInfinity
    );
}

#[test]
fn a_document_round_trips_by_value_with_its_unknown_members() {
    let text =
        r#"{ "points": [{ "z": 1, "q": [1.0], "minecraft:x": 2 }], "map": { "k": 7 }, "w": {} }"#;
    let json: Value = facet_json::from_str(text).unwrap();
    let mut nested: Nested = facet_json::from_str(text).unwrap();
    keep_unknown_members(&mut nested, &json);
    let written = to_json(&nested).unwrap();
    let again: Value = facet_json::from_str(&written).unwrap();
    assert_eq!(
        compact(&again),
        r#"{"points":[{"z":1,"minecraft:x":2,"q":[1.0]}],"map":{"k":7},"w":{}}"#
    );
}

#[derive(Debug, Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(dead_code)]
enum Untagged {
    Named {
        name: String,
        #[facet(skip)]
        extra: Extra,
    },
    Number(u8),
}

#[derive(Debug, Facet)]
#[repr(u8)]
#[allow(dead_code)]
enum Tagged {
    Named {
        name: String,
        #[facet(skip)]
        extra: Extra,
    },
    Unit,
}

#[test]
fn extra_members_of_a_struct_variant_follow_its_fields() {
    let mut extra = Extra::default();
    extra.insert("u".into(), Value::from(1i64));
    let untagged = Untagged::Named {
        name: "a".into(),
        extra: extra.clone(),
    };
    assert_eq!(compact(&untagged), r#"{"name":"a","u":1}"#);
    let tagged = vec![
        Tagged::Named {
            name: "b".into(),
            extra,
        },
        Tagged::Unit,
    ];
    assert_eq!(compact(&tagged), r#"[{"Named":{"name":"b","u":1}},"Unit"]"#);
}

// --- number respelt: the write-back class of a raw `Value` number ---------------------

/// A number held as a raw `Value` (an unknown member, an opaque member) is written as the
/// shortest spelling of its value in its kind: an integer token stays an integer, a real
/// token stays a real (with `.0` when whole). The author's token is not kept.
#[test]
fn a_raw_number_is_written_as_the_shortest_spelling_of_its_value_in_its_kind() {
    for (written, expected) in [
        // Reals: the exponent, the sign of the exponent, trailing and leading zeros go.
        ("1e2", "100.0"),
        ("1E+2", "100.0"),
        ("100.0", "100.0"),
        ("1.50", "1.5"),
        ("0.1e1", "1.0"),
        ("2.5e3", "2500.0"),
        ("1e-2", "0.01"),
        ("1e22", "1e22"),
        ("1.0e+22", "1e22"),
        ("123456789012345678901234567890.0", "1.2345678901234568e29"),
        ("0.0", "0.0"),
        ("-0.0", "-0.0"),
        ("-0e0", "-0.0"),
        ("0.10000000149011612", "0.10000000149011612"),
        // Integers: the value, no sign on zero, no leading zeros.
        ("100", "100"),
        ("-7", "-7"),
        ("-0", "0"),
        ("007", "7"),
        ("-007", "-7"),
        ("18446744073709551615", "18446744073709551615"),
        ("-9223372036854775808", "-9223372036854775808"),
        // An integer beyond 64 bits is the double it spells.
        ("18446744073709551616", "1.8446744073709552e19"),
    ] {
        let text = format!(r#"{{"n":[{written}]}}"#);
        let value: Value = bedrock_addon_core::json::from_str_jsonc(&text).unwrap();
        let out = compact(&value);
        assert_eq!(out, format!(r#"{{"n":[{expected}]}}"#), "`{written}`");
        // The written token is a fixed point, and the value is the one read.
        let again: Value = bedrock_addon_core::json::from_str_jsonc(&out).unwrap();
        assert_eq!(again, value, "`{written}`");
        assert_eq!(compact(&again), out, "`{written}`");
    }
}

/// The kind is the token's: `1.0` and `1` are different values to a raw `Value`, so neither is
/// written as the other.
#[test]
fn a_raw_number_keeps_its_kind() {
    let read = |text: &str| -> Value { bedrock_addon_core::json::from_str_jsonc(text).unwrap() };
    assert_ne!(compact(&read("1.0")), compact(&read("1")));
    assert_eq!(compact(&read("1.0")), "1.0");
    assert_eq!(compact(&read("1")), "1");
    assert!(read("1.0").as_number().unwrap().is_float());
    assert!(read("1e2").as_number().unwrap().is_float());
    assert!(read("-0").as_number().unwrap().is_integer());
    assert_eq!(read("-0").as_number().unwrap().to_i64(), Some(0));
    assert!(
        read("-0.0")
            .as_number()
            .unwrap()
            .to_f64_lossy()
            .is_sign_negative()
    );
}

/// `to_facet_json_signed` is `to_facet_json` (one line, a whole real as an integer) but for a
/// real negative zero, which keeps its sign: `-0.0`. An integer `-0` is the integer 0.
#[test]
fn the_signed_facet_text_keeps_a_negative_zero_and_nothing_else() {
    use bedrock_addon_core::json::{Value, from_str, to_facet_json, to_facet_json_signed};
    let value: Value = from_str(
        r#"{"a": -0.0, "b": [-0.0, -0, 0.0, 1.0, 2.5, "x\"y", null, true], "c": {"d": -0e0}}"#,
    )
    .unwrap();
    assert_eq!(
        to_facet_json(&value).unwrap(),
        r#"{"a":0,"b":[0,0,0,1,2.5,"x\"y",null,true],"c":{"d":0}}"#
    );
    assert_eq!(
        to_facet_json_signed(&value).unwrap(),
        r#"{"a":-0.0,"b":[-0.0,0,0,1,2.5,"x\"y",null,true],"c":{"d":-0.0}}"#
    );
    // Without a negative zero the two are one text.
    let plain: Value = from_str(r#"{"a": 1.0, "b": [0, 0.0, {"c": "d"}]}"#).unwrap();
    assert_eq!(
        to_facet_json_signed(&plain).unwrap(),
        to_facet_json(&plain).unwrap()
    );
}
