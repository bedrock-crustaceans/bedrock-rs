//! The door to facet-json (`json/read.rs`): no text reaches its parsers unless its string
//! escapes are RFC 8259's, its nesting is within the limit and nothing follows its root value.
//!
//! facet-json 0.50.0-rc.7 decodes `"\é"` from a `&str` by reading a string from the middle of
//! a character: undefined behaviour, an abort in a debug build. An abort kills the test
//! binary, so the tests of the unsafe classes run their case in a child process and look at
//! how it ended.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fmt::Write as _;
use std::process::{Command, Stdio};

use bedrock_addon_core::diagnostic::{Span, code_for_deserialize_error_kind, codes};
use bedrock_addon_core::json::{
    JsonSlice, MAX_JSON_DEPTH, Scanned, UnsafeText, Value, from_slice_jsonc, from_str,
    from_str_borrowed_jsonc, from_str_jsonc, last_wins, parse_at, probe_envelope, scan,
    trailing_content,
};
use facet_format::DeserializeErrorKind;
use indexmap::IndexMap;

/// The contents of JSON strings that facet-json cannot decode safely or decodes wrongly, by
/// class (the scanner of 0.50.0-rc.7, `decode_string_owned_unchecked`).
const UNSAFE_STRINGS: &[(&str, &str)] = &[
    // The reviewer's reproducer: the lead byte is pushed as a character and the next read
    // starts on the continuation byte (`str::from_utf8_unchecked` on no UTF-8).
    ("a backslash before a two-byte character", r"\é"),
    ("a backslash before a three-byte character", r"\€"),
    ("a backslash before a four-byte character", r"\😀"),
    ("a bad escape after text", r"ab\éc"),
    // `\u` takes the next four bytes as a `str` whatever they are.
    ("a unicode escape cut by a character", r"\u00é0"),
    ("a unicode escape cut inside a character", r"\u000é"),
    ("a low half cut inside a character", r"\ud83d\u00éé"),
    // Read without a fault, wrongly: `q`, `'`, `0`, and `A` from a signed number.
    ("an escape JSON does not have", r"\q"),
    ("an escaped apostrophe", r"\'"),
    ("an escaped digit", r"\0"),
    ("an escaped upper-case letter", r"\N"),
    ("a signed unicode escape", r"\u+041"),
    ("a unicode escape of three digits", r"\u041"),
    ("a unicode escape of no digits", r"\u"),
    ("a unicode escape with a letter past f", r"\u00g0"),
];

