//! Every `VanillaText::Formatted` the dialogue module builds has one argument per `{}` of its
//! template: `texts.rs` is scanned for its templates and the module's sources for every
//! `formatted(TEMPLATE, &[…])`, the arity checked statically. The texts themselves are compared
//! byte for byte in the row and rule tests.

use std::collections::BTreeMap;
use std::path::PathBuf;

/// The module's directory.
fn module() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// `name → placeholders` of every template constant of `texts.rs`; each has the prefix's `{}`
/// (the stem) and the ones of its own text.
fn templates() -> BTreeMap<String, usize> {
    let source = std::fs::read_to_string(module().join("texts.rs")).unwrap();
    let mut found = BTreeMap::new();
    let mut rest = source.as_str();
    while let Some(start) = rest.find("pub(crate) const ") {
        rest = &rest[start + "pub(crate) const ".len()..];
        let name = rest.split(':').next().unwrap().to_owned();
        let end = rest.find(';').unwrap();
        let body = &rest[..end];
        assert!(body.contains("text!("), "{name} is not built by `text!`");
        found.insert(name, 1 + body.matches("{}").count());
        rest = &rest[end..];
    }
    found
}

/// How many top-level arguments the `[…]` list starting `text` holds.
fn arity(text: &str) -> usize {
    let (mut depth, mut count, mut token) = (0usize, 0usize, false);
    for c in text.chars() {
        match c {
            '(' | '[' => {
                depth += 1;
                token = true;
            }
            ']' if depth == 0 => return count + usize::from(token),
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                count += usize::from(token);
                token = false;
            }
            c if c.is_whitespace() => {}
            _ => token = true,
        }
    }
    panic!("unterminated argument list: {text}");
}

#[test]
fn every_formatted_text_has_one_argument_per_placeholder() {
    let templates = templates();
    assert_eq!(templates.len(), 16, "{templates:?}");
    let mut used = BTreeMap::new();
    for entry in std::fs::read_dir(module()).unwrap() {
        let path = entry.unwrap().path();
        let source = std::fs::read_to_string(&path).unwrap();
        let mut rest = source.as_str();
        while let Some(start) = rest.find("formatted(") {
            rest = &rest[start + "formatted(".len()..];
            let name = rest.split(',').next().unwrap().trim().to_owned();
            // The helper's own definition names no template.
            if !templates.contains_key(&name) {
                assert!(name.starts_with("template"), "{}: {name}", path.display());
                continue;
            }
            let list = &rest[rest.find("&[").unwrap() + 2..];
            assert_eq!(
                arity(list),
                templates[&name],
                "{}: formatted({name}, …)",
                path.display()
            );
            *used.entry(name).or_insert(0usize) += 1;
        }
    }
    let unused: Vec<_> = templates
        .keys()
        .filter(|name| !used.contains_key(*name))
        .collect();
    assert!(unused.is_empty(), "templates nothing builds: {unused:?}");
}
