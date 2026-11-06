//! The door rule: `bedrock_addon_core::json::read` is the only module of the crate
//! family that calls a facet-json parser, so no text reaches facet without the scan that
//! refuses what facet cannot read safely; and `json::write` the only one that calls a
//! facet-json writer, which writes a value by a call of itself for each level and
//! has no limit either. This test reads the sources and fails for any other use of
//! `facet_json` than the names below.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::path::{Path, PathBuf};

/// The source roots the rule holds for, as paths below the workspace's `crates/`.
const ROOTS: &[&str] = &[
    "addon/src",
    "core/src",
    "std/src",
    "derive/src",
    "manifest/src",
    "storage/src",
];

/// The module that may call the parsers.
const DOOR: &str = "core/src/json/read.rs";

/// The module that may call the writers (`to_facet_json`), and the one whose tests measure
/// the stack facet takes when it is called without the door.
const OTHER_DOORS: &[&str] = &["core/src/json/write.rs", "core/src/json/stack.rs"];

/// What any module may name of facet-json: the raw slice type and the error type. Neither
/// reads or writes text.
const ALLOWED: &[&str] = &["RawJson", "DeserializeError"];

/// facet-json's owned reads (`from_str`, `from_str_jsonc`, rc.7) lend a `Cow<'_, str>` member
/// the text they are given: a `Cow<'static, str>`, or the `Cow<'a, str>` of a `Foo<'a>` read
/// as `Foo<'static>`, would outlive what it points at. No type of the crate family that
/// derives `Facet` has such a member, whatever its lifetime; this keeps it so. A
/// file's `#[cfg(test)] mod` is not looked at: the door's own tests read such a type through
/// the borrowed entry point, which is what it is for.
#[test]
fn no_type_holds_a_static_cow_a_read_could_lend_the_text_to() {
    let crates = crates_dir();
    let mut files = Vec::new();
    for root in ROOTS {
        rust_files(&crates.join(root), &mut files);
    }
    // Every family is a crate of its own under `families/`.
    if let Ok(families) = std::fs::read_dir(crates.join("families")) {
        for family in families.map(|entry| entry.unwrap().path()) {
            if family.join("src").is_dir() {
                rust_files(&family.join("src"), &mut files);
            }
        }
    }
    assert!(files.len() > 300, "the scan found {} files", files.len());
    let (mut found, mut derives) = (Vec::new(), 0);
    for file in &files {
        let code = code_of(&std::fs::read_to_string(file).unwrap());
        let (types, holders) = facet_types_with_a_cow_str(&code);
        derives += types;
        for holder in holders {
            found.push(format!("{}: {holder}", file.display()));
        }
    }
    assert!(
        derives > 300,
        "the scan found {derives} types deriving Facet: it sees none"
    );
    assert!(
        found.is_empty(),
        "a `Cow<'_, str>` in a type deriving Facet (hold a `String`, or a `&str` read through \
         the borrowed door):\n{}",
        found.join("\n")
    );
}

/// The types of `code` (comments removed) that derive `Facet`, outside its test module: how
/// many there are, and the start of each one whose body names a `Cow<'_, str>`.
fn facet_types_with_a_cow_str(code: &str) -> (usize, Vec<String>) {
    let code: String = code.split_whitespace().collect();
    let code = code
        .split_once("#[cfg(test)]mod")
        .map_or(code.as_str(), |(before, _)| before);
    let (mut types, mut holders) = (0, Vec::new());
    for (at, _) in code.match_indices("derive(") {
        let rest = &code[at..];
        let Some(close) = rest.find(')') else {
            continue;
        };
        let derived = rest["derive(".len()..close]
            .split(',')
            .any(|name| name == "Facet" || name.ends_with("::Facet"));
        if !derived {
            continue;
        }
        types += 1;
        let item = item_after(&rest[close..]);
        if names_a_cow_str(item) {
            holders.push(head(item, 60));
        }
    }
    (types, holders)
}

