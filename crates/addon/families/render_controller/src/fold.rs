//! The case folding the game applies to a render controller file's text before it reads it as
//! JSON.

/// A file's text after folding, and the spans the fold left alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Folded {
    /// The folded text: the same bytes but for `A`–`Z` outside the quoted spans.
    pub(crate) text: String,
    /// Each quoted span as (the offset of the apostrophe that opened it, the offset one past
    /// its end): the apostrophe that closed it, or the end of the text.
    pub(crate) quoted: Vec<(usize, usize)>,
}

impl Folded {
    /// The offset of the apostrophe whose quoted span holds `offset`, if one does.
    pub(crate) fn quote_before(&self, offset: usize) -> Option<usize> {
        self.quoted
            .iter()
            .find(|(start, end)| *start < offset && offset < *end)
            .map(|(start, _)| *start)
    }
}

/// Folds `text` as the game does, byte by byte: an apostrophe toggles a quoted span and is
/// kept; a backslash is kept and the byte after it is skipped (neither folded nor able to
/// toggle); outside a quoted span `A`–`Z` become `a`–`z`; every other byte is unchanged.
pub(crate) fn fold(text: &str) -> Folded {
    let mut bytes = text.as_bytes().to_vec();
    let mut quoted = Vec::new();
    let mut open: Option<usize> = None;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' => match open.take() {
                Some(start) => quoted.push((start, index)),
                None => open = Some(index),
            },
            b'\\' => index += 1,
            byte if open.is_none() => bytes[index] = byte.to_ascii_lowercase(),
            _ => {}
        }
        index += 1;
    }
    if let Some(start) = open {
        quoted.push((start, bytes.len()));
    }
    Folded {
        // Only ASCII letters changed, so the bytes are still the text's UTF-8.
        text: String::from_utf8(bytes).unwrap_or_else(|_| text.to_owned()),
        quoted,
    }
}

/// JSON text in which the game's fold changes nothing: every ASCII capital the fold would
/// lower (outside an apostrophe span, not the byte after a backslash) becomes a `\u00xx`
/// escape, whose `u` the fold skips and whose digits hold no capital. `text` is JSON as a
/// writer emits it (no comments), so a capital the fold would lower is inside a string, and
/// both readers decode the escape to the capital.
pub(crate) fn protect(text: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    let mut quoted = false;
    let mut index = 0;
    while index < text.len() {
        let byte = text[index];
        match byte {
            b'\'' => {
                quoted = !quoted;
                out.push(byte);
            }
            b'\\' => {
                out.push(byte);
                index += 1;
                let Some(&escaped) = text.get(index) else {
                    break;
                };
                out.push(escaped);
                if escaped == b'u' {
                    // The four hex digits of an escape the writer made: the fold may lower
                    // them, which names the same character.
                    let end = (index + 5).min(text.len());
                    out.extend(text[index + 1..end].iter().map(u8::to_ascii_lowercase));
                    index = end - 1;
                }
            }
            b'A'..=b'Z' if !quoted => out.extend(format!("\\u{:04x}", u32::from(byte)).bytes()),
            _ => out.push(byte),
        }
        index += 1;
    }
    out
}

/// The text of a render controller file as the game reads it: ASCII upper-case letters become
/// lower case everywhere except between apostrophes (Molang string literals). An apostrophe
/// anywhere in the file counts, also in a comment or in a JSON string that is no Molang, and a
/// backslash protects the one byte after it; text after an unpaired apostrophe keeps its case
/// to the end of the file.
///
/// ```
/// use bedrock_addon_family_render_controller::fold_case;
/// assert_eq!(fold_case(r#"{"Geometry": "Geometry.Default"}"#), r#"{"geometry": "geometry.default"}"#);
/// assert_eq!(fold_case("Q.Get_Name == 'Toast'"), "q.get_name == 'Toast'");
/// ```
pub fn fold_case(text: &str) -> String {
    fold(text).text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_outside_apostrophes_are_folded() {
        assert_eq!(
            fold("Array.Skins[Q.Variant]").text,
            "array.skins[q.variant]"
        );
    }

    #[test]
    fn a_quoted_span_keeps_its_case_and_its_apostrophes() {
        let folded = fold("A 'Toast' B");
        assert_eq!(folded.text, "a 'Toast' b");
        assert_eq!(folded.quoted, vec![(2, 8)]);
    }

    #[test]
    fn an_unpaired_apostrophe_stops_folding_to_the_end() {
        let folded = fold("A bob's HAT");
        assert_eq!(folded.text, "a bob's HAT");
        assert_eq!(folded.quoted, vec![(5, 11)]);
        assert_eq!(folded.quote_before(8), Some(5));
        assert_eq!(folded.quote_before(5), None);
        assert_eq!(folded.quote_before(2), None);
    }

    #[test]
    fn a_backslash_skips_the_next_byte() {
        // The escaped apostrophe does not toggle and the escaped letter keeps its case.
        assert_eq!(fold(r"A\'B \QC").text, r"a\'b \Qc");
        assert!(fold(r"A\'B").quoted.is_empty());
    }

    #[test]
    fn a_trailing_backslash_ends_the_text() {
        assert_eq!(fold("AB\\").text, "ab\\");
    }

    #[test]
    fn bytes_outside_ascii_are_unchanged() {
        assert_eq!(fold("ÄB é").text, "Äb é");
    }

    /// `text` with `~` standing for a backslash (the escapes are easier to read that way).
    fn bs(text: &str) -> String {
        text.replace('~', &char::from(92_u8).to_string())
    }

    fn protected(text: &str) -> String {
        String::from_utf8(protect(bs(text).as_bytes())).unwrap()
    }

    #[test]
    fn protect_escapes_the_capitals_the_fold_would_lower() {
        assert_eq!(
            protected("\"Foo 'Bar' Baz\""),
            bs("\"~u0046oo 'Bar' ~u0042az\"")
        );
        // An unpaired apostrophe protects everything after it, as it does for the fold.
        assert_eq!(protected("\"bob's Hat\", \"B\""), "\"bob's Hat\", \"B\"");
        assert_eq!(protected("\"lower_9.:*\""), "\"lower_9.:*\"");
    }

    #[test]
    fn protect_keeps_the_writer_s_escapes_whole() {
        // The hex digits of an escape are lowered (the fold would lower them anyway), not
        // escaped; an escaped apostrophe-free backslash pair is followed by an ordinary byte.
        assert_eq!(protected("\"~u00E9A\""), bs("\"~u00e9~u0041\""));
        assert_eq!(protected("\"~~A~n~\"B\""), bs("\"~~~u0041~n~\"~u0042\""));
        assert_eq!(protected("\"x~"), bs("\"x~"));
        assert_eq!(protected("\"~u00"), bs("\"~u00"));
    }

    #[test]
    fn the_fold_changes_nothing_in_protected_text() {
        for text in [
            "\"Foo 'Bar' Baz\"",
            "\"A'B\", \"C'D'E\"",
            "\"~u00C9~~Q~\"R'S\"",
        ] {
            let protected = protected(text);
            assert_eq!(fold(&protected).text, protected, "{text}");
        }
    }
}
