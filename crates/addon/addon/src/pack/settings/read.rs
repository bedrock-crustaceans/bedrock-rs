//! Reading the payload of a saved-settings document (`minecraft:pack_settings`), shared by both
//! structs of the chain.
//!
//! The loader is strict (mask 0xfffc): an unknown member, a missing member and a value of the
//! wrong JSON type each fail the whole file. The payload is walked on the reader kit with its
//! text, so every member's JSON type is checked here (facet would coerce scalars) and every
//! finding keeps its file offset.

use facet_json::RawJson;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
    code_for_deserialize_error_kind,
};
use crate::json::kit::{Disposition, Node, Read};
use crate::json::{EncodeError, JsonSlice, Value, parse_at};
use crate::version::FormatVersion;
use crate::versioning::{ParseCx, Reading, VersionStruct, Versioned};

use super::{SavedSettings, SettingValue, Settings, write};

impl Versioned for SavedSettings {
    // The newest document version, 1.26.40: the loader's end version.
    const VERSION: FormatVersion = FormatVersion::new(1, 26, 40);
}

impl VersionStruct for SavedSettings {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        Some(Self {
            settings: read(payload, cx, d, Grammar::WithLists)?,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write::payload(&self.settings)
    }
}

/// The members of the document: `settings`, required.
const DOCUMENT_MEMBERS: [&str; 1] = ["settings"];
/// The members of an entry: `pack_id` and `values` required, `last_used` optional.
const SETTINGS_MEMBERS: [&str; 3] = ["pack_id", "last_used", "values"];

/// How the loader reads every level: strict (mask 0xfffc), so an unknown member, a missing
/// member and a value of the wrong JSON type are each an error that fails the file.
const READING: Reading = Reading::DOCUMENT;

/// What a setting's value may be in a document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Grammar {
    /// A number, a boolean or a string (before 1.26.40).
    Scalars,
    /// Also an array of strings (1.26.40).
    WithLists,
}

/// Reads the `settings` of `payload` under `grammar`. `None` when anything failed (the
/// reasons are in `d`).
pub(super) fn read(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    grammar: Grammar,
) -> Option<Vec<Settings>> {
    let Ok(json) = crate::json::from_str_jsonc::<Value>(payload.text) else {
        // Not reached for a payload the door has read; the typed read words the failure.
        if let Err(failure) = parse_at::<IndexMap<String, RawJson<'_>>>(payload) {
            slice_error(d, &failure, payload, &cx.at);
        }
        return None;
    };
    let node = Node::with_text(&json, cx.at.clone(), payload);
    let mut reader = Reader {
        grammar,
        read: Read::new(READING, d),
    };
    reader.document(&node)
}

/// The state of one read: the grammar, and the kit's read telling the findings.
struct Reader<'d> {
    grammar: Grammar,
    read: Read<'d>,
}

