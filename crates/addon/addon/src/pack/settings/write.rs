//! Writing a saved-settings document: `format_version` as a string, then
//! `minecraft:pack_settings`.

use facet::Facet;
use indexmap::IndexMap;

use crate::document::Extra;
use crate::json::{EncodeError, Value, WriteStyle, write_json};
use crate::version::FormatVersion;
use crate::versioning::{Family, Versioned, write_payload};

use super::history::PackSettingsFamily;
use super::{SavedSettings, SettingValue, Settings};

impl SavedSettings {
    /// The `format_version` the writer gives the document: 1.21.100, the only version clients
    /// before 1.26.50 read, unless a value is a list of strings, which needs 1.26.40.
    // The game writes its newest document version; the crate writes the floor version unless a
    // value needs the newest one, so older clients can read the file. This is the one deliberate
    // difference from the game's writer.
    pub fn format_version(&self) -> FormatVersion {
        let lists = self.settings.iter().any(|entry| {
            entry
                .values
                .values()
                .any(|value| matches!(value, SettingValue::StringList(_)))
        });
        if lists {
            <Self as Versioned>::VERSION
        } else {
            PackSettingsFamily::FLOOR
        }
    }

    /// Writes the document to `out` in `style`: `format_version` (a string,
    /// [`format_version`](Self::format_version)) and the payload, entries and values in their
    /// order, `last_used` only where an entry has one.
    pub fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        let mut root = facet_value::VObject::new();
        root.insert(
            "format_version",
            Value::from(self.format_version().to_string()),
        );
        root.insert(PackSettingsFamily::PAYLOAD_KEY, payload(&self.settings)?);
        write_json(&Value::from(root), out, style)
    }

    /// The document as JSON text in the default style.
    pub fn to_json(&self) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        self.write_json(&mut out, WriteStyle::default())?;
        // The writer emits only UTF-8.
        Ok(String::from_utf8(out).unwrap_or_default())
    }
}

/// `Document` as written.
#[derive(Facet)]
struct DocumentOut {
    settings: Vec<SettingsOut>,
}

/// `Settings` as written: the UUID in its 36-character form.
#[derive(Facet)]
struct SettingsOut {
    pack_id: String,
    last_used: Option<u64>,
    values: IndexMap<String, ValueOut>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

/// A value as written: a number, a boolean, a string or an array of strings.
#[derive(Facet)]
#[facet(untagged)]
#[allow(dead_code, reason = "read by the writer through facet's reflection")]
#[repr(u8)]
enum ValueOut {
    Float(f32),
    Bool(bool),
    String(String),
    List(Vec<String>),
}

/// The payload `{"settings": [...]}` of `settings`.
pub(super) fn payload(settings: &[Settings]) -> Result<Value, EncodeError> {
    let document = DocumentOut {
        settings: settings.iter().map(entry).collect(),
    };
    write_payload(&document)
}

/// One entry as written.
fn entry(settings: &Settings) -> SettingsOut {
    SettingsOut {
        pack_id: settings.pack_id.hyphenated().to_string(),
        last_used: settings.last_used,
        values: settings
            .values
            .iter()
            .map(|(name, value)| (name.clone(), value_out(value)))
            .collect(),
        extra: settings.extra.clone(),
    }
}

/// One value as written.
fn value_out(value: &SettingValue) -> ValueOut {
    match value {
        SettingValue::Float(value) => ValueOut::Float(*value),
        SettingValue::Bool(value) => ValueOut::Bool(*value),
        SettingValue::String(value) => ValueOut::String(value.clone()),
        SettingValue::StringList(value) => ValueOut::List(value.clone()),
    }
}
