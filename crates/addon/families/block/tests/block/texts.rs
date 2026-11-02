//! Every `VanillaText::Formatted` the block module builds has one argument per `{}` of its
//! template: the module's sources are scanned for `text("…", &[…])`,
//! `text_owned("…", …)` and `template: "…", args: vec![…]`, the arity checked statically.

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

/// `(template, argument count)` of every formatted text in `source`.
fn formatted(source: &str) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    for (index, _) in source.match_indices("text(") {
        let before = source[..index].chars().last();
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            continue;
        }
        let Some((template, rest)) = literal(&source[index + "text(".len()..]) else {
            continue;
        };
        let rest = rest
            .trim_start()
            .strip_prefix(',')
            .unwrap_or(rest)
            .trim_start();
        let rest = rest.strip_prefix('&').unwrap_or(rest);
        if let Some(count) = arity(rest) {
            found.push((template, count));
        }
    }
    for (index, _) in source.match_indices("text_owned(") {
        if let Some((template, _)) = literal(&source[index + "text_owned(".len()..]) {
            found.push((template, 1));
        }
    }
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

#[test]
fn every_formatted_text_has_one_argument_per_placeholder() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&root, &mut files);
    let mut checked = 0;
    let mut wrong = Vec::new();
    for file in files {
        let source = std::fs::read_to_string(&file).unwrap();
        for (template, count) in formatted(&source) {
            checked += 1;
            if template.matches("{}").count() != count {
                wrong.push(format!("{}: {template:?} with {count}", file.display()));
            }
        }
    }
    assert!(checked >= 20, "the scan finds the templates ({checked})");
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn the_scanner_counts_arguments() {
    assert_eq!(arity(r#"[name, "a, b", f(x, y)]"#), Some(3));
    assert_eq!(arity("[]"), Some(0));
    assert_eq!(arity("[a,]"), Some(1));
    assert_eq!(
        formatted(
            r#"x(text("A {} b {}", &[a, b])) text_owned("C {}", &k) template: "D {}", args: vec![s.to_owned()]"#
        ),
        [
            ("A {} b {}".to_owned(), 2),
            ("C {}".to_owned(), 1),
            ("D {}".to_owned(), 1)
        ]
    );
}
