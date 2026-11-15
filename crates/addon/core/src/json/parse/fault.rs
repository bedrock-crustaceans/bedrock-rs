//! What [`parse`](super::parse) refuses a text for, in the words and at the span the door
//! (`from_str_jsonc::<Value>` over facet-json) refuses it with.

use facet_format::DeserializeErrorKind;
use facet_json::DeserializeError;

use crate::diagnostic::Span;

/// What a bad string escape should have been.
pub(super) const ESCAPE: &str =
    "one of `\"\\/bfnrtu` after a backslash (bad escape sequence in string)";
/// What a string must have after `\u`.
pub(super) const UNICODE_ESCAPE: &str =
    "four hexadecimal digits after `\\u` (bad unicode escape sequence in string)";
/// What a bracket past the limit should have been.
pub(super) const SHALLOWER: &str = "at most 128 levels of nested arrays and objects";
/// What every byte that starts no token should have been.
pub(super) const TOKEN: &str = "valid JSON token";

/// The first syntax fault of a text: where it is and what it is.
///
/// The faults are the door's: the same span (a byte range of the text, rebased by the offset of
/// a [`JsonSlice`](crate::json::JsonSlice) when the text is one) and the same words as the
/// error facet-json gives (`DeserializeError` carries them: `From<Fault>`), so a finding told
/// from one is told from the other.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{kind} (at byte {}, {} long)", span.offset, span.len)]
pub struct Fault {
    /// Where in the text.
    pub span: Span,
    /// What is wrong there.
    pub kind: FaultKind,
}

/// What a [`Fault`] is. The variants are the shapes of syntax error the door tells; the
/// strings are facet-json's own words for what was wanted.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum FaultKind {
    /// A byte that starts no token, a literal or number that goes wrong at this character, a
    /// bad string escape, or a bracket past the nesting limit.
    #[error("unexpected character {ch:?}, expected {expected}")]
    UnexpectedChar {
        /// The character (a byte that is no ASCII is the character of that byte value).
        ch: char,
        /// What was wanted.
        expected: &'static str,
    },
    /// The text ends where more of a token or of the grammar belongs (an unended string,
    /// literal, comment or unicode escape, or a container or value that was not finished).
    #[error("unexpected end of input, expected {expected}")]
    UnexpectedEof {
        /// What was wanted.
        expected: &'static str,
    },
    /// A token the grammar does not take here.
    #[error("unexpected token: got {got}, expected {expected}")]
    UnexpectedToken {
        /// The token, as facet-json names it (`Comma`, `U64(2)`, `String("a")`).
        got: String,
        /// What was wanted.
        expected: &'static str,
    },
    /// A `\u` escape that does not name a character (a surrogate without its pair).
    #[error("invalid UTF-8")]
    InvalidUtf8,
    /// The text is longer than the 4 GiB a span can address.
    #[error("the text is longer than a span can address")]
    TextTooLong,
}

impl Fault {
    pub(super) fn new(offset: usize, len: usize, kind: FaultKind) -> Self {
        Self {
            span: span_of(offset, len),
            kind,
        }
    }

    /// The fault with its span moved by `offset` (a slice's place in its file).
    #[must_use]
    pub fn rebased(mut self, offset: usize) -> Self {
        self.span.offset = self.span.offset.saturating_add(to_u32(offset));
        self
    }

    /// The error facet-json gives for the same fault: its words and its kind, which
    /// `code_for_deserialize_error_kind` maps to a code.
    #[must_use]
    pub fn to_error(&self) -> DeserializeError {
        self.clone().into()
    }

    /// Whether the text ended before the grammar was done: the one fault a reader may take
    /// for an empty document when the text was blank.
    pub fn is_end_of_input(&self) -> bool {
        matches!(self.kind, FaultKind::UnexpectedEof { .. })
    }
}

/// A span of `len` bytes at `offset`; a number past `u32` is the largest one.
pub(super) fn span_of(offset: usize, len: usize) -> Span {
    Span {
        offset: to_u32(offset),
        len: to_u32(len),
    }
}

/// `n` as a `u32`, saturating.
pub(super) fn to_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// The error facet-json gives for the same fault: the one a door caller already maps to a
/// finding.
impl From<Fault> for DeserializeError {
    fn from(fault: Fault) -> Self {
        let kind = match fault.kind {
            FaultKind::UnexpectedChar { ch, expected } => {
                DeserializeErrorKind::UnexpectedChar { ch, expected }
            }
            FaultKind::UnexpectedEof { expected } => {
                DeserializeErrorKind::UnexpectedEof { expected }
            }
            FaultKind::UnexpectedToken { got, expected } => DeserializeErrorKind::UnexpectedToken {
                got: got.into(),
                expected,
            },
            FaultKind::InvalidUtf8 => DeserializeErrorKind::InvalidUtf8 {
                context: [0; 16],
                context_len: 0,
            },
            FaultKind::TextTooLong => DeserializeErrorKind::InvalidValue {
                message: "the text is longer than a span can address".into(),
            },
        };
        DeserializeError {
            span: Some(facet::Span::new(
                fault.span.offset as usize,
                fault.span.len as usize,
            )),
            path: None,
            kind,
        }
    }
}
