//! The strict wrappers: scalars that take only the JSON type vanilla's reader takes.
//!
//! facet coerces scalars (`5` into a `String`, `"7"` into an integer, `"true"` into a `bool`,
//! `null` into the default) and has no option to stop. Each wrapper
//! is a `#[facet(proxy)]` over [`ScalarShape`], an untagged enum of every JSON scalar shape, and
//! its conversion accepts only the jsoncpp type. Inside an `Option`, `null` stays `None`.

use facet::Facet;

/// Every JSON scalar shape a deserialiser can offer, so a wrapper can reject the wrong ones:
/// without the variants a type does not accept, facet's untagged solver would coerce `null` into
/// a number and a number into a string. Integers have their own variants because the solver
/// matches an integer token only against integer types; `String` comes first so `"1"` is never
/// read as a number.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum ScalarShape {
    /// `null`.
    Null,
    /// A string.
    String(String),
    /// `true` or `false`.
    Bool(bool),
    /// An integer token that fits `u64`.
    UInt(u64),
    /// A negative integer token that fits `i64`.
    Int(i64),
    /// A real token (`1.0`, `1e3`), or an integer outside 64 bits.
    Number(f64),
}

/// A JSON value type as jsoncpp's type tests see it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum JsonType {
    /// `null`.
    Null,
    /// A string.
    String,
    /// `true` or `false`.
    Bool,
    /// An integer token.
    Integer,
    /// A real token.
    Real,
}

impl ScalarShape {
    fn json_type(&self) -> JsonType {
        match self {
            Self::Null => JsonType::Null,
            Self::String(_) => JsonType::String,
            Self::Bool(_) => JsonType::Bool,
            Self::UInt(_) | Self::Int(_) => JsonType::Integer,
            Self::Number(_) => JsonType::Real,
        }
    }
}

/// Why a strict wrapper rejected a JSON value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum StrictError {
    /// The value has a JSON type the field does not take.
    #[error("expected a JSON {expected}, found a JSON {found:?}")]
    WrongType {
        /// What the field takes, in words.
        expected: &'static str,
        /// What the document has.
        found: JsonType,
    },
    /// A number of the right type that does not fit the field (`256` for a `u8`, `1e39` for an
    /// `f32`).
    #[error("the number does not fit the field's type")]
    OutOfRange,
    /// A float that is not finite has no JSON form to write.
    #[error("a non-finite number has no JSON form")]
    NonFinite,
}

/// Which of the game's scalar readers a typed read stands for.
///
/// Every cereal document is read through one of two reader families: the strict one
/// takes the JSON types of a
/// member as they are, and the non-strict one reads a bool and a number as each other: `asBool` takes a number as
/// `value != 0`, an integer reader takes a bool as `0` or `1`, an integer token by its low
/// bits with no range check and a real by its truncation toward zero, `asFloat` takes a bool as
/// `0.0` or `1.0`. A string, `null`, an array and an object are a wrong type in both. Which
/// reader reads a document is the loader's: the strict one unless the slice says otherwise (a flag the
/// loader carries beside the slice's error mask). [`ReaderMode::Strict`]
/// is what every wrapper does by default; [`with_reader_mode`] makes the reads inside it
/// non-strict.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ReaderMode {
    /// The strict readers.
    #[default]
    Strict,
    /// The non-strict readers.
    NonStrict,
}

thread_local! {
    static READER_MODE: std::cell::Cell<ReaderMode> = const { std::cell::Cell::new(ReaderMode::Strict) };
}

/// Runs `read` with the strict wrappers of this thread reading as `mode` does, and puts the
/// previous mode back, a panic included.
pub fn with_reader_mode<R>(mode: ReaderMode, read: impl FnOnce() -> R) -> R {
    struct Restore(ReaderMode);
    impl Drop for Restore {
        fn drop(&mut self) {
            READER_MODE.with(|m| m.set(self.0));
        }
    }
    let _restore = Restore(READER_MODE.with(|m| m.replace(mode)));
    read()
}

fn non_strict() -> bool {
    READER_MODE.with(std::cell::Cell::get) == ReaderMode::NonStrict
}

/// A bool the non-strict readers take as a number: `true` is `1`.
fn bool_as_number(json: &ScalarShape) -> Option<f64> {
    match json {
        ScalarShape::Bool(b) => Some(f64::from(u8::from(*b))),
        _ => None,
    }
}

/// An integer field that takes only integer tokens: `1.0`, `"1"`, `true` and `null` are rejected
/// where facet would coerce them. JSON: an integer.
///
/// Many component members are of this type. The value is the public field: `StrictInt(15)`
/// builds one and `.0` reads it.
///
/// ```
/// use bedrock_addon_core::json::StrictInt;
///
/// let light: StrictInt<u8> = StrictInt(15);
/// assert_eq!(light.0, 15);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
#[facet(proxy = ScalarShape)]
pub struct StrictInt<T: StrictIntType>(pub T);

