//! The tokenizer of the door: what reads a JSON(C) text token by token before facet sees it,
//! the one place that knows how a string, a number, white space and a comment end.
//! [`scan`] and the helpers below were `read`'s; every other scanner of `json` folds onto
//! them.

use std::ops::Range;

use super::text::MAX_JSON_DEPTH;
use crate::diagnostic::{JsonPath, Span};

/// What [`scan`] found in a text facet may read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scanned {
    /// The byte offset just past the root value. `None` when the text has no whole root value
    /// (it is blank, cut off, or not JSON): the parser reports that.
    pub root_end: Option<usize>,
    /// The byte offset of the first byte after the root value that is neither white space nor
    /// a comment: vanilla's reader never looks at it.
    pub trailing: Option<usize>,
}

/// Why [`scan`] keeps a text from facet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum UnsafeText {
    /// A backslash in a string before a character that is no escape.
    #[error("bad escape sequence `\\{found}` in a string at byte {offset}")]
    BadEscape {
        /// The byte offset of the backslash.
        offset: usize,
        /// The length of the backslash and the character.
        len: usize,
        /// The character after the backslash.
        found: char,
    },
    /// `\u` without four hexadecimal digits after it.
    #[error("bad unicode escape sequence in a string at byte {offset}")]
    BadUnicodeEscape {
        /// The byte offset of the backslash.
        offset: usize,
        /// The length up to and including the first character that is no digit.
        len: usize,
        /// That character; `None` when the text ends first.
        found: Option<char>,
    },
    /// An array or object nested deeper than `MAX_JSON_DEPTH`.
    #[error("JSON nesting deeper than the limit at byte {offset}")]
    TooDeep {
        /// The byte offset of the first bracket past the limit.
        offset: usize,
    },
    /// A `/*` comment before the end of the root value that has no `*/` (in a text read with
    /// comments).
    #[error("block comment without an end at byte {offset}")]
    UnterminatedComment {
        /// The byte offset of the comment's slash.
        offset: usize,
        /// The length from the slash to the end of the text.
        len: usize,
    },
}

impl UnsafeText {
    /// Where in the scanned text.
    pub fn span(&self) -> Span {
        let (offset, len) = match *self {
            Self::BadEscape { offset, len, .. }
            | Self::BadUnicodeEscape { offset, len, .. }
            | Self::UnterminatedComment { offset, len } => (offset, len),
            Self::TooDeep { offset } => (offset, 1),
        };
        let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        Span {
            offset: to_u32(offset),
            len: to_u32(len),
        }
    }
}

/// Reads `bytes` token by token the way facet-json's scanner does (white space, `//` and
/// `/* */` comments when `comments`, one byte of punctuation, strings, numbers, the three
/// literals) and refuses a string escape outside RFC 8259, nesting past the limit and a `/*`
/// comment that does not end before the root value does. Every other fault (a string that
/// does not end, a missing comma, a stray byte) is left for the parser to report, which it
/// does without decoding anything.
///
/// Linear in the text, one pass, no allocation beyond the open brackets.
///
/// # Errors
///
/// Returns `UnsafeText` for a string escape outside RFC 8259, nesting past the limit, or a `/*` comment that does not end before the root value.
pub fn scan(bytes: &[u8], comments: bool) -> Result<Scanned, UnsafeText> {
    scan_tokens(bytes, comments, &mut Marks::default()).map_err(|refused| refused.found)
}

/// What [`scan_tokens`] refused, and where the token that holds it starts: the text before
/// that offset is text the scan passed.
pub(super) struct Refused {
    pub(super) found: UnsafeText,
    /// The offset of the string's opening quote, of the bracket past the limit, or of the
    /// slash of the comment without an end.
    pub(super) token: usize,
}

/// What the scan marks for the copy facet reads in place of a text (`read::facet_copy`).
#[derive(Default)]
pub(super) struct Marks {
    /// The integer tokens too large for 64 bits.
    pub(super) wide: Vec<Range<usize>>,
    /// The whole comments, each without the newline that ends a `//` one.
    pub(super) comments: Vec<Range<usize>>,
    /// The most arrays and objects open at once: how deep the text nests.
    pub(super) levels: usize,
}

