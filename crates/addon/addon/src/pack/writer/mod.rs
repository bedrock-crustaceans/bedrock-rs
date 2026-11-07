//! Building a pack in memory and turning it into files, zip bytes, or a `Pack` for a stack.
//!
//! A [`PackWriter`] holds a manifest and a map of files. It starts from a manifest
//! ([`PackWriter::new`]) or from a copy of a pack ([`PackWriter::from_pack`]); documents go in
//! at their default path ([`PackWriter::add`], which never replaces) or at a path the caller
//! gives ([`PackWriter::put`], [`PackWriter::put_bytes`], which do). `manifest.json` is not
//! one of the files: it is written from the manifest, in the format the manifest was read in
//! (a format-0 manifest as format 1), whenever the files are asked for.
//!
//! The writer performs no I/O. [`PackWriter::files`] lists every file, `manifest.json` first
//! and the rest in bytewise path order (a manifest its format cannot hold is left out, and
//! the next two calls fail); `PackWriter::to_archive` (feature `zip`) gives zip
//! bytes that depend on the files alone; [`PackWriter::into_pack`] gives a `Pack` with every
//! file resident. `bedrock_addon_std::write_dir` and `write_archive` put the first two on disk.
//!
//! What a copy is not: the files a `__brarchive` archive serves are copied as loose files and
//! the archive is left out; `contents.json` and `signatures.json` of the pack root are left
//! out; a binary (`\x7FMCB`) document whose layout the crate decodes is copied as the JSON
//! text it stands for, at its path, and any other binary document with its bytes; an
//! encrypted pack is refused. The crate has no encoder for the binary form.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{LazyLock, OnceLock};

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, JsonPath, Location, PackReport, Phase, Severity,
};
use bedrock_addon_core::document::{PlacedWritable, Writable};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{Place, RootId};
use bedrock_addon_core::pack_identity::{PackOrigin, PackPath};

use bytes::Bytes;

use super::open::{Encryption, Store};
use super::{OpenError, OpenOptions, Pack};
use crate::manifest::Manifest;
use crate::storage::StorageError;
use crate::storage::brarchive::{Brarchive, BrarchiveLayer, archive_dir};
use crate::storage::magic::MCB_MAGIC;
use crate::storage::mcb;

/// Where the manifest is written.
static MANIFEST_PATH: LazyLock<PackPath> = LazyLock::new(|| PackPath::from_static("manifest.json"));

/// Builds a pack in memory and turns it into files, zip bytes, or a `Pack` for a stack.
///
/// `manifest.json` is not a file of its own: it is written from the manifest
/// ([`PackWriter::manifest_mut`]) in the format the manifest was read in
/// ([`Manifest::format`]; a new manifest is format 2), each time the files are asked for after
/// a change. A manifest its format cannot hold ([`Manifest::write_json`] fails) is left out of
/// [`PackWriter::files`], and [`PackWriter::to_archive`] and [`PackWriter::into_pack`] fail.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "block")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use bedrock_addon::families::block::Block;
/// use bedrock_addon::id::BlockId;
/// use bedrock_addon::manifest::Manifest;
/// use bedrock_addon::pack::{PackOrigin, PackType, PackWriter};
///
/// let mut writer = PackWriter::new(Manifest::new("Ruby lamps", PackType::Behavior));
/// let path = writer.add(&Block::new(BlockId::from_static("demo:ruby_lamp")))?;
/// assert_eq!(path.as_str(), "blocks/demo/ruby_lamp.json");
/// let pack = writer.into_pack(PackOrigin::World)?;
/// assert!(pack.read(&path).is_ok());
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "block"))]
/// # fn main() {}
/// ```
pub struct PackWriter {
    manifest: Manifest,
    /// `manifest.json` as the manifest is now: written on first need, forgotten whenever
    /// `manifest_mut` hands the manifest out.
    written: OnceLock<Result<Vec<u8>, EncodeError>>,
    /// Every file but `manifest.json`, bytewise by path.
    files: BTreeMap<PackPath, Bytes>,
}

