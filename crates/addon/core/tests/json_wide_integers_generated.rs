//! Generated documents with integers too large for 64 bits at random places, from
//! fixed seeds, and the cost of reading them.
//!
//! The reference is the same document with each such integer replaced by the real token that
//! spells its double (the shortest decimal; `1e999` beyond a double's range): the two read to
//! equal values, or fail with the same finding at the same place of the author's text,
//! through every entry point of the door.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::borrow::Cow;

// The crate's one protocol for cost tests, and this binary's one lock.
use bedrock_addon_core::test_cost as cost;

use bedrock_addon_core::diagnostic::code_for_deserialize_error_kind;
use bedrock_addon_core::json::{
    JsonSlice, StrictFloat, StrictInt, StrictString, Value, from_slice_jsonc, from_str,
    from_str_borrowed_jsonc, from_str_jsonc, last_wins, parse_at, probe_envelope, scan, to_json,
};
use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;

/// xorshift64*: a fixed sequence per seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).unwrap()
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    fn pick<'a>(&mut self, of: &[&'a str]) -> &'a str {
        of[self.below(of.len())]
    }
}

/// Whether the token is an integer that fits neither `i64` nor `u64`: this test's own rule.
fn is_wide(token: &str) -> bool {
    let digits = token.strip_prefix('-').unwrap_or(token);
    !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
        && token.parse::<i64>().is_err()
        && token.parse::<u64>().is_err()
}

/// The real token that spells the double of a wide integer.
fn real_of(token: &str) -> String {
    let value: f64 = token.parse().unwrap();
    if value.is_infinite() {
        (if value < 0.0 { "-1e999" } else { "1e999" }).to_owned()
    } else {
        format!("{value:?}")
    }
}

/// An integer token: around the 64-bit and 128-bit bounds, or of a random length.
fn integer(rng: &mut Rng) -> String {
    let mut digits: String = match rng.below(8) {
        0 => u64::MAX.to_string(),
        1 => "18446744073709551616".into(),
        2 => "9223372036854775808".into(),
        3 => "9223372036854775809".into(),
        4 => u128::MAX.to_string(),
        5 => "340282366920938463463374607431768211456".into(),
        _ => {
            let length = match rng.below(10) {
                0 => 1 + rng.below(18),
                1..=5 => 18 + rng.below(5),
                6 | 7 => 23 + rng.below(40),
                8 => 300 + rng.below(20),
                _ => 63 + rng.below(700),
            };
            (0..length)
                .map(|at| {
                    let low = usize::from(at == 0);
                    char::from(b'0' + u8::try_from(low + rng.below(10 - low)).unwrap())
                })
                .collect()
        }
    };
    if rng.chance(8) {
        // Nines round up to a power of ten.
        digits = "9".repeat(digits.len());
    }
    if rng.chance(10) {
        digits.insert_str(0, &"0".repeat(1 + rng.below(25)));
    }
    if rng.chance(35) {
        digits.insert(0, '-');
    }
    digits
}

/// A scalar token.
fn scalar(rng: &mut Rng) -> String {
    match rng.below(10) {
        0..=4 => integer(rng),
        5 => rng
            .pick(&[
                "0",
                "-1",
                "2.5",
                "1e3",
                "-0.0",
                "1e999",
                "12345678901234567890.0",
            ])
            .into(),
        6 => rng.pick(&["true", "false", "null"]).into(),
        7 => format!("\"{}\"", integer(rng)),
        _ => rng
            .pick(&[
                "\"text\"",
                "\"a\\nb\"",
                "\"é 18446744073709551616\"",
                "\"\"",
            ])
            .into(),
    }
}

/// The tokens of a value, nested up to `depth`.
fn value_tokens(rng: &mut Rng, depth: usize, out: &mut Vec<String>) {
    match rng.below(if depth == 0 { 5 } else { 8 }) {
        0..=4 => out.push(scalar(rng)),
        5 | 6 => {
            out.push("[".into());
            for index in 0..rng.below(5) {
                if index > 0 {
                    out.push(",".into());
                }
                value_tokens(rng, depth - 1, out);
            }
            out.push("]".into());
        }
        _ => {
            out.push("{".into());
            for index in 0..rng.below(4) {
                if index > 0 {
                    out.push(",".into());
                }
                let key = rng.pick(&["a", "b", "12345678901234567890123", "k"]);
                out.push(format!("\"{key}{index}\""));
                out.push(":".into());
                value_tokens(rng, depth - 1, out);
            }
            out.push("}".into());
        }
    }
}

