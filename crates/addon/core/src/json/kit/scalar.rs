//! [`Read::scalar`]: a scalar read by the member of a declared type under every
//! [`Scalars`] profile and [`BeyondWidth`] rule, telling what the typed door tells for the same
//! token.

use std::borrow::Cow;

use super::{Expected, Node, Pass, Read, WrongAt};
use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{Code, Diagnostic, DiagnosticDetail, JsonPath};
use crate::json::scalar::{BeyondWidth, Declared, Respelt, Scalars, Token, Verdict, fits, judge};
use crate::json::{Misread, Misreading};

/// A scalar as the member that reads it takes it ([`Read::scalar`]): within the declared type
/// (an integer fits its width, a float is the number read, beyond its width an infinity).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Scalar<'v> {
    /// A bool.
    Bool(bool),
    /// A non-negative integer.
    UInt(u64),
    /// A negative integer.
    Int(i64),
    /// A real.
    Real(f64),
    /// A string.
    Text(Cow<'v, str>),
}

impl Scalar<'_> {
    /// The integer, when the scalar is one.
    #[must_use]
    pub fn integer(&self) -> Option<i128> {
        match *self {
            Self::UInt(u) => Some(i128::from(u)),
            Self::Int(i) => Some(i128::from(i)),
            _ => None,
        }
    }

    /// The number as a double, when the scalar is one.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // the nearest double is the reading
    pub fn real(&self) -> Option<f64> {
        match *self {
            Self::UInt(u) => Some(u as f64),
            Self::Int(i) => Some(i as f64),
            Self::Real(x) => Some(x),
            _ => None,
        }
    }
}

/// What a member of `declared` expects, in the words of a wrong type.
fn expected(declared: Declared) -> Expected {
    match declared {
        Declared::Bool => Expected::Bool,
        Declared::Text => Expected::String,
        _ => Expected::Number,
    }
}

/// The integer `value` as a scalar.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // in range by construction
fn integer(value: i128) -> Scalar<'static> {
    if value < 0 {
        Scalar::Int(value as i64)
    } else {
        Scalar::UInt(value as u64)
    }
}

/// Whether `x` is beyond a float of `bits`: its nearest float of that width is an infinity.
#[allow(clippy::cast_possible_truncation)] // the nearest f32 is the question
fn beyond(bits: u8, x: f64) -> bool {
    if bits == 32 {
        (x as f32).is_infinite()
    } else {
        x.is_infinite()
    }
}

