//! Every `VanillaText::Formatted` the recipe module builds has one argument per `{}` of its
//! template: the module builds them only through `formatted("…", [ … ])`,
//! and the sources are scanned for those calls.

use std::path::PathBuf;

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

/// How many elements the `[…]` list starting at `text` holds.
fn arity(text: &str) -> Option<usize> {
    let body = text.trim_start().strip_prefix('[')?;
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

/// `(template, argument count)` of every `formatted("…", [ … ])` call in `source`.
fn calls(source: &str) -> Vec<(String, Option<usize>)> {
    source
        .match_indices("formatted(")
        .filter(|(index, _)| {
            !source[..*index]
                .chars()
                .last()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        })
        .filter_map(|(index, _)| {
            let (template, rest) = literal(&source[index + "formatted(".len()..])?;
            let rest = rest.trim_start().strip_prefix(',')?;
            Some((template, arity(rest)))
        })
        .collect()
}

#[test]
fn every_formatted_text_has_one_argument_per_placeholder() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut checked = 0;
    let mut wrong = Vec::new();
    for entry in std::fs::read_dir(&root).unwrap() {
        let path = entry.unwrap().path();
        let source = std::fs::read_to_string(&path).unwrap();
        for (template, count) in calls(&source) {
            checked += 1;
            if Some(template.matches("{}").count()) != count {
                wrong.push(format!("{}: {template:?} with {count:?}", path.display()));
            }
        }
        // no other way of building a formatted text
        let direct = source.matches("VanillaText::Formatted {").count();
        assert!(
            direct <= 1,
            "{}: build formatted texts through `formatted`",
            path.display()
        );
    }
    assert!(checked >= 15, "the scan finds the templates ({checked})");
    assert!(wrong.is_empty(), "{wrong:#?}");
}
