//! The envelope probe of a modern document: `{ "format_version": …, "<payload
//! key>": {…} }`. The root is read into a map of raw slices of the text; `format_version` is read
//! from its slice, and the payload is deserialised from its own slice, never from a `Value`.

use facet_json::RawJson;
use indexmap::IndexMap;

use super::Value;
use super::slice::{JsonSlice, SliceError, parse_at};

/// The root of a modern document as the probe reads it: every member a raw slice of the text,
/// `format_version` included. A shell that typed `format_version` as `Option<Value>` beside a
/// flatten map would read `"format_version": null` as absent; vanilla calls it malformed, so the
/// version is read from its own slice instead. Duplicate keys keep the last value
/// at the first key's position, as the flatten map did.
type EnvelopeShell<'a> = IndexMap<String, RawJson<'a>>;

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
pub struct Envelope<'a> {
    /// The `format_version` member.
    pub format_version: EnvelopeVersion,
    /// The other root members in document order, each a slice of the text with its file offset.
    pub members: IndexMap<String, JsonSlice<'a>>,
}

impl<'a> Envelope<'a> {
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

/// Reads a modern document's root from `document` (the file text after `strip_bom` and
/// `check_depth`). A syntax error, or a root that is not an object, is an error at its file
/// offset ("Failed to parse the Minecraft Document JSON: {}").
///
/// # Errors
///
/// Returns `SliceError` when `document` is not valid JSON or its root is not an object.
pub fn probe_envelope(document: JsonSlice<'_>) -> Result<Envelope<'_>, SliceError> {
    let mut shell = parse_at::<EnvelopeShell<'_>>(document)?;
    let format_version = match shell.shift_remove(FORMAT_VERSION) {
        None => EnvelopeVersion::Absent,
        Some(raw) => version_of(crate::json::from_str_jsonc::<Value>(raw.as_str()).ok()),
    };
    let members = shell
        .iter()
        .filter_map(|(key, raw)| Some((key.clone(), document.nested(raw)?)))
        .collect();
    Ok(Envelope {
        format_version,
        members,
    })
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
