//! The reading kit of the camera and aim-assist documents, which are read whole on the raw-text
//! path: the typed parse after the shape check, the identity of a payload that names itself,
//! the upgrade of an as-written document, the closing line, the source version and the root a
//! raw-text struct writes. The files themselves are read by core's `parse_document`.

use facet::Facet;

use super::shape::{self, Shape};
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Span, VanillaText,
    code_for_deserialize_error_kind,
};
use crate::document::{Document, Rejected};
use crate::json::{JsonSlice, ScalarFailure, Value, parse_with_rules, probe_envelope};
use crate::load::LoadContext;
use crate::version::FormatVersionInput;
use crate::versioning::{
    BeyondWidth, Family, Identity, Loaded, ParseCx, Parsed, Reading, Scalars, beta_refused, finish,
};

/// `reading` with the scalars of vanilla's strict cereal readers, which read every document of
/// these families: a member takes only a token of its own type (an integer member no real), and
/// a number beyond its float's width is a type error. [`typed`] reads by it.
#[must_use]
pub const fn strict(reading: Reading) -> Reading {
    reading
        .with_scalars(Scalars::Strict)
        .with_beyond_width(BeyondWidth::Error)
}

/// The closing line `text` of a failed file.
pub fn closing_line(diagnostics: &mut Diagnostics, text: &'static str) {
    diagnostics.push(Diagnostic::error(DiagnosticCode::DocumentRejected {
        text: Some(VanillaText::Fixed(text)),
    }));
}

