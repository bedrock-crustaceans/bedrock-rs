//! Of a member written twice the last value stands, whole, at every level (the game keeps the
//! last copy of a duplicate key): `json::last_wins` blanks every earlier copy, keeping every
//! other byte where it was, so spans into the text stay right.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon_core::diagnostic::{DiagnosticCode, code_for_deserialize_error_kind};
use bedrock_addon_core::json::{
    JsonSlice, MAX_JSON_DEPTH, Value, from_str_jsonc, last_wins, last_wins_except, parse_at,
};
use facet::Facet;

/// `text` with its earlier copies blanked, and the check that only blanks changed.
fn cleaned(text: &str) -> String {
    let out = last_wins(text).into_owned();
    assert_eq!(out.len(), text.len(), "offsets are kept");
    for (a, b) in text.bytes().zip(out.bytes()) {
        assert!(
            a == b || (b == b' ' && a != b'\n'),
            "only blanks: {text:?} → {out:?}"
        );
    }
    out
}

fn value(text: &str) -> Value {
    facet_json::from_str_jsonc(text).unwrap()
}

#[test]
fn a_text_without_duplicates_is_borrowed_unchanged() {
    for text in [
        r#"{"a": 1, "b": {"a": 2}, "c": [{"a": 1}, {"a": 2}]}"#,
        "[1, 2, 3]",
        "7",
        r#"{"a": "{\"a\": 1, \"a\": 2}"}"#,
    ] {
        assert!(
            matches!(last_wins(text), std::borrow::Cow::Borrowed(_)),
            "{text}"
        );
    }
}

