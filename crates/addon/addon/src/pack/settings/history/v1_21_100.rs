//! Format 1.21.100: the first saved-settings document, whose values are numbers, booleans and
//! strings only. Its step is a plain copy into the 1.26.40 document.

use indexmap::IndexMap;
use uuid::Uuid;

use crate::diagnostic::Diagnostics;
use crate::document::Extra;
use crate::json::{EncodeError, JsonSlice, Value};
use crate::version::FormatVersion;
use crate::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

use super::super::{read, write};

/// A saved-settings document of `[1.21.100, 1.26.40)`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SavedSettings {
    /// One entry per pack (JSON `settings`).
    pub settings: Vec<Settings>,
}

/// The saved values of one pack.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// The pack's header UUID (JSON `pack_id`); a string that is no UUID reads as the nil UUID.
    pub pack_id: Uuid,
    /// When the pack's settings were last used (JSON `last_used`), client clock milliseconds.
    pub last_used: Option<u64>,
    /// The values by setting name (JSON `values`).
    pub values: IndexMap<String, SettingValue>,
    /// Unknown members. Always empty after a load: an unknown member rejects the file.
    pub extra: Extra,
}

/// One saved value of format 1.21.100: a number, a boolean or a string (no arrays yet).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum SettingValue {
    /// A slider's value (JSON number).
    Float(f32),
    /// A toggle's value (JSON boolean).
    Bool(bool),
    /// A dropdown's value (JSON string).
    String(String),
}

impl Versioned for SavedSettings {
    const VERSION: FormatVersion = FormatVersion::new(1, 21, 100);
}

impl VersionStruct for SavedSettings {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let settings = read::read(payload, cx, d, read::Grammar::Scalars)?;
        Some(Self {
            settings: settings.into_iter().map(Settings::from_latest).collect(),
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let latest: Vec<_> = self
            .settings
            .iter()
            .cloned()
            .map(Settings::upgrade)
            .collect();
        write::payload(&latest)
    }
}

impl Settings {
    /// The entry as the 1.26.40 struct holds it.
    fn upgrade(self) -> super::super::Settings {
        super::super::Settings {
            pack_id: self.pack_id,
            last_used: self.last_used,
            values: self
                .values
                .into_iter()
                .map(|(name, value)| (name, value.upgrade()))
                .collect(),
            extra: self.extra,
        }
    }

    /// An entry the shared reader gave; it reads no array at this version.
    fn from_latest(entry: super::super::Settings) -> Self {
        Self {
            pack_id: entry.pack_id,
            last_used: entry.last_used,
            values: entry
                .values
                .into_iter()
                .filter_map(|(name, value)| Some((name, SettingValue::from_latest(value)?)))
                .collect(),
            extra: entry.extra,
        }
    }
}

impl SettingValue {
    /// The same value in the widened 1.26.40 type.
    fn upgrade(self) -> super::super::SettingValue {
        match self {
            Self::Float(value) => super::super::SettingValue::Float(value),
            Self::Bool(value) => super::super::SettingValue::Bool(value),
            Self::String(value) => super::super::SettingValue::String(value),
        }
    }

    /// The value, unless it is an array (which the reader refuses at this version).
    fn from_latest(value: super::super::SettingValue) -> Option<Self> {
        match value {
            super::super::SettingValue::Float(value) => Some(Self::Float(value)),
            super::super::SettingValue::Bool(value) => Some(Self::Bool(value)),
            super::super::SettingValue::String(value) => Some(Self::String(value)),
            super::super::SettingValue::StringList(_) => None,
        }
    }
}

/// The step to format 1.26.40: every member moves unchanged; each value keeps its kind and the
/// names keep their order. No finding.
// The upgrade step copies every member and keeps each value's alternative.
impl Upgrade for SavedSettings {
    type Next = super::super::SavedSettings;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        super::super::SavedSettings {
            settings: self.settings.into_iter().map(Settings::upgrade).collect(),
        }
    }
}
