//! `blocks.json` as a file: the one shape the client reads at every `format_version`.

use indexmap::IndexMap;

use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;

use super::super::version;

/// A resource pack's `blocks.json`: `format_version` and one entry per block name.
///
/// JSON: `{"format_version": "1.21.40", "<block name>": {…}, …}`. The members are held as
/// written, so the file writes back what it read; [`super::super::ClientBlock`] is what the
/// client reads of an entry. `format_version` only decides how legacy block names (`wool`,
/// `log`, `stone`, …) are read: [`effective_version`](Self::effective_version) is what the
/// client makes of it.
///
/// `format_version` is written first, as vanilla's files have it, whatever its place in the
/// file read: the client reads the members by key, so the place changes nothing it reads. Two
/// files are equal when they hold the same `format_version` and the same entries (compared as a
/// map, whatever their order).
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "blocks.json")]
pub struct BlocksJson {
    /// `format_version` as written: a version string or an array of three numbers; `None`
    /// when the file has none.
    pub format_version: Option<Value>,
    /// Every other member, in file order: the block name as written and its value. The client
    /// skips a value that is not an object.
    pub entries: IndexMap<String, Value>,
}

impl Default for BlocksJson {
    /// The same as [`BlocksJson::new`]: an empty file at [`BlocksJson::FORMAT_VERSION`].
    fn default() -> Self {
        Self::new()
    }
}

impl BlocksJson {
    /// The version every flattened block name is expected at and no legacy name is split at:
    /// what a new file should carry.
    pub const FORMAT_VERSION: FormatVersion = version::BLOCKS_JSON_MAX_FORMAT_VERSION;

    /// An empty file at [`FORMAT_VERSION`](Self::FORMAT_VERSION).
    #[must_use]
    pub fn new() -> Self {
        Self {
            format_version: Some(Value::from(Self::FORMAT_VERSION.to_string().as_str())),
            entries: IndexMap::new(),
        }
    }

    /// The version the client reads the file as: 1.1.0 when `format_version` is missing, not a
    /// version, or a whole version above 1.21.40; the version as written otherwise, a partial
    /// one (`"1.21"`, `"2"`) included, with 0 for its missing parts (`"*"` reads as 0.0.0).
    #[must_use]
    pub fn effective_version(&self) -> FormatVersion {
        version::effective(self.format_version.as_ref())
            .effective
            .version
    }
}
