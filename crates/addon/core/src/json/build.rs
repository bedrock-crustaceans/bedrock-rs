//! Building JSON values to write: an array, an object in insertion order, and a value as a
//! file's JSON text (the writers of the fixed-path files).

use super::{EncodeError, Unrepresentable, Value, WriteStyle, write_json};
use crate::diagnostic::JsonPath;
use crate::document::UnknownMembers;

/// An array of values.
pub fn array(values: impl IntoIterator<Item = Value>) -> Value {
    let mut array = facet_value::VArray::new();
    for value in values {
        array.push(value);
    }
    array.into_value()
}

/// An object under construction, members in insertion order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Object(facet_value::VObject);

impl Object {
    /// Adds `key` when `value` is present.
    pub fn put(&mut self, key: &str, value: Option<Value>) {
        if let Some(value) = value {
            self.0.insert(key, value);
        }
    }

    /// Adds the unknown members.
    pub fn extra(&mut self, extra: &UnknownMembers) {
        for (key, value) in extra.iter() {
            self.0.insert(key, value.clone());
        }
    }

    /// The object.
    pub fn finish(self) -> Value {
        self.0.into_value()
    }
}

/// Writes a value as a file's JSON.
///
/// # Errors
///
/// `EncodeError::NotRepresentable` when `value` holds a non-finite number, or nests deeper than the
/// readers of the crate follow.
pub fn json_text(value: &Value, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
    write_json(value, out, style)
}

/// A value as a file's JSON text.
///
/// # Errors
///
/// `EncodeError::NotRepresentable` when `value` holds a non-finite number, or nests deeper than the
/// readers of the crate follow.
pub fn json_string(value: &Value) -> Result<String, EncodeError> {
    super::to_json(value)
}

/// An `f32` in the shortest decimal form that reads back as the same `f32`.
///
/// # Errors
///
/// `EncodeError::NotRepresentable` when `x` is NaN or infinite.
pub fn float(x: f32) -> Result<Value, EncodeError> {
    let refused = |what| EncodeError::NotRepresentable {
        at: JsonPath::new(),
        what,
    };
    if x.is_nan() {
        return Err(refused(Unrepresentable::NaN));
    }
    if x.is_infinite() {
        return Err(refused(if x > 0.0 {
            Unrepresentable::Infinity
        } else {
            Unrepresentable::NegativeInfinity
        }));
    }
    let shortest = x.to_string().parse::<f64>().unwrap_or(f64::from(x));
    Ok(Value::from(shortest))
}

#[cfg(test)]
mod tests {
    fn assert_debug<T: std::fmt::Debug>() {}

    #[test]
    fn a_float_is_its_shortest_decimal_and_never_non_finite() {
        let text = |x: f32| super::super::to_json(&super::float(x).unwrap()).unwrap();
        assert_eq!(text(0.1).trim(), "0.1");
        assert_eq!(text(60.0).trim(), "60.0");
        assert_eq!(text(-0.0).trim(), "-0.0");
        for (x, what) in [
            (f32::NAN, super::Unrepresentable::NaN),
            (f32::INFINITY, super::Unrepresentable::Infinity),
            (f32::NEG_INFINITY, super::Unrepresentable::NegativeInfinity),
        ] {
            assert!(matches!(
                super::float(x),
                Err(super::EncodeError::NotRepresentable { what: refused, .. }) if refused == what
            ));
        }
    }

    #[test]
    fn public_types_implement_debug() {
        assert_debug::<super::Object>();
    }
}
