//! The `.lang` byte machine and the pack keywords, which the `lang` family reads in full and
//! the `languages` family only scans for (the way the game reads `.lang` files).

use bedrock_addon_core::diagnostic::Span;

/// The two keys a pack's `.lang` files define for its own manifest: `pack.name` and
/// `pack.description`.
///
/// They never reach the translations of a language: vanilla's stack loader blocks them, and
/// reads them only as the keywords of the pack that ships them, stored under that pack's
/// key prefix.
pub const RESERVED_KEYS: [&str; 2] = ["pack.name", "pack.description"];

/// Whether a manifest header's `name` or `description` is a pack keyword: exactly `pack.name`
/// or `pack.description`, case-sensitive.
#[must_use]
pub fn is_pack_keyword(text: &str) -> bool {
    // An exact match against the reserved keys.
    RESERVED_KEYS.contains(&text)
}

/// The UTF-8 byte order mark.
pub const BOM: &[u8] = b"\xEF\xBB\xBF";

/// Where the machine is on the current line.
// The four states of the line machine.
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    /// Nothing but blanks so far.
    LineStart,
    /// Reading the key.
    Key,
    /// After the `=`: reading the value.
    Value,
    /// After the value: the rest of the line is not read.
    Comment,
    /// A line that started with `##`.
    CommentLine,
}

/// What ended a line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum End {
    /// A line feed.
    LineFeed,
    /// A NUL: one in the file, or the terminator the machine reads at the end of the file.
    Terminator,
}

/// What one line turned out to be.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Nothing but blanks.
    Blank,
    /// A line with `=` whose key or value is empty: vanilla drops it without a word. The line
    /// as written.
    Empty(Vec<u8>),
    /// A key without `=`: vanilla logs it and drops the line.
    NoEquals {
        /// The key read so far, as written.
        key: Vec<u8>,
        /// The line as written.
        raw: Vec<u8>,
        /// What ended the line.
        end: End,
    },
    /// A comment line: the bytes after its `##`.
    Comment(Vec<u8>),
    /// `key=value`, both as written, and the rest of the line from the byte that ended the
    /// value.
    Entry {
        /// The key, as written.
        key: Vec<u8>,
        /// The value, as written.
        value: Vec<u8>,
        /// The rest of the line, from the byte that ended the value.
        trailing: Vec<u8>,
    },
}

/// One line as the machine read it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    /// The line number vanilla's messages give: one plus the line feeds before the line.
    pub line: usize,
    /// The line's bytes in the file, without what ended it.
    pub span: Span,
    /// What the line is.
    pub kind: Kind,
}

/// The line being read.
#[derive(Default)]
struct Current {
    start: usize,
    key: Vec<u8>,
    value: Vec<u8>,
    trailing: Vec<u8>,
    comment: Vec<u8>,
    /// A space or a tab was skipped before the key.
    blanks: bool,
}