impl Marks {
    /// The marks that lie within the first `len` bytes.
    pub(super) fn within(&self, len: usize) -> Self {
        let within = |ranges: &[Range<usize>]| {
            ranges
                .iter()
                .filter(|range| range.end <= len)
                .cloned()
                .collect()
        };
        Self {
            wide: within(&self.wide),
            comments: within(&self.comments),
            levels: self.levels,
        }
    }
}

/// The refusal of the `/*` comment at `offset` that has no end.
fn unended_comment(bytes: &[u8], offset: usize) -> Refused {
    Refused {
        found: UnsafeText::UnterminatedComment {
            offset,
            len: bytes.len() - offset,
        },
        token: offset,
    }
}

/// [`scan`], which also marks where the integer tokens too large for 64 bits and the
/// comments lie.
pub(super) fn scan_tokens(
    bytes: &[u8],
    comments: bool,
    marks: &mut Marks,
) -> Result<Scanned, Refused> {
    // The open brackets. `sure` is whether the text so far is one the parser reads the same
    // way: once a bracket closes another kind, a literal is misspelt or a byte starts no
    // token, the parser has failed there, and the scan only goes on checking strings (from
    // every position the parser's cursor could be left at) without naming a root end.
    let mut open: Vec<u8> = Vec::new();
    let mut sure = true;
    let mut lexer = Lexer::new(bytes, Rules::strict(comments));
    let root_end = loop {
        let unended = lexer.skip_trivia_with(|comment| {
            // Past a fault the parser reads nothing, and its message may name a byte there:
            // nothing is marked for the copy, so the byte it names is the author's.
            if sure {
                marks.comments.push(comment);
            }
        });
        // A `/*` without an end inside the root: the parser's message for it depends on
        // whether it still knows the text has comments, so the scan reports it, in
        // the parser's words. Past a fault the parser never gets this far.
        if let Some(offset) = unended.filter(|_| sure) {
            return Err(unended_comment(bytes, offset));
        }
        let start = lexer.at();
        let token = match lexer.token() {
            Ok(Some(token)) => token,
            Ok(None) => {
                return Ok(Scanned {
                    root_end: None,
                    trailing: None,
                });
            }
            // A string that does not end is the parser's to report; one with an escape
            // outside RFC 8259 is the scan's.
            Err(found) => {
                return Err(Refused {
                    found,
                    token: start,
                });
            }
        };
        // Whether the token is a whole value (a scalar, or the bracket closing a container).
        let value = match token.kind {
            Kind::String { closed } => closed,
            Kind::OpenObject | Kind::OpenArray => {
                if open.len() >= MAX_JSON_DEPTH {
                    return Err(Refused {
                        found: UnsafeText::TooDeep { offset: start },
                        token: start,
                    });
                }
                open.push(bytes[start]);
                marks.levels = marks.levels.max(open.len());
                false
            }
            Kind::CloseObject | Kind::CloseArray => {
                let opener = if token.kind == Kind::CloseObject {
                    b'{'
                } else {
                    b'['
                };
                if open.last() == Some(&opener) {
                    open.pop();
                    true
                } else {
                    sure = false;
                    false
                }
            }
            Kind::Colon | Kind::Comma => {
                sure &= !open.is_empty();
                false
            }
            Kind::Number => {
                if sure && is_wide_integer(&bytes[token.range()]) {
                    marks.wide.push(token.range());
                }
                let digits = token.len() > usize::from(bytes[start] == b'-');
                sure &= digits;
                digits
            }
            // The parser's cursor stays on the first byte that differs.
            Kind::Literal { whole } => {
                sure &= whole;
                whole
            }
            // No token starts here: the parser stops on this byte.
            Kind::Atom | Kind::Other => {
                sure = false;
                false
            }
        };
        if value && sure && open.is_empty() {
            break lexer.at();
        }
    };
    Ok(Scanned {
        root_end: Some(root_end),
        trailing: content_from(bytes, root_end, comments),
    })
}

