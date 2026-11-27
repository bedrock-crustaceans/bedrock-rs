//! The crate-private half of `tests/characterize_lex.rs`: the scanners of the parser that are
//! not public, over the same hostile texts, compared with
//! `tests/fixtures/characterize_lex/private.txt` (what they answered before the fold onto
//! `json::lex`). A line changes only in the commit that changes the behaviour on purpose.

use std::fmt::Write as _;

include!("../../tests/fixtures/characterize_lex/texts.rs");

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
