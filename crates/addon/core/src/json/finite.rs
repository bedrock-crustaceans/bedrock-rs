//! Numbers no double holds, in a value a family keeps as written.
//!
//! `1e999`, and an integer token of more digits than a double has, read as an
//! infinity. An infinity has no JSON text, so a value that keeps one could not be written
//! back. Where the game's behaviour for such a number is not known, the number
//! is held as the largest finite double of its sign, with a finding of the family's at its
//! place. This is the one implementation of the walk:
//! the family chooses the code and severity of the finding.

use std::collections::HashMap;
use std::ops::Range;

use super::Value;
use super::lex::{Kind, Lexer, Rules, Structure, key_text};
use super::slice::JsonSlice;
use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{
    Code, Diagnostic, Diagnostics, JsonPath, JsonPathSegment, Location, Phase, Severity, Span,
};

/// A number [`clamp_non_finite`] replaced.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Clamped {
    /// Where the number is, from the root of the value.
    pub path: JsonPath,
    /// The number's token in the file, when the text the value was read from was given and
    /// has a number there.
    pub span: Option<Span>,
}

/// [`clamp_non_finite`] with the finding most families give: one
/// `InvalidValue` information of the parse phase for each replaced number, at its path below
/// `at`, on its token when `source` is given. Returns whether a number was replaced.
///
/// An information of the crate's own: the game's `JsonCpp` reader reads the token with
/// `sscanf("%lf")` and holds the infinity without a line; the finite number is only for the
/// write-back. The Minecraft Document reader refuses such a text instead
/// ([`document_json_refusal`](crate::json::document_json_refusal)).
pub fn clamp_non_finite_warned(
    value: &mut Value,
    source: Option<JsonSlice<'_>>,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> bool {
    let clamped = clamp_non_finite(value, source);
    let any = !clamped.is_empty();
    for number in clamped {
        let mut path = at.clone();
        path.append(&number.path);
        d.push(Diagnostic::new(
            Code::from(&core_codes::INVALID_VALUE),
            Severity::Info,
            Phase::Parse,
            Location::new(None, None, path, number.span),
        ));
    }
    any
}

/// Replaces every number of `value` that is an infinity by the largest finite double of its
/// sign (`1.7976931348623157e308` or its negative) and lists them in document order.
///
/// `source` is the text `value` was read from: with it each number comes with the span of its
/// token as written, at file offsets (for a key written twice, the token of the last, the one
/// a value keeps). Linear in the value and the text; a value without such a number
/// is not changed and the text is not looked at. A NaN, which no JSON text reads as, is left.
pub fn clamp_non_finite(value: &mut Value, source: Option<JsonSlice<'_>>) -> Vec<Clamped> {
    if all_finite(value) {
        return Vec::new();
    }
    let mut paths = Vec::new();
    // Steps still to take, the next one last: a hand-built value may nest deeper than a
    // stack frame per level allows. One path is kept for the whole walk, a segment pushed on
    // the way into a node and popped on the way out, so the walk is linear in the value
    // however deep it is; a path is copied only for a number that is replaced.
    let mut at = JsonPath::new();
    let mut open: Vec<Step<'_>> = vec![Step::Enter(value, None)];
    while let Some(step) = open.pop() {
        let (value, segment) = match step {
            Step::Leave => {
                at.pop();
                continue;
            }
            Step::Enter(value, segment) => (value, segment),
        };
        match segment {
            None => {}
            Some(JsonPathSegment::Key(key)) => at.push(key),
            Some(JsonPathSegment::Index(index)) => at.push_index(index),
        }
        if segment.is_some() {
            open.push(Step::Leave);
        }
        let bound = value
            .as_number()
            .filter(|number| number.is_float())
            .map(facet_value::VNumber::to_f64_lossy)
            .filter(|real| real.is_infinite())
            .map(|real| if real < 0.0 { f64::MIN } else { f64::MAX });
        if let Some(bound) = bound {
            *value = Value::from(bound);
            paths.push(at.clone());
        } else if value.as_array().is_some() {
            let first = open.len();
            if let Some(array) = value.as_array_mut() {
                for (index, element) in array.as_mut_slice().iter_mut().enumerate() {
                    open.push(Step::Enter(element, Some(JsonPathSegment::Index(index))));
                }
            }
            open[first..].reverse();
        } else if value.as_object().is_some() {
            let first = open.len();
            if let Some(object) = value.as_object_mut() {
                for (key, member) in object.iter_mut() {
                    open.push(Step::Enter(
                        member,
                        Some(JsonPathSegment::Key(key.as_str())),
                    ));
                }
            }
            open[first..].reverse();
        }
    }
    let spans: HashMap<JsonPath, Span> = source
        .map(|source| infinite_tokens(source).into_iter().collect())
        .unwrap_or_default();
    paths
        .into_iter()
        .map(|path| Clamped {
            span: spans.get(&path).copied(),
            path,
        })
        .collect()
}

