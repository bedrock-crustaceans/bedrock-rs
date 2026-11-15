//! The envelope probe of a modern document: `{ "format_version": …, "<payload
//! key>": {…} }`. The root is parsed once; every member is kept as the slice of the text it
//! spans, `format_version` is read from its slice, and the payload is read from its own slice.
//! Of a key written twice the last value stands at the first key's place.

use indexmap::IndexMap;

use super::Value;
use super::lex::{Kind, Lexer, Rules};
use super::parse::{Fault, Options, SpanKind, parse_slice};
use super::slice::JsonSlice;
use crate::diagnostic::Span;

/// The member that holds the version.
const FORMAT_VERSION: &str = "format_version";

/// What the envelope's `format_version` member holds, by JSON form; the version module turns a
/// string or an array into a `FormatVersion`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EnvelopeVersion {
    /// No `format_version` member.
    Absent,
    /// A string, read with the game's semantic-version string rules.
    String(String),
    /// An array of exactly three non-negative integer tokens.
    Array([u64; 3]),
    /// Anything else: vanilla's "Malformed format version".
    Malformed,
}

/// A modern document's root as the probe found it.
#[derive(Debug)]
pub struct RootProbe<'a> {
    /// The `format_version` member.
    pub format_version: EnvelopeVersion,
    /// The other root members in document order, each a slice of the text with its file offset.
    pub members: IndexMap<String, JsonSlice<'a>>,
}

impl<'a> RootProbe<'a> {
    /// The number of root members, `format_version` included: vanilla requires exactly two.
    /// A duplicate key counts once, as in jsoncpp's object.
    pub fn member_count(&self) -> usize {
        self.members.len() + usize::from(self.format_version != EnvelopeVersion::Absent)
    }

    /// The member `key` (the family's payload key) as a slice of the text.
    pub fn payload(&self, key: &str) -> Option<JsonSlice<'a>> {
        self.members.get(key).copied()
    }
}

/// Why [`probe_envelope`] read no envelope: what it says and where, at file offsets
/// ("Failed to parse the Minecraft Document JSON: {}" is the game's line around it).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum EnvelopeError {
    /// The text is no JSON.
    #[error(transparent)]
    Syntax(#[from] Fault),
    /// The root is a scalar, or an array that holds something: facet-json's words for a text
    /// read as a map, which takes an array as a list of pairs. At the scalar, or at the `[`.
    #[error("unexpected token: got {got}, expected {expected}")]
    NotAnObject {
        /// Where in the file.
        span: Span,
        /// What the root, or its first element, is: `scalar`, `sequence start`, `struct start`.
        got: &'static str,
        /// What a map was wanted as.
        expected: &'static str,
    },
    /// The root is an empty array: facet-json's type mismatch, over the whole slice.
    #[error("type mismatch: expected IndexMap<String, RawJson>, got array")]
    EmptyArray {
        /// Where in the file: the whole slice.
        span: Span,
    },
}

impl EnvelopeError {
    /// Where in the file.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Syntax(fault) => fault.span,
            Self::NotAnObject { span, .. } | Self::EmptyArray { span } => *span,
        }
    }

    /// What it says, without the place: the words of the game's line.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::Syntax(fault) => fault.kind.to_string(),
            other => other.to_string(),
        }
    }
}