/// A float field that takes only numbers (integer or real tokens): `"1.5"`, `true` and `null` are
/// rejected, and so is a number that is not finite in `T` (`1e39` for an `f32`). JSON: a number.
///
/// The value is the public field: `StrictFloat(1.5_f32)` builds one and `.0` reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Facet)]
#[facet(proxy = ScalarShape)]
pub struct StrictFloat<T: StrictFloatType>(pub T);

/// A boolean field that takes only `true` and `false`: `1`, `"true"` and `null` are rejected.
/// JSON: a boolean.
///
/// The value is the public field: `StrictBool(true)` builds one and `.0` reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
#[facet(proxy = ScalarShape)]
pub struct StrictBool(pub bool);

/// A string field that takes only strings: `5`, `true` and `null` are rejected. JSON: a string.
///
/// The value is the public field: `StrictString("stone".to_owned())` builds one and `.0`
/// reads it.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
#[facet(proxy = ScalarShape)]
pub struct StrictString(pub String);

/// The integer types a [`StrictInt`] holds: `i8` to `i64` and `u8` to `u64`.
///
/// The set is closed: the trait cannot be implemented outside this crate.
///
/// ```compile_fail,E0277
/// use bedrock_addon_core::json::{ScalarShape, StrictIntType};
///
/// #[derive(Clone, Copy)]
/// struct Mine(u8);
/// impl TryFrom<u64> for Mine {
///     type Error = ();
///     fn try_from(_: u64) -> Result<Self, ()> { Err(()) }
/// }
/// impl TryFrom<i64> for Mine {
///     type Error = ();
///     fn try_from(_: i64) -> Result<Self, ()> { Err(()) }
/// }
/// impl StrictIntType for Mine {
///     fn to_shape(self) -> ScalarShape { ScalarShape::Null }
///     fn from_non_strict(_: &ScalarShape) -> Option<Self> { None }
/// }
/// ```
pub trait StrictIntType: sealed::Sealed + Copy + TryFrom<u64> + TryFrom<i64> {
    /// The written form: `UInt` for a non-negative value, `Int` for a negative one.
    fn to_shape(self) -> ScalarShape;

    /// The non-strict readers' value of `shape` ([`ReaderMode::NonStrict`]): the low bits of an
    /// integer token (`ldrb`, `ldr w`, `ldr x` of the token's 64 bits), a bool as `0` or `1`,
    /// a real as the truncation toward zero that `fcvtzs` / `fcvtzu` gives (saturating), a
    /// negative real as the negated truncation of its magnitude. `None` for the other types.
    fn from_non_strict(shape: &ScalarShape) -> Option<Self>;
}

/// The float types a [`StrictFloat`] holds: `f32` and `f64`.
///
/// The set is closed: the trait cannot be implemented outside this crate.
///
/// ```compile_fail,E0277
/// use bedrock_addon_core::json::StrictFloatType;
///
/// #[derive(Clone, Copy)]
/// struct Mine(f64);
/// impl From<Mine> for f64 {
///     fn from(mine: Mine) -> f64 { mine.0 }
/// }
/// impl StrictFloatType for Mine {
///     fn from_f64(x: f64) -> Option<Self> { Some(Mine(x)) }
/// }
/// ```
pub trait StrictFloatType: sealed::Sealed + Copy + Into<f64> {
    /// The `f64` read from the document, if it is finite in `Self`.
    fn from_f64(x: f64) -> Option<Self>;
}

/// The seal of [`StrictIntType`] and [`StrictFloatType`]: private, so no type outside this
/// module has it.
mod sealed {
    pub trait Sealed {}
}

macro_rules! strict_int_types {
    ($($t:ty => $real:ident),*) => {$(
        impl sealed::Sealed for $t {}
        impl StrictIntType for $t {
            fn to_shape(self) -> ScalarShape {
                // Every value of these types fits `u64` or, when negative, `i64`; the `Null`
                // arm is never taken.
                match (u64::try_from(self), i64::try_from(self)) {
                    (Ok(u), _) => ScalarShape::UInt(u),
                    (Err(_), Ok(i)) => ScalarShape::Int(i),
                    (Err(_), Err(_)) => ScalarShape::Null,
                }
            }

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_possible_wrap)] // the low bits are the reading
            fn from_non_strict(shape: &ScalarShape) -> Option<Self> {
                let bits = match shape {
                    ScalarShape::Bool(b) => u64::from(*b),
                    ScalarShape::UInt(u) => *u,
                    ScalarShape::Int(i) => *i as u64,
                    ScalarShape::Number(x) => $real(*x),
                    ScalarShape::Null | ScalarShape::String(_) => return None,
                };
                Some(bits as $t)
            }
        }
    )*};
}
strict_int_types!(
    i8 => real_as_i32, i16 => real_as_i32, i32 => real_as_i32, i64 => real_as_i64,
    u8 => real_as_i32, u16 => real_as_i32, u32 => real_as_u32, u64 => real_as_u64
);