/// `history::upgrade` of family `F`: rebuilds the parse stage from the document and finishes
/// it. `at` is the payload's path (the root for a family read whole).
///
/// # Errors
///
/// `Rejected` when the upgrade or the family's checks leave no value; it carries the located
/// findings.
pub fn upgrade<F: Family>(
    document: Document<F::History>,
    at: JsonPath,
    cx: &LoadContext,
) -> Result<Document<F::Latest>, Rejected> {
    let (value, origin, version, diagnostics) = document.into_parsed_parts();
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<F>::from_value(Some(value), version, written, at, diagnostics);
    let mut outcome = finish(parsed, cx);
    // A beta document is refused as the blanket `Content::parse` refuses it (core's
    // `beta_refused`), and the file closed as failed.
    if matches!(outcome.value, Some(Loaded::Beta(_))) {
        outcome.diagnostics.push(beta_refused());
    }
    let failed = !matches!(outcome.value, Some(Loaded::Stable(_)));
    <F as Family>::close(&mut outcome.diagnostics, None, &origin.path, failed);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(value)) => {
            Ok(Document::new(value, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The identity of a payload whose identifier is its own member `key` (not in a
/// `description`).
pub fn identity(payload: JsonSlice<'_>, key: &str) -> Identity {
    let identifier = crate::json::from_str_jsonc::<Value>(payload.text)
        .ok()
        .and_then(|value| {
            Some(
                value
                    .as_object()?
                    .get(key)?
                    .as_string()?
                    .as_str()
                    .to_owned(),
            )
        });
    Identity {
        identifier,
        runtime_identifier: None,
    }
}

/// The JSON of `text`, a slice of the text `versioning::parse_as_written` read, in which a
/// member written twice has its last value only (core's last-wins rule).
pub fn last_wins(text: &str) -> Option<Value> {
    crate::json::from_str_jsonc::<Value>(&crate::json::last_wins(text)).ok()
}

/// The first byte of `text` that is neither white space nor inside a comment.
fn first_token(text: &str) -> Option<u8> {
    let mut rest = text.trim_start();
    loop {
        if let Some(comment) = rest.strip_prefix("//") {
            rest = comment.split_once('\n').map_or("", |(_, after)| after);
        } else if let Some(comment) = rest.strip_prefix("/*") {
            rest = comment.split_once("*/").map_or("", |(_, after)| after);
        } else {
            return rest.bytes().next();
        }
        rest = rest.trim_start();
    }
}

/// The typed parse of `slice` after its JSON passed `shape` (the whole of `checked`, which
/// `slice` is, or is the member `key` of). `None` with the findings when either fails.
pub fn typed<'a, T: for<'f> Facet<'f>>(
    checked: JsonSlice<'a>,
    shape: &Shape,
    key: Option<&str>,
    unknown: &'static str,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<T> {
    typed_as(checked, shape, key, unknown, false, cx, d)
}

/// [`typed`] as the game's non-strict cereal reader reads (the legacy camera preset slice):
/// before the shape check each scalar of the other type is read as the bound one
/// ([`shape::coerce_scalars`]), each such place carries an information (`ValueRewritten`), and
/// the typed parse reads the rewritten JSON.
pub fn typed_unstrict<'a, T: for<'f> Facet<'f>>(
    checked: JsonSlice<'a>,
    shape: &Shape,
    key: Option<&str>,
    unknown: &'static str,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<T> {
    typed_as(checked, shape, key, unknown, true, cx, d)
}

/// [`typed`], or [`typed_unstrict`] when `coerce`.
fn typed_as<'a, T: for<'f> Facet<'f>>(
    checked: JsonSlice<'a>,
    shape: &Shape,
    key: Option<&str>,
    unknown: &'static str,
    coerce: bool,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<T> {
    let Some(mut json) = last_wins(checked.text) else {
        d.push(shape::error(
            DiagnosticCode::InvalidType { text: None },
            cx.at.clone(),
        ));
        return None;
    };
    let mut coerced = Vec::new();
    if coerce {
        shape::coerce_scalars(&mut json, shape, &JsonPath::new(), &mut coerced);
    }
    if !shape::check(&json, shape, &cx.at, unknown, d) {
        return None;
    }
    for place in &coerced {
        d.push(coerced_finding(checked, &cx.at, place));
    }
    let (slice, at) = match key {
        Some(key) => (
            probe_envelope(checked).ok()?.payload(key)?,
            shape::child(&cx.at, key),
        ),
        None => (checked, cx.at.clone()),
    };
    // The rewritten JSON, when a scalar was coerced: the text the typed parse reads. A
    // failure's place is still found in the file's text, by its path.
    let rewritten = if coerced.is_empty() {
        None
    } else {
        let payload = match key {
            Some(key) => json.as_object()?.get(key)?,
            None => &json,
        };
        Some(crate::json::to_json(payload).ok()?)
    };
    // The scalars are read by the slice's scalar rules (`LoadContext::scalar_rules`): the
    // strict cereal readers' rule for every family read here.
    let rules = cx.load.scalar_rules;
    let read = match &rewritten {
        Some(text) => parse_with_rules::<T>(JsonSlice::new(text), None, rules),
        None => parse_with_rules::<T>(slice, None, rules),
    };
    let failure = match read {
        Ok(read) => {
            for told in &read.told {
                d.push(told.finding(&at));
            }
            return Some(read.value);
        }
        Err(ScalarFailure::Read(failure)) => failure,
        // A scalar no arm of an untagged enum takes: a type error at the enum.
        Err(ScalarFailure::Refused(misread)) => {
            d.push(misread.spanned_finding(&at));
            return None;
        }
    };
    // The text is core's, in which a member written twice has its last value only
    // (`versioning::parse_as_written` blanks the earlier copies), so the one read decides. No
    // message of facet's is looked at: a duplicate member is never recognised by wording.
    let code =
        code_for_deserialize_error_kind(&failure.error.kind, !failure.path.is_empty(), false);
    // The failing value's own place in the file; facet's span only for a failure that names
    // no member (the whole slice when the text read was the rewritten one).
    let span = if !failure.path.is_empty() {
        span_in(slice, &failure.path)
    } else if rewritten.is_some() {
        span_in(slice, &JsonPath::new())
    } else {
        failure.span
    };
    d.push(typed_failure(code, &at, &failure.path, span));
    None
}

/// The information at `place` (a path below `checked`), a scalar of the other JSON type read
/// as the bound one: write-back writes the member's own type there, so nothing is changed in
/// silence.
fn coerced_finding(checked: JsonSlice<'_>, at: &JsonPath, place: &JsonPath) -> Diagnostic {
    let path = at.join(place);
    Diagnostic::info(DiagnosticCode::ValueRewritten { text: None })
        .at(path)
        .span(span_in(checked, place))
        .detail(crate::diagnostic::DiagnosticDetail::Note(
            shape::SCALAR_READ_AS_BOUND_TYPE,
        ))
}

/// The finding of a failed typed parse: `code` at `base` followed by the failing member.
fn typed_failure(
    code: DiagnosticCode,
    base: &JsonPath,
    member: &JsonPath,
    span: Span,
) -> Diagnostic {
    let path = base.join(member);
    let mut finding = shape::error(code, path);
    finding.location.span = Some(span);
    finding
}

/// The place in the file of the value at `path` below `slice`: the last value of each member
/// on the way (the one that stands). When a step cannot be taken (a missing member, a key the
/// path does not spell unambiguously), the span is the deepest value reached.
fn span_in(slice: JsonSlice<'_>, path: &JsonPath) -> Span {
    use crate::diagnostic::JsonPathSegment;
    use facet_json::RawJson;
    use indexmap::IndexMap;
    let mut current = slice;
    // The path's own segments, not its `Display` split on `/` (a key may hold one).
    for step in path.segments() {
        let next = match (first_token(current.text), step) {
            (Some(b'{'), JsonPathSegment::Key(key)) => {
                crate::json::from_str_borrowed_jsonc::<IndexMap<String, RawJson<'_>>>(current.text)
                    .ok()
                    .and_then(|members| current.nested(members.get(key)?))
            }
            (Some(b'['), JsonPathSegment::Index(index)) => {
                crate::json::from_str_borrowed_jsonc::<Vec<RawJson<'_>>>(current.text)
                    .ok()
                    .and_then(|elements| current.nested(elements.get(index)?))
            }
            _ => None,
        };
        match next {
            Some(next) => current = next,
            None => break,
        }
    }
    let narrow = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    Span {
        offset: narrow(current.offset),
        len: narrow(current.text.len()),
    }
}

/// `{"format_version": version, key: payload}`: the whole document of a family read whole.
pub fn document(version: &str, key: &str, payload: Value) -> Value {
    let mut root = facet_value::VObject::new();
    root.insert("format_version", version);
    root.insert(key, payload);
    root.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Location, Phase, Severity};

    #[test]
    fn the_closing_line_is_a_rejection_with_its_text() {
        let mut lines = Diagnostics::default();
        closing_line(&mut lines, "closing");
        let lines: Vec<_> = lines.iter().cloned().collect();
        assert_eq!(
            lines,
            [Diagnostic::new(
                DiagnosticCode::DocumentRejected {
                    text: Some(VanillaText::Fixed("closing")),
                },
                Severity::Error,
                Phase::Parse,
                Location::default()
            )]
        );
    }
}
