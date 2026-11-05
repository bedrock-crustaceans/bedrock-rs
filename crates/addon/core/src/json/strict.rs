//! The strict wrappers: scalars that take only the JSON type vanilla's reader takes.
//!
//! Hidden, and on its way out: a plain member (`u8`, `f32`, `bool`, `String`) is read by its
//! slice's [`Reading`](crate::versioning::Reading) (the scalar pre-pass,
//! [`parse_with_rules`](super::parse_with_rules)), which is how every family states strictness
//! now. What still holds a wrapper waits on a core gap:
//!
//! | user | lines | waits on |
//! |---|---|---|
//! | block's shell structs (`families/block/src/{types,states,validate}.rs`, `component/deprecated.rs`; `addon/tests/block/{rows,rules}.rs`) | 30 (+5 in tests) | the shell is read under `ScalarRules::PLAIN`, outside the slice's `Reading`: a per-struct strict scalar rule, or the shell read under the slice's `Reading` |
//! | trade's wire structs (`families/trade/src/read.rs`: `TableWire`, `GroupWire`, `TradeWire`, `EntryWire`, `quantity_int`) | 12 | each element is read from its own `RawJson` slice with `parse_at`: a per-element slice attribute (the element read under the slice's rules) |
//! | `shared::MenuCategory::is_hidden_in_commands` (`core/src/shared/menu.rs`; `core/tests/shared_menu_category.rs`) | 2 (+2) | read in block's description, under its shell's `PLAIN` rules, and in item's: the block shell gap, and a pin of item's reading of it |
//! | [`ReaderMode`] / [`with_reader_mode`] (`core/tests/json_scalars.rs`) | 34 (tests) | no reader sets it: kept as the oracle the pre-pass's `Scalars::NonStrict` is held equal to |
//! | the wrappers' own tests and the core tests that read through a proxy (`core/tests/{json_strict,json_scalars,json_slice,json_unknown,json_wide_integers,json_wide_integers_generated,json_write,api_traits}.rs`, the tests in `json/null.rs` and `json/unknown.rs`) | tests | they test the wrappers or the proxy rules (a proxied member judges itself; a proxied float is written by its width), and go with them |
//!
//! facet coerces scalars (`5` into a `String`, `"7"` into an integer, `"true"` into a `bool`,
//! `null` into the default) and has no option to stop. Each wrapper
//! is a `#[facet(proxy)]` over [`ScalarShape`], an untagged enum of every JSON scalar shape, and
//! its conversion accepts only the jsoncpp type. Inside an `Option`, `null` stays `None`.
//! The scalar vocabulary they are read through ([`ScalarShape`], [`JsonType`], [`StrictError`])
//! is `json::scalar`'s, which outlives the wrappers, as does [`StrictIntType`], the integer
//! widths (a bound in entity's `goal/check.rs`).

use facet::Facet;

use super::scalar::{ScalarShape, StrictError, StrictIntType, sealed};

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
