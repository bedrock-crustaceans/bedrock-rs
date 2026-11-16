//! How a scalar token is read against the declared type of the member that takes it: the
//! profiles a slice reads its scalars by ([`Scalars`]), what a number beyond the width of its
//! type does ([`BeyondWidth`]), the scalar shapes a hand-written proxy reads ([`ScalarShape`],
//! [`JsonType`], [`StrictError`]), and the judgement of a token by them ([`judge`]).

use facet::Facet;

use crate::diagnostic::Severity;

/// How a scalar token is read against the field's declared type: one profile per reader the
/// game has. A slice picks one in its `Reading` (`versioning::Reading::scalars`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Scalars {
    /// The strict readers, as the strict wrappers read (`StrictBool` is the one left): an
    /// integer member takes an integer token in its range
    /// (a real token is a type error, integral or not; one out of range too), a float member
    /// an integer or real token finite in its width, a bool member `true` or `false`, a
    /// string member a string. Every other token is a type error.
    Strict,
    /// The non-strict readers of the legacy block loader: a bool and a number read as each
    /// other (`asBool` takes a number as `value != 0`, an integer reader a bool as `0` or
    /// `1`, `asFloat` a bool as `0.0` or `1.0`), an integer token by its low bits with no
    /// range check, a real by its truncation toward zero (saturating). A string, `null`, an
    /// array and an object are a type error, as in [`Strict`](Self::Strict).
    NonStrict,
    /// The typed reader of a plain member as it reads without a wrapper: a token the member's
    /// type converts from is read converted (a numeric string into a number or a bool, a number
    /// into a string, an integral real into an integer), and a conversion across JSON types is
    /// told as a type error at the member, after the read; what the type does not convert from
    /// (a fraction into an integer, a bool into a number, an integer out of range) is the typed
    /// read's own refusal. Every slice but the ones that say otherwise.
    ConvertAndTell,
    /// The jsoncpp variant the sound files are read with (`json::jsoncpp::fork`): every member
    /// is coerced as the game's reader coerces it (`asFloat`, `asBool`, `asInt`, `asString`)
    /// and no token is refused. A `null` reads as the type's zero, a value of another JSON
    /// type than the member's as the reader makes of it (`"1"` for a float is `0.0`, a number
    /// for a bool is `!= 0`, a string for a bool is `!is_empty`, a bool for a text is its
    /// name), and the crate tells it as an information at the member
    /// ([`Read::scalar`](crate::json::kit::Read::scalar)). `judge` never refuses under it.
    Fork,
    /// The jsoncpp variant the texture files are read with (`json::jsoncpp::lenient`): every
    /// member is coerced as the game's reader coerces it, no token is refused, and the crate
    /// tells it as an information at the member (`INVALID_VALUE`) where the value is not of
    /// the type read without a word. It differs from [`Fork`](Self::Fork) in what is quiet and
    /// in the integer rule: an integer keeps its low 32 bits (the sound files' reader reads
    /// one past 64 bits as `0`), an integer member is quiet only for an integer token that fits
    /// 32 bits, a float member for a number that fits a float (a bool is told, a number beyond
    /// a float is read as the largest finite one and told), a bool member for a bool or a
    /// number, a string member for a string. A `null` that reaches the read is told (the owner
    /// that reads a `null` member as absent never asks).
    Lenient,
    /// The jsoncpp accessors as the spawn files' hand parsers read every member (`asInt`,
    /// `asUInt`, `asInt64`, `asFloat`, `asBool`, `asString`; the trade table's legacy reader
    /// too): every member is coerced, no token is refused, and the crate tells at the severity
    /// it carries what the game reads in silence: a value of another JSON type than the
    /// member's `INVALID_TYPE` (a real for an integer, truncated and saturating, an `asInt64`
    /// one through 32 bits; a bool 0 or 1; a number for a bool `!= 0`; a bool for a text its
    /// name; anything else the type's zero, a `null` that reaches the read included), an
    /// integer the member's width does not hold `INVALID_VALUE` (it keeps its low bits), a
    /// number beyond a float `INVALID_VALUE` too (the largest finite one).
    Accessors(Severity),
}

