//! The strict bool: a `bool` member that takes only `true` and `false`.
//!
//! A plain member (`u8`, `f32`, `bool`, `String`) is read by its slice's
//! [`Reading`](crate::versioning::Reading), which is how every family states strictness.
//! [`StrictBool`] stays for `shared::MenuCategory::is_hidden_in_commands`, which reads
//! `true` or `false` whatever the reading's scalars say; it is a `#[facet(proxy)]` over
//! [`ScalarShape`] so that the writer writes it as the `bool` it holds.

use facet::Facet;

use super::scalar::{ScalarShape, StrictError};

impl ScalarShape {
    /// The shape of a scalar `value`, as the parser reads the same token: an integer is
    /// [`UInt`](Self::UInt) when it fits `u64` and [`Int`](Self::Int) when it is negative and fits
    /// `i64`; any other number (a real, an integer past 64 bits) is [`Number`](Self::Number).
    /// `None` for an array or an object.
    #[must_use]
    pub fn of(value: &super::Value) -> Option<Self> {
        if value.is_null() {
            return Some(Self::Null);
        }
        if let Some(text) = value.as_string() {
            return Some(Self::String(text.as_str().to_owned()));
        }
        if let Some(flag) = value.as_bool() {
            return Some(Self::Bool(flag));
        }
        let number = value.as_number()?;
        if number.is_integer() {
            if let Some(unsigned) = number.to_u64() {
                return Some(Self::UInt(unsigned));
            }
            if let Some(signed) = number.to_i64() {
                return Some(Self::Int(signed));
            }
        }
        Some(Self::Number(number.to_f64()?))
    }
}

/// A boolean field that takes only `true` and `false`: `1`, `"true"` and `null` are rejected.
/// JSON: a boolean.
///
/// The value is the public field: `StrictBool(true)` builds one and `.0` reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
#[facet(proxy = ScalarShape)]
pub struct StrictBool(pub bool);

impl From<bool> for StrictBool {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl TryFrom<ScalarShape> for StrictBool {
    type Error = StrictError;

    fn try_from(json: ScalarShape) -> Result<Self, StrictError> {
        match json {
            ScalarShape::Bool(b) => Ok(StrictBool(b)),
            other => Err(StrictError::WrongType {
                expected: "boolean",
                found: other.json_type(),
            }),
        }
    }
}

impl From<&StrictBool> for ScalarShape {
    fn from(value: &StrictBool) -> Self {
        ScalarShape::Bool(value.0)
    }
}
