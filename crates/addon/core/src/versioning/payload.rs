//! Parsing and writing one version struct the plain way: typed from its slice of the text,
//! unknown members kept in the document's table; and the findings' paths placed below the payload.

use facet::Facet;

use super::ParseCx;
use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{
    Class, Code, Diagnostic, Diagnostics, JsonPath, Location, Phase, Severity, Span,
    code_for_deserialize_error_kind,
};
use crate::json::{
    EncodeError, JsonSlice, NullRule, ScalarFailure, ScalarRead, SliceError, Unrepresentable,
    Value, parse_with_rules, reject_nulls,
};
use crate::load::LoadContext;
use crate::schema::UnknownRule;

/// Parses a version struct from `payload` (typed, straight from the text, never from a
/// `Value`) and keeps the unknown members of its structs that keep them in `d`'s table
/// (below `cx.at`; which keep is `json::walk`'s rule, under the slice's
/// `Reading::keeps_unknown`), each told by its disposition: kept or reported, an
/// `UnrecognizedField`; `schema::unknown = "reject"` a `DroppedField` and `"ignore"` an
/// `IgnoredMember`; each graded by the slice's reading (`LoadContext::grade`). Its scalars are
/// read by the slice's scalar rules (`LoadContext::scalar_rules`, the scalar pre-pass). A failed typed parse
/// is one error (its code chosen by facet's error kind) at the failing member, and `None`.
pub fn parse_payload<'a, T: Facet<'a>>(
    payload: JsonSlice<'a>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<T> {
    let rule = cx.load.null_rule;
    let ScalarRead {
        value,
        told,
        json,
        absent,
    } = match parse_with_rules::<T>(payload, rule, cx.load.scalar_rules) {
        Ok(read) => read,
        Err(failure) => {
            d.push(failure_finding(&failure, &cx.at, payload));
            return None;
        }
    };
    // Under the `Rejected` null rule a `null` where the struct reads a value is a type error.
    if rule == Some(NullRule::Rejected) && reject_nulls(&value, payload, &cx.at, d) {
        return None;
    }
    if let Some(json) = json {
        // Each member read as absent (rule N1; the pre-pass left it out of `json`) is marked:
        // the write-back writes its default.
        crate::json::tell_null_members(&absent, &cx.at, d);
        let mut kept = crate::document::Unknown::new();
        let unknown =
            crate::json::door_unknown_members(&value, &json, cx.load.keeps_unknown, &mut kept);
        d.unknown_mut().absorb(&cx.at, kept);
        tell_unknown(unknown, &cx.at, cx.load, d);
        // A scalar its member read as another value is told where it is.
        for told in told {
            d.push(told.finding(&cx.at));
        }
    }
    Some(value)
}

/// Tells each unknown member the door found, at `base` followed by its path, by its
/// disposition: kept, or not kept by a struct that says nothing or `report`, an
/// `UnrecognizedField` (the game logs an unknown member at content-log level 2, an error under
/// a strict slice's mask); rejected (`schema::unknown = "reject"`) a `DroppedField`, ignored
/// (`"ignore"`) an `IgnoredMember`. Each is graded by the slice's reading
/// (`LoadContext::grade`).
pub(super) fn tell_unknown(
    unknown: Vec<(JsonPath, UnknownRule)>,
    base: &JsonPath,
    cx: &LoadContext,
    d: &mut Diagnostics,
) {
    for (path, rule) in unknown {
        let (code, severity) = match rule {
            UnknownRule::Reject => (
                Code::from(&core_codes::DROPPED_FIELD),
                cx.grade(Class::Unknown),
            ),
            UnknownRule::Ignore => (
                Code::from(&core_codes::IGNORED_MEMBER),
                cx.grade(Class::Unknown),
            ),
            UnknownRule::Keep | UnknownRule::Report => (
                Code::from(&core_codes::UNRECOGNIZED_FIELD),
                cx.grade(Class::Unknown),
            ),
        };
        d.push(Diagnostic::new(
            code,
            severity,
            Phase::Parse,
            Location::new(None, None, join(base, &path), None),
        ));
    }
}

/// The finding of a read that failed, at `base` followed by the failing member: an error of
/// facet's kind, or the scalar of an untagged enum no arm took (a type error at the enum).
pub(super) fn failure_finding(
    failure: &ScalarFailure,
    base: &JsonPath,
    slice: JsonSlice<'_>,
) -> Diagnostic {
    match failure {
        ScalarFailure::Read(error) => slice_error(error, base, slice, Severity::Error),
        ScalarFailure::Refused(refused) => refused.misread.finding(base),
    }
}

/// A version struct as JSON, written as the crate writes documents (`json::to_json`).
///
/// # Errors
///
/// Returns `EncodeError` when the struct has no JSON form.
pub fn write_payload<'f, T: Facet<'f>>(value: &T) -> Result<Value, EncodeError> {
    let text = crate::json::to_json(value)?;
    // The writer's output is JSON this reader accepts; were it not, the value has no JSON form.
    crate::json::from_str::<Value>(&text).map_err(|_| EncodeError::NotRepresentable {
        at: JsonPath::new(),
        what: Unrepresentable::Unsupported,
    })
}

/// The finding of a failed typed parse of `slice`, at `base` followed by the failing member.
pub(super) fn slice_error(
    error: &SliceError,
    base: &JsonPath,
    slice: JsonSlice<'_>,
    severity: Severity,
) -> Diagnostic {
    let blank = slice.text.trim().is_empty();
    let code = code_for_deserialize_error_kind(&error.error.kind, !error.path.is_empty(), blank);
    Diagnostic::new(
        code,
        severity,
        Phase::Parse,
        Location::new(None, None, join(base, &error.path), Some(error.span)),
    )
}

/// The root span of a slice.
pub(super) fn span_of(slice: JsonSlice<'_>) -> Span {
    let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    Span {
        offset: to_u32(slice.offset),
        len: to_u32(slice.text.len()),
    }
}

/// `base` followed by the segments of `relative`.
pub(super) fn join(base: &JsonPath, relative: &JsonPath) -> JsonPath {
    let mut path = base.clone();
    path.append(relative);
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_continues_the_base_with_the_relative_path() {
        let mut base = JsonPath::new();
        base.push("minecraft:block");
        base.push("components");
        let mut relative = JsonPath::new();
        relative.push("minecraft:friction");
        relative.push_index(3);
        let joined = join(&base, &relative);
        assert_eq!(
            joined.to_string(),
            "minecraft:block/components/minecraft:friction/3"
        );
        assert_eq!(join(&base, &JsonPath::new()), base);
    }
}
