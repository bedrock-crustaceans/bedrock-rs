//! Parsing and writing one version struct the plain way: typed from its slice of the text,
//! unknown members kept in the document's table; and the findings' paths placed below the payload.

use facet::Facet;

use super::ParseCx;
use crate::diagnostic::{
    Class, Diagnostic, Diagnostics, JsonPath, Location, Phase, Severity, Span,
};
use crate::json::{EncodeError, JsonSlice, Unrepresentable, Value};

/// Reads a version struct from `payload` on the new reader ([`FromJson`](crate::schema::FromJson),
/// member by member), under the slice's reading (`cx.load.reading`), by the profile of
/// facet-json's typed read
/// ([`Profile::FIRST_FAILURE`](crate::json::kit::Profile::FIRST_FAILURE)): what the old
/// `parse_payload` told for the same text, at the same paths, each finding with the span of the
/// value it is about. The read stops at its first failure, which is then all it tells; a
/// document that reads tells, after it, the members written `null` that its reading refuses,
/// those read as absent, its unknown members (kept ones in `d`'s table) and the scalars read as
/// another value. `None` when a member failed (or a member written `null` was refused); a value
/// whose only errors are its unknown members or its converted scalars is built, as the old
/// `parse_payload` built it, for the pipeline to drop. No bound is checked here: the family runs
/// `Schema::check` where it runs it today.
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
        // Not reached for a payload the envelope has read.
        if let Err(fault) =
            crate::json::parse::parse_slice(payload, crate::json::parse::Options::default())
        {
            d.push(fault_finding(&fault, &cx.at, payload, Severity::Error));
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

/// A version struct as JSON, written as the crate writes documents (`json::to_json`).
///
/// # Errors
///
/// Returns `EncodeError` when the struct has no JSON form.
pub fn write_payload<'f, T: Facet<'f>>(value: &T) -> Result<Value, EncodeError> {
    let text = crate::json::to_json(value)?;
    // The writer's output is JSON this reader accepts; were it not, the value has no JSON form.
    crate::json::from_str_jsonc(&text).map_err(|_| EncodeError::NotRepresentable {
        at: JsonPath::new(),
        what: Unrepresentable::Unsupported,
    })
}

/// The finding of a syntax fault of `slice`, at `base`.
pub(super) fn fault_finding(
    fault: &crate::json::parse::Fault,
    base: &JsonPath,
    slice: JsonSlice<'_>,
    severity: Severity,
) -> Diagnostic {
    let blank = slice.text.trim().is_empty();
    Diagnostic::new(
        fault.code(blank),
        severity,
        Phase::Parse,
        Location::new(
            None,
            None,
            base.clone(),
            Some(crate::json::slice_fault_span(fault, slice)),
        ),
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
