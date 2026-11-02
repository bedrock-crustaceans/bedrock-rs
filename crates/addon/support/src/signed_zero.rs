//! The text of a JSON value for a conversion that types it by writing and reading it, with
//! the sign of a negative zero kept wherever the target reads it.

use bedrock_addon_core::diagnostic::{JsonPath, JsonPathSegment};
use bedrock_addon_core::json::{JsonSlice, Value, parse_at, to_facet_json, to_facet_json_signed};

/// The value at `path` below `value`, to change.
fn at_mut<'v>(value: &'v mut Value, path: &JsonPath) -> Option<&'v mut Value> {
    let mut at = value;
    for segment in path.segments() {
        at = match segment {
            JsonPathSegment::Key(key) => at.as_object_mut()?.get_mut(key)?,
            JsonPathSegment::Index(index) => at.as_array_mut()?.get_mut(index)?,
        };
    }
    Some(at)
}

/// Whether `value` is a real negative zero.
fn is_negative_zero(value: &Value) -> bool {
    value.as_number().is_some_and(|number| {
        let real = number.to_f64_lossy();
        number.is_float() && real == 0.0 && real.is_sign_negative()
    })
}

/// `value` as the conversion writes it (one line, a whole real as an integer), with each real
/// negative zero written `-0.0` where `T` reads the text so (a float member then holds the
/// zero of the file) and `0` where it does not (an integer member, which takes the integer).
/// The text of [`to_facet_json`] when `T` reads neither.
#[must_use]
pub fn text_for<T: for<'a> facet::Facet<'a>>(value: &Value) -> Option<String> {
    let plain = to_facet_json(value).ok()?;
    let mut signed = to_facet_json_signed(value).ok()?;
    if signed == plain {
        return Some(plain);
    }
    // Each round makes one negative zero an integer, so the loop ends.
    let mut value = value.clone();
    loop {
        let Err(error) = parse_at::<T>(JsonSlice {
            text: &signed,
            offset: 0,
        }) else {
            return Some(signed);
        };
        match at_mut(&mut value, &error.path) {
            Some(zero) if is_negative_zero(zero) => *zero = Value::from(0_i64),
            _ => return Some(plain),
        }
        signed = to_facet_json_signed(&value).ok()?;
    }
}
