//! How a scalar token is read against the declared type of the member that takes it: the
//! profiles a slice reads its scalars by ([`Scalars`]), what a number beyond the width of its
//! type does ([`BeyondWidth`]), the scalar shapes a hand-written proxy reads ([`ScalarShape`],
//! [`JsonType`], [`StrictError`]), and the pre-pass that reads a slice by them ([`judge`],
//! [`parse_with_rules`]).

use facet::Facet;

/// How a scalar token is read against the field's declared type: one profile per reader the
/// game has. A slice picks one in its `Reading` (`versioning::Reading::scalars`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Scalars {
    /// The strict readers, as the strict wrappers read (`StrictInt`, `StrictFloat`,
    /// `StrictBool`, `StrictString`): an integer member takes an integer token in its range
    /// (a real token is a type error, integral or not; one out of range too), a float member
    /// an integer or real token finite in its width, a bool member `true` or `false`, a
    /// string member a string. Every other token is a type error.
    Strict,
    /// The `JsonCpp` schema readers, the accept set of entity's generated game table: an
    /// `int` takes an integer or real token (a real read as its truncation toward zero), a
    /// `float` an integer, real or bool token (`true` read as `1.0`), a `string` a string, a
    /// `bool` a bool. No slice reads by it yet.
    JsonCpp,
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
    /// Whether a `null` read as a struct (a struct of defaults, which facet makes of it) is
    /// told in [`ScalarRead::told`] as a misread of another JSON type. `true` in every rule set
    /// the crate builds, today's reading. A family whose null rule reports that `null` itself
    /// (`NullRule::Rejected` through `reject_nulls`, or its own walk) turns it off, so the told
    /// list holds only what its rule does not tell, and can be pushed whole.
    ///
    /// The decision whether every slice should tell it, and so whether the option can go, is
    /// left to the family uptake: until then a family that left the told list unread for this
    /// one entry keeps its findings by turning it off.
    pub tells_null_struct: bool,
}