/// The manifest and the files; the cached `manifest.json` text is written again on first need.
impl Clone for PackWriter {
    fn clone(&self) -> Self {
        Self {
            manifest: self.manifest.clone(),
            written: OnceLock::new(),
            files: self.files.clone(),
        }
    }
}

impl PackWriter {
    /// A writer with this manifest and no files.
    pub fn new(manifest: Manifest) -> Self {
        Self {
            manifest,
            written: OnceLock::new(),
            files: BTreeMap::new(),
        }
    }

    /// `manifest.json` as the manifest is now, or why its format cannot hold it.
    fn manifest_json(&self) -> Result<&[u8], &EncodeError> {
        self.written
            .get_or_init(|| write_manifest(&self.manifest))
            .as_deref()
    }

    /// Copies every file of `pack` (fails for encrypted packs: the crate never writes content
    /// keys).
    ///
    /// The manifest is copied as a value and every other file with its bytes, as
    /// [`Pack::read`] gives them, so the files a `__brarchive` archive serves become loose
    /// files and the archive is left out. A binary document (`\x7FMCB`, the baked form of a
    /// JSON document) is copied as the JSON text it stands for, at its path, where its
    /// layout is one this crate decodes; any other binary document is copied as its bytes,
    /// and a stack that loads the file from the copy reports it as unsupported, as one that
    /// loads it from `pack` does. Not copied:
    /// `contents.json` and `signatures.json` of the pack root, which list and sign the bytes
    /// this writer is there to change.
    ///
    /// `Encrypted` for a pack with encrypted files, with or without its key, and for one out
    /// of a whole-file encrypted archive: a copy would be the pack in plain text.
    /// `Storage(NotResident)` when a file is indexed and its bytes are not loaded.
    ///
    /// # Errors
    ///
    /// [`WriteError`] as described above: `Encrypted` for an encrypted pack, `Storage` when a file is
    /// not loaded.
    pub fn from_pack(pack: &Pack) -> Result<Self, WriteError> {
        let store = pack.files().store();
        if is_encrypted(store) {
            return Err(WriteError::Encrypted);
        }
        let mut files = BTreeMap::new();
        for path in pack.paths() {
            if NOT_COPIED.contains(&path.as_str()) {
                continue;
            }
            let bytes = pack.read(path)?;
            if is_served(&store.brarchive, path, &bytes) {
                continue;
            }
            files.insert(path.clone(), into_json_text(bytes));
        }
        Ok(Self {
            manifest: pack.manifest().clone(),
            written: OnceLock::new(),
            files,
        })
    }

    /// The manifest, as it will be written.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// The manifest, to change before writing.
    pub fn manifest_mut(&mut self) -> &mut Manifest {
        // Whatever the caller does with it, the text written before is stale.
        self.written = OnceLock::new();
        &mut self.manifest
    }

    /// Adds a document at its default path (`blocks/<namespace>/<name>.json`,
    /// `entities/<namespace>/<name>.json`, …); `DuplicatePath` if taken. A value or a loaded
    /// [`Document`](bedrock_addon_core::document::Document), which writes its unknown members
    /// with it, as [`put`](Self::put) writes them.
    ///
    /// A taken path is never overwritten, whatever holds it: two identifiers can ask for one
    /// path, and the second document goes where [`PackWriter::put`] is told to put it. A value
    /// with no JSON spelling is `Encode`. Either way the writer stays as it was.
    ///
    /// # Errors
    ///
    /// [`WriteError`]: `DuplicatePath` when the default path is taken, `Encode` for a value with no JSON
    /// spelling, `ReservedPath` for a path that is no file's.
    pub fn add<W: PlacedWritable>(&mut self, value: &W) -> Result<PackPath, WriteError> {
        let path = file_path(value.add_path())?;
        if self.files.contains_key(&path) {
            return Err(WriteError::DuplicatePath { path });
        }
        self.put(path.clone(), value)?;
        Ok(path)
    }

