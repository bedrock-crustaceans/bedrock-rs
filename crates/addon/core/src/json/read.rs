//! The one door to facet-json's parsers: [`scan`] reads the text first, and the `from_*`
//! functions here are the only calls of `facet_json::from_*` in the crate family
//! (`tests/json_door.rs` holds the sources to it).
//!
//! facet-json (0.50.0-rc.7, pinned) decodes a string of a `&str` input without checking what
//! follows a backslash: `"\é"` makes it read a string from the middle of a character, which is
//! undefined behaviour (an abort in a debug build), `"\u00é…"` builds a `str` that is no UTF-8,
//! `"\q"` reads as `q`, and `"\u+041"` as `A`. It also has no nesting limit. So no text
//! reaches it unless
//! - every string it can decode has only the escapes of RFC 8259 §7 (`\"`, `\\`, `\/`, `\b`,
//!   `\f`, `\n`, `\r`, `\t`, `\u` and four hexadecimal digits): anything else is vanilla's
//!   "Bad escape sequence in string", a syntax error of the file;
//! - no array or object is nested deeper than `MAX_JSON_DEPTH`;
//! - nothing follows the root value: facet is given the text up to its end, so what vanilla's
//!   reader never looks at never reaches facet either ([`trailing_content`] says where it is).
//!
//! An untyped [`Value`] is not read by facet's deserialiser: it reads one by a call of itself
//! for each level, some fourteen kilobytes of stack apiece in a debug build, and a megabyte
//! of stack ended 74 levels down. The door reads a value itself (`untyped`), from facet-json's
//! events and with facet's builder, the open containers in a list: the same value and the same
//! faults, on the same stack however deep the text is. A typed target is facet's to
//! read, a call for each level of the type and of an untyped member below it: the door
//! runs that read with the stack the depth of the text can take (`stack`).
//!
//! And one token is respelt before facet reads it. An integer token (no fraction, no
//! exponent) that fits neither `i64` nor `u64` is, to vanilla's readers (rapidjson, jsoncpp,
//! the Minecraft Document reader), the double it spells. facet gives an untyped `Value` its
//! digits as a string up to 128 bits and refuses the token, and so the file, beyond. The door
//! hands facet a copy of the text in which each such token is a real token of the same
//! length that spells the same double (`respell`), so every offset, and with it every
//! span and raw slice, is the one of the author's text; a text without such a token is read
//! as it is, with no copy.
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "this module is the JSON door: the one caller of facet-json"
)]

use std::borrow::Cow;
use std::collections::VecDeque;
use std::ops::Range;

use facet::{Facet, Partial, ReflectError};
use facet_format::{
    DeserializeErrorKind, FieldKey, FormatDeserializer, FormatParser, ParseError, ParseEvent,
    ParseEventKind, SavePoint, ScalarValue,
};
use facet_json::{DeserializeError, JsonParser};

use super::Value;
use super::slice::JsonSlice;
use super::stack::with_stack_for;
use super::text::MAX_JSON_DEPTH;
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, JsonPath, Location, Phase, Severity, Span, VanillaText,
};

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
    #[must_use]
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

/// What a string may have after a backslash.
const ESCAPE: &str = "one of `\"\\/bfnrtu` after a backslash (bad escape sequence in string)";
/// What a string must have after `\u`.
const UNICODE_ESCAPE: &str =
    "four hexadecimal digits after `\\u` (bad unicode escape sequence in string)";
/// What a bracket past the limit should have been.
const SHALLOWER: &str = "at most 128 levels of nested arrays and objects";
const _: () = assert!(MAX_JSON_DEPTH == 128, "`SHALLOWER` names the limit");
/// What a block comment must have before the text ends: facet-json's own words for it, so
/// the message is the same whether the door or the parser meets the comment.
const COMMENT_END: &str = "in block comment";

