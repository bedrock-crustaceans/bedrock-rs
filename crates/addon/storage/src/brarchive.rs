//! `.brarchive`: Mojang's uncompressed bundle of a folder's files, and the read-through layer a
//! pack consults before its loose files. The `brarchive` crate checks the format and gives each
//! entry as a byte range, so an entry stays a zero-copy slice of the archive.

use std::collections::BTreeMap;
use std::ops::Range;

use bytes::Bytes;

use super::{PackPath, StorageError};

/// The folder that holds a pack's archives.
const ARCHIVE_FOLDER: &str = "__brarchive";
/// The extension of an archive file.
const ARCHIVE_EXTENSION: &str = ".brarchive";

/// One decoded `.brarchive`: its entries, each a range of the archive bytes.
#[derive(Clone, Debug)]
pub struct Brarchive {
    bytes: Bytes,
    entries: Vec<BrarchiveEntry>,
}

/// One entry of a [`Brarchive`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrarchiveEntry {
    /// The entry name, relative to the folder the archive stands for.
    pub name: Box<str>,
    /// Where its bytes are in the archive.
    range: Range<usize>,
}

impl Brarchive {
    /// Decodes the header and descriptors of `bytes`; `Brarchive` when the magic or version is
    /// wrong, a name is longer than 247 bytes or not UTF-8, or an entry lies outside the file.
    pub fn decode(bytes: Bytes) -> Result<Brarchive, StorageError> {
        let entries = brarchive::entries(&bytes)
            .map_err(|_| StorageError::Brarchive)?
            .map(|entry| {
                let entry = entry.map_err(|_| StorageError::Brarchive)?;
                Ok(BrarchiveEntry {
                    name: entry.name.into(),
                    range: entry.range,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Brarchive { bytes, entries })
    }

    /// Every entry, in archive order.
    pub fn entries(&self) -> &[BrarchiveEntry] {
        &self.entries
    }

    /// The bytes of `entry`: a slice of the archive, no copy.
    pub fn read(&self, entry: &BrarchiveEntry) -> Bytes {
        self.bytes.slice(entry.range.clone())
    }

    /// The bytes of the entry named `name`; the last one when the name repeats. For the tests
    /// of this reader: a pack reads its archives through a [`BrarchiveLayer`].
    #[cfg(test)]
    pub(crate) fn get(&self, name: &str) -> Option<Bytes> {
        self.entries
            .iter()
            .rev()
            .find(|entry| &*entry.name == name)
            .map(|entry| self.read(entry))
    }

    /// The archive written back from its entries (the `brarchive` crate's writer). For the
    /// tests of this reader: `PackWriter` writes a layer's files loose and no archive.
    #[cfg(test)]
    pub(crate) fn encode(&self) -> Result<Vec<u8>, StorageError> {
        brarchive::serialize(
            self.entries
                .iter()
                .map(|entry| (&*entry.name, &self.bytes[entry.range.clone()])),
        )
        .map_err(|_| StorageError::Brarchive)
    }
}

/// The folder an archive stands for: `__brarchive/<dir>.brarchive` serves `<dir>/`, relative to
/// the folder that holds `__brarchive` (`subpacks/x/__brarchive/textures.brarchive` serves
/// `subpacks/x/textures/`). `None` for any other path.
pub fn archive_dir(path: &PackPath) -> Option<PackPath> {
    let text = path.as_str();
    let (before, after) = match text.split_once(&format!("{ARCHIVE_FOLDER}/")) {
        Some(("", after)) => ("", after),
        Some((before, after)) if before.ends_with('/') => (before, after),
        _ => return None,
    };
    let dir = after.strip_suffix(ARCHIVE_EXTENSION)?;
    if dir.is_empty() {
        return None;
    }
    PackPath::new(&format!("{before}{dir}")).ok()
}

/// The files of a pack's `.brarchive`s by pack path. A pack read checks this layer first and
/// falls through to the loose file; the pack's index is the union of both.
#[derive(Clone, Debug, Default)]
pub struct BrarchiveLayer {
    files: BTreeMap<PackPath, Bytes>,
}

impl BrarchiveLayer {
    /// Adds every entry of `archive` under `dir` (from [`archive_dir`]); a later entry for the
    /// same path replaces an earlier one. `PathRejected` when an entry name breaks the
    /// pack-path rules, and then nothing of the archive is added.
    pub fn add(&mut self, dir: &PackPath, archive: &Brarchive) -> Result<(), StorageError> {
        let mut files = Vec::with_capacity(archive.entries().len());
        for entry in archive.entries() {
            let name = PackPath::new(&entry.name).map_err(|reason| StorageError::PathRejected {
                path: entry.name.clone(),
                reason,
            })?;
            files.push((dir.join(&name), archive.read(entry)));
        }
        self.files.extend(files);
        Ok(())
    }

    /// The bytes the archives hold for `path`. A zero-size entry gives `None`, so it never
    /// shadows the loose file: the game's library packs ship a `scripts.brarchive` whose
    /// entries are all empty next to the loose scripts, and those load in full.
    pub fn get(&self, path: &PackPath) -> Option<Bytes> {
        self.files
            .get(path)
            .filter(|bytes| !bytes.is_empty())
            .cloned()
    }

    /// Some archive has an entry for `path`, empty or not.
    pub fn contains(&self, path: &PackPath) -> bool {
        self.files.contains_key(path)
    }

    /// Every entry path, sorted bytewise.
    pub fn paths(&self) -> impl Iterator<Item = &PackPath> + '_ {
        self.files.keys()
    }
}
