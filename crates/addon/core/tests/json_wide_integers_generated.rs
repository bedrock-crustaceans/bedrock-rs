//! Generated documents with integers too large for 64 bits at random places, from
//! fixed seeds, and the cost of reading them.
//!
//! The reference is the same document with each such integer replaced by the real token that
//! spells its double (the shortest decimal; `1e999` beyond a double's range): the two read to
//! equal values, or fail with the same finding at the same place of the author's text,
//! through every entry point of the text reader and the typed reader.

// The crate's one protocol for cost tests, and this binary's one lock.
use bedrock_addon_core::test_cost as cost;

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::parse::{
    Options, elements_of, members_of, parse, parse_bytes, parse_slice,
};
use bedrock_addon_core::json::{
    JsonSlice, Value, from_str, from_str_jsonc, last_wins, probe_envelope, scan, to_json,
};
use bedrock_addon_core::schema::{Schema, read_document, read_value};
use bedrock_addon_core::versioning::Reading;
use facet::Facet;
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
#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
enum Either {
    Text(String),
    Number(f64),
}

/// A typed target with a member of every kind a pack document has.
#[derive(Debug, PartialEq, Facet, Schema)]
struct Typed {
    double: Option<f64>,
    float: Option<f32>,
    signed: Option<i64>,
    unsigned: Option<u64>,
    small: Option<i32>,
    text: Option<String>,
    borrowed: Option<String>,
    either: Option<Either>,
    strict: Option<f64>,
    strict_float: Option<f32>,
    strict_int: Option<i64>,
    strict_text: Option<String>,
    #[facet(default)]
    list: Vec<f64>,
    raw: Option<Value>,
    #[facet(default)]
    map: IndexMap<String, f64>,
    any: Option<Value>,
    nested: Option<Box<Typed>>,
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

/// What the plain typed read of `slice` as [`Typed`] gives: the value, or the first error as
/// its code, its path and its span.
#[allow(
    clippy::type_complexity,
    reason = "a value or a failure of three parts"
)]
fn typed_read(slice: JsonSlice<'_>) -> Result<Typed, (String, String, (usize, usize))> {
    let parsed = parse_slice(slice, Options::default()).expect("JSON");
    let mut d = Diagnostics::default();
    let value = {
        let mut read = Read::new(Reading::OWN, &mut d).profile(Profile::FIRST_FAILURE);
        read_document::<Typed>(&Node::parsed(&parsed, JsonPath::new()), &mut read)
    };
    // A value the read built is the read's (a converted scalar is told and the value kept, as
    // the old typed read kept it); else the first error.
    match (value, d.iter().find(|f| f.severity >= Severity::Error)) {
        (Some(value), _) => Ok(value),
        (None, Some(error)) => {
            let span = error.location.span.expect("a span");
            Err((
                format!("{:?}", error.code),
                error.location.json_path.to_string(),
                (span.offset as usize, span.len as usize),
            ))
        }
        (None, None) => panic!("`{}` failed and told nothing", slice.text),
    }
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
        let expected: Value =
            from_str_jsonc(&pair.reference).unwrap_or_else(|e| panic!("`{}`: {e}", pair.reference));
        let read = from_str_jsonc(text).unwrap_or_else(|e| panic!("`{text}`: {e}"));
        assert_eq!(read, expected, "`{text}`");
        // `Value` holds 1 and 1.0 equal: the written text tells an integer from a real.
        assert_eq!(to_json(&read).ok(), to_json(&expected).ok(), "`{text}`");
        assert_eq!(
            parse(text, Options::default()).unwrap().value,
            expected,
            "`{text}`"
        );
        assert_eq!(
            parse_bytes(text.as_bytes(), Options::default()).unwrap(),
            expected,
            "`{text}`"
        );
        assert_eq!(
            parse_slice(JsonSlice::new(text), Options::default())
                .unwrap()
                .value,
            expected,
            "`{text}`"
        );
        if !comments {
            assert_eq!(from_str(text).unwrap(), expected, "`{text}`");
        }
        let cleaned = last_wins(text);
        let _ = from_str_jsonc(&cleaned).unwrap_or_else(|e| panic!("`{cleaned}`: {e}"));
        let _ = probe_envelope(JsonSlice::new(text));
        // No string but where a string token stands.
        let strings = tokens.iter().filter(|t| t.starts_with('"')).count();
        let colons = tokens.iter().filter(|t| *t == ":").count();
        assert!(count_strings(&read) <= strings - colons, "`{text}`");
        if let Ok(written) = to_json(&read) {
            let again = from_str_jsonc(&written).unwrap();
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
/// fails with the same code and path at the span of the author's text that the reference's
/// span stands for.
#[test]
fn generated_typed_documents_read_or_fail_as_their_references() {
    let mut rng = Rng(0x00B1_6157_5EED_0002);
    let (mut read, mut failed, mut after_wide) = (0, 0, 0);
    for _ in 0..6000 {
        let mut tokens = Vec::new();
        typed_tokens(&mut rng, 2, &mut tokens);
        let pair = Pair::of(&mut rng, &tokens, true);
        let text = &pair.written;
        let crate_result = typed_read(JsonSlice::new(text));
        let reference = typed_read(JsonSlice::new(&pair.reference));
        match (crate_result, reference) {
            (Ok(crate_result), Ok(reference)) => {
                assert_eq!(crate_result, reference, "`{text}`");
                read += 1;
            }
            (Err(crate_result), Err(reference)) => {
                let (code, path, span) = reference;
                let expected = (code, path, pair.written_span(span.0, span.1));
                after_wide += usize::from(
                    pair.replaced
                        .first()
                        .is_some_and(|(at, len, _)| at + len <= crate_result.2.0),
                );
                assert_eq!(crate_result, expected, "`{text}`");
                failed += 1;
            }
            (crate_result, reference) => {
                panic!("`{text}`: {crate_result:?}, its reference {reference:?}")
            }
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
        ("wide integers, the typed read", &|text| {
            let value = from_str_jsonc(text).unwrap();
            assert!(!read_value::<Vec<f64>>(&value).unwrap().is_empty());
        }),
        ("wide integers, the slices of the elements", &|text| {
            let raw = elements_of(JsonSlice::new(text)).unwrap();
            assert!(!raw.is_empty());
        }),
        ("wide integers, the untyped read", &|text| {
            let value = parse(text, Options::default()).unwrap().value;
            assert!(!value.as_array().unwrap().is_empty());
        }),
    ];
    cost::assert_each_linear_at(1_000_000, document_of_wide_integers, &steps);
}

/// One integer of a million digits against one of four million is read in linear time: as a
/// value (infinity), skipped as an unread member, and held raw.
#[test]
fn an_integer_of_a_million_digits_is_read_in_linear_time() {
    #[derive(Facet, Schema)]
    struct Read {
        a: f64,
        b: u8,
    }
    #[derive(Facet, Schema)]
    struct Skipped {
        b: u8,
    }
    let document = |digits: usize| format!("{{\"a\": -{}, \"b\": 1}}", "7".repeat(digits));
    let steps: [Step<'_>; 4] = [
        ("a million digits, the scan", &|text| {
            assert!(scan(text.as_bytes(), true).unwrap().root_end.unwrap() > 0);
        }),
        ("a million digits, read", &|text| {
            let read: Read = plain(text).unwrap();
            assert_eq!(read.a, f64::NEG_INFINITY);
            assert_eq!(read.b, 1);
        }),
        ("a million digits, skipped", &|text| {
            assert_eq!(plain::<Skipped>(text).unwrap().b, 1);
        }),
        ("a million digits, raw", &|text| {
            let raw = members_of(JsonSlice::new(text)).unwrap();
            assert!(raw["a"].text.len() > 1_000_000);
        }),
    ];
    cost::assert_each_linear_at(1_000_000, document, &steps);
    // A long run of zeros before a small number is no wide integer, and as cheap.
    let zeros = |digits: usize| format!("[{}7]", "0".repeat(digits));
    let steps: [Step<'_>; 1] = [("a million leading zeros", &|text| {
        let read: Vec<u8> = read_value(&from_str_jsonc(text).unwrap()).unwrap();
        assert_eq!(read[0], 7);
    })];
    cost::assert_each_linear_at(1_000_000, zeros, &steps);
}

/// `text` read as `T` by the plain typed read (a number beyond its float read as an infinity),
/// an unknown member let be.
fn plain<T: bedrock_addon_core::schema::FromJson>(text: &str) -> Option<T> {
    let parsed = parse(text, Options::default()).ok()?;
    let mut d = Diagnostics::default();
    let mut read = Read::new(Reading::OWN, &mut d).profile(Profile::FIRST_FAILURE);
    read_document::<T>(&Node::parsed(&parsed, JsonPath::new()), &mut read)
}
