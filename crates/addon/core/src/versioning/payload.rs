//! Parsing and writing one version struct the plain way: typed from its slice of the text,
//! unknown members kept in its `extra`; and the findings' paths placed below the payload.

use facet::Facet;

use super::ParseCx;
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, Span,
    code_for_deserialize_error_kind,
};
use crate::json::{
    EncodeError, JsonSlice, NullRule, SliceError, Unrepresentable, Value, keep_members, parse_at,
    parse_at_null_absent, reject_nulls, remove_null_members,
};

/// Parses a version struct from `payload` (typed, straight from the text, never from a
/// `Value`) and keeps its unknown members in the `extra` fields of the structs they belong to,
/// each reported as an `UnrecognizedField` warning. A failed typed parse is one error (its code
/// chosen by facet's error kind) at the failing member, and `None`.
pub fn parse_payload<'a, T: Facet<'a>>(
    payload: JsonSlice<'a>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<T> {
    let rule = cx.load.null_rule;
    let mut value = match read_with_rule::<T>(payload, rule) {
        Ok(value) => value,
        Err(error) => {
            d.push(slice_error(&error, &cx.at, payload, Severity::Error));
            return None;
        }
    };
    // Under the `Rejected` null rule a `null` where the struct reads a value is a type error.
    if rule == Some(NullRule::Rejected) && reject_nulls(&value, payload, &cx.at, d) {
        return None;
    }
    if let Ok(mut json) = crate::json::from_str_jsonc::<Value>(payload.text) {
        if rule == Some(NullRule::Absent) {
            // Each member read as absent is marked: the write-back writes its default.
            let absent = remove_null_members(T::SHAPE, &mut json);
            crate::json::tell_null_members(&absent, &cx.at, d);
        }
        let (unknown, misread) = keep_members(&mut value, &json);
        for (path, _) in unknown {
            // The game logs an unknown member at content-log level 2, a warning; a `Strict`
            // slice makes it an error after the parse.
            d.push(Diagnostic::new(
                DiagnosticCode::UnrecognizedField { text: None },
                Severity::Warning,
                Phase::Parse,
                Location::new(None, None, join(&cx.at, &path), None),
            ));
        }
        // A scalar its member read as another value is told where it is.
        for misread in misread {
            d.push(misread.finding(&cx.at));
        }
    }
    Some(value)
}

/// The typed read of `slice` under the family's null rule: the `Absent` rule reads a struct
/// member written as `null` as absent; otherwise the plain read.
pub(super) fn read_with_rule<'a, T: Facet<'a>>(
    slice: JsonSlice<'a>,
    rule: Option<NullRule>,
) -> Result<T, SliceError> {
    if rule == Some(NullRule::Absent) {
        parse_at_null_absent::<T>(slice)
    } else {
        parse_at::<T>(slice)
    }
}

/// A version struct as JSON, written as the crate writes documents (`json::to_json`).
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
