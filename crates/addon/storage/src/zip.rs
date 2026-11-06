//! Zip archives over resident bytes: the index (paths, sizes, limits, duplicates), entry reads,
//! nested archives, and the writer `PackWriter` calls.
//!
//! The `zip` crate runs over `Cursor<Bytes>` only; nothing here touches the host file system
//! (`extract` is on the clippy disallowed list).

use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read, Write};

use ::zip::result::ZipError;
use ::zip::write::SimpleFileOptions;
use ::zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};
use bedrock_addon_core::load::Limits;
use bytes::Bytes;

use super::magic::{FileType, detect};
use super::{PackPath, StorageError, ZipFault, ZipLimit};

/// The longest entry name, in bytes.
const MAX_ENTRY_NAME_BYTES: usize = 255;
/// At most this much is preallocated for an entry read; the rest grows as it inflates, so a
/// declared size alone never allocates.
const READ_CHUNK: usize = 1 << 20;
/// The central-directory file-header signature, `PK\x01\x02`.
const CENTRAL_HEADER_MAGIC: [u8; 4] = [0x50, 0x4B, 0x01, 0x02];
/// The fixed part of a central-directory file header.
const CENTRAL_HEADER_LEN: usize = 46;
/// The end-of-central-directory signature, `PK\x05\x06`, and the record's fixed size.
const END_RECORD_MAGIC: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];
const END_RECORD_LEN: usize = 22;
/// The zip64 end-record locator, `PK\x06\x07`, 20 bytes before the end record.
const ZIP64_LOCATOR_MAGIC: [u8; 4] = [0x50, 0x4B, 0x06, 0x07];
const ZIP64_LOCATOR_LEN: usize = 20;
/// The zip64 end record, `PK\x06\x06`, and its fixed size.
const ZIP64_END_RECORD_MAGIC: [u8; 4] = [0x50, 0x4B, 0x06, 0x06];
const ZIP64_END_RECORD_LEN: usize = 56;

/// The file index of one zip archive held in memory: every file's path and declared size,
/// checked against the zip-bomb and path limits. Reads inflate on the calling thread.
///
/// Cheap to clone: the archive's metadata is shared and the bytes are a [`Bytes`].
#[derive(Clone, Debug)]
pub struct ZipIndex {
    /// The archive as given: `Pack::to_archive` serves it unchanged.
    bytes: Bytes,
    archive: ZipArchive<Cursor<Bytes>>,
    /// Sorted bytewise by path.
    files: Vec<ZipEntry>,
    /// Paths written more than once (by name or after normalising), sorted.
    duplicates: Vec<PackPath>,
}

/// The nested archives of a composite: [`ZipIndex::nested_archives`].
#[derive(Debug, Default)]
pub struct NestedArchives {
    /// The top-level files that are zip archives, sorted.
    pub archives: Vec<PackPath>,
    /// The top-level files that could not be read to tell, with the reason.
    pub unreadable: Vec<(PackPath, StorageError)>,
}

/// One file of a [`ZipIndex`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZipEntry {
    /// The entry's name as a pack path.
    pub path: PackPath,
    /// Its decompressed size, as the central directory declares it.
    pub size: u64,
    /// Its index in the `zip` crate's archive.
    index: usize,
}

