//! Every `VanillaText::Formatted` the jigsaw module builds has one argument per `{}` of its
//! template: the module's sources are scanned for `template: "…"` followed by
//! `args: vec![…]`, and for the templates held in constants and macro arguments (`… = "…{}…"`,
//! each given one argument by the function that uses it).

use std::path::{Path, PathBuf};

/// Every `.rs` file below `dir`.
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The string literal starting at `text` (after leading whitespace), unescaped, and the rest.
fn literal(text: &str) -> Option<(String, &str)> {
    let text = text.trim_start();
    let body = text.strip_prefix('"')?;
    let mut out = String::new();
    let mut chars = body.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => {
                if let Some((_, next)) = chars.next() {
                    out.push(next);
                }
            }
            '"' => return Some((out, &body[i + 1..])),
            _ => out.push(c),
        }
    }
    None
}

/// How many arguments a `[…]` list starting at `text` holds (top-level commas, a trailing one
/// allowed).
fn arity(text: &str) -> Option<usize> {
    let text = text.trim_start();
    let body = text.strip_prefix('[')?;
    let (mut depth, mut count, mut token) = (0usize, 0usize, false);
    let mut in_string = false;
    let mut escaped = false;
    for c in body.chars() {
        if in_string {
            match (escaped, c) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => in_string = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                token = true;
            }
            '(' | '[' | '{' => {
                depth += 1;
                token = true;
            }
            ']' if depth == 0 => return Some(count + usize::from(token)),
            ')' | '}' | ']' => depth -= 1,
            ',' if depth == 0 => {
                count += usize::from(token);
                token = false;
            }
            c if c.is_whitespace() => {}
            _ => token = true,
        }
    }
    None
}

/// `(template, argument count)` of every literal template in `source`.
fn formatted(source: &str) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    for (index, _) in source.match_indices("template: ") {
        let Some((template, rest)) = literal(&source[index + "template: ".len()..]) else {
            continue;
        };
        let Some(start) = rest.find("args: vec!") else {
            continue;
        };
        if let Some(count) = arity(&rest[start + "args: vec!".len()..]) {
            found.push((template, count));
        }
    }
    found
}

/// The templates held in constants and macro arguments (`: &str = "…"`, `failed = "…"`): each
/// is formatted with one argument (the identifier or the file's path).
fn constants(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    for marker in [": &str =", "failed = "] {
        for (index, _) in source.match_indices(marker) {
            if let Some((template, _)) = literal(&source[index + marker.len()..])
                && template.contains("{}")
            {
                found.push(template);
            }
        }
    }
    found
}

#[test]
fn every_formatted_text_has_one_argument_per_placeholder() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&root, &mut files);
    let (mut literals, mut held) = (0, 0);
    let mut wrong = Vec::new();
    for file in files {
        let source = std::fs::read_to_string(&file).unwrap();
        for (template, count) in formatted(&source) {
            literals += 1;
            if template.matches("{}").count() != count {
                wrong.push(format!("{}: {template:?} with {count}", file.display()));
            }
        }
        for template in constants(&source) {
            held += 1;
            if template.matches("{}").count() != 1 {
                wrong.push(format!("{}: {template:?} with 1", file.display()));
            }
        }
    }
    assert!(
        literals >= 7,
        "the scan finds the literal templates ({literals})"
    );
    assert!(held >= 8, "the scan finds the held templates ({held})");
    assert!(wrong.is_empty(), "{wrong:#?}");
}