/// The item `rest` starts with, after its attributes: to its `;`, or to the brace that closes
/// its body.
fn item_after(rest: &str) -> &str {
    let mut depth = 0_usize;
    // The brackets of an attribute are skipped, whatever they hold.
    let mut attribute = 0_usize;
    for (at, byte) in rest.bytes().enumerate() {
        match byte {
            b'[' => attribute += 1,
            b']' => attribute = attribute.saturating_sub(1),
            _ if attribute > 0 => {}
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return &rest[..=at];
                }
            }
            b';' if depth == 0 => return &rest[..=at],
            _ => {}
        }
    }
    rest
}

/// Whether `item` (no white space) names a `Cow<'lifetime, str>`.
fn names_a_cow_str(item: &str) -> bool {
    item.match_indices("Cow<'").any(|(at, start)| {
        let lifetime = &item[at + start.len()..];
        let end = lifetime
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(lifetime.len());
        lifetime[end..].starts_with(",str>")
    })
}

#[test]
fn the_cow_scan_sees_each_form_of_a_member() {
    let holders = |code: &str| facet_types_with_a_cow_str(&code_of(code));
    for held in [
        "#[derive(Facet)]\nstruct A { x: Cow<'static, str> }",
        "#[derive(Debug, Facet)]\npub struct A<'a> {\n    #[facet(default)]\n    x: Option<Cow<'a, str>>,\n}",
        "#[derive(facet::Facet, Clone)]\nstruct A<'a>(std::borrow::Cow<'a, str>);",
        "#[derive(Facet)]\n#[facet(rename_all = \"{x}\")]\n#[repr(u8)]\nenum A<'v> { B { y: u8 }, C(Cow<'v, str>) }",
        "#[derive(Facet)]\nstruct A<'a> { m: IndexMap<Cow<'a,\n        str>, u8>, n: [u8; 2] }",
        "#[cfg_attr(feature = \"x\", derive(Facet))]\nstruct A<'a> { x: Cow<'a, str> }",
    ] {
        let (types, found) = holders(held);
        assert_eq!((types, found.len()), (1, 1), "{held}");
    }
    for free in [
        // No `Facet`: no read lends it anything.
        "#[derive(Debug)]\nstruct A<'a> { x: Cow<'a, str> }\n#[derive(Facet)]\nstruct B { x: String }",
        // The member is of the next type, which derives nothing.
        "#[derive(Facet)]\nstruct B { x: [u8; 2] }\nstruct A<'a> { x: Cow<'a, str> }",
        "#[derive(Facet)]\nstruct B;\nenum A<'a> { X(Cow<'a, str>) }",
        // A `Cow` of something else, a function's return, a comment.
        "#[derive(Facet)]\nstruct B<'a> { x: Cow<'a, [u8]>, y: &'a str }\nfn f(t: &str) -> Cow<'_, str> { t.into() }",
        "#[derive(Facet)]\nstruct B {\n    /// once a `Cow<'static, str>`\n    x: String,\n}",
        // The test module of a file is its own.
        "#[derive(Facet)]\nstruct B { x: u8 }\n#[cfg(test)]\nmod tests {\n    #[derive(Facet)]\n    struct T<'a> { x: Cow<'a, str> }\n}",
    ] {
        let (types, found) = holders(free);
        assert_eq!((types, found), (1, Vec::new()), "{free}");
    }
}