/// What a number beyond the width of its float member does (`1e39` for an `f32`): the reading
/// of the slice, since the strict readers refuse it and the plain typed reader narrows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BeyondWidth {
    /// A type error at the number, as the strict wrappers refuse it.
    Error,
    /// The member reads the largest finite float of its width, and an information at the
    /// number tells it (`BEYOND_FLOAT_MEMBER`): the versioned load clamps the number in the
    /// text and reads again.
    ClampAndTell,
    /// The member reads the largest finite float of its width, and an information at the
    /// number tells it (`INVALID_VALUE`), in the one read: the game's schema node tree's float
    /// node. It is `schema::finite` for every float the read meets, a list's elements and a
    /// map's values too (one rule in `Read::scalar`), as [`Read::clamp_molang`] and
    /// [`Read::clamp_kept`] say it for a Molang number and for the numbers of a value kept
    /// whole; the findings are the same, told at once there and after the read here.
    ///
    /// [`Read::clamp_molang`]: crate::json::kit::Read::clamp_molang
    /// [`Read::clamp_kept`]: crate::json::kit::Read::clamp_kept
    Finite,
}

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
    /// The JSON type of the shape.
    pub(crate) fn json_type(&self) -> JsonType {
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

/// What [`judge`] reads a scalar token by: the slice's profile and its rule for a number
/// beyond its float, and what a refused member does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ScalarRules {
    /// The profile.
    pub scalars: Scalars,
    /// What a number beyond the width of its float member does.
    pub beyond_width: BeyondWidth,
    /// Whether a member whose scalar is refused takes its default and the reader tells it (an
    /// information at the scalar), instead of failing the read there.
    pub default_and_tell: bool,
}

impl ScalarRules {
    /// How a plain member reads where no slice says otherwise: [`Scalars::ConvertAndTell`],
    /// a number beyond its float clamped and told, a refusal failing the read.
    pub const PLAIN: Self = Self {
        scalars: Scalars::ConvertAndTell,
        beyond_width: BeyondWidth::ClampAndTell,
        default_and_tell: false,
    };

    /// The rules with the profile `scalars`.
    #[must_use]
    pub const fn with_scalars(self, scalars: Scalars) -> Self {
        Self { scalars, ..self }
    }

    /// The rules with a number beyond its float doing `beyond_width`.
    #[must_use]
    pub const fn with_beyond_width(self, beyond_width: BeyondWidth) -> Self {
        Self {
            beyond_width,
            ..self
        }
    }

    /// The rules with a refused member defaulted and told, or failing the read.
    #[must_use]
    pub const fn with_default_and_tell(self, default_and_tell: bool) -> Self {
        Self {
            default_and_tell,
            ..self
        }
    }
}

impl Default for ScalarRules {
    fn default() -> Self {
        Self::PLAIN
    }
}

/// The declared type of a scalar member, as [`judge`] tells types apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Declared {
    /// `bool`.
    Bool,
    /// An integer of `bits` bits, signed or not.
    Int {
        /// The width.
        bits: u8,
        /// Whether it is signed.
        signed: bool,
    },
    /// A float of `bits` bits (32 or 64).
    Float {
        /// The width.
        bits: u8,
    },
    /// A string (or a `char`, which no member of a pack document has).
    Text,
}

impl Declared {
    /// The declared type of a member of the scalar type `ty`; `None` for `()` and the types no
    /// pack document member has.
    #[must_use]
    pub fn of(ty: facet::ScalarType) -> Option<Self> {
        use facet::ScalarType as T;
        let int = |bits: u8, signed: bool| Some(Self::Int { bits, signed });
        match ty {
            T::Bool => Some(Self::Bool),
            T::I8 => int(8, true),
            T::I16 => int(16, true),
            T::I32 => int(32, true),
            T::I64 | T::ISize => int(64, true),
            T::I128 => int(128, true),
            T::U8 => int(8, false),
            T::U16 => int(16, false),
            T::U32 => int(32, false),
            T::U64 | T::USize => int(64, false),
            T::U128 => int(128, false),
            T::F32 => Some(Self::Float { bits: 32 }),
            T::F64 => Some(Self::Float { bits: 64 }),
            T::String | T::Str | T::CowStr | T::Char => Some(Self::Text),
            _ => None,
        }
    }
}

/// A JSON scalar token, as the parser reads it: an integer token that fits 64 bits is `UInt` (not
/// negative) or `Int`, every other number (a real, an integer past 64 bits, which the parser reads
/// as the double it spells) is `Real`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Token<'a> {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A non-negative integer token.
    UInt(u64),
    /// A negative integer token.
    Int(i64),
    /// A real token.
    Real(f64),
    /// A string.
    Str(&'a str),
}

impl<'a> Token<'a> {
    /// The token `value` is; `None` for an array or object.
    #[must_use]
    pub fn of(value: &'a super::Value) -> Option<Self> {
        if value.is_null() {
            return Some(Self::Null);
        }
        if let Some(b) = value.as_bool() {
            return Some(Self::Bool(b));
        }
        if let Some(text) = value.as_string() {
            return Some(Self::Str(text.as_str()));
        }
        let number = value.as_number()?;
        Some(if number.is_float() {
            Self::Real(number.to_f64_lossy())
        } else if let Some(u) = number.to_u64() {
            Self::UInt(u)
        } else if let Some(i) = number.to_i64() {
            Self::Int(i)
        } else {
            Self::Real(number.to_f64_lossy())
        })
    }
}

