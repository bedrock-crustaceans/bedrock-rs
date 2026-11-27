//! The text parser: one pass over a JSON(C) text that gives the [`Value`] and a tree of
//! spans ([`Spans`]), with the reading policies as [`Options`] and not as rewrites of the text.
//!
//! It reads what `from_str_jsonc::<Value>` reads, the same value for every text it takes and the
//! same refusal ([`Fault`]: the span and the words of facet-json's error) for every text it
//! does not, and it never looks at what follows the root value (vanilla's reader stops there;
//! [`Parsed::trailing`] says where it lies). The dialect is JSON with `//` and `/* */`
//! comments, trailing commas, the white space of RFC 8259, strings with the escapes of
//! RFC 8259 only, and nesting no deeper than [`MAX_JSON_DEPTH`]. A byte order mark is no white
//! space here: blank it first (`blank_bom`).
//!
//! # Text rewrites and what the parse does instead
//!
//! - A key written twice (`Options::duplicates`): the copy that stands is decided while
//!   parsing. (The versioned pipeline still blanks the copies that do not stand in the text,
//!   `last_wins_except` at `versioning/pipeline.rs`, so that offsets stay, before it parses.)
//!   `Winner::Last` leaves the member where its last copy stands, `First` where its first
//!   stands, and `Keep` reads every copy as facet-value does (two objects merge member by
//!   member, two arrays join, anything else is replaced by the later copy at the first copy's
//!   place); `LastInPlace` puts the last copy's value, whole, at the first copy's place. The
//!   faults of the text are the same whatever the rule: a copy that does not stand is still
//!   text of the file.
//! - An integer past 64 bits is the double it spells ([`NumberKind::WideInteger`]); a token no
//!   double holds is an infinity ([`NumberKind::BeyondDouble`]); nothing is respelt. A reader
//!   that must clamp a number at a member of another width finds the token with
//!   [`Spans::at`].
//! - Spans are made as the text is read, so a finding at any path costs its path, not the text
//!   ([`Spans::at`] walks the tree, not the text).
//!
//! The parse runs on one stack frame however deep the text is; the open containers are in a
//! list, no more than the 128 levels `MAX_JSON_DEPTH` allows.

mod fault;
mod spans;
mod token;

use std::borrow::Cow;
use std::collections::HashMap;

use facet_value::{Destructured, VArray, VNumber, VObject, VString};

pub use fault::{Fault, FaultKind};
pub use spans::{NumberKind, NumberNote, SpanKind, SpanMember, SpanNode, Spans};

use fault::{ESCAPE, SHALLOWER, UNICODE_ESCAPE, to_u32};
use spans::{Edge, Node};
use token::{Lit, Mat, Scanner, Tk};

use super::lex::{Kind, Lexer, Rules, UnsafeText, scan_tokens};
use super::slice::JsonSlice;
use super::text::MAX_JSON_DEPTH;
use super::{Value, Winner};
use crate::diagnostic::{JsonPath, Span};

/// Which copy of a key written twice stands, per object.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum Duplicates<'a> {
    /// The same rule at every object.
    Uniform(Winner),
    /// The last copy stands, except in the object that is the value of a member named in the
    /// list (at any depth), whose own members are all kept: `last_wins_except`.
    LastExcept(&'a [&'a str]),
    /// The rule for each object, by its path from the root.
    ByPath(&'a dyn Fn(&JsonPath) -> Winner),
}

impl std::fmt::Debug for Duplicates<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uniform(winner) => f.debug_tuple("Uniform").field(winner).finish(),
            Self::LastExcept(keep) => f.debug_tuple("LastExcept").field(keep).finish(),
            Self::ByPath(_) => f.write_str("ByPath(..)"),
        }
    }
}

/// How [`parse`] reads a text.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct Options<'a> {
    /// Which copy of a key written twice stands. The default keeps every copy as
    /// `from_str_jsonc::<Value>` reads them (`Winner::Keep`).
    pub duplicates: Duplicates<'a>,
    /// `//` and `/* */` comments are white space (the default); without them a `/` is a byte
    /// that starts no token, as plain JSON has it (`from_str`).
    pub comments: bool,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            duplicates: Duplicates::Uniform(Winner::Keep),
            comments: true,
        }
    }
}

impl<'a> Options<'a> {
    /// The same rule for every object.
    #[must_use]
    pub fn winner(winner: Winner) -> Self {
        Self {
            duplicates: Duplicates::Uniform(winner),
            ..Self::default()
        }
    }

