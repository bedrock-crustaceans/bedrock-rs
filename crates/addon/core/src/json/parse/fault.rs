//! What [`parse`](super::parse) refuses a text for: the words and the span of each fault.

use crate::diagnostic::{Code, Span, builders, codes};

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
/// The span is a byte range of the text, rebased by the offset of a
/// [`JsonSlice`](crate::json::JsonSlice) when the text is one; the words are those the crate
/// family has always told for the fault (facet-json's, from when it read the text).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{kind} (at byte {}, {} long)", span.offset, span.len)]
pub struct Fault {
    /// Where in the text.
    pub span: Span,
    /// What is wrong there.
    pub kind: FaultKind,
}

/// What a [`Fault`] is. The variants are the shapes of syntax error the parser tells; the
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
    /// A `\u` escape that does not name a character (a surrogate without its pair), or a
    /// string of bytes that are not UTF-8 ([`parse_bytes`](super::parse_bytes)).
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

    /// Whether the text ended before the grammar was done: the one fault a reader may take
    /// for an empty document when the text was blank.
    pub fn is_end_of_input(&self) -> bool {
        matches!(self.kind, FaultKind::UnexpectedEof { .. })
    }

    /// The code of a finding of the fault: `EMPTY_DOCUMENT` where a `blank` text (empty or
    /// white space only) ended, else `core/syntax` with the fault's words.
    #[must_use]
    pub fn code(&self, blank: bool) -> Code {
        if blank && self.is_end_of_input() {
            Code::from(&codes::EMPTY_DOCUMENT)
        } else {
            builders::syntax(self.kind.to_string())
        }
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
