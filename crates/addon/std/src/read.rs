//! The server-side files around packs, read with `std::fs` and parsed by the pure functions of
//! `bedrock_addon`.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use bedrock_addon::load::{AppData, EntryKind, ListDepth};
use bedrock_addon::pack::WorldPacks;
use bedrock_addon::storage::PackPath;

use crate::{HostIoError, HostOp, ReadError, io as host};

/// Reads a world's `world_behavior_packs.json` and `world_resource_packs.json` and parses them
/// with the pure `WorldPacks::parse`. A missing file is an empty list, as in vanilla; a file
/// that exists but cannot be read is `ReadError::Io`, one that is rejected `ReadError::Rejected`.
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use bedrock_addon_std::read_world_packs;
///
/// let world = read_world_packs("worlds/Bedrock level")?;
/// # let _ = world;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// `ReadError::Io` when a `world_*_packs.json` exists but cannot be read, and
/// `ReadError::Rejected` when one is read but rejected.
pub fn read_world_packs(world_dir: impl AsRef<Path>) -> Result<WorldPacks, ReadError> {
    let world_dir = world_dir.as_ref();
    let behavior = read_optional(&world_dir.join(WorldPacks::BEHAVIOR_FILE))?;
    let resource = read_optional(&world_dir.join(WorldPacks::RESOURCE_FILE))?;
    Ok(WorldPacks::parse(behavior.as_deref(), resource.as_deref())?)
}

/// Reads the app-data directories (`definitions/attachables`, `definitions/features`,
/// `definitions/feature_rules`, `definitions/spawn_groups`) and the game's own
/// `credits/credits.json` ([`AppData::FILES`]) into an [`AppData`].
///
/// `dir` is the app-data root: the folder holding `definitions/` (a BDS install, a client's
/// `data/`). Every file at any depth below each of [`AppData::DIRECTORIES`] is read, following
/// symlinks, with its path relative to `dir`; the families decide which of them they parse. A
/// directory or single file that does not exist is skipped.
///
/// # Errors
///
/// `ReadError::Io` when `dir` itself is missing or cannot be looked up, and when a directory
/// or file cannot be listed or read. `ReadError::NotADirectory` when `dir` is not a
/// directory. `ReadError::NotUtf8` when a file's name is not valid UTF-8, and
/// `ReadError::NotAPackPath` when it is valid UTF-8 but breaks a pack-path rule (a `:`, say).
pub fn read_app_data(dir: impl AsRef<Path>) -> Result<AppData, ReadError> {
    let root = dir.as_ref();
    match fs::metadata(root) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => return Err(ReadError::NotADirectory { path: root.into() }),
        Err(source) => return Err(HostIoError::new(HostOp::Metadata, root, source).into()),
    }
    let mut files = Vec::new();
    for directory in AppData::DIRECTORIES {
        let directory = PackPath::from_static(directory);
        let base = host::host_path(root, &directory);
        let entries = match host::walk(&base, ListDepth::Recursive) {
            Ok(entries) => entries,
            Err(source) if source.kind() == ErrorKind::NotFound => continue,
            Err(source) => return Err(HostIoError::new(HostOp::List, &base, source).into()),
        };
        for (path, entry) in entries {
            if !matches!(entry.kind, EntryKind::File { .. }) {
                continue;
            }
            if entry.lossy {
                return Err(ReadError::NotUtf8 { path });
            }
            let pack_path = match PackPath::new(&entry.name) {
                Ok(name) => directory.join(&name),
                Err(source) => return Err(ReadError::NotAPackPath { path, source }),
            };
            let bytes =
                fs::read(&path).map_err(|source| HostIoError::new(HostOp::Read, &path, source))?;
            files.push((pack_path, bytes));
        }
    }
    for file in AppData::FILES {
        let pack_path = PackPath::from_static(file);
        let path = host::host_path(root, &pack_path);
        if let Some(bytes) = read_optional(&path)? {
            files.push((pack_path, bytes));
        }
    }
    Ok(AppData::from_files(files))
}

/// The bytes of the file at `path`, or `None` if it does not exist.
pub(crate) fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, ReadError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(source) if source.kind() == ErrorKind::NotFound => Ok(None),
        Err(source) => Err(HostIoError::new(HostOp::Read, path, source).into()),
    }
}
