//! Members written twice: which copy stands, as the game's reader reads it.
//!
//! facet refuses a struct field written twice ("duplicate field") and `facet_value` merges two
//! objects of one key member by member; jsoncpp, which vanilla reads with, keeps the last
//! value, and the script server's `commands.json` reader the first. [`blank_repeats`] gives
//! the text in which every copy that does not stand is blank, so every reader sees one
//! member, and every other byte keeps its offset; [`last_wins`] and [`first_wins`] are its two
//! rules.
//!
//! The game's parser reads the whole text: a copy that loses is still text of the file, and
//! a syntax fault in it is the file's fault. No copy is blanked from the first fault of the
//! text on, so the reader reports that fault where it stands.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ops::Range;

use super::lex::{Lexer, Rules, Structure, loose_string_end};
use crate::diagnostic::JsonPath;

/// Which copy of a member written twice stands, in one object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Winner {
    /// The first copy stands: the script server's `commands.json`. A later copy is blanked
    /// from the comma before it to the comma or brace after it.
    First,
    /// The last copy stands, whole: jsoncpp's `std::map`, every other reader. An earlier copy
    /// is blanked from its key to the next member's key or the closing brace.
    Last,
    /// Every copy stays as written, for a family rule that reads them (block rule 11,
    /// "Duplicate component … found").
    Keep,
}

/// `text` with every member that does not stand blanked (newlines kept), at every level:
/// `winner` says, for the path of each object from the root, which copy of its repeated keys
/// stands. Borrowed when no object repeats a key, and also when the text is not well-formed
/// JSON(C) as the reader reads it (a member's key without its colon, a value where none goes,
/// nesting past the reader's limit): the parser reports that, and so that it does, a text with
/// a syntax fault keeps every copy that does not end before the fault. Keys compare by their
/// value (`"a"` and `"a"` are one key). One pass over the tokens, linear in the text.
pub fn blank_repeats<'t>(text: &'t str, winner: &dyn Fn(&JsonPath) -> Winner) -> Cow<'t, str> {
    let Some(mut blanks) = Scan::new(text, winner).run() else {
        return Cow::Borrowed(text);
    };
    if blanks.is_empty() {
        return Cow::Borrowed(text);
    }
    // The scan follows brackets and keys, not the grammar: a copy it would blank may hold
    // the text's syntax fault, or end where a comma is missing. Such a copy stays, and so
    // does every later one, so the reader meets the fault as the text has it. The
    // copies that end before the fault are blanked as in any text: the reader reports the
    // fault, not a member written twice before it.
    if let Some(fault) = super::read::first_fault(text) {
        blanks.retain(|range| range.end < fault);
        if blanks.is_empty() {
            return Cow::Borrowed(text);
        }
    }
    let mut out = text.as_bytes().to_vec();
    for range in blanks {
        for byte in &mut out[range] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    }
    // Only whole members are blanked, so the text stays UTF-8.
    String::from_utf8(out).map_or(Cow::Borrowed(text), Cow::Owned)
}

/// [`blank_repeats`] with the last copy standing at every level ([`Winner::Last`]): the text
/// every jsoncpp reader sees.
pub fn last_wins(text: &str) -> Cow<'_, str> {
    blank_repeats(text, &|_| Winner::Last)
}

/// [`last_wins`], except inside the object value of a member named in `keep` (at any depth),
/// whose own members are left as written ([`Winner::Keep`]): for a family rule that reads
/// them (block rule 11, "Duplicate component … found").
pub fn last_wins_except<'t>(text: &'t str, keep: &[&str]) -> Cow<'t, str> {
    blank_repeats(text, &|path: &JsonPath| {
        if path.last_key().is_some_and(|key| keep.contains(&key)) {
            Winner::Keep
        } else {
            Winner::Last
        }
    })
}

/// [`blank_repeats`] with the first copy standing at every level ([`Winner::First`]): the
/// script server's reading of `commands.json`.
pub fn first_wins(text: &str) -> Cow<'_, str> {
    blank_repeats(text, &|_| Winner::First)
}

/// The deepest nesting the scan follows: the limit of the readers. A deeper text is left as
/// it is for the reader, which refuses it.
const MAX_DEPTH: usize = super::MAX_JSON_DEPTH;

/// One object the scan is in, with its members so far.
struct Object<'t> {
    winner: Winner,
    members: Vec<Member<'t>>,
    /// The last comma, or the `{`.
    comma: usize,
}

/// One member of an object the scan is in.
struct Member<'t> {
    /// Its key's value.
    name: Cow<'t, str>,
    /// Where its key starts.
    start: usize,
    /// The last comma before it; the object's `{` when none is.
    comma_before: usize,
    /// The first comma after it, once the scan is past one.
    comma_after: Option<usize>,
}

/// The scan's frames: an object with its members, or an array.
enum Open<'t> {
    Object(Object<'t>),
    Array,
}

/// The walk of [`blank_repeats`]: the reader's grammar as far as members go (a key, its colon,
/// a value; commas anywhere between them), any other value a run of bytes to the next
/// `,`, `}`, `]`, white space or `/`.
struct Scan<'t, 'w> {
    text: &'t str,
    lexer: Lexer<'t>,
    winner: &'w dyn Fn(&JsonPath) -> Winner,
    open: Vec<Open<'t>>,
    path: Structure,
    blanks: Vec<Range<usize>>,
}

