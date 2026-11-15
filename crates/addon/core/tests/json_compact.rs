//! `json::to_facet_json`: a `Value` on one line, spelt as facet-json's writer spelt it when it
//! wrote the crate family's values. The texts below are what that writer gave for each value.

use bedrock_addon_core::json::{
    EncodeError, Unrepresentable, Value, from_str_jsonc, to_facet_json,
};

fn compact(text: &str) -> String {
    let value: Value = from_str_jsonc(text).expect("test JSON");
    to_facet_json(&value).expect("a value with a text")
}

#[test]
fn arrays_and_objects_are_written_without_white_space() {
    assert_eq!(
        compact(r#"{"a": [1, {"b": [], "c": {}}], "d": null, "e": true, "f": false}"#),
        r#"{"a":[1,{"b":[],"c":{}}],"d":null,"e":true,"f":false}"#
    );
}

#[test]
fn an_integral_real_is_an_integer_and_another_real_is_written_out_in_full() {
    let reals = compact("[0, -0, 0.0, -0.0, 1.0, -1.5, 0.1, 2.5e-7, 1e21, 1e300, 5e-324]");
    let expected = format!(
        "[0,0,0,0,1,-1.5,0.1,0.00000025,1000000000000000000000,1{},0.{}5]",
        "0".repeat(300),
        "0".repeat(323)
    );
    assert_eq!(reals, expected);
}

/// An integer past 64 bits is read as the double it spells; a double at the edge of a 64 bit
/// integer is written as the nearest one of its range.
#[test]
fn a_number_at_the_edge_of_64_bits_is_written_as_the_integer_in_range() {
    assert_eq!(
        compact(
            "[9223372036854775807, 9223372036854775808, -9223372036854775808, \
             -9223372036854775809, 18446744073709551615, 18446744073709551616, \
             9.223372036854776e18]"
        ),
        "[9223372036854775807,9223372036854775808,-9223372036854775808,\
         -9223372036854775808,18446744073709551615,18446744073709551615,\
         9223372036854775807]"
    );
}

#[test]
fn a_string_escapes_its_quote_backslash_and_control_characters() {
    assert_eq!(
        compact(r#"["\"\\/", "\n\r\t\b\f", "\u0000\u001f\u007f\u0080", "é漢🎉"]"#),
        "[\"\\\"\\\\/\",\"\\n\\r\\t\\b\\f\",\"\\u0000\\u001f\\u007f\u{80}\",\"é漢🎉\"]"
    );
    assert_eq!(compact(r#"{"k\"ey\n": "v"}"#), r#"{"k\"ey\n":"v"}"#);
}

#[test]
fn a_real_that_is_not_finite_is_null() {
    for real in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(to_facet_json(&Value::from(real)).unwrap(), "null");
    }
}

#[test]
fn a_value_past_the_nesting_limit_has_no_text() {
    let mut value = Value::from(7_u64);
    for _ in 0..=bedrock_addon_core::json::MAX_JSON_DEPTH {
        value = Value::from(facet_value::VArray::from_iter([value]));
    }
    assert!(matches!(
        to_facet_json(&value),
        Err(EncodeError::NotRepresentable {
            what: Unrepresentable::TooDeep,
            ..
        })
    ));
}
