//! The writing both file kinds share: members in declaration order, only those the value holds,
//! each in the form it was read.

use facet_value::{VArray, VObject};
use indexmap::IndexMap;

use super::types::{EventString, ParticleEffect, SoundEffect};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{EncodeError, Unrepresentable, Value};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::version::FormatVersion;

/// The error for a value the format cannot hold, at the root.
#[must_use]
pub fn refused(what: Unrepresentable) -> EncodeError {
    EncodeError::NotRepresentable {
        at: JsonPath::new(),
        what,
    }
}

/// An `f32` in the shortest decimal form that reads back as the same `f32`.
///
/// # Errors
///
/// `EncodeError::NotRepresentable` when `x` is NaN or infinite.
pub fn float(x: f32) -> Result<Value, EncodeError> {
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

/// A Molang value as a JSON value (a constant is a number, anything else is its source).
///
/// # Errors
///
/// `EncodeError::NotRepresentable` when the value is a constant that is not finite.
pub fn molang(value: &Molang) -> Result<Value, EncodeError> {
    match value {
        Molang::Const(x) => float(*x),
        other => bedrock_addon_core::molang::write(other).map_err(refused),
    }
}

/// A JSON string.
#[must_use]
pub fn string(text: &str) -> Value {
    Value::from(text)
}

/// A `format_version` as its JSON string.
#[must_use]
pub fn version(version: FormatVersion) -> Value {
    string(&version.to_string())
}

/// An object from the members that are present.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Object(VObject);

impl Object {
    /// An empty object.
    #[must_use]
    pub fn new() -> Self {
        Self(VObject::new())
    }

    /// Set `key` to `value`.
    pub fn put(&mut self, key: &str, value: Value) {
        self.0.insert(key, value);
    }

    /// Set `key` to what `write` makes of `value` when there is one.
    ///
    /// # Errors
    ///
    /// The error of `write` for the value, when there is one.
    pub fn opt<T>(
        &mut self,
        key: &str,
        value: Option<&T>,
        write: impl FnOnce(&T) -> Result<Value, EncodeError>,
    ) -> Result<(), EncodeError> {
        if let Some(value) = value {
            self.put(key, write(value)?);
        }
        Ok(())
    }

    /// The finished JSON object.
    #[must_use]
    pub fn done(self) -> Value {
        Value::from(self.0)
    }
}

/// A map as a JSON object, each value written by `write`.
///
/// # Errors
///
/// The first error of `write` for a value.
pub fn map<T>(
    map: &IndexMap<String, T>,
    mut write: impl FnMut(&T) -> Result<Value, EncodeError>,
) -> Result<Value, EncodeError> {
    let mut object = Object::new();
    for (key, value) in map {
        object.put(key, write(value)?);
    }
    Ok(object.done())
}

/// A slice as a JSON array, each element written by `write`.
///
/// # Errors
///
/// The first error of `write` for an element.
pub fn list<T>(
    list: &[T],
    write: impl FnMut(&T) -> Result<Value, EncodeError>,
) -> Result<Value, EncodeError> {
    list.iter()
        .map(write)
        .collect::<Result<VArray, _>>()
        .map(Value::from)
}

/// A map of floats as a JSON object.
///
/// # Errors
///
/// `EncodeError::NotRepresentable` when a value is NaN or infinite.
pub fn floats(curve: &IndexMap<String, f32>) -> Result<Value, EncodeError> {
    map(curve, |x| float(*x))
}

/// An event string as its JSON string.
///
/// # Errors
///
/// Never: the signature is the one every member writer has.
#[allow(
    clippy::unnecessary_wraps,
    reason = "the shape every member writer has"
)]
pub fn event(e: &EventString) -> Result<Value, EncodeError> {
    Ok(string(&e.to_text()))
}

/// A particle effect as its JSON object.
///
/// # Errors
///
/// The error of [`molang`] for `pre_effect_script`.
pub fn particle(p: &ParticleEffect) -> Result<Value, EncodeError> {
    let mut o = Object::new();
    o.put("effect", string(&p.effect));
    o.opt("locator", p.locator.as_ref(), |s| Ok(string(s)))?;
    o.opt("pre_effect_script", p.pre_effect_script.as_ref(), molang)?;
    o.opt("bind_to_actor", p.bind_to_actor.as_ref(), |b| {
        Ok(Value::from(*b))
    })?;
    Ok(o.done())
}

/// A sound effect as its JSON object.
///
/// # Errors
///
/// Never in practice: the members are strings, though the signature is the one every member
/// writer has.
pub fn sound(s: &SoundEffect) -> Result<Value, EncodeError> {
    let mut o = Object::new();
    o.put("effect", string(&s.effect));
    o.opt("locator", s.locator.as_ref(), |l| Ok(string(l)))?;
    Ok(o.done())
}
