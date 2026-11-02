//! The pre-parse helpers: `strip_bom` and `check_depth`: the game skips a leading UTF-8 BOM, and a
//! document nested deeper than the limit is refused before any parser sees it.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon_core::json::{MAX_JSON_DEPTH, TooDeep, blank_bom, check_depth, strip_bom};

#[test]
fn strip_bom_removes_a_leading_utf8_bom() {
    assert_eq!(strip_bom(b"\xEF\xBB\xBF{}"), b"{}");
}

#[test]
fn strip_bom_keeps_text_without_a_bom() {
    assert_eq!(strip_bom(b"{}"), b"{}");
    assert_eq!(strip_bom(b""), b"");
}

#[test]
fn strip_bom_removes_only_one_bom() {
    assert_eq!(strip_bom(b"\xEF\xBB\xBF\xEF\xBB\xBF{}"), b"\xEF\xBB\xBF{}");
}

#[test]
fn strip_bom_ignores_a_bom_after_the_start() {
    assert_eq!(strip_bom(b" \xEF\xBB\xBF{}"), b" \xEF\xBB\xBF{}");
}

/// A BOM followed by `//` and `/* */` comments, as vanilla's chemistry manifests are written:
/// read through `strip_bom`, `check_depth` and the jsonc entry point.
#[test]
fn a_bom_and_comments_read_through_the_pre_parse_helpers() {
    let path = format!(
        "{}/tests/fixtures/json/bom_comments.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let bytes = std::fs::read(path).unwrap();
    assert!(facet_json::from_slice_borrowed_jsonc::<facet_value::Value>(&bytes).is_err());
    let text = strip_bom(&bytes);
    assert_eq!(check_depth(text, MAX_JSON_DEPTH), Ok(()));
    let value = facet_json::from_slice_borrowed_jsonc::<facet_value::Value>(text).unwrap();
    let object = value.as_object().unwrap();
    assert_eq!(
        object
            .get("a")
            .and_then(|a| a.as_number())
            .and_then(|n| n.to_i64()),
        Some(1)
    );
}

#[test]
fn max_json_depth_is_128() {
    assert_eq!(MAX_JSON_DEPTH, 128);
}

fn nested(levels: usize) -> String {
    let mut text = "[".repeat(levels);
    text.push_str(&"]".repeat(levels));
    text
}

#[test]
fn check_depth_accepts_128_levels() {
    assert_eq!(check_depth(nested(128).as_bytes(), MAX_JSON_DEPTH), Ok(()));
}

#[test]
fn check_depth_rejects_129_levels_at_the_129th_bracket() {
    assert_eq!(
        check_depth(nested(129).as_bytes(), MAX_JSON_DEPTH),
        Err(TooDeep { offset: 128 })
    );
}

#[test]
fn check_depth_counts_objects_and_arrays_together() {
    let text = r#"{"a":[{"b":[1]}]}"#;
    assert_eq!(check_depth(text.as_bytes(), 4), Ok(()));
    assert_eq!(check_depth(text.as_bytes(), 3), Err(TooDeep { offset: 11 }));
}

#[test]
fn check_depth_skips_brackets_inside_strings() {
    let text = r#"["[[[[", "\"[[[", "\\", ["]"]]"#;
    assert_eq!(check_depth(text.as_bytes(), 2), Ok(()));
}

#[test]
fn check_depth_skips_brackets_inside_comments() {
    let text = "[ // [[[[\n /* [[[[ */ [ ] ]";
    assert_eq!(check_depth(text.as_bytes(), 2), Ok(()));
}

#[test]
fn check_depth_skips_an_unterminated_block_comment_to_the_end() {
    assert_eq!(check_depth(b"[] /* [[[[", 1), Ok(()));
}

#[test]
fn check_depth_lets_unbalanced_closers_through_for_the_parser() {
    // The pre-scan only bounds nesting; syntax errors are facet's to report.
    assert_eq!(check_depth(b"]]]{", 1), Ok(()));
}

/// The reason for the limit: facet-json has none, and a deep document overflows the stack.
/// At 128 levels a parse of the text succeeds on a 2 MiB thread (the probe saw the overflow
/// between 150 and 200 levels in a debug build).
#[test]
fn a_document_at_the_limit_parses_on_a_2_mib_thread() {
    let text = nested(MAX_JSON_DEPTH);
    let parsed = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            check_depth(text.as_bytes(), MAX_JSON_DEPTH)
                .map(|()| facet_json::from_str_borrowed_jsonc::<facet_value::Value>(&text).is_ok())
        })
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(parsed, Ok(true));
}

#[test]
fn a_document_over_the_limit_is_rejected_before_parsing_on_a_2_mib_thread() {
    let text = nested(MAX_JSON_DEPTH + 1);
    let checked = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || check_depth(text.as_bytes(), MAX_JSON_DEPTH))
        .unwrap()
        .join()
        .unwrap();
    assert!(matches!(checked, Err(TooDeep { offset: 128 })));
}

/// Vanilla's reader stops at the end of the root value: what follows is not the
/// document's nesting, however deep it is.
#[test]
fn check_depth_stops_at_the_end_of_the_root_value() {
    let pit = "[".repeat(MAX_JSON_DEPTH + 50);
    for text in [
        format!("{{}} {pit}"),
        format!("[[1], {{\"a\": [2]}}]\n{pit}"),
        format!("// [\n /* {{ */ {{\"a\": \"]\"}}{pit}"),
        // A root that is no array or object has no nesting.
        format!("1 {pit}"),
        format!("\"a\" {pit}"),
        format!("true{pit}"),
    ] {
        assert_eq!(
            check_depth(text.as_bytes(), MAX_JSON_DEPTH),
            Ok(()),
            "`{text}`"
        );
    }
    // Inside the root the limit holds, after a closed inner value too.
    let inside = format!("[[1], {pit}");
    assert_eq!(
        check_depth(inside.as_bytes(), MAX_JSON_DEPTH),
        Err(TooDeep {
            offset: 6 + MAX_JSON_DEPTH - 1
        })
    );
    let leading = format!(" \n\t// c\n{pit}");
    assert_eq!(
        check_depth(leading.as_bytes(), MAX_JSON_DEPTH),
        Err(TooDeep {
            offset: 8 + MAX_JSON_DEPTH
        })
    );
}

// ---- the mark, for a reader whose findings have spans ----

#[test]
fn blank_bom_puts_three_spaces_where_the_mark_is() {
    let read = blank_bom(b"\xEF\xBB\xBF{\"a\": 1}");
    assert_eq!(&*read, b"   {\"a\": 1}");
    // One mark, as vanilla's reader skips one: a second is the parser's to refuse.
    assert_eq!(
        &*blank_bom(b"\xEF\xBB\xBF\xEF\xBB\xBF{}"),
        b"   \xEF\xBB\xBF{}"
    );
    assert_eq!(&*blank_bom(b"\xEF\xBB\xBF"), b"   ");
}

#[test]
fn blank_bom_borrows_a_file_without_a_mark() {
    for bytes in [&b"{}"[..], b"", b"\xEF\xBB", b" \xEF\xBB\xBF{}"] {
        let read = blank_bom(bytes);
        assert!(matches!(read, std::borrow::Cow::Borrowed(_)), "{bytes:?}");
        assert_eq!(&*read, bytes);
    }
}