    /// Writes a document at `path`, replacing what was there: the text of
    /// [`Content::write_json`](bedrock_addon_core::document::Content::write_json) in the
    /// default style, of a value or of a loaded
    /// [`Document`](bedrock_addon_core::document::Document), which writes its unknown members
    /// with it ([`Writable`]). A value with no JSON spelling is `Encode` and a path that is no
    /// file's (`manifest.json`, the root) `ReservedPath`; either leaves the writer as it was.
    /// An unknown member of a document whose object the value no longer holds is not written
    /// ([`put_reporting`](Self::put_reporting) tells it).
    ///
    /// # Errors
    ///
    /// [`WriteError`]: `Encode` for a value with no JSON spelling, `ReservedPath` for a path that is no
    /// file's.
    pub fn put<W: Writable>(&mut self, path: PackPath, value: &W) -> Result<(), WriteError> {
        self.put_reporting(path, value).map(drop)
    }

    /// [`put`](Self::put), and the findings of the write: each unknown member of a document
    /// left out because the value no longer holds its object, a `DroppedField` warning at the
    /// member's path.
    ///
    /// # Errors
    ///
    /// As [`put`](Self::put).
    pub fn put_reporting<W: Writable>(
        &mut self,
        path: PackPath,
        value: &W,
    ) -> Result<Vec<Diagnostic>, WriteError> {
        let path = file_path(path)?;
        let mut out = Vec::new();
        let told = value.write_file(&mut out, WriteStyle::default())?;
        self.files.insert(path, Bytes::from(out));
        Ok(told)
    }

    /// Writes raw bytes at `path`, replacing what was there. `ReservedPath` for
    /// `manifest.json` (change the manifest through [`PackWriter::manifest_mut`]) and for the
    /// pack root.
    ///
    /// Accepts anything `Into<Bytes>` (`Bytes`, `Vec<u8>`, `&'static [u8]`); a fixed-size array
    /// needs `.to_vec()`.
    ///
    /// # Errors
    ///
    /// [`WriteError::ReservedPath`] for `manifest.json` and for the pack root.
    pub fn put_bytes(&mut self, path: PackPath, bytes: impl Into<Bytes>) -> Result<(), WriteError> {
        let path = file_path(path)?;
        self.files.insert(path, bytes.into());
        Ok(())
    }

    /// Removes the file at `path`; `false` if there was none. The manifest is not a file that
    /// can be removed.
    pub fn remove(&mut self, path: &PackPath) -> bool {
        self.files.remove(path).is_some()
    }

