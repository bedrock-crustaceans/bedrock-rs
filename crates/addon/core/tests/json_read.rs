//! The text readers (`json::parse` and the entry points over it): a string escape outside
//! RFC 8259, nesting past the limit and what follows the root value are refused or never read,
//! as the old reader over facet-json refused them (facet-json 0.50.0-rc.7 decoded `"\é"` from
//! a `&str` by reading a string from the middle of a character: undefined behaviour, an abort
//! in a debug build). The tests of the unsafe classes still run their case in a child process
//! and look at how it ended.

use std::fmt::Write as _;
use std::process::{Command, Stdio};

use bedrock_addon_core::diagnostic::{Span, codes};
use bedrock_addon_core::json::parse::{Fault, FaultKind, Options, parse, parse_bytes, parse_slice};
use bedrock_addon_core::json::{
    JsonSlice, MAX_JSON_DEPTH, Scanned, UnsafeText, Value, from_str_jsonc, last_wins,
    probe_envelope, scan, trailing_content,
};
use bedrock_addon_core::schema::{Schema, read_value};
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
    ("read_value into a map of values", |text| {
        from_str_jsonc(text)
            .ok()
            .and_then(|json| read_value::<IndexMap<String, Value>>(&json))
            .is_none()
    }),
    ("parse_slice into a value", |text| {
        parse_slice(JsonSlice::new(text), Options::default()).is_err()
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
fn malformed(fault: &Fault) -> String {
    let code = fault.code(false);
    match code.args_of(&codes::SYNTAX) {
        Some(message) => message[0].clone(),
        None => panic!("not core/syntax: {code:?}"),
    }
}

/// The fault of `text` as JSON with comments.
fn fault(text: &str) -> Fault {
    parse(text, Options::default()).unwrap_err()
}

/// The reviewer's reproducer, in this process: a `MalformedInput` that names the escape, on
/// the backslash and the character, at line 2.
#[test]
fn a_backslash_before_a_non_ascii_character_is_malformed_input_at_its_span() {
    let text = "{\n  \"a\": \"\\é\"}";
    let failure = probe_envelope(JsonSlice::new(text)).unwrap_err();
    let at = text.find('\\').unwrap();
    assert_eq!(
        failure.span(),
        Span {
            offset: u32::try_from(at).unwrap(),
            len: 3
        }
    );
    assert_eq!(failure.span().line_col(text), (2, 9));
    assert_eq!(&text[at..at + 3], "\\é");
    assert_eq!(
        failure.message(),
        "unexpected character 'é', expected one of `\"\\/bfnrtu` after a backslash (bad escape \
         sequence in string)"
    );
}

/// The span is a file offset: the slice's own offset is added.
#[test]
fn the_span_of_a_bad_escape_is_rebased_to_the_file() {
    let file = r#"    {"a": "\q"}"#;
    let failure = parse_slice(
        JsonSlice {
            text: &file[4..],
            offset: 4,
        },
        Options::default(),
    )
    .unwrap_err();
    assert_eq!(failure.span, Span { offset: 11, len: 2 });
}

/// Every unsafe class is `MalformedInput` through each reader, in this process.
#[test]
fn every_unsafe_class_is_malformed_input_through_every_entry_point() {
    for (class, content) in UNSAFE_STRINGS {
        for document in documents(content) {
            let envelope = probe_envelope(JsonSlice::new(&document)).unwrap_err();
            assert!(envelope.message().contains("escape"), "{class}");
            assert!(from_str_jsonc(&document).is_err(), "{class}");
            for error in errors(&document) {
                assert!(malformed(&error).contains("escape"), "{class}: {error:?}");
                let span = error.span;
                let bad = &document[span.offset as usize..(span.offset + span.len) as usize];
                assert!(bad.starts_with('\\'), "{class}: the span is `{bad}`");
            }
        }
    }
}

#[test]
fn a_cut_unicode_escape_names_what_stands_in_its_place() {
    let message = |text: &str| malformed(&fault(text));
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

/// What the scan leaves to the parser it reports: a string or a text that ends early, a
/// backslash that ends the text, a surrogate without its pair.
#[test]
fn faults_facet_reports_safely_stay_its_own() {
    let kind = |text: &str| fault(text).kind;
    for text in [
        r#"{"a": "b"#,
        r#"{"a": "b\"#,
        r#""\"#,
        r#"{"a\": 1}"#,
        r#"["a", "b"#,
    ] {
        assert_eq!(scan(text.as_bytes(), true).map(|s| s.root_end), Ok(None));
        assert!(
            matches!(kind(text), FaultKind::UnexpectedEof { .. }),
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
        assert!(matches!(kind(text), FaultKind::InvalidUtf8), "`{text}`");
    }
}

/// The fixture facet reads as `q` is refused by the parser; raw control characters in a string stay
/// accepted.
#[test]
fn the_escapes_fixture_is_refused_and_raw_control_characters_are_kept() {
    let text = String::from_utf8(fixture("escapes.json")).unwrap();
    assert!(malformed(&fault(&text)).contains("'q'"));
    let kept: Vec<String> =
        read_value(&from_str_jsonc("[\"line\nbreak\", \"tab\there\", \"\u{1}\"]").unwrap())
            .unwrap();
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
    let strings = |value: Value| -> Vec<String> { read_value(&value).expect("strings") };
    assert_eq!(strings(from_str_jsonc(text).unwrap()), expected);
    let plain = parse(text, Options::default().without_comments()).unwrap();
    assert_eq!(strings(plain.value), expected);
    assert_eq!(
        strings(parse_bytes(text.as_bytes(), Options::default()).unwrap()),
        expected
    );
    let sliced = parse_slice(JsonSlice::new(text), Options::default()).unwrap();
    assert_eq!(strings(sliced.value), expected);
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
    let inner = parse_slice(inner, Options::default()).unwrap().value;
    let map: IndexMap<String, String> = read_value(&inner).unwrap();
    assert_eq!(map["c\"d"], "A");
}

/// A backslash in a comment is no escape; a quote in a comment opens no string.
#[test]
fn comments_are_not_strings() {
    let text = "// C:\\path \"\\q\n{ /* \"\\é */ \"a\": 1 } // \\";
    let map: IndexMap<String, u32> = read_value(&from_str_jsonc(text).unwrap()).unwrap();
    assert_eq!(map["a"], 1);
    // Plain JSON has no comments: the parser stops on the `/`, as before.
    assert!(matches!(
        parse("// x\n1", Options::default().without_comments())
            .unwrap_err()
            .kind,
        FaultKind::UnexpectedChar { ch: '/', .. }
    ));
}

/// Bytes that are not UTF-8: in a comment accepted, in a string an error on the string, as facet's
/// byte entry point had it.
#[test]
fn bytes_that_are_not_utf8_read_as_facet_reads_them() {
    let comment = fixture("invalid_utf8_comment.json");
    assert!(std::str::from_utf8(&comment).is_err());
    let value: Value = parse_bytes(&comment, Options::default()).unwrap();
    assert!(value.as_object().unwrap().get("a").is_some());

    let string = fixture("invalid_utf8_string.json");
    let error = parse_bytes(&string, Options::default()).unwrap_err();
    assert!(matches!(error.kind, FaultKind::InvalidUtf8));
    // On the string: from its opening quote to its closing one.
    let (start, end) = (
        error.span.offset as usize,
        (error.span.offset + error.span.len) as usize,
    );
    assert_eq!((string[start], string[end - 1]), (b'"', b'"'));

    let stray = parse_bytes(b"/* \xff */ [1, \xff]", Options::default()).unwrap_err();
    assert!(matches!(
        stray.kind,
        FaultKind::UnexpectedChar { ch: 'ÿ', .. }
    ));
    assert_eq!(stray.span.offset, 12);
    assert!(parse_bytes(b"/* \xff */ [\"\\\xc3\"]", Options::default()).is_err());
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
        let map: IndexMap<String, String> = read_value(&from_str_jsonc(&text).unwrap()).unwrap();
        assert_eq!(map["a"], "b\n", "`{tail}`");
        let envelope = probe_envelope(JsonSlice::new(&text)).unwrap();
        assert_eq!(envelope.payload("a").unwrap().text, r#""b\n""#);
        assert!(parse_bytes(text.as_bytes(), Options::default()).is_ok());
    }
}

#[test]
fn nesting_past_the_limit_is_refused_before_facet() {
    let nested = |levels: usize| format!("{}{}", "[".repeat(levels), "]".repeat(levels));
    assert!(from_str_jsonc(&nested(MAX_JSON_DEPTH)).is_ok());
    let deep = nested(MAX_JSON_DEPTH + 1);
    assert_eq!(
        scan(deep.as_bytes(), true),
        Err(UnsafeText::TooDeep {
            offset: MAX_JSON_DEPTH
        })
    );
    let error = fault(&deep);
    assert_eq!(
        malformed(&error),
        "unexpected character '[', expected at most 128 levels of nested arrays and objects"
    );
    // Far past any stack: refused by the scan, in a mix of both brackets, never parsed.
    let abyss = "[{\"a\":".repeat(200_000);
    assert!(parse_slice(JsonSlice::new(&abyss), Options::default()).is_err());
    assert!(parse_bytes(abyss.as_bytes(), Options::default()).is_err());
    // Brackets in strings and comments are not nesting.
    let flat = format!("[\"{}\" /* {} */]", "[".repeat(500), "{".repeat(500));
    assert!(from_str_jsonc(&flat).is_ok());
}

// --- the order of faults (reviews/CORE-ESC minors 2, 4, 5) -----------------------------------

/// The fault of `text` through each entry point of the parser.
fn errors(text: &str) -> [Fault; 4] {
    [
        parse(text, Options::default().without_comments()).unwrap_err(),
        parse(text, Options::default()).unwrap_err(),
        parse_slice(JsonSlice::new(text), Options::default()).unwrap_err(),
        parse_bytes(text.as_bytes(), Options::default()).unwrap_err(),
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
        let expected = parse(&repaired, Options::default()).unwrap_err();
        for error in errors(text) {
            let message = malformed(&error);
            assert!(!message.contains("escape"), "`{text}`: {message}");
            // A fault on the string that holds the escape names the string as written.
            let expected_message =
                malformed(&expected).replace(r#"String("q")"#, r#"String("\q")"#);
            assert_eq!(message, expected_message, "`{text}`");
            let span = error.span;
            assert_eq!(span.offset, expected.span.offset, "`{text}`");
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
            let span = error.span;
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
            assert_eq!(error.span.offset as usize, fault, "{message}");
        }
    }
    // Nothing wrong before the bracket past the limit: that is the fault.
    let deep = format!("[1, {}", "[".repeat(MAX_JSON_DEPTH + 10));
    for error in errors(&deep) {
        assert!(malformed(&error).contains("128 levels"));
        assert_eq!(error.span.offset as usize, 4 + MAX_JSON_DEPTH - 1);
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
        assert_eq!(error.span.offset as usize, 5 * MAX_JSON_DEPTH);
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
    assert!(from_str_jsonc("{} /* open").is_ok());
    assert!(from_str_jsonc("{ /* open").is_err());
}

/// A `/*` that never ends inside the root value is the parser's to report, in the parser's own
/// words, at the comment: facet-json rc.7 forgets that a text has comments once it has
/// restored (an untagged enum) and read 512 events on, and then calls the slash an
/// unexpected character. Near or far, every entry point says the
/// same.
#[test]
fn a_comment_that_never_ends_inside_the_root_is_reported_the_same_near_and_far() {
    let end_of_input = "unexpected end of input, expected in block comment";
    let near = "{\"first\": {\"name\": \"n\"}, /* open";
    let error = fault(near);
    assert_eq!(malformed(&error), end_of_input);
    assert_eq!(error.span, Span { offset: 25, len: 7 });
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
    let span = Span {
        offset: u32::try_from(comment).unwrap(),
        len: 9,
    };
    for error in [
        fault(&far),
        parse_bytes(far.as_bytes(), Options::default()).unwrap_err(),
    ] {
        assert_eq!(
            (malformed(&error), error.span),
            (end_of_input.to_owned(), span)
        );
    }

    // Before the root value there is no value at all: the same report.
    let error = fault("  /* open");
    assert_eq!(malformed(&error), end_of_input);
    assert_eq!(error.span, Span { offset: 2, len: 7 });

    // A syntax fault before the comment is the file's first fault, in the parser's words.
    let error = fault("{\"a\" 1, /* open");
    assert_ne!(malformed(&error), end_of_input);
    assert!((error.span.offset as usize) < 9);
    // Past a bracket that closes another kind the parser has stopped: its report stands.
    let error = fault("[1} /* open");
    assert_ne!(malformed(&error), end_of_input);
}

// --- facet-json's restore (reviews/CORE-ESC upstream note) -----------------------------------

#[derive(Debug, PartialEq, facet::Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
enum Untagged {
    Pair { a: u8, b: u8 },
    Named { name: String },
    List(Vec<u8>),
    Number(f64),
}

#[derive(Debug, PartialEq, facet::Facet, Schema)]
struct Holder {
    first: Untagged,
    second: Option<Untagged>,
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
    let typed = |text: &str| read_value::<Holder>(&from_str_jsonc(text).unwrap());
    assert_eq!(typed(text), Some(expected));
    // The same beside an integer beyond 64 bits (the read over the respelt copy).
    let wide = "{\"first\": /* c */ 18446744073709551616 // c\n, \"second\": {\"a\": 1, /* c */ \
                \"b\": 2} // c\n}";
    let read = typed(wide).unwrap();
    assert_eq!(read.first, Untagged::Number(18_446_744_073_709_551_616.0));
    assert_eq!(read.second, Some(Untagged::Pair { a: 1, b: 2 }));
    // Without comments a comment stays a syntax error.
    assert!(parse(text, Options::default().without_comments()).is_err());
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
    let read = |text: &str| -> Vec<Untagged> {
        let json = parse(text, Options::default()).unwrap_or_else(|fault| {
            let at = fault.span.offset as usize;
            panic!(
                "{fault} at byte {at}: `{}`",
                &text[at..(at + 40).min(text.len())]
            )
        });
        read_value(&json.value).expect("the list reads")
    };
    assert_eq!(read(&text), expected);
    assert_eq!(
        read_value::<Vec<Untagged>>(&parse_bytes(text.as_bytes(), Options::default()).unwrap())
            .unwrap(),
        expected
    );
    // The same over the respelt copy of a text with an integer beyond 64 bits.
    let wide = text.replacen("1.5 // the last", "18446744073709551616 // the last", 1);
    let read: Vec<Untagged> = read(&wide);
    assert_eq!(
        read.last(),
        Some(&Untagged::Number(18_446_744_073_709_551_616.0))
    );
    assert_eq!(read.len(), expected.len());
}
