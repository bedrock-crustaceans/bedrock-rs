//! A member of a document's root taken out of the text before the envelope is read: the root
//! `use_beta_features` of an item or an entity, which the game reads beside the version and
//! which the envelope check would count as a third member.

use std::borrow::Cow;
use std::ops::Range;

use facet_json::RawJson;
use indexmap::IndexMap;

/// `text` with its root member `key` blanked, and the member's value as written.
///
/// Where the root of `text` is an object with that member, the text returned is an owned
/// copy in which the member is blanked from its key to the end of its value, with the one
/// comma that sets it apart: the comma after the value, else, for the last member, the one
/// before the key. White space, the comments around the member and every offset are kept;
/// a comment between the key and the value goes with the member. The value is the raw text
/// of the member's value, trimmed (`true`, `"yes"`, `[1, 2]`).
///
/// The key is compared by its decoded value: a key written with an escape
/// (`"k\u0065y"` for `key`) is the member.
///
/// The text returned with a value is the text [`last_wins`](super::last_wins) gives: of a
/// member written twice the last copy stands, at EVERY level and for every key, not only
/// for `key`. A family that keeps doubled members somewhere for a rule of its own
/// (`Family::DUPLICATES_KEPT_IN`, read through
/// [`last_wins_except`](super::last_wins_except)) must not take a root member with this
/// function first: the copies it wants to see are gone from the text it is handed.
///
/// In every other case `text` is returned as it is, borrowed, with `None`: the root is no
/// object or has no such member, or the reader refuses the text (a syntax fault, a
/// byte-order mark in front, nesting past `MAX_JSON_DEPTH`). No doubled member is blanked
/// then, and the fault is left for the reader that comes next to report.
#[must_use]
pub fn take_root_member<'a>(text: &'a str, key: &str) -> (Cow<'a, str>, Option<String>) {
    let cleaned = super::last_wins(text);
    match blanked(&cleaned, key) {
        Some((blanked, written)) => (blanked.into(), Some(written)),
        None => (text.into(), None),
    }
}

/// [`take_root_member`] of a text without doubled members; `None` where nothing is taken.
fn blanked(text: &str, key: &str) -> Option<(String, String)> {
    let root = super::from_str_borrowed_jsonc::<IndexMap<String, RawJson<'_>>>(text).ok()?;
    // Where a member's value lies in `text`.
    let span = |raw: &RawJson<'_>| -> Option<Range<usize>> {
        let Cow::Borrowed(value) = raw.0 else {
            return None;
        };
        let start = (value.as_ptr() as usize).checked_sub(text.as_ptr() as usize)?;
        (start + value.len() <= text.len()).then_some(start..start + value.len())
    };
    let raw = root.get(key)?;
    let value = span(raw)?;
    let bytes = text.as_bytes();
    // The key is the first token after the comma that follows the member before, or after
    // the root's brace: found from there, a comment that holds a comma or the key's own text
    // is passed over as the comment it is.
    let before = root
        .values()
        .filter_map(span)
        .map(|other| other.end)
        .filter(|end| *end <= value.start)
        .max();
    let (comma_before, key_start) = if let Some(end) = before {
        let comma = after_trivia(bytes, end);
        (bytes.get(comma) == Some(&b',')).then_some(())?;
        (Some(comma), after_trivia(bytes, comma + 1))
    } else {
        let brace = after_trivia(bytes, 0);
        (bytes.get(brace) == Some(&b'{')).then_some(())?;
        (None, after_trivia(bytes, brace + 1))
    };
    (key_start < value.start && bytes.get(key_start) == Some(&b'"')).then_some(())?;
    let comma_after =
        Some(after_trivia(bytes, value.end)).filter(|at| bytes.get(*at) == Some(&b','));

    let mut out = bytes.to_vec();
    let mut blank = |range: Range<usize>| {
        for byte in &mut out[range] {
            if !byte.is_ascii_whitespace() {
                *byte = b' ';
            }
        }
    };
    blank(key_start..value.end);
    if let Some(comma) = comma_after.or(comma_before) {
        blank(comma..comma + 1);
    }
    let written = raw.as_str().trim().to_owned();
    Some((String::from_utf8(out).ok()?, written))
}