/// How a [`Lexer`] reads a text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Rules {
    /// `//` and `/* */` comments are white space.
    pub(super) comments: bool,
    /// A form feed (0x0C) is white space too, as `u8::is_ascii_whitespace` has it: the
    /// lenient scanners that took it so before the fold. JSON's white space, facet's and the
    /// game's, is the other four bytes.
    pub(super) form_feed: bool,
    /// Whether strings, numbers and literals are read by the grammar or loosely.
    pub(super) mode: Mode,
}

impl Rules {
    /// facet-json's reading ([`scan`]'s): JSON white space, comments when `comments`.
    pub(super) const fn strict(comments: bool) -> Self {
        Self {
            comments,
            form_feed: false,
            mode: Mode::Strict,
        }
    }

    /// The pre-parse scanners' reading: comments always, a backslash skips the byte after
    /// it unread, anything that is no punctuation and no string a run of bytes.
    pub(super) const fn lenient(form_feed: bool) -> Self {
        Self {
            comments: true,
            form_feed,
            mode: Mode::Lenient,
        }
    }

    /// Whether `byte` is white space.
    fn is_space(self, byte: u8) -> bool {
        matches!(byte, b' ' | b'\t' | b'\n' | b'\r') || (self.form_feed && byte == 0x0C)
    }
}

/// How a [`Lexer`] reads what is no punctuation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    /// By facet-json's grammar: a string's escapes are RFC 8259's or the text is refused
    /// ([`UnsafeText`]); a number to the end of facet's number; `true`, `false`, `null` to
    /// the first byte that differs; any other byte alone.
    Strict,
    /// The way the scans that only bound or place a text read it, which the parser then
    /// reads in full: a backslash in a string skips the byte after it unread, and a token
    /// that is no punctuation and no string runs to the next white space or `{}[],:"/`
    /// ([`Kind::Atom`]).
    Lenient,
}

/// What a token is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    /// `{`
    OpenObject,
    /// `}`
    CloseObject,
    /// `[`
    OpenArray,
    /// `]`
    CloseArray,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// A string, and whether it has its closing quote (it runs to the end of the text when
    /// it has none).
    String {
        /// The closing quote is there.
        closed: bool,
    },
    /// A number, by facet's grammar ([`Mode::Strict`]): `-` alone is one.
    Number,
    /// `true`, `false` or `null`, up to the first byte that differs ([`Mode::Strict`]).
    Literal {
        /// The whole word is there.
        whole: bool,
    },
    /// A run of bytes up to the next white space or `{}[],:"/`, at least one
    /// ([`Mode::Lenient`]).
    Atom,
    /// One byte that starts no token ([`Mode::Strict`]).
    Other,
}

/// One token of a text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Token {
    /// What it is.
    pub(super) kind: Kind,
    /// Its first byte.
    pub(super) start: usize,
    /// The byte after it.
    pub(super) end: usize,
}

impl Token {
    /// Its bytes, as a range of the text.
    pub(super) fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    /// Its length in bytes.
    pub(super) fn len(&self) -> usize {
        self.end - self.start
    }
}

/// The one tokenizer of the door: a text read token by token, white space and comments
/// between them, by [`Rules`]. Linear in the text; it holds nothing but its place.
#[derive(Clone, Debug)]
pub(super) struct Lexer<'a> {
    bytes: &'a [u8],
    at: usize,
    rules: Rules,
}

impl<'a> Lexer<'a> {
    /// A lexer at the start of `bytes`.
    pub(super) fn new(bytes: &'a [u8], rules: Rules) -> Self {
        Self {
            bytes,
            at: 0,
            rules,
        }
    }

    /// Where it is.
    pub(super) fn at(&self) -> usize {
        self.at
    }

    /// Goes on from `at`.
    pub(super) fn seek(&mut self, at: usize) {
        self.at = at;
    }

