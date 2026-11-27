//! The tokens of the text as facet-json's scanner and parser meet them: what each is, where it
//! ends, what it spells, and the faults a token can hold. The grammar is `parse`'s.
//!
//! facet-json reads a token in two steps and `parse` keeps them apart because the order of the
//! faults depends on it: [`Scanner::scan`] finds the token and refuses a byte that starts none,
//! a number or literal gone wrong and a string without its end; [`Scanner::materialize`] gives
//! it its value and refuses a number no float reads and a `\u` escape without its pair. A
//! token that is where the grammar does not take it is refused between the two.

use std::borrow::Cow;

use super::fault::{ESCAPE, Fault, FaultKind, TOKEN};
use super::spans::NumberKind;
use crate::json::lex::{Kind, Lexer, Rules, UnsafeText, is_wide_integer, string_end};

/// What a number token is by its spelling, as facet-json's scanner hints it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Hint {
    /// Digits alone.
    Unsigned,
    /// A `-` and digits alone.
    Signed,
    /// A fraction or an exponent (and an integer past 64 bits, which is read as the real
    /// token that spells its double).
    Float,
}

/// What a token is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tk {
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    Colon,
    Comma,
    Null,
    True,
    False,
    /// A string whose escapes are RFC 8259's.
    Str {
        escapes: bool,
    },
    /// A string with an escape that is not: the text is refused where it is read as a value
    /// or a key, and where it is not, the grammar's fault is the one told.
    Bad(UnsafeText),
    Number(Hint),
    /// The end of the text.
    Eof,
}

/// One token: what it is and where.
#[derive(Clone, Copy, Debug)]
pub(super) struct Token {
    pub(super) kind: Tk,
    pub(super) start: usize,
    pub(super) end: usize,
}

impl Token {
    pub(super) fn len(&self) -> usize {
        self.end - self.start
    }
}

/// What a token spells, once it is materialized.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Lit<'t> {
    /// A token that spells nothing (a bracket, a comma, a literal).
    None,
    Str(Cow<'t, str>),
    U64(u64),
    I64(i64),
    F64(f64),
}

/// A token and what it spells.
#[derive(Clone, Debug)]
pub(super) struct Mat<'t> {
    pub(super) token: Token,
    pub(super) lit: Lit<'t>,
    /// What a reader may have to be told of the number.
    pub(super) note: Option<NumberKind>,
}

impl Mat<'_> {
    /// The token as facet-json's parser names it in `unexpected token: got ..`: its kind, and
    /// for a value its payload.
    pub(super) fn name(&self, text: &str) -> String {
        match (&self.token.kind, &self.lit) {
            (Tk::Str { .. }, Lit::Str(value)) => format!("String({value:?})"),
            (Tk::Bad(_), _) => bad_name(text, &self.token),
            (Tk::Number(_), Lit::U64(n)) => format!("U64({n})"),
            (Tk::Number(_), Lit::I64(n)) => format!("I64({n})"),
            (Tk::Number(_), Lit::F64(n)) => format!("F64({n:?})"),
            (kind, _) => plain_name(*kind).to_owned(),
        }
    }
}

/// The name of a token that spells no payload.
fn plain_name(kind: Tk) -> &'static str {
    match kind {
        Tk::ObjectStart => "ObjectStart",
        Tk::ObjectEnd => "ObjectEnd",
        Tk::ArrayStart => "ArrayStart",
        Tk::ArrayEnd => "ArrayEnd",
        Tk::Colon => "Colon",
        Tk::Comma => "Comma",
        Tk::Null => "Null",
        Tk::True => "True",
        Tk::False => "False",
        Tk::Eof => "Eof",
        Tk::Str { .. } | Tk::Bad(_) | Tk::Number(_) => "",
    }
}

/// The name of a string with a bad escape where the grammar does not take a string: the text
/// as written, since it does not unescape.
fn bad_name(text: &str, token: &Token) -> String {
    format!("String({})", text.get(token.start..token.end).unwrap_or(""))
}

/// The name of a token as the scanner names it (where the parser asked for a colon and read
/// the token without materializing it).
pub(super) fn scan_name(text: &str, token: &Token) -> String {
    match token.kind {
        Tk::Str { escapes } => format!(
            "String {{ start: {}, end: {}, has_escapes: {escapes} }}",
            token.start + 1,
            token.end - 1
        ),
        Tk::Number(hint) => format!(
            "Number {{ start: {}, end: {}, hint: {hint:?} }}",
            token.start, token.end
        ),
        Tk::Bad(_) => bad_name(text, token),
        kind => plain_name(kind).to_owned(),
    }
}

