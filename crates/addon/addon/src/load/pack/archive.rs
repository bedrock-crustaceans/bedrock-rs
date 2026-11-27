//! The archive `Index` job: the central directory, the zip limits, the manifest rule and, in a
//! composite, the packs it holds; and the `Opener` steps that start it and take its layout.

use std::sync::Arc;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, JsonPath, Location, Phase, Severity};
use bedrock_addon_core::load::{Limits, Place};
use bedrock_addon_core::pack_identity::PackPath;
use bytes::Bytes;

use super::super::io::Io;
use super::super::job::{JobDone, JobKind};
use super::{CONTENTS, MANIFEST, Seal, Step, UnitKey, strip};
use crate::pack::FileSlot;

/// How deep a bare pack folder may sit in a composite.
const MAX_FOLDER_DEPTH: usize = MAX_ARCHIVE_DEPTH as usize;
use crate::pack::OpenError;
use crate::storage::zip::ZipIndex;
use crate::storage::{ContentKey, StorageError};

/// What an archive holds.
pub(super) enum Layout {
    /// One pack.
    Pack(ArchivePack),
    /// Several: bare pack folders and nested archives (inflated).
    Composite {
        index: ZipIndex,
        folders: Vec<PackPath>,
        nested: Vec<(PackPath, Result<Bytes, StorageError>)>,
        /// The composite was a whole-file encrypted archive, decrypted here.
        decrypted: bool,
    },
}

/// One pack in an archive.
pub(super) struct ArchivePack {
    pub(super) index: ZipIndex,
    /// The pack root inside the archive.
    pub(super) prefix: PackPath,
    /// The manifest's bytes.
    pub(super) manifest: Result<Bytes, StorageError>,
    /// `contents.json`'s bytes, when the pack has one.
    pub(super) contents: Option<Result<Bytes, StorageError>>,
    /// Paths the archive holds more than once.
    pub(super) duplicates: Vec<PackPath>,
    /// The encrypted bytes of a whole-file encrypted archive.
    pub(super) original: Option<Bytes>,
}

/// Why an archive yields no pack.
pub(super) enum Failure {
    Storage(StorageError),
    NoManifest,
    Several(usize),
}

impl Failure {
    pub(super) fn into_error(self, location: Place) -> OpenError {
        match self {
            Failure::Storage(error) => OpenError::Storage(error),
            Failure::NoManifest => OpenError::NoManifest { location },
            Failure::Several(count) => OpenError::MultipleManifests { location, count },
        }
    }
}

/// Decodes `bytes` (decrypting a whole-file encrypted archive with `key`) and applies the
/// manifest rule: the shallowest `manifest.json`. A `composite` archive without a manifest
/// at its root holds the packs of its folders and its nested archives.
pub(super) fn layout(
    bytes: Bytes,
    key: Option<&ContentKey>,
    limits: &Limits,
    composite: bool,
) -> Result<Layout, Failure> {
    // A whole-file encrypted archive keeps its ciphertext for `Pack::to_archive`.
    let original = key.is_some().then(|| bytes.clone());
    let bytes = match key {
        #[cfg(feature = "encrypted")]
        Some(key) => {
            crate::storage::crypt::decrypt_archive(&bytes, Some(key)).map_err(Failure::Storage)?
        }
        _ => bytes,
    };
    let index = ZipIndex::decode(bytes, limits).map_err(Failure::Storage)?;
    let root_manifest = index.get(&PackPath::from_static(MANIFEST)).is_some();
    if composite && !root_manifest {
        let (folders, too_deep) = pack_folders(&index);
        let nested = index.nested_archives();
        if folders.len() > 1
            || !too_deep.is_empty()
            || !nested.archives.is_empty()
            || !nested.unreadable.is_empty()
        {
            let mut members: Vec<(PackPath, Result<Bytes, StorageError>)> = nested
                .archives
                .into_iter()
                .map(|path| {
                    let bytes = index.read_path(&path);
                    (path, bytes)
                })
                .collect();
            members.extend(
                nested
                    .unreadable
                    .into_iter()
                    .map(|(path, error)| (path, Err(error))),
            );
            // A pack folder deeper than four is `folder_structure_too_deep`.
            members.extend(too_deep.into_iter().map(|path| (path, Err(nesting()))));
            members.sort_by(|a, b| a.0.cmp(&b.0));
            return Ok(Layout::Composite {
                index,
                folders,
                nested: members,
                decrypted: original.is_some(),
            });
        }
    }
    let prefix = shallowest_manifest(&index)?;
    let mut pack = pack_at(index, prefix);
    pack.original = original;
    Ok(Layout::Pack(pack))
}