    /// `Winner::Last` everywhere but in the object that is the value of a member named in
    /// `keep`: [`last_wins_except`](crate::json::last_wins_except).
    #[must_use]
    pub fn last_except(keep: &'a [&'a str]) -> Self {
        Self {
            duplicates: Duplicates::LastExcept(keep),
            ..Self::default()
        }
    }

    /// The rule for each object, by its path from the root.
    #[must_use]
    pub fn by_path(winner: &'a dyn Fn(&JsonPath) -> Winner) -> Self {
        Self {
            duplicates: Duplicates::ByPath(winner),
            ..Self::default()
        }
    }

    /// The same options for plain JSON: a comment is no white space.
    #[must_use]
    pub fn without_comments(self) -> Self {
        Self {
            comments: false,
            ..self
        }
    }
}

/// A text read: its value, the spans of its parts, and what a reader may need to be told of.
#[derive(Clone, Debug)]
pub struct Parsed<'t> {
    /// The value, as `from_str_jsonc::<Value>` gives it for the text (with the repeated keys
    /// the [`Options`] ask for).
    pub value: Value,
    /// Where each value, key and element lies.
    pub spans: Spans<'t>,
    /// The numbers the value does not hold as written, in the order of the text.
    pub notes: Vec<NumberNote>,
    /// Where the content after the root value lies (white space and whole comments are not
    /// content), to the last byte that is not white space; vanilla's reader never looks at it.
    /// `None` when nothing follows. What
    /// [`trailing_content`](crate::json::trailing_content) says of the text.
    pub trailing: Option<Span>,
}

/// Reads `text`.
///
/// # Errors
///
/// The [`Fault`] of the first syntax error of the text, as `from_str_jsonc::<Value>` tells it.
pub fn parse<'t>(text: &'t str, options: Options<'_>) -> Result<Parsed<'t>, Fault> {
    read(text, 0, options)
}

/// Reads the text of `slice`; every span, those of a fault too, is a byte range of the file
/// the slice lies in.
///
/// # Errors
///
/// As [`parse`], at file offsets.
pub fn parse_slice<'t>(slice: JsonSlice<'t>, options: Options<'_>) -> Result<Parsed<'t>, Fault> {
    read(slice.text, slice.offset, options).map_err(|fault| fault.rebased(slice.offset))
}

/// Reads `bytes`, which may not be UTF-8: UTF-8 bytes as [`parse`] reads their text; other
/// bytes are read when only their comments are not UTF-8 (a byte of a comment that is no ASCII
/// reads as `?`, so offsets stay), and refused otherwise: a string that is not UTF-8 is
/// [`FaultKind::InvalidUtf8`] on the string, a byte outside strings and comments a byte that
/// starts no token. What follows the root value is not read, UTF-8 or not.
///
/// # Errors
///
/// The [`Fault`] of the first syntax error, as [`parse`] tells it.
pub fn parse_bytes(bytes: &[u8], options: Options<'_>) -> Result<Value, Fault> {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return parse(text, options).map(|parsed| parsed.value);
    }
    let scanned = scan_tokens(bytes, options.comments);
    let root = match scanned {
        Ok(scanned) => scanned
            .root_end
            .and_then(|end| bytes.get(..end))
            .unwrap_or(bytes),
        Err(refused) => return Err(scan_refusal(bytes, refused.found, refused.token, options)),
    };
    let text = match std::str::from_utf8(root) {
        Ok(text) => Cow::Borrowed(text),
        Err(_) => Cow::Owned(text_outside_comments(root, options.comments)?),
    };
    parse(&text, options).map(|parsed| parsed.value)
}

