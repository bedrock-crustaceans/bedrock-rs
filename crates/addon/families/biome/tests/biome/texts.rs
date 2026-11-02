//! Every `VanillaText::Formatted` the biome module builds has one argument per `{}` of its
//! template: the templates are the constants of `biome/texts.rs`; the module's sources
//! are scanned for `text(NAME, &[…])` / `formatted(NAME, &[…])` calls and the arity checked
//! statically.

use std::collections::HashMap;
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

/// The text constants of `texts.rs`: name → number of `{}`.
fn templates(texts: &str) -> HashMap<String, usize> {
    let mut out = HashMap::new();
    for chunk in texts.split("pub(crate) const ").skip(1) {
        let Some((name, rest)) = chunk.split_once(": &str =") else {
            continue;
        };
        let body = rest.split(";\n").next().unwrap_or_default();
        out.insert(name.trim().to_owned(), body.matches("{}").count());
    }
    out
}

/// How many arguments a `[…]` list starting at `text` holds (top-level commas).
fn arity(text: &str) -> usize {
    let body = text
        .trim_start()
        .strip_prefix('[')
        .expect("an argument list");
    let (mut depth, mut count, mut token, mut in_string) = (0usize, 0usize, false, false);
    for c in body.chars() {
        if in_string {
            if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                token = true;
            }
            '[' | '(' => {
                depth += 1;
                token = true;
            }
            ']' if depth == 0 => break,
            ']' | ')' => depth -= 1,
            ',' if depth == 0 => {
                count += 1;
                token = false;
            }
            c if !c.is_whitespace() => token = true,
            _ => {}
        }
    }
    count + usize::from(token)
}

#[test]
fn every_formatted_text_has_one_argument_per_placeholder() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let templates = templates(&std::fs::read_to_string(root.join("texts.rs")).unwrap());
    assert!(templates.len() > 20, "the constants are read");
    let mut files = Vec::new();
    sources(&root, &mut files);
    let mut checked = 0;
    for file in files.iter().filter(|f| !f.ends_with("client")) {
        let source = std::fs::read_to_string(file).unwrap();
        for call in ["text(", "formatted("] {
            for (index, _) in source.match_indices(call) {
                let rest = &source[index + call.len()..];
                let Some((name, args)) = rest.split_once(',') else {
                    continue;
                };
                let name = name.trim();
                let Some(&placeholders) = templates.get(name) else {
                    continue;
                };
                let args = args.trim_start().strip_prefix('&').unwrap_or(args);
                assert_eq!(arity(args), placeholders, "{}: {name}", file.display());
                checked += 1;
            }
        }
    }
    assert!(checked >= 8, "{checked} calls checked");
}