impl<'t, 'w> Scan<'t, 'w> {
    fn new(text: &'t str, winner: &'w dyn Fn(&JsonPath) -> Winner) -> Self {
        Self {
            text,
            lexer: Lexer::new(text.as_bytes(), Rules::lenient(false)),
            winner,
            open: Vec::new(),
            path: Structure::new(),
            blanks: Vec::new(),
        }
    }

    /// The next byte after white space and comments; `None` at the end of the text or in a
    /// `/*` comment without an end.
    fn peek(&mut self) -> Option<u8> {
        if self.lexer.skip_trivia().is_some() {
            return None;
        }
        self.text.as_bytes().get(self.lexer.at()).copied()
    }

    /// The ranges to blank; `None` for a text that is not well-formed.
    fn run(mut self) -> Option<Vec<Range<usize>>> {
        self.value()?;
        while !self.open.is_empty() {
            let byte = self.peek()?;
            let at = self.lexer.at();
            match (self.open.last_mut()?, byte) {
                (Open::Object(_), b'}') => {
                    let Some(Open::Object(object)) = self.open.pop() else {
                        return None;
                    };
                    self.path.close();
                    self.lexer.seek(at + 1);
                    self.blank(&object, at);
                }
                (Open::Array, b']') => {
                    self.open.pop();
                    self.path.close();
                    self.lexer.seek(at + 1);
                }
                (Open::Object(object), b',') => {
                    object.comma = at;
                    if let Some(member) = object.members.last_mut() {
                        member.comma_after.get_or_insert(at);
                    }
                    self.path.comma();
                    self.lexer.seek(at + 1);
                }
                (Open::Array, b',') => {
                    self.path.comma();
                    self.lexer.seek(at + 1);
                }
                (Open::Object(_), b'"') => self.member()?,
                (Open::Object(_), _) => return None,
                (Open::Array, _) => self.value()?,
            }
        }
        Some(self.blanks)
    }

    /// A member of the innermost object, from its key: the key, its colon, its value.
    fn member(&mut self) -> Option<()> {
        let key = self.string()?;
        let raw = &self.text[key.clone()];
        let name = if raw.contains('\\') {
            Cow::Owned(crate::json::from_str::<String>(raw).ok()?)
        } else {
            Cow::Borrowed(&raw[1..raw.len() - 1])
        };
        if self.peek()? != b':' {
            return None;
        }
        self.lexer.seek(self.lexer.at() + 1);
        self.path.leave_key();
        self.path.enter_key(&name);
        let Some(Open::Object(object)) = self.open.last_mut() else {
            return None;
        };
        let comma_before = object.comma;
        object.members.push(Member {
            name,
            start: key.start,
            comma_before,
            comma_after: None,
        });
        self.value()
    }

    /// One value from the current position: an object or array is opened, a string or a
    /// run of bytes read whole.
    fn value(&mut self) -> Option<()> {
        let byte = self.peek()?;
        let at = self.lexer.at();
        match byte {
            b'{' | b'[' if self.open.len() >= MAX_DEPTH => None,
            b'{' => {
                let winner = (self.winner)(self.path.path());
                self.open.push(Open::Object(Object {
                    winner,
                    members: Vec::new(),
                    comma: at,
                }));
                self.path.open_object();
                self.lexer.seek(at + 1);
                Some(())
            }
            b'[' => {
                self.open.push(Open::Array);
                self.path.open_array();
                self.lexer.seek(at + 1);
                Some(())
            }
            b'"' => self.string().map(|_| ()),
            _ => {
                let run = self.text.as_bytes()[at..]
                    .iter()
                    .take_while(|byte| {
                        !matches!(
                            byte,
                            b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r' | b'/'
                        )
                    })
                    .count();
                self.lexer.seek(at + run);
                (run > 0).then_some(())
            }
        }
    }

    /// A string at the current position: its text with the quotes. `None` when it has no
    /// closing quote.
    fn string(&mut self) -> Option<Range<usize>> {
        let start = self.lexer.at();
        let (end, closed) = loose_string_end(self.text.as_bytes(), start);
        self.lexer.seek(end);
        closed.then_some(start..end)
    }

    /// The members of `object`, which closes at `close`, that do not stand.
    fn blank(&mut self, object: &Object<'t>, close: usize) {
        let members = &object.members;
        match object.winner {
            Winner::Keep => {}
            Winner::Last => {
                // The last index of each key, in one pass: every member before its key's last
                // is a copy to blank, to the next member's key (linear in the members).
                let mut last: HashMap<&str, usize> = HashMap::with_capacity(members.len());
                for (index, member) in members.iter().enumerate() {
                    last.insert(&member.name, index);
                }
                if last.len() == members.len() {
                    return;
                }
                for (index, member) in members.iter().enumerate() {
                    if last
                        .get(&*member.name)
                        .is_some_and(|&final_index| final_index > index)
                    {
                        let end = members.get(index + 1).map_or(close, |next| next.start);
                        self.blanks.push(member.start..end);
                    }
                }
            }
            Winner::First => {
                // A copy after the first: from the comma before it to the comma after it, or
                // the closing brace.
                let mut seen: HashSet<&str> = HashSet::with_capacity(members.len());
                for member in members {
                    if !seen.insert(&member.name) {
                        let end = member.comma_after.unwrap_or(close);
                        self.blanks.push(member.comma_before..end);
                    }
                }
            }
        }
    }
}