/// The fault of bytes, not UTF-8, the scan refused at `found`: the first fault of the UTF-8
/// text before the refused token when there is one there (the token itself stands in by a
/// token of its kind with nothing in it), else what the scan refused.
fn scan_refusal(bytes: &[u8], found: UnsafeText, token: usize, options: Options<'_>) -> Fault {
    let before = bytes
        .get(..token)
        .and_then(|before| std::str::from_utf8(before).ok());
    if let Some(before) = before {
        let stand_in = match (found, bytes.get(token)) {
            (UnsafeText::UnterminatedComment { .. }, _) => "",
            (_, Some(b'{')) => "{",
            (_, Some(b'[')) => "[",
            _ => "\"\"",
        };
        let probe = format!("{before}{stand_in}");
        if let Err(mut earlier) = parse(&probe, options)
            && !earlier.is_end_of_input()
            && (earlier.span.offset as usize) <= token
        {
            if earlier.span.offset as usize == token {
                let len = token_len(bytes, token);
                earlier.span = fault::span_of(token, len);
                if let FaultKind::UnexpectedToken { got, .. } = &mut earlier.kind
                    && bytes.get(token) == Some(&b'"')
                {
                    let written = &bytes[token..token + len];
                    *got = format!("String({})", String::from_utf8_lossy(written));
                }
            }
            return earlier;
        }
    }
    let mut fault = Parser::refused(found);
    if let (UnsafeText::TooDeep { offset }, FaultKind::UnexpectedChar { ch, .. }) =
        (found, &mut fault.kind)
        && bytes.get(offset) == Some(&b'{')
    {
        *ch = '{';
    }
    fault
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

/// `bytes` with every byte of a comment that is not ASCII replaced by `?` (offsets stay), as
/// text; a fault where a string, or a byte outside strings and comments, is not UTF-8.
fn text_outside_comments(bytes: &[u8], comments: bool) -> Result<String, Fault> {
    let mut copy = bytes.to_vec();
    let mut lexer = Lexer::new(bytes, Rules::strict(comments));
    loop {
        let from = lexer.at();
        lexer.skip_trivia();
        for byte in &mut copy[from..lexer.at()] {
            if !byte.is_ascii() {
                *byte = b'?';
            }
        }
        let Some(token) = lexer.token().map_err(Parser::refused)? else {
            break;
        };
        if matches!(token.kind, Kind::String { .. })
            && std::str::from_utf8(&bytes[token.range()]).is_err()
        {
            return Err(Fault::new(token.start, token.len(), FaultKind::InvalidUtf8));
        }
    }
    String::from_utf8(copy).map_err(|failure| {
        // A byte that starts no token.
        let at = failure.utf8_error().valid_up_to();
        let ch = bytes.get(at).copied().map_or('?', char::from);
        Fault::new(
            at,
            1,
            FaultKind::UnexpectedChar {
                ch,
                expected: fault::TOKEN,
            },
        )
    })
}

/// The text a JSON string literal spells, its escapes decoded; `None` when `token` is no string
/// literal.
#[must_use]
pub fn string_literal(token: &str) -> Option<String> {
    let parsed = parse(token, Options::default()).ok()?;
    Some(parsed.value.as_string()?.as_str().to_owned())
}

/// The members of the object `slice` holds, each the slice of its value (file offsets kept), in
/// the order of the first copy of a key and holding its last copy; `None` when the text is no
/// JSON or its root is no object.
#[must_use]
pub fn members_of(slice: JsonSlice<'_>) -> Option<indexmap::IndexMap<String, JsonSlice<'_>>> {
    let parsed = parse_slice(slice, Options::winner(Winner::LastInPlace)).ok()?;
    let root = parsed.spans.root();
    (root.kind() == SpanKind::Object).then(|| {
        root.members()
            .map(|member| (member.key.into_owned(), member.value.text()))
            .collect()
    })
}

/// The elements of the array `slice` holds, each the slice of its value (file offsets kept);
/// `None` when the text is no JSON or its root is no array.
#[must_use]
pub fn elements_of(slice: JsonSlice<'_>) -> Option<Vec<JsonSlice<'_>>> {
    let parsed = parse_slice(slice, Options::default()).ok()?;
    let root = parsed.spans.root();
    (root.kind() == SpanKind::Array)
        .then(|| root.elements().map(|element| element.text()).collect())
}

fn read<'t>(text: &'t str, base: usize, options: Options<'_>) -> Result<Parsed<'t>, Fault> {
    if u32::try_from(text.len()).is_err() {
        return Err(Fault::new(0, 0, FaultKind::TextTooLong));
    }
    let mut parser = Parser::new(text, options);
    parser.run()?;
    Ok(parser.finish(base))
}

/// What the parser is about to read, by the container it is in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    /// After `{` or after a comma in an object: a key or the end.
    Key,
    /// After a key and its colon: the member's value.
    Value,
    /// After a member's value: a comma or the end.
    ObjectNext,
    /// After `[` or a comma in an array: a value or the end.
    Element,
    /// After an element: a comma or the end.
    ArrayNext,
}

/// A key and where its token lies.
struct Key<'t> {
    name: Cow<'t, str>,
    offset: u32,
    len: u32,
}

/// An open object.
struct ObjectFrame<'t> {
    state: State,
    winner: Winner,
    node: u32,
    /// The first of its members in `Parser::slots`.
    first: usize,
    /// The key read, until its value is.
    key: Option<Key<'t>>,
    /// The owners of the keys by key, once the object has more members than a scan is cheap.
    owners: Option<HashMap<Cow<'t, str>, usize>>,
    /// The member of the parent object whose value this object is read into (`Winner::Keep`: a
    /// key written twice whose first value is an object takes the members of the second as they
    /// come, as facet-value's builder does).
    merging: Option<usize>,
}

