//! `blocks.json` as a file: the one shape the client reads at every `format_version`.

use indexmap::IndexMap;

use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;

use super::super::version;

/// A resource pack's `blocks.json`: `format_version` and one entry per block name.
///
/// JSON: `{"format_version": "1.21.40", "<block name>": {…}, …}`. The members are held as
/// written, so the file writes back unchanged; [`super::super::ClientBlock`] is what the
/// client reads of an entry. `format_version` only decides how legacy block names (`wool`,
/// `log`, `stone`, …) are read: [`effective_version`](Self::effective_version) is what the
/// client makes of it.
///
/// Two files are equal when they hold the same `format_version`, write it at the same place
/// and hold the same entries (compared as a map, whatever their order).
#[derive(Clone, Debug)]
#[doc(alias = "blocks.json")]
pub struct BlocksJson {
    /// `format_version` as written: a version string or an array of three numbers; `None`
    /// when the file has none.
    pub format_version: Option<Value>,
    /// How many of [`entries`](Self::entries) stand before `format_version` in the file: 0 for
    /// a version that is the first member, as vanilla's files have it. A number beyond the
    /// entries writes the version last; without a `format_version` it says nothing. The
    /// client reads the members in the order of their keys, so the place changes nothing it
    /// reads. Two files compare it as the place it writes: every number beyond the entries is
    /// the same place, and without a `format_version` it is not compared.
    pub format_version_index: usize,
    /// Every other member, in file order: the block name as written and its value. The client
    /// skips a value that is not an object.
    pub entries: IndexMap<String, Value>,
}

/// `format_version_index` is compared as the place the version is written at, so that a file
/// built by hand equals what its own text reads back to.
impl PartialEq for BlocksJson {
    fn eq(&self, other: &Self) -> bool {
        self.format_version == other.format_version
            && self.entries == other.entries
            && self.version_place() == other.version_place()
    }
}

impl Default for BlocksJson {
    /// The same as [`BlocksJson::new`]: an empty file at [`BlocksJson::FORMAT_VERSION`].
    fn default() -> Self {
        Self::new()
    }
}

impl BlocksJson {
    /// Where `format_version` is written among the entries; `None` without one.
    fn version_place(&self) -> Option<usize> {
        self.format_version
            .as_ref()
            .map(|_| self.format_version_index.min(self.entries.len()))
    }

    /// The version every flattened block name is expected at and no legacy name is split at:
    /// what a new file should carry.
    pub const FORMAT_VERSION: FormatVersion = version::BLOCKS_JSON_MAX_FORMAT_VERSION;

    /// An empty file at [`FORMAT_VERSION`](Self::FORMAT_VERSION).
    #[must_use]
    pub fn new() -> Self {
        Self {
            format_version: Some(Value::from(Self::FORMAT_VERSION.to_string().as_str())),
            format_version_index: 0,
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
