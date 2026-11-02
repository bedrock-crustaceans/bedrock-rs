//! The game's jsoncpp accessors, which the hand-written spawn parsers read every member
//! with: a default applies only to a missing or `null` member, and every other JSON type is
//! coerced, never rejected.
//!
//! The crate adds a warning: the value is the one vanilla computes, and a member of the wrong
//! JSON type also gets an `InvalidType` warning at its path (vanilla is silent).

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::json::Value;

/// A finding of the spawn readers at `at` (a parse finding without a span: the readers walk a
/// `Value`, so a finding carries a path and no span).
pub fn finding(code: DiagnosticCode, severity: Severity, at: &JsonPath, d: &mut Diagnostics) {
    d.push(Diagnostic::new(
        code,
        severity,
        Phase::Parse,
        Location::new(None, None, at.clone(), None),
    ));
}

/// An `InvalidType` information at `at`, with vanilla's text when it has one; without one it is
/// the crate's own information: the legacy parsers read through jsoncpp's
/// `asInt`/`asString`/`asBool`, which convert or default without a line, and skip what is not
/// an object or an array in silence.
pub fn wrong_type(at: &JsonPath, text: Option<VanillaText>, d: &mut Diagnostics) {
    finding(DiagnosticCode::InvalidType { text }, Severity::Info, at, d);
}

/// `at` followed by `key`.
#[must_use]
pub fn child(at: &JsonPath, key: &str) -> JsonPath {
    let mut path = at.clone();
    path.push(key);
    path
}

/// `at` followed by the element `index`.
#[must_use]
pub fn element(at: &JsonPath, index: usize) -> JsonPath {
    let mut path = at.clone();
    path.push_index(index);
    path
}

/// The member as jsoncpp's accessors see it: `None` for a missing or `null` member.
fn present(value: Option<&Value>) -> Option<&Value> {
    value.filter(|v| !v.is_null())
}

/// An integer token's two's-complement bits (the low 64), or `None` for a real or no number.
fn integer_bits(value: &Value) -> Option<u64> {
    let number = value.as_number().filter(|n| n.is_integer())?;
    #[allow(clippy::cast_sign_loss, reason = "two's-complement bits")]
    let bits = |i: i64| i as u64;
    // An integer token fits 64 bits: past them the reader hands over a real.
    number.to_i64().map(bits).or_else(|| number.to_u64())
}

/// An integer token's value where one of 128 bits holds every token `integer_bits` reads.
fn integer(value: &Value) -> Option<i128> {
    let number = value.as_number().filter(|n| n.is_integer())?;
    number
        .to_i64()
        .map(i128::from)
        .or_else(|| number.to_u64().map(i128::from))
}

/// The crate's own information: an integer token outside `range`, of which vanilla keeps the
/// low 32 bits (`4294967296` is 0, `-2147483649` is 2147483647), is read as vanilla reads it
/// and gets an `InvalidValue` information at its path; vanilla is silent.
fn told_beyond(
    value: &Value,
    range: std::ops::RangeInclusive<i128>,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    if integer(value).is_some_and(|integer| !range.contains(&integer)) {
        finding(
            DiagnosticCode::InvalidValue { text: None },
            Severity::Info,
            at,
            d,
        );
    }
}

/// A real token's value.
fn real(value: &Value) -> Option<f64> {
    value
        .as_number()
        .filter(|n| !n.is_integer())
        .map(facet_value::VNumber::to_f64_lossy)
}

/// `asInt(default)`: `None` for a missing or `null` member (the caller applies the default); an
/// integer's low 32 bits; a real truncated (saturating); a bool 0 or 1; a string, array or object
/// 0. Every type but an integer is warned, and an integer no `i32` holds.
pub fn int(value: Option<&Value>, at: &JsonPath, d: &mut Diagnostics) -> Option<i32> {
    let value = present(value)?;
    if let Some(bits) = integer_bits(value) {
        told_beyond(value, i128::from(i32::MIN)..=i128::from(i32::MAX), at, d);
        #[allow(
            clippy::cast_possible_truncation,
            reason = "jsoncpp keeps the low 32 bits"
        )]
        return Some((bits as u32).cast_signed());
    }
    wrong_type(at, None, d);
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the game's float-to-integer conversion saturates, as `as` does"
    )]
    Some(match (real(value), value.as_bool()) {
        (Some(f), _) => f as i32,
        (None, Some(b)) => i32::from(b),
        (None, None) => 0,
    })
}

/// `asUInt(default)`: as [`int`], a negative real 0. An integer no `u32` holds is warned (a
/// negative one wraps to a large `u32`).
pub fn uint(value: Option<&Value>, at: &JsonPath, d: &mut Diagnostics) -> Option<u32> {
    let value = present(value)?;
    if let Some(bits) = integer_bits(value) {
        told_beyond(value, 0..=i128::from(u32::MAX), at, d);
        #[allow(
            clippy::cast_possible_truncation,
            reason = "jsoncpp keeps the low 32 bits"
        )]
        return Some(bits as u32);
    }
    wrong_type(at, None, d);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the game's float-to-unsigned conversion saturates, negatives to 0, as `as` does"
    )]
    Some(match (real(value), value.as_bool()) {
        (Some(f), _) => f as u32,
        (None, Some(b)) => u32::from(b),
        (None, None) => 0,
    })
}