/// An open array.
struct ArrayFrame {
    state: State,
    node: u32,
    /// The first of its elements in `Parser::elements`.
    first: usize,
}

enum Frame<'t> {
    Object(ObjectFrame<'t>),
    Array(ArrayFrame),
}

impl Frame<'_> {
    fn state(&self) -> State {
        match self {
            Self::Object(frame) => frame.state,
            Self::Array(frame) => frame.state,
        }
    }
}

/// A member of an open object.
struct Slot<'t> {
    key: Cow<'t, str>,
    key_offset: u32,
    key_len: u32,
    node: u32,
    /// The value the member stands for in the object; `None` when a rule dropped it or
    /// `Winner::Keep` merged it into the first copy of its key.
    value: Option<Value>,
    /// The member is the one that stands for its key.
    owner: bool,
    /// The member is in the tree of spans.
    live: bool,
}

/// The most members an object has before its keys are looked up in a map and not by a scan.
const SCAN_LIMIT: usize = 12;

struct Parser<'t, 'o> {
    scanner: Scanner<'t>,
    options: Options<'o>,
    frames: Vec<Frame<'t>>,
    slots: Vec<Slot<'t>>,
    elements: Vec<(Value, u32)>,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    /// The path of the value being read, kept only for a rule by path.
    path: Option<JsonPath>,
    /// A number the reader may need to be told of was read.
    noted: bool,
    /// Where the last value read ends.
    end: usize,
    root: Option<Value>,
}

impl<'t, 'o> Parser<'t, 'o> {
    fn new(text: &'t str, options: Options<'o>) -> Self {
        let by_path = matches!(options.duplicates, Duplicates::ByPath(_));
        Self {
            scanner: Scanner::new(text, options.comments),
            options,
            frames: Vec::new(),
            slots: Vec::new(),
            elements: Vec::new(),
            nodes: Vec::with_capacity(text.len() / 8 + 1),
            edges: Vec::with_capacity(text.len() / 8 + 1),
            path: by_path.then(JsonPath::new),
            noted: false,
            end: 0,
            root: None,
        }
    }

    /// Reads the text to the end of its root value.
    fn run(&mut self) -> Result<(), Fault> {
        while self.root.is_none() {
            let state = self.frames.last().map(Frame::state);
            let found = self.consume()?;
            match state {
                None | Some(State::Value | State::Element) => self.value_or_end(&found, state)?,
                Some(State::Key) => self.key_or_end(&found)?,
                Some(State::ObjectNext) => self.next(&found, true)?,
                Some(State::ArrayNext) => self.next(&found, false)?,
            }
        }
        Ok(())
    }

