//! `IntRange` / `FloatRange`: an inclusive range in any of the JSON forms vanilla's readers take.

use facet_value::{VArray, VObject};

use super::read::{self, Failure, ROOT, ShapeError, ShapeErrorKind, wrong_type};
use super::{finding, sealed};
use crate::diagnostic::{DiagnosticCode, Diagnostics, JsonPath, Severity};
use crate::json::Value;

/// An inclusive range of `T`: the game's integer and float ranges.
///
/// JSON: a number (both ends), a two-element array `[min, max]`, `{"min", "max"}` (the cereal
/// readers) or `{"range_min", "range_max"}` (the readers of the game's older schema tree). Every
/// form is read; which one a member's vanilla reader takes is the using family's rule. `form` keeps the form
/// read, so writing reproduces it.
///
/// When the document writes `max < min`, the ends are swapped on reading, as vanilla's reader
/// does: `min ≤ max` holds whenever both ends were written. An object may leave an end out;
/// it then holds `T::default()` and the member's default applies ([`Range::resolve`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, facet::Facet)]
#[facet(opaque, proxy = Value)]
#[doc(alias = "range")]
pub struct Range<T: RangeEnd> {
    /// The lower end (JSON `min`, `range_min` or the first array element).
    pub min: T,
    /// The upper end (JSON `max`, `range_max` or the second array element).
    pub max: T,
    /// The JSON form read; writing uses it.
    pub form: RangeForm,
    /// The document wrote the ends in the wrong order; they were swapped, and writing restores
    /// the document's order.
    pub swapped: bool,
    /// The object form left out the lower end.
    pub missing_min: bool,
    /// The object form left out the upper end.
    pub missing_max: bool,
}

/// An integer range; ends are integer tokens only (`1.0` and `"1"` are rejected).
pub type IntRange = Range<i32>;

/// A float range; ends are numbers (integer or real tokens).
pub type FloatRange = Range<f32>;

/// The JSON form a [`Range`] was read from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RangeForm {
    /// A number: both ends.
    Scalar,
    /// `[min, max]`.
    Array,
    /// `{"min": …, "max": …}`: the cereal readers.
    #[default]
    MinMax,
    /// `{"range_min": …, "range_max": …}`: the readers of the game's older schema tree.
    RangeMinMax,
}

/// The end types of a [`Range`]: `i32` and `f32`.
pub trait RangeEnd: Copy + Default + PartialOrd + sealed::Sealed {
    /// Reads one end; `member` names it in the error.
    // hidden: methods of a public trait, which cannot be narrowed; only core implements them
    #[doc(hidden)]
    fn read_end(member: &'static str, value: &Value) -> Result<Self, ShapeError>;
    /// Writes one end.
    // hidden: methods of a public trait, which cannot be narrowed; only core implements them
    #[doc(hidden)]
    fn write_end(self) -> Result<Value, ShapeError>;
    /// Reads the array form.
    // hidden: methods of a public trait, which cannot be narrowed; only core implements them
    #[doc(hidden)]
    fn read_array(array: &VArray) -> Result<(Self, Self), ShapeError>;
}

impl sealed::Sealed for i32 {}
impl sealed::Sealed for f32 {}

impl RangeEnd for i32 {
    fn read_end(member: &'static str, value: &Value) -> Result<Self, ShapeError> {
        read::int(member, value).map_err(Into::into)
    }

    fn write_end(self) -> Result<Value, ShapeError> {
        Ok(Value::from(i64::from(self)))
    }

    fn read_array(array: &VArray) -> Result<(Self, Self), ShapeError> {
        // An array of at least two elements is `[a[0], a[1]]`.
        match array.as_slice() {
            [min, max, ..] => Ok((
                read::int(ROOT, min).map_err(|f| f.index(0))?,
                read::int(ROOT, max).map_err(|f| f.index(1))?,
            )),
            short => Err(Failure::new(ShapeErrorKind::ArrayLength {
                expected: "at least 2",
                len: short.len(),
            })
            .into()),
        }
    }
}

impl RangeEnd for f32 {
    fn read_end(member: &'static str, value: &Value) -> Result<Self, ShapeError> {
        read::float(member, value).map_err(Into::into)
    }

    fn write_end(self) -> Result<Value, ShapeError> {
        read::f32_value(self).map_err(Into::into)
    }