/// The documents each case is read from: the string as a value, as a key, and nested.
fn documents(content: &str) -> [String; 3] {
    [
        format!(r#"{{"a": "{content}"}}"#),
        format!(r#"{{"{content}": 1}}"#),
        format!(r#"{{"a": {{"b": ["{content}"]}}}}"#),
    ]
}

/// A reader by name: `true` when it refused the document.
type Reader = (&'static str, fn(&str) -> bool);

/// The readers of the crate a document goes through; each is `true` when it refused the
/// document (and `last_wins` when it left the text alone).
const READERS: &[Reader] = &[
    ("probe_envelope", |text| {
        probe_envelope(JsonSlice::new(text)).is_err()
    }),
    ("parse_at into a map of strings", |text| {
        parse_at::<IndexMap<String, Value>>(JsonSlice::new(text)).is_err()
    }),
    ("parse_at into a value", |text| {
        parse_at::<Value>(JsonSlice::new(text)).is_err()
    }),
    ("last_wins", |text| {
        matches!(last_wins(text), std::borrow::Cow::Borrowed(_))
    }),
];

/// Runs `child_case` of this binary for one reader and one document in a process of its own;
/// `Ok` when the reader refused the document, else how the process ended.
fn in_a_child(reader: usize, document: &str) -> Result<(), String> {
    let status = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "child_case", "--ignored", "--test-threads=1"])
        .env("JSON_READ_CASE", format!("{reader}\n{document}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    if status.success() {
        Ok(())
    } else {
        Err(format!("{status}"))
    }
}

/// The child of [`in_a_child`]: succeeds when the reader refuses the document. Without the
/// variable it is not a child and does nothing.
#[test]
#[ignore = "not a test of its own: the child process of in_a_child, which runs it with --ignored"]
fn child_case() {
    let Ok(case) = std::env::var("JSON_READ_CASE") else {
        return;
    };
    let (reader, document) = case.split_once('\n').unwrap();
    let (_, refuses) = READERS[reader.parse::<usize>().unwrap()];
    assert!(refuses(document), "the document was read");
}

/// Every unsafe class, through every reader, in a child each: the reader refuses the document
/// and the process lives.
#[test]
fn no_reader_dies_of_or_reads_an_unsafe_string() {
    let mut failures = Vec::new();
    for (class, content) in UNSAFE_STRINGS {
        for document in documents(content) {
            for (index, (reader, _)) in READERS.iter().enumerate() {
                if let Err(ended) = in_a_child(index, &document) {
                    failures.push(format!("{reader}: {class}: `{document}`: {ended}"));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} cases were read or killed the process:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// --- the finding ---------------------------------------------------------------------------

fn fixture(name: &str) -> Vec<u8> {
    let path = format!("{}/tests/fixtures/json/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The message of the `MalformedInput` a refused text is reported as.
fn malformed(error: &facet_json::DeserializeError) -> String {
    let code = code_for_deserialize_error_kind(&error.kind, error.path.is_some(), false);
    match code.args_of(&codes::SYNTAX) {
        Some(message) => message[0].clone(),
        None => panic!("not MalformedInput: {code:?}"),
    }
}

/// The reviewer's reproducer, in this process: a `MalformedInput` that names the escape, on
/// the backslash and the character, at line 2.
#[test]
fn a_backslash_before_a_non_ascii_character_is_malformed_input_at_its_span() {
    let text = "{\n  \"a\": \"\\é\"}";
    let failure = probe_envelope(JsonSlice::new(text)).unwrap_err();
    let at = text.find('\\').unwrap();
    assert_eq!(
        failure.span,
        Span {
            offset: u32::try_from(at).unwrap(),
            len: 3
        }
    );
    assert_eq!(failure.span.line_col(text), (2, 9));
    assert_eq!(&text[at..at + 3], "\\é");
    assert_eq!(
        malformed(&failure.error),
        "unexpected character 'é', expected one of `\"\\/bfnrtu` after a backslash (bad escape \
         sequence in string)"
    );
}

/// The span is a file offset: the slice's own offset is added.
#[test]
fn the_span_of_a_bad_escape_is_rebased_to_the_file() {
    let file = r#"    {"a": "\q"}"#;
    let failure = parse_at::<Value>(JsonSlice {
        text: &file[4..],
        offset: 4,
    })
    .unwrap_err();
    assert_eq!(failure.span, Span { offset: 11, len: 2 });
}

/// Every unsafe class is `MalformedInput` through each reader, in this process (after the
/// scan nothing here reaches facet).
#[test]
fn every_unsafe_class_is_malformed_input_through_every_entry_point() {
    for (class, content) in UNSAFE_STRINGS {
        for document in documents(content) {
            let envelope = probe_envelope(JsonSlice::new(&document)).unwrap_err();
            assert!(malformed(&envelope.error).contains("escape"), "{class}");
            let typed = parse_at::<IndexMap<String, Value>>(JsonSlice::new(&document)).unwrap_err();
            assert!(malformed(&typed.error).contains("escape"), "{class}");
            for error in [
                from_str::<Value>(&document).unwrap_err(),
                from_str_jsonc::<Value>(&document).unwrap_err(),
                from_str_borrowed_jsonc::<Value>(&document).unwrap_err(),
                from_slice_jsonc::<Value>(document.as_bytes()).unwrap_err(),
            ] {
                assert!(error.path.is_none(), "{class}: a parser-level error");
                assert!(malformed(&error).contains("escape"), "{class}: {error:?}");
                let span = error.span.unwrap();
                let bad = &document[span.offset as usize..(span.offset + span.len) as usize];
                assert!(bad.starts_with('\\'), "{class}: the span is `{bad}`");
            }
        }
    }
}

#[test]
fn a_cut_unicode_escape_names_what_stands_in_its_place() {
    let message = |text: &str| malformed(&from_str_jsonc::<Value>(text).unwrap_err());
    assert_eq!(
        message(r#""\u00g0""#),
        "unexpected character 'g', expected four hexadecimal digits after `\\u` (bad unicode \
         escape sequence in string)"
    );
    assert_eq!(
        message(r#""\u00"#),
        "unexpected end of input, expected four hexadecimal digits after `\\u` (bad unicode \
         escape sequence in string)"
    );
    assert_eq!(
        scan(r#""\u00é0""#.as_bytes(), true),
        Err(UnsafeText::BadUnicodeEscape {
            offset: 1,
            len: 6,
            found: Some('é')
        })
    );
    assert_eq!(
        scan(br#"["\q"]"#, true),
        Err(UnsafeText::BadEscape {
            offset: 2,
            len: 2,
            found: 'q'
        })
    );
}

/// What facet reports without a fault stays facet's to report: a string or a text that ends
/// early, a backslash that ends the text, a surrogate without its pair.
#[test]
fn faults_facet_reports_safely_stay_its_own() {
    let kind = |text: &str| from_str_jsonc::<Value>(text).unwrap_err().kind;
    for text in [
        r#"{"a": "b"#,
        r#"{"a": "b\"#,
        r#""\"#,
        r#"{"a\": 1}"#,
        r#"["a", "b"#,
    ] {
        assert_eq!(scan(text.as_bytes(), true).map(|s| s.root_end), Ok(None));
        assert!(
            matches!(kind(text), DeserializeErrorKind::UnexpectedEof { .. }),
            "`{text}`"
        );
    }
    for text in [
        r#""\ud800""#,
        r#""\udc00""#,
        r#""\ud800\u0041""#,
        r#""\ud800x""#,
        r#"{"\ud83d": 1}"#,
    ] {
        assert!(scan(text.as_bytes(), true).is_ok(), "`{text}`");
        assert!(
            matches!(kind(text), DeserializeErrorKind::InvalidUtf8 { .. }),
            "`{text}`"
        );
    }
}

/// The fixture facet reads as `q` is refused by the door; raw control characters in a string stay
/// accepted.
#[test]
fn the_escapes_fixture_is_refused_and_raw_control_characters_are_kept() {
    let text = String::from_utf8(fixture("escapes.json")).unwrap();
    assert!(malformed(&from_str_jsonc::<Value>(&text).unwrap_err()).contains("'q'"));
    let kept: Vec<String> = from_str_jsonc("[\"line\nbreak\", \"tab\there\", \"\u{1}\"]").unwrap();
    assert_eq!(kept, ["line\nbreak", "tab\there", "\u{1}"]);
}

// --- what is still read ----------------------------------------------------------------------

#[test]
fn every_escape_of_rfc_8259_decodes_as_before() {
    let text = r#"["\"", "\\", "\/", "\b", "\f", "\n", "\r", "\t", "é", "\u00e9", "\u00E9",
        "\ud83d\ude00", "😀", "a\\\"b", "\u0000", "\\u0041", "\\é", "x\\"]"#;
    let expected = [
        "\"", "\\", "/", "\u{8}", "\u{c}", "\n", "\r", "\t", "é", "é", "é", "😀", "😀", "a\\\"b",
        "\0", "\\u0041", "\\é", "x\\",
    ];
    let owned: Vec<String> = from_str_jsonc(text).unwrap();
    assert_eq!(owned, expected);
    let strict: Vec<String> = from_str(text).unwrap();
    assert_eq!(strict, expected);
    let borrowed: Vec<std::borrow::Cow<'_, str>> = from_str_borrowed_jsonc(text).unwrap();
    assert_eq!(borrowed, expected);
    let bytes: Vec<String> = from_slice_jsonc(text.as_bytes()).unwrap();
    assert_eq!(bytes, expected);
    let through_parse_at: Vec<String> = parse_at(JsonSlice::new(text)).unwrap();
    assert_eq!(through_parse_at, expected);
    // The door gives what facet gives.
    let facet: Vec<String> = facet_json::from_str_jsonc(text).unwrap();
    assert_eq!(facet, expected);
}

#[test]
fn escaped_keys_and_raw_slices_are_read_as_before() {
    let text = r#"{"a\nb": {"c\"d": "\u0041"}, "\u00e9": [1]}"#;
    let envelope = probe_envelope(JsonSlice::new(text)).unwrap();
    let keys: Vec<&str> = envelope.members.keys().map(String::as_str).collect();
    assert_eq!(keys, ["a\nb", "é"]);
    let inner = envelope.payload("a\nb").unwrap();
    assert_eq!(inner.text, r#"{"c\"d": "\u0041"}"#);
    assert_eq!(inner.offset, 9);
    let map: IndexMap<String, String> = parse_at(inner).unwrap();
    assert_eq!(map["c\"d"], "A");
}

/// A backslash in a comment is no escape; a quote in a comment opens no string.
#[test]
fn comments_are_not_strings() {
    let text = "// C:\\path \"\\q\n{ /* \"\\é */ \"a\": 1 } // \\";
    let map: IndexMap<String, u32> = from_str_jsonc(text).unwrap();
    assert_eq!(map["a"], 1);
    // Plain JSON has no comments: facet stops on the `/`, as before.
    assert!(matches!(
        from_str::<Value>("// x\n1").unwrap_err().kind,
        DeserializeErrorKind::UnexpectedChar { ch: '/', .. }
    ));
}

/// Bytes that are not UTF-8: in a comment accepted, in a string an error on the string, as facet's
/// byte entry point has it.
#[test]
fn bytes_that_are_not_utf8_read_as_facet_reads_them() {
    let comment = fixture("invalid_utf8_comment.json");
    assert!(std::str::from_utf8(&comment).is_err());
    let value: Value = from_slice_jsonc(&comment).unwrap();
    assert!(value.as_object().unwrap().get("a").is_some());

    let string = fixture("invalid_utf8_string.json");
    let crate_error = from_slice_jsonc::<Value>(&string).unwrap_err();
    let facet = facet_json::from_slice_jsonc::<Value>(&string).unwrap_err();
    assert!(matches!(
        crate_error.kind,
        DeserializeErrorKind::InvalidUtf8 { .. }
    ));
    assert!(matches!(
        facet.kind,
        DeserializeErrorKind::InvalidUtf8 { .. }
    ));
    assert_eq!(crate_error.span, facet.span);

    let stray = from_slice_jsonc::<Value>(b"/* \xff */ [1, \xff]").unwrap_err();
    assert!(matches!(
        stray.kind,
        DeserializeErrorKind::UnexpectedChar { ch: 'ÿ', .. }
    ));
    assert_eq!(stray.span.unwrap().offset, 12);
    assert!(from_slice_jsonc::<Value>(b"/* \xff */ [\"\\\xc3\"]").is_err());
}

// --- the scan --------------------------------------------------------------------------------

fn scanned(text: &str) -> Scanned {
    scan(text.as_bytes(), true).unwrap()
}

#[test]
fn the_scan_finds_the_end_of_the_root_value() {
    let end = |text: &str| scanned(text).root_end;
    assert_eq!(end(r#"{"a": [1, {"b": "}"}]}"#), Some(22));
    assert_eq!(end("  [ ] "), Some(5));
    assert_eq!(end(r#""a\"b" x"#), Some(6));
    assert_eq!(end("-12.5e+3,"), Some(8));
    assert_eq!(end("true false"), Some(4));
    assert_eq!(end("null"), Some(4));
    assert_eq!(end("/* [ */ // {\n 7"), Some(15));
    // No whole root value: blank, cut off, or not what the parser reads.
    for text in [
        "",
        "  // only\n",
        "{",
        r#"{"a": 1"#,
        "[1, 2",
        "-",
        "tru",
        "nul l",
        "x",
        "}",
        ",",
    ] {
        assert_eq!(end(text), None, "`{text}`");
    }
    // A bracket that closes another kind is the parser's error: no end is named after it.
    assert_eq!(end("[}] 1"), None);
    assert_eq!(end("{]} 1"), None);
}

#[test]
fn the_scan_says_where_content_follows_the_root() {
    let trailing = |text: &str| scanned(text).trailing;
    assert_eq!(trailing("{} x"), Some(3));
    assert_eq!(trailing("{}{}"), Some(2));
    assert_eq!(trailing("{}é"), Some(2));
    assert_eq!(trailing("[1] ]"), Some(4));
    assert_eq!(trailing("1 2"), Some(2));
    assert_eq!(trailing("{} // done\n /* x */ \n"), None);
    assert_eq!(trailing("{}\n"), None);
    // A comment that never ends is content from its slash.
    assert_eq!(trailing("{} /* open"), Some(3));
    assert_eq!(trailing("{"), None);
    // Without comments a `/` is content.
    assert_eq!(scan(b"{} // x", false).unwrap().trailing, Some(3));
}

#[test]
fn trailing_content_is_a_span_to_the_last_byte_that_is_not_white_space() {
    assert_eq!(
        trailing_content("{\"a\": 1}\n garbage é \n"),
        Some(Span {
            offset: 10,
            len: 10
        })
    );
    assert_eq!(trailing_content("{\"a\": 1}\n"), None);
    assert_eq!(trailing_content(r#"{"a": "\q"} x"#), None);
}

/// What follows the root value is never given to facet: whatever it holds, the root reads,
/// and facet's errors keep their offsets.
#[test]
fn content_after_the_root_never_reaches_facet() {
    for tail in [
        "é",
        " garbage",
        r#" "\é""#,
        r#" {"\q": "\u00é0"}"#,
        " \\",
        " \"",
        " /* \"",
        " [[[[",
        " \u{0}",
    ] {
        let text = format!(r#"{{"a": "b\n"}}{tail}"#);
        let map: IndexMap<String, String> = from_str_jsonc(&text).unwrap();
        assert_eq!(map["a"], "b\n", "`{tail}`");
        let map: IndexMap<String, String> = parse_at(JsonSlice::new(&text)).unwrap();
        assert_eq!(map["a"], "b\n", "`{tail}`");
        let envelope = probe_envelope(JsonSlice::new(&text)).unwrap();
        assert_eq!(envelope.payload("a").unwrap().text, r#""b\n""#);
        assert!(from_slice_jsonc::<Value>(text.as_bytes()).is_ok());
    }
    let text = r#"{"a": "x"} tail"#;
    let error = from_str_jsonc::<IndexMap<String, u32>>(text).unwrap_err();
    let facet = facet_json::from_str_jsonc::<IndexMap<String, u32>>(text).unwrap_err();
    assert!(error.span.is_some());
    assert_eq!(error.span, facet.span);
}

#[test]
fn nesting_past_the_limit_is_refused_before_facet() {
    let nested = |levels: usize| format!("{}{}", "[".repeat(levels), "]".repeat(levels));
    assert!(from_str_jsonc::<Value>(&nested(MAX_JSON_DEPTH)).is_ok());
    let deep = nested(MAX_JSON_DEPTH + 1);
    assert_eq!(
        scan(deep.as_bytes(), true),
        Err(UnsafeText::TooDeep {
            offset: MAX_JSON_DEPTH
        })
    );
    let error = from_str_jsonc::<Value>(&deep).unwrap_err();
    assert_eq!(
        malformed(&error),
        "unexpected character '[', expected at most 128 levels of nested arrays and objects"
    );
    // Far past any stack: refused by the scan, in a mix of both brackets, never parsed.
    let abyss = "[{\"a\":".repeat(200_000);
    assert!(parse_at::<Value>(JsonSlice::new(&abyss)).is_err());
    assert!(from_slice_jsonc::<Value>(abyss.as_bytes()).is_err());
    // Brackets in strings and comments are not nesting.
    let flat = format!("[\"{}\" /* {} */]", "[".repeat(500), "{".repeat(500));
    assert!(from_str_jsonc::<Value>(&flat).is_ok());
}

// --- the order of faults (reviews/CORE-ESC minors 2, 4, 5) -----------------------------------

/// The error of `text` through each entry point of the door.
fn errors(text: &str) -> [facet_json::DeserializeError; 4] {
    [
        from_str::<Value>(text).unwrap_err(),
        from_str_jsonc::<Value>(text).unwrap_err(),
        from_str_borrowed_jsonc::<Value>(text).unwrap_err(),
        from_slice_jsonc::<Value>(text.as_bytes()).unwrap_err(),
    ]
}

/// A file has one syntax error, its first: a missing colon, a stray byte or a bracket that
/// closes another kind before a bad escape is reported where it stands, in the words the text
/// gets without the escape.
#[test]
fn an_earlier_syntax_fault_is_reported_before_a_later_bad_escape() {
    for (text, fault) in [
        (r#"{"a" 1, "b": "\q"}"#, 5),
        (r#"[1 x, "\q"]"#, 3),
        (r#"[1, 2} "\é""#, 5),
        (r#"{"a": tru, "b": "\u00g0"}"#, 9),
        (r#"{"a": 1 "\q": 2}"#, 8),
    ] {
        let repaired = text
            .replace("\\q", "q")
            .replace("\\é", "é")
            .replace("\\u00g0", "g");
        let expected = from_str_jsonc::<Value>(&repaired).unwrap_err();
        for error in errors(text) {
            let message = malformed(&error);
            assert!(!message.contains("escape"), "`{text}`: {message}");
            // A fault on the string that holds the escape names the string as written.
            let expected_message =
                malformed(&expected).replace(r#"String("q")"#, r#"String("\q")"#);
            assert_eq!(message, expected_message, "`{text}`");
            let span = error.span.unwrap();
            assert_eq!(span.offset, expected.span.unwrap().offset, "`{text}`");
            assert!(
                span.offset as usize <= fault && fault < (span.offset + span.len.max(1)) as usize,
                "`{text}`: {span:?} is not on byte {fault}"
            );
        }
    }
    // The escape comes first: it is the fault.
    for text in [r#"["\q" 1 x]"#, r#"{"\q" 1}"#, r#"["a", "\q"} x"#] {
        for error in errors(text) {
            assert!(malformed(&error).contains("escape"), "`{text}`");
            let span = error.span.unwrap();
            assert_eq!(&text[span.offset as usize..][..2], "\\q", "`{text}`");
        }
    }
}

#[test]
fn an_earlier_syntax_fault_is_reported_before_nesting_past_the_limit() {
    // Brackets that close another kind: the first one is the fault, however many follow.
    let mismatched = "[}".repeat(MAX_JSON_DEPTH + 10);
    // A missing comma before the pit.
    let pit = format!("[1 {}", "[".repeat(MAX_JSON_DEPTH + 10));
    for (text, fault) in [(&mismatched, 1), (&pit, 3)] {
        for error in errors(text) {
            let message = malformed(&error);
            assert!(!message.contains("levels"), "{message}");
            assert_eq!(error.span.unwrap().offset as usize, fault, "{message}");
        }
    }
    // Nothing wrong before the bracket past the limit: that is the fault.
    let deep = format!("[1, {}", "[".repeat(MAX_JSON_DEPTH + 10));
    for error in errors(&deep) {
        assert!(malformed(&error).contains("128 levels"));
        assert_eq!(error.span.unwrap().offset as usize, 4 + MAX_JSON_DEPTH - 1);
    }
}

#[test]
fn nesting_past_the_limit_names_the_bracket_it_stops_on() {
    let objects = "{\"a\":".repeat(MAX_JSON_DEPTH + 1);
    for error in errors(&objects) {
        assert_eq!(
            malformed(&error),
            "unexpected character '{', expected at most 128 levels of nested arrays and objects"
        );
        assert_eq!(error.span.unwrap().offset as usize, 5 * MAX_JSON_DEPTH);
    }
    let arrays = "[".repeat(MAX_JSON_DEPTH + 1);
    for error in errors(&arrays) {
        assert_eq!(
            malformed(&error),
            "unexpected character '[', expected at most 128 levels of nested arrays and objects"
        );
    }
}

/// A `/*` that never ends is refused by the parser inside the root value; after it, it is
/// content like any other: the file loads and the author is told.
#[test]
fn a_comment_that_never_ends_after_the_root_is_content() {
    assert_eq!(scanned("{} /* open").trailing, Some(3));
    assert_eq!(scanned("{} /* closed */ /* open *").trailing, Some(16));
    assert_eq!(scanned("{} /* closed */ // line").trailing, None);
    assert_eq!(scanned("{} /*/").trailing, Some(3));
    assert_eq!(scanned("{} /**/").trailing, None);
    assert_eq!(
        trailing_content("{} /* open \n"),
        Some(Span { offset: 3, len: 7 })
    );
    assert!(from_str_jsonc::<Value>("{} /* open").is_ok());
    assert!(from_str_jsonc::<Value>("{ /* open").is_err());
}

/// A `/*` that never ends inside the root value is the door's to report, in the parser's own
/// words, at the comment: facet-json rc.7 forgets that a text has comments once it has
/// restored (an untagged enum) and read 512 events on, and then calls the slash an
/// unexpected character. Near or far, every door says the
/// same.
#[test]
fn a_comment_that_never_ends_inside_the_root_is_reported_the_same_near_and_far() {
    let end_of_input = "unexpected end of input, expected in block comment";
    let near = "{\"first\": {\"name\": \"n\"}, /* open";
    let error = from_str_jsonc::<Holder>(near).unwrap_err();
    assert_eq!(malformed(&error), end_of_input);
    assert_eq!(error.span, Some(facet::Span::new(25, 7)));
    assert_eq!(
        scan(near.as_bytes(), true),
        Err(UnsafeText::UnterminatedComment { offset: 25, len: 7 })
    );
    // Without comments a slash is a stray byte, the parser's to name.
    assert_eq!(scan(near.as_bytes(), false).unwrap().root_end, None);

    // Far: an untagged enum, then more events than the deserialiser reads ahead.
    let mut far = String::from("[{\"a\": 1, \"b\": 2}");
    for _ in 0..600 {
        far.push_str(", 1");
    }
    let comment = far.len() + 2;
    far.push_str(", /* open *");
    let span = Some(facet::Span::new(comment, 9));
    let error = from_str_jsonc::<Vec<Untagged>>(&far).unwrap_err();
    assert_eq!(
        (malformed(&error), error.span),
        (end_of_input.to_owned(), span)
    );
    let error = from_str_borrowed_jsonc::<Vec<Untagged>>(&far).unwrap_err();
    assert_eq!(
        (malformed(&error), error.span),
        (end_of_input.to_owned(), span)
    );
    let error = from_slice_jsonc::<Vec<Untagged>>(far.as_bytes()).unwrap_err();
    assert_eq!(
        (malformed(&error), error.span),
        (end_of_input.to_owned(), span)
    );

    // Before the root value there is no value at all: the same report.
    let error = from_str_jsonc::<Value>("  /* open").unwrap_err();
    assert_eq!(malformed(&error), end_of_input);
    assert_eq!(error.span, Some(facet::Span::new(2, 7)));

    // A syntax fault before the comment is the file's first fault, in the parser's words.
    let error = from_str_jsonc::<Value>("{\"a\" 1, /* open").unwrap_err();
    assert_ne!(malformed(&error), end_of_input);
    assert!(error.span.is_some_and(|span| (span.offset as usize) < 9));
    // Past a bracket that closes another kind the parser has stopped: its report stands.
    let error = from_str_jsonc::<Value>("[1} /* open").unwrap_err();
    assert_ne!(malformed(&error), end_of_input);
}

// --- facet-json's restore (reviews/CORE-ESC upstream note) -----------------------------------

#[derive(Debug, PartialEq, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
enum Untagged {
    Pair { a: u8, b: u8 },
    Named { name: String },
    List(Vec<u8>),
    Number(f64),
}

#[derive(Debug, PartialEq, facet::Facet)]
struct Holder {
    first: Untagged,
    #[facet(default)]
    second: Option<Untagged>,
    #[facet(default)]
    last: Option<String>,
}

/// facet-json rc.7's `JsonParser::restore` rebuilds its scanner without the comment flag. An
/// untagged enum is read by saving and restoring; a comment after one, inside one and between
/// its members must still be trivia in a text with comments.
#[test]
fn a_comment_after_an_untagged_enum_is_still_a_comment() {
    let text = "{\n  // before\n  \"first\": { /* in */ \"name\": \"n\" // after the member\n  } \
                /* after the value */,\n  // between\n  \"second\": [1, /* in a list */ 2] // \
                tail\n  , \"last\": /* before a string */ \"x\" // end\n}\n// after the root\n";
    let expected = Holder {
        first: Untagged::Named {
            name: "n".to_owned(),
        },
        second: Some(Untagged::List(vec![1, 2])),
        last: Some("x".to_owned()),
    };
    assert_eq!(from_str_jsonc::<Holder>(text).unwrap(), expected);
    assert_eq!(from_str_borrowed_jsonc::<Holder>(text).unwrap(), expected);
    assert_eq!(
        from_slice_jsonc::<Holder>(text.as_bytes()).unwrap(),
        expected
    );
    assert_eq!(parse_at::<Holder>(JsonSlice::new(text)).unwrap(), expected);
    // The same beside an integer beyond 64 bits (the read over the respelt copy).
    let wide = "{\"first\": /* c */ 18446744073709551616 // c\n, \"second\": {\"a\": 1, /* c */ \
                \"b\": 2} // c\n}";
    let read = from_str_jsonc::<Holder>(wide).unwrap();
    assert_eq!(read.first, Untagged::Number(18_446_744_073_709_551_616.0));
    assert_eq!(read.second, Some(Untagged::Pair { a: 1, b: 2 }));
    // Without comments a comment stays a syntax error.
    assert!(from_str::<Holder>(text).is_err());
}

/// The deserialiser reads events 512 at a time, so in a small document every event is read
/// before the first restore and the scanner is never asked again. In a long one it is: the
/// comments after the first batch must still be trivia.
#[test]
fn a_comment_far_after_an_untagged_enum_is_still_a_comment() {
    let mut text = String::from("[\n");
    let mut expected = Vec::new();
    for index in 0..3_000_u32 {
        let small = u8::try_from(index % 200).unwrap();
        match index % 3 {
            0 => {
                writeln!(
                    text,
                    "  {{\"a\": {small}, /* in */ \"b\": 2}}, // after {index}"
                )
                .unwrap();
                expected.push(Untagged::Pair { a: small, b: 2 });
            }
            1 => {
                writeln!(
                    text,
                    "  /* before */ {{\"name\": \"n{index}\"}} /* after */,"
                )
                .unwrap();
                expected.push(Untagged::Named {
                    name: format!("n{index}"),
                });
            }
            _ => {
                write!(text, "  [{small}, // in a list\n 7],\n").unwrap();
                expected.push(Untagged::List(vec![small, 7]));
            }
        }
    }
    text.push_str("  1.5 // the last\n] // after the root\n");
    expected.push(Untagged::Number(1.5));
    let read = |result: Result<Vec<Untagged>, facet_json::DeserializeError>| {
        result.unwrap_or_else(|error| {
            let at = error.span.map_or(0, |span| span.offset as usize);
            panic!(
                "{error} at byte {at}: `{}`",
                &text[at..(at + 40).min(text.len())]
            )
        })
    };
    assert_eq!(read(from_str_jsonc(&text)), expected);
    assert_eq!(read(from_str_borrowed_jsonc(&text)), expected);
    assert_eq!(read(from_slice_jsonc(text.as_bytes())), expected);
    assert_eq!(
        parse_at::<Vec<Untagged>>(JsonSlice::new(&text)).unwrap(),
        expected
    );
    // The same over the respelt copy of a text with an integer beyond 64 bits.
    let wide = text.replacen("1.5 // the last", "18446744073709551616 // the last", 1);
    let read: Vec<Untagged> = from_str_jsonc(&wide).unwrap();
    assert_eq!(
        read.last(),
        Some(&Untagged::Number(18_446_744_073_709_551_616.0))
    );
    assert_eq!(read.len(), expected.len());
}