/// Runs the machine over `bytes`: one segment per line, in order.
#[must_use]
pub fn scan(bytes: &[u8]) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut state = State::LineStart;
    // A UTF-8 BOM at offset 0 is skipped; nothing else is decoded.
    let first = if bytes.starts_with(BOM) { 3 } else { 0 };
    let mut current = Current {
        start: first,
        ..Current::default()
    };
    // The loop runs to the index `len` inclusive: the terminator ends the last line.
    let mut skip = 0_usize;
    // The line counter starts at 1 and counts line feeds only.
    let mut number = 1_usize;
    for index in first..=bytes.len() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        let byte = bytes.get(index).copied().unwrap_or(0);
        // A carriage return is ignored everywhere, inside a value too: the test runs before
        // the state switch.
        if byte == b'\r' {
            continue;
        }
        // A NUL in the file ends the line as the terminator does: the line is stored, the
        // state resets, and the line counter does not advance.
        if byte == b'\n' || byte == 0 {
            let line = std::mem::take(&mut current);
            let end = if byte == b'\n' {
                End::LineFeed
            } else {
                End::Terminator
            };
            let kind = match state {
                // An empty key or an empty value: dropped silently.
                State::Value | State::Comment if line.key.is_empty() || line.value.is_empty() => {
                    Some(Kind::Empty(as_written(&bytes[line.start..index])))
                }
                State::Value | State::Comment => Some(Kind::Entry {
                    key: line.key,
                    value: line.value,
                    trailing: line.trailing,
                }),
                State::CommentLine => Some(Kind::Comment(line.comment)),
                State::Key => Some(Kind::NoEquals {
                    key: line.key,
                    raw: as_written(&bytes[line.start..index]),
                    end,
                }),
                // What a line feed does not end is a line only if it has blanks: carriage
                // returns, which the machine never sees, make none.
                State::LineStart if end == End::LineFeed || line.blanks => Some(Kind::Blank),
                State::LineStart => None,
            };
            if let Some(kind) = kind {
                segments.push(Segment {
                    line: number,
                    span: span(line.start, index),
                    kind,
                });
            }
            if end == End::LineFeed {
                number += 1;
            }
            state = State::LineStart;
            current.start = index + 1;
            continue;
        }
        // A `#` followed by another starts a comment. The byte after it is tested as stored,
        // like the byte after a backslash, so `#\r#` is no comment and its carriage return is
        // dropped afterwards; a `#` that is the last byte of the file has no byte after it
        // The writer uses that to write a `##` that is no comment.
        let double_pound = byte == b'#' && bytes.get(index + 1) == Some(&b'#');
        match state {
            // Spaces and tabs before the key are skipped.
            State::LineStart if byte == b' ' || byte == b'\t' => current.blanks = true,
            State::LineStart if double_pound => {
                skip = 1;
                state = State::CommentLine;
            }
            State::LineStart => {
                current.key.push(byte);
                state = State::Key;
            }
            // `=` switches to the value.
            State::Key if byte == b'=' => state = State::Value,
            State::Key => current.key.push(byte),
            // A tab ends the value, and so does `##`; the rest of the line is a comment.
            State::Value if byte == b'\t' || double_pound => {
                current.trailing.push(byte);
                state = State::Comment;
            }
            // Escapes, in a value only: `\n` is a line feed and `\\` a backslash. Before any
            // other byte, and as the last byte of the file, the backslash is kept and the byte
            // after it is read normally.
            State::Value if byte == b'\\' => match bytes.get(index + 1) {
                Some(b'n') => {
                    current.value.push(b'\n');
                    skip = 1;
                }
                Some(b'\\') => {
                    current.value.push(b'\\');
                    skip = 1;
                }
                _ => current.value.push(byte),
            },
            State::Value => current.value.push(byte),
            State::Comment => current.trailing.push(byte),
            State::CommentLine => current.comment.push(byte),
        }
    }
    segments
}

/// A line as written, without the carriage returns the machine never sees.
fn as_written(line: &[u8]) -> Vec<u8> {
    line.iter().copied().filter(|byte| *byte != b'\r').collect()
}

/// The bytes `start..end` of the file.
fn span(start: usize, end: usize) -> Span {
    let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    Span {
        offset: to_u32(start),
        len: to_u32(end.saturating_sub(start)),
    }
}

/// Whether a `.lang` file with these bytes defines `pack.name` and `pack.description`, in
/// that order: what reading the file and asking for the pack keywords would find, without reading
/// the file's other lines.
///
/// A line of the machine ends at a line feed or a NUL and depends on no other line, so only
/// the lines whose first key byte is a `p` are run through it.
#[must_use]
pub fn defines_pack_keywords(bytes: &[u8]) -> [bool; 2] {
    let bytes = bytes.strip_prefix(BOM).unwrap_or(bytes);
    let mut found = [false; 2];
    for line in bytes.split(|byte| matches!(byte, b'\n' | 0)) {
        // the first byte the machine takes for the key: past blanks and carriage returns
        let first = line
            .iter()
            .find(|byte| !matches!(byte, b' ' | b'\t' | b'\r'));
        if !first.is_some_and(|byte| byte.eq_ignore_ascii_case(&b'p')) {
            continue;
        }
        for segment in scan(line) {
            if let Kind::Entry { key, .. } = segment.kind {
                let key = key.to_ascii_lowercase();
                for (index, keyword) in RESERVED_KEYS.iter().enumerate() {
                    found[index] |= key == keyword.as_bytes();
                }
            }
        }
    }
    found
}
