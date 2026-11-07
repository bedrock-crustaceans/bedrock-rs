//! The typed read of a part of an entity (a component, a goal): facet's read of the slice
//! behind the scalar pre-pass, under the scalar rules every slice of the family reads by, and
//! the finding of a read that fails.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, JsonPath, JsonPathSegment, Severity, Span};
use bedrock_addon_core::json::{JsonSlice, NullRule, ScalarFailure, parse_with_rules, span_at};
use facet::Facet;

/// Reads `slice` as `T` under `null` and the entity slices' scalar rules
/// ([`SCALAR_RULES`](crate::SCALAR_RULES)): every scalar judged against its member's type
/// before facet reads the text (the pre-pass). The typed read of every component and goal.
pub(crate) fn read<'a, T: Facet<'a>>(
    slice: JsonSlice<'a>,
    null: Option<NullRule>,
) -> Result<T, ScalarFailure> {
    // Under strict scalars nothing is converted or clamped, so nothing is told beside the value.
    parse_with_rules::<T>(slice, null, crate::SCALAR_RULES).map(|read| read.value)
}

/// Where a failed typed read failed, below the slice.
pub(crate) fn failure_path(error: &ScalarFailure) -> JsonPath {
    match error {
        ScalarFailure::Read(error) => error.path.clone(),
        ScalarFailure::Refused(misread) => misread.path.clone(),
    }
}

/// The span of the key (its quotes included) of the member at `at` whose value spans `value`:
/// the last key of its object before the value, found by reading the object's text forward
/// (strings and JSONC comments skipped, so a comment between key and value is passed over).
/// `None` for a path that names no member, or an object not in the text.
fn key_span(slice: JsonSlice<'_>, at: &JsonPath, value: Span) -> Option<Span> {
    let mut object = at.clone();
    if !matches!(at.segments().last(), Some(JsonPathSegment::Key(_))) || !object.pop() {
        return None;
    }
    let object = span_at(slice, &object)?;
    let start = usize::try_from(object.offset)
        .ok()?
        .checked_sub(slice.offset)?;
    let text = slice
        .text
        .as_bytes()
        .get(start..start + usize::try_from(object.len).ok()?)?;
    let value = usize::try_from(value.offset)
        .ok()?
        .checked_sub(slice.offset + start)?;
    let (mut depth, mut key, mut i) = (0_usize, None, 1);
    while i < text.len() {
        if i == value && depth == 0 {
            let (open, close) = key?;
            return Some(Span {
                offset: u32::try_from(slice.offset + start + open).ok()?,
                len: u32::try_from(close + 1 - open).ok()?,
            });
        }
        match (text[i], text.get(i + 1)) {
            (b'"', _) => {
                let open = i;
                i += 1;
                while *text.get(i)? != b'"' {
                    i += if text[i] == b'\\' { 2 } else { 1 };
                }
                if depth == 0 {
                    key = Some((open, i));
                }
            }
            (b'/', Some(b'/')) => {
                while text.get(i).is_some_and(|b| *b != b'\n') {
                    i += 1;
                }
            }
            (b'/', Some(b'*')) => {
                i += 2;
                while !(text.get(i)? == &b'*' && text.get(i + 1) == Some(&b'/')) {
                    i += 1;
                }
                i += 1;
            }
            (b'{' | b'[', _) => depth += 1,
            (b'}' | b']', _) => depth = depth.checked_sub(1)?,
            _ => {}
        }
        i += 1;
    }
    None
}

/// The finding of a failed typed read; the span only when the text read is the file's
/// (`as_written`).
pub(crate) fn finding(
    error: &ScalarFailure,
    as_written: Option<JsonSlice<'_>>,
    severity: Severity,
) -> Diagnostic {
    let refused = match error {
        ScalarFailure::Read(error) => {
            let finding = Diagnostic::error(error.code_with_path(error.has_path()))
                .at(error.path.clone())
                .severity(severity);
            return if as_written.is_some() {
                finding.span(error.span)
            } else {
                finding
            };
        }
        ScalarFailure::Refused(refused) => refused,
    };
    // A scalar no arm of an untagged enum takes: a type error at the enum, spanned as facet
    // spans a member it fails: the member's key (for a list element the scalar's own span,
    // the one core carries).
    let finding = Diagnostic::of(&core_codes::INVALID_TYPE)
        .at(refused.path.clone())
        .severity(severity);
    let span = as_written.and_then(|slice| {
        let value = refused.span?;
        match refused.path.segments().last() {
            Some(JsonPathSegment::Key(_)) => key_span(slice, &refused.path, value),
            _ => Some(value),
        }
    });
    match span {
        Some(span) => finding.span(span),
        None => finding,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Attack;

    /// The finding of reading `text` as an `attack` component, spanned as the file has it.
    fn refused(text: &str) -> Diagnostic {
        let slice = JsonSlice::new(text);
        let error = read::<Attack>(slice, None).expect_err("a string other than `infinite`");
        assert!(matches!(error, ScalarFailure::Refused(_)), "{error:?}");
        finding(&error, Some(slice), Severity::Error)
    }

    /// The span of the first `needle` in `text`.
    fn span_of(text: &str, needle: &str) -> Option<Span> {
        let offset = text.find(needle)?;
        Some(Span {
            offset: u32::try_from(offset).ok()?,
            len: u32::try_from(needle.len()).ok()?,
        })
    }

    #[test]
    fn a_refused_member_is_spanned_at_its_key() {
        let text = r#"{"effect_duration": "soon"}"#;
        let found = refused(text);
        assert!(found.is(&core_codes::INVALID_TYPE));
        assert_eq!(found.location.span, span_of(text, r#""effect_duration""#));
    }

    #[test]
    fn a_refused_member_keeps_its_key_span_past_a_comment_before_its_value() {
        for text in [
            r#"{"effect_duration": /* a "quoted": comment */ "soon"}"#,
            "{\"effect_duration\": // a line comment\n \"soon\"}",
            r#"{"effect_duration" /* before the colon */ : "soon"}"#,
            r#"{"damage": {"min": 1, "max": 2}, "effect_name": "a // \" /*", "effect_duration": "soon"}"#,
        ] {
            let found = refused(text);
            assert_eq!(
                found.location.span,
                span_of(text, r#""effect_duration""#),
                "{text}"
            );
        }
    }
}