impl Read<'_> {
    /// The scalar `node` holds, as a member of the type `declared` reads it under the reading's
    /// [`scalars`](crate::versioning::Reading::scalars) profile and its
    /// [`beyond_width`](crate::versioning::Reading::beyond_width) rule ([`judge`]), telling what
    /// the typed door tells for the same token:
    ///
    /// - [`Scalars::Strict`], [`Scalars::JsonCpp`], [`Scalars::NonStrict`]: the token the
    ///   profile reads (kept, or respelt: `257` is `1` to the non-strict `u8` reader), or a
    ///   [wrong type](Self::wrong_kind) where it refuses it (out of range included);
    /// - [`Scalars::ConvertAndTell`]: a token of the member's JSON type as written (an integer
    ///   out of the member's range an invalid value, an integral real read as the integer, any
    ///   other real a wrong type); a numeric string for a number, `"true"`/`"false"` for a bool
    ///   or a number for a string read converted and told as a wrong type that does not fail
    ///   the member (a [`Pass::Misread`] error under [`Passes::AfterRead`](super::Passes::AfterRead)); a bool for a number or a
    ///   string, and a string that does not convert, a wrong type;
    /// - a number beyond its float read as the infinity it is, and an information that says so
    ///   (`BEYOND_FLOAT_MEMBER`, [`BeyondWidth::ClampAndTell`]: the versioned load clamps it in
    ///   the text and reads again) or a wrong type ([`BeyondWidth::Error`]);
    /// - `null`, an array or an object a wrong type under every profile.
    ///
    /// Under [`Passes::AfterRead`](super::Passes::AfterRead) with `DefaultAndTell` a member (not an array element) whose token
    /// the profile refuses is read as absent, an information in [`Pass::Misread`]; the owner
    /// takes it as absent ([`take_defaulted`](Self::take_defaulted)).
    ///
    /// ```
    /// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    /// # use bedrock_addon_core::json::kit::{Disposition, Node, Read};
    /// # use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc};
    /// # use bedrock_addon_core::versioning::Reading;
    /// # fn told(d: &Diagnostics) -> Vec<String> {
    /// #     d.iter().map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code)).collect()
    /// # }
    /// use bedrock_addon_core::json::Declared;
    /// use bedrock_addon_core::json::kit::Scalar;
    /// use bedrock_addon_core::versioning::Scalars;
    /// let byte = Declared::Int { bits: 8, signed: false };
    /// let json: Value = from_str_jsonc(r#"[3, 3.0, 256, "3"]"#).unwrap();
    /// let root = Node::new(&json, JsonPath::of_keys(["v"]));
    /// let mut found = Diagnostics::default();
    /// let mut read = Read::new(Reading::DOCUMENT.with_scalars(Scalars::Strict), &mut found);
    /// let strict: Vec<Option<Scalar>> = read.array(&root).unwrap().iter().map(|n| read.scalar(&n, byte)).collect();
    /// assert_eq!(strict, [Some(Scalar::UInt(3)), None, None, None]);
    /// assert_eq!(told(&found).len(), 3);
    /// ```
    pub fn scalar<'v>(&mut self, node: &Node<'v>, declared: Declared) -> Option<Scalar<'v>> {
        if self.stopped {
            return None;
        }
        let Some(token) = Token::of(node.value) else {
            self.wrong_kind(node, expected(declared));
            return None;
        };
        if let Declared::Float { bits } = declared
            && let Some(x) = number(token)
            && beyond(bits, x)
            && self.member_rules(node).is_some_and(|rules| rules.finite)
        {
            // The field reads the largest finite float of the sign (`schema::finite`).
            self.defer(
                Pass::Misread,
                crate::molang::beyond_f32_warning(node.path.clone(), node.span()),
            );
            let largest = if bits == 32 {
                f64::from(f32::MAX)
            } else {
                f64::MAX
            };
            return Some(Scalar::Real(largest.copysign(x)));
        }
        let rules = self.reading.scalar_rules();
        if rules.scalars == Scalars::ConvertAndTell {
            return self.convert_and_tell(node, declared, token);
        }
        match judge(declared, token, rules) {
            Verdict::Keep => Some(as_written(token)),
            Verdict::Respell(respelt) => {
                self.tell_rewrite(node, token, &respelt);
                Some(match respelt {
                    Respelt::Bool(b) => Scalar::Bool(b),
                    Respelt::UInt(u) => Scalar::UInt(u),
                    Respelt::Int(i) => Scalar::Int(i),
                    Respelt::Real(x) => Scalar::Real(x),
                })
            }
            Verdict::BeyondFloat => {
                self.misread(node, Misreading::BeyondFloat);
                Some(as_written(token))
            }
            // Only `ConvertAndTell` converts, and it is read above.
            Verdict::Converted => {
                self.converted(node, declared);
                Some(as_written(token))
            }
            Verdict::Refused(_) => {
                if !self.refuse_negative(node, declared, token) {
                    self.refused_scalar(node, expected(declared));
                }
                None
            }
        }
    }

    /// Tells a negative whole number given to an unsigned integer member under the member's
    /// `schema::negative` code, when it states one: `true` when told.
    fn refuse_negative(&mut self, node: &Node<'_>, declared: Declared, token: Token<'_>) -> bool {
        let (Declared::Int { signed: false, .. }, Token::Int(i)) = (declared, token) else {
            return false;
        };
        let Some(code) = self
            .member_rules(node)
            .and_then(|rules| rules.negative)
            .filter(|_| i < 0)
        else {
            return false;
        };
        self.refuse(node, Code::from(code));
        true
    }

    /// Tells a token read as the other JSON type (a number as a bool, a bool as a number), when
    /// the read says to ([`Read::non_strict`]).
    fn tell_rewrite(&mut self, node: &Node<'_>, token: Token<'_>, respelt: &Respelt) {
        let Some(told) = self.rewrite_note else {
            return;
        };
        if matches!(token, Token::Bool(_)) == matches!(respelt, Respelt::Bool(_)) {
            return;
        }
        let mut finding = Diagnostic::of(&core_codes::VALUE_REWRITTEN)
            .at(node.path.clone())
            .detail(DiagnosticDetail::Note(told));
        finding.location.span = node.span();
        self.push(finding);
    }

    /// The plain typed read's rule ([`Scalars::ConvertAndTell`]).
    fn convert_and_tell<'v>(
        &mut self,
        node: &Node<'v>,
        declared: Declared,
        token: Token<'v>,
    ) -> Option<Scalar<'v>> {
        let wrong = |read: &mut Self| {
            read.wrong_kind(node, expected(declared));
            None
        };
        match (declared, token) {
            (Declared::Text, Token::Str(text)) => Some(Scalar::Text(Cow::Borrowed(text))),
            (Declared::Text, Token::UInt(u)) => {
                Some(self.converted_to(node, declared, u.to_string()))
            }
            (Declared::Text, Token::Int(i)) => {
                Some(self.converted_to(node, declared, i.to_string()))
            }
            (Declared::Text, Token::Real(x)) => {
                Some(self.converted_to(node, declared, format!("{x}")))
            }
            (Declared::Bool, Token::Bool(b)) => Some(Scalar::Bool(b)),
            (Declared::Bool, Token::Str(text)) => match text.parse::<bool>() {
                Ok(b) => {
                    self.converted(node, declared);
                    Some(Scalar::Bool(b))
                }
                Err(_) => wrong(self),
            },
            (Declared::Int { bits, signed }, Token::UInt(_) | Token::Int(_)) => {
                let value = as_written(token).integer().unwrap_or_default();
                if fits(value, bits, signed) {
                    Some(as_written(token))
                } else {
                    if !self.refuse_negative(node, declared, token) {
                        self.refuse(node, Code::from(&core_codes::INVALID_VALUE));
                    }
                    None
                }
            }
            (Declared::Int { bits, signed }, Token::Real(x)) => {
                let integral = x.is_finite() && x.fract() == 0.0 && x.abs() < 1e38;
                // An unsigned member takes no negative real, `-0.0` included.
                let sign = signed || !x.is_sign_negative();
                #[allow(clippy::cast_possible_truncation)] // integral and below 1e38
                let value = x as i128;
                if integral && sign && fits(value, bits, signed) {
                    Some(integer(value))
                } else {
                    wrong(self)
                }
            }
            (Declared::Int { bits, signed }, Token::Str(text)) => match text.parse::<i128>() {
                Ok(value) if fits(value, bits, signed) => {
                    self.converted(node, declared);
                    Some(integer(value))
                }
                _ => wrong(self),
            },
            (Declared::Float { bits }, Token::UInt(_) | Token::Int(_) | Token::Real(_)) => {
                let x = as_written(token).real().unwrap_or_default();
                if beyond(bits, x) {
                    match self.reading.beyond_width {
                        BeyondWidth::Error => return wrong(self),
                        _ => self.misread(node, Misreading::BeyondFloat),
                    }
                }
                Some(as_written(token))
            }
            (Declared::Float { bits }, Token::Str(text)) => {
                let parsed = if bits == 32 {
                    text.parse::<f32>().map(f64::from).ok()
                } else {
                    text.parse::<f64>().ok()
                };
                match parsed {
                    Some(x) => {
                        self.converted(node, declared);
                        Some(Scalar::Real(x))
                    }
                    None => wrong(self),
                }
            }
            // `null`, and a bool for anything but a bool, under every profile.
            _ => wrong(self),
        }
    }

    /// `text` read for a string member from a token of another type, and told.
    fn converted_to<'v>(
        &mut self,
        node: &Node<'v>,
        declared: Declared,
        text: String,
    ) -> Scalar<'v> {
        self.converted(node, declared);
        Scalar::Text(Cow::Owned(text))
    }

    /// Tells that `node`'s token was read converted across JSON types: in [`Pass::Misread`]
    /// under [`Passes::AfterRead`](super::Passes::AfterRead) (the door's misread wrong type), a
    /// wrong type otherwise. The member keeps the value read.
    fn converted(&mut self, node: &Node<'_>, declared: Declared) {
        if self.profile.defers() {
            self.misread(node, Misreading::WrongType);
        } else {
            // The member reads the value; the finding stands.
            self.wrong_kind(node, expected(declared));
        }
    }

    /// Tells a scalar token the reading's profile refuses where a value that `expected` was
    /// read: a wrong type, or under [`Passes::AfterRead`](super::Passes::AfterRead) with `DefaultAndTell` a member (not an array
    /// element) read as absent, as the door's scalar pre-pass reads it; the owner takes it as
    /// absent ([`take_defaulted`](Self::take_defaulted)).
    // the derived reads' protocol (an untagged enum's scalar no arm takes)
    pub(crate) fn refused_scalar(&mut self, node: &Node<'_>, expected: Expected) {
        if self.stopped {
            return;
        }
        // The door's pre-pass blanks a struct's member and a map's value, but a map's `null`
        // stays for the null rule.
        let member = node.path.last_key().is_some() && !(self.map_value && node.value.is_null());
        let defaults = self.reading.scalar_rules().default_and_tell;
        // The door's pre-pass blanks it; a read that recovers otherwise fails the value.
        let blanked = self.profile.recovery == super::Recovery::Member;
        if self.profile.defers() && defaults && member && blanked {
            if node.value.is_null()
                && self.reading.null == crate::versioning::NullHandling::Rejected
            {
                // The pre-pass blanks the member, and the door's null rule still finds the
                // `null` where the struct reads a value: a null error, in its pass.
                self.null_error(node);
            } else {
                self.misread(node, Misreading::Defaulted);
            }
            self.defaulted = true;
        } else {
            self.wrong_kind(node, expected);
        }
    }

    /// Tells that the reading refuses the `null` at `node` ([`NullHandling::Rejected`]) in
    /// [`Pass::Null`]: the null pass's error, after the read.
    ///
    /// [`NullHandling::Rejected`]: crate::versioning::NullHandling::Rejected
    pub(crate) fn null_error(&mut self, node: &Node<'_>) {
        let mut finding = match self.texts.wrong_at {
            // The family words a wrong type by where it is; a refused `null` is one.
            Some(WrongAt(word)) => Diagnostic::error(word(&node.path)),
            None => Diagnostic::of(&core_codes::INVALID_TYPE),
        }
        .at(node.path.clone());
        finding.location.span = node.span();
        self.defer(Pass::Null, finding);
    }

    /// Tells the misreading `kind` at `node`, as the door tells it ([`Misread::finding`]),
    /// spanning the token, in [`Pass::Misread`].
    fn misread(&mut self, node: &Node<'_>, kind: Misreading) {
        let mut finding = Misread {
            path: JsonPath::new(),
            kind,
        }
        .finding(&node.path);
        finding.location.span = node.span();
        self.defer(Pass::Misread, finding);
    }
}

/// The number of a numeric token.
#[allow(clippy::cast_precision_loss)] // the nearest double is the reading
fn number(token: Token<'_>) -> Option<f64> {
    match token {
        Token::UInt(u) => Some(u as f64),
        Token::Int(i) => Some(i as f64),
        Token::Real(x) => Some(x),
        _ => None,
    }
}

/// The token as a scalar, as written.
fn as_written(token: Token<'_>) -> Scalar<'_> {
    match token {
        Token::Bool(b) => Scalar::Bool(b),
        Token::UInt(u) => Scalar::UInt(u),
        Token::Int(i) => Scalar::Int(i),
        Token::Real(x) => Scalar::Real(x),
        Token::Str(text) => Scalar::Text(Cow::Borrowed(text)),
        // Not asked: `null` is refused before.
        Token::Null => Scalar::Bool(false),
    }
}