/// Reads a modern document's root from `document` (the file text after `strip_bom` and
/// `check_depth`). A syntax error, or a root that is not an object, is an error at its file
/// offset ("Failed to parse the Minecraft Document JSON: {}").
///
/// # Errors
///
/// Returns `EnvelopeError` when `document` is not valid JSON or its root is not an object.
pub fn probe_envelope(document: JsonSlice<'_>) -> Result<RootProbe<'_>, EnvelopeError> {
    let parsed = parse_slice(document, Options::winner(super::Winner::LastInPlace)).map_err(
        |mut fault| {
            fault.span = slice_fault_span(&fault, document);
            fault
        },
    )?;
    let root = parsed.spans.root();
    match root.kind() {
        SpanKind::Object => {}
        SpanKind::Array => {
            let Some(first) = root.elements().next() else {
                let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
                let span = Span {
                    offset: to_u32(document.offset),
                    len: to_u32(document.text.len()),
                };
                return Err(EnvelopeError::EmptyArray { span });
            };
            let got = match first.kind() {
                SpanKind::Object => "struct start",
                SpanKind::Array => "sequence start",
                SpanKind::Scalar => "scalar",
            };
            let span = Span {
                offset: root.span().offset,
                len: 1,
            };
            return Err(EnvelopeError::NotAnObject {
                span,
                got,
                expected: "ordered field or sequence end for map",
            });
        }
        SpanKind::Scalar => {
            return Err(EnvelopeError::NotAnObject {
                span: root.span(),
                got: "scalar",
                expected: "struct start or sequence start for map",
            });
        }
    }
    let mut format_version = EnvelopeVersion::Absent;
    let mut members = IndexMap::new();
    for member in root.members() {
        let slice = member.value.text();
        if member.key == FORMAT_VERSION {
            let value = parse_slice(slice, Options::default()).ok();
            format_version = version_of(value.map(|parsed| parsed.value));
        } else {
            members.insert(member.key.into_owned(), slice);
        }
    }
    Ok(RootProbe {
        format_version,
        members,
    })
}

/// Where a typed read of `slice` placed `fault`: the fault's own span, but the whole slice for
/// a text that ends where the root value had to begin (a blank text) or where the root object
/// waits for a key (after its `{` or a comma): there the door's read named no member.
pub(crate) fn slice_fault_span(fault: &Fault, slice: JsonSlice<'_>) -> Span {
    let whole = fault.span.len == 0
        && fault.is_end_of_input()
        && (slice.text.trim().is_empty() || awaits_a_root_key(slice.text.as_bytes()));
    if !whole {
        return fault.span;
    }
    let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    Span {
        offset: to_u32(slice.offset),
        len: to_u32(slice.text.len()),
    }
}

/// Whether `bytes` end inside the root object, after its `{` or a comma between its members.
fn awaits_a_root_key(bytes: &[u8]) -> bool {
    let mut lexer = Lexer::new(bytes, Rules::strict(true));
    let (mut depth, mut root_is_object, mut last) = (0usize, false, None);
    while let Ok(Some(token)) = lexer.next_token() {
        match token.kind {
            Kind::OpenObject | Kind::OpenArray => {
                if depth == 0 {
                    root_is_object = token.kind == Kind::OpenObject;
                }
                depth += 1;
            }
            Kind::CloseObject | Kind::CloseArray => depth = depth.saturating_sub(1),
            _ => {}
        }
        last = Some(token.kind);
    }
    root_is_object && depth == 1 && matches!(last, Some(Kind::OpenObject | Kind::Comma))
}

/// The JSON form of a present `format_version`, as the game's versioned loaders read it: a string,
/// or an array of exactly three integers; anything else, `null` included, is malformed.
fn version_of(json: Option<Value>) -> EnvelopeVersion {
    let Some(json) = json else {
        return EnvelopeVersion::Malformed;
    };
    if let Some(text) = json.as_string() {
        return EnvelopeVersion::String(text.as_str().to_owned());
    }
    // The integer tag is jsoncpp's int type; it is not known whether the game accepts a real
    // element there, and the crate accepts integers only.
    let parts: Option<Vec<u64>> = json.as_array().and_then(|array| {
        array
            .iter()
            .map(|part| part.as_number().filter(|n| n.is_integer())?.to_u64())
            .collect()
    });
    match parts.as_deref() {
        Some(&[major, minor, patch]) => EnvelopeVersion::Array([major, minor, patch]),
        _ => EnvelopeVersion::Malformed,
    }
}