    fn read_array(array: &VArray) -> Result<(Self, Self), ShapeError> {
        // Any other length is "Float range array was not parsed. Float range arrays should
        // have 2 values."
        match array.as_slice() {
            [min, max] => Ok((
                read::float(ROOT, min).map_err(|f| f.index(0))?,
                read::float(ROOT, max).map_err(|f| f.index(1))?,
            )),
            other => Err(Failure::new(ShapeErrorKind::ArrayLength {
                expected: "exactly 2",
                len: other.len(),
            })
            .into()),
        }
    }
}

/// Whether `shape` is a [`Range`]'s.
pub(crate) fn is_range(shape: &'static facet::Shape) -> bool {
    use facet::Facet;
    shape.id == IntRange::SHAPE.id || shape.id == FloatRange::SHAPE.id
}

/// The members of a range's object form that the range does not read: every key but the two
/// ends of the form it is read as (`range_min` / `range_max` when one of them is there, else
/// `min` / `max`), in document order.
pub(crate) fn unread_range_members(object: &VObject) -> Vec<&str> {
    let json_util = object.contains_key("range_min") || object.contains_key("range_max");
    let ends = if json_util {
        ["range_min", "range_max"]
    } else {
        ["min", "max"]
    };
    object
        .keys()
        .map(facet_value::VString::as_str)
        .filter(|key| !ends.contains(key))
        .collect()
}

impl<T: RangeEnd> Range<T> {
    /// A range with both ends written, in `form`.
    pub fn new(min: T, max: T, form: RangeForm) -> Self {
        Self {
            min,
            max,
            form,
            swapped: false,
            missing_min: false,
            missing_max: false,
        }
    }

    /// The ends vanilla's reader uses for a member whose defaults are `lo` and `hi`
    /// (the game's integer range reader): an end the object left out takes the default, and
    /// the two are ordered.
    #[must_use]
    pub fn resolve(&self, lo: T, hi: T) -> (T, T) {
        let min = if self.missing_min { lo } else { self.min };
        let max = if self.missing_max { hi } else { self.max };
        if max < min { (max, min) } else { (min, max) }
    }

    /// The findings vanilla's reader reports for this range, located under `at`: a swapped
    /// range is an `InvalidValue` info.
    pub fn check(&self, at: &JsonPath) -> Diagnostics {
        let mut out = Diagnostics::default();
        if self.swapped {
            // The game swaps the ends silently; the information is the crate's own.
            out.push(finding(
                DiagnosticCode::InvalidValue { text: None },
                Severity::Info,
                at,
            ));
        }
        out
    }

    /// Reads the object form.
    fn read_object(object: &VObject) -> Result<Self, ShapeError> {
        let cereal = (object.get("min"), object.get("max"));
        let json_util = (object.get("range_min"), object.get("range_max"));
        let (form, (min, max), names) = if json_util.0.is_some() || json_util.1.is_some() {
            (
                RangeForm::RangeMinMax,
                json_util,
                ("range_min", "range_max"),
            )
        } else {
            (RangeForm::MinMax, cereal, ("min", "max"))
        };
        let end = |name, value: Option<&Value>| value.map(|v| T::read_end(name, v)).transpose();
        let (min, max) = (end(names.0, min)?, end(names.1, max)?);
        let mut range = Self::new(min.unwrap_or_default(), max.unwrap_or_default(), form);
        range.missing_min = min.is_none();
        range.missing_max = max.is_none();
        Ok(range)
    }

    /// Swaps the ends when both were written and `max < min`.
    fn ordered(mut self) -> Self {
        if !self.missing_min && !self.missing_max && self.max < self.min {
            std::mem::swap(&mut self.min, &mut self.max);
            self.swapped = true;
        }
        self
    }

    /// The ends in the document's order.
    fn written_ends(&self) -> (T, T) {
        if self.swapped {
            (self.max, self.min)
        } else {
            (self.min, self.max)
        }
    }
}

impl<T: RangeEnd> TryFrom<Value> for Range<T> {
    type Error = ShapeError;

    fn try_from(value: Value) -> Result<Self, ShapeError> {
        let range = if value.is_number() {
            let x = T::read_end(ROOT, &value)?;
            Self::new(x, x, RangeForm::Scalar)
        } else if let Some(array) = value.as_array() {
            let (min, max) = T::read_array(array)?;
            Self::new(min, max, RangeForm::Array)
        } else if let Some(object) = value.as_object() {
            // Other members are ignored, as vanilla's readers ignore them; a range has no
            // `extra`, so they are not written back.
            Self::read_object(object)?
        } else {
            return Err(wrong_type(ROOT, "a number, an array or an object", &value).into());
        };
        Ok(range.ordered())
    }
}

impl<T: RangeEnd> TryFrom<&Range<T>> for Value {
    type Error = ShapeError;

    fn try_from(range: &Range<T>) -> Result<Self, ShapeError> {
        let (min, max) = range.written_ends();
        let names = match range.form {
            RangeForm::Scalar if min == max && !range.missing_min && !range.missing_max => {
                return min.write_end();
            }
            RangeForm::Scalar | RangeForm::Array => {
                let mut array = VArray::new();
                array.push(min.write_end()?);
                array.push(max.write_end()?);
                return Ok(Value::from(array));
            }
            RangeForm::MinMax => ("min", "max"),
            RangeForm::RangeMinMax => ("range_min", "range_max"),
        };
        let mut object = VObject::new();
        if !range.missing_min {
            object.insert(names.0, min.write_end()?);
        }
        if !range.missing_max {
            object.insert(names.1, max.write_end()?);
        }
        Ok(Value::from(object))
    }
}