#[test]
fn the_last_value_stands_whole_at_every_level() {
    let text = r#"{"a": {"x": 1, "y": 2}, "b": [ {"k": 1, "k": [2]} ], "a": {"z": 3}, "c": {"d": 1, "d": {"e": 1, "e": 2}}}"#;
    assert_eq!(
        value(&cleaned(text)),
        value(r#"{"b": [{"k": [2]}], "a": {"z": 3}, "c": {"d": {"e": 2}}}"#),
        "no member of the first `a` survives (facet_value would merge the two objects)"
    );
    // Three copies, comments and white space between members.
    let text = "{\"v\": 1, /* one */ \"v\" : 2 // two\n, \"w\": 0, \"v\": 3}";
    assert_eq!(value(&cleaned(text)), value(r#"{"w": 0, "v": 3}"#));
}

#[test]
fn keys_compare_by_their_value_not_their_spelling() {
    let text = r#"{"a": 1, "\u0061": 2, "b/": 3, "b\/": 4, "c\"": 5, "c\"": 6}"#;
    assert_eq!(
        value(&cleaned(text)),
        value(r#"{"a": 2, "b/": 4, "c\"": 6}"#)
    );
}

#[test]
fn a_typed_struct_reads_the_last_value_where_facet_refuses_a_second_one() {
    #[derive(Debug, Facet, PartialEq)]
    struct Description {
        identifier: String,
    }
    #[derive(Debug, Facet, PartialEq)]
    struct Item {
        description: Description,
    }
    let text = r#"{"description": {"identifier": "a:first", "identifier": "a:last"}}"#;
    assert!(
        parse_at::<Item>(JsonSlice::new(text)).is_err(),
        "facet alone refuses"
    );
    let text = cleaned(text);
    let item = parse_at::<Item>(JsonSlice::new(&text)).unwrap();
    assert_eq!(item.description.identifier, "a:last");
}

#[test]
fn malformed_text_is_left_for_the_parser() {
    for text in [r#"{"a": 1, "a": "#, r#"{"a" 1, "a": 2}"#, "", "/* x"] {
        let _ = last_wins(text);
    }
    assert_eq!(last_wins(r#"{"a": 1, "a": "#), r#"{"a": 1, "a": "#);
}

#[test]
fn a_deep_text_is_scanned_to_the_limit_and_left_alone_beyond_it() {
    // Within the scan's depth the duplicate at the bottom is blanked …
    let deep = |depth: usize| {
        format!(
            "{}{{\"a\": 1, \"a\": 2}}{}",
            "[".repeat(depth),
            "]".repeat(depth)
        )
    };
    // (the scan's depth is the readers': the object is level 128).
    let text = deep(MAX_JSON_DEPTH - 1);
    assert!(matches!(last_wins(&text), std::borrow::Cow::Owned(_)));
    // … beyond it the text is the parser's, which refuses it, unchanged, and nothing
    // overflows.
    for depth in [MAX_JSON_DEPTH, 600, 5000] {
        let text = deep(depth);
        assert!(
            matches!(last_wins(&text), std::borrow::Cow::Borrowed(_)),
            "{depth}"
        );
    }
}

/// The fault a reader reports for `text` as a family reads it (`last_wins`, then the door),
/// and the text it read.
fn fault_after_last_wins(text: &str) -> (facet_json::DeserializeError, String) {
    let cleaned = last_wins(text).into_owned();
    let fault = from_str_jsonc::<Value>(&cleaned)
        .err()
        .unwrap_or_else(|| panic!("`{text}` was read, as `{cleaned}`"));
    (fault, cleaned)
}

/// The texts with a syntax fault inside a copy that loses, or between it and the next.
const FAULTS_IN_A_LOSING_COPY: &[&str] = &[
    // The task's case: a misspelt literal inside the object that loses.
    r#"{"a": {"x": tru}, "a": 1}"#,
    // The losing value itself.
    r#"{"a": tru, "a": 1}"#,
    r#"{"a": -, "a": 1}"#,
    r#"{"a": 1x, "a": 1}"#,
    // In an array.
    r#"{"a": [1, 2,, 3], "a": 1}"#,
    r#"{"a": [1 2], "a": 1}"#,
    r#"{"a": [1, nul], "a": 1}"#,
    r#"{"a": [1, {"b" 2}], "a": 1}"#,
    // In a member that loses inside a member that loses, and below one that wins.
    r#"{"a": {"b": [fals], "b": 2}, "a": 1}"#,
    r#"{"k": {"a": {"x": }, "a": 1}}"#,
    r#"{"k": [{"a": [1,, 2], "a": 1}]}"#,
    // Between the copies: no comma, two commas.
    r#"{"a": 1 "a": 2}"#,
    r#"{"a": 1,, "a": 2}"#,
    r#"{"a": {"x": 1} "a": 2}"#,
    // With comments around the fault.
    "{\"a\": [1, // one\n tru], \"a\": 2}",
    r#"{"a": {"x": 1 /* c */ 2}, "a": 2}"#,
    r#"{"a": /* c */ tru /* d */, "a": 2}"#,
    "{\"a\": 1 // c\n \"a\": 2}",
    // An escape that is none, in a value and in a key of the copy that loses.
    r#"{"a": "\q", "a": 1}"#,
    r#"{"a": {"\q": 1}, "a": 1}"#,
    // Three copies: the fault in the first, in the second.
    r#"{"a": [tru], "a": 1, "a": 2}"#,
    r#"{"a": 0, "a": [tru], "a": 2}"#,
];

/// A syntax fault anywhere in the text is the document's fault, whichever copy of a
/// member wins. `last_wins` blanked the copy that loses with the fault in it, and malformed
/// JSON loaded as valid.
#[test]
fn a_syntax_fault_in_a_copy_that_loses_is_the_document_s_fault() {
    for &text in FAULTS_IN_A_LOSING_COPY {
        // The fault the text has, read as it is written (a value merges copies: it reads on).
        let written = from_str_jsonc::<Value>(text)
            .err()
            .unwrap_or_else(|| panic!("`{text}` has no fault"));
        // It lies before the copy that stands: `"a": <a number>}` ends each text.
        let last = text.rfind("\"a\": ").unwrap();
        assert!(
            written
                .span
                .is_some_and(|span| span.offset as usize <= last),
            "`{text}`: {written:?}"
        );
        let (fault, cleaned) = fault_after_last_wins(text);
        assert_eq!(
            std::mem::discriminant(&fault.kind),
            std::mem::discriminant(&written.kind),
            "`{text}` as `{cleaned}`: {fault:?}, not {written:?}"
        );
        assert_eq!(fault.span, written.span, "`{text}` as `{cleaned}`");
        assert!(
            matches!(
                code_for_deserialize_error_kind(&fault.kind, fault.path.is_some(), false),
                DiagnosticCode::MalformedInput { .. }
            ),
            "`{text}`: {fault:?}"
        );
        // Kept members of a family's exception are no exception to this.
        let kept = last_wins_except(text, &["k"]).into_owned();
        assert!(from_str_jsonc::<Value>(&kept).is_err(), "`{text}`");
    }
}

/// xorshift64*: a fixed sequence per seed.
struct Rng(u64);

impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        usize::try_from(self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) % n as u64).unwrap()
    }

    fn pick<'a>(&mut self, of: &[&'a str]) -> &'a str {
        of[self.below(of.len())]
    }
}