impl ZipIndex {
    /// Decodes the central directory of `bytes` and checks every limit that the directory
    /// alone can decide: the archive size, the entry count, entry-name lengths, each entry's
    /// declared size and their sum, and the path rules.
    ///
    /// Directory entries are not files and are left out. Duplicate names keep the last entry
    /// and are listed in [`ZipIndex::duplicates`] for the `duplicate_entry` warning.
    pub fn decode(bytes: Bytes, limits: &Limits) -> Result<ZipIndex, StorageError> {
        if bytes.len() as u64 > limits.max_archive_bytes {
            return Err(StorageError::LimitExceeded {
                limit: ZipLimit::ArchiveBytes,
            });
        }
        // The count the end record declares is checked before the `zip` crate reads (and
        // allocates for) the whole central directory, so a declared count cannot make the
        // read allocate first.
        let end = end_record(&bytes);
        if end.is_some_and(|end| end.entries > u64::from(limits.max_entries)) {
            return Err(entries_exceeded());
        }
        // The `zip` crate keeps one record per name, so its count hides duplicates and its
        // records skip shadowed ones; the raw central directory has them all.
        let raw = end.and_then(|end| raw_central_names(&bytes, end.directory));
        if let Some(raw) = &raw {
            check_raw_records(raw, limits)?;
        }
        let mut archive = ZipArchive::new(Cursor::new(bytes.clone())).map_err(|e| zip_error(&e))?;
        // Data prepended to the archive shifts the directory: walk it where the crate found it.
        let raw = match raw {
            Some(raw) if !raw.is_empty() || archive.is_empty() => Some(raw),
            _ => raw_central_names(&bytes, archive.central_directory_start()),
        };
        if let Some(raw) = &raw {
            check_raw_records(raw, limits)?;
        }
        if archive.len() > limits.max_entries as usize {
            return Err(entries_exceeded());
        }
        let repeated = repeated_names(raw.as_deref().unwrap_or_default());

        let mut by_path: HashMap<PackPath, usize> = HashMap::new();
        let mut files: Vec<ZipEntry> = Vec::new();
        let mut duplicates = Vec::new();
        let mut total: u64 = 0;
        for index in 0..archive.len() {
            let file = archive.by_index_raw(index).map_err(|e| zip_error(&e))?;
            if file.name_raw().len() > MAX_ENTRY_NAME_BYTES {
                return Err(StorageError::LimitExceeded {
                    limit: ZipLimit::EntryName,
                });
            }
            if file.is_dir() {
                continue;
            }
            let name = file.name();
            let path = PackPath::new(name).map_err(|reason| StorageError::PathRejected {
                path: name.into(),
                reason,
            })?;
            if path.is_root() {
                // A file entry whose name is only separators names nothing a read can reach.
                continue;
            }
            let size = file.size();
            if size > limits.max_entry_bytes {
                return Err(StorageError::LimitExceeded {
                    limit: ZipLimit::EntryBytes,
                });
            }
            total = total.saturating_add(size);
            if total > limits.max_pack_bytes {
                return Err(StorageError::LimitExceeded {
                    limit: ZipLimit::PackBytes,
                });
            }
            if repeated.contains(file.name_raw()) {
                duplicates.push(path.clone());
            }
            let entry = ZipEntry { path, size, index };
            // Two names that normalise to one path: the later entry wins, as for equal names.
            if let Some(&slot) = by_path.get(&entry.path) {
                duplicates.push(entry.path.clone());
                files[slot] = entry;
            } else {
                by_path.insert(entry.path.clone(), files.len());
                files.push(entry);
            }
        }
        files.sort_by(|a, b| a.path.cmp(&b.path));
        duplicates.sort();
        duplicates.dedup();
        Ok(ZipIndex {
            bytes,
            archive,
            files,
            duplicates,
        })
    }

    /// Every file, sorted bytewise by path.
    pub fn files(&self) -> &[ZipEntry] {
        &self.files
    }

    /// Paths written more than once; the index keeps the last entry of each.
    pub fn duplicates(&self) -> &[PackPath] {
        &self.duplicates
    }

    /// The archive bytes the index was decoded from.
    pub fn bytes(&self) -> &Bytes {
        &self.bytes
    }

    /// The file at `path`.
    pub fn get(&self, path: &PackPath) -> Option<&ZipEntry> {
        self.files
            .binary_search_by(|entry| entry.path.cmp(path))
            .ok()
            .map(|slot| &self.files[slot])
    }

    /// Inflates one entry. The output is capped at the declared size (which the index already
    /// checked against the limits): an entry that inflates past it is malformed, so a lying
    /// central directory cannot make a read allocate more than it declared.
    pub fn read(&self, entry: &ZipEntry) -> Result<Bytes, StorageError> {
        let bytes = self.read_at_most(entry, entry.size.saturating_add(1))?;
        if bytes.len() as u64 != entry.size {
            return Err(StorageError::Zip {
                fault: ZipFault::Malformed,
            });
        }
        Ok(bytes)
    }

    /// Inflates the file at `path`; `NotFound` if the archive has none.
    pub fn read_path(&self, path: &PackPath) -> Result<Bytes, StorageError> {
        let entry = self.get(path).ok_or(StorageError::NotFound)?;
        self.read(entry)
    }

    /// The top-level files that are themselves zip archives, by magic bytes, not by extension:
    /// the `.mcpack` / `.mctemplate` / `.mcworld` / nested `.mcaddon` members of a composite
    /// (archives sit at the top level only). Each is opened from the outer bytes with
    /// [`ZipIndex::read`] and [`ZipIndex::decode`]. A top-level file that cannot be read to tell
    /// (zip-level encryption, an unsupported method) is listed with its error and the rest are
    /// still examined.
    pub fn nested_archives(&self) -> NestedArchives {
        let mut nested = NestedArchives::default();
        for entry in self
            .files
            .iter()
            .filter(|entry| entry.path.segments().count() == 1)
        {
            match self.read_at_most(entry, 4) {
                Ok(prefix) if detect(&prefix) == FileType::Zip => {
                    nested.archives.push(entry.path.clone());
                }
                Ok(_) => {}
                Err(error) => nested.unreadable.push((entry.path.clone(), error)),
            }
        }
        nested
    }