/// The members of the typed target of the generated documents.
const MEMBERS: &[&str] = &[
    "double",
    "float",
    "signed",
    "unsigned",
    "small",
    "text",
    "borrowed",
    "either",
    "strict",
    "strict_float",
    "strict_int",
    "strict_text",
    "list",
    "raw",
    "map",
    "any",
    "nested",
    "unknown",
];

/// A string or a number.
#[derive(Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
enum Either {
    Text(String),
    Number(f64),
}

/// A typed target with a member of every kind a pack document has.
#[derive(Debug, PartialEq, Facet)]
struct Typed<'a> {
    #[facet(default)]
    double: Option<f64>,
    #[facet(default)]
    float: Option<f32>,
    #[facet(default)]
    signed: Option<i64>,
    #[facet(default)]
    unsigned: Option<u64>,
    #[facet(default)]
    small: Option<i32>,
    #[facet(default)]
    text: Option<String>,
    #[facet(default)]
    borrowed: Option<Cow<'a, str>>,
    #[facet(default)]
    either: Option<Either>,
    #[facet(default)]
    strict: Option<StrictFloat<f64>>,
    #[facet(default)]
    strict_float: Option<StrictFloat<f32>>,
    #[facet(default)]
    strict_int: Option<StrictInt<i64>>,
    #[facet(default)]
    strict_text: Option<StrictString>,
    #[facet(default)]
    list: Vec<f64>,
    #[facet(default)]
    raw: Option<RawJson<'a>>,
    #[facet(default)]
    map: IndexMap<String, f64>,
    #[facet(default)]
    any: Option<Value>,
    #[facet(default)]
    nested: Option<Box<Typed<'a>>>,
}

/// The tokens of an object for [`Typed`]: most members get a value their type takes.
fn typed_tokens(rng: &mut Rng, depth: usize, out: &mut Vec<String>) {
    out.push("{".into());
    let mut written = 0;
    for &member in MEMBERS {
        if !rng.chance(35) {
            continue;
        }
        if written > 0 {
            out.push(",".into());
        }
        written += 1;
        out.push(format!("\"{member}\""));
        out.push(":".into());
        if rng.chance(6) {
            // Any value at all: mostly a type error at this member.
            value_tokens(rng, 2, out);
            continue;
        }
        match member {
            "text" | "borrowed" | "strict_text" => {
                out.push(if rng.chance(85) {
                    format!("\"{}\"", integer(rng))
                } else {
                    integer(rng)
                });
            }
            "list" => {
                out.push("[".into());
                for index in 0..rng.below(5) {
                    if index > 0 {
                        out.push(",".into());
                    }
                    out.push(integer(rng));
                }
                out.push("]".into());
            }
            "map" => {
                out.push("{".into());
                for index in 0..rng.below(4) {
                    if index > 0 {
                        out.push(",".into());
                    }
                    out.push(format!("\"k{index}\""));
                    out.push(":".into());
                    out.push(integer(rng));
                }
                out.push("}".into());
            }
            "raw" | "any" | "unknown" => value_tokens(rng, 2, out),
            "nested" if depth > 0 => typed_tokens(rng, depth - 1, out),
            "nested" => out.push("null".into()),
            "signed" | "unsigned" | "small" | "strict_int" if rng.chance(60) => {
                out.push(
                    rng.pick(&["0", "7", "-3", "2147483647", "9223372036854775807"])
                        .into(),
                );
            }
            _ => out.push(integer(rng)),
        }
    }
    out.push("}".into());
}

/// A document as written and its reference, with the place of each replaced token: its offset
/// in the written text, its length there, and its length in the reference.
struct Pair {
    written: String,
    reference: String,
    replaced: Vec<(usize, usize, usize)>,
}

