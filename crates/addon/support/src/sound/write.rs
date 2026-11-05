//! Writing a sound document's typed value: each object's known members in declaration order,
//! then the members the client does not read.
//!
//! The client merges the copies of a sound file as JSON before it reads them, member by
//! member, so a member a copy does not write and one it writes with its default differ: the
//! first keeps what a lower pack wrote, the second replaces it. A struct therefore remembers
//! which of its known members it was read with ([`Present`]), and its writer writes those and
//! every other member whose value is not what an absent member reads as.

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::Written;
use bedrock_addon_core::json::{EncodeError, Value};
use facet_value::VObject;

/// The known members an object was read with, left out of equality (two values with the
/// same fields are equal whatever they were read from); `None` for a value built by hand.
pub type Present = Written<Vec<&'static str>>;

/// The members of `known` that the object `value` has, `null` ones included.
pub fn present(value: &Value, known: &[&'static str]) -> Present {
    let object = value.as_object();
    Written(Some(
        known
            .iter()
            .copied()
            .filter(|key| object.is_some_and(|object| object.contains_key(key)))
            .collect(),
    ))
}

/// An object being written from a struct read with the members `present`, whose members the
/// client does not read are `extra`.
#[derive(Debug)]
pub struct Object<'p> {
    members: VObject,
    present: &'p Present,
    extra: &'p UnknownMembers,
}

impl<'p> Object<'p> {
    /// An empty object for a struct read with the members `present`, with the members `extra`
    /// after its known ones. A known member `extra` holds (one the struct did not read, kept
    /// as written) is written from `extra`, not from the field.
    pub fn new(present: &'p Present, extra: &'p UnknownMembers) -> Self {
        Self {
            members: VObject::new(),
            present,
            extra,
        }
    }

    /// Whether the struct was read with the member `key`.
    pub fn was_read(&self, key: &str) -> bool {
        self.present
            .as_ref()
            .is_some_and(|keys| keys.contains(&key))
    }

    /// Writes `key` when the struct was read with it or `absent` is false (its value is not
    /// what an absent member reads as). `value` is asked for only then.
    ///
    /// # Errors
    ///
    /// The error of `value`, at its path below `key`.
    pub fn member(
        &mut self,
        key: &str,
        absent: bool,
        value: impl FnOnce() -> Result<Value, EncodeError>,
    ) -> Result<(), EncodeError> {
        if self.extra.get(key).is_none() && (self.was_read(key) || !absent) {
            self.members.insert(key, in_member(key, value())?);
        }
        Ok(())
    }

    /// The object, with the members of `extra` after the known ones.
    pub fn finish(mut self) -> Value {
        for (key, value) in self.extra.iter() {
            self.members.insert(key, value.clone());
        }
        self.members.into_value()
    }
}

/// An `f32` member: the shortest decimal that reads back as the same `f32`, written as a real.
///
/// An infinity is written as the largest double of its sign: the client reads the member as
/// a double and casts it to a float, which makes it the same infinity again (and a number no
/// float holds, such as `1e39` or `1e999`, loads as that infinity), so a document that loads
/// can be written.
///
/// # Errors
///
/// `EncodeError::NotRepresentable` when `x` is NaN (no document reads as one).
pub fn float(x: f32) -> Result<Value, EncodeError> {
    if x.is_infinite() {
        return Ok(Value::from(if x > 0.0 { f64::MAX } else { f64::MIN }));
    }
    crate::animation::write::float(x)
}

/// `result`, with the path of its error put below the member `key`.
///
/// # Errors
///
/// The error of `result`, at its path below `key`.
pub fn in_member<T>(key: &str, result: Result<T, EncodeError>) -> Result<T, EncodeError> {
    result.map_err(|error| below(&JsonPath::new().child(key), error))
}

/// `result`, with the path of its error put below the array element `index`.
///
/// # Errors
///
/// The error of `result`, at its path below `index`.
pub fn in_element<T>(index: usize, result: Result<T, EncodeError>) -> Result<T, EncodeError> {
    result.map_err(|error| below(&JsonPath::new().index(index), error))
}

/// `error`, at its path below `at`.
fn below(at: &JsonPath, error: EncodeError) -> EncodeError {
    match error {
        EncodeError::NotRepresentable { at: inner, what } => EncodeError::NotRepresentable {
            at: at.join(&inner),
            what,
        },
        other => other,
    }
}

/// An object of `members`, in order.
pub fn object<'k>(members: impl IntoIterator<Item = (&'k str, Value)>) -> Value {
    let mut object = VObject::new();
    for (key, value) in members {
        object.insert(key, value);
    }
    object.into_value()
}

/// An array of values.
pub fn array(values: impl IntoIterator<Item = Value>) -> Value {
    let mut array = facet_value::VArray::new();
    for value in values {
        array.push(value);
    }
    array.into_value()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json(text: &str) -> Value {
        bedrock_addon_core::json::from_str_jsonc(text).expect("JSON")
    }

    #[test]
    fn a_member_is_written_when_it_was_read_or_is_not_its_absent_reading() {
        let read = present(
            &json(r#"{"a": null, "b": 1, "z": 2}"#),
            &["a", "b", "c", "d"],
        );
        assert_eq!(read.0, Some(vec!["a", "b"]));
        let mut extra = UnknownMembers::default();
        extra.insert("z".to_owned(), Value::from(2));
        // A known member kept as written: written from `extra`, after the others.
        extra.insert("f".to_owned(), Value::from("as written"));
        let mut object = Object::new(&read, &extra);
        for key in ["d", "c", "b", "a"] {
            // Every value its absent reading: only those read are written.
            object
                .member(key, true, || Ok(Value::from(key)))
                .expect("written");
        }
        object
            .member("e", false, || Ok(Value::from(5)))
            .expect("written");
        object
            .member("f", false, || unreachable!("written from extra"))
            .expect("skipped");
        assert_eq!(
            object.finish(),
            json(r#"{"b": "b", "a": "a", "e": 5, "z": 2, "f": "as written"}"#)
        );
        // Built by hand: nothing was read.
        let built = Present::default();
        let none = UnknownMembers::default();
        let mut object = Object::new(&built, &none);
        object
            .member("a", true, || unreachable!("not asked for"))
            .expect("skipped");
        assert_eq!(object.finish(), json("{}"));
    }

    #[test]
    fn a_float_is_its_shortest_real() {
        let text = |x: f32| {
            let text = bedrock_addon_core::json::to_json(&float(x).expect("finite")).unwrap();
            text.trim().to_owned()
        };
        assert_eq!(text(0.1), "0.1");
        assert_eq!(text(60.0), "60.0");
        assert_eq!(text(-0.0), "-0.0");
        assert_eq!(text(f32::INFINITY), "1.7976931348623157e308");
        assert_eq!(text(f32::NEG_INFINITY), "-1.7976931348623157e308");
        assert!(float(f32::NAN).is_err());
    }

    #[test]
    fn a_refused_member_is_refused_at_its_path() {
        let read = Present::default();
        let none = UnknownMembers::default();
        let mut object = Object::new(&read, &none);
        let error = object
            .member("a", false, || in_element(2, float(f32::NAN)))
            .unwrap_err();
        assert!(
            matches!(&error, EncodeError::NotRepresentable { at, .. } if *at == JsonPath::new().child("a").index(2)),
            "{error:?}"
        );
    }
}
