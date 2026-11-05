//! The saved-settings types: the document, one pack's entry, a value, and which file holds it.

use indexmap::IndexMap;
use uuid::Uuid;

use crate::document::Extra;
use crate::pack::PackType;

use super::{GLOBAL_FILE, WORLD_FILE};

/// The saved values of pack settings: one `world_behavior_pack_settings.json` or
/// `global_resource_pack_settings.json`, at the newest format (1.26.40).
///
/// JSON: `{"format_version": "1.21.100", "minecraft:pack_settings": {"settings": [{"pack_id",
/// "last_used"?, "values"}]}}`. The file holds only the values that differ from their
/// definition's default. Unknown members, wrong types and a version other than 1.21.100 to
/// 1.26.40 reject the whole file, and every pack in it falls back to its defaults.
///
/// Hand-written read, no `JsonDocument`: the file is in no pack (no `Content`), it is read by
/// the caller's name, which its closing line quotes, and a foreign payload key is renamed in
/// the text before the chain parses it.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:pack_settings")]
#[doc(alias = "world_behavior_pack_settings.json")]
#[doc(alias = "global_resource_pack_settings.json")]
pub struct SavedSettings {
    /// One entry per pack (JSON `settings`). The first entry of a pack UUID is the one read and
    /// the one a save updates; a later one with the same UUID is kept as it is.
    pub settings: Vec<Settings>,
}

/// The saved values of one pack: one element of `settings`.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// The pack's header UUID (JSON `pack_id`). A string that is no UUID reads as the nil UUID,
    /// which matches no pack.
    pub pack_id: Uuid,
    /// When the pack's settings were last used (JSON `last_used`): milliseconds of the
    /// writing client's monotonic clock, comparable only within one session. Only the global
    /// file carries it; the oldest entries are dropped first when it holds too many.
    pub last_used: Option<u64>,
    /// The values by setting name (JSON `values`): the `name` of a slider, toggle, dropdown
    /// or multiselect definition in the pack's manifest.
    pub values: IndexMap<String, SettingValue>,
    /// Unknown members. Always empty after a load: an unknown member rejects the file.
    pub extra: Extra,
}

/// One saved setting value: what a slider, toggle, dropdown or multiselect holds.
///
/// JSON: a number, a boolean, a string or an array of strings. An integer reads as a float.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum SettingValue {
    /// A slider's value (JSON number).
    Float(f32),
    /// A toggle's value (JSON boolean).
    Bool(bool),
    /// A dropdown's value: the `name` of an option (JSON string).
    String(String),
    /// A multiselect's value: the `name`s of the chosen options (JSON array of strings; format
    /// 1.26.40, read by the game from 1.26.50).
    StringList(Vec<String>),
}

/// Which of the two settings files holds a pack's values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingsFile {
    /// `world_behavior_pack_settings.json`: behaviour packs, per world, without `last_used`.
    World,
    /// `global_resource_pack_settings.json`: resource packs, shared by every world, with
    /// `last_used` and at most [`MAX_GLOBAL_ENTRIES`](super::MAX_GLOBAL_ENTRIES) entries.
    Global,
}

impl SettingsFile {
    /// The file's name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::World => WORLD_FILE,
            Self::Global => GLOBAL_FILE,
        }
    }

    /// The file a pack of `pack_type` keeps its values in: behaviour packs the world file,
    /// resource packs the global file; other packs have no saved settings.
    pub fn for_pack_type(pack_type: PackType) -> Option<Self> {
        match pack_type {
            PackType::Behavior => Some(Self::World),
            PackType::Resource => Some(Self::Global),
            _ => None,
        }
    }
}

impl From<f32> for SettingValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<bool> for SettingValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<String> for SettingValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for SettingValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Vec<String>> for SettingValue {
    fn from(value: Vec<String>) -> Self {
        Self::StringList(value)
    }
}

impl Settings {
    /// An entry for `pack_id` with `values` and no `last_used`.
    pub fn new(pack_id: Uuid, values: IndexMap<String, SettingValue>) -> Self {
        Self {
            pack_id,
            last_used: None,
            values,
            extra: Extra::default(),
        }
    }
}

impl SavedSettings {
    /// The first entry for `pack_id`: the one vanilla reads and updates.
    pub fn get(&self, pack_id: Uuid) -> Option<&Settings> {
        self.settings.iter().find(|entry| entry.pack_id == pack_id)
    }
}