    /// The next token, with what it spells: facet-json's `consume_token`.
    fn consume(&mut self) -> Result<Mat<'t>, Fault> {
        let token = self.scanner.scan()?;
        self.scanner.materialize(token)
    }

    fn unexpected(&self, found: &Mat<'_>, expected: &'static str) -> Fault {
        Fault::new(
            found.token.start,
            found.token.len(),
            FaultKind::UnexpectedToken {
                got: found.name(self.scanner.text()),
                expected,
            },
        )
    }

    fn ended(found: &Mat<'_>, expected: &'static str) -> Fault {
        Fault::new(found.token.start, 0, FaultKind::UnexpectedEof { expected })
    }

    /// The fault of the string with a bad escape that is read as a value or a key.
    fn refused(found: UnsafeText) -> Fault {
        let span = found.span();
        let (offset, len) = (span.offset as usize, span.len as usize);
        let (ch, expected) = match found {
            UnsafeText::BadEscape { found, .. } => (found, ESCAPE),
            UnsafeText::BadUnicodeEscape {
                found: Some(found), ..
            } => (found, UNICODE_ESCAPE),
            UnsafeText::BadUnicodeEscape { found: None, .. } => {
                let kind = FaultKind::UnexpectedEof {
                    expected: UNICODE_ESCAPE,
                };
                return Fault::new(offset, len, kind);
            }
            UnsafeText::UnterminatedComment { .. } => {
                let kind = FaultKind::UnexpectedEof {
                    expected: "in block comment",
                };
                return Fault::new(offset, len, kind);
            }
            // Not what a string holds; the depth is checked where a container opens.
            UnsafeText::TooDeep { .. } => ('[', SHALLOWER),
        };
        Fault::new(offset, len, FaultKind::UnexpectedChar { ch, expected })
    }

    /// A value of the root, of an object member or of an array, or the end of the array.
    fn value_or_end(&mut self, found: &Mat<'t>, state: Option<State>) -> Result<(), Fault> {
        let in_array = state == Some(State::Element);
        match found.token.kind {
            Tk::ArrayEnd if in_array => {
                self.close_array(found.token.end);
                Ok(())
            }
            Tk::Eof if in_array => Err(Self::ended(found, "value or ']'")),
            Tk::Comma | Tk::Colon if in_array => Err(self.unexpected(found, "value or ']'")),
            _ => self.start_value(found),
        }
    }

    /// A token where a value goes.
    fn start_value(&mut self, found: &Mat<'t>) -> Result<(), Fault> {
        let (start, end) = (found.token.start, found.token.end);
        let value = match (found.token.kind, &found.lit) {
            (Tk::ObjectStart, _) => return self.open_object(start),
            (Tk::ArrayStart, _) => return self.open_array(start),
            (Tk::Null, _) => Value::NULL,
            (Tk::True, _) => Value::from(true),
            (Tk::False, _) => Value::from(false),
            (Tk::Str { .. }, Lit::Str(text)) => VString::new(text).into_value(),
            (Tk::Number(_), Lit::U64(n)) => VNumber::from_u64(*n).into_value(),
            (Tk::Number(_), Lit::I64(n)) => VNumber::from_i64(*n).into_value(),
            (Tk::Number(_), Lit::F64(n)) => VNumber::from_f64(*n).into_value(),
            (Tk::Bad(bad), _) => return Err(Self::refused(bad)),
            (Tk::Eof, _) => return Err(Self::ended(found, "value")),
            _ => {
                return Err(Fault::new(
                    start,
                    end - start,
                    FaultKind::UnexpectedToken {
                        got: found.name(self.scanner.text()),
                        expected: "value",
                    },
                ));
            }
        };
        self.begin();
        let node = self.leaf(start, end, found.note);
        self.end = end;
        self.complete(value, node);
        Ok(())
    }

    /// A token where an object has a key or ends.
    fn key_or_end(&mut self, found: &Mat<'t>) -> Result<(), Fault> {
        match (found.token.kind, found.lit.clone()) {
            (Tk::ObjectEnd, _) => {
                self.close_object(found.token.end);
                Ok(())
            }
            (Tk::Str { .. }, Lit::Str(name)) => {
                self.expect_colon()?;
                if let Some(Frame::Object(frame)) = self.frames.last_mut() {
                    frame.key = Some(Key {
                        name,
                        offset: to_u32(found.token.start),
                        len: to_u32(found.token.len()),
                    });
                    frame.state = State::Value;
                }
                Ok(())
            }
            (Tk::Eof, _) => Err(Self::ended(found, "field name or '}'")),
            (Tk::Bad(bad), _) => Err(Self::refused(bad)),
            _ => Err(self.unexpected(found, "field name or '}'")),
        }
    }

    /// The colon after a key: the token is read, not materialized.
    fn expect_colon(&mut self) -> Result<(), Fault> {
        // A block comment without its end here is told as facet-json told it: the text before
        // it ends where the colon belongs, one byte at the comment's slash.
        let colon = self.scanner.take(b':').map_err(|fault| match fault.kind {
            FaultKind::UnexpectedEof { .. } => Fault::new(
                fault.span.offset as usize,
                1,
                FaultKind::UnexpectedToken {
                    got: "Eof".to_owned(),
                    expected: "':'",
                },
            ),
            _ => fault,
        })?;
        if colon {
            return Ok(());
        }
        let token = self.scanner.scan()?;
        if token.kind == Tk::Colon {
            return Ok(());
        }
        Err(Fault::new(
            token.start,
            token.len(),
            FaultKind::UnexpectedToken {
                got: token::scan_name(self.scanner.text(), &token),
                expected: "':'",
            },
        ))
    }

    /// A token after a member or an element: a comma or the end.
    fn next(&mut self, found: &Mat<'t>, in_object: bool) -> Result<(), Fault> {
        let (expected, end) = if in_object {
            ("',' or '}'", Tk::ObjectEnd)
        } else {
            ("',' or ']'", Tk::ArrayEnd)
        };
        match found.token.kind {
            Tk::Comma => {
                let state = if in_object {
                    State::Key
                } else {
                    State::Element
                };
                match self.frames.last_mut() {
                    Some(Frame::Object(frame)) => frame.state = state,
                    Some(Frame::Array(frame)) => frame.state = state,
                    None => {}
                }
                Ok(())
            }
            kind if kind == end && in_object => {
                self.close_object(found.token.end);
                Ok(())
            }
            kind if kind == end => {
                self.close_array(found.token.end);
                Ok(())
            }
            Tk::Eof => Err(Self::ended(found, expected)),
            _ => Err(self.unexpected(found, expected)),
        }
    }

    /// A value is about to be read: its place goes on the path.
    fn begin(&mut self) {
        let Some(path) = self.path.as_mut() else {
            return;
        };
        match self.frames.last() {
            Some(Frame::Object(frame)) => {
                if let Some(key) = &frame.key {
                    path.push(&key.name);
                }
            }
            Some(Frame::Array(frame)) => path.push_index(self.elements.len() - frame.first),
            None => {}
        }
    }

    /// A node of a value that holds nothing.
    fn leaf(&mut self, start: usize, end: usize, number: Option<NumberKind>) -> u32 {
        self.noted |= number.is_some();
        self.node(SpanKind::Scalar, start, end - start, number)
    }

    fn node(
        &mut self,
        kind: SpanKind,
        start: usize,
        len: usize,
        number: Option<NumberKind>,
    ) -> u32 {
        let index = to_u32(self.nodes.len());
        self.nodes.push(Node {
            offset: to_u32(start),
            len: to_u32(len),
            kind,
            number,
            first: 0,
            count: 0,
        });
        index
    }

    fn too_deep(&self, start: usize) -> Option<Fault> {
        (self.frames.len() >= MAX_JSON_DEPTH).then(|| {
            let ch = self.scanner.text().as_bytes()[start];
            Fault::new(
                start,
                1,
                FaultKind::UnexpectedChar {
                    ch: char::from(ch),
                    expected: SHALLOWER,
                },
            )
        })
    }

    fn open_object(&mut self, start: usize) -> Result<(), Fault> {
        if let Some(fault) = self.too_deep(start) {
            return Err(fault);
        }
        self.begin();
        let winner = self.winner_here();
        let node = self.node(SpanKind::Object, start, 0, None);
        let merging = if winner == Winner::Keep {
            self.take_object_to_merge_into()
        } else {
            None
        };
        let mut frame = ObjectFrame {
            state: State::Key,
            winner,
            node,
            first: self.slots.len(),
            key: None,
            owners: None,
            merging: merging.as_ref().map(|(at, _)| *at),
        };
        if let Some((_, existing)) = merging {
            // The members already there are the object's first.
            for (key, value) in into_object(existing) {
                self.slots.push(Slot {
                    key: Cow::Owned(key.as_str().to_owned()),
                    key_offset: 0,
                    key_len: 0,
                    node: u32::MAX,
                    value: Some(value),
                    owner: true,
                    live: false,
                });
            }
            if self.slots.len() - frame.first > SCAN_LIMIT {
                frame.owners = Some(
                    self.slots[frame.first..]
                        .iter()
                        .enumerate()
                        .map(|(i, slot)| (slot.key.clone(), frame.first + i))
                        .collect(),
                );
            }
        }
        self.frames.push(Frame::Object(frame));
        Ok(())
    }

    fn open_array(&mut self, start: usize) -> Result<(), Fault> {
        if let Some(fault) = self.too_deep(start) {
            return Err(fault);
        }
        self.begin();
        let node = self.node(SpanKind::Array, start, 0, None);
        self.frames.push(Frame::Array(ArrayFrame {
            state: State::Element,
            node,
            first: self.elements.len(),
        }));
        Ok(())
    }

    /// For an object that opens as the value of a key the parent object has, under
    /// `Winner::Keep`, with an object as its value: that member's place in `slots` and the
    /// object, taken out of it.
    fn take_object_to_merge_into(&mut self) -> Option<(usize, Value)> {
        let Some(Frame::Object(parent)) = self.frames.last() else {
            return None;
        };
        if parent.winner != Winner::Keep {
            return None;
        }
        let key = parent.key.as_ref()?;
        let at = find_owner(&self.slots, parent, &key.name)?;
        let slot = &mut self.slots[at];
        slot.value.as_ref()?.is_object().then_some(())?;
        slot.value.take().map(|existing| (at, existing))
    }

    /// Which copy of a repeated key stands in the object that opens now.
    fn winner_here(&self) -> Winner {
        match self.options.duplicates {
            Duplicates::Uniform(winner) => winner,
            Duplicates::LastExcept(keep) => match self.frames.last() {
                Some(Frame::Object(parent))
                    if parent
                        .key
                        .as_ref()
                        .is_some_and(|key| keep.contains(&key.name.as_ref())) =>
                {
                    Winner::Keep
                }
                _ => Winner::Last,
            },
            Duplicates::ByPath(winner) => self.path.as_ref().map_or(Winner::Keep, winner),
        }
    }

    fn close_object(&mut self, end: usize) {
        let Some(Frame::Object(frame)) = self.frames.pop() else {
            return;
        };
        let first_edge = self.edges.len();
        let owners = self.slots[frame.first..]
            .iter()
            .filter(|slot| slot.value.is_some())
            .count();
        let mut object = VObject::with_capacity(owners);
        for slot in self.slots.drain(frame.first..) {
            if slot.live {
                self.edges.push(Edge {
                    key_offset: slot.key_offset,
                    key_len: slot.key_len,
                    node: slot.node,
                });
            }
            if let Some(value) = slot.value {
                object.insert(VString::new(&slot.key), value);
            }
        }
        let count = self.edges.len() - first_edge;
        self.finish_node(frame.node, end, first_edge, count);
        self.end = end;
        match frame.merging {
            Some(into) => self.complete_into(into, object.into_value(), frame.node),
            None => self.complete(object.into_value(), frame.node),
        }
    }

    fn close_array(&mut self, end: usize) {
        let Some(Frame::Array(frame)) = self.frames.pop() else {
            return;
        };
        let first_edge = self.edges.len();
        let mut array = VArray::with_capacity(self.elements.len() - frame.first);
        for (value, node) in self.elements.drain(frame.first..) {
            self.edges.push(Edge {
                key_offset: 0,
                key_len: 0,
                node,
            });
            array.push(value);
        }
        let count = self.edges.len() - first_edge;
        self.finish_node(frame.node, end, first_edge, count);
        self.end = end;
        self.complete(array.into_value(), frame.node);
    }

    fn finish_node(&mut self, node: u32, end: usize, first: usize, count: usize) {
        let node = &mut self.nodes[node as usize];
        node.len = to_u32(end - node.offset as usize);
        node.first = to_u32(first);
        node.count = to_u32(count);
    }

    /// An object read into the value of the member `into` of the open object: it is that
    /// member's value now, and this copy is a member of the tree of spans beside it.
    fn complete_into(&mut self, into: usize, value: Value, node: u32) {
        if let Some(path) = self.path.as_mut() {
            path.pop();
        }
        let Some(Frame::Object(frame)) = self.frames.last_mut() else {
            return;
        };
        frame.state = State::ObjectNext;
        let Some(key) = frame.key.take() else {
            return;
        };
        self.slots[into].value = Some(value);
        self.slots.push(Slot {
            key: key.name,
            key_offset: key.offset,
            key_len: key.len,
            node,
            value: None,
            owner: false,
            live: true,
        });
    }

    /// A value of the open container, or the root, is read.
    fn complete(&mut self, value: Value, node: u32) {
        if let Some(path) = self.path.as_mut()
            && !self.frames.is_empty()
        {
            path.pop();
        }
        match self.frames.last_mut() {
            None => self.root = Some(value),
            Some(Frame::Array(frame)) => {
                frame.state = State::ArrayNext;
                self.elements.push((value, node));
            }
            Some(Frame::Object(frame)) => {
                frame.state = State::ObjectNext;
                let Some(key) = frame.key.take() else {
                    return;
                };
                put_member(&mut self.slots, frame, key, value, node);
            }
        }
    }

    fn finish(self, base: usize) -> Parsed<'t> {
        let Self {
            mut scanner,
            nodes,
            edges,
            noted,
            end,
            root,
            ..
        } = self;
        let text = scanner.text();
        let trailing = scanner.content_from(end).map(|start| {
            let len = text.get(start..).map_or(0, |rest| rest.trim_end().len());
            Span {
                offset: to_u32(start.saturating_add(base)),
                len: to_u32(len),
            }
        });
        let spans = Spans::new(text, base, nodes, edges);
        let notes = if noted { spans.notes() } else { Vec::new() };
        Parsed {
            value: root.unwrap_or(Value::NULL),
            spans,
            notes,
            trailing,
        }
    }
}