    /// Steps past white space and comments, handing `whole` each comment that ends: a `//`
    /// comment without its newline, a `/* */` comment with its `*/`. The offset of the slash of
    /// a `/*` comment that has no end, which runs to the end of the text.
    pub(super) fn skip_trivia_with(
        &mut self,
        mut whole: impl FnMut(Range<usize>),
    ) -> Option<usize> {
        let bytes = self.bytes;
        loop {
            match bytes.get(self.at..) {
                Some([byte, ..]) if self.rules.is_space(*byte) => self.at += 1,
                Some([b'/', b'/', rest @ ..]) if self.rules.comments => {
                    let line = rest
                        .iter()
                        .position(|&byte| byte == b'\n')
                        .unwrap_or(rest.len());
                    whole(self.at..self.at + 2 + line);
                    self.at += 2 + line;
                }
                Some([b'/', b'*', rest @ ..]) if self.rules.comments => {
                    if let Some(n) = rest.windows(2).position(|pair| pair == b"*/") {
                        whole(self.at..self.at + n + 4);
                        self.at += n + 4;
                    } else {
                        let slash = self.at;
                        self.at = bytes.len();
                        return Some(slash);
                    }
                }
                _ => return None,
            }
        }
    }

    /// [`Self::skip_trivia_with`], the comments unlooked at.
    pub(super) fn skip_trivia(&mut self) -> Option<usize> {
        self.skip_trivia_with(|_| {})
    }

    /// The token where the lexer is (white space is not skipped: it is a token of one byte
    /// in [`Mode::Strict`] and starts none in [`Mode::Lenient`]), and steps past it. `None`
    /// at the end of the text.
    ///
    /// # Errors
    ///
    /// In [`Mode::Strict`], the [`UnsafeText`] of a string with an escape outside RFC 8259;
    /// the lexer has not moved.
    pub(super) fn token(&mut self) -> Result<Option<Token>, UnsafeText> {
        let bytes = self.bytes;
        let start = self.at;
        let Some(&byte) = bytes.get(start) else {
            return Ok(None);
        };
        let (kind, end) = match byte {
            b'{' => (Kind::OpenObject, start + 1),
            b'}' => (Kind::CloseObject, start + 1),
            b'[' => (Kind::OpenArray, start + 1),
            b']' => (Kind::CloseArray, start + 1),
            b':' => (Kind::Colon, start + 1),
            b',' => (Kind::Comma, start + 1),
            b'"' => {
                let (end, closed) = match self.rules.mode {
                    Mode::Strict => string_end(bytes, start)?,
                    Mode::Lenient => loose_string_end(bytes, start),
                };
                (Kind::String { closed }, end)
            }
            _ if self.rules.mode == Mode::Lenient => {
                let run = bytes[start..]
                    .iter()
                    .position(|&byte| {
                        self.rules.is_space(byte)
                            || matches!(byte, b'{' | b'}' | b'[' | b']' | b',' | b':' | b'"' | b'/')
                    })
                    .unwrap_or(bytes.len() - start);
                (Kind::Atom, start + run.max(1))
            }
            b'-' | b'0'..=b'9' => (Kind::Number, number_end(bytes, start)),
            b't' | b'f' | b'n' => {
                let literal: &[u8] = match byte {
                    b't' => b"true",
                    b'f' => b"false",
                    _ => b"null",
                };
                let matched = bytes[start..]
                    .iter()
                    .zip(literal)
                    .take_while(|(a, b)| a == b)
                    .count();
                (
                    Kind::Literal {
                        whole: matched == literal.len(),
                    },
                    start + matched,
                )
            }
            _ => (Kind::Other, start + 1),
        };
        self.at = end;
        Ok(Some(Token { kind, start, end }))
    }

    /// The next token after white space and comments; a `/*` comment without an end runs
    /// to the end of the text. `None` at the end.
    ///
    /// # Errors
    ///
    /// As [`Self::token`].
    pub(super) fn next_token(&mut self) -> Result<Option<Token>, UnsafeText> {
        self.skip_trivia();
        self.token()
    }
}

