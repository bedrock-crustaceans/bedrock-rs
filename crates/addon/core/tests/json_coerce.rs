//! `Coerce`: jsoncpp's `isInt` and `asInt` under both rules the tree holds, each pinned against
//! the copies it stands for (bodies quoted below from the files named). Neither rule is chosen.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![allow(clippy::cast_possible_truncation)] // the quoted bodies cast as their sources do
#![allow(clippy::redundant_closure_for_method_calls)] // VNumber is not named here

use bedrock_addon_core::json::{Coerce, Coercing, IsIntRule, Value, WideInt, from_str_jsonc};

const TOKEN: Coercing = Coercing::new(IsIntRule::Token, WideInt::LowWord);
const INTEGRAL: Coercing = Coercing::new(IsIntRule::Integral, WideInt::LowWord);
const ZERO: Coercing = Coercing::new(IsIntRule::Token, WideInt::Zero);

const TEXTS: &[&str] = &[
    "0",
    "2",
    "-7",
    "2.0",
    "-0.0",
    "2.5",
    "-2.5",
    "1e2",
    "2147483647",
    "2147483648",
    "-2147483648",
    "-2147483649",
    "2147483647.0",
    "2147483648.0",
    "-2147483648.0",
    "1e10",
    "-1e10",
    "9223372036854775807",
    "9223372036854775808",
    "18446744073709551614",
    "1e300",
    "\"2\"",
    "\"\"",
    "true",
    "false",
    "null",
    "[]",
    "[1]",
    "{}",
];