    /// Every file, `manifest.json` first, then bytewise path order. To disk:
    /// `bedrock_addon_std::write_dir(&writer, dir)`.
    pub fn files(&self) -> impl Iterator<Item = (&PackPath, &[u8])> + '_ {
        let manifest = self
            .manifest_json()
            .ok()
            .map(|bytes| (&*MANIFEST_PATH, bytes));
        manifest
            .into_iter()
            .chain(self.files.iter().map(|(path, bytes)| (path, &bytes[..])))
    }

    /// The pack as zip bytes (deflate, `manifest.json` first). To disk:
    /// `bedrock_addon_std::write_archive(&writer, path)`.
    ///
    /// The entries are the files in the order of [`PackWriter::files`], each dated 1980-01-01:
    /// the same manifest and files give the same bytes, from any writer, on any host. `Encode`
    /// when the manifest's format cannot hold it.
    ///
    /// An archive [`Pack::from_archive`] refuses under the default
    /// [`Limits`](bedrock_addon_core::load::Limits) is not written: `Zip` with the reader's
    /// `LimitExceeded` for a file whose path is longer than 255 bytes, more files than an
    /// archive may have entries, a file or the files together beyond their sizes. The limits
    /// are the archive's: [`PackWriter::files`] and [`PackWriter::into_pack`] take such a
    /// writer.
    ///
    /// # Errors
    ///
    /// [`WriteError`]: `Encode` for a manifest its format cannot hold, `Zip` when the archive
    /// writer refuses the files (see above).
    #[cfg(feature = "zip")]
    #[cfg_attr(docsrs, doc(cfg(feature = "zip")))]
    pub fn to_archive(&self) -> Result<Vec<u8>, WriteError> {
        // `files()` leaves an unwritable manifest out; here it is the error.
        if self.manifest_json().is_err() {
            write_manifest(&self.manifest)?;
        }
        // What the reader refuses under its default limits is not written.
        let limits = bedrock_addon_core::load::Limits::default();
        let exceeded = |limit| WriteError::Zip {
            source: StorageError::LimitExceeded { limit },
        };
        if let Some(limit) = crate::storage::zip::refused_by(self.files(), &limits) {
            return Err(exceeded(limit));
        }
        let archive = crate::storage::zip::encode(self.files())
            .map_err(|source| WriteError::Zip { source })?;
        if archive.len() as u64 > limits.max_archive_bytes {
            return Err(exceeded(crate::storage::ZipLimit::ArchiveBytes));
        }
        Ok(Vec::from(archive))
    }

    /// An in-memory pack, e.g. a server-generated resource pack added to the stack through
    /// `Source::Packs` and sent to clients.
    ///
    /// The files go through [`Pack::from_files`], the loader a pack on disk goes through, with
    /// `origin` and otherwise the default [`OpenOptions`]; every file is resident and the
    /// location is `RootId::MEMORY`. A manifest its format cannot hold is `InvalidManifest`,
    /// the report naming the member ([`Manifest::write_json`] gives the same answer before
    /// any file is built).
    ///
    /// # Errors
    ///
    /// [`OpenError`] when the files do not load as a pack, e.g. `InvalidManifest` for a manifest its
    /// format cannot hold.
    pub fn into_pack(self, origin: PackOrigin) -> Result<Pack, OpenError> {
        let Self {
            manifest,
            written,
            files,
        } = self;
        let json = written
            .into_inner()
            .unwrap_or_else(|| write_manifest(&manifest))
            .map_err(|error| unwritable(&manifest, error, origin))?;
        let files = std::iter::once((MANIFEST_PATH.clone(), Bytes::from(json))).chain(files);
        Pack::from_files(
            files,
            &OpenOptions {
                origin,
                ..OpenOptions::default()
            },
        )
    }
}

/// The identity and how many files, never the bytes.
impl fmt::Debug for PackWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PackWriter")
            .field("id", &self.manifest.identity())
            .field("files", &self.files.len())
            .finish_non_exhaustive()
    }
}

/// `manifest.json` of `manifest`, in the default style.
fn write_manifest(manifest: &Manifest) -> Result<Vec<u8>, EncodeError> {
    let mut out = Vec::new();
    manifest.write_json(&mut out, WriteStyle::default())?;
    Ok(out)
}

/// Why [`PackWriter::into_pack`] has no pack when the manifest cannot be written: an invalid
/// manifest, with one error at the member its format cannot hold.
// `OpenError` has no variant for a write failure, so a manifest that cannot be written is
// reported as an invalid manifest.
fn unwritable(manifest: &Manifest, error: EncodeError, origin: PackOrigin) -> OpenError {
    let at = match error {
        EncodeError::NotRepresentable { at, .. } => at,
        // `EncodeError` is non-exhaustive: a reason without a place is reported at the root.
        _ => JsonPath::new(),
    };
    let location = Place::root(RootId::MEMORY);
    let mut report = PackReport::new(location.clone(), manifest.pack_type(), origin);
    report.identity = Some(manifest.identity());
    report.original_name.clone_from(&manifest.header.name);
    report.original_version = manifest.header.version.to_string();
    report.diagnostics.push(Diagnostic::new(
        DiagnosticCode::from(&core_codes::INVALID_VALUE),
        Severity::Error,
        Phase::Validate,
        Location::new(None, Some(MANIFEST_PATH.clone()), at, None),
    ));
    OpenError::InvalidManifest {
        location,
        report: Box::new(report),
    }
}

