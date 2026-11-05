//! `Prepared`: the gate every JSON reader runs over a file's bytes (BOM, UTF-8, nesting), and
//! what each of the readers' four meanings of "empty" is in its terms.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::borrow::Cow;

use bedrock_addon_core::json::{
    MAX_JSON_DEPTH, PrepareError, Prepared, TooDeep, Value, from_str_jsonc, strip_bom,
};

const BOM: &[u8] = b"\xEF\xBB\xBF";

#[test]
fn prepared_blanks_the_bom_and_borrows_without_one() {
    let with = Prepared::new(b"\xEF\xBB\xBF{}").unwrap();
    assert_eq!(with.as_str(), "   {}");
    assert!(matches!(with.text(), Cow::Owned(_)));
    let without = Prepared::new(b"{}").unwrap();
    assert_eq!(without.as_str(), "{}");
    assert!(matches!(without.text(), Cow::Borrowed(_)));
    // Offsets into the text are offsets into the file.
    assert_eq!(with.slice().offset, 0);
    assert_eq!(with.slice().text, "   {}");
}

#[test]
fn prepared_refuses_bytes_that_are_not_utf8() {
    assert!(matches!(
        Prepared::new(&[0xff]),
        Err(PrepareError::NotUtf8(_))
    ));
    // After a BOM too, where the text is owned.
    assert!(matches!(
        Prepared::new(b"\xEF\xBB\xBF\"\xff\""),
        Err(PrepareError::NotUtf8(_))
    ));
}

#[test]
fn prepared_refuses_nesting_past_the_limit_with_its_offset() {
    let deep = "[".repeat(MAX_JSON_DEPTH + 1);
    assert_eq!(
        Prepared::new(deep.as_bytes()).err(),
        Some(PrepareError::TooDeep(TooDeep {
            offset: MAX_JSON_DEPTH
        }))
    );
    let limit = "[".repeat(MAX_JSON_DEPTH);
    assert!(Prepared::new(limit.as_bytes()).is_ok());
}

/// Bytes that are both over-deep and not UTF-8 are told as not UTF-8: UTF-8 is checked first.
#[test]
fn prepared_checks_utf8_before_depth() {
    let mut bytes = "[".repeat(MAX_JSON_DEPTH + 1).into_bytes();
    bytes.push(0xff);
    assert!(matches!(
        Prepared::new(&bytes),
        Err(PrepareError::NotUtf8(_))
    ));
}

/// `new` never refuses empty input; `non_empty` refuses zero bytes and nothing else.
#[test]
fn only_non_empty_refuses_zero_bytes() {
    assert_eq!(Prepared::new(b"").unwrap().as_str(), "");
    assert_eq!(Prepared::non_empty(b"").err(), Some(PrepareError::Empty));
    // A BOM alone is not zero bytes.
    assert_eq!(Prepared::non_empty(BOM).unwrap().as_str(), "   ");
    assert!(Prepared::non_empty(b" ").is_ok());
    // The other checks still run.
    assert!(matches!(
        Prepared::non_empty(&[0xff]),
        Err(PrepareError::NotUtf8(_))
    ));
}

// ---- the four meanings of "empty" the readers use today -----------------------------------

/// Meaning 1 (fog, `behavior_tree`, credits, `json_ui`): zero bytes, before any BOM handling.
fn zero_bytes(bytes: &[u8]) -> bool {
    bytes.is_empty()
}

/// Meaning 2 (`texture_set`, `read.rs`): zero bytes after `strip_bom`, so a BOM alone is empty.
fn empty_after_strip(bytes: &[u8]) -> bool {
    strip_bom(bytes).is_empty()
}

/// Meaning 3 (geometry `read/mod.rs`): ASCII white space only after `strip_bom`.
fn ascii_white(bytes: &[u8]) -> bool {
    strip_bom(bytes).iter().all(u8::is_ascii_whitespace)
}

/// Meaning 4 (the envelope's `EmptyDocument`, `versioning` and loot `is_blank`): white space
/// and comments only, or `null`.
fn blank_or_null(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(strip_bom(bytes)) else {
        return false;
    };
    let null = |text: &str| from_str_jsonc::<Value>(text).is_ok_and(|v| v.is_null());
    null(text) || null(&format!("{text}\nnull"))
}

/// Each input with today's four answers and `Prepared`'s two (`non_empty` refusing it, and
/// `is_blank`). The rows where a meaning and `Prepared` disagree are the reason no gate can
/// swap one for the other without deciding it.
#[test]
fn the_four_meanings_of_empty_against_prepared() {
    // bytes, zero, after strip, ascii white, blank or null, non_empty refuses, is_blank
    #[rustfmt::skip]
    let rows: &[(&[u8], [bool; 6])] = &[
        (b"",                 [true,  true,  true,  true,  true,  true ]),
        (BOM,                 [false, true,  true,  true,  false, true ]),
        (b" ",                [false, false, true,  true,  false, true ]),
        (b"\xEF\xBB\xBF \n",  [false, false, true,  true,  false, true ]),
        (b"\t\r\n",           [false, false, true,  true,  false, true ]),
        // Form feed: ASCII white space to Rust, not JSON white space.
        (b"\x0C",             [false, false, true,  false, false, false]),
        (b"// a comment",     [false, false, false, true,  false, true ]),
        (b"/* a */ ",         [false, false, false, true,  false, true ]),
        (b"null",             [false, false, false, true,  false, false]),
        (b" null ",           [false, false, false, true,  false, false]),
        (b"{}",               [false, false, false, false, false, false]),
        (b"0",                [false, false, false, false, false, false]),
    ];
    for (bytes, expected) in rows {
        let prepared = Prepared::new(bytes).unwrap();
        let actual = [
            zero_bytes(bytes),
            empty_after_strip(bytes),
            ascii_white(bytes),
            blank_or_null(bytes),
            Prepared::non_empty(bytes) == Err(PrepareError::Empty),
            prepared.is_blank(),
        ];
        assert_eq!(&actual, expected, "{:?}", String::from_utf8_lossy(bytes));
    }
}

/// An unterminated block comment: facet-json accepts it at the end of a text, so `is_blank`
/// counts it a comment, while meaning 4 (which parses) does not.
#[test]
fn an_unterminated_block_comment_is_blank_to_prepared_only() {
    assert!(Prepared::new(b"/* open").unwrap().is_blank());
    assert!(!blank_or_null(b"/* open"));
}

/// `Prepared` does no last-wins, envelope probe or finite clamp: the text is the file's.
#[test]
fn prepared_leaves_duplicates_and_huge_numbers_as_written() {
    let text = r#"{"a": 1, "a": 2, "b": 1e999}"#;
    assert_eq!(Prepared::new(text.as_bytes()).unwrap().as_str(), text);
}