    /// Up to `limit` inflated bytes of `entry`.
    fn read_at_most(&self, entry: &ZipEntry, limit: u64) -> Result<Bytes, StorageError> {
        // `ZipArchive` is a cheap clone (shared metadata, a `Bytes` cursor), so reads take
        // `&self` and may run on any thread.
        let mut archive = self.archive.clone();
        let file = archive.by_index(entry.index).map_err(|e| zip_error(&e))?;
        let capacity = usize::try_from(limit).unwrap_or(usize::MAX).min(READ_CHUNK);
        let mut out = Vec::with_capacity(capacity);
        file.take(limit)
            .read_to_end(&mut out)
            .map_err(|_| StorageError::Zip {
                fault: ZipFault::Malformed,
            })?;
        Ok(Bytes::from(out))
    }
}

/// Zip bytes of `files`: `manifest.json` first, the rest in the order given; every entry
/// deflated and dated 1980-01-01 (`DateTime::DEFAULT`), so the same files always give the same
/// bytes: the wall clock never enters the archive.
pub fn encode<'a>(
    files: impl IntoIterator<Item = (&'a PackPath, &'a [u8])>,
) -> Result<Bytes, StorageError> {
    let manifest = PackPath::from_static("manifest.json");
    let (first, rest): (Vec<_>, Vec<_>) =
        files.into_iter().partition(|(path, _)| **path == manifest);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(DateTime::DEFAULT)
        .unix_permissions(0o644);
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (path, bytes) in first.into_iter().chain(rest) {
        writer
            .start_file(path.as_str(), options)
            .map_err(|e| zip_error(&e))?;
        writer.write_all(bytes).map_err(|_| StorageError::Zip {
            fault: ZipFault::Malformed,
        })?;
    }
    let cursor = writer.finish().map_err(|e| zip_error(&e))?;
    Ok(Bytes::from(cursor.into_inner()))
}

/// The limit of `limits` an archive of `files` breaks when it is read: more entries than an
/// archive may have, an entry name longer than the reader takes, a file or the files
/// together beyond their sizes. `None` for files the reader takes.
pub fn refused_by<'a>(
    files: impl IntoIterator<Item = (&'a PackPath, &'a [u8])>,
    limits: &Limits,
) -> Option<ZipLimit> {
    let mut total: u64 = 0;
    for (index, (path, bytes)) in files.into_iter().enumerate() {
        if index >= limits.max_entries as usize {
            return Some(ZipLimit::Entries);
        }
        if path.as_str().len() > MAX_ENTRY_NAME_BYTES {
            return Some(ZipLimit::EntryName);
        }
        let size = bytes.len() as u64;
        if size > limits.max_entry_bytes {
            return Some(ZipLimit::EntryBytes);
        }
        total = total.saturating_add(size);
        if total > limits.max_pack_bytes {
            return Some(ZipLimit::PackBytes);
        }
    }
    None
}

/// The storage error for a `zip` crate error.
fn zip_error(error: &ZipError) -> StorageError {
    match error {
        ZipError::UnsupportedArchive(_) | ZipError::InvalidPassword => StorageError::Zip {
            fault: ZipFault::Unsupported,
        },
        ZipError::FileNotFound => StorageError::NotFound,
        _ => StorageError::Zip {
            fault: ZipFault::Malformed,
        },
    }
}

/// What the end-of-central-directory record declares.
#[derive(Clone, Copy, Debug)]
struct EndRecord {
    /// The total entry count (from the zip64 record when the classic field is saturated).
    entries: u64,
    /// The central directory's offset.
    directory: u64,
}

/// The end-of-central-directory record of `bytes`: the last `PK\x05\x06` in the final
/// 22 + 65 535 bytes whose fields are consistent with the file, read through its zip64 locator
/// when a field is saturated. A signature that is not the record (inside the archive comment)
/// is passed over for the one before it. `None` when there is none (the `zip` crate then
/// reports the archive malformed, or finds a record this check does not).
fn end_record(bytes: &[u8]) -> Option<EndRecord> {
    let last = bytes.len().checked_sub(END_RECORD_LEN)?;
    let first = last.saturating_sub(usize::from(u16::MAX));
    (first..=last)
        .rev()
        .filter(|&at| bytes[at..at + 4] == END_RECORD_MAGIC)
        .find_map(|at| end_record_at(bytes, at))
}