/// The arrays and objects a walk of the tokens is inside, and the path they spell: the one
/// copy of the open frames the scanners of the door keep. It is told what the walk met
/// (a bracket, a key, the end of a member) and keeps the path in step, a segment pushed or
/// popped at a time, so the path costs nothing to keep. When a member's key is taken and
/// when its member ends is the walk's to say: a scan that places a token takes the first
/// string after `{` or `,` as the key ([`Self::awaits_key`]), the clamp walk ends a member
/// with its value.
#[derive(Clone, Debug, Default)]
pub(super) struct Structure {
    frames: Vec<Frame>,
    path: JsonPath,
}

/// One open array or object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Frame {
    /// An object, and whether the key of the member being read is on the path.
    Object {
        /// The key is on the path.
        keyed: bool,
    },
    /// An array, with the index of the element being read on the path.
    Array {
        /// That index.
        index: usize,
    },
}

impl Structure {
    /// No frame open.
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// The path from the root to the member or element being read: a key for each object
    /// whose member has its key, an index for each array.
    pub(super) fn path(&self) -> &JsonPath {
        &self.path
    }

    /// How many arrays and objects are open.
    pub(super) fn depth(&self) -> usize {
        self.frames.len()
    }

    /// The innermost open frame.
    pub(super) fn top(&self) -> Option<Frame> {
        self.frames.last().copied()
    }

    /// Whether the innermost frame is an object whose member has no key yet.
    pub(super) fn awaits_key(&self) -> bool {
        self.top() == Some(Frame::Object { keyed: false })
    }

    /// A `{` opens an object.
    pub(super) fn open_object(&mut self) {
        self.frames.push(Frame::Object { keyed: false });
    }

    /// A `[` opens an array, at its first element.
    pub(super) fn open_array(&mut self) {
        self.frames.push(Frame::Array { index: 0 });
        self.path.push_index(0);
    }

    /// A `}` or `]` closes the innermost frame, whichever it is: the frame closed.
    pub(super) fn close(&mut self) -> Option<Frame> {
        let frame = self.frames.pop()?;
        if matches!(frame, Frame::Array { .. } | Frame::Object { keyed: true }) {
            self.path.pop();
        }
        Some(frame)
    }

    /// The member being read in the innermost object is `key`; nothing in an array or an
    /// object whose member has its key.
    pub(super) fn enter_key(&mut self, key: &str) {
        if let Some(Frame::Object { keyed }) = self.frames.last_mut()
            && !*keyed
        {
            *keyed = true;
            self.path.push(key);
        }
    }

    /// The member being read in the innermost object has ended: the next string is a key.
    pub(super) fn leave_key(&mut self) {
        if let Some(Frame::Object { keyed }) = self.frames.last_mut()
            && *keyed
        {
            *keyed = false;
            self.path.pop();
        }
    }

    /// The innermost array goes on to its next element.
    pub(super) fn next_element(&mut self) {
        if let Some(Frame::Array { index }) = self.frames.last_mut() {
            *index += 1;
            self.path.pop();
            self.path.push_index(*index);
        }
    }

    /// A `,`: the next element of an array, or the next member of an object, whose key is
    /// the next string.
    pub(super) fn comma(&mut self) {
        match self.top() {
            Some(Frame::Array { .. }) => self.next_element(),
            Some(Frame::Object { .. }) => self.leave_key(),
            None => {}
        }
    }
}

/// A key token's text as the reader decodes it: its unescaped value, or the text between the
/// quotes when it does not unescape (a key the reader refused still has a name for a
/// message).
pub(super) fn key_text(token: &str) -> String {
    super::from_str::<String>(token).unwrap_or_else(|_| token.trim_matches('"').to_owned())
}

/// The index after the string whose opening quote is at `at`, read loosely: a backslash
/// skips the byte after it, whatever it is. Whether it has its closing quote; the end of the
/// text when it has none.
pub(super) fn loose_string_end(bytes: &[u8], at: usize) -> (usize, bool) {
    let mut i = at + 1;
    while let Some(&byte) = bytes.get(i) {
        match byte {
            b'\\' => i += 2,
            b'"' => return (i + 1, true),
            _ => i += 1,
        }
    }
    (bytes.len(), false)
}