impl Pair {
    fn of(rng: &mut Rng, tokens: &[String], comments: bool) -> Self {
        let mut pair = Pair {
            written: String::new(),
            reference: String::new(),
            replaced: Vec::new(),
        };
        let gaps: &[&str] = if comments {
            &[
                "",
                "",
                " ",
                "\n",
                "\t",
                "\r\n",
                " // 18446744073709551616\n",
                "/* 1e5 */",
            ]
        } else {
            &["", "", " ", "\n", "\t", "\r\n"]
        };
        for token in tokens {
            let gap = rng.pick(gaps);
            pair.written.push_str(gap);
            pair.reference.push_str(gap);
            if is_wide(token) {
                let real = real_of(token);
                pair.replaced
                    .push((pair.written.len(), token.len(), real.len()));
                pair.reference.push_str(&real);
            } else {
                pair.reference.push_str(token);
            }
            pair.written.push_str(token);
        }
        pair
    }

    /// The offset in the written text of an offset in the reference.
    fn written_offset(&self, reference: usize) -> usize {
        let mut shift = 0isize;
        for &(at, written, real) in &self.replaced {
            let start = usize::try_from(at.cast_signed() - shift).unwrap();
            if reference <= start {
                break;
            }
            // Inside a replaced token: no finding points there.
            assert!(
                reference >= start + real,
                "offset {reference} of the reference is inside a replaced token"
            );
            shift += written.cast_signed() - real.cast_signed();
        }
        usize::try_from(reference.cast_signed() + shift).unwrap()
    }

    /// A span of the reference as a span of the written text.
    fn written_span(&self, offset: usize, len: usize) -> (usize, usize) {
        let start = self.written_offset(offset);
        (start, self.written_offset(offset + len) - start)
    }
}

/// A failure in comparable form: the code, the path, the message and the span.
fn failure(
    error: &bedrock_addon_core::json::SliceError,
) -> (String, String, String, (usize, usize)) {
    let code =
        code_for_deserialize_error_kind(&error.error.kind, error.error.path.is_some(), false);
    (
        format!("{code:?}"),
        error.path.to_string(),
        error.error.kind.to_string(),
        (error.span.offset as usize, error.span.len as usize),
    )
}

/// `typed` with its raw members read (its own, then its nested ones), so two documents
/// compare by value.
fn by_value(mut typed: Typed<'_>) -> (Typed<'_>, Vec<Option<Value>>) {
    let read = |raw: RawJson<'_>| {
        from_str_jsonc::<Value>(raw.as_str()).unwrap_or_else(|e| panic!("`{}`: {e}", raw.as_str()))
    };
    let mut raws = vec![typed.raw.take().map(read)];
    if let Some(nested) = typed.nested.take() {
        let (nested, inner) = by_value(*nested);
        typed.nested = Some(Box::new(nested));
        raws.extend(inner);
    }
    (typed, raws)
}

/// For 6000 untyped documents: the written text and its reference read to equal values
/// through every entry point, the written value has no string where the author wrote a
/// number, and it writes back to text that reads to the same value and the same text.
#[test]
fn generated_untyped_documents_read_as_their_references() {
    let mut rng = Rng(0x00B1_6157_5EED_0001);
    let (mut with_wide, mut wide_tokens, mut beyond) = (0, 0, 0);
    for case in 0..6000 {
        let mut tokens = Vec::new();
        value_tokens(&mut rng, 4, &mut tokens);
        let comments = case % 3 != 0;
        let pair = Pair::of(&mut rng, &tokens, comments);
        let text = &pair.written;
        let expected: Value = facet_json::from_str_jsonc(&pair.reference)
            .unwrap_or_else(|e| panic!("`{}`: {e}", pair.reference));
        let read = from_str_jsonc::<Value>(text).unwrap_or_else(|e| panic!("`{text}`: {e}"));
        assert_eq!(read, expected, "`{text}`");
        // `Value` holds 1 and 1.0 equal: the written text tells an integer from a real.
        assert_eq!(to_json(&read).ok(), to_json(&expected).ok(), "`{text}`");
        assert_eq!(
            from_str_borrowed_jsonc::<Value>(text).unwrap(),
            expected,
            "`{text}`"
        );
        assert_eq!(
            from_slice_jsonc::<Value>(text.as_bytes()).unwrap(),
            expected,
            "`{text}`"
        );
        assert_eq!(
            parse_at::<Value>(JsonSlice::new(text)).unwrap(),
            expected,
            "`{text}`"
        );
        if !comments {
            assert_eq!(from_str::<Value>(text).unwrap(), expected, "`{text}`");
        }
        let cleaned = last_wins(text);
        let _ = from_str_jsonc::<Value>(&cleaned).unwrap_or_else(|e| panic!("`{cleaned}`: {e}"));
        let _ = probe_envelope(JsonSlice::new(text));
        // No string but where a string token stands.
        let strings = tokens.iter().filter(|t| t.starts_with('"')).count();
        let colons = tokens.iter().filter(|t| *t == ":").count();
        assert!(count_strings(&read) <= strings - colons, "`{text}`");
        if let Ok(written) = to_json(&read) {
            let again = from_str_jsonc::<Value>(&written).unwrap();
            assert_eq!(again, read, "`{text}` written as {written}");
            assert_eq!(to_json(&again).unwrap(), written, "`{text}`");
        } else {
            beyond += 1;
        }
        with_wide += usize::from(!pair.replaced.is_empty());
        wide_tokens += pair.replaced.len();
    }
    assert!(
        with_wide > 2000 && wide_tokens > 3000 && beyond > 100,
        "{with_wide} documents with {wide_tokens} wide integers, {beyond} beyond a double"
    );
}

