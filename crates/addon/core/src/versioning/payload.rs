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

/// Reads a version struct from `payload` on the new reader ([`FromJson`](crate::schema::FromJson),
/// member by member), under the slice's reading (`cx.load.reading`), by the profile of the
/// typed door ([`Profile::FIRST_FAILURE`](crate::json::kit::Profile::FIRST_FAILURE)): what [`parse_payload`] tells for the same
/// text, at the same paths, each finding with the span of the value it is about. The read
/// stops at its first failure, which is then all it tells; a document that reads tells, after
/// it, the members written `null` that its reading refuses, those read as absent, its unknown
/// members (kept ones in `d`'s table) and the scalars read as another value. `None` when a
/// member failed (or a member written `null` was refused); a value whose only errors are its
/// unknown members or its converted scalars is built, as `parse_payload` builds it, for the
/// pipeline to drop. No bound is checked here: the family runs `Schema::check` where it runs it
/// today.
///
/// ```
/// use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
/// use bedrock_addon_core::json::JsonSlice;
/// use bedrock_addon_core::load::LoadContext;
/// use bedrock_addon_core::schema::{self, Schema};
/// use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
/// use bedrock_addon_core::versioning::{ParseCx, read_payload};
///
/// #[derive(Debug, PartialEq, facet::Facet, Schema)]
/// struct Lamp {
///     level: u8,
///     #[facet(schema::fallback = false)]
///     lit: Option<bool>,
/// }
///
/// let load = LoadContext::detached();
/// let cx = ParseCx {
///     load: &load,
///     written: FormatVersionInput { version: Some(FormatVersion::new(1, 0, 0)), beta_literal: false, prerelease: false },
///     original_version: FormatVersion::new(1, 0, 0),
///     at: JsonPath::of_keys(["minecraft:lamp"]),
/// };
/// let mut d = Diagnostics::default();
/// let lamp: Option<Lamp> = read_payload(JsonSlice::new(r#"{"level": 3}"#), &cx, &mut d);
/// assert_eq!(lamp, Some(Lamp { level: 3, lit: None }));
/// let lamp: Option<Lamp> = read_payload(JsonSlice::new(r#"{"level": "high"}"#), &cx, &mut d);
/// assert_eq!(lamp, None);
/// let told: Vec<String> = d.iter().map(|f| format!("{} {}", f.location.json_path, f.code)).collect();
/// assert_eq!(told, ["minecraft:lamp/level wrong JSON type"]);
/// ```
pub fn read_payload<T: crate::schema::FromJson>(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<T> {
    read_payload_with(payload, cx, d, |read| read)
}

/// [`read_payload`] with the read set by `configure` before it reads: a family whose game
/// tells every failure reads by [`Profile::EVERY_FAILURE`](crate::json::kit::Profile::EVERY_FAILURE), one that
/// words the kit's findings its own way gives its [`Texts`](crate::json::kit::Texts).
///
/// ```
/// # use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
/// # use bedrock_addon_core::json::JsonSlice;
/// # use bedrock_addon_core::load::LoadContext;
/// # use bedrock_addon_core::schema::{self, Schema};
/// # use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
/// # use bedrock_addon_core::versioning::{ParseCx, read_payload_with};
/// use bedrock_addon_core::json::kit::Profile;
/// #[derive(Debug, PartialEq, facet::Facet, Schema)]
/// struct Pair {
///     a: u8,
///     b: u8,
/// }
/// # let load = LoadContext::detached();
/// # let cx = ParseCx {
/// #     load: &load,
/// #     written: FormatVersionInput { version: Some(FormatVersion::new(1, 0, 0)), beta_literal: false, prerelease: false },
/// #     original_version: FormatVersion::new(1, 0, 0),
/// #     at: JsonPath::new(),
/// # };
/// let mut d = Diagnostics::default();
/// let text = JsonSlice::new(r#"{"a": "x", "b": "y"}"#);
/// let pair: Option<Pair> = read_payload_with(text, &cx, &mut d, |read| read.profile(Profile::EVERY_FAILURE));
/// assert_eq!(pair, None);
/// // Both failures, where the first failure's profile tells the first.
/// assert_eq!(d.len(), 2);
/// ```
pub fn read_payload_with<T: crate::schema::FromJson>(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    configure: impl for<'r> FnOnce(crate::json::kit::Read<'r>) -> crate::json::kit::Read<'r>,
) -> Option<T> {
    // One pass over the text gives the value and every span the read needs.
    let Ok(parsed) =
        crate::json::parse::parse_slice(payload, crate::json::parse::Options::default())
    else {
        // Not reached for a payload the envelope has read; the door's typed read words the
        // failure.
        if let Err(error) = crate::json::parse_at::<Value>(payload) {
            d.push(slice_error(&error, &cx.at, payload, Severity::Error));
        }
        return None;
    };
    let node = crate::json::kit::Node::parsed(&parsed, cx.at.clone());
    let read = crate::json::kit::Read::new(cx.load.reading, d)
        .unknown_severity(cx.load.grade(Class::Unknown))
        .profile(crate::json::kit::Profile::FIRST_FAILURE)
        .load(cx.load)
        .format(Some(cx.original_version));
    let mut read = configure(read);
    if let Some(finding) = no_object::<T>(&node, &read) {
        read.push(finding);
        return None;
    }
    crate::schema::read_document(&node, &mut read)
}

/// The finding of a root that is no object where `T` reads one, when the read stops at its
/// first failure: that is facet's typed read, which names no member for a value of another
/// kind than the struct's (a `null` is the null rule's), but tells its malformed input in its
/// words.
pub(super) fn no_object<T: crate::schema::FromJson>(
    node: &crate::json::kit::Node<'_>,
    read: &crate::json::kit::Read<'_>,
) -> Option<Diagnostic> {
    let json = node.value();
    if !read.telling().stops()
        || T::EXPECTED != Some(crate::json::kit::Expected::Object)
        || json.is_object()
        || json.is_null()
    {
        return None;
    }
    let got = if json.is_array() {
        "sequence start"
    } else {
        "scalar"
    };
    let mut finding = Diagnostic::of(crate::diagnostic::builders::syntax(format!(
        "unexpected token: got {got}, expected struct start"
    )))
    .at(node.path().clone());
    finding.location.span = node.span();
    Some(finding)
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
            UnknownRule::Silent => continue,
            UnknownRule::Reject => (
                Code::from(&core_codes::DROPPED_FIELD),
                cx.grade(Class::Unknown),
            ),
            UnknownRule::Ignore => (
                Code::from(&core_codes::IGNORED_MEMBER),
                cx.grade(Class::Unknown),
            ),
            UnknownRule::Component => (
                Code::from(&core_codes::UNRECOGNIZED_COMPONENT),
                cx.grade(Class::Unknown),
            ),
            UnknownRule::Keep | UnknownRule::Report => (
                Code::from(&core_codes::UNRECOGNIZED_FIELD),
                cx.grade(Class::Unknown),
            ),
            UnknownRule::NotKept => (
                Code::from(&core_codes::MEMBER_NOT_KEPT),
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
pub(crate) fn failure_finding(
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
    crate::json::from_str_jsonc::<Value>(&text).map_err(|_| EncodeError::NotRepresentable {
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
