//! The text parser (`json::parse`) against the door it replaced: facet-json's read of a text
//! as a `Value` (`from_str_borrowed_jsonc::<Value>`, which still reads through the scan and
//! facet's builder; `from_str_jsonc::<Value>` is `parse` itself now), with the text rewrites of
//! `duplicates` around it.
//!
//! The door is the oracle. Every test feeds one text to both and requires the same value (the
//! same keys in the same order, the same tag and bits for every number) or the same refusal (the
//! span and the kind, in facet-json's words). The texts are real ones (every fixture of the
//! add-on crates and, when `BEDROCK_ADDON_CORPUS` is set, every JSON file of the game's packs),
//! the string literals of the pins of `json`, those and the files cut and cut again at every
//! place and with a stray byte put in, and every spelling of a number up to a length.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bedrock_addon_core::diagnostic::{JsonPath, JsonPathSegment, Span};
use bedrock_addon_core::json::parse::{
    Duplicates, Fault, NumberKind, Options, Parsed, SpanKind, parse, parse_slice,
};
use bedrock_addon_core::json::{
    JsonSlice, Value, Winner, blank_repeats, first_wins, from_str_borrowed_jsonc, last_wins,
    last_wins_except, span_at, span_reached, trailing_content,
};
use bedrock_addon_test_support::corpus;

// ---------------------------------------------------------------------------------------------
// The two readers, and what makes them agree
// ---------------------------------------------------------------------------------------------

/// What a refusal says: where, and what facet-json names it.
#[derive(Debug, PartialEq, Eq)]
struct Refusal {
    span: Option<(usize, usize)>,
    kind: String,
    words: String,
}

fn door(text: &str) -> Result<Value, Refusal> {
    from_str_borrowed_jsonc::<Value>(text).map_err(|error| Refusal {
        span: error
            .span
            .map(|span| (span.offset as usize, span.len as usize)),
        kind: format!("{:?}", error.kind),
        words: error.kind.to_string(),
    })
}

fn refusal(fault: Fault) -> Refusal {
    let words = fault.kind.to_string();
    let error = facet_json::DeserializeError::from(fault);
    Refusal {
        span: error
            .span
            .map(|span| (span.offset as usize, span.len as usize)),
        kind: format!("{:?}", error.kind),
        words,
    }
}

fn ours<'t>(text: &'t str, options: Options<'_>) -> Result<Parsed<'t>, Refusal> {
    parse(text, options).map_err(refusal)
}

/// Whether two values are the same down to the tag and the bits of every number, and the order
/// of every object's keys (`PartialEq` of a `Value` is looser: `0` equals `-0.0`).
fn identical(a: &Value, b: &Value) -> bool {
    if let (Some(a), Some(b)) = (a.as_number(), b.as_number()) {
        return a.is_float() == b.is_float()
            && a.to_i64() == b.to_i64()
            && a.to_u64() == b.to_u64()
            && a.to_f64_lossy().to_bits() == b.to_f64_lossy().to_bits();
    }
    if let (Some(a), Some(b)) = (a.as_array(), b.as_array()) {
        return a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| identical(a, b));
    }
    if let (Some(a), Some(b)) = (a.as_object(), b.as_object()) {
        return a.len() == b.len()
            && a.iter()
                .zip(b.iter())
                .all(|((ka, a), (kb, b))| ka.as_str() == kb.as_str() && identical(a, b));
    }
    a.is_number() == b.is_number()
        && a.is_array() == b.is_array()
        && a.is_object() == b.is_object()
        && a == b
}

/// The first line of the difference between the reader's answers for `text`, if they differ.
fn difference(
    text: &str,
    door: &Result<Value, Refusal>,
    ours: &Result<Parsed<'_>, Refusal>,
) -> Option<String> {
    match (door, ours) {
        (Ok(door), Ok(ours)) if identical(door, &ours.value) => None,
        (Ok(door), Ok(ours)) => Some(format!(
            "{}: the door reads {door:?}, the parser {:?}",
            shown(text),
            ours.value
        )),
        (Err(door), Err(ours)) if door == ours => None,
        (door, ours) => Some(format!(
            "{}: the door answers {}, the parser {}",
            shown(text),
            summary(door),
            summary(ours)
        )),
    }
}

fn summary<T>(answer: &Result<T, Refusal>) -> String
where
    T: std::fmt::Debug,
{
    match answer {
        Ok(value) => format!("{value:?}"),
        Err(refusal) => format!("{refusal:?}"),
    }
}

fn shown(text: &str) -> String {
    let mut shown: String = text.chars().take(160).collect();
    if shown.len() < text.len() {
        shown.push_str("...");
    }
    format!("{shown:?}")
}

/// How many times the default amount of damage to read: `BEDROCK_PARSE_FUZZ=20` for a long run.
fn scale() -> usize {
    std::env::var("BEDROCK_PARSE_FUZZ")
        .ok()
        .and_then(|scale| scale.parse().ok())
        .unwrap_or(1)
        .max(1)
}

/// What a run of comparisons counted.
#[derive(Default)]
struct Tally {
    texts: usize,
    accepted: usize,
    refused: usize,
    differed: Vec<String>,
}

impl Tally {
    /// Compares the readers on `text`, the parser at its default (every copy of a key kept, as
    /// the door reads them).
    fn agree(&mut self, text: &str) {
        self.texts += 1;
        let door = door(text);
        let ours = ours(text, Options::default());
        match &door {
            Ok(_) => self.accepted += 1,
            Err(_) => self.refused += 1,
        }
        if let Some(difference) = difference(text, &door, &ours) {
            self.differed.push(difference);
        }
    }

    fn report(&self, what: &str) {
        eprintln!(
            "{what}: {} texts, {} read, {} refused, {} differ",
            self.texts,
            self.accepted,
            self.refused,
            self.differed.len()
        );
    }

    #[track_caller]
    fn assert_none_differ(&self, what: &str) {
        self.report(what);
        assert!(
            self.differed.is_empty(),
            "the parser differs from the door on {} of {} texts of {what}; the first: \n{}",
            self.differed.len(),
            self.texts,
            self.differed
                .iter()
                .take(8)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The texts
// ---------------------------------------------------------------------------------------------

fn addon_crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

/// Every file below `root` whose name `keep` takes, sorted; folders named `target` or `.git`
/// are not entered.
fn files_below(root: &Path, keep: &dyn Fn(&Path) -> bool) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut seen = BTreeSet::new();
    let mut open = vec![root.to_path_buf()];
    while let Some(folder) = open.pop() {
        // A folder is read once, whatever links lead to it (the corpus is a folder of links).
        if !fs::canonicalize(&folder).is_ok_and(|real| seen.insert(real)) {
            continue;
        }
        let Ok(entries) = fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            // `metadata` follows a link; a link that leads nowhere is neither.
            let Ok(kind) = fs::metadata(&path).map(|meta| meta.file_type()) else {
                continue;
            };
            if kind.is_dir() {
                if !matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("target" | ".git")
                ) {
                    open.push(path);
                }
            } else if keep(&path) {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// Every `*.json` below a `tests/fixtures` folder of the add-on crates.
fn fixture_files() -> Vec<PathBuf> {
    files_below(&addon_crates(), &|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
            && path.components().any(|part| part.as_os_str() == "fixtures")
            && path.components().any(|part| part.as_os_str() == "tests")
    })
}

fn read_text(path: &Path) -> Option<String> {
    fs::read(path)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
}

/// The string literals of a Rust source, as the strings they spell: what the pins of the door
/// feed it, and a good deal that is not JSON, which both readers refuse alike.
fn literals(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut found = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = source[i + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |n| i + 2 + n + 2);
            }
            // A character literal (`'"'`, `'\''`) holds a quote that starts no string; a lifetime
            // is a tick and a name.
            b'\'' => {
                if bytes.get(i + 1) == Some(&b'\\') {
                    i += 2;
                    while i < bytes.len() && bytes[i] != b'\'' {
                        i += 1;
                    }
                    i += 1;
                } else if bytes.get(i + 2) == Some(&b'\'') || bytes.get(i + 3) == Some(&b'\'') {
                    i = source[i + 1..]
                        .find('\'')
                        .map_or(bytes.len(), |n| i + 1 + n + 1);
                } else {
                    i += 1;
                }
            }
            b'r' | b'b' if raw_string_at(bytes, i).is_some() => {
                if let Some((text, end)) = raw_string_at(bytes, i).and_then(|(hashes, from)| {
                    let close = format!("\"{}", "#".repeat(hashes));
                    let at = source[from..].find(&close)?;
                    Some((source[from..from + at].to_owned(), from + at + close.len()))
                }) {
                    found.push(text);
                    i = end;
                } else {
                    i += 1;
                }
            }
            b'"' => {
                let (text, end) = quoted(source, i + 1);
                found.push(text);
                i = end;
            }
            _ => i += 1,
        }
    }
    found
}

