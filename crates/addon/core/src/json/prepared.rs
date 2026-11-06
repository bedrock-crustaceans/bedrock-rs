//! The gate every JSON reader runs over a file's bytes before it reads them: the BOM blanked,
//! UTF-8 checked, the nesting limit checked.
//!
//! It stops there on purpose. Last-wins over duplicate members, the envelope probe and the
//! clamp of numbers beyond a double run inside `versioning::parse_as_written` for the version
//! chains, in an order the chain decides (the version is picked from the text as written,
//! before a duplicate is blanked); doing any of them here would apply them twice.

use std::borrow::Cow;

use super::lex::{Lexer, Rules};
use super::slice::JsonSlice;
use super::text::{MAX_JSON_DEPTH, TooDeep, blank_bom, check_depth};

/// Why a file's bytes are no JSON text a reader takes. Each family maps the variant to its own
/// finding and wording.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PrepareError {
    /// Zero bytes. Raised by [`Prepared::non_empty`] only; a BOM alone is not empty.
    #[error("empty input")]
    Empty,
    /// The nesting passes [`MAX_JSON_DEPTH`].
    #[error(transparent)]
    TooDeep(#[from] TooDeep),
    /// The bytes are not UTF-8.
    #[error(transparent)]
    NotUtf8(#[from] std::str::Utf8Error),
}

/// A file's text as a JSON reader is given it: a leading BOM replaced by three spaces (so every
/// offset into the text is an offset into the file), valid UTF-8, nested no deeper than
/// [`MAX_JSON_DEPTH`]. Otherwise exactly the bytes as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prepared<'a> {
    text: Cow<'a, str>,
}

impl<'a> Prepared<'a> {
    /// Blanks the BOM, then checks UTF-8, then the nesting. Never refuses empty input: zero
    /// bytes are the empty text, which a version chain reports as `EmptyDocument` at the
    /// family's severity.
    ///
    /// # Errors
    ///
    /// `NotUtf8` for bytes that are not UTF-8 (checked first, so a text both malformed and
    /// over-deep is told as not UTF-8); `TooDeep` with the offset of the first bracket past
    /// the limit.
    pub fn new(bytes: &'a [u8]) -> Result<Self, PrepareError> {
        let text = match blank_bom(bytes) {
            Cow::Borrowed(bytes) => Cow::Borrowed(std::str::from_utf8(bytes)?),
            Cow::Owned(bytes) => Cow::Owned(String::from_utf8(bytes).map_err(|e| e.utf8_error())?),
        };
        check_depth(text.as_bytes(), MAX_JSON_DEPTH)?;
        Ok(Self { text })
    }

    /// [`new`](Self::new), refusing zero bytes first: the gate of the readers that tell an
    /// empty file apart before any BOM handling (fog, `behavior_tree`, credits, `json_ui`). A BOM
    /// alone passes.
    ///
    /// # Errors
    ///
    /// `Empty` for zero bytes, then as [`new`](Self::new).
    pub fn non_empty(bytes: &'a [u8]) -> Result<Self, PrepareError> {
        if bytes.is_empty() {
            return Err(PrepareError::Empty);
        }
        Self::new(bytes)
    }

    /// The text.
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The text, borrowed from the bytes unless a BOM was blanked.
    pub fn text(&self) -> &Cow<'a, str> {
        &self.text
    }

    /// The text, owned or borrowed.
    pub fn into_text(self) -> Cow<'a, str> {
        self.text
    }

    /// The whole text as a slice at offset 0: its spans are offsets into the file.
    pub fn slice(&self) -> JsonSlice<'_> {
        JsonSlice::new(&self.text)
    }

    /// Whether the text holds nothing but JSON white space (space, tab, line feed, carriage
    /// return) and comments. An unterminated block comment counts as a comment, as facet-json
    /// accepts it at the end of a text. `null` is not blank here; the envelope's
    /// `EmptyDocument` rule adds it on its own.
    pub fn is_blank(&self) -> bool {
        let mut lexer = Lexer::new(self.text.as_bytes(), Rules::lenient(false));
        lexer.skip_trivia();
        lexer.at() >= self.text.len()
    }
}
