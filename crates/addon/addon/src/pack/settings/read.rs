//! Reading the payload of a saved-settings document (`minecraft:pack_settings`), shared by both
//! structs of the chain.
//!
//! The loader is strict (mask 0xfffc): an unknown member, a missing member and a value of the
//! wrong JSON type each fail the whole file. Each object level is read as a map of raw slices of
//! the text, so every member's JSON type is checked here (facet would coerce scalars) and every
//! finding keeps its file offset.

use facet_json::RawJson;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, Span,
    code_for_deserialize_error_kind,
};
use crate::json::{EncodeError, JsonSlice, Value, parse_at};
use crate::version::FormatVersion;
use crate::versioning::{ParseCx, VersionStruct, Versioned};

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
    let mut reader = Reader { grammar, d };
    reader.document(payload, &cx.at)
}

/// The state of one read: where findings go.
struct Reader<'d> {
    grammar: Grammar,
    d: &'d mut Diagnostics,
}

impl Reader<'_> {
    /// The payload object: `{"settings": [...]}`.
    fn document(&mut self, payload: JsonSlice<'_>, at: &JsonPath) -> Option<Vec<Settings>> {
        let members = self.object(payload, at)?;
        self.members(&members, payload, at, &DOCUMENT_MEMBERS);
        let settings = self.required(&members, payload, at, "settings")?;
        let at_settings = child(at, "settings");
        // An array is expected here: "expected an array".
        if json(settings).is_some_and(|value| value.as_array().is_none()) {
            self.error(
                DiagnosticCode::InvalidType { text: None },
                at_settings,
                settings,
            );
            return None;
        }
        let entries = match parse_at::<Vec<RawJson<'_>>>(settings) {
            Ok(entries) => entries,
            Err(failure) => {
                self.slice_error(&failure, settings, &at_settings);
                return None;
            }
        };
        let mut read = Vec::with_capacity(entries.len());
        let mut failed = false;
        for (index, raw) in entries.iter().enumerate() {
            let at_entry = at_settings.index(index);
            match settings.nested(raw) {
                Some(slice) => match self.entry(slice, &at_entry) {
                    Some(entry) => read.push(entry),
                    None => failed = true,
                },
                None => failed = true,
            }
        }
        (!failed).then_some(read)
    }

    /// One `settings[]` entry.
    fn entry(&mut self, slice: JsonSlice<'_>, at: &JsonPath) -> Option<Settings> {
        let members = self.object(slice, at)?;
        self.members(&members, slice, at, &SETTINGS_MEMBERS);
        let pack_id = self.required(&members, slice, at, "pack_id");
        let values = self.required(&members, slice, at, "values");
        let pack_id = self.pack_id(pack_id?, child(at, "pack_id"));
        let last_used = match members.get("last_used") {
            Some(raw) => {
                let slice = slice.nested(raw)?;
                Some(self.last_used(slice, child(at, "last_used"))?)
            }
            None => None,
        };
        let values = self.values(values?, &child(at, "values"));
        Some(Settings {
            pack_id: pack_id?,
            last_used,
            values: values?,
        })
    }

    /// `values`: an object of setting values.
    fn values(
        &mut self,
        slice: JsonSlice<'_>,
        at: &JsonPath,
    ) -> Option<IndexMap<String, SettingValue>> {
        let members = self.object(slice, at)?;
        let mut values = IndexMap::with_capacity(members.len());
        let mut failed = false;
        for (name, raw) in &members {
            let Some(value) = slice.nested(raw) else {
                failed = true;
                continue;
            };
            match self.value(value, child(at, name)) {
                Some(value) => {
                    values.insert(name.clone(), value);
                }
                None => failed = true,
            }
        }
        (!failed).then_some(values)
    }

    /// `pack_id`: a string, read as a UUID.
    fn pack_id(&mut self, slice: JsonSlice<'_>, at: JsonPath) -> Option<Uuid> {
        let json = json(slice)?;
        let Some(text) = json.as_string().filter(|_| is_string_token(slice)) else {
            self.error(DiagnosticCode::InvalidType { text: None }, at, slice);
            return None;
        };
        // The game's setter cannot report: a string that is no UUID loads silently as the nil
        // UUID. The crate reports an information.
        let id = uuid_from_string(text.as_str());
        if id.is_none() {
            self.d.push(
                Diagnostic::info(DiagnosticCode::InvalidValue { text: None })
                    .at(at)
                    .span(span_of(slice)),
            );
        }
        Some(id.unwrap_or(Uuid::nil()))
    }

    /// `last_used`: an unsigned 64-bit integer (`std::optional<uint64>`).
    fn last_used(&mut self, slice: JsonSlice<'_>, at: JsonPath) -> Option<u64> {
        let value = json(slice)?
            .as_number()
            .filter(|n| n.is_integer())
            .and_then(facet_value::VNumber::to_u64);
        if value.is_none() {
            self.error(DiagnosticCode::InvalidType { text: None }, at, slice);
        }
        value
    }

    /// One value of `values`: a number, a boolean, a string, or (1.26.40) an array of strings.
    // A value is tried as a float, a bool and a string in turn; anything else is a type error.
    fn value(&mut self, slice: JsonSlice<'_>, at: JsonPath) -> Option<SettingValue> {
        let json = json(slice)?;
        let value = if let Some(number) = json.as_number() {
            Some(self.float(number.to_f64_lossy(), &at, slice))
        } else if let Some(flag) = json.as_bool() {
            Some(SettingValue::Bool(flag))
        } else if let Some(text) = json.as_string() {
            // A string value is a string token: an integer beyond 64 bits is a number to the
            // reader, not its digits.
            Some(SettingValue::String(text.as_str().to_owned()))
        } else if json.as_array().is_some() && self.grammar == Grammar::WithLists {
            parse_at::<Vec<RawJson<'_>>>(slice).ok().and_then(|items| {
                items
                    .iter()
                    .map(|raw| {
                        let item = slice.nested(raw).filter(|item| is_string_token(*item))?;
                        Some(self::json(item)?.as_string()?.as_str().to_owned())
                    })
                    .collect::<Option<Vec<_>>>()
                    .map(SettingValue::StringList)
            })
        } else {
            None
        };
        if value.is_none() {
            self.error(DiagnosticCode::InvalidType { text: None }, at, slice);
        }
        value
    }

    /// A number as the `f32` the game reads. One whose nearest `f32` is infinite (`1e39`,
    /// `1e400`, an integer of 400 digits) has no JSON form to write back: it is read as the
    /// largest finite `f32` of its sign, with an information at the value (the crate's
    /// own; a slider clamps either to its own maximum). The game reads the float without a line,
    /// so the note is an information.
    fn float(&mut self, number: f64, at: &JsonPath, slice: JsonSlice<'_>) -> SettingValue {
        #[allow(clippy::cast_possible_truncation)] // the game reads a float
        let value = number as f32;
        if value.is_finite() {
            return SettingValue::Float(value);
        }
        self.d.push(
            Diagnostic::info(DiagnosticCode::InvalidValue { text: None })
                .at(at.clone())
                .span(span_of(slice)),
        );
        SettingValue::Float(if number < 0.0 { f32::MIN } else { f32::MAX })
    }

    /// The members of the object `slice`, or a finding when it is no object.
    fn object<'a>(
        &mut self,
        slice: JsonSlice<'a>,
        at: &JsonPath,
    ) -> Option<IndexMap<String, RawJson<'a>>> {
        // facet reads an empty array as an empty map: the JSON type is checked first.
        if json(slice).is_some_and(|value| value.as_object().is_none()) {
            self.error(
                DiagnosticCode::InvalidType { text: None },
                at.clone(),
                slice,
            );
            return None;
        }
        match parse_at::<IndexMap<String, RawJson<'a>>>(slice) {
            Ok(members) => Some(members),
            Err(failure) => {
                self.slice_error(&failure, slice, at);
                None
            }
        }
    }

    /// Reports every member of `members` not in `known`: `UnrecognizedField`, an error under
    /// the strict mask, so the file is rejected whatever else it holds.
    fn members(
        &mut self,
        members: &IndexMap<String, RawJson<'_>>,
        slice: JsonSlice<'_>,
        at: &JsonPath,
        known: &[&str],
    ) {
        for (key, raw) in members {
            if !known.contains(&key.as_str()) {
                let span = slice.nested(raw).map(span_of);
                self.push(
                    DiagnosticCode::UnrecognizedField { text: None },
                    child(at, key),
                    span,
                );
            }
        }
    }

    /// The required member `key`, or `MissingRequiredField`.
    fn required<'a>(
        &mut self,
        members: &IndexMap<String, RawJson<'a>>,
        slice: JsonSlice<'a>,
        at: &JsonPath,
        key: &str,
    ) -> Option<JsonSlice<'a>> {
        let found = members.get(key).and_then(|raw| slice.nested(raw));
        if found.is_none() {
            self.push(
                DiagnosticCode::MissingRequiredField { text: None },
                child(at, key),
                Some(span_of(slice)),
            );
        }
        found
    }

    /// A failed typed read of `slice` at `at`.
    fn slice_error(
        &mut self,
        failure: &crate::json::SliceError,
        slice: JsonSlice<'_>,
        at: &JsonPath,
    ) {
        let blank = slice.text.trim().is_empty();
        let code =
            code_for_deserialize_error_kind(&failure.error.kind, !failure.path.is_empty(), blank);
        let path = at.join(&failure.path);
        self.push(code, path, Some(failure.span));
    }

    /// An error spanning `slice`.
    fn error(&mut self, code: DiagnosticCode, at: JsonPath, slice: JsonSlice<'_>) {
        self.push(code, at, Some(span_of(slice)));
    }

    /// An error-level parse finding.
    fn push(&mut self, code: DiagnosticCode, at: JsonPath, span: Option<Span>) {
        self.d.push(Diagnostic::new(
            code,
            Severity::Error,
            Phase::Parse,
            Location::new(None, None, at, span),
        ));
    }
}

/// Whether the slice is a JSON string (the untyped reader also gives a string for an integer too
/// large for its number type).
fn is_string_token(slice: JsonSlice<'_>) -> bool {
    slice.text.trim_start().starts_with('"')
}

/// The untyped value of a slice the map reader already read (valid JSON).
fn json(slice: JsonSlice<'_>) -> Option<Value> {
    crate::json::from_str_jsonc::<Value>(slice.text).ok()
}

/// `at` followed by the member `key`.
fn child(at: &JsonPath, key: &str) -> JsonPath {
    at.child(key)
}

/// The UUID a string spells, as the game reads it: 36 characters with `-` at 8, 13, 18 and 23
/// and hex digits elsewhere, or 32 hex digits; either letter case. Anything else is `None`
/// (the game gives the nil UUID without a message).
fn uuid_from_string(text: &str) -> Option<Uuid> {
    matches!(text.len(), 32 | 36)
        .then(|| Uuid::parse_str(text).ok())
        .flatten()
}

/// The span of a slice in its file.
fn span_of(slice: JsonSlice<'_>) -> Span {
    let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    Span {
        offset: to_u32(slice.offset),
        len: to_u32(slice.text.len()),
    }
}