/// `r#"` (or `br#"`) at `i`: the number of hashes and where the text starts.
fn raw_string_at(bytes: &[u8], i: usize) -> Option<(usize, usize)> {
    let mut at = i;
    if bytes.get(at) == Some(&b'b') {
        at += 1;
    }
    if bytes.get(at) != Some(&b'r') {
        return None;
    }
    // The `r` must start a token, not end an identifier.
    if i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_') {
        return None;
    }
    at += 1;
    let hashes = bytes[at..].iter().take_while(|&&byte| byte == b'#').count();
    at += hashes;
    (bytes.get(at) == Some(&b'"')).then_some((hashes, at + 1))
}

/// The string literal whose text starts at `from` (after the opening quote): what it spells
/// and where it ends.
fn quoted(source: &str, from: usize) -> (String, usize) {
    let bytes = source.as_bytes();
    let mut out = String::new();
    let mut i = from;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => return (out, i + 1),
            b'\\' => {
                i += 1;
                match bytes.get(i) {
                    Some(b'n') => out.push('\n'),
                    Some(b'r') => out.push('\r'),
                    Some(b't') => out.push('\t'),
                    Some(b'0') => out.push('\0'),
                    Some(b'\\') => out.push('\\'),
                    Some(b'"') => out.push('"'),
                    Some(b'\'') => out.push('\''),
                    Some(b'x') => {
                        let digits = source.get(i + 1..i + 3).unwrap_or("0");
                        out.push(char::from(u8::from_str_radix(digits, 16).unwrap_or(b'?')));
                        i += 2;
                    }
                    Some(b'u') => {
                        let close = source[i..].find('}').map_or(i, |n| i + n);
                        let digits = source.get(i + 2..close).unwrap_or("0").replace('_', "");
                        out.push(
                            u32::from_str_radix(&digits, 16)
                                .ok()
                                .and_then(char::from_u32)
                                .unwrap_or('?'),
                        );
                        i = close;
                    }
                    // A backslash at the end of a line takes the white space after it.
                    Some(b'\n' | b'\r') => {
                        while bytes.get(i + 1).is_some_and(u8::is_ascii_whitespace) {
                            i += 1;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
            _ => {
                let ch = source[i..].chars().next().unwrap_or('?');
                out.push(ch);
                i += ch.len_utf8();
            }
        }
    }
    (out, bytes.len())
}

/// Every string literal of the sources of the pins of the door: `tests/json_*.rs` and the
/// tests of `src/json`, and the characterization texts.
fn pin_literals() -> BTreeSet<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sources = files_below(root, &|path| {
        path.extension().is_some_and(|extension| extension == "rs")
            && (path.starts_with(root.join("src/json")) || path.starts_with(root.join("tests")))
    });
    sources
        .iter()
        .filter_map(|path| read_text(path))
        .flat_map(|source| literals(&source))
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Differential 1: real files
// ---------------------------------------------------------------------------------------------

#[test]
fn every_fixture_of_the_addon_crates_reads_as_the_door_reads_it() {
    let files = fixture_files();
    assert!(files.len() > 100, "found {} fixtures", files.len());
    let mut tally = Tally::default();
    for path in &files {
        let Some(text) = read_text(path) else {
            continue;
        };
        tally.agree(&text);
    }
    assert!(tally.accepted > 100, "{} fixtures read", tally.accepted);
    tally.assert_none_differ("the fixtures of the add-on crates");
}

#[test]
fn every_json_file_of_the_corpus_reads_as_the_door_reads_it() {
    let Ok(root) = std::env::var(corpus::CORPUS) else {
        corpus::skipped(
            "the parser over every JSON file of the corpus",
            corpus::not_set(corpus::CORPUS),
        );
        return;
    };
    let files = files_below(Path::new(&root), &|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    let mut tally = Tally::default();
    let mut last = Tally::default();
    for path in &files {
        let Some(text) = read_text(path) else {
            continue;
        };
        tally.agree(&text);
        // The jsoncpp readers read it with the last copy of a repeated key standing.
        compare_winner(&mut last, &text, Winner::Last, &last_wins(&text));
    }
    assert!(!files.is_empty(), "{root} holds no JSON file");
    assert!(tally.texts > 0, "{root}: no JSON file could be read");
    tally.assert_none_differ("the corpus");
    last.assert_none_differ("the corpus, the last copy of a key standing");
}

// ---------------------------------------------------------------------------------------------
// Differential 2: the inputs of the pins
// ---------------------------------------------------------------------------------------------

#[test]
fn every_string_literal_of_the_pins_of_the_door_reads_as_the_door_reads_it() {
    let texts = pin_literals();
    assert!(texts.len() > 1_000, "found {} literals", texts.len());
    let mut tally = Tally::default();
    for text in &texts {
        tally.agree(text);
    }
    assert!(tally.accepted > 200 && tally.refused > 200);
    tally.assert_none_differ("the string literals of the pins");
}

/// The texts of the pins that name one dialect rule each, by hand: what the brief lists.
const RULES: &[&str] = &[
    // syntax
    "",
    " ",
    "\n\t\r ",
    "{",
    "[",
    "}",
    "]",
    ",",
    ":",
    "{}",
    "[]",
    "{\"a\"}",
    "{\"a\":}",
    "{\"a\" 1}",
    "{1:2}",
    "{\"a\":1 \"b\":2}",
    "[1 2]",
    "[1,,2]",
    "[,]",
    "{,}",
    "{\"a\":1,,\"b\":2}",
    "[1}",
    "{\"a\":1]",
    "]]",
    "{]",
    "[\"a\":1]",
    "tru",
    "truex",
    "nul",
    "nulll",
    "fals",
    "falsey",
    "True",
    "NaN",
    "Infinity",
    "-Infinity",
    "'a'",
    "\"a",
    "\"a\\",
    "\"\\",
    "\"\\\"",
    "\u{0}",
    "\"\u{0}\"",
    // comments
    "// a",
    "// a\n1",
    "/* a */ 1",
    "/* a",
    "/* a *",
    "1 /* a",
    "[1 /* a */, 2]",
    "[1 // a\n,2]",
    "/",
    "/ 1",
    "1 /",
    "{ /* a */ \"a\" /* b */ : /* c */ 1 /* d */ , /* e */ }",
    "/*/ 1",
    "/**/1",
    "// \u{e9}\n1",
    "/* \u{e9} */ 1",
    "[/*]*/1]",
    "{\"a\":/*",
    "[1,/*",
    // trailing commas
    "[1,]",
    "{\"a\":1,}",
    "[[1,],]",
    "[1,\n]",
    "{\"a\":[1,],}",
    "[1,/* c */]",
    // trailing content
    "1 2",
    "[1] [2]",
    "{} {}",
    "1 }",
    "[1]]",
    "[1],",
    "1x",
    "{}\n{",
    "null null",
    "\"a\" \"b\"",
    "[1] /* a",
    "[1] // a",
    "[1] \u{0}",
    // byte order mark
    "\u{feff}1",
    "\u{feff}{}",
    "   1",
    "\u{feff}",
    " \u{feff}1",
    // escapes
    r#""\n\t\r\b\f\"\\\/""#,
    r#""\u0041""#,
    r#""\uD83D\uDE00""#,
    r#""\uD83D""#,
    r#""\uDE00""#,
    r#""\uD83D\u0041""#,
    r#""\uD83Dx""#,
    r#""\uD83D\n""#,
    r#""\u00""#,
    r#""\u00zz""#,
    r#""\u""#,
    r#""\q""#,
    r#""\é""#,
    r#""\'""#,
    r#""\0""#,
    r#""\x41""#,
    r#""\u+041""#,
    r#""\u 041""#,
    r#""\uD83D\uDE0""#,
    r#""\uD83D\uDE00\uD83D\uDE00""#,
    r#"{"\q":1}"#,
    r#"{"a":1,"\q":1}"#,
    r#"[1,"\q"]"#,
    r#"[1 "\q"]"#,
    r#"{"a" "\q"}"#,
    r#"{"a":1 "\q"}"#,
    r#"["\q" 1]"#,
    r#"{"\uD800":1}"#,
    r#"{"\uD800" 1}"#,
    r#"[1 "\uD800"]"#,
    r#"{"a\\":"b\""}"#,
    "\"\u{e9}\u{1F600}\"",
    "\"a\nb\"",
    "\"\t\"",
    // numbers
    "0",
    "-0",
    "-0.0",
    "0.0",
    "0e0",
    "-0e0",
    "1",
    "-1",
    "1.5",
    "-1.5",
    "1e3",
    "1E3",
    "1e+3",
    "1e-3",
    "1.e3",
    "01",
    "00",
    "-01",
    "1.",
    "-1.",
    "1e",
    "1e+",
    "-",
    "--1",
    "+1",
    ".5",
    "-.5",
    "1.2.3",
    "1e2e3",
    "1e1.5",
    "0x10",
    "1_000",
    "12abc",
    "9223372036854775807",
    "9223372036854775808",
    "-9223372036854775808",
    "-9223372036854775809",
    "18446744073709551615",
    "18446744073709551616",
    "-18446744073709551616",
    "340282366920938463463374607431768211455",
    "340282366920938463463374607431768211456",
    "123456789012345678901234567890",
    "-123456789012345678901234567890",
    "1e308",
    "1e309",
    "-1e309",
    "1e999",
    "-1e999",
    "1e-999",
    "1.7976931348623157e308",
    "1.7976931348623159e308",
    "2.2250738585072014e-308",
    "5e-324",
    "4.9e-325",
    "0.1",
    "0.30000000000000004",
    "100000000000000000000",
    "1e20",
    "1e21",
    "[18446744073709551616, -18446744073709551616, 1e999]",
    "{\"a\":123456789012345678901234567890}",
    "[1 123456789012345678901234567890]",
    "{\"a\" 123456789012345678901234567890}",
    "{123456789012345678901234567890:1}",
    "[123456789012345678901234567890 1]",
    "0000000000000000000000000000000000001",
    "-0000000000000000000000000000000000001",
    "00000000000000000000000000000000000000000000000000000000000000000000000000000000",
    // duplicates, as written (merged)
    "{\"a\":1,\"a\":2}",
    "{\"a\":{\"b\":1},\"a\":{\"c\":2}}",
    "{\"a\":[1],\"a\":[2]}",
    "{\"a\":{\"b\":1},\"a\":3}",
    "{\"a\":3,\"a\":{\"b\":1}}",
    "{\"a\":[1],\"a\":{\"b\":1}}",
    "{\"a\":{\"b\":1},\"a\":[1]}",
    "{\"a\":1,\"b\":2,\"a\":3}",
    "{\"a\":null,\"a\":[]}",
    "{\"a\":{\"x\":{\"p\":1}},\"a\":{\"x\":{\"q\":1}},\"a\":{\"x\":3}}",
    "{\"a\":1,\"a\":",
    "{\"a\":1,\"\\u0061\":2}",
    "{\"a\":1,\"a\" 2}",
    "{\"a\":1,\"a\":2,}",
    "{\"a\":1,,\"a\":2}",
];

#[test]
fn the_rules_of_the_dialect_read_as_the_door_reads_them() {
    let mut tally = Tally::default();
    for text in RULES {
        tally.agree(text);
    }
    tally.assert_none_differ("the rules of the dialect");
}

#[test]
fn nesting_to_the_limit_is_read_and_one_level_more_is_refused_as_the_door_does() {
    let mut tally = Tally::default();
    for depth in [1, 2, 127, 128, 129, 130, 200] {
        for (open, close) in [("[", "]"), ("{\"a\":", "}")] {
            let deep = format!("{}1{}", open.repeat(depth), close.repeat(depth));
            tally.agree(&deep);
            // Then a fault inside the limit before the bracket that is past it.
            tally.agree(&format!("{}1 2{}", open.repeat(depth), close.repeat(depth)));
            tally.agree(&format!("{}{}", open.repeat(depth), "]".repeat(depth)));
            // And the limit past a string with a bad escape, or a bracket where none goes.
            tally.agree(&format!("{}\"\\q\"", open.repeat(depth)));
            tally.agree(&format!("{}[1 [", open.repeat(depth)));
        }
        let mixed = (0..depth)
            .map(|level| if level % 2 == 0 { "[" } else { "{\"k\":" })
            .collect::<String>();
        tally.agree(&format!("{mixed}0"));
    }
    tally.assert_none_differ("the nesting limit");
    let at_limit = format!("{}{}", "[".repeat(128), "]".repeat(128));
    assert!(parse(&at_limit, Options::default()).is_ok());
    let past = format!("{}{}", "[".repeat(129), "]".repeat(129));
    let fault = parse(&past, Options::default()).unwrap_err();
    assert_eq!((fault.span.offset, fault.span.len), (128, 1));
}

#[test]
fn what_follows_the_root_value_is_never_read_and_trailing_content_says_where_it_lies() {
    for text in [
        "1 2",
        "[1] [2]",
        "{} {",
        "[1] /* a",
        "[1] // a",
        "[1]  x  ",
        "[1] /* a */ // b\n \"c\"  ",
        "1",
        "1 ",
        "1 // a",
        "1 /* a */",
        "{\"a\":1}\n\n",
        "\"é\" é é  ",
        "[1]/**/",
    ] {
        let parsed = parse(text, Options::default()).unwrap();
        assert!(identical(&door(text).unwrap(), &parsed.value), "{text:?}");
        let door = trailing_content(text).map(|span| (span.offset, span.len));
        let ours = parsed.trailing.map(|span| (span.offset, span.len));
        assert_eq!(ours, door, "{text:?}");
    }
}

// ---------------------------------------------------------------------------------------------
// Differential 3: every cut and every stray byte
// ---------------------------------------------------------------------------------------------

/// The text cut at `at` places, and with each of the stray bytes in `junk` put in and the
/// character taken out, at the same places.
fn damaged(text: &str, places: usize, junk: &[&str]) -> Vec<String> {
    let boundaries: Vec<usize> = (0..=text.len())
        .filter(|&at| text.is_char_boundary(at))
        .collect();
    let step = (boundaries.len() / places).max(1);
    let mut out = Vec::new();
    for &at in boundaries.iter().step_by(step) {
        out.push(text[..at].to_owned());
        if let Some(ch) = text[at..].chars().next() {
            out.push(format!("{}{}", &text[..at], &text[at + ch.len_utf8()..]));
        }
        for stray in junk {
            out.push(format!("{}{stray}{}", &text[..at], &text[at..]));
        }
    }
    out
}

/// The strays: each one starts a token, ends a string or container, or is a byte that is one of
/// no token.
const JUNK: &[&str] = &[
    ",",
    ":",
    "\"",
    "\\",
    "}",
    "]",
    "{",
    "[",
    "/",
    "/*",
    "//",
    "-",
    "0",
    "e",
    ".",
    "t",
    "n",
    "\n",
    "\u{e9}",
    "\\u",
    "\\q",
    "\u{feff}",
    "1e999",
    "123456789012345678901234567890",
];

#[test]
fn a_text_cut_anywhere_or_with_a_stray_byte_is_read_or_refused_as_the_door_does() {
    let mut seeds: Vec<String> = pin_literals()
        .into_iter()
        .filter(|text| text.len() <= 240 && text.contains(['{', '[', '"', ':']))
        .collect();
    seeds.extend(RULES.iter().map(|text| (*text).to_owned()));
    let stride = (seeds.len() / (700 * scale())).max(1);
    let mut tally = Tally::default();
    for seed in seeds.iter().step_by(stride) {
        for text in damaged(seed, 20 * scale(), &JUNK[..14]) {
            tally.agree(&text);
        }
    }
    tally.assert_none_differ("the cut and damaged literals");
}

#[test]
fn a_fixture_cut_anywhere_or_with_a_stray_byte_is_read_or_refused_as_the_door_does() {
    let files = fixture_files();
    let mut tally = Tally::default();
    let stride = (files.len() / (60 * scale())).max(1);
    for path in files.iter().step_by(stride) {
        let Some(text) = read_text(path) else {
            continue;
        };
        if text.len() > 20_000 {
            continue;
        }
        for text in damaged(&text, 12 * scale(), JUNK) {
            tally.agree(&text);
        }
    }
    tally.assert_none_differ("the cut and damaged fixtures");
}

/// The pieces of a text, each a token or a bit of one: every sequence of the first four and
/// a spread of longer ones are read.
const PIECES: &[&str] = &[
    "{",
    "}",
    "[",
    "]",
    ",",
    ":",
    "\"a\"",
    "\"\\u0041\"",
    "\"\\q\"",
    "\"\\ud800\"",
    "1",
    "-1",
    "1.5",
    "1e999",
    "123456789012345678901234567890",
    "true",
    "null",
    "nul",
    "/*c*/",
    "//c\n",
    "/*",
    "\"a",
    "-",
    "x",
];

#[test]
fn every_short_sequence_of_tokens_and_a_spread_of_long_ones_read_as_the_door_reads_them() {
    let mut tally = Tally::default();
    let mut texts = Vec::new();
    for length in 1..=4 {
        let mut counter = vec![0usize; length];
        'sequences: loop {
            texts.push(counter.iter().map(|&at| PIECES[at]).collect::<String>());
            texts.push(
                counter
                    .iter()
                    .map(|&at| PIECES[at])
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            let mut at = length;
            while at > 0 {
                at -= 1;
                counter[at] += 1;
                if counter[at] < PIECES.len() {
                    continue 'sequences;
                }
                counter[at] = 0;
            }
            break;
        }
    }
    // A generator of the spread: a fixed multiplier, no clock.
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut next = |bound: usize| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(state >> 33).unwrap_or(0) % bound
    };
    for _ in 0..60_000 * scale() {
        let length = 5 + next(8);
        let joiner = if next(2) == 0 { "" } else { " " };
        let text: Vec<&str> = (0..length).map(|_| PIECES[next(PIECES.len())]).collect();
        texts.push(text.join(joiner));
    }
    for text in &texts {
        tally.agree(text);
    }
    tally.assert_none_differ("sequences of tokens");
}

#[test]
fn every_spelling_of_a_number_up_to_a_length_is_read_as_the_door_reads_it() {
    let alphabet = ['-', '+', '0', '1', '9', '.', 'e', 'E'];
    let longest = if std::env::var("BEDROCK_PARSE_LONG").is_ok() {
        7
    } else {
        5
    };
    let mut tally = Tally::default();
    let mut spelling = vec![0usize];
    let mut count = 0usize;
    // Every string of the alphabet of 1 to `longest` characters, in counting order.
    let mut length = 1;
    while length <= longest {
        let mut digits = vec![0usize; length];
        'next: loop {
            let token: String = digits.iter().map(|&d| alphabet[d]).collect();
            tally.agree(&token);
            // And in the places a token has neighbours.
            if count.is_multiple_of(7) {
                tally.agree(&format!("[{token}]"));
                tally.agree(&format!("[{token} 1]"));
                tally.agree(&format!("{{\"a\":{token}}}"));
            }
            count += 1;
            let mut at = length;
            while at > 0 {
                at -= 1;
                digits[at] += 1;
                if digits[at] < alphabet.len() {
                    continue 'next;
                }
                digits[at] = 0;
            }
            break;
        }
        length += 1;
    }
    spelling.clear();
    tally.assert_none_differ("the spellings of a number");
}

#[test]
fn long_decimals_round_to_the_double_the_door_rounds_them_to() {
    let mut state = 777_u64;
    let mut tally = Tally::default();
    for _ in 0..30_000 * scale() {
        let integer_digits = draw(&mut state, 22);
        let fraction_digits = draw(&mut state, 40);
        let mut token = String::new();
        if draw(&mut state, 3) == 0 {
            token.push('-');
        }
        for _ in 0..integer_digits.max(1) {
            token.push(char::from(
                b'0' + u8::try_from(draw(&mut state, 10)).unwrap_or(0),
            ));
        }
        if fraction_digits > 0 {
            token.push('.');
            for _ in 0..fraction_digits {
                token.push(char::from(
                    b'0' + u8::try_from(draw(&mut state, 10)).unwrap_or(0),
                ));
            }
        }
        match draw(&mut state, 4) {
            0 => {
                let exponent = draw(&mut state, 700);
                let sign = if exponent < 350 { "-" } else { "" };
                let magnitude = exponent.abs_diff(350);
                token = format!("{token}e{sign}{magnitude}");
            }
            1 => token = format!("{token}E+{}", draw(&mut state, 40)),
            _ => {}
        }
        tally.agree(&token);
    }
    for halfway in [
        "9007199254740993",
        "9007199254740993.0",
        "9007199254740992.5",
        "2.2250738585072011e-308",
        "2.2250738585072012e-308",
        "4.9406564584124654e-324",
        "2.4703282292062328e-324",
        "2.4703282292062327e-324",
        "1.7976931348623158e308",
        "1.7976931348623159e308",
        "0.000000000000000000000000000000000000000000000000000000000000000001",
    ] {
        tally.agree(halfway);
    }
    tally.assert_none_differ("long decimals");
}

#[test]
fn the_edges_of_the_integer_widths_read_as_the_door_reads_them() {
    let mut tally = Tally::default();
    let mut edges: Vec<String> = Vec::new();
    for base in [
        "9223372036854775807",
        "9223372036854775808",
        "18446744073709551615",
        "18446744073709551616",
        "170141183460469231731687303715884105727",
        "170141183460469231731687303715884105728",
        "340282366920938463463374607431768211455",
        "340282366920938463463374607431768211456",
        "9007199254740993",
        "999999999999999999",
        "1000000000000000000",
        "0000000000000000000000000009223372036854775807",
    ] {
        edges.push(base.to_owned());
        edges.push(format!("-{base}"));
        edges.push(format!("{base}.0"));
        edges.push(format!("{base}e0"));
        edges.push(format!("[{base}]"));
        edges.push(format!("{{\"a\":{base},\"a\":{base}}}"));
    }
    edges.push("9".repeat(308));
    edges.push("9".repeat(309));
    edges.push("-".to_owned() + &"9".repeat(309));
    edges.push("1".to_owned() + &"0".repeat(308));
    edges.push("1".to_owned() + &"0".repeat(309));
    edges.push("17976931348623157".to_owned() + &"0".repeat(292));
    edges.push("17976931348623159".to_owned() + &"0".repeat(292));
    for text in &edges {
        tally.agree(text);
    }
    tally.assert_none_differ("the integer widths");
}

// ---------------------------------------------------------------------------------------------
// Repeated keys
// ---------------------------------------------------------------------------------------------

/// Texts with keys written twice at several levels, in objects and arrays, with the faults a
/// copy that does not stand may hold.
fn repeating_texts() -> Vec<String> {
    let mut texts: Vec<String> = RULES
        .iter()
        .chain(&[
            r#"{"a":1,"b":2,"a":3,"c":{"d":1,"d":2,"e":[{"f":1,"f":2},{"f":3}]},"b":4}"#,
            r#"{"components":{"x":{"p":1},"x":{"q":2},"y":1},"components":{"z":1}}"#,
            r#"{"k":{"a":1,"a":2},"k":{"a":3,"b":4}}"#,
            r#"[{"a":1,"a":2},{"a":3,"a":4,"a":5}]"#,
            r#"{"a":[{"b":1,"b":2}],"a":[{"b":3,"b":4}]}"#,
            r#"{"a":1,"a":2,"a":3,"a":4}"#,
            r#"{"a":{"a":{"a":1,"a":2},"a":{"a":3}},"a":{"a":5}}"#,
            r#"{"a":1,"a":2 "b":3}"#,
            r#"{"a":1,"a":2,"b" 3}"#,
            r#"{"a":1,"a":"\q"}"#,
            r#"{"a":"\q","a":1}"#,
            r#"{"a":1,"a":[1,,2]}"#,
            r#"{"a":[1,,2],"a":1}"#,
            r#"{"a":1,"a":2}/* c */ {"a":3}"#,
            r#"{"a":1, // c
               "a":2, /* d */ "b":3}"#,
            r#"{"a":123456789012345678901234567890,"a":1e999}"#,
            r#"{"é":1,"e\u0301":2,"é":3}"#,
            r#"{"":1,"":2}"#,
            r#"{"a":1,"a":{"b":1,"b":{"c":1,"c":2}}}"#,
        ])
        .map(|text| (*text).to_owned())
        .collect();
    // Objects with many members, past the number a scan is cheaper than a map.
    let many: String = (0..40)
        .map(|i| format!("\"k{}\":{i}", i % 25))
        .collect::<Vec<_>>()
        .join(",");
    texts.push(format!("{{{many}}}"));
    texts.push(format!("{{{many},\"k3\":[1],\"k3\":[2]}}"));
    let nested: String = (0..60)
        .map(|i| {
            format!(
                "\"k{}\":{{\"x{}\":{i},\"x{}\":{}}}",
                i % 31,
                i % 3,
                i % 3,
                i + 1
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    texts.push(format!("{{{nested}}}"));
    texts
}

#[test]
fn the_last_copy_of_a_key_stands_as_last_wins_has_it() {
    let mut tally = Tally::default();
    for text in repeating_texts() {
        compare_winner(&mut tally, &text, Winner::Last, &last_wins(&text));
    }
    tally.assert_none_differ("last wins");
}

#[test]
fn the_first_copy_of_a_key_stands_as_first_wins_has_it() {
    let mut tally = Tally::default();
    for text in repeating_texts() {
        compare_winner(&mut tally, &text, Winner::First, &first_wins(&text));
    }
    tally.assert_none_differ("first wins");
}

/// A text read as the animation loaders' schema reader reads it: each object as an `IndexMap` of its members'
/// texts (of a key written twice, the first place and the last value), each member again.
fn in_place(text: &str) -> Option<Value> {
    use facet_json::RawJson;
    let value = from_str_borrowed_jsonc::<Value>(text).ok()?;
    if value.is_object()
        && let Ok(members) =
            from_str_borrowed_jsonc::<indexmap::IndexMap<String, RawJson<'_>>>(text)
    {
        let mut object = facet_value::VObject::new();
        for (key, raw) in members {
            object.insert(key.as_str(), in_place(raw.as_ref())?);
        }
        return Some(Value::from(object));
    }
    if value.is_array()
        && let Ok(elements) = from_str_borrowed_jsonc::<Vec<RawJson<'_>>>(text)
    {
        let elements: Option<Vec<Value>> =
            elements.iter().map(|raw| in_place(raw.as_ref())).collect();
        return Some(Value::from(
            elements?.into_iter().collect::<facet_value::VArray>(),
        ));
    }
    from_str_borrowed_jsonc::<Value>(text).ok()
}

#[test]
fn the_last_copy_of_a_key_stands_at_the_first_place_as_an_indexmap_keeps_it() {
    let mut texts = repeating_texts();
    texts.push(r#"{"a":1,"b":2,"a":{"x":[1,2]},"c":3,"a":{"y":4}}"#.to_owned());
    let mut read = 0;
    for text in &texts {
        let (Some(walked), Ok(parsed)) = (
            in_place(text),
            parse(text, Options::winner(Winner::LastInPlace)),
        ) else {
            continue;
        };
        read += 1;
        assert!(
            identical(&parsed.value, &walked),
            "{text}\n{:?}\n{walked:?}",
            parsed.value
        );
        // A member's span is its last copy's.
        if let Some(object) = parsed.value.as_object() {
            for (key, _) in object {
                let mut at = JsonPath::new();
                at.push(key.as_str());
                let span = parsed.spans.at(&at).unwrap();
                let raw = &text[span.offset as usize..][..span.len as usize];
                assert_eq!(
                    in_place(raw).as_ref(),
                    object.get(key.as_str()),
                    "{text} {key}"
                );
            }
        }
    }
    assert!(read > 10, "{read}");
}

#[test]
fn every_copy_stands_as_the_door_reads_a_text_that_is_left_as_written() {
    let mut tally = Tally::default();
    for text in repeating_texts() {
        let kept = blank_repeats(&text, &|_| Winner::Keep).into_owned();
        assert_eq!(kept, text);
        compare_winner(&mut tally, &text, Winner::Keep, &kept);
    }
    tally.assert_none_differ("keep");
}

fn compare_winner(tally: &mut Tally, text: &str, winner: Winner, blanked: &str) {
    tally.texts += 1;
    let door = door(blanked);
    let ours = ours(text, Options::winner(winner));
    match &door {
        Ok(_) => tally.accepted += 1,
        Err(_) => tally.refused += 1,
    }
    if let Some(difference) = difference(text, &door, &ours) {
        tally.differed.push(format!("{winner:?}: {difference}"));
    }
}

/// The next number below `bound` of a generator with a fixed multiplier and no clock.
fn draw(state: &mut u64, bound: usize) -> usize {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    usize::try_from(*state >> 33).unwrap_or(0) % bound
}

/// A document of objects and arrays over a few keys, so that keys repeat at every level.
fn random_document(state: &mut u64, depth: usize) -> String {
    let kind = draw(state, if depth == 0 { 4 } else { 8 });
    match kind {
        0 => draw(state, 100).to_string(),
        1 => ["true", "null", "\"s\"", "1e999"][draw(state, 4)].to_owned(),
        2 => format!("[{}]", draw(state, 3)),
        3 => "{}".to_owned(),
        4 | 5 => {
            let count = draw(state, 6);
            let mut members = Vec::new();
            for _ in 0..count {
                let key = ["a", "b", "c", "\\u0061"][draw(state, 4)];
                members.push(format!("\"{key}\":{}", random_document(state, depth - 1)));
            }
            format!("{{{}}}", members.join(","))
        }
        _ => {
            let count = draw(state, 4);
            let elements: Vec<String> = (0..count)
                .map(|_| random_document(state, depth - 1))
                .collect();
            format!("[{}]", elements.join(","))
        }
    }
}

#[test]
fn random_documents_with_repeated_keys_read_as_the_door_reads_them_under_every_rule() {
    let mut state = 12345_u64;
    let mut tally = Tally::default();
    let mut texts = Vec::new();
    for _ in 0..3_000 * scale() {
        texts.push(random_document(&mut state, 4));
    }
    let cut: Vec<String> = texts
        .iter()
        .step_by(15)
        .flat_map(|text| damaged(text, 6, &JUNK[..8]).into_iter().take(40))
        .collect();
    texts.extend(cut);
    for text in &texts {
        compare_winner(&mut tally, text, Winner::Last, &last_wins(text));
        compare_winner(&mut tally, text, Winner::First, &first_wins(text));
        compare_winner(&mut tally, text, Winner::Keep, text);
    }
    tally.assert_none_differ("random documents under the three rules");
}

#[test]
fn a_rule_that_keeps_the_members_of_the_top_objects_reads_as_the_door_chain_reads_it() {
    // Keep in the root and its members, last wins below: a copy that is merged into the first
    // holds objects that last wins has read. (Keep below last wins below Keep is the one
    // arrangement the parser does not read as the door chain does; see `NOTES-parser.md`.)
    let rule = |path: &JsonPath| {
        if path.len() <= 1 {
            Winner::Keep
        } else {
            Winner::Last
        }
    };
    let mut state = 4242_u64;
    let mut tally = Tally::default();
    for _ in 0..4_000 * scale() {
        let text = random_document(&mut state, 5);
        let blanked = blank_repeats(&text, &rule);
        tally.texts += 1;
        let door = door(&blanked);
        let ours = parse_by_path(&text, &rule);
        if let Some(difference) = difference(&text, &door, &ours) {
            tally.differed.push(difference);
        }
    }
    tally.assert_none_differ("a rule by depth");
}

#[test]
fn damaged_texts_with_repeated_keys_are_refused_where_the_door_refuses_them() {
    let mut tally = Tally::default();
    for text in repeating_texts() {
        for text in damaged(&text, 40, &JUNK[..14]) {
            compare_winner(&mut tally, &text, Winner::Last, &last_wins(&text));
            compare_winner(&mut tally, &text, Winner::First, &first_wins(&text));
        }
    }
    tally.assert_none_differ("damaged texts with repeated keys");
}

#[test]
fn last_wins_except_a_named_member_is_the_parsers_last_except() {
    let keep = ["components", "k"];
    let mut tally = Tally::default();
    for text in repeating_texts() {
        let blanked = last_wins_except(&text, &keep).into_owned();
        tally.texts += 1;
        let door = door(&blanked);
        let ours = ours(&text, Options::last_except(&keep));
        if let Some(difference) = difference(&text, &door, &ours) {
            tally.differed.push(difference);
        }
        // And by path, with the rule that last_wins_except states.
        let rule = |path: &JsonPath| {
            if path.last_key().is_some_and(|key| keep.contains(&key)) {
                Winner::Keep
            } else {
                Winner::Last
            }
        };
        let by_path = parse_by_path(&text, &rule);
        if let Some(difference) = difference(&text, &door, &by_path) {
            tally.differed.push(format!("by path: {difference}"));
        }
    }
    tally.assert_none_differ("last wins except");
}

fn parse_by_path<'t>(
    text: &'t str,
    rule: &dyn Fn(&JsonPath) -> Winner,
) -> Result<Parsed<'t>, Refusal> {
    ours(text, Options::by_path(rule))
}

// ---------------------------------------------------------------------------------------------
// Spans
// ---------------------------------------------------------------------------------------------

/// The paths of the members and elements of `value`, the root's empty path first, and a few that
/// lead nowhere: a key that is not there, an index past the end, a step into a scalar.
fn paths_of(value: &Value, limit: usize) -> Vec<JsonPath> {
    let mut paths = Vec::new();
    let mut open: Vec<(JsonPath, &Value)> = vec![(JsonPath::new(), value)];
    while let Some((path, value)) = open.pop() {
        if paths.len() >= limit {
            break;
        }
        if let Some(object) = value.as_object() {
            for (key, member) in object {
                let mut below = path.clone();
                below.push(key.as_str());
                open.push((below, member));
            }
            let mut missing = path.clone();
            missing.push("\u{0}not a member");
            paths.push(missing);
        } else if let Some(array) = value.as_array() {
            for (index, element) in array.iter().enumerate() {
                let mut below = path.clone();
                below.push_index(index);
                open.push((below, element));
            }
            let mut past = path.clone();
            past.push_index(array.len());
            paths.push(past);
            let mut wrong = path.clone();
            wrong.push("a");
            paths.push(wrong);
        } else {
            let mut into = path.clone();
            into.push_index(0);
            paths.push(into);
        }
        paths.push(path);
    }
    paths
}

/// The spans of `text` at the paths of its value are what `span_at` and `span_reached` say of
/// the text, the file offset of the slice added.
fn assert_spans_are_span_ats(text: &str, offset: usize, options: Options<'_>) -> usize {
    let slice = JsonSlice { text, offset };
    let Ok(parsed) = parse_slice(slice, options) else {
        return 0;
    };
    // `span_at` takes the last copy of a key; where the first copy stands, it is asked of the
    // text in which the later copies are blank.
    let first;
    let oracle = if matches!(options.duplicates, Duplicates::Uniform(Winner::First)) {
        first = first_wins(text);
        JsonSlice {
            text: &first,
            offset,
        }
    } else {
        slice
    };
    let paths = paths_of(&parsed.value, 30);
    for path in &paths {
        assert_eq!(
            parsed.spans.at(path),
            span_at(oracle, path),
            "{}: the span at {path}",
            shown(text)
        );
        assert_eq!(
            parsed.spans.reached(path),
            span_reached(oracle, path),
            "{}: the span reached at {path}",
            shown(text)
        );
    }
    paths.len()
}

#[test]
fn the_spans_of_the_fixtures_are_what_span_at_says() {
    let files = fixture_files();
    let mut compared = 0;
    let mut documents = 0;
    for path in &files {
        let Some(text) = read_text(path) else {
            continue;
        };
        // `span_at` reads the text once for each step of a path.
        if text.len() > 2_500 {
            continue;
        }
        let paths = assert_spans_are_span_ats(&text, 0, Options::default())
            + assert_spans_are_span_ats(&text, 1234, Options::winner(Winner::Last));
        documents += usize::from(paths > 0);
        compared += paths;
    }
    eprintln!("spans: {documents} documents, {compared} paths");
    assert!(documents > 100 && compared > 2_000);
}

#[test]
fn the_spans_of_a_spread_of_texts_are_what_span_at_says() {
    let mut compared = 0;
    for text in RULES
        .iter()
        .map(|text| (*text).to_owned())
        .chain(repeating_texts())
    {
        for options in [
            Options::default(),
            Options::winner(Winner::Last),
            Options::winner(Winner::First),
        ] {
            compared += assert_spans_are_span_ats(&text, 0, options);
        }
    }
    for text in [
        "  {  \"a\" : [ 1 , { \"b\" : null } , \"x\" ] , /* c */ \"d\" : { } }  ",
        "// lead\n[ [ ] , [ 1 ] , { } ] // tail",
        "{\"\\u0061\":1,\"a\":2,\"a\\\"b\":{\"c\":[true,false]}}",
        "\"just a string\"",
        "  7  ",
        "[[[[[[[[[[1]]]]]]]]]]",
    ] {
        compared += assert_spans_are_span_ats(text, 0, Options::default());
        compared += assert_spans_are_span_ats(text, 77, Options::default());
    }
    assert!(compared > 300, "{compared}");
}

#[test]
fn a_node_of_the_tree_is_the_text_of_the_value_it_stands_for() {
    // For each node, the text at its span reads as the value, and each member's key token reads as
    // its key: the tree mirrors the value.
    let mut checked = 0;
    for path in fixture_files() {
        let Some(text) = read_text(&path) else {
            continue;
        };
        let Ok(parsed) = parse(&text, Options::winner(Winner::Last)) else {
            continue;
        };
        let mut open = vec![(parsed.spans.root(), &parsed.value)];
        while let Some((node, value)) = open.pop() {
            let span = node.span();
            let slice = &text[span.offset as usize..(span.offset + span.len) as usize];
            let reread = parse(slice, Options::winner(Winner::Last))
                .unwrap_or_else(|fault| panic!("{}: {slice:?}: {fault}", path.display()));
            assert!(
                identical(&reread.value, value),
                "{}: {slice:?}",
                path.display()
            );
            checked += 1;
            match node.kind() {
                SpanKind::Object => {
                    let object = value.as_object().expect("an object node of an object");
                    assert_eq!(node.members().count(), object.len());
                    for (member, (key, value)) in node.members().zip(object.iter()) {
                        assert_eq!(member.key, key.as_str());
                        let at = member.key_span;
                        let token = &text[at.offset as usize..(at.offset + at.len) as usize];
                        assert!(token.starts_with('"') && token.ends_with('"'), "{token}");
                        open.push((member.value, value));
                    }
                }
                SpanKind::Array => {
                    let array = value.as_array().expect("an array node of an array");
                    assert_eq!(node.elements().count(), array.len());
                    open.extend(node.elements().zip(array.iter()));
                }
                SpanKind::Scalar => assert_eq!(node.len(), 0),
            }
        }
    }
    assert!(checked > 5_000, "{checked}");
}

#[test]
fn the_tree_holds_the_copies_the_rule_for_repeated_keys_says() {
    let text = r#"{"a":1,"b":2,"a":3,"c":{"d":1,"d":2}}"#;
    let key_spans = |winner| {
        let parsed = parse(text, Options::winner(winner)).unwrap();
        let members: Vec<_> = parsed
            .spans
            .root()
            .members()
            .map(|member| (member.key.into_owned(), member.value.span().offset))
            .collect();
        members
    };
    let at = |needle: &str| u32::try_from(text.find(needle).unwrap()).unwrap();
    assert_eq!(
        key_spans(Winner::Last),
        [
            ("b".into(), at("2,")),
            ("a".into(), at("3")),
            ("c".into(), at("{\"d"))
        ]
    );
    assert_eq!(
        key_spans(Winner::First),
        [
            ("a".into(), at("1")),
            ("b".into(), at("2,")),
            ("c".into(), at("{\"d"))
        ]
    );
    let all = key_spans(Winner::Keep);
    assert_eq!(
        all.iter().map(|(key, _)| key.as_str()).collect::<String>(),
        "abac"
    );
    // The last copy is what a member lookup finds, as `span_at` finds it.
    let kept = parse(text, Options::winner(Winner::Keep)).unwrap();
    assert_eq!(
        kept.spans.root().member("a").unwrap().span().offset,
        at("3")
    );
    assert_eq!(
        kept.spans
            .root()
            .member("c")
            .unwrap()
            .member("d")
            .unwrap()
            .span()
            .offset,
        at("2}")
    );
}

// ---------------------------------------------------------------------------------------------
// The numbers a reader is told of
// ---------------------------------------------------------------------------------------------

#[test]
fn the_numbers_the_value_does_not_hold_as_written_are_noted_with_their_token() {
    let text = r#"{"big":18446744073709551616,"inf":1e999,"neg":-1e999,"wide":[-123456789012345678901234567890,
        -0, -0.0, 0, 0.0, 1.5, 12345678901234567890], "huge":1e400, "copy":{"x":1e999}, "copy":{"y":-0}}"#;
    let parsed = parse(text, Options::winner(Winner::Last)).unwrap();
    let noted: Vec<(String, NumberKind, String)> = parsed
        .notes
        .iter()
        .map(|note| {
            let token =
                &text[note.span.offset as usize..(note.span.offset + note.span.len) as usize];
            (note.path.to_string(), note.kind, token.to_owned())
        })
        .collect();
    let wide =
        |path: &str, token: &str| (path.to_owned(), NumberKind::WideInteger, token.to_owned());
    let beyond =
        |path: &str, token: &str| (path.to_owned(), NumberKind::BeyondDouble, token.to_owned());
    let zero =
        |path: &str, token: &str| (path.to_owned(), NumberKind::NegativeZero, token.to_owned());
    assert_eq!(
        noted,
        [
            wide("big", "18446744073709551616"),
            beyond("inf", "1e999"),
            beyond("neg", "-1e999"),
            wide("wide/0", "-123456789012345678901234567890"),
            zero("wide/1", "-0"),
            zero("wide/2", "-0.0"),
            beyond("huge", "1e400"),
            zero("copy/y", "-0"),
        ]
    );
    // The value holds what the door holds: infinities, the double of a wide integer, `-0` as
    // the integer 0 and `-0.0` as a float.
    let wide_value = parsed
        .value
        .as_object()
        .unwrap()
        .get("wide")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(
        wide_value.get(1).unwrap().as_number().unwrap().to_i64(),
        Some(0)
    );
    assert!(
        wide_value
            .get(2)
            .unwrap()
            .as_number()
            .unwrap()
            .to_f64_lossy()
            .is_sign_negative()
    );
    let inf = parsed
        .value
        .as_object()
        .unwrap()
        .get("inf")
        .unwrap()
        .as_number()
        .unwrap();
    assert!(inf.to_f64_lossy().is_infinite());
    assert_eq!(
        parsed.spans.root().member("inf").unwrap().number(),
        Some(NumberKind::BeyondDouble)
    );
    // A text with none of them has no note.
    assert!(
        parse(r#"{"a":[1,2.5,-3,1e300]}"#, Options::default())
            .unwrap()
            .notes
            .is_empty()
    );
}

// ---------------------------------------------------------------------------------------------
// Faults
// ---------------------------------------------------------------------------------------------

#[test]
fn a_fault_is_at_file_offsets_when_the_text_is_a_slice_of_a_file() {
    let slice = JsonSlice {
        text: "[1, ?]",
        offset: 100,
    };
    let fault = parse_slice(slice, Options::default()).unwrap_err();
    assert_eq!((fault.span.offset, fault.span.len), (104, 1));
    let plain = parse(slice.text, Options::default()).unwrap_err();
    assert_eq!((plain.span.offset, plain.span.len), (4, 1));
    assert!(!fault.is_end_of_input());
    assert!(
        parse("[1,", Options::default())
            .unwrap_err()
            .is_end_of_input()
    );
}

#[test]
fn the_faults_of_the_door_are_the_faults_of_the_parser() {
    use bedrock_addon_core::json::parse::FaultKind;
    let kind = |text: &str| parse(text, Options::default()).unwrap_err().kind;
    assert!(matches!(kind("[1 2]"), FaultKind::UnexpectedToken { .. }));
    assert!(matches!(
        kind("?"),
        FaultKind::UnexpectedChar { ch: '?', .. }
    ));
    assert!(matches!(kind("[1"), FaultKind::UnexpectedEof { .. }));
    assert!(matches!(kind("\"\\ud800\""), FaultKind::InvalidUtf8));
    assert!(matches!(
        kind("\"\\q\""),
        FaultKind::UnexpectedChar { ch: 'q', .. }
    ));
}

// ---------------------------------------------------------------------------------------------
// Cost
// ---------------------------------------------------------------------------------------------

/// A document 120 levels deep with `n` numbers in the innermost array and, at each level, a
/// sibling member: the shape on which finding a path costs the text for each level.
fn deep_document(n: usize) -> String {
    let levels = 120;
    let numbers: String = (0..n).map(|i| i.to_string()).collect::<Vec<_>>().join(",");
    let mut text = format!("[{numbers}]");
    for level in 0..levels {
        text = format!("{{\"level{level}\":{text},\"sibling\":{level}}}");
    }
    text
}

fn deepest_path() -> JsonPath {
    let mut path = JsonPath::new();
    for level in (0..120).rev() {
        path.push(&format!("level{level}"));
    }
    path.push_index(0);
    path
}

#[test]
fn parsing_a_deeply_nested_document_costs_what_its_length_costs() {
    use bedrock_addon_core::test_cost;
    test_cost::assert_linear_at(
        "parse of a document 120 levels deep",
        20_000,
        deep_document,
        |text| {
            let parsed = parse(text, Options::default()).unwrap();
            assert!(parsed.value.is_object());
            parsed.spans.at(&deepest_path()).unwrap()
        },
    );
}

#[test]
fn finding_the_span_of_the_deepest_member_costs_the_levels_and_not_the_text() {
    use bedrock_addon_core::test_cost;
    // The span of one member 120 levels down: `span_at` reads the text for each level; the tree
    // is walked for each level.
    test_cost::assert_each_linear_timed_at(["the tree"], 2_000, |size| {
        let text = deep_document(size);
        let parsed = parse(&text, Options::default()).unwrap();
        let path = deepest_path();
        let (found, took) = test_cost::timed(|| {
            let mut found = None;
            for _ in 0..200 {
                found = parsed.spans.at(&path);
            }
            found
        });
        assert!(found.is_some());
        [took]
    });
    // And the tree agrees with `span_at` on that path.
    let text = deep_document(300);
    let parsed = parse(&text, Options::default()).unwrap();
    assert_eq!(
        parsed.spans.at(&deepest_path()),
        span_at(JsonSlice::new(&text), &deepest_path())
    );
}

/// One pass: the parser makes no call of itself, so a text of the greatest depth is read on a
/// thread with a small stack.
#[test]
fn a_text_of_the_greatest_depth_is_read_on_a_small_stack() {
    let text = deep_document(10);
    std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(move || {
            let parsed = parse(&text, Options::winner(Winner::Last)).unwrap();
            assert_eq!(parsed.notes.len(), 0);
            parsed.spans.at(&deepest_path()).unwrap();
        })
        .unwrap()
        .join()
        .unwrap();
}

/// The release timings of the notes: set `BEDROCK_PARSE_TIMINGS` and run with `--release
/// --no-capture`. The door and the parser on the same 2 MB documents.
#[test]
fn timings_against_the_door() {
    if std::env::var("BEDROCK_PARSE_TIMINGS").is_err() {
        eprintln!(
            "SKIPPED (not a pass): the parser's timings against the door: BEDROCK_PARSE_TIMINGS is not set"
        );
        return;
    }
    let documents = [
        ("wide: 2 MB of small objects", wide_document()),
        ("deep: 2 MB at 120 levels", deep_document(300_000)),
    ];
    for (what, text) in &documents {
        let best = |run: &dyn Fn()| {
            (0..5)
                .map(|_| {
                    let started = Instant::now();
                    run();
                    started.elapsed()
                })
                .min()
                .unwrap_or_default()
        };
        let door = best(&|| {
            std::hint::black_box(from_str_borrowed_jsonc::<Value>(text).unwrap());
        });
        let ours = best(&|| {
            std::hint::black_box(parse(text, Options::default()).unwrap());
        });
        let last = best(&|| {
            std::hint::black_box(parse(text, Options::winner(Winner::Last)).unwrap());
        });
        let path = deepest_path();
        let span_at_time = best(&|| {
            std::hint::black_box(span_at(JsonSlice::new(text), &path));
        });
        let parsed = parse(text, Options::default()).unwrap();
        let tree = best(&|| {
            std::hint::black_box(parsed.spans.at(&path));
        });
        eprintln!(
            "timing: {what}: {} bytes: door {door:?}, parser {ours:?} (last wins {last:?}); the span of the deepest member: span_at {span_at_time:?}, tree {tree:?}",
            text.len()
        );
    }
}

fn wide_document() -> String {
    let member = |i: usize| {
        format!(
            "{{\"identifier\":\"minecraft:thing_{i}\",\"weight\":{i},\"ratio\":{}.{},\"tags\":[\"a\",\"b\",{i}],\"on\":true,\"none\":null,\"nested\":{{\"x\":{i},\"y\":\"text with \\n escape\"}}}}",
            i % 97,
            i % 13
        )
    };
    let mut items = Vec::new();
    let mut length = 0;
    let mut i = 0;
    while length < 2_000_000 {
        let item = member(i);
        length += item.len() + 1;
        items.push(item);
        i += 1;
    }
    format!("[{}]", items.join(",\n"))
}

#[allow(dead_code, reason = "the options a test names by their shape")]
fn _shapes(_: Duplicates<'_>, _: Span, _: JsonPathSegment<'_>) {}

/// The slices a raw member read of the old door gives, as text and file offset.
fn old_members(text: &str) -> Option<Vec<(String, (String, usize))>> {
    let slice = JsonSlice::new(text);
    let raw = bedrock_addon_core::json::parse_at::<
        indexmap::IndexMap<String, facet_json::RawJson<'_>>,
    >(slice)
    .ok()?;
    Some(
        raw.iter()
            .filter_map(|(key, raw)| {
                let nested = slice.nested(raw)?;
                Some((key.clone(), (nested.text.to_owned(), nested.offset)))
            })
            .collect(),
    )
}

#[test]
fn members_of_agrees_with_the_raw_members_of_the_door() {
    for text in [
        r#"{"a": 1, "b": [1, 2], "c": {"d": null}}"#,
        r#"{ "a" : 1 , /* c */ "b" : "x" // end
        }"#,
        r#"{"a": 1, "b": 2, "a": 3}"#,
        r#"{"a": {"x": 1}, "b": 2, "a": {"y": 2}}"#,
        r#"{"kA": 1, "kA": 2}"#,
        r#"{"a": 1e999, "b": 18446744073709551616,}"#,
        "{}",
        "[1, 2]",
        r#""x""#,
        r#"{"a": 1"#,
        "",
    ] {
        let ours =
            bedrock_addon_core::json::parse::members_of(JsonSlice::new(text)).map(|members| {
                members
                    .into_iter()
                    .map(|(key, slice)| (key, (slice.text.to_owned(), slice.offset)))
                    .collect::<Vec<_>>()
            });
        assert_eq!(ours, old_members(text), "{text}");
    }
}

