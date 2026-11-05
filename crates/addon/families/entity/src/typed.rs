//! The typed read of a part of an entity (a component, a goal): facet's read of the slice
//! behind the scalar pre-pass, under the scalar rules every slice of the family reads by, and
//! the finding of a read that fails.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, JsonPath, JsonPathSegment, Severity, Span,
};
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

/// The span of the key (its quotes included) of the member whose value spans `value` in
/// `slice`: the string before the `:` before the value.
fn key_span(slice: JsonSlice<'_>, value: Span) -> Option<Span> {
    let text = slice.text.as_bytes();
    let start = usize::try_from(value.offset)
        .ok()?
        .checked_sub(slice.offset)?;
    let before = text.get(..start)?;
    let colon = before.iter().rposition(|b| !b.is_ascii_whitespace())?;
    if before[colon] != b':' {
        return None;
    }
    let close = before[..colon]
        .iter()
        .rposition(|b| !b.is_ascii_whitespace())?;
    if before[close] != b'"' {
        return None;
    }
    // The opening quote: the first `"` before the closing one that no backslash escapes.
    let mut open = close;
    loop {
        open = before[..open].iter().rposition(|b| *b == b'"')?;
        let escapes = before[..open]
            .iter()
            .rev()
            .take_while(|b| **b == b'\\')
            .count();
        if escapes % 2 == 0 {
            break;
        }
    }
    Some(Span {
        offset: u32::try_from(slice.offset + open).ok()?,
        len: u32::try_from(close + 1 - open).ok()?,
    })
}

/// The finding of a failed typed read; the span only when the text read is the file's
/// (`as_written`).
pub(crate) fn finding(
    error: &ScalarFailure,
    as_written: Option<JsonSlice<'_>>,
    severity: Severity,
) -> Diagnostic {
    let ScalarFailure::Read(error) = error else {
        // A scalar no arm of an untagged enum takes: a type error at the enum.
        let at = failure_path(error);
        let finding = Diagnostic::error(DiagnosticCode::InvalidType { text: None })
            .at(at.clone())
            .severity(severity);
        // Spanned as facet spans a member it fails: the member's key (the value's own span
        // for a list element).
        let span = as_written.and_then(|slice| {
            let value = span_at(slice, &at)?;
            match at.segments().last() {
                Some(JsonPathSegment::Key(_)) => key_span(slice, value),
                _ => Some(value),
            }
        });
        return match span {
            Some(span) => finding.span(span),
            None => finding,
        };
    };
    let finding = Diagnostic::error(error.code_with_path(error.has_path()))
        .at(error.path.clone())
        .severity(severity);
    if as_written.is_some() {
        finding.span(error.span)
    } else {
        finding
    }
}
