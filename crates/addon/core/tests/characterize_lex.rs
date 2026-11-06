//! Characterization of the door's text scanners before they fold onto one tokenizer
//! (`json::lex`): every public scanner over the hostile texts of
//! `fixtures/characterize_lex/texts.rs`, printed one line per scanner and compared with
//! `fixtures/characterize_lex/public.txt`, which is what they answered before the fold. Not a
//! test of what is right: of what is. A line changes only in the commit that changes the
//! behaviour on purpose, and says so.

use std::borrow::Cow;
use std::fmt::Write as _;

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::json::jsoncpp::lenient;
use bedrock_addon_core::json::{
    ClampPlan, JsonSlice, Prepared, Value, Width, check_depth, clamp_non_finite, clamp_text,
    clamp_tokens, from_str_jsonc, last_wins, last_wins_except, scan, span_at,
};

include!("fixtures/characterize_lex/texts.rs");

fn cow(text: Cow<'_, str>) -> String {
    match text {
        Cow::Borrowed(_) => "borrowed".to_owned(),
        Cow::Owned(text) if text.len() > 120 => format!("owned({} bytes)", text.len()),
        Cow::Owned(text) => format!("owned({text:?})"),
    }
}

fn findings(d: &Diagnostics) -> String {
    d.iter()
        .map(|finding| {
            let span = finding.location.span.map_or("-".to_owned(), |span| {
                format!("{}+{}", span.offset, span.len)
            });
            format!(
                "{:?} {}@{span}",
                finding.severity, finding.location.json_path
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn path(steps: &[&str]) -> JsonPath {
    let mut path = JsonPath::new();
    for step in steps {
        match step.parse::<usize>() {
            Ok(index) => path.push_index(index),
            Err(_) => path.push(step),
        }
    }
    path
}

/// Every public scanner's answer for `text`, a line each.
fn characterize(name: &str, text: &str, out: &mut String) {
    let bytes = text.as_bytes();
    let _ = writeln!(out, "## {name}");
    let _ = writeln!(out, "scan(comments)    {:?}", scan(bytes, true));
    let _ = writeln!(out, "scan(no comments) {:?}", scan(bytes, false));
    let _ = writeln!(out, "check_depth(128)  {:?}", check_depth(bytes, 128));
    let _ = writeln!(out, "check_depth(3)    {:?}", check_depth(bytes, 3));
    let _ = writeln!(out, "last_wins         {}", cow(last_wins(text)));
    let _ = writeln!(
        out,
        "last_wins_except  {}",
        cow(last_wins_except(text, &["a", "k"]))
    );
    let _ = writeln!(out, "lenient.root_end  {:?}", lenient::root_end(text));
    let _ = writeln!(
        out,
        "lenient.trivia    {:?}",
        lenient::skip_trivia(bytes, 0)
    );
    let quote = text.find('"');
    let _ = writeln!(
        out,
        "lenient.string    {:?}",
        quote.map(|at| lenient::string_end(bytes, at))
    );
    let _ = writeln!(
        out,
        "prepared.blank    {:?}",
        Prepared::new(bytes).map(|prepared| prepared.is_blank())
    );
    let _ = writeln!(
        out,
        "opens_an_object   {:?}",
        JsonSlice::new(text).opens_an_object()
    );
    for steps in [&["a"][..], &["a", "0"], &["b"]] {
        let _ = writeln!(
            out,
            "span_at({})  {:?}",
            steps.join("/"),
            span_at(JsonSlice::new(text), &path(steps))
        );
    }
    if text.len() > 200 {
        return;
    }
    let slice = JsonSlice { text, offset: 10 };
    let at = path(&["root"]);
    let float = |_: &[bedrock_addon_core::diagnostic::JsonPathSegment<'_>]| Some(Width::Float);
    for (label, first_kept_in) in [("clamp_tokens", &[][..]), ("clamp_tokens(a)", &["a"][..])] {
        let mut d = Diagnostics::default();
        let plan = ClampPlan::widths(&float).first_kept_in(first_kept_in);
        let clamped = clamp_tokens(slice, &plan, &at, &mut d);
        let _ = writeln!(
            out,
            "{label:<17} {:?} [{}]",
            clamped.map(|clamped| clamped.text),
            findings(&d)
        );
    }
    let mut d = Diagnostics::default();
    let clamped = clamp_tokens(slice, &ClampPlan::doubles(), &at, &mut d);
    let _ = writeln!(
        out,
        "clamp_double      {:?} [{}]",
        clamped.map(|clamped| clamped.text),
        findings(&d)
    );
    match from_str_jsonc::<Value>(text) {
        Ok(mut value) => {
            let mut d = Diagnostics::default();
            let clamped = clamp_text(slice, &value, &float, &at, &mut d);
            let _ = writeln!(
                out,
                "clamp_text        {:?} [{}]",
                clamped.map(|clamped| clamped.text),
                findings(&d)
            );
            let clamped: Vec<String> = clamp_non_finite(&mut value, Some(slice))
                .into_iter()
                .map(|clamped| {
                    let span = clamped.span.map_or("-".to_owned(), |span| {
                        format!("{}+{}", span.offset, span.len)
                    });
                    format!("{}@{span}", clamped.path)
                })
                .collect();
            let _ = writeln!(out, "clamp_non_finite  {clamped:?}");
        }
        Err(_) => {
            let _ = writeln!(out, "(no value)");
        }
    }
}

#[test]
fn the_public_scanners_answer_as_they_did_before_the_fold() {
    let mut out = String::new();
    for text in TEXTS {
        characterize(&format!("{text:?}"), text, &mut out);
    }
    for (name, text) in deep_texts() {
        characterize(&name, &text, &mut out);
    }
    let expected = include_str!("fixtures/characterize_lex/public.txt");
    if std::env::var_os("CHARACTERIZE_LEX_PRINT").is_some() {
        print!("{out}");
    }
    assert!(
        out == expected,
        "the scanners answer differently (CHARACTERIZE_LEX_PRINT=1 prints what they answer)"
    );
}