/// Every `.rs` file below `dir`.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// `text` without its comments (`//` to the end of the line, doc comments included, and
/// `/* */`, nested), so prose may name the parsers. A comment marker inside a string, a raw
/// string or a character literal is text of the literal: literals are kept whole and the code
/// after them is seen.
fn code_of(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while let Some(&c) = chars.get(at) {
        let next = chars.get(at + 1).copied();
        match c {
            '/' if next == Some('/') => {
                while chars.get(at).is_some_and(|&c| c != '\n') {
                    at += 1;
                }
            }
            '/' if next == Some('*') => {
                let mut depth = 0_usize;
                while at < chars.len() {
                    match (chars[at], chars.get(at + 1)) {
                        ('/', Some('*')) => {
                            depth += 1;
                            at += 2;
                        }
                        ('*', Some('/')) => {
                            depth -= 1;
                            at += 2;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => at += 1,
                    }
                }
                out.push(' ');
            }
            '"' => at = copy_string(&chars, at, &mut out),
            // A raw string: `r`, hashes, a quote; not the tail of a longer name.
            'r' if matches!(next, Some('"' | '#'))
                && !at.checked_sub(1).is_some_and(|before| {
                    chars[before].is_alphanumeric() || chars[before] == '_'
                }) =>
            {
                let hashes = chars[at + 1..].iter().take_while(|&&c| c == '#').count();
                if chars.get(at + 1 + hashes) == Some(&'"') {
                    at = copy_raw_string(&chars, at, hashes, &mut out);
                } else {
                    out.push(c);
                    at += 1;
                }
            }
            // A character literal (`'x'`, `'\n'`, `'"'`); a lifetime has no closing quote.
            '\'' => {
                let end = match next {
                    Some('\\') => chars[at + 2..]
                        .iter()
                        .skip(1)
                        .position(|&c| c == '\'')
                        .map(|n| at + 3 + n),
                    Some(_) if chars.get(at + 2) == Some(&'\'') => Some(at + 2),
                    _ => None,
                };
                let end = end.map_or(at, |end| end.min(chars.len() - 1));
                out.extend(&chars[at..=end]);
                at = end + 1;
            }
            _ => {
                out.push(c);
                at += 1;
            }
        }
    }
    out
}

/// Copies the string literal whose opening quote is at `at`; the index after it.
fn copy_string(chars: &[char], mut at: usize, out: &mut String) -> usize {
    out.push('"');
    at += 1;
    while let Some(&c) = chars.get(at) {
        out.push(c);
        at += 1;
        match c {
            '\\' => {
                if let Some(&escaped) = chars.get(at) {
                    out.push(escaped);
                    at += 1;
                }
            }
            '"' => break,
            _ => {}
        }
    }
    at
}

/// Copies the raw string literal whose `r` is at `at` and which has `hashes` hashes; the
/// index after it.
fn copy_raw_string(chars: &[char], at: usize, hashes: usize, out: &mut String) -> usize {
    let body = at + 2 + hashes;
    let mut end = body;
    while let Some(&c) = chars.get(end) {
        end += 1;
        if c == '"' && chars[end..].iter().take_while(|&&c| c == '#').count() >= hashes {
            end += hashes;
            break;
        }
    }
    let end = end.min(chars.len());
    out.extend(&chars[at..end]);
    end
}

/// The uses of `facet_json` in `code` that are not one of [`ALLOWED`], each as the text from
/// the crate name to the end of what it names.
fn offenders(code: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (at, _) in code.match_indices("facet_json") {
        let before = code[..at].chars().next_back();
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            continue; // part of a longer name
        }
        let rest = &code[at + "facet_json".len()..];
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix("::") else {
            // `use facet_json;`, `facet_json as j`, `extern crate`: a handle the scan cannot
            // follow.
            out.push(format!("facet_json{}", head(rest, 24)));
            continue;
        };
        let rest = rest.trim_start();
        let names: Vec<&str> = match rest.strip_prefix('{') {
            Some(list) => list
                .split_once('}')
                .map_or(list, |(list, _)| list)
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .collect(),
            None => vec![
                rest.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '*'))
                    .next()
                    .unwrap_or(""),
            ],
        };
        for name in names {
            if !ALLOWED.contains(&name) {
                out.push(format!("facet_json::{name}"));
            }
        }
    }
    out
}

/// The first `n` characters of `text`'s first line.
fn head(text: &str, n: usize) -> String {
    text.lines().next().unwrap_or("").chars().take(n).collect()
}

fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_owned()
}

