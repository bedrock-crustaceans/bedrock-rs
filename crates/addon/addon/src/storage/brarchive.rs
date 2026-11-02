//! `.brarchive`: Mojang's uncompressed bundle of a folder's files, and the read-through layer a
//! pack consults before its loose files.
//!
//! Layout: a 16-byte header (u64 magic `0x267052A0B125277D`, u32 entry count, u32 version 1),
//! one 256-byte descriptor per entry (u8 name length, 247 name bytes, u32 offset at 0xF8, u32
//! size at 0xFC), then the data, at `0x10 + count * 256 + offset`. The `brarchive` crate checks
//! the header and the names; the entry ranges are read here so that an entry stays a zero-copy
//! slice of the archive.

use std::collections::BTreeMap;
use std::ops::Range;

use bytes::Bytes;

use super::{PackPath, StorageError};

/// The header size.
const HEADER_LEN: usize = 16;
/// One entry descriptor's size.
const DESCRIPTOR_LEN: usize = 256;
/// The offset of the u32 data offset inside a descriptor; the u32 size follows it.
const DESCRIPTOR_OFFSET_FIELD: usize = 0xF8;
/// The folder that holds a pack's archives.
const ARCHIVE_FOLDER: &str = "__brarchive";
/// The extension of an archive file.
const ARCHIVE_EXTENSION: &str = ".brarchive";

/// One decoded `.brarchive`: its entries, each a range of the archive bytes.
#[derive(Clone, Debug)]
pub(crate) struct Brarchive {
    bytes: Bytes,
    entries: Vec<BrarchiveEntry>,
}

/// One entry of a [`Brarchive`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BrarchiveEntry {
    /// The entry name, relative to the folder the archive stands for.
    pub(crate) name: Box<str>,
    /// Where its bytes are in the archive.
    range: Range<usize>,
}

impl Brarchive {
    /// Decodes the header and descriptors of `bytes`; `Brarchive` when the magic or version is
    /// wrong, a name is longer than 247 bytes or not UTF-8, or an entry lies outside the file.
    pub(crate) fn decode(bytes: Bytes) -> Result<Brarchive, StorageError> {
        // `brarchive::list` indexes the buffer for a name without a bounds check, so the
        // descriptors must be known to fit before it runs.
        let count = bytes
            .get(8..12)
            .map(|field| u64::from(u32::from_le_bytes([field[0], field[1], field[2], field[3]])))
            .ok_or(StorageError::Brarchive)?;
        let data_start = HEADER_LEN as u64 + count * DESCRIPTOR_LEN as u64;
        if data_start > bytes.len() as u64 {
            return Err(StorageError::Brarchive);
        }
        let names = brarchive::list(&bytes).map_err(|_| StorageError::Brarchive)?;
        // The descriptors fit, so every index below is in bounds and fits a `usize`.
        let data_start = usize::try_from(data_start).map_err(|_| StorageError::Brarchive)?;
        let entries = names
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                let field = HEADER_LEN + index * DESCRIPTOR_LEN + DESCRIPTOR_OFFSET_FIELD;
                let offset = read_u32(&bytes, field);
                let size = read_u32(&bytes, field + 4);
                let start = data_start
                    .checked_add(offset)
                    .ok_or(StorageError::Brarchive)?;
                let end = start.checked_add(size).ok_or(StorageError::Brarchive)?;
                if end > bytes.len() {
                    return Err(StorageError::Brarchive);
                }
                Ok(BrarchiveEntry {
                    name: name.into(),
                    range: start..end,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Brarchive { bytes, entries })
    }

    /// Every entry, in archive order.
    pub(crate) fn entries(&self) -> &[BrarchiveEntry] {
        &self.entries
    }

    /// The bytes of `entry`: a slice of the archive, no copy.
    pub(crate) fn read(&self, entry: &BrarchiveEntry) -> Bytes {
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

/// The u32 at `at` (in bounds: the caller checked the descriptors fit), as a `usize`.
fn read_u32(bytes: &[u8], at: usize) -> usize {
    let field = [bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]];
    u32::from_le_bytes(field) as usize
}

/// The folder an archive stands for: `__brarchive/<dir>.brarchive` serves `<dir>/`, relative to
/// the folder that holds `__brarchive` (`subpacks/x/__brarchive/textures.brarchive` serves
/// `subpacks/x/textures/`). `None` for any other path.
pub(crate) fn archive_dir(path: &PackPath) -> Option<PackPath> {
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
pub(crate) struct BrarchiveLayer {
    files: BTreeMap<PackPath, Bytes>,
}

impl BrarchiveLayer {
    /// Adds every entry of `archive` under `dir` (from [`archive_dir`]); a later entry for the
    /// same path replaces an earlier one. `PathRejected` when an entry name breaks the
    /// pack-path rules, and then nothing of the archive is added.
    pub(crate) fn add(&mut self, dir: &PackPath, archive: &Brarchive) -> Result<(), StorageError> {
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
    pub(crate) fn get(&self, path: &PackPath) -> Option<Bytes> {
        self.files
            .get(path)
            .filter(|bytes| !bytes.is_empty())
            .cloned()
    }

    /// Some archive has an entry for `path`, empty or not.
    pub(crate) fn contains(&self, path: &PackPath) -> bool {
        self.files.contains_key(path)
    }

    /// Every entry path, sorted bytewise.
    pub(crate) fn paths(&self) -> impl Iterator<Item = &PackPath> + '_ {
        self.files.keys()
    }
}