/// The files of a pack root [`PackWriter::from_pack`] does not copy: the manifest, which the
/// writer holds as a value, and the two that describe the pack's bytes.
// 03: `contents.json` lists the pack's files (with their keys when encrypted);
// `signatures.json` signs them.
const NOT_COPIED: [&str; 3] = ["manifest.json", "contents.json", "signatures.json"];

/// The pack's files cannot be written in plain text: some are encrypted (decrypted or not),
/// or the pack came out of a whole-file encrypted archive.
fn is_encrypted(store: &Store) -> bool {
    if !matches!(store.encryption, Encryption::Plain) {
        return true;
    }
    #[cfg(feature = "zip")]
    if store
        .archive
        .as_ref()
        .is_some_and(|archive| archive.original.is_some() || archive.sealed)
    {
        return true;
    }
    false
}

/// `path` is a `__brarchive` archive (its bytes `bytes`) whose every entry `layer` serves:
/// the pack reads those files through the layer, so a copy holds them as loose files and has
/// no use for the archive. An archive the layer does not hold (malformed, or never read into
/// it) is a file like any other.
fn is_served(layer: &BrarchiveLayer, path: &PackPath, bytes: &Bytes) -> bool {
    let Some(dir) = archive_dir(path) else {
        return false;
    };
    let Ok(archive) = Brarchive::decode(bytes.clone()) else {
        return false;
    };
    archive
        .entries()
        .iter()
        .all(|entry| PackPath::new(&entry.name).is_ok_and(|name| layer.contains(&dir.join(&name))))
}

/// The bytes a copy holds for a file: a binary (`\x7FMCB`) document whose body is decoded as
/// the JSON text it stands for, anything else as it is. A binary document of no known layout
/// has no text and stays as its bytes; the pack that reads them says so of the file
/// (`UnsupportedFileFormat`), the copy's as the source's.
// The text is the one every loader of the stack reads the document through
// (`stack::walk::mcb_text`).
fn into_json_text(bytes: Bytes) -> Bytes {
    if !bytes.starts_with(&MCB_MAGIC) {
        return bytes;
    }
    match mcb::to_json_text(&bytes) {
        Ok(text) => Bytes::from(text),
        Err(_) => bytes,
    }
}

/// `path` if a file can be written there: not `manifest.json`, which is written from the
/// manifest, and not the pack root.
fn file_path(path: PackPath) -> Result<PackPath, WriteError> {
    if path.is_root() || path == *MANIFEST_PATH {
        return Err(WriteError::ReservedPath { path });
    }
    Ok(path)
}

/// Why a [`PackWriter`] could not write.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WriteError {
    /// The zip archive could not be written.
    #[error("cannot write the zip archive")]
    Zip {
        /// What the archive writer refused.
        #[source]
        source: StorageError,
    },
    /// The path is not one the writer keeps a file at: `manifest.json` is written from the
    /// manifest ([`PackWriter::manifest_mut`]), and the pack root is a folder.
    #[error("`{path}` is not a path the writer keeps a file at")]
    ReservedPath {
        /// The path asked for.
        path: PackPath,
    },
    /// [`PackWriter::add`] found its default path taken.
    #[error("a file already exists at `{path}`")]
    DuplicatePath {
        /// The default path, which holds a file.
        path: PackPath,
    },
    /// A value with no JSON spelling.
    #[error(transparent)]
    Encode(#[from] EncodeError),
    /// The source pack is encrypted; content keys are never written.
    #[error("the pack is encrypted")]
    Encrypted,
    /// A file could not be read from the pack (`NotResident` for a file that is indexed but
    /// not loaded).
    #[error(transparent)]
    Storage(#[from] StorageError),
}

#[cfg(test)]
mod tests;
