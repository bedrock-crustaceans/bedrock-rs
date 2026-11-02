//! Writing a `.lang` file back, line by line (round-trip level Lines).

use super::{LangFile, Language, Line};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{EncodeError, Unrepresentable};

/// The bytes no part of a line can hold: a line feed and a NUL end the line, and a carriage
/// return is never read.
const LINE_BREAKERS: [char; 3] = ['\n', '\r', '\0'];

/// The byte order mark as a character.
const BOM: char = '\u{feff}';

impl LangFile {
    /// Writes the file as `.lang` text: one line per [`Line`], each ended by a line feed, the
    /// value of an entry escaped (`\n` for a line feed, `\\` where a backslash would otherwise
    /// read as an escape). A file that was read is written with the byte order mark it had,
    /// and without a final line feed if it had none. On failure nothing is appended to `out`.
    ///
    /// A carriage return, which the reader ignores, is written in the two places where the
    /// text would otherwise read as something else: between two `#` that are not a comment
    /// (in a value, or at the start of a key), and before a first line that starts with
    /// U+FEFF in a file without a byte order mark.
    ///
    /// # Errors
    ///
    /// Fails for a line the format cannot hold: a key that is empty, starts with a blank, or
    /// holds `=` after its first byte; an empty value; a tab, a carriage return or a NUL in a
    /// value; text after a value that starts with neither a tab nor `##`; a line feed outside
    /// a value; a [`Line::Dropped`] whose text the reader would not read as one dropped line
    /// (empty or only blanks, or text with a key, `=` and a value).
    pub fn write_text(&self, out: &mut Vec<u8>) -> Result<(), EncodeError> {
        let mut text = String::new();
        if self.layout.bom {
            text.push(BOM);
        } else if self.lines.first().is_some_and(starts_with_bom) {
            // As the first bytes of the file the reader would skip the character as the
            // file's byte order mark; after a carriage return it is a key byte.
            text.push('\r');
        }
        let last = self.lines.len().saturating_sub(1);
        for (index, line) in self.lines.iter().enumerate() {
            if !write_line(line, &mut text) {
                let mut at = JsonPath::new();
                at.push_index(index);
                if let Line::Entry { key, .. } = line {
                    at.push(key);
                }
                return Err(EncodeError::NotRepresentable {
                    at,
                    what: Unrepresentable::Unsupported,
                });
            }
            // The last line is ended unless the file was read without its final line feed; an
            // empty last line is only a line with one.
            if index < last || self.layout.final_line_feed || matches!(line, Line::Blank) {
                text.push('\n');
            }
        }
        out.extend_from_slice(text.as_bytes());
        Ok(())
    }

    /// [`write_text`](Self::write_text) into a string.
    ///
    /// # Errors
    ///
    /// Returns the error [`write_text`](Self::write_text) returns for a line the format cannot hold.
    pub fn to_text(&self) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        self.write_text(&mut out)?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }
}

/// The line's text starts with U+FEFF: as the first bytes of a file, the reader skips it.
fn starts_with_bom(line: &Line) -> bool {
    match line {
        Line::Entry { key, .. } => key.starts_with(BOM),
        Line::Dropped(text) => text.starts_with(BOM),
        Line::Blank | Line::Comment(_) => false,
    }
}

/// Appends `line` without its line feed; `false` when the reader would not read the written
/// text back as this line.
fn write_line(line: &Line, out: &mut String) -> bool {
    match line {
        Line::Blank => true,
        Line::Comment(text) => {
            out.push_str("##");
            out.push_str(text);
            !text.contains(LINE_BREAKERS)
        }
        Line::Dropped(text) => {
            // A line vanilla drops is no comment: past its blanks, a leading `##` is the start
            // of a key.
            let key = text.trim_start_matches([' ', '\t']);
            let mut written = text[..text.len() - key.len()].to_owned();
            push_line_start(key, &mut written);
            out.push_str(&written);
            reads_as_dropped(&written, text)
        }
        Line::Entry {
            key,
            value,
            trailing,
        } => {
            push_line_start(key, out);
            out.push('=');
            escape(value, out);
            // `v#` right before a `##` comment would start the comment one byte early
            if value.ends_with('#') && trailing.starts_with('#') {
                out.push('\r');
            }
            out.push_str(trailing);
            key_is_writable(key) && value_is_writable(value) && trailing_is_writable(trailing)
        }
    }
}

/// Whether the reader reads `written` as the one dropped line `text`: the reader is the
/// oracle of what vanilla drops. Text it reads as a blank line or as an entry, or that spans
/// lines, is no dropped line.
fn reads_as_dropped(written: &str, text: &str) -> bool {
    // After a line feed, so that a leading U+FEFF is not taken for the file's byte order mark.
    let probe = format!("\n{written}");
    let (read, _) = LangFile::from_bytes(Language::new(""), probe.as_bytes());
    matches!(read.lines.as_slice(), [Line::Blank, Line::Dropped(dropped)] if dropped == text)
}

/// Appends the text a line starts with, a key or a dropped line past its blanks: a leading
/// `##` that is no comment is written `#\r#`. The reader tests the byte right after a `#` as
/// stored and drops the carriage return afterwards.
fn push_line_start(text: &str, out: &mut String) {
    match text.strip_prefix("##") {
        Some(rest) => {
            out.push_str("#\r#");
            out.push_str(rest);
        }
        None => out.push_str(text),
    }
}

/// A key the reader reads back: it starts the line (no blank), `=` ends it (so only its first
/// byte may be `=`), and it stays on its line.
fn key_is_writable(key: &str) -> bool {
    !key.is_empty()
        && !key.starts_with([' ', '\t'])
        && !key.chars().skip(1).any(|c| c == '=')
        && !key.contains(LINE_BREAKERS)
}

/// A value the reader reads back: not empty (the line would be dropped), and without the
/// bytes that end a value or never reach it. A line feed is written as its escape.
fn value_is_writable(value: &str) -> bool {
    !value.is_empty() && !value.contains('\t') && !value.contains(['\r', '\0'])
}

/// What may follow a value: nothing, or text from the tab or the `##` that ends the value.
fn trailing_is_writable(trailing: &str) -> bool {
    (trailing.is_empty() || trailing.starts_with('\t') || trailing.starts_with("##"))
        && !trailing.contains(LINE_BREAKERS)
}

/// Appends `value` with a line feed as `\n`, a backslash doubled where the byte written
/// after it would otherwise make an escape (`n`, or another backslash), and a carriage return
/// between two `#`, which would otherwise start a comment.
fn escape(value: &str, out: &mut String) {
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\n' => out.push_str("\\n"),
            '\\' if matches!(chars.peek(), Some('n' | '\\' | '\n')) => out.push_str("\\\\"),
            '#' if chars.peek() == Some(&'#') => out.push_str("#\r"),
            _ => out.push(c),
        }
    }
}
