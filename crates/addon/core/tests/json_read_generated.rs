#![allow(clippy::pedantic, missing_docs)]
//! Generated inputs for the door to facet-json (`json/read.rs`), from fixed seeds: string
//! contents over every byte after a backslash, cut escapes and random UTF-8, and whole
//! documents with faults put in. Nothing here panics; a string that reads is the string a
//! reference decoder of RFC 8259 gives; and a text the scan passes reads through the door
//! exactly as facet reads it on its own. The cost of the scan is linear.
//!
//! facet is only ever called on a text the scan passed: an input it would decode out of
//! bounds aborts the process and takes the test binary with it.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

// The crate's one protocol for cost tests, and this binary's one lock.
#[path = "../src/test_cost.rs"]
mod cost;

use bedrock_addon_core::json::{
    JsonSlice, Value, from_slice_jsonc, from_str, from_str_borrowed_jsonc, from_str_jsonc,
    last_wins, parse_at, probe_envelope, scan, trailing_content,
};
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

    /// A character from anywhere in Unicode.
    fn character(&mut self) -> char {
        loop {
            let limit = [0x80, 0x800, 0x1_0000, 0x11_0000][self.below(4)];
            if let Some(c) = char::from_u32(u32::try_from(self.below(limit)).unwrap()) {
                return c;
            }
        }
    }

    fn hex(&mut self, digits: usize) -> String {
        (0..digits)
            .map(|_| char::from(b"0123456789abcdefABCDEF"[self.below(22)]))
            .collect()
    }
}

// --- string contents -------------------------------------------------------------------------

/// What reading `"` + content + `"` must give.
#[derive(Debug, PartialEq)]
enum Expected {
    /// An escape outside RFC 8259: the scan refuses the text.
    Refused,
    /// The scan passes it and the reader fails: no closing quote, a surrogate without its
    /// pair, bytes that are not UTF-8.
    Fails,
    /// The decoded string.
    Reads(String),
}

