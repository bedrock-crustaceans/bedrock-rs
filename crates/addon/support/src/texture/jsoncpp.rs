//! The game's JSON reader as the texture files see it: the lenient parse of one copy (comments,
//! any root, content after the root ignored) and the `Json::Value` accessors whose coercions
//! the atlas and flipbook readers reproduce on the merged value.

use bedrock_addon_core::json::{MAX_JSON_DEPTH, Value, check_depth, last_wins, strip_bom};

use crate::jsoncpp::ReadError;

/// One copy of a merge-before-parse file, read the way `Json::Reader::parse` reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct Lenient {
    /// The root value.
    pub value: Value,
    /// Whether anything but white space and comments follows the root value (vanilla never
    /// looks at it).
    pub trailing: bool,
}

/// Reads `bytes` as vanilla's reader does: a BOM is skipped, comments are allowed, any root
/// type is accepted and what follows the root value is ignored.
///
/// # Errors
///
/// A [`ReadError`] for nesting past the limit, bytes that are no UTF-8, or text that is no JSON.
// jsoncpp's parse with comments allowed and a lenient root, never checking for the end of
// input.
pub fn read_lenient(bytes: &[u8]) -> Result<Lenient, ReadError> {
    // The BOM is blanked, not cut, so every offset is an offset into the file as written.
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH)?;
    let text = std::str::from_utf8(bytes)?;
    let (root, trailing) = match root_end(text) {
        Some(end) => {
            let (root, rest) = text.split_at(end);
            (root, !only_trivia(rest))
        }
        None => (text, false),
    };
    // Of a member written twice the last stands, as in jsoncpp's `std::map`.
    let value = bedrock_addon_core::json::from_str_jsonc::<Value>(&last_wins(root))?;
    Ok(Lenient { value, trailing })
}

/// The byte offset just past the first JSON value of `text`, when its brackets and strings
/// close; `None` leaves the whole text to the reader (and its error).
pub fn root_end(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut at = skip_trivia(bytes, 0);
    let mut depth = 0usize;
    loop {
        match *bytes.get(at)? {
            b'"' => at = string_end(bytes, at)?,
            b'{' | b'[' => {
                depth += 1;
                at += 1;
            }
            b'}' | b']' => {
                depth = depth.checked_sub(1)?;
                at += 1;
            }
            b',' | b':' if depth > 0 => at += 1,
            b',' | b':' => return None,
            _ => {
                // A number or a literal: up to the next structural byte or white space.
                let start = at;
                while bytes
                    .get(at)
                    .is_some_and(|byte| !byte.is_ascii_whitespace() && !b"{}[],:\"/".contains(byte))
                {
                    at += 1;
                }
                if at == start {
                    return None;
                }
            }
        }
        if depth == 0 {
            return Some(at);
        }
        at = skip_trivia(bytes, at);
    }
}

/// The offset just past the string that opens at `at`.
pub fn string_end(bytes: &[u8], at: usize) -> Option<usize> {
    let mut at = at + 1;
    loop {
        match *bytes.get(at)? {
            b'"' => return Some(at + 1),
            b'\\' => at += 2,
            _ => at += 1,
        }
    }
}

/// The offset of the first byte at or after `at` that is neither white space nor inside a
/// comment.
pub fn skip_trivia(bytes: &[u8], mut at: usize) -> usize {
    loop {
        match (bytes.get(at), bytes.get(at + 1)) {
            (Some(byte), _) if byte.is_ascii_whitespace() => at += 1,
            (Some(b'/'), Some(b'/')) => {
                while bytes.get(at).is_some_and(|byte| *byte != b'\n') {
                    at += 1;
                }
            }
            (Some(b'/'), Some(b'*')) => {
                at += 2;
                while at < bytes.len() && bytes.get(at..at + 2) != Some(b"*/") {
                    at += 1;
                }
                at = (at + 2).min(bytes.len());
            }
            _ => return at,
        }
    }
}

/// Whether `text` holds white space and comments only.
fn only_trivia(text: &str) -> bool {
    skip_trivia(text.as_bytes(), 0) >= text.len()
}

/// `Json::Value::asInt(default)`: `null` is the default; an integer keeps its low 32 bits; a
/// real is truncated toward zero and saturates; a bool is 0 or 1; a string, an array or an
/// object is 0.
pub fn as_int(value: Option<&Value>, default: i32) -> i32 {
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
        return saturating(number.to_f64_lossy());
    }
    // An integer token fits 64 bits: past them the reader hands over a real, which took the
    // branch above.
    number.to_i64().map_or_else(
        || low_word(number.to_u64().unwrap_or_default().to_le_bytes()),
        |wide| low_word(wide.to_le_bytes()),
    )
}