/// What a member reads a token as.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Verdict {
    /// The token as written.
    Keep,
    /// Another token in its place: what the profile's reader makes of it (`257` read by the
    /// non-strict `u8` reader is `1`).
    Respell(Respelt),
    /// The token as the typed read converts it, a conversion across JSON types: told after the
    /// read as a type error at the member ([`Scalars::ConvertAndTell`]).
    Converted,
    /// A number beyond the member's float, read as the largest finite float and told
    /// ([`BeyondWidth::ClampAndTell`]).
    BeyondFloat,
    /// The member refuses the token: a type error at it.
    Refused(Refusal),
}

/// The token a member reads in place of the written one ([`Verdict::Respell`]).
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Respelt {
    /// A bool.
    Bool(bool),
    /// A non-negative integer.
    UInt(u64),
    /// A negative integer.
    Int(i64),
    /// A real.
    Real(f64),
}

/// Why a member refuses a token ([`Verdict::Refused`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Refusal {
    /// A token of a JSON type the member does not take under the profile.
    WrongType,
    /// An integer outside the member's type.
    OutOfRange,
    /// A number beyond the member's float ([`BeyondWidth::Error`]).
    BeyondWidth,
}

/// What a member of the type `declared` reads `token` as under `rules`.
///
/// `null` is judged as a token here; where the slice's null rule reads a member written as
/// `null` as absent, the caller does not ask.
#[must_use]
pub fn judge(declared: Declared, token: Token<'_>, rules: ScalarRules) -> Verdict {
    let verdict = match rules.scalars {
        Scalars::Strict => strict(declared, token),
        Scalars::NonStrict => non_strict(declared, token),
        Scalars::ConvertAndTell => convert_and_tell(declared, token),
        // Read by `Read::scalar` over the value, whose coercions are not tokens.
        Scalars::Fork | Scalars::Lenient | Scalars::Accessors(_) => Verdict::Keep,
    };
    match verdict {
        Verdict::Refused(Refusal::BeyondWidth)
            if rules.beyond_width == BeyondWidth::ClampAndTell =>
        {
            Verdict::BeyondFloat
        }
        verdict => verdict,
    }
}

/// A number of the token, as a float member reads it: `None` for a token of another type.
#[allow(clippy::cast_precision_loss)] // the nearest double is the reading
fn number(token: Token<'_>) -> Option<f64> {
    match token {
        Token::UInt(u) => Some(u as f64),
        Token::Int(i) => Some(i as f64),
        Token::Real(x) => Some(x),
        _ => None,
    }
}

/// A number into a float of `bits`: kept when finite in that width.
#[allow(clippy::cast_possible_truncation)] // the nearest f32 is the question
fn float(bits: u8, x: f64) -> Verdict {
    let finite = if bits == 32 {
        (x as f32).is_finite()
    } else {
        x.is_finite()
    };
    if finite {
        Verdict::Keep
    } else {
        Verdict::Refused(Refusal::BeyondWidth)
    }
}

/// Whether the integer `value` fits an integer of `bits`, signed or not: total over every
/// width (no bits hold only `0`, 128 and more every `i128` of the sign). The profiles' and the
/// reader kit's one rule.
pub(crate) fn fits(value: i128, bits: u8, signed: bool) -> bool {
    match (bits, signed) {
        (0, _) => value == 0,
        (128.., true) => true,
        (128.., false) => value >= 0,
        // The bits above the sign bit all equal it.
        (bits, true) => matches!(value >> (bits - 1), 0 | -1),
        (bits, false) => value >= 0 && value >> bits == 0,
    }
}

/// The respelling of the integer `value`: `UInt` when not negative, `Int` otherwise.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // in range by construction
fn integer(value: i128) -> Respelt {
    if value < 0 {
        Respelt::Int(value as i64)
    } else {
        Respelt::UInt(value as u64)
    }
}

/// `Keep` when `read` is the token as written, the respelling otherwise.
fn read_as(token: Token<'_>, read: Respelt) -> Verdict {
    let same = match (token, read) {
        (Token::UInt(a), Respelt::UInt(b)) => a == b,
        (Token::Int(a), Respelt::Int(b)) => a == b,
        (Token::Bool(a), Respelt::Bool(b)) => a == b,
        _ => false,
    };
    if same {
        Verdict::Keep
    } else {
        Verdict::Respell(read)
    }
}

/// The strict wrappers' rule.
fn strict(declared: Declared, token: Token<'_>) -> Verdict {
    let wrong = Verdict::Refused(Refusal::WrongType);
    match declared {
        Declared::Int { bits, signed } => match token {
            Token::UInt(u) if fits(i128::from(u), bits, signed) => Verdict::Keep,
            Token::Int(i) if fits(i128::from(i), bits, signed) => Verdict::Keep,
            Token::UInt(_) | Token::Int(_) => Verdict::Refused(Refusal::OutOfRange),
            _ => wrong,
        },
        Declared::Float { bits } => number(token).map_or(wrong, |x| float(bits, x)),
        Declared::Bool => match token {
            Token::Bool(_) => Verdict::Keep,
            _ => wrong,
        },
        Declared::Text => match token {
            Token::Str(_) => Verdict::Keep,
            _ => wrong,
        },
    }
}