/// The scanner of a text: facet-json's `Scanner` over the parser's lexer.
pub(super) struct Scanner<'t> {
    text: &'t str,
    bytes: &'t [u8],
    lexer: Lexer<'t>,
}

impl<'t> Scanner<'t> {
    pub(super) fn new(text: &'t str, comments: bool) -> Self {
        Self {
            text,
            bytes: text.as_bytes(),
            lexer: Lexer::new(text.as_bytes(), Rules::strict(comments)),
        }
    }

    pub(super) fn text(&self) -> &'t str {
        self.text
    }

    /// Steps past white space and comments; a block comment without its end is a fault.
    pub(super) fn skip(&mut self) -> Result<(), Fault> {
        match self.lexer.skip_trivia() {
            None => Ok(()),
            Some(slash) => Err(Fault::new(
                slash,
                self.bytes.len() - slash,
                FaultKind::UnexpectedEof {
                    expected: "in block comment",
                },
            )),
        }
    }

    /// The offset of the first content from `at` that is neither white space nor a whole
    /// comment (a `/*` without its end is content from its slash); `None` when only those
    /// follow.
    pub(super) fn content_from(&mut self, at: usize) -> Option<usize> {
        self.lexer.seek(at);
        match self.lexer.skip_trivia() {
            Some(slash) => Some(slash),
            None => (self.lexer.at() < self.bytes.len()).then_some(self.lexer.at()),
        }
    }

    /// If the next byte after white space and comments is `byte`, takes it.
    ///
    /// # Errors
    ///
    /// The fault of a comment without its end.
    pub(super) fn take(&mut self, byte: u8) -> Result<bool, Fault> {
        self.skip()?;
        let at = self.lexer.at();
        if self.bytes.get(at) == Some(&byte) {
            self.lexer.seek(at + 1);
            return Ok(true);
        }
        Ok(false)
    }

    /// The next token, after white space and comments.
    ///
    /// # Errors
    ///
    /// A comment without its end, a byte that starts no token, a number that is a `-` and
    /// nothing, a literal that goes wrong, a string without its end.
    pub(super) fn scan(&mut self) -> Result<Token, Fault> {
        self.skip()?;
        let start = self.lexer.at();
        let token = match self.lexer.token() {
            Ok(None) => {
                return Ok(Token {
                    kind: Tk::Eof,
                    start,
                    end: start,
                });
            }
            Err(bad) => {
                return Ok(Token {
                    kind: Tk::Bad(bad),
                    start,
                    end: start + loose_len(self.bytes, start),
                });
            }
            Ok(Some(token)) => token,
        };
        let (start, end) = (token.start, token.end);
        let kind = match token.kind {
            Kind::OpenObject => Tk::ObjectStart,
            Kind::CloseObject => Tk::ObjectEnd,
            Kind::OpenArray => Tk::ArrayStart,
            Kind::CloseArray => Tk::ArrayEnd,
            Kind::Colon => Tk::Colon,
            Kind::Comma => Tk::Comma,
            Kind::String { closed: true } => Tk::Str {
                escapes: self.bytes[start + 1..end - 1].contains(&b'\\'),
            },
            Kind::String { closed: false } => {
                return Err(Fault::new(
                    start,
                    end - start,
                    FaultKind::UnexpectedEof {
                        expected: "in string",
                    },
                ));
            }
            Kind::Number => {
                // A `-` with no digit: the fault names the byte after it.
                if end - start == 1 && self.bytes[start] == b'-' {
                    let ch = self.bytes.get(end).map_or('?', |&byte| char::from(byte));
                    return Err(Fault::new(
                        start,
                        1,
                        FaultKind::UnexpectedChar {
                            ch,
                            expected: TOKEN,
                        },
                    ));
                }
                Tk::Number(hint_of(&self.bytes[start..end]))
            }
            Kind::Literal { whole: true } => match self.bytes[start] {
                b't' => Tk::True,
                b'f' => Tk::False,
                _ => Tk::Null,
            },
            Kind::Literal { whole: false } => return Err(self.literal_fault(start, end)),
            // The lexer is strict: `Atom` is its lenient reading's.
            Kind::Atom | Kind::Other => {
                return Err(Fault::new(
                    start,
                    1,
                    FaultKind::UnexpectedChar {
                        ch: char::from(self.bytes[start]),
                        expected: TOKEN,
                    },
                ));
            }
        };
        Ok(Token { kind, start, end })
    }

    /// The fault of a literal `true`, `false` or `null` that stops at `end`.
    fn literal_fault(&self, start: usize, end: usize) -> Fault {
        match self.bytes.get(end) {
            Some(&byte) => {
                // The byte that is no part of the literal may start a string with a bad
                // escape: the fault is told on the whole string.
                let len = if byte == b'"' && string_end(self.bytes, end).is_err() {
                    loose_len(self.bytes, end)
                } else {
                    1
                };
                Fault::new(
                    end,
                    len,
                    FaultKind::UnexpectedChar {
                        ch: char::from(byte),
                        expected: TOKEN,
                    },
                )
            }
            None => Fault::new(
                start,
                end - start,
                FaultKind::UnexpectedEof {
                    expected: "in literal",
                },
            ),
        }
    }

    /// What `token` spells.
    ///
    /// # Errors
    ///
    /// A number no float reads (`1e`), a `\u` escape that is half a surrogate pair.
    pub(super) fn materialize(&self, token: Token) -> Result<Mat<'t>, Fault> {
        let (lit, note) = match token.kind {
            Tk::Str { escapes } => {
                let content = &self.text[token.start + 1..token.end - 1];
                let value = if escapes {
                    Cow::Owned(decode(content, token.start + 1)?)
                } else {
                    Cow::Borrowed(content)
                };
                (Lit::Str(value), None)
            }
            Tk::Number(_) => number(&self.bytes[token.start..token.end], token.start)?,
            _ => (Lit::None, None),
        };
        Ok(Mat { token, lit, note })
    }
}