/// The pack whose root is `prefix` in `index`.
fn pack_at(index: ZipIndex, prefix: PackPath) -> ArchivePack {
    let manifest = index.read_path(&prefix.join(&PackPath::from_static(MANIFEST)));
    let contents_path = prefix.join(&PackPath::from_static(CONTENTS));
    let contents = index.get(&contents_path).map(|entry| index.read(entry));
    let duplicates = index.duplicates().to_vec();
    ArchivePack {
        index,
        prefix,
        manifest,
        contents,
        duplicates,
        original: None,
    }
}

/// The bare pack folder `prefix` of a composite.
pub(super) fn folder(index: ZipIndex, prefix: PackPath) -> Layout {
    Layout::Pack(pack_at(index, prefix))
}

/// The folder of the shallowest `manifest.json`: one at the least depth, else no pack.
fn shallowest_manifest(index: &ZipIndex) -> Result<PackPath, Failure> {
    let manifests: Vec<(usize, &PackPath)> = index
        .files()
        .iter()
        .filter(|entry| entry.path.segments().last() == Some(MANIFEST))
        .map(|entry| (entry.path.segments().count(), &entry.path))
        .collect();
    let Some(least) = manifests.iter().map(|(depth, _)| *depth).min() else {
        return Err(Failure::NoManifest);
    };
    // A manifest more than four folders deep is `folder_structure_too_deep`.
    if least - 1 > MAX_FOLDER_DEPTH {
        return Err(Failure::Storage(nesting()));
    }
    let shallowest: Vec<&PackPath> = manifests
        .iter()
        .filter(|(depth, _)| *depth == least)
        .map(|(_, path)| *path)
        .collect();
    // Several at the same depth: "Cannot determine which pack manifest to use".
    if shallowest.len() > 1 {
        return Err(Failure::Several(shallowest.len()));
    }
    let segments: Vec<&str> = shallowest[0].segments().collect();
    let folder = segments[..segments.len() - 1].join("/");
    PackPath::new(&folder).map_err(|_| Failure::NoManifest)
}

/// The bare pack folders of a composite: every folder holding a `manifest.json`, at most
/// four folders deep and not inside another pack folder (`MyPack/`, an `.mcworld`'s
/// `behavior_packs/<pack>/`, a zipped world folder's `<world>/resource_packs/<pack>/`).
///
/// The game leaves the composite layout open: the crate takes every folder that holds a
/// `manifest.json`, at most four folders deep and not inside another pack folder, as a pack.
/// Pack folders deeper than four come back separately, for their
/// `folder_structure_too_deep`.
fn pack_folders(index: &ZipIndex) -> (Vec<PackPath>, Vec<PackPath>) {
    let mut candidates: Vec<(usize, PackPath)> = index
        .files()
        .iter()
        .filter_map(|entry| {
            let segments: Vec<&str> = entry.path.segments().collect();
            let (name, folder) = segments.split_last()?;
            if *name != MANIFEST || folder.is_empty() {
                return None;
            }
            Some((folder.len(), PackPath::new(&folder.join("/")).ok()?))
        })
        .collect();
    // Shallowest first, so a pack folder is kept before the folders below it.
    candidates.sort();
    let mut kept: Vec<PackPath> = Vec::new();
    let mut too_deep = Vec::new();
    for (depth, folder) in candidates {
        if kept
            .iter()
            .chain(&too_deep)
            .any(|root| strip(&folder, root).is_some())
        {
            continue;
        }
        if depth > MAX_FOLDER_DEPTH {
            too_deep.push(folder);
        } else {
            kept.push(folder);
        }
    }
    kept.sort();
    too_deep.sort();
    (kept, too_deep)
}

/// `folder_structure_too_deep`.
fn nesting() -> StorageError {
    StorageError::LimitExceeded {
        limit: crate::storage::ZipLimit::Nesting,
    }
}

/// How deep archives may nest inside a composite. The limit is the crate's own; a deeper
/// archive is `folder_structure_too_deep`.
const MAX_ARCHIVE_DEPTH: u8 = 4;

impl super::Opener {
    pub(super) fn index_archive(
        &mut self,
        io: &mut Io,
        index: usize,
        bytes: Bytes,
        key: Option<ContentKey>,
    ) {
        let unit = &mut self.units[index];
        unit.step = Step::IndexingArchive;
        let limits = *io.limits();
        let composite = unit.composite;
        let weight = bytes.len() as u64;
        self.job(io, index, JobKind::Index, weight, move || {
            Box::new(layout(bytes, key.as_ref(), &limits, composite))
        });
    }