/// The finding as the error facet's own parser would give: a parser-level kind (no path), so
/// every caller's mapping reads it as a syntax error (`MalformedInput`) at its span.
impl From<UnsafeText> for DeserializeError {
    fn from(found: UnsafeText) -> Self {
        let span = found.span();
        let kind = match found {
            UnsafeText::BadEscape { found, .. } => DeserializeErrorKind::UnexpectedChar {
                ch: found,
                expected: ESCAPE,
            },
            UnsafeText::BadUnicodeEscape {
                found: Some(found), ..
            } => DeserializeErrorKind::UnexpectedChar {
                ch: found,
                expected: UNICODE_ESCAPE,
            },
            UnsafeText::BadUnicodeEscape { found: None, .. } => {
                DeserializeErrorKind::UnexpectedEof {
                    expected: UNICODE_ESCAPE,
                }
            }
            UnsafeText::TooDeep { .. } => DeserializeErrorKind::UnexpectedChar {
                ch: '[',
                expected: SHALLOWER,
            },
            UnsafeText::UnterminatedComment { .. } => DeserializeErrorKind::UnexpectedEof {
                expected: COMMENT_END,
            },
        };
        DeserializeError {
            span: Some(facet::Span::new(span.offset as usize, span.len as usize)),
            path: None,
            kind,
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
struct Refused {
    found: UnsafeText,
    /// The offset of the string's opening quote, of the bracket past the limit, or of the
    /// slash of the comment without an end.
    token: usize,
}

/// What the scan marks for the copy facet reads in place of a text ([`facet_copy`]).
#[derive(Default)]
struct Marks {
    /// The integer tokens too large for 64 bits.
    wide: Vec<Range<usize>>,
    /// The whole comments, each without the newline that ends a `//` one.
    comments: Vec<Range<usize>>,
    /// The most arrays and objects open at once: how deep the text nests.
    levels: usize,
}

impl Marks {
    /// The marks that lie within the first `len` bytes.
    fn within(&self, len: usize) -> Self {
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
fn scan_tokens(bytes: &[u8], comments: bool, marks: &mut Marks) -> Result<Scanned, Refused> {
    // The open brackets. `sure` is whether the text so far is one the parser reads the same
    // way: once a bracket closes another kind, a literal is misspelt or a byte starts no
    // token, the parser has failed there, and the scan only goes on checking strings (from
    // every position the parser's cursor could be left at) without naming a root end.
    let mut open: Vec<u8> = Vec::new();
    let mut sure = true;
    let mut at = 0;
    let root_end = loop {
        let unended;
        (at, unended) = skip_trivia_with(bytes, at, comments, |comment| {
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
        let Some(&byte) = bytes.get(at) else {
            return Ok(Scanned {
                root_end: None,
                trailing: None,
            });
        };
        let start = at;
        // Whether the token is a whole value (a scalar, or the bracket closing a container).
        let value = match byte {
            b'"' => {
                // A string that does not end is the parser's to report.
                let (end, closed) = string_end(bytes, at).map_err(|found| Refused {
                    found,
                    token: start,
                })?;
                at = end;
                closed
            }
            b'{' | b'[' => {
                if open.len() >= MAX_JSON_DEPTH {
                    return Err(Refused {
                        found: UnsafeText::TooDeep { offset: at },
                        token: at,
                    });
                }
                open.push(byte);
                marks.levels = marks.levels.max(open.len());
                at += 1;
                false
            }
            b'}' | b']' => {
                at += 1;
                let opener = if byte == b'}' { b'{' } else { b'[' };
                if open.last() == Some(&opener) {
                    open.pop();
                    true
                } else {
                    sure = false;
                    false
                }
            }
            b':' | b',' => {
                at += 1;
                sure &= !open.is_empty();
                false
            }
            b'-' | b'0'..=b'9' => {
                at = number_end(bytes, at);
                if sure && is_wide_integer(&bytes[start..at]) {
                    marks.wide.push(start..at);
                }
                let digits = at - start > usize::from(byte == b'-');
                sure &= digits;
                digits
            }
            b't' | b'f' | b'n' => {
                let literal: &[u8] = match byte {
                    b't' => b"true",
                    b'f' => b"false",
                    _ => b"null",
                };
                let matched = bytes[at..]
                    .iter()
                    .zip(literal)
                    .take_while(|(a, b)| a == b)
                    .count();
                // The parser's cursor stays on the first byte that differs.
                at += matched;
                sure &= matched == literal.len();
                matched == literal.len()
            }
            _ => {
                // No token starts here: the parser stops on this byte.
                at += 1;
                sure = false;
                false
            }
        };
        if value && sure && open.is_empty() {
            break at;
        }
    };
    Ok(Scanned {
        root_end: Some(root_end),
        trailing: content_from(bytes, root_end, comments),
    })
}

/// The offset of the first content from `at`: the first byte that is neither white space nor
/// part of a whole comment. A `/*` that never ends is content from its slash (the parser
/// refuses such a comment inside the root value; after it, it is what vanilla never looks
/// at, like anything else). `None` when only white space and whole comments follow.
fn content_from(bytes: &[u8], mut at: usize, comments: bool) -> Option<usize> {
    loop {
        match bytes.get(at..) {
            Some([b' ' | b'\t' | b'\n' | b'\r', ..]) => at += 1,
            Some([b'/', b'/', rest @ ..]) if comments => {
                at += 2 + rest
                    .iter()
                    .position(|&byte| byte == b'\n')
                    .map_or(rest.len(), |n| n + 1);
            }
            Some([b'/', b'*', rest @ ..]) if comments => {
                match rest.windows(2).position(|pair| pair == b"*/") {
                    Some(n) => at += n + 4,
                    None => return Some(at),
                }
            }
            Some([]) | None => return None,
            Some(_) => return Some(at),
        }
    }
}

/// The index after the white space and (when `comments`) comments from `at`, as facet-json's
/// `skip_whitespace` reads them: a `//` comment to its newline, a `/* */` comment to its
/// `*/`, or to the end when it has none (the parser reports that).
pub(super) fn skip_trivia(bytes: &[u8], at: usize, comments: bool) -> usize {
    skip_trivia_with(bytes, at, comments, |_| {}).0
}

/// [`skip_trivia`], handing `whole` each comment that ends: a `//` comment without its
/// newline, a `/* */` comment with its `*/`. With the index comes the offset of the slash of
/// a `/*` comment that has no end.
fn skip_trivia_with(
    bytes: &[u8],
    mut at: usize,
    comments: bool,
    mut whole: impl FnMut(Range<usize>),
) -> (usize, Option<usize>) {
    loop {
        match bytes.get(at..) {
            Some([b' ' | b'\t' | b'\n' | b'\r', ..]) => at += 1,
            Some([b'/', b'/', rest @ ..]) if comments => {
                let line = rest
                    .iter()
                    .position(|&byte| byte == b'\n')
                    .unwrap_or(rest.len());
                whole(at..at + 2 + line);
                at += 2 + line;
            }
            Some([b'/', b'*', rest @ ..]) if comments => {
                match rest.windows(2).position(|pair| pair == b"*/") {
                    Some(n) => {
                        whole(at..at + n + 4);
                        at += n + 4;
                    }
                    None => return (bytes.len(), Some(at)),
                }
            }
            _ => return (at, None),
        }
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
fn is_wide_integer(token: &[u8]) -> bool {
    let digits = token.strip_prefix(b"-").unwrap_or(token);
    // Eighteen digits fit an `i64`, whatever they are.
    if digits.len() <= 18 || !digits.iter().all(u8::is_ascii_digit) {
        return false;
    }
    std::str::from_utf8(token)
        .is_ok_and(|token| token.parse::<i64>().is_err() && token.parse::<u64>().is_err())
}

/// The digits and exponent of the real token that spells the double the integer `digits`
/// spells (the nearest double, as `str::parse` rounds): the shortest digits that read back
/// to that double, then `e` and the exponent that restores its size. `18446744073709551616`
/// gives `18446744073709552e3`. Beyond the range of a double it is `1e999`, so such an
/// integer is read exactly as `1e999` is (infinity in a float; what a family does with a
/// number that large is its rule).
fn real_digits(digits: &str) -> Option<String> {
    let value: f64 = digits.parse().ok()?;
    if value.is_infinite() {
        return Some("1e999".to_owned());
    }
    // `{:e}` gives the shortest digits that read back to the value, as `d.ddde<n>`.
    let scientific = format!("{value:e}");
    let (mantissa, exponent) = scientific.split_once('e')?;
    let significant = mantissa.replace('.', "");
    let exponent = exponent
        .parse::<usize>()
        .ok()?
        .checked_sub(significant.len() - 1)?;
    Some(format!("{significant}e{exponent}"))
}

/// Respells the wide integer `token` in place as a real token of the same length that spells
/// the same double: its sign, zeros to fill the length, then [`real_digits`].
/// `18446744073709551616` becomes `018446744073709552e3`.
///
/// The spelling always fits: a wide integer has at least 19 significant digits, a double at
/// most 17, and the exponent that makes up the difference is shorter than the digits it
/// replaces. The scan hands this wide integers only; bytes that are no number, or too few
/// for the spelling, are left as they are.
fn respell(token: &mut [u8]) {
    let sign = usize::from(token.first() == Some(&b'-'));
    let digits = &mut token[sign..];
    let Some(spelling) = std::str::from_utf8(digits).ok().and_then(real_digits) else {
        return;
    };
    let Some(zeros) = digits.len().checked_sub(spelling.len()) else {
        return;
    };
    let (fill, real) = digits.split_at_mut(zeros);
    fill.fill(b'0');
    real.copy_from_slice(spelling.as_bytes());
}

/// The copy of `text` facet reads in its place: each of the wide integer tokens respelt as a
/// real token of its length, and each whole comment blanked (a space for each of its bytes).
/// `None` when the text has neither: it is read as it is.
///
/// The comments are blanked because facet-json (rc.7) forgets that a text has comments:
/// `JsonParser::restore` rebuilds its scanner without the flag, and the deserialiser restores
/// whenever it reads an untagged enum, so a comment the scanner meets after that (beyond the
/// 512 events the deserialiser had already read) is "unexpected character '/'". A
/// copy without comments has none to forget. A `/*` that never ends inside the root never
/// reaches the copy: the scan reports it ([`UnsafeText::UnterminatedComment`]).
fn facet_copy(text: &str, marks: &Marks) -> Option<String> {
    if marks.wide.is_empty() && marks.comments.is_empty() {
        return None;
    }
    let mut copy = text.as_bytes().to_vec();
    for range in &marks.wide {
        if let Some(token) = copy.get_mut(range.clone()) {
            respell(token);
        }
    }
    for range in &marks.comments {
        if let Some(comment) = copy.get_mut(range.clone()) {
            comment.fill(b' ');
        }
    }
    // Digits and whole comments were replaced by ASCII of the same length.
    String::from_utf8(copy).ok()
}

/// What facet may read of a text [`scan`] passed.
struct Checked<'text> {
    /// The text up to the end of its root value.
    root: &'text str,
    /// The copy of `root` facet reads in its place ([`facet_copy`]), when it needs one.
    copy: Option<String>,
    /// How deep the text nests: what a typed read is given its stack by (`stack`).
    levels: usize,
}

/// The error for what the scan refused in `bytes`, naming the bracket it stopped on.
fn unsafe_error(found: UnsafeText, bytes: &[u8]) -> DeserializeError {
    let mut error = DeserializeError::from(found);
    if let (UnsafeText::TooDeep { offset }, DeserializeErrorKind::UnexpectedChar { ch, .. }) =
        (found, &mut error.kind)
        && bytes.get(offset) == Some(&b'{')
    {
        *ch = '{';
    }
    error
}

/// The error of a text the scan refused: the file's first syntax fault. The scan looks at
/// strings and brackets only, so a fault of the grammar may stand before what it refused (a
/// missing colon, a stray byte, a bracket closing another kind). The text before the refused
/// token is text the scan passed: the parser reads it, and what it reports there, other than
/// the end of the text, is the earlier fault, in its own words.
fn refusal(bytes: &[u8], comments: bool, refused: &Refused, marks: &Marks) -> DeserializeError {
    // The text before the token may be UTF-8 where the whole is not.
    let before = bytes
        .get(..refused.token)
        .and_then(|before| std::str::from_utf8(before).ok());
    if let Some(before) = before {
        // The token itself may be the fault (a string where a comma belongs): a token of
        // its kind with nothing in it stands in for it.
        // A comment is no token: nothing stands in for it.
        let stand_in = match (refused.found, bytes.get(refused.token)) {
            (UnsafeText::UnterminatedComment { .. }, _) => "",
            (_, Some(b'{')) => "{",
            (_, Some(b'[')) => "[",
            _ => "\"\"",
        };
        let probe = format!("{before}{stand_in}");
        let copy = facet_copy(&probe, &marks.within(before.len()));
        if let Err(mut earlier) = read_owned::<Value>(&probe, copy.as_deref(), comments, 0)
            && !matches!(earlier.kind, DeserializeErrorKind::UnexpectedEof { .. })
            && let Some(span) = earlier.span
            && (span.offset as usize) <= refused.token
        {
            if span.offset as usize == refused.token {
                // On the token: its length is the one it has in the text.
                let len = token_len(bytes, refused.token);
                earlier.span = Some(facet::Span::new(refused.token, len));
                // And its text is the author's, not the stand-in's.
                if let DeserializeErrorKind::UnexpectedToken { got, .. } = &mut earlier.kind
                    && bytes.get(refused.token) == Some(&b'"')
                {
                    let written = &bytes[refused.token..refused.token + len];
                    *got = Cow::Owned(format!("String({})", String::from_utf8_lossy(written)));
                }
            }
            return earlier;
        }
    }
    unsafe_error(refused.found, bytes)
}

/// The length of the token at `at`: a string to its closing quote (whatever follows its
/// backslashes) or to the end of the text; one byte for a bracket.
fn token_len(bytes: &[u8], at: usize) -> usize {
    if bytes.get(at) != Some(&b'"') {
        return 1;
    }
    let mut i = at + 1;
    loop {
        match bytes.get(i) {
            None => return bytes.len() - at,
            Some(b'"') => return i + 1 - at,
            Some(b'\\') => i += 2,
            Some(_) => i += 1,
        }
    }
}

/// Reads `root`, a text the scan passed that nests `levels` deep, into an owned type: over
/// `copy`, the copy facet reads in its place, when it has one.
fn read_owned<T: Facet<'static>>(
    root: &str,
    copy: Option<&str>,
    comments: bool,
    levels: usize,
) -> Result<T, DeserializeError> {
    if is_untyped::<T>() {
        let wip = Partial::alloc_owned::<T>()?;
        return untyped(copy.unwrap_or(root), comments, wip)?
            .materialize::<T>()
            .map_err(DeserializeError::from);
    }
    with_stack_for(levels, || {
        let mut parser = Rehomed::new(copy.unwrap_or(root), root, comments).owning();
        FormatDeserializer::new_owned(&mut parser).deserialize_root()
    })
}

fn checked(text: &str, comments: bool) -> Result<Checked<'_>, DeserializeError> {
    let mut marks = Marks::default();
    let scanned = scan_tokens(text.as_bytes(), comments, &mut marks)
        .map_err(|refused| refusal(text.as_bytes(), comments, &refused, &marks))?;
    // The root ends after an ASCII byte, so the cut is on a character boundary.
    let root = scanned
        .root_end
        .and_then(|end| text.get(..end))
        .unwrap_or(text);
    Ok(Checked {
        root,
        copy: facet_copy(root, &marks.within(root.len())),
        levels: marks.levels,
    })
}

/// `facet_json::from_str` behind [`scan`]: plain JSON (no comments) into an owned type.
///
/// Needs no stack of the caller that grows with the text: a text nested to `MAX_JSON_DEPTH`
/// is read, and a deeper one refused with an error, on a thread with a megabyte of stack,
/// whatever the target (a typed read that may need more runs on a stack made for it, on
/// this thread). `MAX_JSON_DEPTH` names what is outside that rule: WebAssembly, a target
/// without a stack switch, a host that cannot map the stack (a panic).
///
/// # Errors
///
/// Returns `DeserializeError` for text the scan refuses or that facet-json cannot read as `T`.
pub fn from_str<T: Facet<'static>>(text: &str) -> Result<T, DeserializeError> {
    let checked = checked(text, false)?;
    read_owned(checked.root, checked.copy.as_deref(), false, checked.levels)
}

/// `facet_json::from_str_jsonc` behind [`scan`]: JSON with comments into an owned type.
///
/// Needs no stack of the caller that grows with the text: a text nested to `MAX_JSON_DEPTH`
/// is read, and a deeper one refused with an error, on a thread with a megabyte of stack,
/// whatever the target (a typed read that may need more runs on a stack made for it, on
/// this thread). `MAX_JSON_DEPTH` names what is outside that rule: WebAssembly, a target
/// without a stack switch, a host that cannot map the stack (a panic).
///
/// # Errors
///
/// Returns `DeserializeError` for text the scan refuses or that facet-json cannot read as `T`.
pub fn from_str_jsonc<T: Facet<'static>>(text: &str) -> Result<T, DeserializeError> {
    let checked = checked(text, true)?;
    read_owned(checked.root, checked.copy.as_deref(), true, checked.levels)
}

/// `facet_json::from_str_borrowed_jsonc` behind [`scan`]: JSON with comments into a type that
/// may borrow from `text`; the way every typed pack file is read.
///
/// Needs no stack of the caller that grows with the text: a text nested to `MAX_JSON_DEPTH`
/// is read, and a deeper one refused with an error, on a thread with a megabyte of stack,
/// whatever the target (a typed read that may need more runs on a stack made for it, on
/// this thread). `MAX_JSON_DEPTH` names what is outside that rule: WebAssembly, a target
/// without a stack switch, a host that cannot map the stack (a panic).
///
/// # Errors
///
/// Returns `DeserializeError` for text the scan refuses or that facet-json cannot read as `T`.
pub fn from_str_borrowed_jsonc<'input, 'facet, T>(text: &'input str) -> Result<T, DeserializeError>
where
    T: Facet<'facet>,
    'input: 'facet,
{
    let checked = checked(text, true)?;
    if is_untyped::<T>() {
        // A value borrows nothing: it is read as the owned reads read it.
        let wip = Partial::alloc::<T>()?;
        return untyped(checked.copy.as_deref().unwrap_or(checked.root), true, wip)?
            .materialize::<T>()
            .map_err(DeserializeError::from);
    }
    with_stack_for(checked.levels, || match &checked.copy {
        None => facet_json::from_str_borrowed_jsonc(checked.root),
        Some(copy) => {
            let mut parser = Rehomed::new(copy, checked.root, true);
            FormatDeserializer::new(&mut parser).deserialize_root()
        }
    })
}

/// [`from_str_borrowed_jsonc`] reading `text` as if the object members `absent` named were not
/// written: each of them whose value is `null` is blanked to white space in the copy facet
/// reads, with the comma that joins it to a neighbour, so offsets stay the file's and what
/// the read borrows is the file's text (rule N1, `json::null`). A member of `absent` whose
/// value is not `null` is read as written.
pub(super) fn from_str_borrowed_jsonc_without<'input, 'facet, T>(
    text: &'input str,
    absent: &[super::path::MemberPath],
) -> Result<T, DeserializeError>
where
    T: Facet<'facet>,
    'input: 'facet,
{
    let checked = checked(text, true)?;
    let read = checked.copy.as_deref().unwrap_or(checked.root);
    let blanks = null_member_blanks(read, absent);
    if blanks.is_empty() {
        return from_str_borrowed_jsonc(text);
    }
    let mut copy = read.as_bytes().to_vec();
    for range in blanks {
        if let Some(part) = copy.get_mut(range) {
            part.fill(b' ');
        }
    }
    // Whole tokens and the white space between them became spaces of the same length.
    let copy = String::from_utf8(copy).map_err(|_| ended("a text", facet::Span::default()))?;
    with_stack_for(checked.levels, || {
        let mut parser = Rehomed::new(&copy, checked.root, true);
        FormatDeserializer::new(&mut parser).deserialize_root()
    })
}

/// The byte ranges of `text` (no comments left: the copy facet reads) to blank so that each
/// member `absent` names whose value is `null` is gone: from its key to the next member's
/// key, or, for the last member of its object, from the end of the last member kept before
/// it (its comma included) to the end of the `null`.
fn null_member_blanks(text: &str, absent: &[super::path::MemberPath]) -> Vec<Range<usize>> {
    use super::path::MemberPath;
    /// A container the scan is in.
    enum Frame {
        Object {
            /// Where the current member's key starts.
            key_start: usize,
            /// Where the value of the last member kept ends.
            kept_end: Option<usize>,
            /// A `null` member to blank: its key's start and the `null`'s end.
            pending: Option<(usize, usize)>,
        },
        Array {
            index: usize,
        },
    }
    /// A value of the innermost container ended at `end`.
    fn value_done(frames: &mut [Frame], path: &mut MemberPath, end: usize) {
        match frames.last_mut() {
            Some(Frame::Object { kept_end, .. }) => {
                *kept_end = Some(end);
                path.pop();
            }
            Some(Frame::Array { index }) => {
                *index += 1;
                path.pop();
            }
            None => {}
        }
    }
    /// A value of the innermost container starts: an element gets its index.
    fn value_starts(frames: &[Frame], path: &mut MemberPath) {
        if let Some(Frame::Array { index }) = frames.last() {
            path.push_index(*index);
        }
    }
    let mut parser = JsonParser::<true>::new_jsonc(text.as_bytes());
    let mut frames: Vec<Frame> = Vec::new();
    let mut path = MemberPath::default();
    let mut out = Vec::new();
    while let Ok(Some(event)) = parser.next_event() {
        let start = event.span.offset as usize;
        let end = start + event.span.len as usize;
        match event.kind {
            ParseEventKind::FieldKey(key) => {
                if let Some(Frame::Object {
                    key_start, pending, ..
                }) = frames.last_mut()
                {
                    if let Some((from, _)) = pending.take() {
                        out.push(from..start);
                    }
                    *key_start = start;
                    path.push_key(key.name().map_or("@", |name| name.as_ref()));
                }
            }
            ParseEventKind::StructStart(_) => {
                value_starts(&frames, &mut path);
                frames.push(Frame::Object {
                    key_start: start,
                    kept_end: None,
                    pending: None,
                });
            }
            ParseEventKind::SequenceStart(_) => {
                value_starts(&frames, &mut path);
                frames.push(Frame::Array { index: 0 });
            }
            ParseEventKind::StructEnd => {
                if let Some(Frame::Object {
                    kept_end,
                    pending: Some((from, to)),
                    ..
                }) = frames.pop()
                {
                    out.push(kept_end.unwrap_or(from)..to);
                }
                value_done(&mut frames, &mut path, end);
            }
            ParseEventKind::SequenceEnd => {
                frames.pop();
                value_done(&mut frames, &mut path, end);
            }
            ParseEventKind::Scalar(scalar) => {
                value_starts(&frames, &mut path);
                if let (
                    ScalarValue::Null,
                    Some(Frame::Object {
                        key_start, pending, ..
                    }),
                ) = (&scalar, frames.last_mut())
                    && absent.contains(&path)
                {
                    *pending = Some((*key_start, end));
                    path.pop();
                    continue;
                }
                value_done(&mut frames, &mut path, end);
            }
            _ => {}
        }
    }
    out
}

/// The offset of the first syntax fault of `text` (JSON with comments), where a reader of
/// the door reports it; `None` when the text is read to the end of its root value. The
/// scan's refusals and the parser's faults, and nothing a target adds (a type, a field
/// written twice): the text is read as its tokens, no value is built.
pub(super) fn first_fault(text: &str) -> Option<usize> {
    let checked = match checked(text, true) {
        Ok(checked) => checked,
        Err(refused) => return Some(refused.span.map_or(0, |span| span.offset as usize)),
    };
    let read = checked.copy.as_deref().unwrap_or(checked.root);
    let mut parser = JsonParser::<true>::new_jsonc(read.as_bytes());
    let mut open = 0usize;
    loop {
        let event = match parser.next_event() {
            Ok(Some(event)) => event,
            // A text that ends before its root value does.
            Ok(None) => return Some(text.len()),
            Err(fault) => return Some(fault.span.offset as usize),
        };
        match event.kind {
            ParseEventKind::StructStart(_) | ParseEventKind::SequenceStart(_) => open += 1,
            ParseEventKind::StructEnd | ParseEventKind::SequenceEnd => {
                open = open.saturating_sub(1);
                if open == 0 {
                    return None;
                }
            }
            ParseEventKind::Scalar(_) if open == 0 => return None,
            _ => {}
        }
    }
}

/// Whether `T` is the untyped [`Value`], which the door reads itself ([`untyped`]).
fn is_untyped<'facet, T: Facet<'facet>>() -> bool {
    T::SHAPE.is_type::<Value>()
}

/// The error of a step of the builder that failed: facet's own form of it
/// (`From<ReflectError>`), at the last event read.
fn reflect(at: facet::Span) -> impl Fn(ReflectError) -> DeserializeError {
    move |error| DeserializeError {
        span: Some(at),
        path: Some(error.path),
        kind: DeserializeErrorKind::Reflect {
            kind: error.kind,
            context: "",
        },
    }
}

/// The text ended where `expected` belongs: facet's error for it, at the last event read.
fn ended(expected: &'static str, at: facet::Span) -> DeserializeError {
    DeserializeErrorKind::UnexpectedEof { expected }.with_span(at)
}

/// Reads the text the scan passed into `wip`, the builder of an untyped [`Value`], and
/// builds it: what facet's `deserialize_dynamic_value` does (0.50.0-rc.7), event for event
/// and step for step of the builder, with the open arrays and objects in a list instead of
/// one call of itself for each.
///
/// facet's function takes some fourteen kilobytes of stack a level in a debug build: a text
/// nested 74 deep overflowed a megabyte of stack, well within `MAX_JSON_DEPTH`. This takes
/// the same stack however deep the text is. A typed target is still read by facet's own
/// recursion, a level of it for each level of the type, on a stack sized for the text.
fn untyped<'facet, const BORROW: bool>(
    text: &str,
    comments: bool,
    mut wip: Partial<'facet, BORROW>,
) -> Result<facet::HeapValue<'facet, BORROW>, DeserializeError> {
    let mut parser = if comments {
        JsonParser::<true>::new_jsonc(text.as_bytes())
    } else {
        JsonParser::<true>::new(text.as_bytes())
    };
    // Whether each open container is an object, the innermost last.
    let mut open: Vec<bool> = Vec::new();
    // The span of the last event read: where facet places a text that ends early.
    let mut at = facet::Span::new(0, 0);
    loop {
        // The event a value starts with: the next one, or, inside a container, the one
        // after its end was looked for (and, in an object, after the member's key).
        let expected = match open.last() {
            None => "value for dynamic value",
            Some(false) => "value or end",
            Some(true) => "field key or end",
        };
        let mut event = parser.next_event()?.ok_or_else(|| ended(expected, at))?;
        at = event.span;
        match (open.last().copied(), &event.kind) {
            (Some(false), ParseEventKind::SequenceEnd)
            | (Some(true), ParseEventKind::StructEnd) => {
                open.pop();
                if open.is_empty() {
                    break;
                }
                // The container was an element or a member's value: that is whole now.
                wip = wip.end().map_err(reflect(at))?;
                continue;
            }
            (Some(false), _) => wip = wip.begin_list_item().map_err(reflect(at))?,
            (Some(true), ParseEventKind::FieldKey(key)) => {
                // facet names a key without a name `@`; JSON has no such key.
                let key = key.name().map_or("@", |name| name.as_ref());
                wip = wip.begin_object_entry(key).map_err(reflect(at))?;
                event = parser
                    .next_event()?
                    .ok_or_else(|| ended("value for dynamic value", at))?;
                at = event.span;
            }
            (Some(true), _) => {
                return Err(DeserializeError {
                    span: Some(at),
                    path: Some(wip.path()),
                    kind: DeserializeErrorKind::UnexpectedToken {
                        expected: "field key",
                        got: event.kind_name().into(),
                    },
                });
            }
            (None, _) => {}
        }
        match event.kind {
            ParseEventKind::Scalar(scalar) => {
                wip = match scalar {
                    ScalarValue::Null | ScalarValue::Unit => wip.set_default(),
                    ScalarValue::Bool(value) => wip.set(value),
                    ScalarValue::Char(value) => wip.set(value),
                    ScalarValue::I64(value) => wip.set(value),
                    ScalarValue::U64(value) => wip.set(value),
                    ScalarValue::I128(value) => wip.set(value),
                    ScalarValue::U128(value) => wip.set(value),
                    ScalarValue::F64(value) => wip.set(value),
                    ScalarValue::Str(value) => wip.set(value.into_owned()),
                    ScalarValue::Bytes(value) => wip.set(value.into_owned()),
                    // facet-json (rc.7) has no other scalar.
                    _ => return Err(unknown_event(at).into()),
                }
                .map_err(reflect(at))?;
                if open.is_empty() {
                    break;
                }
                wip = wip.end().map_err(reflect(at))?;
            }
            ParseEventKind::SequenceStart(_) => {
                wip = wip.init_list().map_err(reflect(at))?;
                open.push(false);
            }
            ParseEventKind::StructStart(_) => {
                wip = wip.init_map().map_err(reflect(at))?;
                open.push(true);
            }
            _ => {
                return Err(DeserializeError {
                    span: Some(at),
                    path: Some(wip.path()),
                    kind: DeserializeErrorKind::UnexpectedToken {
                        expected: "scalar, sequence, or struct",
                        got: event.kind_name().into(),
                    },
                });
            }
        }
    }
    wip.build().map_err(reflect(at))
}

/// facet-json's parser over the copy of a text ([`facet_copy`]), handing the deserialiser what
/// it borrows (a key or a string without escapes, a raw slice) from the text itself, at the
/// same offsets: the copy lives only as long as the read, the text as long as the value.
/// Numbers and comments are the only things the copy spells otherwise; neither is ever
/// borrowed as a string, and a raw slice is cut from the text.
///
/// Each entry point reads a respelt copy as facet-json's function of its name reads a text
/// (the same parser, the same deserialiser, owned or borrowing), with this in between. The
/// owned reads go through it too: facet's owned deserialiser keeps a raw slice as a copy of
/// the input's bytes, which must be the author's, and lends a `Cow<'static, str>` member the
/// input itself (rc.7; no type of the crate family has one, `tests/json_door.rs`), which
/// must not be a copy that dies with the read.
struct Rehomed<'copy, 'input> {
    parser: JsonParser<'copy, true>,
    copy: &'copy str,
    text: &'input str,
    /// Whether every string is handed over as a copy of its own: the owned reads.
    owned: bool,
}

impl<'copy, 'input> Rehomed<'copy, 'input> {
    /// facet-json's parser of `copy`, the respelt copy of `text`, as its entry points make
    /// it for a `&str`: with comments or without.
    fn new(copy: &'copy str, text: &'input str, comments: bool) -> Self {
        // A respelt token has the length of the token it stands for: an offset in the copy
        // is the same offset in the text.
        debug_assert_eq!(copy.len(), text.len(), "the respelt copy moved an offset");
        let parser = if comments {
            JsonParser::<true>::new_jsonc(copy.as_bytes())
        } else {
            JsonParser::<true>::new(copy.as_bytes())
        };
        Self {
            parser,
            copy,
            text,
            owned: false,
        }
    }

    /// The same parser for an owned read: it lends nothing. facet's owned deserialiser (rc.7,
    /// `deserialize_pointer`) hands a `Cow<'_, str>` member the borrowed string of the event
    /// as it is, so a `Foo<'static>` read from a text that is not `'static` would point into
    /// it; an event that owns its string cannot be lent.
    fn owning(mut self) -> Self {
        self.owned = true;
        self
    }

    /// The part of the text that lies where `part` lies in the copy; `None` when `part` is
    /// no part of the copy.
    fn in_text(&self, part: &'copy str) -> Option<&'input str> {
        let start = (part.as_ptr() as usize).checked_sub(self.copy.as_ptr() as usize)?;
        let end = start.checked_add(part.len())?;
        if end > self.copy.len() {
            return None;
        }
        self.text.get(start..end)
    }

    fn rehome_str(&self, part: Cow<'copy, str>) -> Cow<'input, str> {
        match part {
            Cow::Borrowed(part) if self.owned => Cow::Owned(part.to_owned()),
            Cow::Borrowed(part) => self
                .in_text(part)
                .map_or_else(|| Cow::Owned(part.to_owned()), Cow::Borrowed),
            Cow::Owned(part) => Cow::Owned(part),
        }
    }

    /// The event with its borrowed text taken from the text; its span stays.
    fn rehome(&self, event: ParseEvent<'copy>) -> Result<ParseEvent<'input>, ParseError> {
        let kind = match event.kind {
            ParseEventKind::StructStart(kind) => ParseEventKind::StructStart(kind),
            ParseEventKind::StructEnd => ParseEventKind::StructEnd,
            ParseEventKind::SequenceStart(kind) => ParseEventKind::SequenceStart(kind),
            ParseEventKind::SequenceEnd => ParseEventKind::SequenceEnd,
            ParseEventKind::FieldKey(FieldKey::Name(name)) => {
                ParseEventKind::FieldKey(FieldKey::Name(self.rehome_str(name)))
            }
            ParseEventKind::Scalar(scalar) => ParseEventKind::Scalar(match scalar {
                ScalarValue::Str(text) => ScalarValue::Str(self.rehome_str(text)),
                ScalarValue::Bytes(bytes) => ScalarValue::Bytes(Cow::Owned(bytes.into_owned())),
                ScalarValue::Unit => ScalarValue::Unit,
                ScalarValue::Null => ScalarValue::Null,
                ScalarValue::Bool(value) => ScalarValue::Bool(value),
                ScalarValue::Char(value) => ScalarValue::Char(value),
                ScalarValue::I64(value) => ScalarValue::I64(value),
                ScalarValue::U64(value) => ScalarValue::U64(value),
                ScalarValue::I128(value) => ScalarValue::I128(value),
                ScalarValue::U128(value) => ScalarValue::U128(value),
                ScalarValue::F64(value) => ScalarValue::F64(value),
                _ => return Err(unknown_event(event.span)),
            }),
            // facet-json (rc.7) has no other event: no ordered field, option marker, variant
            // tag or key with metadata.
            _ => return Err(unknown_event(event.span)),
        };
        Ok(ParseEvent::new(kind, event.span))
    }
}

/// The error for an event facet-json does not have (a facet bump would add it here).
fn unknown_event(span: facet::Span) -> ParseError {
    ParseError::new(
        span,
        DeserializeErrorKind::Bug {
            error: Cow::Borrowed("an event facet-json does not emit"),
            context: "reading a text with an integer beyond 64 bits",
        },
    )
}

/// Every method facet-json's parser has of its own goes to it; the rest are the trait's
/// defaults for both.
impl<'input> FormatParser<'input> for Rehomed<'_, 'input> {
    fn next_event(&mut self) -> Result<Option<ParseEvent<'input>>, ParseError> {
        let event = self.parser.next_event()?;
        event.map(|event| self.rehome(event)).transpose()
    }

    fn next_events(
        &mut self,
        buf: &mut VecDeque<ParseEvent<'input>>,
        limit: usize,
    ) -> Result<usize, ParseError> {
        // The parser leaves the events it read before a fault in the buffer.
        let mut events = VecDeque::new();
        let read = self.parser.next_events(&mut events, limit);
        for event in events {
            buf.push_back(self.rehome(event)?);
        }
        read
    }

    fn peek_event(&mut self) -> Result<Option<ParseEvent<'input>>, ParseError> {
        let event = self.parser.peek_event()?;
        event.map(|event| self.rehome(event)).transpose()
    }

    fn skip_value(&mut self) -> Result<(), ParseError> {
        self.parser.skip_value()
    }

    fn save(&mut self) -> SavePoint {
        self.parser.save()
    }

    fn restore(&mut self, save_point: SavePoint) {
        self.parser.restore(save_point);
    }

    fn capture_raw(&mut self) -> Result<Option<&'input str>, ParseError> {
        let raw = self.parser.capture_raw()?;
        Ok(raw.and_then(|raw| self.in_text(raw)))
    }

    /// The text, not the copy: the deserialiser cuts a raw slice out of this by the spans of
    /// the value, so a raw slice is the author's text, wide integers as written.
    fn input(&self) -> Option<&'input [u8]> {
        Some(self.text.as_bytes())
    }

    fn raw_capture_shape(&self) -> Option<&'static facet::Shape> {
        self.parser.raw_capture_shape()
    }

    fn current_span(&self) -> Option<facet::Span> {
        self.parser.current_span()
    }

    fn format_namespace(&self) -> Option<&'static str> {
        self.parser.format_namespace()
    }
}