fn value(text: &str) -> Value {
    from_str_jsonc(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

// ---- rule A: `support/src/fixed_path.rs` `as_int` (also `shared::read::int`) --------------

/// jsoncpp's `isInt` for a member read into an `i32`: a number written without a fraction or
/// an exponent, in `i32`.
fn fixed_path_as_int(value: &Value) -> Option<i32> {
    let number = value.as_number()?;
    if number.is_float() {
        return None;
    }
    number.to_i32()
}

/// An integer token that fits `i32` (`core/src/shared/read.rs` `int`, its error dropped).
fn shared_read_int(value: &Value) -> Option<i32> {
    match value.as_number() {
        Some(number) if !number.is_float() => number.to_i64().and_then(|i| i32::try_from(i).ok()),
        _ => None,
    }
}

// ---- rule B: `families/loot/src/parse.rs` `whole` + `int`, `families/feature/src/values.rs`,
// `families/ui_texture_info/src/lib.rs` `as_int` --------------------------------------------

/// loot's `whole` (integral test) then `int` (its read), findings dropped.
fn loot_whole(value: &Value) -> Option<i32> {
    let integral = value
        .as_number()
        .is_some_and(|n| !n.is_float() || n.to_f64_lossy().fract() == 0.0);
    if !integral {
        return None;
    }
    value.as_number().and_then(|number| {
        if number.is_float() {
            let real = number.to_f64_lossy();
            let x = real.trunc();
            (x >= f64::from(i32::MIN) && x <= f64::from(i32::MAX)).then_some(x as i32)
        } else {
            number.to_i64().and_then(|i| i32::try_from(i).ok())
        }
    })
}

/// feature's `JsonInt` read (`TryFrom<ScalarShape>`), over the number a `Value` holds.
fn feature_json_int(value: &Value) -> Option<i32> {
    let number = value.as_number()?;
    if !number.is_float() {
        return number.to_i64().and_then(|i| i32::try_from(i).ok());
    }
    let x = number.to_f64_lossy();
    (x.fract() == 0.0 && x >= f64::from(i32::MIN) && x <= f64::from(i32::MAX)).then_some(x as i32)
}

/// jsoncpp's `isInt`: an integral number in `i32` (`ui_texture_info`).
fn ui_texture_info_as_int(value: Option<&Value>) -> Option<i32> {
    let number = value?.as_number()?;
    if number.is_float() {
        let float = number.to_f64()?;
        return (float.fract() == 0.0 && float.abs() <= f64::from(i32::MAX))
            .then_some(float as i32);
    }
    number.to_i32()
}

#[test]
fn rule_a_is_the_token_copies() {
    for text in TEXTS {
        let v = value(text);
        assert_eq!(v.is_int(TOKEN), fixed_path_as_int(&v), "{text}");
        assert_eq!(v.is_int(TOKEN), shared_read_int(&v), "{text}");
    }
    assert_eq!(value("2.0").is_int(TOKEN), None);
    assert_eq!(value("2").is_int(TOKEN), Some(2));
}

#[test]
fn rule_b_is_the_integral_copies() {
    for text in TEXTS {
        let v = value(text);
        assert_eq!(v.is_int(INTEGRAL), loot_whole(&v), "{text}");
        assert_eq!(v.is_int(INTEGRAL), feature_json_int(&v), "{text}");
        // ui_texture_info bounds the real by `abs() <= i32::MAX`, so it alone refuses
        // `-2147483648.0`; everywhere else it agrees.
        if *text != "-2147483648.0" {
            assert_eq!(
                v.is_int(INTEGRAL),
                ui_texture_info_as_int(Some(&v)),
                "{text}"
            );
        }
    }
    assert_eq!(value("2.0").is_int(INTEGRAL), Some(2));
    assert_eq!(value("1e2").is_int(INTEGRAL), Some(100));
    assert_eq!(value("2.5").is_int(INTEGRAL), None);
    assert_eq!(value("-2147483648.0").is_int(INTEGRAL), Some(i32::MIN));
    assert_eq!(ui_texture_info_as_int(Some(&value("-2147483648.0"))), None);
}

/// The two rules part only on a real token with an integral value in range.
#[test]
fn the_rules_differ_only_on_integral_reals() {
    for text in TEXTS {
        let v = value(text);
        let integral_real =
            v.as_number().is_some_and(|n| n.is_float()) && v.is_int(INTEGRAL).is_some();
        assert_eq!(
            v.is_int(TOKEN) != v.is_int(INTEGRAL),
            integral_real,
            "{text}"
        );
    }
}

// ---- asInt: `support/src/texture/jsoncpp.rs` (= persona, entity) and `support/src/sound/fork.rs`

/// texture's `as_int`.
fn texture_as_int(value: Option<&Value>, default: i32) -> i32 {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return default;
    };
    if let Some(flag) = value.as_bool() {
        return i32::from(flag);
    }
    let Some(number) = value.as_number() else {
        return 0;
    };
    if number.is_float() {
        return number.to_f64_lossy() as i32;
    }
    let low_word = |bytes: [u8; 8]| {
        let [a, b, c, d, ..] = bytes;
        i32::from_le_bytes([a, b, c, d])
    };
    number.to_i64().map_or_else(
        || low_word(number.to_u64().unwrap_or_default().to_le_bytes()),
        |wide| low_word(wide.to_le_bytes()),
    )
}

/// sound fork's `as_int` (over its `stored`).
fn fork_as_int(value: &Value, default: i32) -> i32 {
    if value.is_null() {
        default
    } else if value.is_bool() {
        i32::from(value.is_true())
    } else if let Some(number) = value.as_number() {
        if number.is_float() {
            number.to_f64_lossy() as i32
        } else {
            number.to_i64().map_or(0, |n| n as i32)
        }
    } else {
        0
    }
}

#[test]
fn as_int_with_the_low_word_is_the_texture_copy() {
    for text in TEXTS {
        let v = value(text);
        for default in [0, -1, 9] {
            assert_eq!(
                v.as_int(default, TOKEN),
                texture_as_int(Some(&v), default),
                "{text}"
            );
            assert_eq!(
                v.as_int(default, INTEGRAL),
                texture_as_int(Some(&v), default),
                "{text}"
            );
        }
    }
    assert_eq!(value("18446744073709551614").as_int(0, TOKEN), -2);
}

#[test]
fn as_int_with_zero_is_the_sound_fork_copy() {
    for text in TEXTS {
        let v = value(text);
        for default in [0, -1, 9] {
            assert_eq!(v.as_int(default, ZERO), fork_as_int(&v, default), "{text}");
        }
    }
    assert_eq!(value("18446744073709551614").as_int(0, ZERO), 0);
}

#[test]
fn as_int_reads_each_kind_of_value() {
    for (text, expected) in [
        ("null", 9),
        ("true", 1),
        ("false", 0),
        ("2.9", 2),
        ("-2.9", -2),
        ("1e300", i32::MAX),
        ("-1e300", i32::MIN),
        ("4294967297", 1),
        ("\"7\"", 0),
        ("[7]", 0),
        ("{}", 0),
    ] {
        assert_eq!(value(text).as_int(9, TOKEN), expected, "{text}");
    }
}
