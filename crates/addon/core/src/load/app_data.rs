//! App data outside packs, in memory.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use bytes::Bytes;

use crate::pack_identity::PackPath;

/// App data outside packs (`definitions/attachables`, `features`, `feature_rules`,
/// `spawn_groups`), in memory.
///
/// Vanilla reads these from the application's own data folder, not through a pack; the
/// families that use them decide where the layer sits relative to the packs (attachables
/// before every pack, spawn groups only from here). Cheap to clone: the files are shared.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct AppData {
    /// Path → bytes, ordered bytewise by path.
    files: Arc<BTreeMap<PackPath, Bytes>>,
}

impl AppData {
    /// The directories read, relative to the app-data root.
    // One directory for each family that reads app data (attachable, feature, feature_rules,
    // spawn_groups), all four under the app's `definitions/` folder.
    pub const DIRECTORIES: &'static [&'static str] = &[
        "definitions/attachables",
        "definitions/features",
        "definitions/feature_rules",
        "definitions/spawn_groups",
    ];

    /// The single files read, relative to the app-data root: the game's own credits, which no
    /// pack supplies.
    // The credits are read from the app's own files, not from a pack.
    pub const FILES: &'static [&'static str] = &["credits/credits.json"];

    /// App data from files already in memory, paths relative to the app-data root. A path
    /// given twice keeps its last copy.
    pub fn from_files<B: Into<Bytes>>(files: impl IntoIterator<Item = (PackPath, B)>) -> AppData {
        let files = files
            .into_iter()
            .map(|(path, bytes)| (path, bytes.into()))
            .collect();
        AppData {
            files: Arc::new(files),
        }
    }

    /// The file at `path`, if present.
    #[doc(hidden)]
    #[must_use]
    pub fn get(&self, path: &PackPath) -> Option<&Bytes> {
        self.files.get(path)
    }

    /// Every file, ordered bytewise by path.
    #[doc(hidden)]
    pub fn iter(&self) -> impl Iterator<Item = (&PackPath, &Bytes)> + '_ {
        self.files.iter()
    }

    /// The files at any depth below `directory` (`definitions/spawn_groups`), ordered bytewise
    /// by path.
    #[doc(hidden)]
    pub fn files_under<'a>(
        &'a self,
        directory: &'a str,
    ) -> impl Iterator<Item = (&'a PackPath, &'a Bytes)> + 'a {
        self.files.iter().filter(move |(path, _)| {
            path.as_str()
                .strip_prefix(directory)
                .is_some_and(|rest| rest.starts_with('/'))
        })
    }

    /// How many files.
    #[doc(hidden)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// No files.
    #[doc(hidden)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// The paths only, never the bytes.
impl fmt::Debug for AppData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppData")
            .field("files", &self.files.keys().collect::<Vec<_>>())
            .finish()
    }
}