/// The offset of the first byte at or after `at` that is no white space and in no comment
/// (`// …` to the end of its line, `/* … */`); the length of `bytes` when there is none.
fn after_trivia(bytes: &[u8], mut at: usize) -> usize {
    loop {
        match (bytes.get(at), bytes.get(at + 1)) {
            (Some(byte), _) if byte.is_ascii_whitespace() => at += 1,
            (Some(b'/'), Some(b'/')) => {
                at = bytes[at..]
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map_or(bytes.len(), |line_end| at + line_end + 1);
            }
            (Some(b'/'), Some(b'*')) => {
                at = bytes[at + 2..]
                    .windows(2)
                    .position(|pair| pair == b"*/")
                    .map_or(bytes.len(), |close| at + 2 + close + 2);
            }
            (Some(_), _) => return at,
            (None, _) => return bytes.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The text after the member is taken, and the value it had.
    fn taken(text: &str) -> (String, Option<String>) {
        let (out, written) = take_root_member(text, "k");
        (out.into_owned(), written)
    }

    #[test]
    fn the_member_and_one_comma_are_blanked_and_every_offset_is_kept() {
        for (text, left) in [
            (r#"{"k": true, "a": 1}"#, r#"{           "a": 1}"#),
            (r#"{"a": 1, "k": true}"#, r#"{"a": 1           }"#),
            (
                r#"{"a": 1, "k": [1, 2], "b": 2}"#,
                r#"{"a": 1,              "b": 2}"#,
            ),
            (
                r#"{"k": true /* c, */, "a": 1}"#,
                r#"{          /* c, */  "a": 1}"#,
            ),
            (
                "{\"a\": 1, // x, \"k\"\n \"k\" /* \"k\" */ : 7 }",
                "{\"a\": 1  // x, \"k\"\n                   }",
            ),
            (
                r#"{"k": 1, "k": "é", "a": 1}"#,
                r#"{                   "a": 1}"#,
            ),
        ] {
            let (out, written) = taken(text);
            assert_eq!(out, left, "{text}");
            assert_eq!(out.len(), text.len(), "{text}");
            assert!(written.is_some(), "{text}");
        }
        assert_eq!(taken(r#"{"k":  "yes" }"#).1.as_deref(), Some(r#""yes""#));
    }

    /// With the member taken, the text is the last-wins text: a doubled member of another
    /// key, at another level, is blanked too, so a reader that keeps doubled members there
    /// (`last_wins_except`) no longer finds them. Without the member nothing is blanked.
    #[test]
    fn taking_the_member_blanks_every_doubled_member_of_the_text() {
        let text = r#"{"k": true, "c": {"a": 1, "a": 2}}"#;
        let kept = crate::json::last_wins_except(text, &["c"]);
        assert_eq!(kept, text, "the family's own reader keeps both copies");
        let (out, written) = taken(text);
        assert_eq!(written.as_deref(), Some("true"));
        assert_eq!(out, r#"{           "c": {        "a": 2}}"#);
        assert_eq!(crate::json::last_wins_except(&out, &["c"]), out);
        let untouched = r#"{"j": true, "c": {"a": 1, "a": 2}}"#;
        assert!(matches!(
            take_root_member(untouched, "k"),
            (Cow::Borrowed(same), None) if same == untouched
        ));
    }

    /// The key is the member's by its value, however it is spelled.
    #[test]
    fn a_key_written_with_an_escape_is_the_member() {
        let (out, written) = taken(r#"{"\u006b": 7, "a": 1}"#);
        assert_eq!(written.as_deref(), Some("7"));
        assert_eq!(out, r#"{             "a": 1}"#);
    }

    /// A text the reader refuses is handed back as it is.
    #[test]
    fn a_text_the_reader_refuses_is_left_as_it_is() {
        let deep = format!(
            r#"{{"k": true, "a": {}{}}}"#,
            "[".repeat(crate::json::MAX_JSON_DEPTH),
            "]".repeat(crate::json::MAX_JSON_DEPTH)
        );
        for text in [
            "\u{feff}{\"k\": true}",
            r#"{"k": true,, "a": 1}"#,
            r#"{"k": true, "a": 1, "a": }"#,
            deep.as_str(),
        ] {
            let (out, written) = take_root_member(text, "k");
            assert!(matches!(out, Cow::Borrowed(_)), "{text}");
            assert_eq!(out, text);
            assert_eq!(written, None, "{text}");
        }
    }

    #[test]
    fn a_text_without_the_member_at_its_root_is_left_as_it_is() {
        for text in [
            r#"{"a": {"k": true}}"#,
            r#"[{"k": true}]"#,
            r#"{"k": true"#,
            "",
        ] {
            let (out, written) = take_root_member(text, "k");
            assert!(matches!(out, Cow::Borrowed(_)), "{text}");
            assert_eq!(out, text);
            assert_eq!(written, None, "{text}");
        }
    }
}
