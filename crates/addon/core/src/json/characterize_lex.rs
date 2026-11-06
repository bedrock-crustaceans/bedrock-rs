//! The crate-private half of `tests/characterize_lex.rs`: the scanners of the door that are
//! not public, over the same hostile texts, compared with
//! `tests/fixtures/characterize_lex/private.txt` (what they answered before the fold onto
//! `json::lex`). A line changes only in the commit that changes the behaviour on purpose.

use std::fmt::Write as _;

include!("../../tests/fixtures/characterize_lex/texts.rs");

/// The path `path_at` names at each offset of `text` where it differs from the offset before.
fn paths(text: &str) -> String {
    let mut out = String::new();
    let mut last = None;
    for offset in (0..=text.len()).filter(|&offset| text.is_char_boundary(offset)) {
        let path = super::path::path_at(text, offset).to_string();
        if last.as_ref() != Some(&path) {
            let _ = write!(out, " {offset}:{path:?}");
            last = Some(path);
        }
    }
    out
}

fn characterize(name: &str, text: &str, out: &mut String) {
    let _ = writeln!(out, "## {name}");
    let _ = writeln!(
        out,
        "first_fault          {:?}",
        super::read::first_fault(text)
    );
    let _ = writeln!(
        out,
        "first_beyond_double  {:?}",
        super::clamp_text::first_beyond_double(text)
    );
    let _ = writeln!(
        out,
        "has_empty_array      {:?}",
        super::slice::has_empty_array(text)
    );
    let _ = writeln!(
        out,
        "array_of_comments    {:?}",
        super::slice::has_array_of_comments(text)
    );
    if text.len() <= 200 {
        let _ = writeln!(out, "path_at             {}", paths(text));
    }
}

#[test]
fn the_private_scanners_answer_as_they_did_before_the_fold() {
    let mut out = String::new();
    for text in TEXTS {
        characterize(&format!("{text:?}"), text, &mut out);
    }
    for (name, text) in deep_texts() {
        characterize(&name, &text, &mut out);
    }
    let expected = include_str!("../../tests/fixtures/characterize_lex/private.txt");
    if std::env::var_os("CHARACTERIZE_LEX_PRINT").is_some() {
        print!("{out}");
    }
    assert!(
        out == expected,
        "the scanners answer differently (CHARACTERIZE_LEX_PRINT=1 prints what they answer)"
    );
}