impl ScalarRules {
    /// How a plain member reads where no slice says otherwise: [`Scalars::ConvertAndTell`],
    /// a number beyond its float clamped and told, a refusal failing the read.
    pub const PLAIN: Self = Self {
        scalars: Scalars::ConvertAndTell,
        beyond_width: BeyondWidth::ClampAndTell,
        default_and_tell: false,
        tells_null_struct: true,
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

impl ScalarRules {
    /// The rules with a `null` read as a struct told, or left to the slice's null rule.
    #[must_use]
    pub const fn with_tells_null_struct(self, tells_null_struct: bool) -> Self {
        Self {
            tells_null_struct,
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

/// A JSON scalar token, as the door reads it: an integer token that fits 64 bits is `UInt` (not
/// negative) or `Int`, every other number (a real, an integer past 64 bits, which the door reads
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
        Scalars::JsonCpp => json_cpp(declared, token),
        Scalars::ConvertAndTell => convert_and_tell(declared, token),
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
/// width (no bits hold only `0`, 128 and more every `i128` of the sign).
fn fits(value: i128, bits: u8, signed: bool) -> bool {
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

/// The `JsonCpp` schema readers' rule.
#[allow(clippy::cast_possible_truncation)] // the truncation toward zero is the reading
fn json_cpp(declared: Declared, token: Token<'_>) -> Verdict {
    match (declared, token) {
        (Declared::Int { bits, signed }, Token::Real(x)) => {
            let truncated = x.trunc();
            // A real beyond every integer has no truncation that fits.
            if truncated.abs() < 1e38 && fits(truncated as i128, bits, signed) {
                Verdict::Respell(integer(truncated as i128))
            } else {
                Verdict::Refused(Refusal::OutOfRange)
            }
        }
        (Declared::Float { .. }, Token::Bool(b)) => {
            Verdict::Respell(Respelt::Real(f64::from(u8::from(b))))
        }
        _ => strict(declared, token),
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

/// A typed read under a slice's scalar rules ([`parse_with_rules`]).
#[derive(Debug)]
pub struct ScalarRead<T> {
    /// The value read.
    pub value: T,
    /// What is told of its scalars, in document order (paths from the root of the slice): each
    /// conversion across JSON types ([`Verdict::Converted`]), each number beyond its float
    /// ([`Verdict::BeyondFloat`]) and each refused member defaulted
    /// ([`ScalarRules::default_and_tell`]). The caller places them after the read, where the
    /// post-hoc misread walk told them.
    pub told: Vec<super::Misread>,
    /// The value of the text the pre-pass judged, for the caller's walk of the unknown members:
    /// under [`NullRule::Absent`](super::NullRule::Absent) without the members read as absent
    /// (as `remove_null_members` leaves it), else as written; `None` for a text that is no JSON.
    pub json: Option<super::Value>,
    /// The members read as absent under [`NullRule::Absent`](super::NullRule::Absent), in
    /// document order (paths from the root of the slice, for `tell_null_members`); empty under
    /// another rule.
    pub absent: Vec<crate::diagnostic::JsonPath>,
}

/// Why [`parse_with_rules`] read no value.
#[derive(Debug)]
pub enum ScalarFailure {
    /// The typed read failed: facet's own refusal, or the one it gives the token the pre-pass
    /// handed it for a refused scalar (a type error at that scalar, with its span, as a strict
    /// wrapper's refusal is).
    Read(super::SliceError),
    /// A scalar of an untagged enum none of whose scalar arms takes it, and no fault before it
    /// in document order: refused before the typed read, a type error at the enum
    /// ([`Misread::finding`](super::Misread::finding), which has no span), with the scalar's
    /// place in the file beside it ([`RefusedScalar::span`]).
    Refused(RefusedScalar),
}

/// The scalar of an untagged enum no arm takes ([`ScalarFailure::Refused`]): what the misread
/// tells, and where the scalar is.
///
/// It reads as its [`Misread`](super::Misread) (`refused.path`, `refused.finding(at)`), so a
/// caller written for the misread alone reads it unchanged;
/// [`spanned_finding`](Self::spanned_finding) is that finding with the span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefusedScalar {
    /// The type error at the enum, its path from the root of the slice.
    pub misread: super::Misread,
    /// The scalar's place in the file (the slice's offsets); `None` only where the path cannot
    /// be found in the text again.
    pub span: Option<crate::diagnostic::Span>,
}

impl RefusedScalar {
    /// [`Misread::finding`](super::Misread::finding) at `at`, spanning the scalar.
    #[must_use]
    pub fn spanned_finding(
        &self,
        at: &crate::diagnostic::JsonPath,
    ) -> crate::diagnostic::Diagnostic {
        let mut finding = self.misread.finding(at);
        finding.location.span = self.span;
        finding
    }
}

/// The misread, for the family readers that tell it as a [`Misread`](super::Misread); core
/// reads the field.
impl std::ops::Deref for RefusedScalar {
    type Target = super::Misread;

    fn deref(&self) -> &Self::Target {
        &self.misread
    }
}

/// Reads `slice` as `T` under `null` (the slice's null rule) and `rules`: the scalar pre-pass.
///
/// Before facet reads the text, every scalar of it is judged against its member's declared type
/// ([`judge`]), on the value of the text with the members `null` reads as absent left out:
/// - a scalar read as another token ([`Verdict::Respell`]) is handed facet as that token, at
///   its own offset and span;
/// - a refused scalar ([`Verdict::Refused`]) is handed facet as a token of a type its member
///   never takes, so that the typed read fails there as a strict wrapper's does; or, under
///   [`ScalarRules::default_and_tell`], its member is read as left out and told;
/// - a scalar of an untagged enum is read by the arm the rules choose; one no arm takes is
///   [`ScalarFailure::Refused`] and nothing is read, unless a scalar before it in document order
///   fails the typed read, which then fails there as it would without the enum;
/// - what is converted or clamped is told ([`ScalarRead::told`]).
///
/// Under [`Scalars::ConvertAndTell`] nothing is respelt or refused: facet reads the text as
/// written (rule N1 aside) and the pre-pass tells what the post-hoc misread walk told. A text
/// that is no JSON is the typed read's to report.
///
/// A flattened struct (`#[facet(flatten)]`) is not walked: its plain scalars are facet's under
/// every profile, read and converted as written and never told, so a `Strict` slice whose
/// struct flattens one does not judge them. Members behind a proxy (the strict wrappers) judge
/// themselves.
///
/// # Errors
///
/// [`ScalarFailure`], as above.
pub fn parse_with_rules<'a, T: facet::Facet<'a>>(
    slice: super::JsonSlice<'a>,
    null: Option<super::NullRule>,
    rules: ScalarRules,
) -> Result<ScalarRead<T>, ScalarFailure> {
    use super::path::{Member, MemberPath};
    use super::read::Plan;
    use super::{Misread, Misreading, Value};
    use facet_format::ScalarValue;

    let Ok(json) = super::from_str_jsonc::<Value>(slice.text) else {
        // Not JSON: the typed read reports it.
        let value = super::slice::parse_planned::<T>(slice, &Plan::default())
            .map_err(ScalarFailure::Read)?;
        return Ok(ScalarRead {
            value,
            told: Vec::new(),
            json: None,
            absent: Vec::new(),
        });
    };
    let mut json = json;
    let mut plan = Plan::default();
    if null == Some(super::NullRule::Absent) {
        plan.absent = super::null::member_paths(T::SHAPE, &json);
        super::null::remove_members(&mut json, &plan.absent);
    }
    let walked = super::unknown::walk_scalars(T::SHAPE, &json, rules);
    let mut told = Vec::new();
    let tell = |path: &super::path::MemberPath, kind| Misread {
        path: path.to_json_path(),
        kind,
    };
    // Whether a scalar before the one at hand fails the typed read there (a poisoned token, an
    // array where a scalar belongs), and the first scalar no arm of its enum takes.
    let mut faulted = false;
    let mut refused_arm = None;
    for call in &walked.scalars {
        if call.list.is_some() {
            // facet refuses the array itself; `wrapper_place` moves its finding.
            faulted = true;
            continue;
        }
        match call.verdict {
            Verdict::Keep => {}
            // A `null` read as a struct is the one conversion with no declared scalar type.
            Verdict::Converted if call.declared.is_none() && !rules.tells_null_struct => {}
            Verdict::Converted => told.push(tell(&call.path, Misreading::WrongType)),
            Verdict::BeyondFloat => told.push(tell(&call.path, Misreading::BeyondFloat)),
            Verdict::Respell(respelt) => plan.fixes.push((call.path.clone(), event(respelt))),
            Verdict::Refused(_) => {
                let member = matches!(call.path.steps().last(), Some(Member::Key(_)));
                if rules.default_and_tell && member {
                    plan.blank.push(call.path.clone());
                    told.push(tell(&call.path, Misreading::Defaulted));
                } else if call.arm {
                    let misread = tell(&call.path, Misreading::WrongType);
                    let span = super::span_at(slice, &misread.path);
                    let misread = RefusedScalar { misread, span };
                    if !faulted {
                        return Err(ScalarFailure::Refused(misread));
                    }
                    // A fault comes first: the typed read fails there, and nothing after it
                    // is read.
                    refused_arm = Some(misread);
                    break;
                } else {
                    faulted = true;
                    // The poison is a real token facet reads. Where an untagged enum's object
                    // arms share the member's key with fields of different scalar types, facet
                    // narrows the arms by it: an arm whose field takes the poison is read, and
                    // the refusal is lost (`{"k": "7"}` with arms `{k: u8}` and `{k: bool}` reads
                    // the bool arm as `false`; `json_scalars` pins it).
                    let poison = match call.declared {
                        // A number is no bool to a bool member.
                        Some(Declared::Bool) => ScalarValue::U64(2),
                        // A bool is no number and no string to any other.
                        _ => ScalarValue::Bool(false),
                    };
                    plan.fixes.push((call.path.clone(), poison));
                }
            }
        }
    }
    let read = super::slice::parse_planned::<T>(slice, &plan)
        .map_err(|error| ScalarFailure::Read(wrapper_place(error, slice, &walked.scalars)));
    if let Some(misread) = refused_arm {
        // Were the earlier fault read after all, the arm is still refused.
        return Err(read.err().unwrap_or(ScalarFailure::Refused(misread)));
    }
    let value = read?;
    let absent = plan.absent.iter().map(MemberPath::to_json_path).collect();
    Ok(ScalarRead {
        value,
        told,
        json: Some(json),
        absent,
    })
}

/// `error` at the place a strict wrapper's refusal has it, when facet refused an array where a
/// scalar belongs at that array's `[` (`calls` names each such array): a wrapper reads the array
/// through its untagged scalar proxy and fails at the last element, at the closing bracket (at
/// index 0 of an empty array).
fn wrapper_place(
    mut error: super::SliceError,
    slice: super::JsonSlice<'_>,
    calls: &[super::unknown::ScalarCall],
) -> super::SliceError {
    let Some(len) = calls
        .iter()
        .find_map(|call| call.list.filter(|_| call.path.to_json_path() == error.path))
    else {
        return error;
    };
    let Some(array) = super::span_at(slice, &error.path) else {
        return error;
    };
    if error.span.offset != array.offset || array.len == 0 {
        return error;
    }
    error.path.push_index(len.saturating_sub(1));
    error.span = crate::diagnostic::Span {
        offset: array.offset + array.len - 1,
        len: 1,
    };
    error
}

/// The parser event of a respelt token.
fn event(respelt: Respelt) -> facet_format::ScalarValue<'static> {
    use facet_format::ScalarValue;
    match respelt {
        Respelt::Bool(b) => ScalarValue::Bool(b),
        Respelt::UInt(u) => ScalarValue::U64(u),
        Respelt::Int(i) => ScalarValue::I64(i),
        Respelt::Real(x) => ScalarValue::F64(x),
    }
}

#[cfg(test)]
#[allow(dead_code, reason = "the test types are read, not looked at")]
mod tests {
    use super::*;
    use crate::json::unknown::{ScalarCall, walk_scalars};
    use crate::json::{Misreading, Value, from_str, keep_members};
    use facet::Facet;
    use indexmap::IndexMap;

    #[derive(Debug, Facet)]
    struct Plain {
        #[facet(default)]
        int: i32,
        #[facet(default)]
        small: u8,
        #[facet(default)]
        float: f32,
        #[facet(default)]
        flag: bool,
        #[facet(default)]
        name: String,
        #[facet(default)]
        maybe: Option<u8>,
        #[facet(default)]
        list: Vec<u16>,
        #[facet(default)]
        map: IndexMap<String, bool>,
        #[facet(default)]
        inner: Option<Inner>,
        #[facet(default)]
        any: Value,
    }

    #[derive(Debug, Default, Facet)]
    struct Inner {
        #[facet(default)]
        depth: f64,
    }

    #[derive(Debug, Facet)]
    #[facet(untagged)]
    #[repr(u8)]
    enum Shorthand {
        Enabled(bool),
        Params(Params),
    }

    #[derive(Debug, Facet)]
    struct Params {
        resistance: f32,
    }

    #[derive(Debug, Facet)]
    #[facet(untagged)]
    #[repr(u8)]
    enum CountOrName {
        Count(u8),
        Name(String),
    }

    #[derive(Debug, Facet)]
    struct Arms {
        #[facet(default)]
        shorthand: Option<Shorthand>,
        #[facet(default)]
        count_or_name: Option<CountOrName>,
    }

    fn calls<'f, T: Facet<'f>>(text: &str, rules: ScalarRules) -> Vec<(String, Verdict, bool)> {
        let json: Value = from_str(text).unwrap();
        walk_scalars(T::SHAPE, &json, rules)
            .scalars
            .into_iter()
            .map(
                |ScalarCall {
                     path, verdict, arm, ..
                 }| (path.to_json_path().to_string(), verdict, arm),
            )
            .collect()
    }

    const STRICT: ScalarRules = ScalarRules::PLAIN
        .with_scalars(Scalars::Strict)
        .with_beyond_width(BeyondWidth::Error);

    const WRONG: Verdict = Verdict::Refused(Refusal::WrongType);

    #[test]
    fn the_walk_judges_every_scalar_of_a_member_in_document_order() {
        let text = r#"{"name": 5, "int": 1.5, "small": 256, "float": 1e39, "flag": 1,
            "maybe": null, "list": [1, "2", null], "map": {"a": true, "b": "no"},
            "inner": {"depth": "deep"}, "any": {"int": "x"}}"#;
        assert_eq!(
            calls::<Plain>(text, STRICT),
            [
                ("name".to_owned(), WRONG, false),
                ("int".to_owned(), WRONG, false),
                (
                    "small".to_owned(),
                    Verdict::Refused(Refusal::OutOfRange),
                    false
                ),
                (
                    "float".to_owned(),
                    Verdict::Refused(Refusal::BeyondWidth),
                    false
                ),
                ("flag".to_owned(), WRONG, false),
                ("list/1".to_owned(), WRONG, false),
                ("list/2".to_owned(), WRONG, false),
                ("map/b".to_owned(), WRONG, false),
                ("inner/depth".to_owned(), WRONG, false),
            ]
        );
        // The same members under the plain reading: told where the misread walk tells, the
        // rest facet's.
        assert_eq!(
            calls::<Plain>(text, ScalarRules::PLAIN),
            [
                ("name".to_owned(), Verdict::Converted, false),
                ("float".to_owned(), Verdict::BeyondFloat, false),
                ("flag".to_owned(), Verdict::Converted, false),
                ("list/1".to_owned(), Verdict::Converted, false),
                ("list/2".to_owned(), Verdict::Converted, false),
                ("map/b".to_owned(), Verdict::Converted, false),
                ("inner/depth".to_owned(), Verdict::Converted, false),
            ]
        );
    }

    #[test]
    fn a_scalar_no_arm_takes_is_refused_at_the_enum_under_the_strict_rule() {
        let refused_at = |text: &str| calls::<Arms>(text, STRICT);
        assert_eq!(
            refused_at(r#"{"shorthand": 5}"#),
            [("shorthand".to_owned(), WRONG, true)]
        );
        assert_eq!(
            refused_at(r#"{"shorthand": "true"}"#),
            [("shorthand".to_owned(), WRONG, true)]
        );
        assert!(refused_at(r#"{"shorthand": true}"#).is_empty());
        // Inside the object arm, its members are judged as any member.
        assert_eq!(
            refused_at(r#"{"shorthand": {"resistance": "3"}}"#),
            [("shorthand/resistance".to_owned(), WRONG, false)]
        );
        // The first arm that takes the token: `7` is a count, `"x"` a name, `true` neither.
        assert!(refused_at(r#"{"count_or_name": 7}"#).is_empty());
        assert!(refused_at(r#"{"count_or_name": "x"}"#).is_empty());
        assert_eq!(
            refused_at(r#"{"count_or_name": true}"#),
            [("count_or_name".to_owned(), WRONG, true)]
        );
        assert_eq!(
            refused_at(r#"{"count_or_name": 300}"#),
            [("count_or_name".to_owned(), WRONG, true)]
        );
    }

    /// The plain reading tells what the misread walk of the value facet read tells, the arm
    /// of an untagged enum chosen as facet chose it: the two agree over every input facet reads.
    #[test]
    fn the_plain_reading_tells_what_the_typed_misread_walk_tells() {
        for text in [
            r#"{"name": 5, "int": 1e1, "float": "2", "flag": "true", "list": [1, "2"]}"#,
            r#"{"map": {"a": "true"}, "inner": {"depth": 7}, "maybe": "3"}"#,
            r#"{"float": 1e39, "inner": {"depth": "1"}, "name": "ok"}"#,
            r#"{"int": null, "name": null, "inner": null}"#,
        ] {
            told_as_the_typed_walk::<Plain>(text);
        }
        for text in [
            r#"{"count_or_name": 7}"#,
            r#"{"count_or_name": "7"}"#,
            r#"{"count_or_name": "x"}"#,
            r#"{"shorthand": "true"}"#,
            r#"{"shorthand": true}"#,
            r#"{"shorthand": {"resistance": "3"}}"#,
        ] {
            told_as_the_typed_walk::<Arms>(text);
        }
    }

    fn told_as_the_typed_walk<T: for<'f> Facet<'f> + std::fmt::Debug>(text: &str) {
        let Ok(read) = from_str::<T>(text) else {
            panic!("facet refuses {text}");
        };
        let json: Value = from_str(text).unwrap();
        let typed: Vec<(String, Verdict)> =
            keep_members(&read, &json, &mut crate::document::Unknown::new())
                .1
                .into_iter()
                .map(|misread| {
                    let verdict = match misread.kind {
                        Misreading::WrongType => Verdict::Converted,
                        Misreading::BeyondFloat => Verdict::BeyondFloat,
                        other => panic!("the misread walk tells no {other:?}"),
                    };
                    (misread.path.to_string(), verdict)
                })
                .collect();
        let prepass: Vec<(String, Verdict)> = calls::<T>(text, ScalarRules::PLAIN)
            .into_iter()
            .map(|(path, verdict, _)| (path, verdict))
            .collect();
        assert_eq!(prepass, typed, "{text}: {read:?}");
    }
}

/// The integer types a [`StrictInt`](super::StrictInt) holds: `i8` to `i64` and `u8` to `u64`.
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

    /// The non-strict readers' value of `shape` ([`ReaderMode::NonStrict`](super::ReaderMode::NonStrict)): the low bits of an
    /// integer token (`ldrb`, `ldr w`, `ldr x` of the token's 64 bits), a bool as `0` or `1`,
    /// a real as the truncation toward zero that `fcvtzs` / `fcvtzu` gives (saturating), a
    /// negative real as the negated truncation of its magnitude. `None` for the other types.
    fn from_non_strict(shape: &ScalarShape) -> Option<Self>;
}

/// The seal of [`StrictIntType`] and `StrictFloatType`: private to `json`, so no type outside
/// this crate has it.
pub(super) mod sealed {
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