#[test]
fn only_the_door_calls_a_facet_json_parser() {
    let crates = crates_dir();
    let mut files = Vec::new();
    for root in ROOTS {
        rust_files(&crates.join(root), &mut files);
    }
    files.sort();
    assert!(files.len() > 300, "the scan found {} files", files.len());
    let mut found = Vec::new();
    let mut door_calls = 0;
    for file in &files {
        let relative = file.strip_prefix(&crates).unwrap().to_str().unwrap();
        let relative = relative.replace('\\', "/");
        let code = code_of(&std::fs::read_to_string(file).unwrap());
        let offenders = offenders(&code);
        if relative == DOOR {
            door_calls = offenders.len();
            continue;
        }
        if OTHER_DOORS.contains(&relative.as_str()) {
            continue;
        }
        for offender in offenders {
            found.push(format!("{relative}: {offender}"));
        }
    }
    assert!(
        door_calls >= 3,
        "the door calls {door_calls} parsers: the scan sees none"
    );
    assert!(
        found.is_empty(),
        "{} uses of facet-json outside `json::read` and `json::write` (call \
         `json::from_str_jsonc` and its siblings, which scan the text first, or \
         `json::to_facet_json`, which bounds the depth):\n{}",
        found.len(),
        found.join("\n")
    );
}

/// The crates must name facet-json by its own name: a renamed dependency would pass the scan.
#[test]
fn no_crate_renames_facet_json() {
    for name in ["addon", "core", "std", "derive", "manifest", "storage"] {
        let manifest = std::fs::read_to_string(crates_dir().join(name).join("Cargo.toml")).unwrap();
        for line in manifest.lines() {
            assert!(
                !(line.contains("package") && line.contains("facet-json")),
                "{name}/Cargo.toml renames facet-json: `{line}`"
            );
        }
    }
}

#[test]
fn the_scan_sees_each_form_of_a_use() {
    let hits = |code: &str| offenders(code);
    assert_eq!(
        hits("facet_json::from_str::<T>(x)"),
        ["facet_json::from_str"]
    );
    assert_eq!(
        hits("use facet_json::{RawJson, from_str_jsonc};"),
        ["facet_json::from_str_jsonc"]
    );
    assert_eq!(
        hits("facet_json\n    ::from_slice(b)"),
        ["facet_json::from_slice"]
    );
    assert_eq!(hits("use facet_json::*;"), ["facet_json::*"]);
    assert_eq!(hits("use facet_json as j;").len(), 1);
    assert_eq!(
        hits("facet_json::JsonParser::<true>::new(b)"),
        ["facet_json::JsonParser"]
    );
    assert!(hits("use facet_json::{DeserializeError, RawJson};").is_empty());
    assert_eq!(hits("facet_json::to_string(&v)"), ["facet_json::to_string"]);
    assert!(hits("my_facet_json::from_str(x)").is_empty());
    assert!(hits(&code_of("/// see `facet_json::from_str`\nfn f() {}")).is_empty());
}

/// A `//` inside a string, a raw string or a character is no comment: the code after it on
/// the line is seen. A `/* */` comment is one.
#[test]
fn the_scan_sees_code_after_a_comment_marker_inside_a_literal() {
    let hits = |code: &str| offenders(&code_of(code));
    for seen in [
        "let url = \"http://x\"; facet_json::from_str::<T>(x)",
        "let a = \"\\\"//\"; facet_json::from_str::<T>(x)",
        "let raw = r#\"a \"// b\"#; facet_json::from_str::<T>(x)",
        "let raw = r\"//\"; facet_json::from_str::<T>(x)",
        "let c = '/'; let d = '/'; facet_json::from_str::<T>(x)",
        "let q = '\"'; let s = \"//\"; facet_json::from_str::<T>(x)",
        "fn f<'a>(x: &'a str) { let s = \"//\"; facet_json::from_str::<T>(x) }",
        "let s = \"a\n// not a comment\"; facet_json::from_str::<T>(x)",
        "/* a */ facet_json::from_str::<T>(x) /* b */",
    ] {
        assert_eq!(hits(seen), ["facet_json::from_str"], "{seen}");
    }
    for hidden in [
        "// facet_json::from_str(x)",
        "let s = \"//\"; // facet_json::from_str(x)",
        "/* facet_json::from_str(x)\n facet_json::from_slice(x) */ fn f() {}",
        "/* a /* nested */ facet_json::from_str(x) */",
        "let c = '\\''; // facet_json::from_str(x)",
    ] {
        assert!(hits(hidden).is_empty(), "{hidden}");
    }
}
