//! The strict wrappers: where vanilla's reader rejects a value of the wrong
//! JSON type, a field takes `StrictInt<T>`, `StrictFloat<T>`, `StrictBool` or `StrictString`,
//! because facet coerces scalars and cannot be told not to. Each wrapper against every JSON
//! shape: `1`, `1.0`, `"-64"`, `true`, `null`, `3000000000` and a value out of range.

#![allow(clippy::disallowed_methods, clippy::disallowed_types, reason = "tests read fixtures and call facet-json directly")]
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, StrictString};
use facet::Facet;

#[derive(Debug, Facet)]
struct Holder<T> {
    v: T,
}

/// Parses `{ "v": <json> }` and returns the field.
fn read<T: for<'a> Facet<'a>>(json: &str) -> Result<T, facet_json::DeserializeError> {
    let text = format!(r#"{{ "v": {json} }}"#);
    facet_json::from_str_jsonc::<Holder<T>>(&text).map(|h| h.v)
}

const SHAPES: [&str; 6] = ["1", "1.0", "\"-64\"", "true", "null", "3000000000"];

/// The shapes `T` accepts, in `SHAPES` order.
fn accepted<T: for<'a> Facet<'a>>() -> Vec<bool> {
    SHAPES.iter().map(|json| read::<T>(json).is_ok()).collect()
}

#[test]
fn strict_int_i32_takes_only_integer_tokens_in_range() {
    assert_eq!(
        accepted::<StrictInt<i32>>(),
        [true, false, false, false, false, false]
    );
    assert_eq!(read::<StrictInt<i32>>("1").unwrap(), StrictInt(1));
    assert_eq!(read::<StrictInt<i32>>("-64").unwrap(), StrictInt(-64));
}

#[test]
fn strict_int_rejects_a_value_outside_its_type() {
    assert!(read::<StrictInt<u8>>("255").is_ok());
    assert!(read::<StrictInt<u8>>("256").is_err());
    assert!(read::<StrictInt<u8>>("-1").is_err());
    assert!(read::<StrictInt<i64>>("18446744073709551615").is_err());
    assert_eq!(
        read::<StrictInt<u64>>("18446744073709551615").unwrap(),
        StrictInt(u64::MAX)
    );
    assert_eq!(
        read::<StrictInt<i64>>("-9223372036854775808").unwrap(),
        StrictInt(i64::MIN)
    );
}

#[test]
fn strict_int_i64_takes_3000000000() {
    assert_eq!(
        accepted::<StrictInt<i64>>(),
        [true, false, false, false, false, true]
    );
    assert_eq!(
        read::<StrictInt<i64>>("3000000000").unwrap(),
        StrictInt(3_000_000_000)
    );
}

#[test]
fn strict_float_f32_takes_integer_and_real_tokens() {
    assert_eq!(
        accepted::<StrictFloat<f32>>(),
        [true, true, false, false, false, true]
    );
    assert_eq!(read::<StrictFloat<f32>>("1").unwrap(), StrictFloat(1.0));
    assert_eq!(read::<StrictFloat<f32>>("1.0").unwrap(), StrictFloat(1.0));
    assert_eq!(read::<StrictFloat<f32>>("0.1").unwrap(), StrictFloat(0.1));
    assert_eq!(
        read::<StrictFloat<f32>>("3000000000").unwrap(),
        StrictFloat(3e9)
    );
}

#[test]
fn strict_float_rejects_a_number_not_finite_in_its_type() {
    assert!(read::<StrictFloat<f32>>("1e39").is_err());
    assert!(read::<StrictFloat<f32>>("-1e39").is_err());
    assert_eq!(read::<StrictFloat<f64>>("1e39").unwrap(), StrictFloat(1e39));
    assert!(read::<StrictFloat<f64>>("1e999").is_err());
}

#[test]
fn strict_bool_takes_only_true_and_false() {
    assert_eq!(
        accepted::<StrictBool>(),
        [false, false, false, true, false, false]
    );
    assert!(read::<StrictBool>("\"true\"").is_err());
    assert_eq!(read::<StrictBool>("false").unwrap(), StrictBool(false));
}

#[test]
fn strict_string_takes_only_strings() {
    assert_eq!(
        accepted::<StrictString>(),
        [false, false, true, false, false, false]
    );
    assert_eq!(
        read::<StrictString>("\"-64\"").unwrap(),
        StrictString("-64".to_owned())
    );
}

#[test]
fn null_inside_an_option_stays_none() {
    assert_eq!(read::<Option<StrictInt<i32>>>("null").unwrap(), None);
    assert_eq!(read::<Option<StrictFloat<f32>>>("null").unwrap(), None);
    assert_eq!(read::<Option<StrictBool>>("null").unwrap(), None);
    assert_eq!(read::<Option<StrictString>>("null").unwrap(), None);
    assert_eq!(
        read::<Option<StrictInt<i32>>>("7").unwrap(),
        Some(StrictInt(7))
    );
    assert!(read::<Option<StrictInt<i32>>>("1.0").is_err());
}

#[test]
fn a_rejection_carries_a_span() {
    for json in ["1.0", "\"-64\"", "true", "null", "3000000000"] {
        let error = read::<StrictInt<i32>>(json).unwrap_err();
        assert!(error.span.is_some(), "{json}: {error:?}");
    }
}

#[test]
fn a_rejection_is_reported_as_the_wrong_type_not_a_missing_field() {
    // Without a flatten field the proxy's message survives; with one, facet reports
    // ``missing field `0` ``.
    // The proxy's rejection arrives as `Reflect` (custom deserialisation failed), which the
    // crate maps to `InvalidType`.
    let error = read::<StrictInt<i32>>("true").unwrap_err();
    assert!(
        matches!(
            error.kind,
            facet_format::DeserializeErrorKind::Reflect { .. }
        ),
        "{error:?}"
    );
}

#[test]
fn wrappers_write_back_as_their_json_type() {
    assert_eq!(facet_json::to_string(&StrictInt(-64i32)).unwrap(), "-64");
    assert_eq!(facet_json::to_string(&StrictInt(7u8)).unwrap(), "7");
    assert_eq!(facet_json::to_string(&StrictBool(true)).unwrap(), "true");
    assert_eq!(
        facet_json::to_string(&StrictString("x".into())).unwrap(),
        "\"x\""
    );
    assert_eq!(facet_json::to_string(&StrictFloat(0.5f32)).unwrap(), "0.5");
}