/// The end record at `at` if it is consistent with the file: its comment ends inside the file
/// (trailing bytes after it are tolerated, as the `zip` crate does), and the central directory
/// it declares ends at or before the record and has room for the entries it counts.
fn end_record_at(bytes: &[u8], at: usize) -> Option<EndRecord> {
    let comment = usize::from(u16_at(bytes, at + 20));
    if at + END_RECORD_LEN + comment > bytes.len() {
        return None;
    }
    let entries = u16_at(bytes, at + 10);
    let size = u32_at(bytes, at + 12);
    let directory = u32_at(bytes, at + 16);
    let saturated = entries == u16::MAX || size == u32::MAX || directory == u32::MAX;
    if saturated && has_zip64_locator(bytes, at) {
        return zip64_end_record(bytes, at);
    }
    consistent(
        EndRecord {
            entries: u64::from(entries),
            directory: u64::from(directory),
        },
        u64::from(size),
        at,
    )
}

/// Whether the zip64 end-record locator sits just before the end record at `end_at`.
fn has_zip64_locator(bytes: &[u8], end_at: usize) -> bool {
    end_at
        .checked_sub(ZIP64_LOCATOR_LEN)
        .is_some_and(|locator| bytes[locator..locator + 4] == ZIP64_LOCATOR_MAGIC)
}

/// The zip64 end record named by the locator just before the end record at `end_at`, if it is
/// consistent with the file.
fn zip64_end_record(bytes: &[u8], end_at: usize) -> Option<EndRecord> {
    let locator = end_at.checked_sub(ZIP64_LOCATOR_LEN)?;
    let at = usize::try_from(u64_at(bytes, locator + 8)).ok()?;
    if at.checked_add(ZIP64_END_RECORD_LEN)? > locator {
        return None;
    }
    let record = &bytes[at..at + ZIP64_END_RECORD_LEN];
    if record[..4] != ZIP64_END_RECORD_MAGIC {
        return None;
    }
    consistent(
        EndRecord {
            entries: u64_at(record, 32),
            directory: u64_at(record, 48),
        },
        u64_at(record, 40),
        at,
    )
}

/// `end` if its central directory, `size` bytes long, ends at or before `record_at` and has
/// room for a fixed header per entry. Data prepended to the archive only moves the directory
/// later, so the declared offset plus size can fall short of the record but never past it.
fn consistent(end: EndRecord, size: u64, record_at: usize) -> Option<EndRecord> {
    let ends = end.directory.checked_add(size)?;
    let room = end.entries.checked_mul(CENTRAL_HEADER_LEN as u64)?;
    (ends <= record_at as u64 && room <= size).then_some(end)
}

/// The entry-count and name-length limits over every raw central-directory record.
fn check_raw_records(raw: &[&[u8]], limits: &Limits) -> Result<(), StorageError> {
    if raw.len() > limits.max_entries as usize {
        return Err(entries_exceeded());
    }
    if raw.iter().any(|name| name.len() > MAX_ENTRY_NAME_BYTES) {
        return Err(StorageError::LimitExceeded {
            limit: ZipLimit::EntryName,
        });
    }
    Ok(())
}

fn entries_exceeded() -> StorageError {
    StorageError::LimitExceeded {
        limit: ZipLimit::Entries,
    }
}

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    let mut field = [0; 8];
    field.copy_from_slice(&bytes[at..at + 8]);
    u64::from_le_bytes(field)
}

/// The raw name of every central-directory record from `start`, duplicates included; `None`
/// when the records do not walk cleanly (the `zip` crate has already accepted the directory,
/// so this only loses the duplicate report).
fn raw_central_names(bytes: &[u8], start: u64) -> Option<Vec<&[u8]>> {
    let mut at = usize::try_from(start).ok()?;
    let mut names = Vec::new();
    while bytes.get(at..at + 4) == Some(&CENTRAL_HEADER_MAGIC[..]) {
        let header = bytes.get(at..at + CENTRAL_HEADER_LEN)?;
        let field =
            |offset: usize| usize::from(u16::from_le_bytes([header[offset], header[offset + 1]]));
        let (name_len, extra_len, comment_len) = (field(28), field(30), field(32));
        let name_start = at + CENTRAL_HEADER_LEN;
        names.push(bytes.get(name_start..name_start + name_len)?);
        at = name_start + name_len + extra_len + comment_len;
    }
    Some(names)
}

/// The names that occur more than once.
fn repeated_names<'a>(names: &[&'a [u8]]) -> HashSet<&'a [u8]> {
    let mut seen = HashSet::new();
    names
        .iter()
        .filter(|name| !seen.insert(**name))
        .copied()
        .collect()
}
