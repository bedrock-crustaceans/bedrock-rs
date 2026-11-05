//! Writing a settings payload: one object at a time, the members the value holds and nothing
//! else, each in a form the loader of this module reads back. A value the loader
//! would refuse (outside its range, not finite, keyframes it drops) is not written: the
//! writer returns `EncodeError::NotRepresentable` with the member's path.

use facet_value::{VArray, VObject};

use super::read::Range;
use super::value::{Description, Keyframes, Rgb8, Rgba8, Vec3};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::Kind;
use bedrock_addon_core::json::{EncodeError, Unrepresentable, Value};

/// The outcome of writing one value.
pub(crate) type Written<T = Value> = Result<T, EncodeError>;

/// The refusal of the value at `at`.
pub(crate) fn refuse(at: &JsonPath) -> EncodeError {
    EncodeError::NotRepresentable {
        at: at.clone(),
        what: Unrepresentable::Unsupported,
    }
}

/// A JSON object being written, at a path.
pub(crate) struct Object {
    map: VObject,
    at: JsonPath,
}

impl Object {
    /// An empty object at `at`.
    pub(crate) fn new(at: &JsonPath) -> Self {
        Self {
            map: VObject::new(),
            at: at.clone(),
        }
    }

    /// The path of the member `key`.
    pub(crate) fn path(&self, key: &str) -> JsonPath {
        self.at.child(key)
    }

    /// Writes the member `key` when `value` holds one, by `write` (which gets the member's
    /// path).
    pub(crate) fn opt<T>(
        &mut self,
        key: &str,
        value: Option<&T>,
        write: impl FnOnce(&T, &JsonPath) -> Written,
    ) -> Written<()> {
        if let Some(value) = value {
            self.req(key, value, write)?;
        }
        Ok(())
    }

    /// Writes the member `key`.
    pub(crate) fn req<T>(
        &mut self,
        key: &str,
        value: &T,
        write: impl FnOnce(&T, &JsonPath) -> Written,
    ) -> Written<()> {
        let written = write(value, &self.path(key))?;
        self.map.insert(key, written);
        Ok(())
    }

    /// Ends the object with its unknown members, after the known ones. An unknown member
    /// named like a member of `known` would be read back as that member: refused.
    pub(crate) fn finish(mut self, extra: &UnknownMembers, known: &[&str]) -> Written {
        for (key, value) in extra.iter() {
            if known.contains(&key) {
                return Err(refuse(&self.path(key)));
            }
            self.map.insert(key, value.clone());
        }
        Ok(self.map.into())
    }
}

/// A number as the shortest text that reads back to the same `f32`; a value that is not
/// finite is refused.
pub(crate) fn float(value: f32, at: &JsonPath) -> Written {
    if !value.is_finite() {
        return Err(refuse(at));
    }
    // `{value}` is the shortest decimal that parses to this `f32`; as a double it is the
    // number the author wrote, not the widened float (`0.02`, not `0.0199999995529651…`).
    match value.to_string().parse::<f64>() {
        Ok(shortest) => Ok(Value::from(shortest)),
        Err(_) => Err(refuse(at)),
    }
}

/// A number in `range`; one outside it is refused (the loader refuses the file).
pub(crate) fn float_in(range: Range, value: f32, at: &JsonPath) -> Written {
    if range.contains(value) {
        float(value, at)
    } else {
        Err(refuse(at))
    }
}

/// An integer in `[min, max]`; one outside it is refused.
pub(crate) fn integer_in(min: i64, max: i64, value: i64, at: &JsonPath) -> Written {
    if (min..=max).contains(&value) {
        Ok(Value::from(value))
    } else {
        Err(refuse(at))
    }
}

/// The name of `value` in `table`.
pub(crate) fn choice<T: Copy + PartialEq>(table: &[(&str, T)], value: T, at: &JsonPath) -> Written {
    table
        .iter()
        .find(|(_, known)| *known == value)
        .map(|(name, _)| Value::from(*name))
        .ok_or_else(|| refuse(at))
}

/// An array of channel values.
fn channels(values: &[u8]) -> Value {
    let mut array = VArray::new();
    for value in values {
        array.push(Value::from(i64::from(*value)));
    }
    array.into()
}

/// `[r, g, b]`.
#[allow(
    clippy::unnecessary_wraps,
    clippy::trivially_copy_pass_by_ref,
    clippy::ptr_arg,
    reason = "the shape every member writer has"
)]
pub(crate) fn rgb(value: &Rgb8, _: &JsonPath) -> Written {
    Ok(channels(&[value.red, value.green, value.blue]))
}

/// `[r, g, b, a]`.
#[allow(
    clippy::unnecessary_wraps,
    clippy::trivially_copy_pass_by_ref,
    clippy::ptr_arg,
    reason = "the shape every member writer has"
)]
pub(crate) fn rgba(value: &Rgba8, _: &JsonPath) -> Written {
    Ok(channels(&[value.red, value.green, value.blue, value.alpha]))
}

/// The key of a keyframe at `time`, as the loader reads it back to the same `f32`.
pub(crate) fn time_key(time: f32) -> String {
    format!("{time:?}")
}

/// A keyframed value: the single value, or the object of its keyframes. Keyframes the loader
/// would not read back as they are refused: none at all, a time outside `[0, 1]`, two of one
/// time.
pub(crate) fn keyframes<T>(
    value: &Keyframes<T>,
    at: &JsonPath,
    write: impl Fn(&T, &JsonPath) -> Written,
) -> Written {
    match value {
        Keyframes::Constant(value) => write(value, at),
        Keyframes::Frames(frames) => {
            if frames.is_empty() {
                return Err(refuse(at));
            }
            let mut map = VObject::new();
            for frame in frames {
                if !(0.0..=1.0).contains(&frame.time) {
                    return Err(refuse(at));
                }
                // `-0.0` is the time `0.0`.
                let key = time_key(frame.time + 0.0);
                let written = write(&frame.value, &at.child(&key))?;
                if map.insert(key.as_str(), written).is_some() {
                    return Err(refuse(at));
                }
            }
            Ok(map.into())
        }
    }
}

/// `[x, y, z]`, each component in `range`.
pub(crate) fn vec3_in(range: Range, value: &Vec3, at: &JsonPath) -> Written {
    let mut array = VArray::new();
    for (index, component) in [value.x, value.y, value.z].into_iter().enumerate() {
        let path = at.index(index);
        array.push(float_in(range, component, &path)?);
    }
    Ok(array.into())
}

/// `true` or `false`.
#[allow(
    clippy::unnecessary_wraps,
    clippy::trivially_copy_pass_by_ref,
    clippy::ptr_arg,
    reason = "the shape every member writer has"
)]
pub(crate) fn boolean(value: &bool, _: &JsonPath) -> Written {
    Ok(Value::from(*value))
}

/// A string.
#[allow(
    clippy::unnecessary_wraps,
    clippy::trivially_copy_pass_by_ref,
    clippy::ptr_arg,
    reason = "the shape every member writer has"
)]
pub(crate) fn string(value: &String, _: &JsonPath) -> Written {
    Ok(Value::from(value.as_str()))
}

/// A `description` object.
pub(crate) fn description<K: Kind>(value: &Description<K>, at: &JsonPath) -> Written {
    let mut object = Object::new(at);
    object.req("identifier", &value.identifier, |id, _| {
        Ok(Value::from(id.as_str()))
    })?;
    object.finish(&value.extra, &["identifier"])
}

/// The path of the payload member `key`, below which a struct's writer reports.
pub(crate) fn payload(key: &str) -> JsonPath {
    JsonPath::new().child(key)
}