/// The low 32 bits of a 64-bit integer, as `asInt` loads them.
fn low_word(bytes: [u8; 8]) -> i32 {
    let [a, b, c, d, ..] = bytes;
    i32::from_le_bytes([a, b, c, d])
}

/// The game's float-to-integer conversion: toward zero, saturating at the `i32` bounds, NaN
/// is 0.
#[allow(
    clippy::cast_possible_truncation,
    reason = "a float-to-int `as` saturates, which is the game's conversion"
)]
fn saturating(real: f64) -> i32 {
    real as i32
}

/// Whether `asInt` changes what `value` says: it is not `null`, and not an integer that fits
/// an `i32`.
pub fn int_is_coerced(value: &Value) -> bool {
    if value.is_null() {
        return false;
    }
    value
        .as_number()
        .filter(|number| !number.is_float())
        .and_then(facet_value::VNumber::to_i32)
        .is_none()
}

/// `Json::Value::asFloat(default)`: `null` is the default; a number is itself; a bool is 0 or
/// 1; a string, an array or an object is 0. A number beyond a float is the largest finite
/// float of its sign (this crate's choice; vanilla stores the infinity).
#[allow(
    clippy::cast_possible_truncation,
    reason = "vanilla stores the double as a float"
)]
pub fn as_float(value: Option<&Value>, default: f32) -> f32 {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return default;
    };
    if let Some(flag) = value.as_bool() {
        return if flag { 1.0 } else { 0.0 };
    }
    value
        .as_number()
        .map_or(0.0, |number| finite(number.to_f64_lossy() as f32))
}

/// `value`, or the largest finite float of its sign for an infinity: a number beyond a float
/// is never held as an infinity, which no JSON number writes back.
fn finite(value: f32) -> f32 {
    value.clamp(f32::MIN, f32::MAX)
}

/// Whether [`as_float`] holds `value` as the largest finite float instead of the number
/// written: a number whose nearest float is an infinity.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the question is what the cast gives"
)]
pub fn float_is_clamped(value: &Value) -> bool {
    value
        .as_number()
        .is_some_and(|number| (number.to_f64_lossy() as f32).is_infinite())
}

/// `Json::Value::asBool(default)`: `null` is the default; a number is `!= 0`; a string, an
/// array or an object is true when it is not empty (so `"false"` is true).
pub fn as_bool(value: Option<&Value>, default: bool) -> bool {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return default;
    };
    if let Some(flag) = value.as_bool() {
        flag
    } else if let Some(number) = value.as_number() {
        number.to_f64_lossy() != 0.0
    } else if let Some(text) = value.as_string() {
        !text.is_empty()
    } else if let Some(array) = value.as_array() {
        !array.is_empty()
    } else {
        value.as_object().is_some_and(|object| !object.is_empty())
    }
}

/// `Json::Value::asString("")`: a string is itself; a bool is `"true"` or `"false"`; `null`,
/// a number, an array or an object is `""`.
pub fn to_text(value: Option<&Value>) -> String {
    match value {
        Some(value) if value.is_string() => value
            .as_string()
            .map(|text| text.as_str().to_owned())
            .unwrap_or_default(),
        Some(value) => match value.as_bool() {
            Some(true) => "true".to_owned(),
            Some(false) => "false".to_owned(),
            None => String::new(),
        },
        None => String::new(),
    }
}

/// `Json::Value::empty()`: `null`, an empty array or an empty object. A string, even `""`, is
/// never empty.
pub fn is_empty(value: Option<&Value>) -> bool {
    match value {
        None => true,
        Some(value) => {
            value.is_null()
                || value.as_array().is_some_and(facet_value::VArray::is_empty)
                || value
                    .as_object()
                    .is_some_and(facet_value::VObject::is_empty)
        }
    }
}

/// A present member that is not `null` (`!isNull()`).
pub fn non_null(value: Option<&Value>) -> Option<&Value> {
    value.filter(|value| !value.is_null())
}