// The non-strict readers' reals: `x >= 0` is the saturating truncation toward zero, `x < 0`
// the negated truncation of `-x`. The 8 and 16 bit readers go through the signed 32 bit
// conversion.

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn real_as_i32(x: f64) -> u64 {
    let v = if x >= 0.0 {
        x as i32
    } else {
        (-x as i32).wrapping_neg()
    };
    u64::from(v as u32)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn real_as_u32(x: f64) -> u64 {
    let v = if x >= 0.0 {
        x as u32
    } else {
        (-x as u32).wrapping_neg()
    };
    u64::from(v)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn real_as_i64(x: f64) -> u64 {
    let v = if x >= 0.0 {
        x as i64
    } else {
        (-x as i64).wrapping_neg()
    };
    v as u64
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn real_as_u64(x: f64) -> u64 {
    if x >= 0.0 {
        x as u64
    } else {
        (-x as u64).wrapping_neg()
    }
}

impl sealed::Sealed for f32 {}
impl sealed::Sealed for f64 {}

impl StrictFloatType for f32 {
    #[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
    fn from_f64(x: f64) -> Option<Self> {
        Some(x as f32).filter(|x| x.is_finite())
    }
}

impl StrictFloatType for f64 {
    fn from_f64(x: f64) -> Option<Self> {
        Some(x).filter(|x| x.is_finite())
    }
}

impl<T: StrictIntType> TryFrom<ScalarShape> for StrictInt<T> {
    type Error = StrictError;

    fn try_from(json: ScalarShape) -> Result<Self, StrictError> {
        // jsoncpp `isInt` / `isUInt` on an int-typed value; a real token is a real even when it
        // is integral.
        if non_strict() {
            return T::from_non_strict(&json)
                .map(StrictInt)
                .ok_or_else(|| wrong_type("integer", &json));
        }
        match json {
            ScalarShape::UInt(u) => T::try_from(u).map_err(|_| StrictError::OutOfRange),
            ScalarShape::Int(i) => T::try_from(i).map_err(|_| StrictError::OutOfRange),
            other => Err(wrong_type("integer", &other)),
        }
        .map(StrictInt)
    }
}

/// The rejection of a value whose JSON type the field does not take.
fn wrong_type(expected: &'static str, found: &ScalarShape) -> StrictError {
    StrictError::WrongType {
        expected,
        found: found.json_type(),
    }
}

impl<T: StrictIntType> From<T> for StrictInt<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: StrictFloatType> From<T> for StrictFloat<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl From<bool> for StrictBool {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl From<String> for StrictString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl<T: StrictIntType> From<&StrictInt<T>> for ScalarShape {
    fn from(value: &StrictInt<T>) -> Self {
        value.0.to_shape()
    }
}

impl<T: StrictFloatType> TryFrom<ScalarShape> for StrictFloat<T> {
    type Error = StrictError;

    fn try_from(json: ScalarShape) -> Result<Self, StrictError> {
        // jsoncpp `isDouble` is true for int and real values alike.
        #[allow(clippy::cast_precision_loss)] // the nearest float is the reading
        let x = match json {
            ScalarShape::UInt(u) => u as f64,
            ScalarShape::Int(i) => i as f64,
            ScalarShape::Number(x) => x,
            other => match bool_as_number(&other).filter(|_| non_strict()) {
                Some(x) => x,
                None => return Err(wrong_type("number", &other)),
            },
        };
        T::from_f64(x)
            .map(StrictFloat)
            .ok_or(StrictError::OutOfRange)
    }
}

impl<T: StrictFloatType> TryFrom<&StrictFloat<T>> for ScalarShape {
    type Error = StrictError;

    fn try_from(value: &StrictFloat<T>) -> Result<Self, StrictError> {
        let x: f64 = value.0.into();
        if x.is_finite() {
            Ok(ScalarShape::Number(x))
        } else {
            Err(StrictError::NonFinite)
        }
    }
}

impl TryFrom<ScalarShape> for StrictBool {
    type Error = StrictError;

    fn try_from(json: ScalarShape) -> Result<Self, StrictError> {
        match json {
            ScalarShape::Bool(b) => Ok(StrictBool(b)),
            // `asBool` of the non-strict readers: a number is `value != 0`.
            ScalarShape::UInt(u) if non_strict() => Ok(StrictBool(u != 0)),
            ScalarShape::Int(i) if non_strict() => Ok(StrictBool(i != 0)),
            ScalarShape::Number(x) if non_strict() => Ok(StrictBool(x != 0.0)),
            other => Err(wrong_type("boolean", &other)),
        }
    }
}

impl From<&StrictBool> for ScalarShape {
    fn from(value: &StrictBool) -> Self {
        ScalarShape::Bool(value.0)
    }
}

impl TryFrom<ScalarShape> for StrictString {
    type Error = StrictError;

    fn try_from(json: ScalarShape) -> Result<Self, StrictError> {
        match json {
            ScalarShape::String(s) => Ok(StrictString(s)),
            other => Err(wrong_type("string", &other)),
        }
    }
}

impl From<&StrictString> for ScalarShape {
    fn from(value: &StrictString) -> Self {
        ScalarShape::String(value.0.clone())
    }
}
