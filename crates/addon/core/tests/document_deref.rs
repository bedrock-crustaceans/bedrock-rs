//! The `Deref` name-collision rule: `Document<T>` dereferences to `T`, so a
//! method of `Document` hides one of `T` with the same name. No root type may name a field or a
//! method `value`, `into_value`, `origin`, `version`, `diagnostics`, `map`, `unknown`,
//! `unknown_mut`, `write_json`, `to_json` or `write_json_reporting` (the document's writers,
//! which write its unknown members, stand before the `Content` methods of those names).
//!
//! A source-text scan over `crates/addon/addon/src/**` for those names inside the root types of
//! `tests/document/root_types.txt` (their `struct` bodies and every `impl` block whose self type
//! is the root type), so every family's root types are covered as soon as they are listed.
//!
//! Methods a macro generates (`version_chain!` emits `history::Any`'s, `#[derive(Components)]`
//! its conversions) are not in the source text, so this scan cannot see them; the generated methods
//! are checked separately. The accessor of `history::Any` is `parsed_version` for that reason.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs;
use std::path::{Path, PathBuf};

/// `Document<T>`'s own methods.
const DOCUMENT_METHODS: [&str; 11] = [
    "value",
    "into_value",
    "origin",
    "version",
    "diagnostics",
    "map",
    "unknown",
    "unknown_mut",
    "write_json",
    "to_json",
    "write_json_reporting",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The root type names, one per line, `#` comments.
fn root_types() -> Vec<String> {
    let text = fs::read_to_string(manifest_dir().join("tests/document/root_types.txt"))
        .expect("tests/document/root_types.txt");
    text.lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// `source` without `//` comments (doc comments included) and string literal contents, so
/// prose and messages never count.
fn strip(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for line in source.lines() {
        let mut in_string = false;
        let mut escaped = false;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                    out.push('"');
                }
                continue;
            }
            if c == '/' && chars.peek() == Some(&'/') {
                break;
            }
            if c == '"' {
                in_string = true;
            }
            out.push(c);
        }
        out.push('\n');
    }
    out
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The text from `open` (a `{`) to its matching `}`, exclusive.
fn braced(text: &str, open: usize) -> &str {
    let mut depth = 0usize;
    for (offset, c) in text[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &text[open + 1..open + offset];
                }
            }
            _ => {}
        }
    }
    &text[open + 1..]
}

/// The last path segment of a type, without generics or references: `crate::a::Block<'a>` →
/// `Block`.
fn self_type(text: &str) -> &str {
    let text = text
        .trim()
        .trim_start_matches('&')
        .trim_start_matches("mut ")
        .trim();
    let text = text.split('<').next().unwrap_or(text);
    let text = text.split_whitespace().next().unwrap_or("");
    text.rsplit("::").next().unwrap_or(text)
}

/// Strips one leading `<…>` (with nesting) from `text`.
fn skip_generics(text: &str) -> &str {
    let text = text.trim_start();
    if !text.starts_with('<') {
        return text;
    }
    let mut depth = 0usize;
    for (index, c) in text.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return &text[index + 1..];
                }
            }
            _ => {}
        }
    }
    ""
}

/// The bodies of `type_name`'s struct and of every `impl` whose self type is `type_name`.
fn bodies<'a>(source: &'a str, type_name: &str) -> Vec<(&'static str, &'a str)> {
    let mut found = Vec::new();
    let mut search = 0;
    while let Some(at) = source[search..].find(type_name) {
        let start = search + at;
        search = start + type_name.len();
        let before = source[..start].chars().next_back();
        let after = source[search..].chars().next();
        if before.is_some_and(is_ident) || after.is_some_and(is_ident) {
            continue;
        }
        let rest = &source[search..];
        if source[..start].trim_end().ends_with("struct")
            && let Some(open) = rest.find(['{', ';', '('])
            && rest.as_bytes()[open] == b'{'
        {
            found.push(("struct", braced(source, search + open)));
        }
    }
    let mut search = 0;
    while let Some(at) = source[search..].find("impl") {
        let start = search + at;
        search = start + 4;
        let before = source[..start].chars().next_back();
        let after = source[search..].chars().next();
        if before.is_some_and(is_ident) || after.is_some_and(is_ident) {
            continue;
        }
        let Some(open) = source[search..].find('{') else {
            break;
        };
        let header = skip_generics(&source[search..search + open]);
        let header = header.split(" where").next().unwrap_or(header);
        let target = match header.rfind(" for ") {
            Some(index) => &header[index + 5..],
            None => header,
        };
        if self_type(target) == type_name {
            found.push(("impl", braced(source, search + open)));
        }
    }
    found
}

/// Members of `type_name` in `source` named like a `Document` method: `fn <name>` in its impl
/// blocks, `pub <name>:` in its struct.
fn collisions(source: &str, type_name: &str) -> Vec<String> {
    let source = strip(source);
    let mut out = Vec::new();
    for (kind, body) in bodies(&source, type_name) {
        for name in DOCUMENT_METHODS {
            let needle = match kind {
                "impl" => format!("fn {name}"),
                _ => format!("pub {name}"),
            };
            let mut search = 0;
            while let Some(at) = body[search..].find(&needle) {
                let start = search + at;
                search = start + needle.len();
                let before = body[..start].chars().next_back();
                let rest = body[search..].trim_start();
                let next = rest.chars().next();
                let member = match kind {
                    "impl" => matches!(next, Some('(' | '<')),
                    _ => next == Some(':') && !rest.starts_with("::"),
                };
                if member && !before.is_some_and(is_ident) {
                    out.push(format!("{type_name}::{name}"));
                }
            }
        }
    }
    out
}

#[test]
fn the_scanner_finds_fields_and_methods_of_the_type_only() {
    let source = r#"
        /// Docs mentioning fn value(&self) do not count.
        pub struct Block {
            pub value: u8,
            pub description: Description,
        }
        impl Block {
            pub fn origin(&self) -> u8 { "fn map(" ; 0 }
            fn helper() {}
        }
        impl<'a> fmt::Display for crate::block::Block {
            fn fmt(&self) {}
        }
        impl Other {
            pub fn version(&self) {}
        }
        impl<T> HasVisuals for Block where T: Clone {
            fn diagnostics(&self) {}
        }
        pub struct BlockState { pub map: u8 }
        pub struct Unit;
    "#;
    let mut found = collisions(source, "Block");
    found.sort();
    assert_eq!(
        found,
        ["Block::diagnostics", "Block::origin", "Block::value"]
    );
    assert!(collisions(source, "Unit").is_empty());
    assert_eq!(collisions(source, "BlockState"), ["BlockState::map"]);
}

#[test]
fn no_root_type_names_a_document_method() {
    let types = root_types();
    assert!(
        types.iter().any(|t| t == "Block"),
        "the list starts with 17's root types"
    );
    let mut files = Vec::new();
    rust_files(&manifest_dir().join("../addon/src"), &mut files);
    assert!(!files.is_empty(), "crates/addon/addon/src exists");
    let mut found = Vec::new();
    for file in &files {
        let source = fs::read_to_string(file).expect("readable source");
        for type_name in &types {
            for collision in collisions(&source, type_name) {
                found.push(format!("{}: {collision}", file.display()));
            }
        }
    }
    assert!(
        found.is_empty(),
        "root types hide behind `Document`'s methods: {found:#?}"
    );
}