/// JSON with comments from bytes that may not be UTF-8, into an owned type. facet's own
/// byte entry points are not used: their checked string decoder validates the rest of a string
/// for each of its characters (quadratic in a long string). UTF-8 bytes are read as text;
/// other bytes are read as text when only their comments are not UTF-8 (facet accepts those), and
/// are a syntax error otherwise.
///
/// Needs no stack of the caller that grows with the text: a text nested to `MAX_JSON_DEPTH`
/// is read, and a deeper one refused with an error, on a thread with a megabyte of stack,
/// whatever the target (a typed read that may need more runs on a stack made for it, on
/// this thread). `MAX_JSON_DEPTH` names what is outside that rule: WebAssembly, a target
/// without a stack switch, a host that cannot map the stack (a panic).
///
/// # Errors
///
/// Returns `DeserializeError` for bytes the scan refuses, a syntax error, or bytes that are not UTF-8 outside comments.
pub fn from_slice_jsonc<T: Facet<'static>>(bytes: &[u8]) -> Result<T, DeserializeError> {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return from_str_jsonc(text);
    }
    // What follows the root value is not read, UTF-8 or not.
    let mut marks = Marks::default();
    let end = scan_tokens(bytes, true, &mut marks)
        .map_err(|refused| refusal(bytes, true, &refused, &marks))?
        .root_end;
    let root = end.and_then(|end| bytes.get(..end)).unwrap_or(bytes);
    match std::str::from_utf8(root) {
        Ok(text) => from_str_jsonc(text),
        Err(_) => from_str_jsonc(&text_outside_comments(root)?),
    }
}