/// Puts the member `key: value` into the object `frame`, by the rule for a key written twice.
fn put_member<'t>(
    slots: &mut Vec<Slot<'t>>,
    frame: &mut ObjectFrame<'t>,
    key: Key<'t>,
    value: Value,
    node: u32,
) {
    let found = find_owner(slots, frame, &key.name);
    let mut slot = Slot {
        key: key.name,
        key_offset: key.offset,
        key_len: key.len,
        node,
        value: Some(value),
        owner: true,
        live: true,
    };
    match (found, frame.winner) {
        (None, _) => {}
        (Some(old), Winner::Keep) => {
            // The first copy's value takes this one in.
            let merged = slot.value.take();
            if let (Some(into), Some(value)) = (slots[old].value.as_mut(), merged) {
                merge(into, value);
            }
            slot.owner = false;
        }
        (Some(_), Winner::First) => {
            slot.value = None;
            slot.owner = false;
            slot.live = false;
        }
        (Some(old), Winner::LastInPlace) => {
            // The first copy's place takes this one's value; both are text of the tree, and
            // a lookup by key finds the last.
            slots[old].value = slot.value.take();
            slot.owner = false;
        }
        (Some(old), _) => {
            let old = &mut slots[old];
            old.value = None;
            old.owner = false;
            old.live = false;
        }
    }
    let at = slots.len();
    if slot.owner {
        if let Some(owners) = frame.owners.as_mut() {
            owners.insert(slot.key.clone(), at);
        } else if at - frame.first >= SCAN_LIMIT {
            let mut owners: HashMap<Cow<'t, str>, usize> = slots[frame.first..]
                .iter()
                .enumerate()
                .filter(|(_, slot)| slot.owner)
                .map(|(i, slot)| (slot.key.clone(), frame.first + i))
                .collect();
            owners.insert(slot.key.clone(), at);
            frame.owners = Some(owners);
        }
    }
    slots.push(slot);
}

