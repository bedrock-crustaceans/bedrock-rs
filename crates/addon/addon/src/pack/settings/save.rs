//! Vanilla's save algorithm over saved-settings documents: the non-default values of each pack
//! replace its entry, the global file keeps its most recently used entries, and a file is
//! written only when it changed.

use std::cmp::Reverse;

use super::{MAX_GLOBAL_ENTRIES, PackSettings, SavedSettings, Settings, SettingsFile};

/// What a settings file held when it was read: what a save compares its result with.
#[derive(Clone, Debug, PartialEq)]
pub enum FileState {
    /// There is no file.
    Missing,
    /// The file loaded.
    Read(SavedSettings),
    /// The file exists and failed to load.
    Failed,
}

/// Why a save does not write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SaveRefused {
    /// The file failed to load, and writing would replace the values it holds with the
    /// current ones alone.
    #[error("the settings file failed to load; it is not overwritten")]
    FileFailedToLoad,
}

impl SavedSettings {
    /// Writes the non-default values of `settings` into the first entry of its pack: the entry's
    /// values are replaced by them (sorted by name); an entry is appended when the pack has
    /// none; and the entry is removed when every value is at its default.
    ///
    /// A world-file entry never carries `last_used`. A global-file entry keeps the `last_used`
    /// it has, and a new one gets none, so it is the first to go when the file is pruned.
    // The game stamps a global entry with the client's steady-clock milliseconds on every
    // save; the crate writes no `last_used` for a new entry, so it is the first to go when the
    // file is pruned.
    pub fn sync(&mut self, settings: &PackSettings, file: SettingsFile) {
        let changed = settings.changed();
        let found = self
            .settings
            .iter()
            .position(|entry| entry.pack_id == settings.pack_id());
        match (found, changed.is_empty()) {
            (Some(index), false) => {
                let entry = &mut self.settings[index];
                entry.values = changed;
                if file == SettingsFile::World {
                    entry.last_used = None;
                }
            }
            (Some(index), true) => {
                self.settings.remove(index);
            }
            (None, false) => self
                .settings
                .push(Settings::new(settings.pack_id(), changed)),
            (None, true) => {}
        }
    }

    /// Keeps the `max` most recently used entries when there are more: the entries are ordered
    /// by `last_used`, newest first, those without one last, and the rest is cut off. Entries
    /// with equal `last_used` keep their order in the file.
    // The comparator is "has a value and (the other has none or this is greater)"; the game's
    // sort leaves the order of ties unspecified, the crate keeps the file's order.
    pub fn prune(&mut self, max: usize) {
        if self.settings.len() <= max {
            return;
        }
        self.settings.sort_by_key(|entry| Reverse(entry.last_used));
        self.settings.truncate(max);
    }

    /// The document a save of `file` writes, or `None` when nothing is to be written.
    ///
    /// The document `previous` held (an empty one when there was no file or it failed to load)
    /// takes the values of every pack in `packs` ([`sync`](Self::sync)); the global file is
    /// then cut to [`MAX_GLOBAL_ENTRIES`] ([`prune`](Self::prune)). The result is written only
    /// when it differs from what `previous` held, so with no file and no changed value none
    /// is created, and returning every value to its default leaves a file with no entries.
    /// The caller passes the packs of the file's kind (`SettingsFile::for_pack_type`).
    ///
    /// A file that failed to load is never replaced: when the save would write,
    /// it is refused instead.
    // The game compares against an empty document when the load failed and overwrites the file
    // as soon as one value differs from its default; the crate refuses instead.
    pub fn save<'a>(
        file: SettingsFile,
        previous: &FileState,
        packs: impl IntoIterator<Item = &'a PackSettings>,
    ) -> Result<Option<SavedSettings>, SaveRefused> {
        let empty = SavedSettings::default();
        let loaded = match previous {
            FileState::Read(document) => document,
            FileState::Missing | FileState::Failed => &empty,
        };
        let mut document = loaded.clone();
        for settings in packs {
            document.sync(settings, file);
        }
        if file == SettingsFile::Global {
            document.prune(MAX_GLOBAL_ENTRIES);
        }
        if document == *loaded {
            return Ok(None);
        }
        if *previous == FileState::Failed {
            return Err(SaveRefused::FileFailedToLoad);
        }
        Ok(Some(document))
    }
}
