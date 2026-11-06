//! Members written twice in `commands.json`: the first value stands, as on the server,
//! unlike every other JSON file, where the last does (`json::last_wins`).

use std::borrow::Cow;
use std::collections::HashSet;
use std::ops::Range;

use bedrock_addon_core::json;

/// One open array or object of the scan.
struct Frame<'t> {
    /// The keys seen so far; `None` for an array.
    seen: Option<HashSet<Cow<'t, str>>>,
    /// Where the comma before the current member is.
    comma: usize,
    /// The next string is a key.
    want_key: bool,
    /// The current member repeats a key: where its blank starts.
    repeat_from: Option<usize>,
}

/// `text` with every member that repeats an earlier key of its object blanked (the comma
/// before it, the member and whatever lies before the next comma or the closing brace;
/// newlines kept), at every level. Every other byte keeps its offset. Borrowed when no object
/// repeats a key, and when the text is not well-formed JSON with comments: the parser
/// reports that. Keys compare by their value. Linear in the text.
pub(super) fn first_wins(text: &str) -> Cow<'_, str> {
    let Some(blanks) = repeats(text) else {
        return Cow::Borrowed(text);
    };
    if blanks.is_empty() {
        return Cow::Borrowed(text);
    }
    let mut out = text.as_bytes().to_vec();
    for range in blanks {
        for byte in &mut out[range] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    }
    // Whole members are blanked, and every boundary is an ASCII byte.
    String::from_utf8(out).map_or(Cow::Borrowed(text), Cow::Owned)
}

/// The ranges of the members that repeat a key; `None` when the text is not well-formed.
fn repeats(text: &str) -> Option<Vec<Range<usize>>> {
    let bytes = text.as_bytes();
    let mut blanks = Vec::new();
    let mut stack: Vec<Frame<'_>> = Vec::new();
    let mut at = 0;
    while let Some(&byte) = bytes.get(at) {
        match byte {
            b'/' => {
                at = match bytes.get(at + 1)? {
                    b'/' => text[at..].find('\n').map_or(bytes.len(), |end| at + end),
                    b'*' => at + 2 + text.get(at + 2..)?.find("*/")? + 2,
                    _ => return None,
                };
                continue;
            }
            b'{' | b'[' => stack.push(Frame {
                seen: (byte == b'{').then(HashSet::new),
                comma: at,
                want_key: byte == b'{',
                repeat_from: None,
            }),
            b'}' | b']' => {
                let frame = stack.pop()?;
                if frame.seen.is_some() != (byte == b'}') {
                    return None;
                }
                if let Some(from) = frame.repeat_from {
                    blanks.push(from..at);
                }
                if stack.is_empty() {
                    // The root is closed: what follows is not part of the value, and the
                    // reader says what it has to say about it.
                    return Some(blanks);
                }
            }
            b',' => {
                let frame = stack.last_mut()?;
                if let Some(from) = frame.repeat_from.take() {
                    blanks.push(from..at);
                }
                frame.comma = at;
                frame.want_key = frame.seen.is_some();
            }
            b'"' => {
                let end = string_end(bytes, at)?;
                if let Some(frame) = stack.last_mut()
                    && let Some(seen) = &mut frame.seen
                    && frame.want_key
                {
                    frame.want_key = false;
                    let raw = text.get(at + 1..end - 1)?;
                    let key = if raw.contains('\\') {
                        Cow::Owned(json::from_str::<String>(text.get(at..end)?).ok()?)
                    } else {
                        Cow::Borrowed(raw)
                    };
                    if !seen.insert(key) {
                        frame.repeat_from = Some(frame.comma);
                    }
                }
                at = end;
                continue;
            }
            _ => {}
        }
        at += 1;
    }
    stack.is_empty().then_some(blanks)
}

/// The offset after the closing quote of the string that opens at `start`.
fn string_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut at = start + 1;
    loop {
        match bytes.get(at)? {
            b'"' => return Some(at + 1),
            b'\\' => at += 2,
            _ => at += 1,
        }
    }
}