/// The length of the string token at `at` the way facet-json's error for it reads it: a
/// backslash and the byte after it count as two, to the closing quote or the end of the text.
fn loose_len(bytes: &[u8], at: usize) -> usize {
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

/// How facet-json's scanner hints a number token, with an integer past 64 bits spelt as the
/// real token the old door gave facet in its place.
fn hint_of(token: &[u8]) -> Hint {
    if is_wide_integer(token) || token.iter().any(|byte| matches!(byte, b'.' | b'e' | b'E')) {
        Hint::Float
    } else if token.first() == Some(&b'-') {
        Hint::Signed
    } else {
        Hint::Unsigned
    }
}

/// The number `token` (at `start`) spells, and what a reader may have to be told of it.
///
/// An integer past 64 bits is the double it spells (what the old door's `respell` made of it),
/// a token no double holds an infinity. Every other integer is `i64` or `u64` and every
/// other token an `f64`, as facet-json reads them.
fn number(token: &[u8], start: usize) -> Result<(Lit<'static>, Option<NumberKind>), Fault> {
    let fault = || {
        Fault::new(
            start,
            token.len(),
            FaultKind::UnexpectedChar {
                ch: '?',
                expected: TOKEN,
            },
        )
    };
    let text = std::str::from_utf8(token).map_err(|_| fault())?;
    let negative = token.first() == Some(&b'-');
    let wide = is_wide_integer(token);
    let lit = match hint_of(token) {
        Hint::Float => {
            let real: f64 = text.parse().map_err(|_| fault())?;
            let note = if real.is_infinite() {
                Some(NumberKind::BeyondDouble)
            } else if wide {
                Some(NumberKind::WideInteger)
            } else if negative && real == 0.0 {
                Some(NumberKind::NegativeZero)
            } else {
                None
            };
            return Ok((Lit::F64(real), note));
        }
        Hint::Signed => Lit::I64(text.parse().map_err(|_| fault())?),
        Hint::Unsigned => Lit::U64(text.parse().map_err(|_| fault())?),
    };
    let zero = matches!(lit, Lit::I64(0));
    Ok((lit, (negative && zero).then_some(NumberKind::NegativeZero)))
}

/// The text of a string with escapes, as facet-json decodes it: `content` is what lies
/// between the quotes, `start` its offset in the text.
///
/// # Errors
///
/// `InvalidUtf8` for a `\u` escape that is a surrogate without its pair, at the span facet-json
/// gives it.
pub(super) fn decode(content: &str, start: usize) -> Result<String, Fault> {
    let bytes = content.as_bytes();
    let bad = |at: usize, len: usize| Fault::new(start + at, len, FaultKind::InvalidUtf8);
    let mut out = String::with_capacity(content.len());
    let mut i = 0;
    while i < bytes.len() {
        // The run up to the next backslash is text as it is: a backslash is ASCII, so the
        // cut is on a character boundary.
        let run = bytes[i..]
            .iter()
            .position(|&byte| byte == b'\\')
            .unwrap_or(bytes.len() - i);
        out.push_str(&content[i..i + run]);
        i += run;
        if i >= bytes.len() {
            break;
        }
        // The backslash.
        i += 1;
        let Some(&escape) = bytes.get(i) else {
            return Err(Fault::new(
                start + i - 1,
                1,
                FaultKind::UnexpectedEof {
                    expected: "in escape sequence",
                },
            ));
        };
        match escape {
            b'"' => out.push('"'),
            b'\\' => out.push('\\'),
            b'/' => out.push('/'),
            b'b' => out.push('\x08'),
            b'f' => out.push('\x0c'),
            b'n' => out.push('\n'),
            b'r' => out.push('\r'),
            b't' => out.push('\t'),
            b'u' => {
                i += 1;
                let high = hex4(bytes, i).ok_or_else(|| {
                    if i + 4 > bytes.len() {
                        Fault::new(
                            start + i - 2,
                            bytes.len() - i + 2,
                            FaultKind::UnexpectedEof {
                                expected: "in unicode escape",
                            },
                        )
                    } else {
                        Fault::new(
                            start + i,
                            4,
                            FaultKind::UnexpectedChar {
                                ch: '?',
                                expected: TOKEN,
                            },
                        )
                    }
                })?;
                i += 4;
                let code = if (0xD800..=0xDBFF).contains(&high) {
                    // A high surrogate wants `\uXXXX` after it.
                    if i + 6 > bytes.len() || bytes[i] != b'\\' || bytes[i + 1] != b'u' {
                        return Err(bad(i - 6, 6));
                    }
                    i += 2;
                    let low = hex4(bytes, i).ok_or_else(|| {
                        Fault::new(
                            start + i,
                            4,
                            FaultKind::UnexpectedChar {
                                ch: '?',
                                expected: TOKEN,
                            },
                        )
                    })?;
                    i += 4;
                    if !(0xDC00..=0xDFFF).contains(&low) {
                        return Err(bad(i - 4, 4));
                    }
                    0x1_0000 + ((u32::from(high) & 0x3FF) << 10) + (u32::from(low) & 0x3FF)
                } else if (0xDC00..=0xDFFF).contains(&high) {
                    return Err(bad(i - 4, 4));
                } else {
                    u32::from(high)
                };
                let Some(found) = char::from_u32(code) else {
                    return Err(bad(i - 4, 4));
                };
                out.push(found);
                continue;
            }
            // Not an escape of RFC 8259: the lexer refuses it before a string is decoded.
            _ => return Err(unknown_escape(start + i - 1, escape)),
        }
        i += 1;
    }
    Ok(out)
}

/// The fault for a backslash before a byte that is no escape.
fn unknown_escape(at: usize, byte: u8) -> Fault {
    Fault::new(
        at,
        2,
        FaultKind::UnexpectedChar {
            ch: char::from(byte),
            expected: ESCAPE,
        },
    )
}

/// The four hexadecimal digits at `at` as a number.
fn hex4(bytes: &[u8], at: usize) -> Option<u16> {
    let digits = bytes.get(at..at + 4)?;
    if !digits.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    u16::from_str_radix(std::str::from_utf8(digits).ok()?, 16).ok()
}

/// The text of the string between whose quotes `raw` lies, unescaped; `raw` itself when it
/// does not unescape (a key that parsed has no such escape).
pub(super) fn unescape(raw: &str) -> String {
    decode(raw, 0).unwrap_or_else(|_| raw.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decoded(content: &str) -> Result<String, Fault> {
        decode(content, 1)
    }

    #[test]
    fn a_surrogate_pair_decodes_to_the_character_it_spells() {
        assert_eq!(decoded(r"😀").as_deref(), Ok("\u{1F600}"));
    }

    #[test]
    fn a_surrogate_without_its_pair_is_refused_at_the_escape() {
        let high = decoded(r"a\ud800").unwrap_err();
        assert_eq!((high.span.offset, high.span.len), (2, 6));
        assert_eq!(high.kind, FaultKind::InvalidUtf8);
        let low = decoded(r"\udc00").unwrap_err();
        assert_eq!((low.span.offset, low.span.len), (3, 4));
    }
}
