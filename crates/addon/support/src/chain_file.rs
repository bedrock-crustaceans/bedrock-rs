//! One file of a camera or aim-assist family through the version chain: the text checks, the
//! public parse stage, the family's vanilla texts and the document or its rejection.

use std::convert::Infallible;

use facet::Facet;

use super::shape::{self, Shape};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, Span,
    VanillaText, code_for_deserialize_error_kind,
};
use bedrock_addon_core::document::{Document, Origin, Rejected};
use bedrock_addon_core::json::{
    EncodeError, EnvelopeVersion, JsonSlice, MAX_JSON_DEPTH, Value, check_depth, parse_at,
    probe_envelope, strip_bom,
};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::FormatVersionInput;
use bedrock_addon_core::versioning::{
    Family, Identity, Loaded, ParseCx, Parsed, VersionStruct, finish, parse_as_written,
};

/// A family of this module: no beta struct, and its own texts for the generic findings.
pub trait FileFamily: Family<Beta = Infallible, Latest: VersionStruct + Clone> {
    /// Rewrites the generic findings of a parse into the family's vanilla texts and, when the
    /// file `failed`, adds the family's closing line. `document` is the text, when it is one.
    fn close(
        diagnostics: &mut Diagnostics,
        document: Option<JsonSlice<'_>>,
        path: &PackPath,
        failed: bool,
    );

    /// The whole document of `value`, `format_version` included.
    ///
    /// # Errors
    ///
    /// `EncodeError::NotRepresentable` when `value` holds something JSON cannot spell, such as a
    /// non-finite number.
    fn document(value: &Self::Latest) -> Result<Value, EncodeError>;
}

/// The text of a file, after the BOM and the depth check; `None` when it is not UTF-8 or too
/// deep.
#[must_use]
pub fn text(bytes: &[u8]) -> Option<&str> {
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).ok()?;
    std::str::from_utf8(bytes).ok()
}

/// The finding of an unreadable file.
fn malformed() -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::MalformedInput {
            message: "not UTF-8 JSON within the nesting limit".to_owned(),
            text: None,
        },
        Severity::Error,
        Phase::Parse,
        Location::default(),
    )
}

/// Gives every finding the document's pack and file.
fn locate(diagnostics: &mut Diagnostics, origin: &Origin) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        diagnostics.push(finding);
    }
}

/// The `format_version` of `document` as written, for a message.
fn written_version(document: JsonSlice<'_>) -> String {
    match probe_envelope(document).map(|envelope| envelope.format_version) {
        Ok(EnvelopeVersion::String(text)) => text,
        Ok(EnvelopeVersion::Array([a, b, c])) => format!("{a}.{b}.{c}"),
        _ => String::new(),
    }
}

/// Below a versioned loader's first slice there is no parser at all: the "no parser available" text
/// where the generic floor check says "To use …, use json format version … or higher".
pub fn puv_below_floor(diagnostics: &mut Diagnostics, document: Option<JsonSlice<'_>>) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        if let DiagnosticCode::UnsupportedVersion {
            text: Some(text @ VanillaText::UseFormatVersion { .. }),
        } = &mut finding.code
        {
            *text = VanillaText::NoParserForVersion {
                version: document.map(written_version).unwrap_or_default(),
            };
        }
        diagnostics.push(finding);
    }
}

/// The closing line `text` of a failed file.
pub fn closing_line(diagnostics: &mut Diagnostics, text: &'static str) {
    diagnostics.push(Diagnostic::new(
        DiagnosticCode::DocumentRejected {
            text: Some(VanillaText::Fixed(text)),
        },
        Severity::Error,
        Phase::Parse,
        Location::default(),
    ));
}

/// What loading one file gives: the identifier its payload names, when it is a string, and the
/// document or its rejection.
pub type FileOutcome<T> = (Option<String>, Result<Document<T>, Rejected>);