    pub(super) fn on_archive_index(&mut self, io: &mut Io, index: usize, done: Option<JobDone>) {
        let location = self.units[index].location.clone();
        let Some(layout) = done.and_then(JobDone::take::<Result<Layout, Failure>>) else {
            // A panicked archive `Index` job leaves no pack.
            self.finish(io, index, Some(Err(OpenError::Internal { location })));
            return;
        };
        match layout {
            Err(failure) => {
                let error = failure.into_error(location);
                self.finish(io, index, Some(Err(error)));
            }
            Ok(Layout::Pack(pack)) => self.archive_pack(io, index, pack),
            Ok(Layout::Composite {
                index: archive,
                folders,
                nested,
                decrypted,
            }) => {
                let unit = &self.units[index];
                let (key, depth, options) =
                    (unit.key.clone(), unit.depth, Arc::clone(&unit.options));
                let seal = if decrypted { Seal::Sealed } else { unit.seal };
                let base = unit.location.clone();
                for prefix in folders {
                    let child = self.push(
                        sub_key(&key, &prefix),
                        base.join(&prefix),
                        Arc::clone(&options),
                        false,
                        depth,
                        Step::IndexingArchive,
                    );
                    self.units[child].archive_file = false;
                    self.units[child].seal = seal;
                    let archive = archive.clone();
                    self.job(io, child, JobKind::Index, 0, move || {
                        Box::new(Ok::<_, Failure>(folder(archive, prefix)))
                    });
                }
                for (path, inner) in nested {
                    let child = self.push(
                        sub_key(&key, &path),
                        base.join(&path),
                        Arc::clone(&options),
                        true,
                        depth + 1,
                        Step::IndexingArchive,
                    );
                    self.units[child].seal = seal;
                    match inner {
                        // Nesting deeper than 4 archives is `folder_structure_too_deep`.
                        Ok(_) if depth + 1 > MAX_ARCHIVE_DEPTH => {
                            let error = OpenError::Storage(StorageError::LimitExceeded {
                                limit: crate::storage::ZipLimit::Nesting,
                            });
                            self.finish(io, child, Some(Err(error)));
                        }
                        Ok(bytes) => {
                            let limits = *io.limits();
                            let weight = bytes.len() as u64;
                            self.job(io, child, JobKind::Index, weight, move || {
                                Box::new(self::layout(bytes, None, &limits, true))
                            });
                        }
                        Err(error) => self.finish(io, child, Some(Err(OpenError::Storage(error)))),
                    }
                }
                self.finish(io, index, None);
            }
        }
    }

    /// One pack in an archive: its files are the archive's below the pack root.
    fn archive_pack(&mut self, io: &mut Io, index: usize, pack: ArchivePack) {
        let unit = &mut self.units[index];
        // A folder of a composite is located at its folder already.
        if unit.archive_file {
            unit.root = unit.location.join(&pack.prefix);
        }
        for duplicate in &pack.duplicates {
            if let Some(path) = strip(duplicate, &pack.prefix) {
                unit.findings.push(Diagnostic::new(
                    Code::from(&core_codes::DUPLICATE_ENTRY),
                    // The game logs nothing for a name written twice in a zip (its only
                    // duplicate-name line is for brarchives), so this finding is the crate's
                    // own and an information.
                    Severity::Info,
                    Phase::Read,
                    Location::new(None, Some(path), JsonPath::new(), None),
                ));
            }
        }
        for entry in pack.index.files() {
            if let Some(path) = strip(&entry.path, &pack.prefix) {
                unit.sizes.insert(path.clone(), entry.size);
                unit.files.insert(path, FileSlot::Archived);
            }
        }
        let own = pack.prefix.is_root() && unit.archive_file;
        unit.contents = pack.contents;
        unit.store.archive = Some(crate::pack::open::Archive {
            index: pack.index,
            prefix: pack.prefix,
            own,
            original: pack.original,
            sealed: unit.seal == Seal::Sealed,
        });
        match pack.manifest {
            Ok(bytes) => {
                self.parse_manifest(io, index, bytes);
                // The manifest stays in the archive; the slot says where it is.
                self.units[index]
                    .files
                    .insert(PackPath::from_static(MANIFEST), FileSlot::Archived);
            }
            Err(error) => self.finish(io, index, Some(Err(OpenError::Storage(error)))),
        }
    }
}

/// The key of a pack inside a composite: the composite's key with the member's path.
fn sub_key(key: &UnitKey, path: &PackPath) -> UnitKey {
    (key.0, key.1, key.2.join(path))
}