/// `bytes` with every byte of a comment that is not ASCII replaced by `?` (offsets stay), as
/// text; an error where a string, or a byte outside strings and comments, is not UTF-8.
fn text_outside_comments(bytes: &[u8]) -> Result<String, DeserializeError> {
    let error = |at: usize, len: usize, kind| DeserializeError {
        span: Some(facet::Span::new(at, len)),
        path: None,
        kind,
    };
    let mut copy = bytes.to_vec();
    let mut at = 0;
    loop {
        let after = skip_trivia(bytes, at, true);
        for byte in &mut copy[at..after] {
            if !byte.is_ascii() {
                *byte = b'?';
            }
        }
        at = after;
        match bytes.get(at) {
            None => break,
            Some(b'"') => {
                let (end, _) = string_end(bytes, at)?;
                if std::str::from_utf8(&bytes[at..end]).is_err() {
                    // facet's error for a string that is not UTF-8, on the string.
                    let kind = DeserializeErrorKind::InvalidUtf8 {
                        context: [0; 16],
                        context_len: 0,
                    };
                    return Err(error(at, end - at, kind));
                }
                at = end;
            }
            Some(_) => at += 1,
        }
    }
    String::from_utf8(copy).map_err(|failure| {
        // A byte that starts no token: facet's error for it.
        let at = failure.utf8_error().valid_up_to();
        let kind = DeserializeErrorKind::UnexpectedChar {
            ch: bytes.get(at).copied().map_or('?', char::from),
            expected: "valid JSON token",
        };
        error(at, 1, kind)
    })
}