/// The member of the object `frame` that stands for `key`, when it has one.
fn find_owner(slots: &[Slot<'_>], frame: &ObjectFrame<'_>, key: &str) -> Option<usize> {
    if let Some(owners) = &frame.owners {
        return owners.get(key).copied();
    }
    slots[frame.first..]
        .iter()
        .position(|slot| slot.owner && slot.key == key)
        .map(|i| frame.first + i)
}

/// What facet-value's builder does with a member whose key an object already has: an object
/// is merged into the object, an array joins the array, any other value replaces what was
/// there, and an array where there is no array changes nothing.
///
/// It descends no deeper than the values it is given, which nest 128 levels at most.
fn merge(existing: &mut Value, new: Value) {
    if new.is_object() {
        let Some(target) = existing.as_object_mut() else {
            *existing = new;
            return;
        };
        for (key, member) in into_object(new) {
            match target.get_mut(key.as_str()) {
                Some(there) => merge(there, member),
                None => {
                    target.insert(key, member);
                }
            }
        }
    } else if new.is_array() {
        if let Some(target) = existing.as_array_mut() {
            for element in into_array(new) {
                target.push(element);
            }
        }
    } else {
        *existing = new;
    }
}

/// The members of an object value, by ownership.
fn into_object(value: Value) -> VObject {
    match value.destructure() {
        Destructured::Object(object) => object,
        _ => VObject::new(),
    }
}

/// The elements of an array value, by ownership.
fn into_array(value: Value) -> VArray {
    match value.destructure() {
        Destructured::Array(array) => array,
        _ => VArray::new(),
    }
}