impl Reader<'_> {
    /// The payload object: `{"settings": [...]}`.
    fn document(&mut self, payload: &Node<'_>) -> Option<Vec<Settings>> {
        let members = self.read.object(payload)?;
        // Told, never kept: the file then fails.
        self.read
            .unknown(&members, &DOCUMENT_MEMBERS, Disposition::Keep);
        let settings = self.read.required(&members, "settings")?;
        // An array is expected here: "expected an array".
        let entries = self.read.array(&settings)?;
        let mut read = Vec::with_capacity(entries.len());
        let mut failed = false;
        for entry in entries.iter() {
            match self.entry(&entry) {
                Some(entry) => read.push(entry),
                None => failed = true,
            }
        }
        (!failed).then_some(read)
    }

    /// One `settings[]` entry.
    fn entry(&mut self, node: &Node<'_>) -> Option<Settings> {
        let members = self.read.object(node)?;
        self.read
            .unknown(&members, &SETTINGS_MEMBERS, Disposition::Keep);
        let pack_id = self.read.required(&members, "pack_id");
        let values = self.read.required(&members, "values");
        let pack_id = self.pack_id(&pack_id?);
        let last_used = match members.get("last_used") {
            Some(node) => Some(self.last_used(&node)?),
            None => None,
        };
        let values = self.values(&values?);
        Some(Settings {
            pack_id: pack_id?,
            last_used,
            values: values?,
        })
    }

    /// `values`: an object of setting values.
    fn values(&mut self, node: &Node<'_>) -> Option<IndexMap<String, SettingValue>> {
        let members = self.read.object(node)?;
        let mut values = IndexMap::with_capacity(members.len());
        let mut failed = false;
        for (name, member) in members.iter() {
            match self.value(&member) {
                Some(value) => {
                    values.insert(name.to_owned(), value);
                }
                None => failed = true,
            }
        }
        (!failed).then_some(values)
    }

    /// `pack_id`: a string, read as a UUID.
    fn pack_id(&mut self, node: &Node<'_>) -> Option<Uuid> {
        let Some(text) = string_token(node) else {
            self.read.wrong_type(node);
            return None;
        };
        // The game's setter cannot report: a string that is no UUID loads silently as the nil
        // UUID. The crate reports an information.
        let id = uuid_from_string(text);
        if id.is_none() {
            self.information(node);
        }
        Some(id.unwrap_or(Uuid::nil()))
    }

    /// `last_used`: an unsigned 64-bit integer (`std::optional<uint64>`).
    fn last_used(&mut self, node: &Node<'_>) -> Option<u64> {
        let value = node
            .value()
            .as_number()
            .filter(|n| n.is_integer())
            .and_then(facet_value::VNumber::to_u64);
        if value.is_none() {
            self.read.wrong_type(node);
        }
        value
    }

    /// One value of `values`: a number, a boolean, a string, or (1.26.40) an array of strings.
    // A value is tried as a float, a bool and a string in turn; anything else is a type error.
    fn value(&mut self, node: &Node<'_>) -> Option<SettingValue> {
        let json = node.value();
        let value = if let Some(number) = json.as_number() {
            Some(self.float(number.to_f64_lossy(), node))
        } else if let Some(flag) = json.as_bool() {
            Some(SettingValue::Bool(flag))
        } else if let Some(text) = json.as_string() {
            // A string value is a string token: an integer beyond 64 bits is a number to the
            // reader, not its digits.
            Some(SettingValue::String(text.as_str().to_owned()))
        } else if json.as_array().is_some() && self.grammar == Grammar::WithLists {
            self.read.array(node).and_then(|items| {
                items
                    .iter()
                    .map(|item| string_token(&item).map(str::to_owned))
                    .collect::<Option<Vec<_>>>()
                    .map(SettingValue::StringList)
            })
        } else {
            None
        };
        if value.is_none() {
            self.read.wrong_type(node);
        }
        value
    }

    /// A number as the `f32` the game reads. One whose nearest `f32` is infinite (`1e39`,
    /// `1e400`, an integer of 400 digits) has no JSON form to write back: it is read as the
    /// largest finite `f32` of its sign, with an information at the value (the crate's
    /// own; a slider clamps either to its own maximum). The game reads the float without a line,
    /// so the note is an information.
    fn float(&mut self, number: f64, node: &Node<'_>) -> SettingValue {
        #[allow(clippy::cast_possible_truncation)] // the game reads a float
        let value = number as f32;
        if value.is_finite() {
            return SettingValue::Float(value);
        }
        self.information(node);
        SettingValue::Float(if number < 0.0 { f32::MIN } else { f32::MAX })
    }

    /// An information at `node`, spanning it: a value the game reads silently as another.
    fn information(&mut self, node: &Node<'_>) {
        let mut finding =
            Diagnostic::info(DiagnosticCode::InvalidValue { text: None }).at(node.path().clone());
        finding.location.span = node.span();
        self.read.push(finding);
    }
}

/// The string `node` holds when its text is a string token (the untyped reader also gives a
/// string for an integer too large for its number type).
fn string_token<'v>(node: &Node<'v>) -> Option<&'v str> {
    let text = node.value().as_string()?.as_str();
    node.text()
        .is_none_or(|slice| is_string_token(slice))
        .then_some(text)
}

/// A failed typed read of `slice` at `at`.
fn slice_error(
    d: &mut Diagnostics,
    failure: &crate::json::SliceError,
    slice: JsonSlice<'_>,
    at: &JsonPath,
) {
    let blank = slice.text.trim().is_empty();
    let code =
        code_for_deserialize_error_kind(&failure.error.kind, !failure.path.is_empty(), blank);
    d.push(Diagnostic::new(
        code,
        Severity::Error,
        Phase::Parse,
        Location::new(None, None, at.join(&failure.path), Some(failure.span)),
    ));
}

/// Whether the slice is a JSON string (the untyped reader also gives a string for an integer too
/// large for its number type).
fn is_string_token(slice: JsonSlice<'_>) -> bool {
    slice.text.trim_start().starts_with('"')
}

/// The UUID a string spells, as the game reads it: 36 characters with `-` at 8, 13, 18 and 23
/// and hex digits elsewhere, or 32 hex digits; either letter case. Anything else is `None`
/// (the game gives the nil UUID without a message).
fn uuid_from_string(text: &str) -> Option<Uuid> {
    matches!(text.len(), 32 | 36)
        .then(|| Uuid::parse_str(text).ok())
        .flatten()
}
