//! Reading saved pack settings from disk.

use std::path::Path;

use bedrock_addon::document::Document;
use bedrock_addon::pack::settings::SavedSettings;

use crate::ReadError;
use crate::read::read_optional;

/// Reads a saved pack-settings file (a world's `world_behavior_pack_settings.json`, or the
/// game's `global_resource_pack_settings.json`) and parses it with the pure
/// `SavedSettings::parse_named`, which names the file by `path` in its findings.
///
/// A file that does not exist holds no values: the result is the empty document, as vanilla
/// uses every default without a message. A file that exists but cannot be read is
/// `ReadError::Io`. A file that is rejected (`ReadError::Rejected`) holds values the caller
/// could not see: every pack then uses its defaults, and a save must not replace the file
/// (`FileState::Failed`). The warnings and infos of a file that loads are not returned;
/// [`read_saved_settings_document`] returns them.
///
/// # Errors
///
/// `ReadError::Io` when the file exists but cannot be read, and `ReadError::Rejected` when
/// it is rejected.
pub fn read_saved_settings(path: impl AsRef<Path>) -> Result<SavedSettings, ReadError> {
    Ok(read_saved_settings_document(path)?
        .map_or_else(SavedSettings::default, Document::into_value))
}

/// [`read_saved_settings`] with what the file said while it loaded: the document carries its
/// warnings and infos (a `pack_id` that is no UUID, a second entry for a pack, a number beyond
/// `f32`, a version vanilla does not read), its origin and its version information.
///
/// `None` is a file that does not exist, which holds no values and has nothing to report;
/// [`read_saved_settings`] gives the empty document for it. The errors are those of
/// [`read_saved_settings`].
///
/// # Errors
///
/// `ReadError::Io` when the file exists but cannot be read, and `ReadError::Rejected` when
/// it is rejected.
pub fn read_saved_settings_document(
    path: impl AsRef<Path>,
) -> Result<Option<Document<SavedSettings>>, ReadError> {
    let path = path.as_ref();
    let Some(bytes) = read_optional(path)? else {
        return Ok(None);
    };
    let name = path.display().to_string();
    Ok(Some(SavedSettings::parse_named(&name, &bytes)?))
}