/// A hex string read as the game reads a colour: `strtoul` in base 16 with a range check.
/// Leading white space, a sign and a `0x` prefix are accepted, the parse stops at the first
/// byte that is no hex digit (no digit at all is 0), and a value past `0xFFFF_FFFF` is
/// `0xFFFF_FFFF`.
// `default_leather_color`.
pub fn strtouint32_hex(text: &str) -> u32 {
    let mut rest = text.trim_start_matches(|c: char| c.is_ascii_whitespace() || c == '\x0b');
    let negative = rest.starts_with('-');
    if let Some(unsigned) = rest.strip_prefix(['+', '-']) {
        rest = unsigned;
    }
    let prefixed = rest
        .strip_prefix("0x")
        .or_else(|| rest.strip_prefix("0X"))
        .filter(|digits| digits.starts_with(|c: char| c.is_ascii_hexdigit()));
    let digits = prefixed.unwrap_or(rest);
    let mut value: u64 = 0;
    let mut overflow = false;
    for digit in digits.chars().map_while(|c| c.to_digit(16)) {
        let (shifted, lost) = value.overflowing_mul(16);
        overflow |= lost;
        value = shifted | u64::from(digit);
    }
    if overflow {
        return u32::MAX;
    }
    if negative {
        value = value.wrapping_neg();
    }
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(text: &str) -> Value {
        bedrock_addon_core::json::from_str_jsonc::<Value>(text).unwrap()
    }

    #[test]
    fn the_root_ends_where_its_brackets_close() {
        assert_eq!(root_end(r#"{ "a": "}" } }"#), Some(12));
        assert_eq!(root_end("// c\n [1, [2]] x"), Some(14));
        assert_eq!(root_end(r#""a\"b" 1"#), Some(6));
        assert_eq!(root_end("null"), Some(4));
        assert_eq!(root_end("{ \"a\": 1"), None);
        assert_eq!(root_end("  "), None);
    }

    #[test]
    fn content_after_the_root_is_noticed_and_comments_are_not() {
        assert!(read_lenient(b"{} }").unwrap().trailing);
        assert!(!read_lenient(b"{} // done\n /* x */ ").unwrap().trailing);
        assert!(read_lenient(b"\xEF\xBB\xBF[]").unwrap().value.is_array());
        assert!(read_lenient(b"{ \"a\": ").is_err());
    }

    #[test]
    fn as_int_wraps_integers_and_saturates_reals() {
        assert_eq!(as_int(None, 7), 7);
        assert_eq!(as_int(Some(&value("null")), 7), 7);
        assert_eq!(as_int(Some(&value("4294967297")), 0), 1);
        assert_eq!(as_int(Some(&value("2147483648")), 0), i32::MIN);
        assert_eq!(as_int(Some(&value("-1")), 0), -1);
        assert_eq!(as_int(Some(&value("1.9")), 0), 1);
        assert_eq!(as_int(Some(&value("-1.9")), 0), -1);
        assert_eq!(as_int(Some(&value("1e12")), 0), i32::MAX);
        assert_eq!(as_int(Some(&value("true")), 0), 1);
        assert_eq!(as_int(Some(&value("\"5\"")), 1), 0);
        assert_eq!(as_int(Some(&value("[1]")), 1), 0);
    }

    #[test]
    fn an_integer_in_range_is_not_coerced() {
        assert!(!int_is_coerced(&value("5")));
        assert!(!int_is_coerced(&value("null")));
        assert!(int_is_coerced(&value("5.0")));
        assert!(int_is_coerced(&value("4294967297")));
        assert!(int_is_coerced(&value("true")));
    }

    #[test]
    fn as_bool_reads_any_non_empty_string_as_true() {
        assert!(as_bool(Some(&value("\"false\"")), false));
        assert!(!as_bool(Some(&value("\"\"")), true));
        assert!(as_bool(Some(&value("1")), false));
        assert!(!as_bool(Some(&value("0")), true));
        assert!(as_bool(Some(&value("[0]")), false));
        assert!(!as_bool(Some(&value("{}")), true));
        assert!(as_bool(None, true));
    }

    #[test]
    fn to_text_spells_bools_and_empties_the_rest() {
        assert_eq!(to_text(Some(&value("true"))), "true");
        assert_eq!(to_text(Some(&value("5"))), "");
        assert_eq!(to_text(Some(&value("\"x\""))), "x");
        assert_eq!(to_text(None), "");
    }

    #[test]
    fn only_null_and_empty_containers_are_empty() {
        assert!(is_empty(Some(&value("[]"))));
        assert!(is_empty(Some(&value("{}"))));
        assert!(is_empty(Some(&value("null"))));
        assert!(!is_empty(Some(&value("\"\""))));
        assert!(!is_empty(Some(&value("0"))));
    }

    #[test]
    fn strtouint32_follows_strtoul() {
        assert_eq!(strtouint32_hex("A06540"), 0x00A0_6540);
        assert_eq!(strtouint32_hex("  0xff"), 0xff);
        assert_eq!(strtouint32_hex("ed"), 0xed);
        assert_eq!(strtouint32_hex("rue"), 0);
        assert_eq!(strtouint32_hex("alse"), 0xa);
        assert_eq!(strtouint32_hex("1ffffffff"), u32::MAX);
        assert_eq!(strtouint32_hex("-1"), u32::MAX);
        assert_eq!(strtouint32_hex("0x"), 0);
        assert_eq!(strtouint32_hex("fffffffffffffffff"), u32::MAX);
    }
}