/// The string values in `value` (keys are not counted).
fn count_strings(value: &Value) -> usize {
    if value.as_string().is_some() {
        1
    } else if let Some(array) = value.as_array() {
        array.iter().map(count_strings).sum()
    } else if let Some(object) = value.as_object() {
        object.iter().map(|(_, member)| count_strings(member)).sum()
    } else {
        0
    }
}

/// For 6000 typed documents: the written text reads to the value its reference reads to, or
/// fails with the same code, path and message at the span of the author's text that the
/// reference's span stands for.
#[test]
fn generated_typed_documents_read_or_fail_as_their_references() {
    let mut rng = Rng(0x00B1_6157_5EED_0002);
    let (mut read, mut failed, mut after_wide) = (0, 0, 0);
    for _ in 0..6000 {
        let mut tokens = Vec::new();
        typed_tokens(&mut rng, 2, &mut tokens);
        let pair = Pair::of(&mut rng, &tokens, true);
        let text = &pair.written;
        let crate_result = parse_at::<Typed<'_>>(JsonSlice::new(text));
        let reference = parse_at::<Typed<'_>>(JsonSlice::new(&pair.reference));
        match (crate_result, reference) {
            (Ok(crate_result), Ok(reference)) => {
                if let Some(Cow::Borrowed(borrowed)) = crate_result.borrowed {
                    let start = borrowed.as_ptr() as usize - text.as_ptr() as usize;
                    assert_eq!(&text[start..start + borrowed.len()], borrowed);
                }
                assert_eq!(by_value(crate_result), by_value(reference), "`{text}`");
                read += 1;
            }
            (Err(crate_result), Err(reference)) => {
                let (code, path, message, span) = failure(&reference);
                let expected = (code, path, message, pair.written_span(span.0, span.1));
                assert_eq!(failure(&crate_result), expected, "`{text}`");
                failed += 1;
                after_wide += usize::from(
                    pair.replaced
                        .first()
                        .is_some_and(|(at, len, _)| at + len <= crate_result.span.offset as usize),
                );
            }
            (crate_result, reference) => panic!(
                "`{text}`: {:?}, its reference {:?}",
                crate_result.map_err(|e| failure(&e)),
                reference.map_err(|e| failure(&e))
            ),
        }
        // The owned and the borrowing read of the text agree, value or fault.
        let owned = from_str_jsonc::<Typed<'static>>(text);
        let borrowing = from_str_borrowed_jsonc::<Typed<'_>>(text);
        match (owned, borrowing) {
            (Ok(owned), Ok(borrowing)) => {
                // A raw slice is the author's text in both.
                let raw = |typed: &Typed<'_>| typed.raw.as_ref().map(|raw| raw.as_str().to_owned());
                assert_eq!(raw(&owned), raw(&borrowing), "`{text}`");
                if let Some(raw) = raw(&owned) {
                    assert!(text.contains(&raw), "`{raw}` is part of `{text}`");
                }
                assert_eq!(by_value(owned), by_value(borrowing), "`{text}`");
            }
            (Err(owned), Err(borrowing)) => {
                assert_eq!(
                    owned.kind.to_string(),
                    borrowing.kind.to_string(),
                    "`{text}`"
                );
                assert_eq!(owned.span, borrowing.span, "`{text}`");
            }
            (owned, borrowing) => panic!(
                "`{text}`: owned {:?}, borrowing {:?}",
                owned.map_err(|e| e.kind.to_string()),
                borrowing.map_err(|e| e.kind.to_string())
            ),
        }
    }
    assert!(
        read > 1500 && failed > 1500 && after_wide > 500,
        "{read} read, {failed} failed, {after_wide} of them after a wide integer"
    );
}