/// `asInt64(default)` as the unsigned value spawn rules store: an integer's 64 bits (a negative
/// one wraps, and is warned); a real truncated to `i32`, then sign-extended; a bool 0 or 1;
/// anything else 0.
pub fn int64(value: Option<&Value>, at: &JsonPath, d: &mut Diagnostics) -> Option<u64> {
    let value = present(value)?;
    if let Some(bits) = integer_bits(value) {
        told_beyond(value, 0..=i128::from(u64::MAX), at, d);
        return Some(bits);
    }
    wrong_type(at, None, d);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "jsoncpp truncates a real to int and sign-extends it; the store is unsigned"
    )]
    Some(match (real(value), value.as_bool()) {
        (Some(f), _) => i64::from(f as i32) as u64,
        (None, Some(b)) => u64::from(b),
        (None, None) => 0,
    })
}

/// `asString(default)`: a string as written; a bool `"true"` / `"false"`; a number, array or
/// object `""`. Every type but a string is warned.
pub fn string(value: Option<&Value>, at: &JsonPath, d: &mut Diagnostics) -> Option<String> {
    let value = present(value)?;
    if let Some(text) = value.as_string() {
        return Some(text.as_str().to_owned());
    }
    wrong_type(at, None, d);
    Some(match value.as_bool() {
        Some(true) => "true".to_owned(),
        Some(false) => "false".to_owned(),
        None => String::new(),
    })
}

/// `asBool(default)`: a bool as written; a number non-zero; a string, array or object non-empty.
/// Every type but a bool is warned.
pub fn boolean(value: Option<&Value>, at: &JsonPath, d: &mut Diagnostics) -> Option<bool> {
    let value = present(value)?;
    if let Some(b) = value.as_bool() {
        return Some(b);
    }
    wrong_type(at, None, d);
    Some(if let Some(bits) = integer_bits(value) {
        bits != 0
    } else if let Some(f) = real(value) {
        f != 0.0
    } else if let Some(text) = value.as_string() {
        !text.as_str().is_empty()
    } else if let Some(array) = value.as_array() {
        !array.is_empty()
    } else {
        value.as_object().is_some_and(|o| !o.is_empty())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What a reader makes of `token`, and whether it gave the one `InvalidValue` warning at
    /// the member.
    fn read<T>(
        reader: fn(Option<&Value>, &JsonPath, &mut Diagnostics) -> Option<T>,
        token: &str,
    ) -> (Option<T>, bool) {
        let value = bedrock_addon_core::json::from_str::<Value>(token).expect("a JSON value");
        let mut at = JsonPath::new();
        at.push("member");
        let mut d = Diagnostics::default();
        let read = reader(Some(&value), &at, &mut d);
        let told = d.iter().any(|finding| {
            matches!(finding.code, DiagnosticCode::InvalidValue { text: None })
                && finding.severity == Severity::Info
                && finding.location.json_path == at
        });
        assert_eq!(d.len(), usize::from(told), "{token}: {d:?}");
        (read, told)
    }

    #[test]
    fn an_integer_no_int_holds_keeps_its_low_32_bits_and_is_told() {
        assert_eq!(read(int, "2147483647"), (Some(i32::MAX), false));
        assert_eq!(read(int, "2147483648"), (Some(i32::MIN), true));
        assert_eq!(read(int, "4294967296"), (Some(0), true));
        assert_eq!(read(int, "-2147483648"), (Some(i32::MIN), false));
        assert_eq!(read(int, "-2147483649"), (Some(i32::MAX), true));
        assert_eq!(read(int, "18446744073709551615"), (Some(-1), true));
    }

    #[test]
    fn an_integer_no_unsigned_int_holds_keeps_its_low_32_bits_and_is_told() {
        assert_eq!(read(uint, "0"), (Some(0), false));
        assert_eq!(read(uint, "4294967295"), (Some(u32::MAX), false));
        assert_eq!(read(uint, "4294967296"), (Some(0), true));
        assert_eq!(read(uint, "-1"), (Some(u32::MAX), true));
    }

    #[test]
    fn a_negative_64_bit_integer_wraps_and_is_told() {
        assert_eq!(read(int64, "0"), (Some(0), false));
        assert_eq!(read(int64, "18446744073709551615"), (Some(u64::MAX), false));
        assert_eq!(read(int64, "-1"), (Some(u64::MAX), true));
    }

    #[test]
    fn a_null_is_the_default_of_the_caller_without_a_finding() {
        assert_eq!(read(int, "null"), (None, false));
        assert_eq!(read(uint, "null"), (None, false));
        assert_eq!(read(int64, "null"), (None, false));
    }
}