#[test]
fn elements_of_lists_the_values_of_an_array_with_their_offsets() {
    let text = r#"[1, "two", {"three": 3} /* c */, [4], ]"#;
    let elements = bedrock_addon_core::json::parse::elements_of(JsonSlice::new(text)).unwrap();
    let seen: Vec<_> = elements.iter().map(|e| (e.text, e.offset)).collect();
    assert_eq!(
        seen,
        [
            ("1", 1),
            (r#""two""#, 4),
            (r#"{"three": 3}"#, 11),
            ("[4]", 33)
        ]
    );
    for text in ["{}", "1", "[1,", ""] {
        assert!(
            bedrock_addon_core::json::parse::elements_of(JsonSlice::new(text)).is_none(),
            "{text}"
        );
    }
}

#[test]
fn string_literal_decodes_what_the_door_reads_as_a_string() {
    use bedrock_addon_core::json::parse::string_literal;
    for token in [r#""a""#, r#""a\nb""#, r#""é😀""#, r#""""#] {
        assert_eq!(
            string_literal(token),
            bedrock_addon_core::json::from_str::<String>(token).ok(),
            "{token}"
        );
    }
    for token in ["1", "null", r#""\q""#, r#""\ud800""#, "\"open", ""] {
        assert_eq!(string_literal(token), None, "{token}");
    }
}

#[test]
fn a_scalar_shape_of_a_value_is_the_shape_the_door_reads_the_token_as() {
    use bedrock_addon_core::json::ScalarShape;
    for token in [
        "null",
        "true",
        r#""x""#,
        "0",
        "-0",
        "7",
        "-7",
        "1.5",
        "-0.0",
        "1e3",
        "18446744073709551615",
        "18446744073709551616",
        "-9223372036854775808",
        "-9223372036854775809",
        "1e999",
    ] {
        let value = parse(token, Options::default()).unwrap().value;
        let old = bedrock_addon_core::json::from_str_jsonc::<ScalarShape>(token).ok();
        assert_eq!(ScalarShape::of(&value), old, "{token}");
    }
    for token in ["[1]", "{}"] {
        let value = parse(token, Options::default()).unwrap().value;
        assert_eq!(ScalarShape::of(&value), None, "{token}");
    }
}

// ---------------------------------------------------------------------------------------------
// Bytes that may not be UTF-8, and plain JSON
// ---------------------------------------------------------------------------------------------

/// Bytes that are no UTF-8, put in a text: a lone lead byte, a lone continuation byte, the lead
/// of a two-byte character cut short.
const BAD_BYTES: &[&[u8]] = &[b"\xff", b"\x80", b"\xc3", b"\xe9 "];

/// `text` with each of [`BAD_BYTES`] put in at a spread of places, and with them in a comment.
fn bad_bytes(text: &str, places: usize) -> Vec<Vec<u8>> {
    let bytes = text.as_bytes();
    let step = (bytes.len() / places).max(1);
    let mut out = Vec::new();
    for at in (0..=bytes.len()).step_by(step) {
        for bad in BAD_BYTES {
            let mut edited = bytes[..at].to_vec();
            edited.extend_from_slice(bad);
            edited.extend_from_slice(&bytes[at..]);
            out.push(edited);
            let mut commented = bytes[..at].to_vec();
            commented.extend_from_slice(b"/* ");
            commented.extend_from_slice(bad);
            commented.extend_from_slice(b" */");
            commented.extend_from_slice(&bytes[at..]);
            out.push(commented);
        }
    }
    out
}

#[test]
fn bytes_that_may_not_be_utf8_read_as_the_door_reads_them() {
    use bedrock_addon_core::json::from_slice_jsonc;
    use bedrock_addon_core::json::parse::parse_bytes;
    let mut seeds: Vec<String> = RULES.iter().map(|text| (*text).to_owned()).collect();
    seeds.extend([
        r#"{"a": "x", "b": [1, 2.5, null]} // after"#.to_owned(),
        "/* head */ [\"é\", {\"k\": \"v\"}] [junk".to_owned(),
        r#"{"deep": [[[{"s": "é"}]]], "n": 18446744073709551616}"#.to_owned(),
    ]);
    let (mut texts, mut differed) = (0, Vec::new());
    for seed in &seeds {
        let mut cases = bad_bytes(seed, 24);
        cases.push(seed.as_bytes().to_vec());
        for bytes in cases {
            texts += 1;
            let door = from_slice_jsonc::<Value>(&bytes).map_err(|error| Refusal {
                span: error
                    .span
                    .map(|span| (span.offset as usize, span.len as usize)),
                kind: format!("{:?}", error.kind),
                words: error.kind.to_string(),
            });
            let ours = parse_bytes(&bytes, Options::default()).map_err(refusal);
            let same = match (&door, &ours) {
                (Ok(a), Ok(b)) => identical(a, b),
                (Err(a), Err(b)) => a == b,
                _ => false,
            };
            if !same {
                differed.push(format!(
                    "{:?}\n  door: {}\n  ours: {}",
                    String::from_utf8_lossy(&bytes),
                    summary(&door),
                    summary(&ours)
                ));
            }
        }
    }
    eprintln!("bytes: {texts} texts, {} differ", differed.len());
    assert!(
        differed.is_empty(),
        "parse_bytes differs from from_slice_jsonc on {} of {texts} byte texts; the first:\n{}",
        differed.len(),
        differed[..differed.len().min(8)].join("\n")
    );
}

#[test]
fn parse_options_without_comments_read_as_the_plain_door_reads() {
    use bedrock_addon_core::json::from_str;
    let mut seeds: Vec<String> = RULES.iter().map(|text| (*text).to_owned()).collect();
    seeds.push("// x\n1".to_owned());
    seeds.push(r#"{"a": "// not a comment", "b": /* c */ 1}"#.to_owned());
    let mut texts = Vec::new();
    for seed in &seeds {
        texts.push(seed.clone());
        texts.extend(damaged(seed, 12, JUNK));
    }
    let mut differed = Vec::new();
    for text in &texts {
        let door = from_str::<Value>(text).map_err(|error| Refusal {
            span: error
                .span
                .map(|span| (span.offset as usize, span.len as usize)),
            kind: format!("{:?}", error.kind),
            words: error.kind.to_string(),
        });
        let ours = ours(text, Options::default().without_comments());
        if let Some(difference) = difference(text, &door, &ours) {
            differed.push(difference);
        }
    }
    eprintln!("plain: {} texts, {} differ", texts.len(), differed.len());
    assert!(
        differed.is_empty(),
        "the parser without comments differs from the plain door on {} of {} texts; the first:\n{}",
        differed.len(),
        texts.len(),
        differed[..differed.len().min(8)].join("\n")
    );
}
