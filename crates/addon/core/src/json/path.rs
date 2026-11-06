//! The JSON member path of a token in a text (`path_at`), and what the walks of this crate ask
//! of a `diagnostic::JsonPath` beyond its public methods.

use crate::diagnostic::{JsonPath, JsonPathSegment};

/// What the walks of this crate ask of a [`JsonPath`] beyond its public methods.
impl JsonPath {
    /// The key that follows `prefix` in this path, when this path starts with `prefix` and
    /// continues with an object member.
    pub(crate) fn key_after(&self, prefix: &JsonPath) -> Option<&str> {
        if !self.starts_with(prefix) {
            return None;
        }
        match self.segments().nth(prefix.len())? {
            JsonPathSegment::Key(key) => Some(key),
            JsonPathSegment::Index(_) => None,
        }
    }

    /// Whether `self` names an array element of `other`'s place: the keys agree step by step,
    /// the indices may differ, and the steps `self` has beyond `other` are array indices only.
    pub(crate) fn refines(&self, other: &JsonPath) -> bool {
        self.len() >= other.len()
            && self
                .segments()
                .zip(other.segments())
                .all(|pair| match pair {
                    (JsonPathSegment::Key(a), JsonPathSegment::Key(b)) => a == b,
                    (JsonPathSegment::Index(_), JsonPathSegment::Index(_)) => true,
                    _ => false,
                })
            && self
                .segments()
                .skip(other.len())
                .all(|step| matches!(step, JsonPathSegment::Index(_)))
    }
}

/// One open container while scanning.
enum Frame {
    /// An object, with the key of the member being read (`None` before its key).
    Object(Option<String>),
    /// An array, with the index of the element being read.
    Array(usize),
}

/// The members leading to the token at byte `offset` of `text`: the value that starts there, or,
/// for a member key, that member. A lexical scan (strings, comments, brackets), not a parse; it
/// runs only to place an error facet reported, and stops at the offset.
pub(crate) fn path_at(text: &str, offset: usize) -> JsonPath {
    let bytes = text.as_bytes();
    let mut frames: Vec<Frame> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if byte == b'/' && matches!(bytes.get(i + 1), Some(b'/' | b'*')) {
            i = skip_comment(bytes, i);
            continue;
        }
        let end = token_end(bytes, i);
        let at_key = byte == b'"' && matches!(frames.last(), Some(Frame::Object(None)));
        if end > offset {
            let mut members = members_of(&frames);
            if at_key {
                members.push(&key_text(&text[i..end]));
            }
            return members;
        }
        match byte {
            b'{' => frames.push(Frame::Object(None)),
            b'[' => frames.push(Frame::Array(0)),
            b'}' | b']' => {
                frames.pop();
            }
            b',' => match frames.last_mut() {
                Some(Frame::Object(key)) => *key = None,
                Some(Frame::Array(index)) => *index += 1,
                None => {}
            },
            b'"' if at_key => {
                if let Some(Frame::Object(key)) = frames.last_mut() {
                    *key = Some(key_text(&text[i..end]));
                }
            }
            _ => {}
        }
        i = end;
    }
    members_of(&frames)
}

/// The members the open frames select.
fn members_of(frames: &[Frame]) -> JsonPath {
    let mut members = JsonPath::new();
    for frame in frames {
        match frame {
            Frame::Object(Some(key)) => members.push(key),
            Frame::Object(None) => {}
            Frame::Array(index) => members.push_index(*index),
        }
    }
    members
}

/// A key token's text: its unescaped value, or the text between the quotes when it does not
/// unescape.
fn key_text(token: &str) -> String {
    crate::json::from_str::<String>(token).unwrap_or_else(|_| token.trim_matches('"').to_owned())
}

/// The end of the token starting at `i`: a string to its closing quote, punctuation one byte,
/// anything else (numbers, literals) to the next delimiter.
fn token_end(bytes: &[u8], i: usize) -> usize {
    match bytes[i] {
        b'"' => {
            let mut j = i + 1;
            while let Some(&byte) = bytes.get(j) {
                match byte {
                    b'\\' => j += 2,
                    b'"' => return j + 1,
                    _ => j += 1,
                }
            }
            bytes.len()
        }
        b'{' | b'}' | b'[' | b']' | b',' | b':' => i + 1,
        _ => bytes[i..]
            .iter()
            .position(|&b| {
                b.is_ascii_whitespace()
                    || matches!(b, b'{' | b'}' | b'[' | b']' | b',' | b':' | b'"' | b'/')
            })
            .map_or(bytes.len(), |n| (i + n).max(i + 1)),
    }
}

/// The index after the comment starting at `i` (`//` to the line end, `/*` to `*/` or the end).
fn skip_comment(bytes: &[u8], i: usize) -> usize {
    let body = i + 2;
    if bytes.get(i + 1) == Some(&b'/') {
        bytes[body.min(bytes.len())..]
            .iter()
            .position(|&b| b == b'\n')
            .map_or(bytes.len(), |n| body + n + 1)
    } else {
        bytes
            .get(body..)
            .and_then(|rest| rest.windows(2).position(|pair| pair == b"*/"))
            .map_or(bytes.len(), |n| body + n + 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str, needle: &str) -> String {
        path_at(text, text.find(needle).unwrap()).to_string()
    }

    #[test]
    fn a_key_token_names_its_member() {
        let text = r#"{ "a": { "b": 1, "c": 2 } }"#;
        assert_eq!(at(text, r#""c""#), "a/c");
        assert_eq!(at(text, r#""a""#), "a");
    }

    #[test]
    fn a_value_token_names_the_member_holding_it() {
        let text = r#"{ "a": [10, 20, { "b": true }] }"#;
        assert_eq!(at(text, "20"), "a/1");
        assert_eq!(at(text, "true"), "a/2/b");
        assert_eq!(at(text, "{ \"b\""), "a/2");
    }

    #[test]
    fn comments_and_strings_with_brackets_are_skipped() {
        let text = "{ /* { [ */ \"a\": \"]}\", // [\n \"b\": 1 }";
        assert_eq!(at(text, "1"), "b");
    }

    #[test]
    fn an_escaped_key_is_unescaped() {
        let text = r#"{ "a\"b": 1 }"#;
        assert_eq!(at(text, "1"), "a\"b");
    }

    #[test]
    fn an_escaped_slash_in_a_key_names_one_segment() {
        let text = r#"{ "a/b": { "c~d": 1 } }"#;
        assert_eq!(at(text, "1"), "a~1b/c~0d");
    }

    #[test]
    fn key_after_reads_the_next_key_of_a_matching_prefix() {
        let full = JsonPath::of_keys(["m", "k", "v"]);
        let mut prefix = JsonPath::of_keys(["m"]);
        assert_eq!(full.key_after(&prefix), Some("k"));
        prefix.pop();
        prefix.push("x");
        assert_eq!(full.key_after(&prefix), None);
    }
}