/// The offset of the first content from `at`: the first byte that is neither white space nor
/// part of a whole comment. A `/*` that never ends is content from its slash (the parser
/// refuses such a comment inside the root value; after it, it is what vanilla never looks
/// at, like anything else). `None` when only white space and whole comments follow.
fn content_from(bytes: &[u8], at: usize, comments: bool) -> Option<usize> {
    let mut lexer = Lexer::new(bytes, Rules::strict(comments));
    lexer.seek(at);
    match lexer.skip_trivia() {
        Some(slash) => Some(slash),
        None => (lexer.at() < bytes.len()).then_some(lexer.at()),
    }
}

/// The index after the string whose opening quote is at `at`, and whether it has a closing
/// quote (the index is the end of the text when it has none). Every escape in it is one of
/// RFC 8259's.
pub(super) fn string_end(bytes: &[u8], at: usize) -> Result<(usize, bool), UnsafeText> {
    let mut i = at + 1;
    loop {
        match bytes.get(i) {
            None => return Ok((bytes.len(), false)),
            Some(b'"') => return Ok((i + 1, true)),
            Some(b'\\') => match bytes.get(i + 1) {
                // A backslash that ends the text: the string does not end.
                None => return Ok((bytes.len(), false)),
                Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => i += 2,
                Some(b'u') => {
                    for digit in i + 2..i + 6 {
                        if !bytes.get(digit).is_some_and(u8::is_ascii_hexdigit) {
                            let found = char_at(bytes, digit);
                            let end = digit + found.map_or(0, |(_, len)| len);
                            return Err(UnsafeText::BadUnicodeEscape {
                                offset: i,
                                len: end - i,
                                found: found.map(|(found, _)| found),
                            });
                        }
                    }
                    i += 6;
                }
                Some(_) => {
                    let (found, len) = char_at(bytes, i + 1).unwrap_or(('\u{fffd}', 1));
                    return Err(UnsafeText::BadEscape {
                        offset: i,
                        len: 1 + len,
                        found,
                    });
                }
            },
            Some(_) => i += 1,
        }
    }
}

/// The character starting at `at` and its length in bytes; U+FFFD and one byte where the bytes
/// are no UTF-8. `None` at the end of the text.
fn char_at(bytes: &[u8], at: usize) -> Option<(char, usize)> {
    let rest = bytes.get(at..).filter(|rest| !rest.is_empty())?;
    let head = &rest[..rest.len().min(4)];
    let valid = match std::str::from_utf8(head) {
        Ok(text) => text,
        Err(error) => std::str::from_utf8(&head[..error.valid_up_to()]).unwrap_or(""),
    };
    Some(
        valid
            .chars()
            .next()
            .map_or(('\u{fffd}', 1), |found| (found, found.len_utf8())),
    )
}

/// The index after the number token starting at `at`, by facet-json's `scan_number`: an
/// optional `-`, digits, an optional `.` and digits, an optional exponent. Whether the
/// characters are a number is the parser's to say.
pub(super) fn number_end(bytes: &[u8], mut at: usize) -> usize {
    let digits = |mut at: usize| {
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        at
    };
    if bytes.get(at) == Some(&b'-') {
        at += 1;
    }
    at = digits(at);
    if bytes.get(at) == Some(&b'.') {
        at = digits(at + 1);
    }
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        at += 1;
        if matches!(bytes.get(at), Some(b'+' | b'-')) {
            at += 1;
        }
        at = digits(at);
    }
    at
}

/// Whether a number token is an integer (digits alone after an optional `-`) that fits
/// neither `i64` nor `u64`: the tokens facet-json reads as 128-bit integers or refuses
/// (`scanner::parse_number`, the one its default `lexical-parse` feature compiles). Every
/// other number token is left to facet.
pub(super) fn is_wide_integer(token: &[u8]) -> bool {
    let digits = token.strip_prefix(b"-").unwrap_or(token);
    // Eighteen digits fit an `i64`, whatever they are.
    if digits.len() <= 18 || !digits.iter().all(u8::is_ascii_digit) {
        return false;
    }
    std::str::from_utf8(token)
        .is_ok_and(|token| token.parse::<i64>().is_err() && token.parse::<u64>().is_err())
}