/// A random value as text, `levels` deep at most, of few keys so that most objects write one
/// twice.
fn random_text(rng: &mut Rng, levels: usize, out: &mut String) {
    const SCALARS: &[&str] = &[
        "null", "true", "false", "0", "-1.5", "1e3", r#""s""#, r#""""#,
    ];
    const KEYS: &[&str] = &[r#""a""#, r#""b""#, r#""a""#, r#""k""#];
    const GAPS: &[&str] = &["", "", " ", "\n", " /* c */ ", " // c\n"];
    out.push_str(rng.pick(GAPS));
    match if levels == 0 { 0 } else { rng.below(5) } {
        0 | 1 => out.push_str(rng.pick(SCALARS)),
        2 => {
            out.push('[');
            for index in 0..rng.below(4) {
                if index > 0 {
                    out.push(',');
                }
                random_text(rng, levels - 1, out);
            }
            out.push(']');
        }
        _ => {
            out.push('{');
            for index in 0..rng.below(6) {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(rng.pick(GAPS));
                out.push_str(rng.pick(KEYS));
                out.push(':');
                random_text(rng, levels - 1, out);
            }
            out.push_str(rng.pick(GAPS));
            out.push('}');
        }
    }
    out.push_str(rng.pick(GAPS));
}

/// Over generated texts, each with one byte changed, dropped or doubled, or cut short: a text
/// with a fault has that fault after `last_wins`, at the same place, and a text without one
/// reads as before. No fault of the grammar hides in a copy that loses.
#[test]
fn no_fault_of_a_generated_text_is_lost_to_last_wins() {
    let mut rng = Rng(0x00B1_6157_5EED_0D06);
    let (mut faults, mut whole, mut blanked_with_a_fault) = (0, 0, 0);
    for case in 0..20_000 {
        let mut text = String::new();
        random_text(&mut rng, 1 + case % 5, &mut text);
        if text.is_empty() {
            continue;
        }
        let at = rng.below(text.len());
        match rng.below(6) {
            0 => {}
            1 => text.truncate(at),
            2 => {
                text.remove(at);
            }
            3 => text.insert_str(
                at,
                rng.pick(&[",", ":", "}", "]", "{", "[", "x", "\"", "\\", "/*"]),
            ),
            4 => {
                let with = rng.pick(&[",", ":", "}", "]", "{", "[", "x", "\"", "\\", " ", "/"]);
                text.replace_range(at..=at, with);
            }
            _ => {
                let doubled = text[at..=at].to_owned();
                text.insert_str(at, &doubled);
            }
        }
        let written = from_str_jsonc::<Value>(&text);
        let cleaned = last_wins(&text);
        let read = from_str_jsonc::<Value>(&cleaned);
        match (&written, &read) {
            (Err(written), Err(read)) => {
                assert_eq!(
                    std::mem::discriminant(&read.kind),
                    std::mem::discriminant(&written.kind),
                    "`{text}` as `{cleaned}`: {read:?}, not {written:?}"
                );
                assert_eq!(read.span, written.span, "`{text}` as `{cleaned}`");
                faults += 1;
                blanked_with_a_fault += usize::from(matches!(cleaned, std::borrow::Cow::Owned(_)));
            }
            (Ok(_), Ok(_)) => whole += 1,
            _ => panic!("`{text}` as `{cleaned}`: written {written:?}, read {read:?}"),
        }
    }
    assert!(
        faults > 5_000 && whole > 5_000 && blanked_with_a_fault > 100,
        "{faults} with a fault ({blanked_with_a_fault} of them blanked before it), {whole} whole"
    );
}

/// The copies that lose before the fault are still blanked: the fault is reported as it is
/// without them, not a "duplicate field" before it.
#[test]
fn a_copy_that_loses_before_the_fault_is_still_blanked() {
    #[derive(Debug, Facet)]
    struct Pair {
        a: u8,
        #[facet(default)]
        b: Option<bool>,
    }
    let text = r#"{"a": 1, "a": 2, "b": tru}"#;
    assert_eq!(cleaned(text), r#"{        "a": 2, "b": tru}"#);
    let fault = parse_at::<Pair>(JsonSlice::new(&cleaned(text))).unwrap_err();
    // (the parser stops on the byte after the letters it took for a literal)
    assert_eq!(fault.span.offset, 25, "{fault:?}");
    // The fault first, then a copy that loses: nothing is blanked past the fault, and the
    // parser never gets there.
    let text = r#"{"b": tru, "a": 1, "a": 2}"#;
    assert!(matches!(last_wins(text), std::borrow::Cow::Borrowed(_)));
    // A copy that loses before the fault, another with the fault in it.
    let text = r#"{"b": 1, "b": true, "a": [tru], "a": 2}"#;
    assert_eq!(cleaned(text), r#"{        "b": true, "a": [tru], "a": 2}"#);
    let fault = parse_at::<Pair>(JsonSlice::new(&cleaned(text))).unwrap_err();
    assert_eq!(fault.span.offset, 29, "{fault:?}");
}

/// A copy that loses and is well formed loses in silence.
#[test]
fn a_well_formed_copy_that_loses_still_loses_in_silence() {
    for (text, last) in [
        (r#"{"a": {"x": true}, "a": 1}"#, r#"{"a": 1}"#),
        (
            r#"{"a": [1, 2, [3]], "a": {"b": 0, "b": null}}"#,
            r#"{"a": {"b": null}}"#,
        ),
        (
            "{\"a\": 1e5 /* c */, // d\n \"a\": \"\\u0041\"}",
            r#"{"a": "A"}"#,
        ),
        (
            r#"{"a": 18446744073709551616, "a": -0.5} trailing"#,
            r#"{"a": -0.5}"#,
        ),
    ] {
        assert_eq!(value(&cleaned(text)), value(last), "`{text}`");
        assert_eq!(
            from_str_jsonc::<Value>(&last_wins(text)).unwrap(),
            value(last),
            "`{text}`"
        );
    }
}

// The scan is linear in an object's members.
#[test]
fn a_wide_object_is_scanned_in_linear_time() {
    let members: Vec<String> = (0..100_000).map(|n| format!("\"k{n}\": {n}")).collect();
    let text = format!("{{{}}}", members.join(", "));
    assert!(matches!(last_wins(&text), std::borrow::Cow::Borrowed(_)));
    // With one duplicate at each end, both copies are found.
    let text = format!("{{\"k99999\": 0, {}, \"k0\": 1}}", members.join(", "));
    let out = last_wins(&text);
    assert_eq!(out.len(), text.len());
    let value: Value = facet_json::from_str(&out).unwrap();
    let object = value.as_object().unwrap();
    assert_eq!(object.len(), 100_000);
    assert_eq!(
        object.get("k0").unwrap().as_number().unwrap().to_i64(),
        Some(1)
    );
    assert_eq!(
        object.get("k99999").unwrap().as_number().unwrap().to_i64(),
        Some(99_999)
    );
}
