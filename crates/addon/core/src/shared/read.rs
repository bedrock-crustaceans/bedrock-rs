//! Reading the shared types from a `json::Value` and writing them back.
//!
//! Each shared type is a `#[facet(opaque, proxy = json::Value)]`: facet reads the value's JSON
//! from the text into a `Value` (which keeps every member, the key order and the int/real number
//! tag) and the type's conversion reads that `Value` by hand. A typed repr cannot do it: facet's
//! untagged solver ignores unknown keys and coerces scalars, and vanilla's
//! readers distinguish `"1"`, `1` and `1.0`, keep a rejected entry (`Invalid`) and keep unknown
//! members for write-back. The unknown-member walk does not descend into a proxied type,
//! so these conversions keep the unknown members themselves, in the type's `extra`.

use molang::json::{MolangSource, SourceForm};

use crate::diagnostic::JsonPath;
use crate::json::{Indent, Value, ValueExt, WriteStyle, write_json};
use crate::molang::JsonType;

/// Why a shared value's JSON could not be read, or a value could not be written as JSON: what
/// failed, and where inside the value.
///
/// facet reports a read failure with this text (the path included) at the span of the member
/// that holds the value; its diagnostic code is `InvalidType`.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub struct ShapeError {
    /// Where inside the value, relative to it (`all_of/1/subject`); empty for the value itself.
    pub path: JsonPath,
    /// What failed.
    pub kind: ShapeErrorKind,
}

impl std::fmt::Display for ShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.kind)
        } else {
            write!(f, "at `{}`: {}", self.path, self.kind)
        }
    }
}

/// What a [`ShapeError`] found.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ShapeErrorKind {
    /// A value of a JSON type the member does not take.
    #[error("expected {expected}, found a JSON {found}")]
    WrongType {
        /// What the member takes, in words.
        expected: &'static str,
        /// What the document has.
        found: JsonType,
    },
    /// A required member is absent from the object at the path.
    #[error("missing member `{member}`")]
    MissingMember {
        /// The member.
        member: &'static str,
    },
    /// A name that is not one of the member's spellings (a filter subject, a creative
    /// category), or a member the type has no place for (`tags` in a block specifier).
    #[error("not one of the accepted names")]
    UnknownName,
    /// A number of the right type that does not fit the member.
    #[error("the number does not fit the member's type")]
    OutOfRange,
    /// A range array with too few elements (or, for a float range, not exactly two).
    #[error("a range array needs {expected} elements, found {len}")]
    ArrayLength {
        /// What the reader takes, in words.
        expected: &'static str,
        /// The array's length.
        len: usize,
    },
    /// A value with no JSON spelling: a non-finite float, or a creative category that vanilla
    /// never writes (`All`).
    #[error("the value has no JSON form")]
    NotRepresentable,
}

/// A [`ShapeError`] while it travels out of nested readers: the steps are innermost first, so
/// each reader adds its own step on the way out.
#[derive(Debug)]
#[doc(hidden)]
pub struct Failure {
    kind: ShapeErrorKind,
    steps: Vec<Step>,
}

/// One step of a [`Failure`]'s path.
#[derive(Debug)]
enum Step {
    Key(Box<str>),
    Index(usize),
}

impl Failure {
    /// A failure of the value itself.
    #[doc(hidden)]
    pub fn new(kind: ShapeErrorKind) -> Self {
        Self {
            kind,
            steps: Vec::new(),
        }
    }

    /// The failure seen from one object further out: it happened inside member `key`. [`ROOT`]
    /// adds no step.
    #[must_use]
    #[doc(hidden)]
    pub fn key(mut self, key: &str) -> Self {
        if key != ROOT {
            self.steps.push(Step::Key(key.into()));
        }
        self
    }

    /// The failure seen from one array further out: it happened inside element `index`.
    #[must_use]
    #[doc(hidden)]
    pub fn index(mut self, index: usize) -> Self {
        self.steps.push(Step::Index(index));
        self
    }
}

impl From<Failure> for ShapeError {
    fn from(failure: Failure) -> Self {
        let mut path = JsonPath::new();
        for step in failure.steps.iter().rev() {
            match step {
                Step::Key(key) => path.push(key),
                Step::Index(index) => path.push_index(*index),
            }
        }
        Self {
            path,
            kind: failure.kind,
        }
    }
}

/// The member name used when the failing value is the value itself.
#[doc(hidden)]
pub const ROOT: &str = "<value>";