/// Loads one file of family `F`.
pub fn parse_file<F: FileFamily>(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> FileOutcome<F::Latest> {
    // The BOM is blanked, not cut, so every offset is an offset into the file as written.
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = Origin::new(pack, side, path.clone(), None, Vec::new());
    let Some(text) = text(bytes) else {
        return (None, Err(unreadable::<F>(origin)));
    };
    let document = JsonSlice::new(text);
    let parsed = parse_as_written::<F>(document, cx);
    let identifier = parsed.identity.identifier.clone();
    (
        identifier,
        finish_file::<F>(parsed, Some(document), origin, cx),
    )
}

/// The rejection of a file that is no text.
fn unreadable<F: FileFamily>(origin: Origin) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(malformed());
    F::close(&mut diagnostics, None, &origin.path, true);
    locate(&mut diagnostics, &origin);
    Rejected::new(origin, diagnostics)
}

/// The rest of the load of a parsed file.
fn finish_file<F: FileFamily>(
    parsed: Parsed<F>,
    document: Option<JsonSlice<'_>>,
    origin: Origin,
    cx: &LoadContext,
) -> Result<Document<F::Latest>, Rejected> {
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    if let Some(Loaded::Stable(value)) = outcome.value {
        F::close(&mut diagnostics, document, &origin.path, false);
        locate(&mut diagnostics, &origin);
        Ok(Document::new(value, origin, outcome.version, diagnostics))
    } else {
        F::close(&mut diagnostics, document, &origin.path, true);
        locate(&mut diagnostics, &origin);
        Err(Rejected::new(origin, diagnostics))
    }
}