/// Where the content after the root value of `text` lies (JSON with comments): from its first
/// byte to the end of the text, trailing white space left out. Vanilla's reader stops at the
/// end of the root value and never looks at it, so the
/// file loads; `None` when nothing follows, and when the text is not one facet is given.
#[must_use]
pub fn trailing_content(text: &str) -> Option<Span> {
    let start = scan(text.as_bytes(), true).ok()?.trailing?;
    let len = text.get(start..)?.trim_end().len();
    Some(Span {
        offset: u32::try_from(start).ok()?,
        len: u32::try_from(len).ok()?,
    })
}

/// What `RapidJSON` refuses of `document` that a `JsonCpp` reader takes, for the Minecraft
/// Document reader: it parses iteratively, with comments and trailing commas allowed and
/// without stopping at the end of the root value, and on a parse error logs "Failed to parse
/// the Minecraft Document JSON: {}", an error in both of the game's log masks; the document
/// is not loaded. The two errors: a number no double holds (`RapidJSON`'s "number too big"
/// error) and content after the root (its "root not singular" error). The first in the text is
/// told, as `RapidJSON` stops there: a rejecting `MalformedInput` error with the game's text, on
/// the number or the content, at file offsets. `None` when the text has neither, and when facet
/// does not read it (its syntax error is the reader's to tell).
#[must_use]
pub fn document_json_refusal(document: JsonSlice<'_>) -> Option<Diagnostic> {
    let number = super::first_beyond_double(document.text);
    let trailing = trailing_content(document.text);
    let number_first = match (&number, trailing) {
        (Some(number), Some(tail)) => number.start < tail.offset as usize,
        (number, _) => number.is_some(),
    };
    let (message, start, len) = match (number, trailing) {
        (Some(number), _) if number_first => (DOCUMENT_NUMBER_TOO_BIG, number.start, number.len()),
        (_, Some(tail)) => (
            DOCUMENT_ROOT_NOT_SINGULAR,
            tail.offset as usize,
            tail.len as usize,
        ),
        _ => return None,
    };
    let offset = document.offset.saturating_add(start);
    Some(Diagnostic::new(
        DiagnosticCode::MalformedInput {
            message: message.to_owned(),
            text: Some(VanillaText::DocumentJson {
                message: message.to_owned(),
            }),
        },
        Severity::Error,
        Phase::Parse,
        Location::new(
            None,
            None,
            JsonPath::new(),
            Some(Span {
                offset: u32::try_from(offset).unwrap_or(u32::MAX),
                len: u32::try_from(len).unwrap_or(u32::MAX),
            }),
        ),
    ))
}