/// A decoder of RFC 8259 §7 written for this test, over the string token that starts
/// `document` (through its first quote that no backslash escapes).
fn reference(document: &[u8]) -> Expected {
    assert_eq!(document.first(), Some(&b'"'));
    // The grammar of the whole token first: the scan reads it all before facet reads any.
    let mut i = 1;
    let mut closed = false;
    while let Some(&byte) = document.get(i) {
        match byte {
            b'"' => {
                closed = true;
                break;
            }
            b'\\' => match document.get(i + 1) {
                None => break,
                Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => i += 2,
                Some(b'u') => {
                    let digits = document.get(i + 2..i + 6);
                    if !digits.is_some_and(|d| d.iter().all(u8::is_ascii_hexdigit)) {
                        return Expected::Refused;
                    }
                    i += 6;
                }
                Some(_) => return Expected::Refused,
            },
            _ => i += 1,
        }
    }
    if !closed {
        return Expected::Fails;
    }
    let token = &document[1..i];
    let unit = |at: usize| {
        let digits = std::str::from_utf8(&token[at..at + 4]).unwrap();
        u32::from_str_radix(digits, 16).unwrap()
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < token.len() {
        if token[i] != b'\\' {
            out.push(token[i]);
            i += 1;
            continue;
        }
        let simple = match token[i + 1] {
            b'"' => Some('"'),
            b'\\' => Some('\\'),
            b'/' => Some('/'),
            b'b' => Some('\u{8}'),
            b'f' => Some('\u{c}'),
            b'n' => Some('\n'),
            b'r' => Some('\r'),
            b't' => Some('\t'),
            _ => None,
        };
        let decoded = if let Some(simple) = simple {
            i += 2;
            simple
        } else {
            let high = unit(i + 2);
            i += 6;
            let scalar = if (0xD800..0xDC00).contains(&high) {
                if token.get(i..i + 2) != Some(b"\\u") {
                    return Expected::Fails;
                }
                let low = unit(i + 2);
                i += 6;
                if !(0xDC00..0xE000).contains(&low) {
                    return Expected::Fails;
                }
                0x1_0000 + ((high - 0xD800) << 10) + (low - 0xDC00)
            } else {
                high
            };
            match char::from_u32(scalar) {
                Some(decoded) => decoded,
                None => return Expected::Fails,
            }
        };
        out.extend_from_slice(decoded.encode_utf8(&mut [0; 4]).as_bytes());
    }
    String::from_utf8(out).map_or(Expected::Fails, Expected::Reads)
}

#[test]
fn the_reference_decoder_reads_rfc_8259() {
    let read = |text: &str| reference(text.as_bytes());
    assert_eq!(
        read(r#""a\n\u00e9\ud83d\ude00é""#),
        Expected::Reads("a\né😀é".into())
    );
    assert_eq!(read(r#""a" \q"#), Expected::Reads("a".into()));
    assert_eq!(read(r#""\q""#), Expected::Refused);
    assert_eq!(read(r#""\u12""#), Expected::Refused);
    assert_eq!(read(r#""\ud800" "#), Expected::Fails);
    assert_eq!(read(r#""\udc00""#), Expected::Fails);
    assert_eq!(read(r#""\ud800\u0041""#), Expected::Fails);
    assert_eq!(read(r#""abc\""#), Expected::Fails);
    assert_eq!(reference(b"\"\xff\""), Expected::Fails);
}

/// One piece of string content. `text` keeps it UTF-8; otherwise any byte may follow a
/// backslash or stand alone.
fn piece(rng: &mut Rng, case: usize, text: bool, out: &mut Vec<u8>) {
    let mut push = |s: &str| out.extend_from_slice(s.as_bytes());
    match rng.below(16) {
        0..=2 => push(rng.pick(&["a", "b c", "minecraft:stone", "0", " ", "/", "'", "u"])),
        3 => push(rng.character().encode_utf8(&mut [0; 4])),
        4 | 5 => push(rng.pick(&["\\\"", "\\\\", "\\/", "\\b", "\\f", "\\n", "\\r", "\\t"])),
        // Every byte value after a backslash, in turn; as text, the character of that value.
        6 | 7 => {
            let byte = u8::try_from(case % 256).unwrap();
            push("\\");
            if text {
                push(char::from(byte).encode_utf8(&mut [0; 4]));
            } else {
                out.push(byte);
            }
        }
        8 => {
            push("\\");
            push(rng.character().encode_utf8(&mut [0; 4]));
        }
        9 => {
            push("\\u");
            push(&rng.hex(4));
        }
        // Cut short, or cut by something that is no digit.
        10 => {
            push("\\u");
            let digits = rng.below(4);
            push(&rng.hex(digits));
            if rng.chance(50) {
                push(rng.pick(&["g", "+", "-", " ", "é", "€", "\\", "x", ""]));
            }
        }
        11 => {
            let (high, low) = (0xD800 + rng.below(0x400), 0xDC00 + rng.below(0x400));
            push(&format!("\\u{high:04x}\\u{low:04X}"));
        }
        // Halves without their pair, a pair cut after `\u`.
        12 => {
            let half = 0xD800 + rng.below(0x800);
            push(&format!("\\u{half:04x}"));
            push(rng.pick(&["", "x", "\\n", "\\u0041", "\\u", "\\ud800", "é"]));
        }
        13 => push(rng.pick(&["\n", "\t", "\u{0}", "\u{1f}", "\u{7f}", "//", "/*", "*/"])),
        14 => {
            if text {
                push(rng.pick(&["\"", "\\"]));
            } else {
                out.push(u8::try_from(rng.below(256)).unwrap());
            }
        }
        _ => push(rng.pick(&["é", "€", "😀", "\u{feff}", "ü\\n", "\\\\é"])),
    }
}

fn content(rng: &mut Rng, case: usize, text: bool) -> Vec<u8> {
    let mut out = Vec::new();
    for _ in 0..rng.below(7) {
        piece(rng, case, text, &mut out);
    }
    if rng.chance(5) {
        out.push(b'\\');
    }
    out
}

/// The tally of a generated run, so a generator that stops making a class fails the test.
#[derive(Debug, Default)]
struct Tally {
    refused: usize,
    fails: usize,
    reads: usize,
}

/// `Ok` implies the reference's string; `Refused` and `Fails` imply an error; and nothing
/// panics. As text through every reader of text, 6000 strings.
#[test]
fn generated_strings_read_as_the_reference_decodes_them_or_are_refused() {
    let mut rng = Rng(0x00C0_FFEE_5EED_0001);
    let mut tally = Tally::default();
    for case in 0..6000 {
        let content = String::from_utf8(content(&mut rng, case, true)).unwrap();
        let document = format!("\"{content}\"");
        let expected = reference(document.as_bytes());
        assert_eq!(
            scan(document.as_bytes(), true).is_err(),
            expected == Expected::Refused,
            "`{document}`"
        );
        let read = [
            from_str_jsonc::<String>(&document).ok(),
            from_str::<String>(&document).ok(),
            from_str_borrowed_jsonc::<std::borrow::Cow<'_, str>>(&document)
                .ok()
                .map(std::borrow::Cow::into_owned),
            from_slice_jsonc::<String>(document.as_bytes()).ok(),
            parse_at::<String>(JsonSlice::new(&document)).ok(),
            from_str_jsonc::<Value>(&document)
                .ok()
                .and_then(|v| Some(v.as_string()?.as_str().to_owned())),
        ];
        let wanted = match &expected {
            Expected::Reads(text) => Some(text.clone()),
            Expected::Refused | Expected::Fails => None,
        };
        for got in &read {
            assert_eq!(got, &wanted, "`{document}`");
        }
        // The string as a value and as a key of a document: read where it is one string of
        // the document, refused where the scan refuses it, and in no case a panic.
        let whole = !content.contains('"');
        for (nested, key) in [
            (format!("{{\"k\": \"{content}\"}}"), None),
            (format!("{{\"{content}\": \"v\"}}"), Some("v")),
        ] {
            let map = parse_at::<IndexMap<String, String>>(JsonSlice::new(&nested));
            let envelope = probe_envelope(JsonSlice::new(&nested));
            let _ = last_wins(&nested);
            let _ = trailing_content(&nested);
            match (&expected, whole) {
                (Expected::Reads(text), true) => {
                    let map = map.unwrap_or_else(|e| panic!("`{nested}`: {e}"));
                    match key {
                        None => assert_eq!(&map["k"], text, "`{nested}`"),
                        Some(value) => assert_eq!(map[text.as_str()], value, "`{nested}`"),
                    }
                    assert!(envelope.is_ok(), "`{nested}`");
                }
                (Expected::Refused, true) => {
                    assert!(map.is_err() && envelope.is_err(), "`{nested}`");
                }
                _ => {}
            }
        }
        match expected {
            Expected::Refused => tally.refused += 1,
            Expected::Fails => tally.fails += 1,
            Expected::Reads(_) => tally.reads += 1,
        }
    }
    assert!(
        tally.refused > 1500 && tally.fails > 300 && tally.reads > 1500,
        "{tally:?}"
    );
}

/// The same over bytes: any byte after a backslash, any byte alone. 6000 strings.
#[test]
fn generated_byte_strings_read_as_the_reference_decodes_them_or_are_refused() {
    let mut rng = Rng(0x00C0_FFEE_5EED_0002);
    let mut tally = Tally::default();
    let mut not_utf8 = 0;
    for case in 0..6000 {
        let mut document = vec![b'"'];
        document.extend(content(&mut rng, case, false));
        document.push(b'"');
        let expected = reference(&document);
        let shown = String::from_utf8_lossy(&document).into_owned();
        assert_eq!(
            scan(&document, true).is_err(),
            expected == Expected::Refused,
            "`{shown}`"
        );
        let got = from_slice_jsonc::<String>(&document).ok();
        let wanted = match &expected {
            Expected::Reads(text) => Some(text.clone()),
            Expected::Refused | Expected::Fails => None,
        };
        assert_eq!(got, wanted, "`{shown}` {document:?}");
        not_utf8 += usize::from(std::str::from_utf8(&document).is_err());
        match expected {
            Expected::Refused => tally.refused += 1,
            Expected::Fails => tally.fails += 1,
            Expected::Reads(_) => tally.reads += 1,
        }
    }
    assert!(
        tally.refused > 1500 && tally.fails > 300 && tally.reads > 1000 && not_utf8 > 500,
        "{tally:?}, {not_utf8} not UTF-8"
    );
}

// --- whole documents -------------------------------------------------------------------------

/// A string token: mostly one facet reads, sometimes one the scan refuses.
fn string_token(rng: &mut Rng, case: usize) -> String {
    let mut bytes = Vec::new();
    for _ in 0..rng.below(4) {
        if rng.chance(8) {
            piece(rng, case, true, &mut bytes);
        } else {
            let safe = rng.pick(&["a", "k", "é", "\\n", "\\\"", "\\u00e9", "\\\\", "x y", "😀"]);
            bytes.extend_from_slice(safe.as_bytes());
        }
    }
    let text = String::from_utf8(bytes).unwrap();
    // A quote of the content would end the token: this generator wants one token.
    format!("\"{}\"", text.replace('"', "'"))
}

/// The tokens of a JSON value, nested up to `depth`.
fn value_tokens(rng: &mut Rng, case: usize, depth: usize, out: &mut Vec<String>) {
    match rng.below(if depth == 0 { 4 } else { 6 }) {
        0 => out.push(string_token(rng, case)),
        1 => out.push(
            rng.pick(&["0", "-1", "2.5", "1e3", "-0.0", "12345678901234567890"])
                .into(),
        ),
        2 => out.push(rng.pick(&["true", "false", "null"]).into()),
        3 => out.push(rng.pick(&["{", "["]).to_owned() + rng.pick(&["}", "]"])),
        4 => {
            out.push("[".into());
            for index in 0..rng.below(4) {
                if index > 0 {
                    out.push(",".into());
                }
                value_tokens(rng, case, depth - 1, out);
            }
            out.push("]".into());
        }
        _ => {
            out.push("{".into());
            for index in 0..rng.below(4) {
                if index > 0 {
                    out.push(",".into());
                }
                out.push(string_token(rng, case));
                out.push(":".into());
                value_tokens(rng, case, depth - 1, out);
            }
            out.push("}".into());
        }
    }
}

/// Something that is no token of the value: faults to put in, and content to put after.
fn stray(rng: &mut Rng, case: usize) -> String {
    match rng.below(5) {
        0 => string_token(rng, case),
        1 => rng.character().to_string(),
        _ => rng
            .pick(&[
                "{",
                "}",
                "[",
                "]",
                ":",
                ",",
                "tru",
                "nul",
                "fals",
                "-",
                "+1",
                "x",
                "'a'",
                "\"",
                "\\",
                "/",
                "/*",
                "*/",
                "//",
                "é",
                "\u{0}",
                "1.",
                ".5",
                "0x10",
                "NaN",
                "\"\\q\"",
                "\"\\é\"",
                "\"\\u00é0\"",
                "{\"a\":",
                "]]",
                "}}",
            ])
            .to_owned(),
    }
}

/// A document: a value, sometimes with faults put in, comments and white space between its
/// tokens, sometimes content after it.
fn document(rng: &mut Rng, case: usize) -> String {
    let mut tokens = Vec::new();
    value_tokens(rng, case, 3, &mut tokens);
    if rng.chance(40) {
        for _ in 0..=rng.below(2) {
            let at = rng.below(tokens.len() + 1);
            match rng.below(3) {
                0 if at < tokens.len() => {
                    tokens.remove(at);
                }
                1 if at < tokens.len() => tokens[at] = stray(rng, case),
                _ => tokens.insert(at, stray(rng, case)),
            }
        }
    }
    if rng.chance(25) {
        for _ in 0..=rng.below(3) {
            tokens.push(stray(rng, case));
        }
    }
    let mut text = String::new();
    for token in tokens {
        text.push_str(rng.pick(&[
            "",
            "",
            "",
            " ",
            "\n",
            "\t",
            "\r\n",
            " // c \"\\q\n",
            "/* \\é \" */",
            "/**/",
        ]));
        text.push_str(&token);
    }
    text.push_str(rng.pick(&["", "", "\n", " ", " // end", " /* open"]));
    text
}

/// Whether `text` has a run of digits that fits neither `u64` nor, behind a `-`, `i64`: an
/// integer too large for 64 bits, which the door reads as a double where facet alone does
/// not (`json_wide_integers_generated.rs` has those documents). A fault of the
/// generator can join two number tokens into one.
fn has_a_wide_integer(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        if !bytes[at].is_ascii_digit() {
            at += 1;
            continue;
        }
        let start = at;
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        let negative = start > 0 && bytes[start - 1] == b'-';
        let digits = &text[start..at];
        if digits.parse::<u64>().is_err() || (negative && digits.parse::<i64>().is_err()) {
            return true;
        }
    }
    false
}

/// For 8000 documents: nothing panics; a text the scan refuses is an error through every
/// reader; a text it passes reads through the door exactly as facet reads the whole text on
/// its own (the cut at the root's end changes nothing, value or error); and the content the
/// scan finds after the root is the content facet did not read. The few documents with an
/// integer too large for 64 bits are left to their own test: there the door reads a double.
#[test]
fn generated_documents_read_through_the_door_as_facet_reads_them() {
    let mut rng = Rng(0x00C0_FFEE_5EED_0003);
    let (mut refused, mut read, mut failed, mut trailing, mut wide) = (0, 0, 0, 0, 0);
    for case in 0..8000 {
        let text = document(&mut rng, case);
        let door = from_str_jsonc::<Value>(&text);
        let at = parse_at::<Value>(JsonSlice::new(&text));
        let bytes = from_slice_jsonc::<Value>(text.as_bytes());
        let strict = from_str::<Value>(&text);
        let _ = probe_envelope(JsonSlice::new(&text));
        let _ = last_wins(&text);
        let tail = trailing_content(&text);
        let Ok(scanned) = scan(text.as_bytes(), true) else {
            assert!(
                door.is_err() && at.is_err() && bytes.is_err() && tail.is_none(),
                "`{text}`"
            );
            refused += 1;
            continue;
        };
        if has_a_wide_integer(&text) {
            wide += 1;
            continue;
        }
        // The scan passed it: facet may read it whole.
        let facet = facet_json::from_str_jsonc::<Value>(&text);
        match (&door, &facet) {
            (Ok(door), Ok(facet)) => {
                assert_eq!(door, facet, "`{text}`");
                assert_eq!(at.as_ref().ok(), Some(facet), "`{text}`");
                assert_eq!(bytes.as_ref().ok(), Some(facet), "`{text}`");
                assert!(scanned.root_end.is_some(), "`{text}`");
                read += 1;
            }
            (Err(door), Err(facet)) => {
                assert_eq!(door.span, facet.span, "`{text}`");
                assert_eq!(door.kind.to_string(), facet.kind.to_string(), "`{text}`");
                assert!(at.is_err() && bytes.is_err(), "`{text}`");
                failed += 1;
            }
            _ => panic!("`{text}`: the door {door:?}, facet {facet:?}"),
        }
        if scan(text.as_bytes(), false).is_ok() {
            let facet = facet_json::from_str::<Value>(&text);
            assert_eq!(strict.as_ref().ok(), facet.as_ref().ok(), "`{text}`");
        }
        assert_eq!(tail.is_some(), scanned.trailing.is_some(), "`{text}`");
        if let (Some(end), Ok(facet)) = (scanned.root_end, &facet) {
            // The root alone is the same value: what follows was not read.
            let root = facet_json::from_str_jsonc::<Value>(&text[..end]).unwrap();
            assert_eq!(&root, facet, "`{text}`");
            trailing += usize::from(scanned.trailing.is_some());
        }
    }
    assert!(
        refused > 800 && read > 3000 && failed > 800 && trailing > 500 && wide < 200,
        "{refused} refused, {read} read, {failed} failed, {trailing} with content after, \
         {wide} with an integer beyond 64 bits"
    );
}

// --- cost ------------------------------------------------------------------------------------

/// A document of about `megabytes` MB: members with escapes, text outside ASCII, comments,
/// nesting and numbers.
fn large_document(megabytes: usize) -> String {
    let member = "  \"minecraft:entry_NNNNNNNN\": { \"text\": \"caf\\u00e9 \\\"é\\\" \\n\\t€ 😀 \
                  C:\\\\path\\/x\", \"list\": [1, 2.5, -3e2, true, null, \"a\\\\b\"] }, // note \
                  \\q \"\n";
    let mut text = String::with_capacity(megabytes * 1_050_000);
    text.push_str("{\n");
    let mut index = 0u32;
    while text.len() < megabytes * 1_000_000 {
        text.push_str(&member.replace("NNNNNNNN", &format!("{index:08}")));
        index += 1;
    }
    text.push_str("  \"last\": 0\n}\n");
    text
}

/// The scan of a 5 MB document against one of 20 MB, by the crate's one protocol: four
/// times the text takes about four times the time (a quadratic scan takes sixteen times).
#[test]
fn the_scan_of_a_five_megabyte_document_is_linear() {
    cost::assert_linear_at("the scan, megabytes", 5, large_document, |text| {
        assert!(text.len() >= 5_000_000);
        scan(text.as_bytes(), true).unwrap().root_end.unwrap()
    });
}

/// The scan is a fraction of the parse it guards. Two times of one document compared under
/// the lock of the cost tests, not a bound on growth: the scan takes about a tenth of
/// facet's parse in a debug build (94 ms against 1 s for 5 MB) and is required to stay under
/// half of it; the scan's time is the best of three, the parse's the one run.
#[test]
fn the_scan_is_a_fraction_of_the_parse_it_guards() {
    let text = large_document(5);
    let (scanned, (parsed, parse)) = cost::alone_for(|| {
        let scanned = (0..3)
            .map(|_| cost::time_of(|| scan(text.as_bytes(), true).unwrap()))
            .min()
            .unwrap();
        let parsed = cost::timed(|| facet_json::from_str_jsonc::<Value>(&text).unwrap());
        (scanned, parsed)
    });
    assert!(parsed.as_object().unwrap().len() > 10_000);
    let share = scanned.as_secs_f64() / parse.as_secs_f64();
    eprintln!(
        "scan: {scanned:?} for {} bytes; facet's parse: {parse:?} (the scan is {share:.3} of it)",
        text.len()
    );
    assert!(
        share < 0.5,
        "the scan took {scanned:?}, the parse {parse:?}: {share:.2} of it"
    );
}

/// Bytes that are not UTF-8 are read in linear time too: facet's own byte reader validates
/// the rest of a string for each of its characters, which the door does not use.
#[test]
fn a_long_string_in_bytes_that_are_not_utf8_is_read_in_linear_time() {
    let document = |characters: usize| {
        let mut bytes = b"/* \xff */ [\"\\n".to_vec();
        bytes.extend("é".repeat(characters).as_bytes());
        bytes.extend(b"\"]");
        (bytes, characters)
    };
    cost::assert_linear_at(
        "a string in bytes that are not UTF-8, characters",
        400_000,
        document,
        |(bytes, characters)| {
            let strings: Vec<String> = from_slice_jsonc(bytes).unwrap();
            assert_eq!(strings[0].len(), 2 * characters + 1);
        },
    );
}

// --- malformed text beside an integer beyond 64 bits (reviews/CORE-BIGINT minors 1, 3) -------

/// A number token of the length and the first characters of the integer `wide` that facet
/// reads on its own: its sign and first digit, zeros, `e0`.
fn stand_in(wide: &str) -> String {
    let sign = usize::from(wide.starts_with('-'));
    let zeros = "0".repeat(wide.len() - sign - 3);
    format!("{}{zeros}e0", &wide[..=sign])
}

/// The message of an error without the token it quotes: the value of a number, or the
/// content of a string (which holds the integer where one was put inside a string).
fn without_the_number(message: &str) -> String {
    for token in ["F64(", "U64(", "I64(", "String("] {
        if let Some(at) = message.find(token) {
            let end = message.rfind(')').map_or(message.len(), |n| n + 1);
            return format!("{}token{}", &message[..at], &message[end..]);
        }
    }
    message.to_owned()
}

/// For 8000 documents of the soup above with one to three integers beyond 64 bits put in at
/// any character (between tokens, inside a string, a comment, an escape, a literal): nothing
/// panics, and the door reads the text or fails on it exactly as it does with a number token
/// of the same length that fits a double in each integer's place: the same span, the same
/// words (the number's value apart). The copy facet reads in place of such a text never
/// shows.
#[test]
fn malformed_text_beside_a_wide_integer_fails_as_it_does_beside_any_number() {
    const WIDE: &[&str] = &[
        "18446744073709551616",
        "-9223372036854775809",
        "99999999999999999999999",
        "-340282366920938463463374607431768211456",
        "7000000000000000000000000000000000000000000",
    ];
    let mut rng = Rng(0x00B1_6157_5EED_0009);
    let (mut read, mut failed, mut refused, mut named) = (0, 0, 0, 0);
    for case in 0..8000 {
        let base = document(&mut rng, case);
        // The places are taken in the soup and filled from the last to the first, so no
        // integer is put inside another.
        let mut places: Vec<usize> = (0..=rng.below(3))
            .map(|_| {
                let mut at = rng.below(base.len() + 1);
                while !base.is_char_boundary(at) {
                    at -= 1;
                }
                at
            })
            .collect();
        places.sort_unstable_by(|a, b| b.cmp(a));
        let (mut edited, mut reference) = (base.clone(), base);
        for at in places {
            let wide = rng.pick(WIDE);
            edited.insert_str(at, &format!(" {wide} "));
            reference.insert_str(at, &format!(" {} ", stand_in(wide)));
        }
        assert_eq!(edited.len(), reference.len());
        let door = from_str_jsonc::<Value>(&edited);
        let at = parse_at::<Value>(JsonSlice::new(&edited));
        let bytes = from_slice_jsonc::<Value>(edited.as_bytes());
        let _ = from_str::<Value>(&edited);
        let _ = probe_envelope(JsonSlice::new(&edited));
        let _ = last_wins(&edited);
        // The reference has no integer beyond 64 bits that the generator produced: the door reads
        // it as the tests above say.
        let expected = from_str_jsonc::<Value>(&reference);
        match (&door, &expected) {
            (Ok(_), Ok(_)) => {
                assert!(at.is_ok() && bytes.is_ok(), "`{edited}`");
                read += 1;
            }
            (Err(door), Err(expected)) => {
                assert_eq!(door.span, expected.span, "`{edited}`");
                let message = door.kind.to_string();
                assert_eq!(
                    without_the_number(&message),
                    without_the_number(&expected.kind.to_string()),
                    "`{edited}`"
                );
                assert!(at.is_err() && bytes.is_err(), "`{edited}`");
                refused += usize::from(scan(edited.as_bytes(), true).is_err());
                named += usize::from(message.contains("F64("));
                failed += 1;
            }
            _ => panic!("`{edited}`: the door {door:?}, with a plain number {expected:?}"),
        }
    }
    assert!(
        read > 300 && failed > 3000 && refused > 300 && named > 100,
        "{read} read, {failed} failed ({refused} refused by the scan, {named} naming the number)"
    );
}

/// The cases the soup may miss, by hand: a fault before the integer leaves its digits alone
/// in the message.
#[test]
fn a_fault_before_a_wide_integer_names_the_author_s_characters() {
    let message = |text: &str| from_str_jsonc::<Value>(text).unwrap_err().kind.to_string();
    for (text, names) in [
        ("tru18446744073709551616", '1'),
        ("[nul99999999999999999999]", '9'),
        ("[fals-9223372036854775809]", '-'),
        ("fals/**/true", '/'),
        ("[tru// c\n, 18446744073709551616]", '/'),
    ] {
        assert_eq!(
            message(text),
            format!("unexpected character '{names}', expected valid JSON token"),
            "`{text}`"
        );
    }
    // Where the integer itself is what should not be there, it is named as the number it is.
    assert_eq!(
        message("[1 18446744073709551616]"),
        "unexpected token: got F64(1.8446744073709552e19), expected ',' or ']'"
    );
}