/// `Root::parse_as_written` of family `F`: the struct the file's `format_version` selects, not
/// upgraded.
///
/// # Errors
///
/// `Rejected` when the text is not readable JSON or the file does not parse into a value; it
/// carries the located findings.
pub fn as_written<F: FileFamily>(
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<F::Any>, Rejected> {
    // The BOM is blanked, not cut, so every offset is an offset into the file as written.
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = Origin::new(PackRef::app_data(), side, path.clone(), None, Vec::new());
    let Some(text) = text(bytes) else {
        return Err(unreadable::<F>(origin));
    };
    let document = JsonSlice::new(text);
    let parsed = parse_as_written::<F>(document, cx);
    let mut diagnostics = parsed.diagnostics;
    let failed = parsed.value.is_none();
    F::close(&mut diagnostics, Some(document), path, failed);
    locate(&mut diagnostics, &origin);
    match parsed.value {
        Some(value) => Ok(Document::new(value, origin, parsed.version, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

/// `history::upgrade` of family `F`: rebuilds the parse stage from the document and finishes
/// it. `at` is the payload's path (the root for a family read whole).
///
/// # Errors
///
/// `Rejected` when the upgrade or the family's checks leave no value; it carries the located
/// findings.
pub fn upgrade<F: FileFamily>(
    document: Document<F::Any>,
    at: JsonPath,
    cx: &LoadContext,
) -> Result<Document<F::Latest>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let value = document.into_value();
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<F>::new(
        Some(value),
        version,
        written,
        Identity::default(),
        ActorType::Mob,
        false,
        at,
        diagnostics,
    );
    finish_file::<F>(parsed, None, origin, cx)
}

/// The identity of a payload whose identifier is its own member `key` (not in a
/// `description`).
#[must_use]
pub fn identity(payload: JsonSlice<'_>, key: &str) -> Identity {
    let identifier = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text)
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
#[must_use]
pub fn last_wins(text: &str) -> Option<Value> {
    bedrock_addon_core::json::from_str_jsonc::<Value>(&bedrock_addon_core::json::last_wins(text))
        .ok()
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
        Some(bedrock_addon_core::json::to_json(payload).ok()?)
    };
    let read = match &rewritten {
        Some(text) => parse_at::<T>(JsonSlice::new(text)),
        None => parse_at::<T>(slice),
    };
    let failure = match read {
        Ok(value) => return Some(value),
        Err(failure) => failure,
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
    let mut path = at.clone();
    path.append(place);
    let mut finding = Diagnostic::new(
        DiagnosticCode::ValueRewritten { text: None },
        Severity::Info,
        Phase::Parse,
        Location::new(None, None, path, Some(span_in(checked, place))),
    );
    finding.detail = Some(Box::new(
        bedrock_addon_core::diagnostic::DiagnosticDetail::Note(shape::SCALAR_READ_AS_BOUND_TYPE),
    ));
    finding
}

/// The finding of a failed typed parse: `code` at `base` followed by the failing member.
fn typed_failure(
    code: DiagnosticCode,
    base: &JsonPath,
    member: &JsonPath,
    span: Span,
) -> Diagnostic {
    let mut path = base.clone();
    path.append(member);
    let mut finding = shape::error(code, path);
    finding.location.span = Some(span);
    finding
}

/// The place in the file of the value at `path` below `slice`: the last value of each member
/// on the way (the one that stands). When a step cannot be taken (a missing member, a key the
/// path does not spell unambiguously), the span is the deepest value reached.
fn span_in(slice: JsonSlice<'_>, path: &JsonPath) -> Span {
    use bedrock_addon_core::diagnostic::JsonPathSegment;
    use facet_json::RawJson;
    use indexmap::IndexMap;
    let mut current = slice;
    // The path's own segments, not its `Display` split on `/` (a key may hold one).
    for step in path.segments() {
        let next = match (first_token(current.text), step) {
            (Some(b'{'), JsonPathSegment::Key(key)) => {
                bedrock_addon_core::json::from_str_borrowed_jsonc::<IndexMap<String, RawJson<'_>>>(
                    current.text,
                )
                .ok()
                .and_then(|members| current.nested(members.get(key)?))
            }
            (Some(b'['), JsonPathSegment::Index(index)) => {
                bedrock_addon_core::json::from_str_borrowed_jsonc::<Vec<RawJson<'_>>>(current.text)
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

/// The `format_version` a history document was read at: a history struct is written back at
/// it, so a document read and written without upgrading keeps the version its author wrote.
/// Not a JSON member of the payload.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
pub struct SourceVersion {
    /// The major part.
    pub major: u16,
    /// The minor part.
    pub minor: u16,
    /// The patch part.
    pub patch: u16,
}

impl SourceVersion {
    /// The version as a [`FormatVersion`](bedrock_addon_core::version::FormatVersion).
    #[must_use]
    pub fn version(self) -> bedrock_addon_core::version::FormatVersion {
        self.into()
    }
}

impl From<SourceVersion> for bedrock_addon_core::version::FormatVersion {
    fn from(version: SourceVersion) -> Self {
        Self::new(version.major, version.minor, version.patch)
    }
}

impl From<bedrock_addon_core::version::FormatVersion> for SourceVersion {
    fn from(version: bedrock_addon_core::version::FormatVersion) -> Self {
        Self {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
        }
    }
}

/// `{"format_version": version, key: payload}`: the whole document of a family read whole.
#[must_use]
pub fn document(version: &str, key: &str, payload: Value) -> Value {
    let mut root = facet_value::VObject::new();
    root.insert("format_version", version);
    root.insert(key, payload);
    root.into()
}

#[cfg(test)]
mod source_version_tests {
    use super::SourceVersion;
    use bedrock_addon_core::version::FormatVersion;

    #[test]
    fn source_version_converts_to_a_format_version_and_back() {
        let source = SourceVersion {
            major: 1,
            minor: 21,
            patch: 0,
        };
        let version = FormatVersion::from(source);
        assert_eq!(version, FormatVersion::new(1, 21, 0));
        assert_eq!(SourceVersion::from(version), source);
        assert_eq!(source.version(), version);
    }
}