/// `RapidJSON`'s text for a number too big for a double.
pub const DOCUMENT_NUMBER_TOO_BIG: &str = "Number too big to be stored in double.";
/// `RapidJSON`'s text for content after the document's root value.
pub const DOCUMENT_ROOT_NOT_SINGULAR: &str =
    "The document root must not be followed by other values.";

/// The message of [`trailing_content_finding`].
pub const TRAILING_CONTENT: &str = "content after the JSON value is ignored";

/// The finding for content after the root value of `document`: an information of the
/// crate's own on that content, at its file offset: the `JsonCpp` reader reads the root value,
/// skips the comments after it and returns without looking further, so it uses the file in
/// silence. The Minecraft Document reader refuses such a text instead (`RapidJSON`'s "root not
/// singular" error; the versioned load's own check). One per file: the
/// reader of the file's root raises it, not the readers of its slices.
#[must_use]
pub fn trailing_content_finding(document: JsonSlice<'_>) -> Option<Diagnostic> {
    let span = trailing_content(document.text)?;
    let offset = u32::try_from(document.offset).unwrap_or(u32::MAX);
    Some(Diagnostic::new(
        DiagnosticCode::MalformedInput {
            message: TRAILING_CONTENT.to_owned(),
            text: None,
        },
        Severity::Info,
        Phase::Parse,
        Location::new(
            None,
            None,
            JsonPath::new(),
            Some(Span {
                offset: offset.saturating_add(span.offset),
                len: span.len,
            }),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use facet_json::RawJson;
    use indexmap::IndexMap;

    use super::*;
    use crate::json::Value;

    /// xorshift64*: a fixed sequence per seed.
    struct Rng(u64);

    impl Rng {
        fn below(&mut self, n: usize) -> usize {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            usize::try_from(self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) % n as u64).unwrap()
        }
    }

    #[test]
    fn a_wide_integer_is_digits_alone_that_fit_neither_64_bit_type() {
        let zeros = "0".repeat(40);
        for (token, wide) in [
            ("9223372036854775807", false),
            ("9223372036854775808", false),
            ("18446744073709551615", false),
            ("18446744073709551616", true),
            ("-9223372036854775808", false),
            ("-9223372036854775809", true),
            ("-18446744073709551615", true),
            ("999999999999999999", false),
            ("-999999999999999999", false),
            ("1000000000000000000", false),
            ("99999999999999999999", true),
            ("18446744073709551616.0", false),
            ("18446744073709551616e0", false),
            ("18446744073709551616E+1", false),
            ("1e999", false),
            ("-", false),
            ("", false),
            ("0", false),
            ("-0", false),
        ] {
            assert_eq!(is_wide_integer(token.as_bytes()), wide, "`{token}`");
        }
        assert!(!is_wide_integer(format!("{zeros}7").as_bytes()));
        assert!(!is_wide_integer(format!("-{zeros}7").as_bytes()));
        assert!(!is_wide_integer(
            format!("{zeros}18446744073709551615").as_bytes()
        ));
        assert!(is_wide_integer(
            format!("{zeros}18446744073709551616").as_bytes()
        ));
        assert!(is_wide_integer(
            format!("-{zeros}9223372036854775809").as_bytes()
        ));
    }

    /// `token` as [`respell`] leaves it.
    fn real_spelling(token: &str) -> String {
        let mut bytes = token.as_bytes().to_vec();
        respell(&mut bytes);
        String::from_utf8(bytes).unwrap()
    }

    /// The spelling of `token` checked against the token: its length, its sign, a real token
    /// of digits and one exponent, and the double the digits spell.
    fn check_spelling(token: &str) {
        let spelling = real_spelling(token);
        assert_ne!(spelling, token, "`{token}` is respelt");
        assert_eq!(spelling.len(), token.len(), "`{token}` as `{spelling}`");
        assert_eq!(
            spelling.starts_with('-'),
            token.starts_with('-'),
            "`{spelling}`"
        );
        let digits = spelling.strip_prefix('-').unwrap_or(&spelling);
        let (mantissa, exponent) = digits.split_once('e').expect("an exponent");
        assert!(
            !mantissa.is_empty() && mantissa.bytes().all(|b| b.is_ascii_digit()),
            "`{spelling}`"
        );
        assert!(
            !exponent.is_empty() && exponent.bytes().all(|b| b.is_ascii_digit()),
            "`{spelling}`"
        );
        assert!(!is_wide_integer(spelling.as_bytes()), "`{spelling}`");
        let spelt: f64 = token.parse().unwrap();
        let read: f64 = spelling.parse().unwrap();
        assert_eq!(read.to_bits(), spelt.to_bits(), "`{token}` as `{spelling}`");
    }

    #[test]
    fn the_real_spelling_has_the_length_of_the_token_and_reads_to_its_double() {
        assert_eq!(
            real_digits("18446744073709551616").unwrap(),
            "18446744073709552e3"
        );
        assert_eq!(
            real_spelling("18446744073709551616"),
            "018446744073709552e3"
        );
        assert_eq!(
            real_spelling("-9223372036854775809"),
            "-09223372036854776e3"
        );
        assert_eq!(
            real_spelling("99999999999999999999"),
            "00000000000000001e20"
        );
        let beyond = "9".repeat(310);
        assert_eq!(real_spelling(&beyond), format!("{}1e999", "0".repeat(305)));
        assert_eq!(
            real_spelling(&format!("-{beyond}")),
            format!("-{}1e999", "0".repeat(305))
        );
        // What is no number, or has no room for the spelling, is left as it is.
        for token in ["-", "", "12", "-123", "x18446744073709551616"] {
            assert_eq!(real_spelling(token), token);
        }
        // The shortest wide integers, whose doubles have the most digits to keep.
        for token in [
            "-9223372036854777856",
            "-9999999999999999999",
            "-9999999999999999488",
            "18446744073709551616",
            "18446744073709553664",
            "18446744073709553665",
            "-9223372036854775809",
            "-9223372036854777857",
            "179769313486231570000000000000000000000",
        ] {
            assert!(is_wide_integer(token.as_bytes()), "`{token}`");
            check_spelling(token);
        }
        // The largest finite double, the tie above it, and the first integer past it.
        let max = format!("{:.0}", f64::MAX);
        assert_eq!(max.len(), 309);
        check_spelling(&max);
        let mut rng = Rng(0x00B1_6157_5EED_0004);
        let (mut longest, mut checked) = (0, 0);
        for case in 0..40_000 {
            let length = match case % 4 {
                0 => 19 + rng.below(2),
                1 => 20 + rng.below(20),
                2 => 39 + rng.below(280),
                _ => 300 + rng.below(40),
            };
            let mut token: String = (0..length)
                .map(|at| {
                    let low = usize::from(at == 0);
                    char::from(b'0' + u8::try_from(low + rng.below(10 - low)).unwrap())
                })
                .collect();
            if case % 7 == 0 {
                token = "9".repeat(length);
            }
            if case % 11 == 0 {
                // A power of two and its neighbours: the ties of the rounding.
                let power = 1u128 << (64 + rng.below(63));
                token = (power + u128::try_from(rng.below(5)).unwrap() - 2).to_string();
            }
            if case % 5 == 0 {
                token.insert_str(0, &"0".repeat(rng.below(30)));
            }
            if case % 3 == 0 {
                token.insert(0, '-');
            }
            if !is_wide_integer(token.as_bytes()) {
                continue;
            }
            check_spelling(&token);
            let spelling = real_spelling(&token);
            let significant = spelling
                .trim_start_matches(['-', '0'])
                .split('e')
                .next()
                .unwrap()
                .len();
            longest = longest.max(significant);
            checked += 1;
        }
        assert!(checked > 30_000, "{checked} wide integers");
        assert_eq!(longest, 17, "a double of seventeen digits was respelt");
    }

    #[test]
    fn a_text_is_copied_only_when_it_has_a_wide_integer_or_a_comment() {
        // A comment is blanked byte for byte, its newline kept; one that never ends inside
        // the root never reaches a copy: the scan refuses the text, in the parser's words.
        let commented = "[1, // é\n 2 /* x */, /**/ 3] // after the root";
        let copied = checked_of(commented);
        assert_eq!(copied.0, &commented[..29]);
        assert_eq!(copied.1.as_deref(), Some("[1,      \n 2        ,      3]"));
        let open = super::checked("[1, /* open", true).err().expect("refused");
        assert!(matches!(
            open.kind,
            DeserializeErrorKind::UnexpectedEof {
                expected: "in block comment"
            }
        ));
        assert_eq!(open.span, Some(facet::Span::new(4, 7)));
        // Without comments a slash is no comment: nothing is blanked.
        assert!(super::checked("[1] // x", false).unwrap().copy.is_none());
        // Past a fault nothing is marked: the parser's message names the author's bytes.
        assert_eq!(checked_of("[tru/* c */, 18446744073709551616]").1, None);

        let plain = r#"{"a": 18446744073709551615, "b": "18446744073709551616", "c": 1e999}"#;
        let checked = checked(plain, true).unwrap();
        assert!(checked.copy.is_none());
        assert_eq!(checked.root, plain);
        let text = "{\"a\": 18446744073709551616, \"é\": [-9223372036854775809, \"x\"]} tail";
        let checked = checked_of(text);
        assert_eq!(checked.0, &text[..text.len() - 5]);
        assert_eq!(
            checked.1.as_deref(),
            Some("{\"a\": 018446744073709552e3, \"é\": [-09223372036854776e3, \"x\"]}")
        );
    }

    fn checked_of(text: &str) -> (&str, Option<String>) {
        let checked = checked(text, true).unwrap();
        (checked.root, checked.copy)
    }

    /// A target with a member of each kind the parser is asked for differently.
    #[derive(Debug, PartialEq, Facet)]
    struct Target<'a> {
        #[facet(default)]
        name: Option<Cow<'a, str>>,
        #[facet(default)]
        plain: Option<&'a str>,
        #[facet(default)]
        number: Option<f64>,
        #[facet(default)]
        either: Option<Either>,
        #[facet(default)]
        shape: Option<Shape>,
        #[facet(default)]
        raw: Option<RawJson<'a>>,
        #[facet(default)]
        members: IndexMap<Cow<'a, str>, RawJson<'a>>,
        #[facet(default)]
        any: Option<Value>,
        #[facet(default)]
        list: Vec<Option<u8>>,
        #[facet(default)]
        inner: Option<Box<Target<'a>>>,
    }

    #[derive(Debug, PartialEq, Facet)]
    #[facet(untagged)]
    #[repr(u8)]
    enum Either {
        Text(String),
        Number(f64),
        Pair { a: u8, b: u8 },
        Named { name: String },
        List(Vec<u8>),
    }

    #[derive(Debug, PartialEq, Facet)]
    #[repr(u8)]
    enum Shape {
        Round,
        Square { side: u8 },
    }

    /// `text` read as `T` by the parser over a copy of it, borrowing from `text`.
    fn through_a_copy<'a, T: Facet<'a>>(text: &'a str) -> Result<T, DeserializeError> {
        let copy = text.to_owned();
        let mut parser = Rehomed::new(&copy, text, true);
        FormatDeserializer::new(&mut parser).deserialize_root()
    }

    /// The parser over a copy reads every document exactly as facet-json reads the text:
    /// the same value, or the same fault at the same place.
    #[test]
    fn a_read_over_a_copy_is_facet_s_read_of_the_text() {
        let documents = [
            r#"{"name": "n", "plain": "p", "number": 1.5, "list": [1, null, 3]}"#,
            r#"{"name": "a\nb", "either": "text", "shape": "Round", "raw": {"x": [1, "é"]}}"#,
            r#"{"either": 2.5, "shape": {"Square": {"side": 4}}, "raw": "é", "any": {"k": ["é", 1]}}"#,
            r#"{"either": {"a": 1, "b": 2}, "members": {"é": 1, "b": [true, null], "c": "x"}}"#,
            r#"{"either": {"name": "n"}, "inner": {"name": "deep", "inner": {"either": [1, 2]}}}"#,
            "{ // c\n \"name\": /* é */ \"n\", \"unknown\": {\"deep\": [1, {\"x\": \"y\"}]}, \"number\": 7 }",
            r#"{"name": null, "number": null, "list": []}"#,
            r"{}",
            // Faults: type, enum name, syntax, end of text, a duplicate, a wrong root.
            r#"{"name": 5, "number": "x"}"#,
            r#"{"number": true}"#,
            r#"{"shape": "Oval"}"#,
            r#"{"shape": {"Square": {"side": 300}}}"#,
            r#"{"list": [1, 2, 256]}"#,
            r#"{"list": [1, "x"]}"#,
            r#"{"either": {"zzz": 1}}"#,
            r#"{"either": true}"#,
            r#"{"name": "n" "number": 1}"#,
            r#"{"name": "n", "raw": {"x": }}"#,
            r#"{"name": "n", "inner": {"plain": "a\nb"}}"#,
            r#"{"name": "n", "name": "m"}"#,
            r#"{"members": {"a": 1, "b": }}"#,
            r#"{"name": "n""#,
            r"[1, 2]",
            r#""text""#,
            "",
        ];
        let (mut read, mut failed) = (0, 0);
        for text in documents {
            let facet = facet_json::from_str_borrowed_jsonc::<Target<'_>>(text);
            let ours = through_a_copy::<Target<'_>>(text);
            match (&ours, &facet) {
                (Ok(ours), Ok(facet)) => {
                    assert_eq!(ours, facet, "`{text}`");
                    read += 1;
                }
                (Err(ours), Err(facet)) => {
                    assert_eq!(ours.kind.to_string(), facet.kind.to_string(), "`{text}`");
                    assert_eq!(ours.span, facet.span, "`{text}`");
                    assert_eq!(
                        ours.path.as_ref().map(ToString::to_string),
                        facet.path.as_ref().map(ToString::to_string),
                        "`{text}`"
                    );
                    failed += 1;
                }
                _ => panic!("`{text}`: over a copy {ours:?}, facet {facet:?}"),
            }
            let value = through_a_copy::<Value>(text);
            let facet = facet_json::from_str_borrowed_jsonc::<Value>(text);
            assert_eq!(value.as_ref().ok(), facet.as_ref().ok(), "`{text}`");
            assert_eq!(
                value.as_ref().err().map(|e| (e.kind.to_string(), e.span)),
                facet.as_ref().err().map(|e| (e.kind.to_string(), e.span)),
                "`{text}`"
            );
        }
        assert!(read >= 8 && failed >= 12, "{read} read, {failed} failed");
    }

    /// `text`, which the scan passed, read as a value by facet's own recursion: the read the
    /// door gave a value before it read one itself.
    fn by_facet(checked: &Checked<'_>, comments: bool) -> Result<Value, DeserializeError> {
        let read = checked.copy.as_deref().unwrap_or(checked.root);
        let mut parser = Rehomed::new(read, checked.root, comments).owning();
        FormatDeserializer::new_owned(&mut parser).deserialize_root()
    }

    /// A random value as text, `levels` deep at most: every scalar JSON has, keys written
    /// twice (facet's value merges two objects of one key), comments and white space.
    fn random_text(rng: &mut Rng, levels: usize, out: &mut String) {
        const SCALARS: &[&str] = &[
            "null",
            "true",
            "false",
            "0",
            "-0",
            "1",
            "-1",
            "7.5",
            "-0.0",
            "1e3",
            "1E-3",
            "1e999",
            "-1e999",
            "9223372036854775807",
            "9223372036854775808",
            "-9223372036854775808",
            "18446744073709551615",
            "18446744073709551616",
            "-9223372036854775809",
            "340282366920938463463374607431768211456",
            "0.1",
            "123456789012345678901234567890.5",
            r#""""#,
            r#""text""#,
            r#""é €""#,
            r#""a\nb\t\"\\\/\u00e9\ud83d\ude00""#,
        ];
        const KEYS: &[&str] = &[r#""a""#, r#""b""#, r#""\u0061""#, r#""é""#, r#""""#];
        const GAPS: &[&str] = &["", " ", "\n", " /* c */ ", " // c\n"];
        out.push_str(GAPS[rng.below(GAPS.len())]);
        match if levels == 0 { 0 } else { rng.below(4) } {
            0 | 1 => out.push_str(SCALARS[rng.below(SCALARS.len())]),
            2 => {
                out.push('[');
                for index in 0..rng.below(4) {
                    if index > 0 {
                        out.push(',');
                    }
                    random_text(rng, levels - 1, out);
                }
                out.push(']');
            }
            _ => {
                out.push('{');
                for index in 0..rng.below(5) {
                    if index > 0 {
                        out.push(',');
                    }
                    out.push_str(GAPS[rng.below(GAPS.len())]);
                    out.push_str(KEYS[rng.below(KEYS.len())]);
                    out.push(':');
                    random_text(rng, levels - 1, out);
                }
                out.push_str(GAPS[rng.below(GAPS.len())]);
                out.push('}');
            }
        }
        out.push_str(GAPS[rng.below(GAPS.len())]);
    }

    /// The door reads a value itself, without a call for each level: it gives what
    /// facet's recursion gives, value for value and fault for fault, for every text.
    #[test]
    fn a_value_read_by_the_door_is_the_value_facet_reads() {
        let mut texts: Vec<String> = [
            "",
            " ",
            "// only a comment",
            "1",
            "\"text\"",
            "[",
            "{",
            "]",
            "}",
            "[1,",
            "[1 2]",
            "[1,]",
            "[,1]",
            "{\"a\"}",
            "{\"a\":}",
            "{\"a\": 1,}",
            "{\"a\": 1 \"b\": 2}",
            "{1: 2}",
            "{\"a\": [1, {\"b\": tru}]}",
            "[1, 2}",
            "{\"a\": 1]",
            "[\"open",
            "[1, /* open",
            "nul",
            "-",
            "1.",
            "01",
            "[1e]",
            "\u{0}",
            "[\"a\tb\"]",
            "{\"a\": {\"x\": 1}, \"a\": {\"y\": 2}}",
            "{\"a\": [1], \"a\": [2, 3]}",
            "{\"a\": {\"x\": 1}, \"a\": 2, \"a\": {\"z\": [{}]}}",
            "{\"a\": 1, \"a\": {\"x\": 1}, \"a\": [1], \"a\": [2]}",
            "[[[[[[[[[[[[[[[[[[[[[[[[1]]]]]]]]]]]]]]]]]]]]]]]] trailing",
        ]
        .map(str::to_owned)
        .to_vec();
        let mut rng = Rng(0x00B1_6157_5EED_0D05);
        for case in 0..3_000 {
            let mut text = String::new();
            random_text(&mut rng, 1 + case % 6, &mut text);
            // One text in five is cut or has a byte changed: the faults.
            match rng.below(10) {
                0 => {
                    let mut cut = rng.below(text.len() + 1);
                    while !text.is_char_boundary(cut) {
                        cut -= 1;
                    }
                    text.truncate(cut);
                }
                1 if !text.is_empty() => {
                    let at = rng.below(text.len());
                    if text.is_char_boundary(at) && text.is_char_boundary(at + 1) {
                        let with = ["}", "]", ",", ":", "x", "\"", "{", "["][rng.below(8)];
                        text.replace_range(at..=at, with);
                    }
                }
                _ => {}
            }
            texts.push(text);
        }
        let (mut read, mut failed) = (0, 0);
        for text in &texts {
            for comments in [true, false] {
                // What the scan refuses reaches neither reader.
                let Ok(checked) = checked(text, comments) else {
                    continue;
                };
                let facet = by_facet(&checked, comments);
                let ours: Result<Value, _> = read_owned(
                    checked.root,
                    checked.copy.as_deref(),
                    comments,
                    checked.levels,
                );
                match (&ours, &facet) {
                    (Ok(ours), Ok(facet)) => {
                        assert_eq!(ours, facet, "`{text}`");
                        // The kind of each number too (an integer is no double).
                        assert_eq!(format!("{ours:?}"), format!("{facet:?}"), "`{text}`");
                        read += 1;
                    }
                    (Err(ours), Err(facet)) => {
                        assert_eq!(ours.kind.to_string(), facet.kind.to_string(), "`{text}`");
                        assert_eq!(ours.span, facet.span, "`{text}`");
                        assert_eq!(
                            ours.path.as_ref().map(ToString::to_string),
                            facet.path.as_ref().map(ToString::to_string),
                            "`{text}`"
                        );
                        failed += 1;
                    }
                    _ => panic!("`{text}`: the door {ours:?}, facet {facet:?}"),
                }
                if comments {
                    let borrowed = from_str_borrowed_jsonc::<Value>(text);
                    assert_eq!(borrowed.as_ref().ok(), ours.as_ref().ok(), "`{text}`");
                    assert_eq!(
                        borrowed
                            .as_ref()
                            .err()
                            .map(|e| (e.kind.to_string(), e.span)),
                        ours.as_ref().err().map(|e| (e.kind.to_string(), e.span)),
                        "`{text}`"
                    );
                }
            }
        }
        assert!(read > 3_000 && failed > 500, "{read} read, {failed} failed");
    }

    /// A type that would keep what a read lends it.
    #[derive(Debug, PartialEq, Facet)]
    struct Lent<'a> {
        name: Cow<'a, str>,
        #[facet(default)]
        members: IndexMap<Cow<'a, str>, Cow<'a, str>>,
        #[facet(default)]
        list: Vec<Cow<'a, str>>,
    }

    /// An owned read lends nothing: a `Cow` of a type read as `'static` from a text that is
    /// not owns its string, with or without a copy in between (reviews/CORE-BIGINT minor 2).
    #[test]
    fn an_owned_read_lends_nothing_of_the_text() {
        let plain = r#"{"name": "n", "members": {"k": "v"}, "list": ["a", "b"]}"#;
        let commented = plain.replace("{\"k\"", "{ /* c */ \"k\"");
        let wide = plain.replace("[\"a\"", "[\"18446744073709551616\"");
        for text in [plain.to_owned(), commented, wide] {
            let within = |part: &str| {
                let (start, base) = (part.as_ptr() as usize, text.as_ptr() as usize);
                start >= base && start < base + text.len()
            };
            for read in [from_str_jsonc::<Lent<'static>>, from_str::<Lent<'static>>] {
                let Ok(lent) = read(&text) else {
                    // Plain JSON has no comments.
                    assert!(text.contains("/*"), "`{text}`");
                    continue;
                };
                let strings = std::iter::once(&lent.name)
                    .chain(lent.members.keys())
                    .chain(lent.members.values())
                    .chain(&lent.list);
                for string in strings {
                    assert!(
                        matches!(string, Cow::Owned(_)) && !within(string),
                        "`{string}` of `{text}` is lent"
                    );
                }
                assert_eq!(lent.name, "n");
            }
            let lent: Lent<'static> = from_slice_jsonc(text.as_bytes()).unwrap();
            assert!(matches!(lent.name, Cow::Owned(_)));
        }
    }

    /// What a read over a copy lends is the text: borrowed strings, keys and raw slices lie
    /// in it, at the offsets they have in the copy.
    #[test]
    fn a_read_over_a_copy_borrows_from_the_text() {
        let text =
            r#"{"name": "é n", "plain": "p", "raw": [1, "é"], "members": {"k é": {"x": 1}}}"#;
        let target: Target<'_> = through_a_copy(text).unwrap();
        let within = |part: &str| {
            let (start, base) = (part.as_ptr() as usize, text.as_ptr() as usize);
            start >= base && start + part.len() <= base + text.len()
        };
        assert!(matches!(target.name, Some(Cow::Borrowed("é n"))));
        assert!(within(target.name.as_deref().unwrap()));
        assert!(within(target.plain.unwrap()));
        let raw = target.raw.unwrap();
        assert!(matches!(raw.0, Cow::Borrowed(r#"[1, "é"]"#)) && within(raw.as_str()));
        let (key, member) = target.members.get_index(0).unwrap();
        assert!(matches!(key, Cow::Borrowed("k é")) && within(key));
        assert!(matches!(member.0, Cow::Borrowed(r#"{"x": 1}"#)) && within(member.as_str()));
    }
}
