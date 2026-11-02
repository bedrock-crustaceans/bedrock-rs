//! Members written twice: the last value stands, whole, at every level, as the game's
//! reader reads it.
//!
//! facet refuses a struct field written twice ("duplicate field") and `facet_value` merges two
//! objects of one key member by member; jsoncpp, which vanilla reads with, keeps the last
//! value. [`last_wins`] gives the text in which every earlier copy is blank, so every reader
//! sees one member, the last, and every other byte keeps its offset.
//!
//! The game's parser reads the whole text: a copy that loses is still text of the file, and
//! a syntax fault in it is the file's fault. No copy is blanked from the first fault of the
//! text on, so the reader reports that fault where it stands.

use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;

/// The deepest nesting the scan follows, a call of itself for each level: the limit of the
/// readers. A deeper text is left as it is for the reader, which refuses it; followed
/// further, it took the stack of a small thread before the reader saw it.
const MAX_DEPTH: usize = super::MAX_JSON_DEPTH;

/// `text` with every member that its object repeats later blanked (the member, its comma and
/// whatever lies before the next member, newlines kept), at every level. Borrowed when no
/// object repeats a key, and also when the text is not well-formed JSON(C): the parser reports
/// that, and so that it does, a text with a syntax fault keeps every copy that does not end
/// before the fault. Keys compare by their value (`"a"` and `"a"` are one key).
pub fn last_wins(text: &str) -> Cow<'_, str> {
    last_wins_except(text, &[])
}

/// [`last_wins`], except inside the object value of a member named in `keep` (at any depth),
/// whose own members are left as written: for a family rule that reads them (block rule 11,
/// "Duplicate component … found").
pub fn last_wins_except<'t>(text: &'t str, keep: &[&str]) -> Cow<'t, str> {
    let mut scan = Scan {
        bytes: text.as_bytes(),
        text,
        at: 0,
        blanks: Vec::new(),
        keep,
    };
    if scan.value(0, false).is_none() || scan.blanks.is_empty() {
        return Cow::Borrowed(text);
    }
    // The scan follows brackets and keys, not the grammar: a copy it would blank may hold
    // the text's syntax fault, or end where a comma is missing. Such a copy stays, and so
    // does every later one, so the reader meets the fault as the text has it. The
    // copies that end before the fault are blanked as in any text: the reader reports the
    // fault, not a member written twice before it.
    let mut blanks = scan.blanks;
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

/// The scan's position and the ranges to blank.
struct Scan<'a> {
    bytes: &'a [u8],
    text: &'a str,
    at: usize,
    blanks: Vec<Range<usize>>,
    keep: &'a [&'a str],
}

impl Scan<'_> {
    /// Skips white space and comments.
    fn skip(&mut self) -> Option<()> {
        loop {
            match self.bytes.get(self.at..)? {
                [b' ' | b'\t' | b'\n' | b'\r', ..] => self.at += 1,
                [b'/', b'/', ..] => {
                    let rest = &self.bytes[self.at..];
                    self.at += rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
                }
                [b'/', b'*', ..] => {
                    let rest = &self.text[self.at + 2..];
                    self.at += 2 + rest.find("*/")? + 2;
                }
                _ => return Some(()),
            }
        }
    }

    /// One value from the current position; an object keeps its duplicates when `keep`.
    fn value(&mut self, depth: usize, keep: bool) -> Option<()> {
        self.skip()?;
        match *self.bytes.get(self.at)? {
            b'{' | b'[' if depth >= MAX_DEPTH => None,
            b'{' => self.object(depth, keep),
            b'[' => self.array(depth),
            b'"' => self.string().map(|_| ()),
            _ => {
                let start = self.at;
                while self.bytes.get(self.at).is_some_and(|b| {
                    !matches!(b, b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r' | b'/')
                }) {
                    self.at += 1;
                }
                (self.at > start).then_some(())
            }
        }
    }

    /// A string; its text with the quotes.
    fn string(&mut self) -> Option<Range<usize>> {
        let start = self.at;
        self.at += 1;
        loop {
            match *self.bytes.get(self.at)? {
                b'"' => {
                    self.at += 1;
                    return Some(start..self.at);
                }
                b'\\' => self.at += 2,
                _ => self.at += 1,
            }
        }
    }

    fn array(&mut self, depth: usize) -> Option<()> {
        self.at += 1;
        loop {
            self.skip()?;
            match *self.bytes.get(self.at)? {
                b']' => {
                    self.at += 1;
                    return Some(());
                }
                b',' => self.at += 1,
                _ => self.value(depth + 1, false)?,
            }
        }
    }

    fn object(&mut self, depth: usize, keep: bool) -> Option<()> {
        self.at += 1;
        // Each member: its key's value and where it starts.
        let mut members: Vec<(Cow<'_, str>, usize)> = Vec::new();
        let close = loop {
            self.skip()?;
            match *self.bytes.get(self.at)? {
                b'}' => break self.at,
                b',' => self.at += 1,
                b'"' => {
                    let key = self.string()?;
                    let raw = &self.text[key.clone()];
                    let name = if raw.contains('\\') {
                        Cow::Owned(crate::json::from_str::<String>(raw).ok()?)
                    } else {
                        Cow::Borrowed(&raw[1..raw.len() - 1])
                    };
                    self.skip()?;
                    if self.bytes.get(self.at) != Some(&b':') {
                        return None;
                    }
                    self.at += 1;
                    let kept = self.keep.contains(&&*name);
                    members.push((name, key.start));
                    self.value(depth + 1, kept)?;
                }
                _ => return None,
            }
        };
        self.at = close + 1;
        if keep {
            return Some(());
        }
        // The last index of each key, in one pass: every member before its key's last is a
        // copy to blank (linear in the members).
        let mut last: HashMap<&str, usize> = HashMap::with_capacity(members.len());
        for (index, (name, _)) in members.iter().enumerate() {
            last.insert(name, index);
        }
        if last.len() == members.len() {
            return Some(());
        }
        for (index, (name, start)) in members.iter().enumerate() {
            if last
                .get(&**name)
                .is_some_and(|&final_index| final_index > index)
            {
                let end = members.get(index + 1).map_or(close, |(_, next)| *next);
                self.blanks.push(*start..end);
            }
        }
        Some(())
    }
}