/// A step of the walk of [`clamp_non_finite`].
enum Step<'value> {
    /// Visit a node, reached from its parent by the segment (`None` for the value the walk
    /// started at).
    Enter(&'value mut Value, Option<JsonPathSegment<'value>>),
    /// The node the walk entered last has been walked: step out of its segment.
    Leave,
}

/// Whether no number of `value` is an infinity.
fn all_finite(value: &Value) -> bool {
    let mut open = vec![value];
    while let Some(value) = open.pop() {
        if let Some(number) = value.as_number() {
            if number.is_float() && number.to_f64_lossy().is_infinite() {
                return false;
            }
        } else if let Some(array) = value.as_array() {
            open.extend(array.iter());
        } else if let Some(object) = value.as_object() {
            open.extend(object.iter().map(|(_, member)| member));
        }
    }
    true
}

/// Every number token of `source` that spells an infinity, in the order of the text: its
/// path from the root of the slice and its span in the file. One pass over the tokens of a
/// text the door reads (a text it would refuse gives what was found before the fault).
pub(super) fn infinite_tokens(source: JsonSlice<'_>) -> Vec<(JsonPath, Span)> {
    let text = source.text;
    let mut lexer = Lexer::new(text.as_bytes(), Rules::strict(true));
    let mut open = Structure::new();
    let mut found = Vec::new();
    while let Ok(Some(token)) = lexer.next_token() {
        match token.kind {
            Kind::OpenObject => open.open_object(),
            Kind::OpenArray => open.open_array(),
            Kind::Comma => open.comma(),
            Kind::String { .. } if open.awaits_key() => {
                open.enter_key(&key_text(&text[token.range()]));
            }
            Kind::CloseObject | Kind::CloseArray | Kind::String { .. } | Kind::Number => {
                if token.kind == Kind::Number
                    && text[token.range()]
                        .parse::<f64>()
                        .is_ok_and(f64::is_infinite)
                {
                    found.push((open.path().clone(), span(source, token.range())));
                }
                if matches!(token.kind, Kind::CloseObject | Kind::CloseArray) {
                    open.close();
                }
                // A whole value: at the root it ends the document.
                if open.depth() == 0 {
                    break;
                }
            }
            // A colon, a literal, or what is no JSON: nothing the walk keeps.
            _ => {}
        }
    }
    found
}

/// The range of the slice's text as a span of the file.
fn span(source: JsonSlice<'_>, range: Range<usize>) -> Span {
    let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    Span {
        offset: to_u32(source.offset + range.start),
        len: to_u32(range.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_cost::{assert_each_linear_timed_at, timed};

    /// One path is kept for the walk and copied only for a number that is
    /// replaced. `n` nodes, each holding one number no double holds, flat and seven lists
    /// further down, without the text and with it (the spans of the tokens).
    #[test]
    fn clamping_the_infinities_is_linear_in_the_numbers() {
        let nested = |number: &str| format!("{}{number}{}", "[".repeat(7), "]".repeat(7));
        assert_each_linear_timed_at(
            [
                "core clamp_non_finite, no text",
                "core clamp_non_finite, with the text",
            ],
            15_000,
            |n| {
                let nodes: Vec<String> = (0..n)
                    .map(|i| {
                        let number = if i % 2 == 0 { "1e999" } else { "-1e999" };
                        if i % 4 < 2 {
                            format!(r#"{{"v{i}": {number}}}"#)
                        } else {
                            format!(r#"{{"v{i}": {}}}"#, nested(number))
                        }
                    })
                    .collect();
                let text = format!("[{}]", nodes.join(", "));
                let read = || -> Value { crate::json::from_str_jsonc(&text).expect("test JSON") };
                let [mut plain, mut spanned] = [read(), read()];
                let (clamped, without) = timed(|| clamp_non_finite(&mut plain, None));
                assert_eq!(clamped.len(), n);
                assert!(clamped.iter().all(|clamped| clamped.span.is_none()));
                assert_eq!(clamped[0].path.to_string(), "0/v0");
                assert_eq!(
                    clamped[n - 1].path.to_string(),
                    format!("{}/v{}{}", n - 1, n - 1, "/0".repeat(7))
                );
                let (clamped, with) =
                    timed(|| clamp_non_finite(&mut spanned, Some(JsonSlice::new(&text))));
                assert_eq!(clamped.len(), n);
                for clamped in [&clamped[0], &clamped[n - 1]] {
                    let span = clamped.span.expect("the number's token");
                    let token = &text[span.offset as usize..(span.offset + span.len) as usize];
                    assert!(token.ends_with("1e999"), "{token}");
                }
                assert_eq!(plain, spanned);
                assert!(all_finite(&plain));
                [without, with]
            },
        );
    }
}