// --- cost ------------------------------------------------------------------------------------

/// One function of a cost test, on the text of a document.
type Step<'a> = cost::Step<'a, String>;

/// A document of about `bytes` bytes of wide integers of mixed sizes.
fn document_of_wide_integers(bytes: usize) -> String {
    let mut rng = Rng(0x00B1_6157_5EED_0003);
    let mut text = String::with_capacity(bytes + 1000);
    text.push('[');
    while text.len() < bytes {
        let digits = 20 + rng.below(40);
        if rng.chance(30) {
            text.push('-');
        }
        for _ in 0..digits {
            text.push(char::from(b'1' + u8::try_from(rng.below(9)).unwrap()));
        }
        text.push_str(", ");
    }
    text.push_str("0]");
    text
}

/// A megabyte of wide integers against four megabytes is scanned and read in linear time,
/// through the owned and the borrowed entry points. The crate's one protocol: both
/// documents are built once and the lock is held from then to the last measurement.
#[test]
fn a_megabyte_of_wide_integers_is_read_in_linear_time() {
    let steps: [Step<'_>; 4] = [
        ("wide integers, the scan", &|text| {
            assert!(scan(text.as_bytes(), true).unwrap().root_end.unwrap() > 0);
        }),
        ("wide integers, the owned read", &|text| {
            assert!(!from_str_jsonc::<Vec<f64>>(text).unwrap().is_empty());
        }),
        ("wide integers, the borrowed read", &|text| {
            let raw = from_str_borrowed_jsonc::<Vec<RawJson<'_>>>(text).unwrap();
            assert!(!raw.is_empty());
        }),
        ("wide integers, the untyped read", &|text| {
            let value = from_str_borrowed_jsonc::<Value>(text).unwrap();
            assert!(!value.as_array().unwrap().is_empty());
        }),
    ];
    cost::assert_each_linear_at(1_000_000, document_of_wide_integers, &steps);
}

/// One integer of a million digits against one of four million is read in linear time: as a
/// value (infinity), skipped as an unread member, and held raw.
#[test]
fn an_integer_of_a_million_digits_is_read_in_linear_time() {
    #[derive(Facet)]
    struct Read {
        a: f64,
        b: u8,
    }
    #[derive(Facet)]
    struct Skipped {
        b: u8,
    }
    let document = |digits: usize| format!("{{\"a\": -{}, \"b\": 1}}", "7".repeat(digits));
    let steps: [Step<'_>; 4] = [
        ("a million digits, the scan", &|text| {
            assert!(scan(text.as_bytes(), true).unwrap().root_end.unwrap() > 0);
        }),
        ("a million digits, read", &|text| {
            let read: Read = from_str_jsonc(text).unwrap();
            assert_eq!(read.a, f64::NEG_INFINITY);
            assert_eq!(read.b, 1);
        }),
        ("a million digits, skipped", &|text| {
            assert_eq!(from_str_jsonc::<Skipped>(text).unwrap().b, 1);
        }),
        ("a million digits, raw", &|text| {
            let raw: IndexMap<String, RawJson<'_>> = from_str_borrowed_jsonc(text).unwrap();
            assert!(raw["a"].as_str().len() > 1_000_000);
        }),
    ];
    cost::assert_each_linear_at(1_000_000, document, &steps);
    // A long run of zeros before a small number is no wide integer, and as cheap.
    let zeros = |digits: usize| format!("[{}7]", "0".repeat(digits));
    let steps: [Step<'_>; 1] = [("a million leading zeros", &|text| {
        let read: Vec<u8> = from_str_jsonc(text).unwrap();
        assert_eq!(read[0], 7);
    })];
    cost::assert_each_linear_at(1_000_000, zeros, &steps);
}