/// The non-strict readers' value of an integer of `bits`, signed or not, from `token`: its low
/// bits (a bool is `0` or `1`, a real its truncation toward zero, saturating; the 8 and 16 bit
/// readers truncate through the signed 32 bit conversion).
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)] // the low bits are the reading
fn low_bits(token: Token<'_>, bits: u8, signed: bool) -> Option<i128> {
    let raw: u64 = match token {
        Token::Bool(b) => u64::from(b),
        Token::UInt(u) => u,
        Token::Int(i) => i as u64,
        Token::Real(x) => match (bits, signed) {
            (..=32, true) | (..=16, false) => {
                let v = if x >= 0.0 {
                    x as i32
                } else {
                    (-x as i32).wrapping_neg()
                };
                u64::from(v as u32)
            }
            (..=32, false) => u64::from(if x >= 0.0 {
                x as u32
            } else {
                (-x as u32).wrapping_neg()
            }),
            (_, true) => {
                (if x >= 0.0 {
                    x as i64
                } else {
                    (-x as i64).wrapping_neg()
                }) as u64
            }
            (_, false) => {
                if x >= 0.0 {
                    x as u64
                } else {
                    (-x as u64).wrapping_neg()
                }
            }
        },
        Token::Null | Token::Str(_) => return None,
    };
    Some(match (bits, signed) {
        (8, true) => i128::from(raw as i8),
        (8, false) => i128::from(raw as u8),
        (16, true) => i128::from(raw as i16),
        (16, false) => i128::from(raw as u16),
        (32, true) => i128::from(raw as i32),
        (32, false) => i128::from(raw as u32),
        (_, true) => i128::from(raw as i64),
        (_, false) => i128::from(raw),
    })
}

/// The non-strict readers' rule.
fn non_strict(declared: Declared, token: Token<'_>) -> Verdict {
    let wrong = Verdict::Refused(Refusal::WrongType);
    match declared {
        Declared::Int { bits, signed } => {
            low_bits(token, bits, signed).map_or(wrong, |value| read_as(token, integer(value)))
        }
        Declared::Float { bits } => match token {
            Token::Bool(b) => Verdict::Respell(Respelt::Real(f64::from(u8::from(b)))),
            _ => number(token).map_or(wrong, |x| float(bits, x)),
        },
        Declared::Bool => match token {
            Token::Bool(_) => Verdict::Keep,
            Token::UInt(u) => Verdict::Respell(Respelt::Bool(u != 0)),
            Token::Int(i) => Verdict::Respell(Respelt::Bool(i != 0)),
            Token::Real(x) => Verdict::Respell(Respelt::Bool(x != 0.0)),
            _ => wrong,
        },
        Declared::Text => strict(declared, token),
    }
}

/// The plain typed read's rule: a token of another JSON type than the member's is converted
/// and told, as the post-hoc misread walk tells it (`json::Misread`); an integer member takes a
/// real (facet reads an integral one and refuses a fraction itself) and a number out of range is
/// facet's to refuse.
fn convert_and_tell(declared: Declared, token: Token<'_>) -> Verdict {
    match (declared, token) {
        (Declared::Text, Token::Str(_))
        | (Declared::Bool, Token::Bool(_))
        | (Declared::Int { .. }, Token::UInt(_) | Token::Int(_) | Token::Real(_)) => Verdict::Keep,
        (Declared::Float { bits }, _) => {
            number(token).map_or(Verdict::Converted, |x| float(bits, x))
        }
        _ => Verdict::Converted,
    }
}

/// The integer types of `i8` to `i64` and `u8` to `u64`: a bound for a reader that takes any
/// of them.
///
/// The set is closed: the trait cannot be implemented outside this crate.
///
/// ```compile_fail,E0277
/// use bedrock_addon_core::json::StrictIntType;
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
/// impl StrictIntType for Mine {}
/// ```
pub trait StrictIntType: sealed::Sealed + Copy + TryFrom<u64> + TryFrom<i64> {}

/// The seal of [`StrictIntType`]: private to `json`, so no type outside this crate has it.
pub(super) mod sealed {
    pub trait Sealed {}
}

macro_rules! strict_int_types {
    ($($t:ty),*) => {$(
        impl sealed::Sealed for $t {}
        impl StrictIntType for $t {}
    )*};
}
strict_int_types!(i8, i16, i32, i64, u8, u16, u32, u64);