/// A failure of `kind` in member `member` ([`ROOT`]: the value itself).
#[doc(hidden)]
pub fn fail(member: &'static str, kind: ShapeErrorKind) -> Failure {
    Failure::new(kind).key(member)
}

/// [`ShapeErrorKind::WrongType`] in member `member` for `found`.
#[doc(hidden)]
pub fn wrong_type(member: &'static str, expected: &'static str, found: &Value) -> Failure {
    fail(
        member,
        ShapeErrorKind::WrongType {
            expected,
            found: found.json_type(),
        },
    )
}

/// A string member.
#[doc(hidden)]
pub fn string(member: &'static str, value: &Value) -> Result<String, Failure> {
    value
        .as_string()
        .map(|s| s.as_str().to_owned())
        .ok_or_else(|| wrong_type(member, "a string", value))
}

/// An integer token that fits `i32` (jsoncpp `isInt`; a real token is a real even when it is
/// integral).
#[doc(hidden)]
pub fn int(member: &'static str, value: &Value) -> Result<i32, Failure> {
    match value.as_number() {
        Some(number) if !number.is_float() => number
            .to_i64()
            .and_then(|i| i32::try_from(i).ok())
            .ok_or_else(|| fail(member, ShapeErrorKind::OutOfRange)),
        _ => Err(wrong_type(member, "an integer", value)),
    }
}

/// A number token (integer or real) that is finite as an `f32` (jsoncpp `isDouble`).
#[doc(hidden)]
pub fn float(member: &'static str, value: &Value) -> Result<f32, Failure> {
    let number = value
        .as_number()
        .ok_or_else(|| wrong_type(member, "a number", value))?;
    #[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
    let x = number.to_f64_lossy() as f32;
    if x.is_finite() {
        Ok(x)
    } else {
        Err(fail(member, ShapeErrorKind::OutOfRange))
    }
}

/// An `f32` as a real-tagged JSON number in its shortest form: the `f64` nearest to the `f32`'s
/// shortest decimal, so `0.1_f32` is written `0.1`, not `0.10000000149011612`.
#[doc(hidden)]
pub fn f32_value(x: f32) -> Result<Value, Failure> {
    let not_representable = || Failure::new(ShapeErrorKind::NotRepresentable);
    if !x.is_finite() {
        return Err(not_representable());
    }
    let shortest: f64 = format!("{x:?}").parse().map_err(|_| not_representable())?;
    Ok(Value::from(shortest))
}

/// A Molang member: a string, or the object form `{"expression": string, "version": integer}`
/// (molang's `MolangSource` reader). `None` for any other value.
#[doc(hidden)]
pub fn molang_source(value: &Value) -> Option<MolangSource> {
    if let Some(text) = value.as_string() {
        return Some(MolangSource::string_without_context(text.as_str()));
    }
    let object = value.as_object()?;
    let expression = object.get("expression")?.as_string()?;
    let version = object.get("version")?.as_number()?;
    if version.is_float() {
        return None;
    }
    let version = i16::try_from(version.to_i64()?).ok()?;
    Some(MolangSource::object(expression.as_str(), version))
}

/// A Molang member as JSON, in the form it was read.
#[doc(hidden)]
pub fn molang_value(source: &MolangSource) -> Value {
    match source.form() {
        SourceForm::Object { version } => {
            let mut object = facet_value::VObject::new();
            object.insert("expression", Value::from(source.as_str()));
            object.insert("version", Value::from(i64::from(version.0)));
            Value::from(object)
        }
        _ => Value::from(source.as_str()),
    }
}

/// `value` as one line of JSON, the way a vanilla message prints a rejected value.
#[doc(hidden)]
pub fn compact(value: &Value) -> String {
    let mut out = Vec::new();
    let style = WriteStyle {
        indent: Indent::Compact,
        trailing_newline: false,
    };
    // A `Value` read from JSON text always has a JSON form.
    let _ = write_json(value, &mut out, style);
    String::from_utf8(out).unwrap_or_default()
}

/// jsoncpp's `ValueType` number of `value`, as vanilla prints a state type.
#[doc(hidden)]
pub fn jsoncpp_type(value: &Value) -> i32 {
    match value.as_number() {
        // jsoncpp's reader keeps every integer that fits `Int64` as `intValue`.
        Some(n) if n.is_float() => 3,
        Some(n) if n.to_i64().is_some() => 1,
        Some(_) => 2,
        None => match value.json_type() {
            JsonType::Null => 0,
            JsonType::String => 4,
            JsonType::Bool => 5,
            JsonType::Array => 6,
            _ => 7,
        },
    }
}
