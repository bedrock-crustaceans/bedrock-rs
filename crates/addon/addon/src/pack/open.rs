//! Opening a pack from bytes already in memory, and reading its files: the container behind a
//! pack's file index (loose files, a zip archive, the brarchive layer, marketplace encryption).

use std::collections::BTreeMap;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::load::{Place, RootId};
use bedrock_addon_core::pack_identity::PackPath;
use bytes::Bytes;

use super::{FileSlot, OpenError, OpenOptions, Pack, PackFiles, PackInner};
use crate::load::{DirEntry, EntryKind, Input, Output, PackLoader, Sans};
use crate::storage::StorageError;
use crate::storage::brarchive::{Brarchive, BrarchiveLayer, archive_dir};
#[cfg(feature = "encrypted")]
use crate::storage::contents::ContentsTable;
#[cfg(feature = "zip")]
use crate::storage::zip::ZipIndex;

/// What stands behind a pack's file index besides its resident loose files.
#[derive(Clone, Debug, Default)]
pub(crate) struct Store {
    /// The archive the pack lives in, when it came from one.
    #[cfg(feature = "zip")]
    pub(crate) archive: Option<Archive>,
    /// The entries of the pack's `__brarchive/*.brarchive` files, read before the loose files.
    pub(crate) brarchive: BrarchiveLayer,
    /// Marketplace encryption.
    pub(crate) encryption: Encryption,
    /// The size of each indexed file, as the listing or the archive reported it.
    pub(crate) sizes: BTreeMap<PackPath, u64>,
}

/// A pack inside a zip archive.
#[cfg(feature = "zip")]
#[derive(Clone, Debug)]
pub(crate) struct Archive {
    /// The archive's index; its bytes stay resident.
    pub(crate) index: ZipIndex,
    /// The pack root inside the archive (the folder of the shallowest manifest).
    pub(crate) prefix: PackPath,
    /// The archive is the pack's own file (root prefix, not a member of a composite), so
    /// `Pack::to_archive` serves it unchanged.
    pub(crate) own: bool,
    /// The file as read when it was a whole-file encrypted archive: what `to_archive` serves,
    /// wherever the pack root sits in it (neither content keys nor plaintext are written).
    pub(crate) original: Option<Bytes>,
    /// The pack came out of a whole-file encrypted composite: it has no ciphertext of its own,
    /// and its files are plaintext, so `to_archive` refuses it.
    pub(crate) sealed: bool,
}

/// How a pack's files are encrypted.
#[derive(Clone, Debug, Default)]
pub(crate) enum Encryption {
    /// Plain files.
    #[default]
    Plain,
    /// `contents.json` decrypted: the assets it lists are decrypted with their own keys.
    #[cfg(feature = "encrypted")]
    Table(ContentsTable),
    /// `contents.json` is encrypted and was not read (no key, a wrong key, no `encrypted`
    /// feature): which files are encrypted is unknown, so every file but the plaintext ones
    /// fails with this error.
    Locked(LockReason),
}

/// Why an encrypted pack's files cannot be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LockReason {
    /// No key for the pack's identity.
    MissingKey,
    /// The key did not decrypt `contents.json` to a contents table.
    Decrypt,
}

/// The files a marketplace pack keeps in plaintext: `manifest.json` and the encrypted
/// `contents.json` itself.
// `manifest.json` identifies the pack, and without the decrypted `contents.json` table there is
// no way to know which other files are encrypted, so both stay readable without a key.
const PLAINTEXT: [&str; 2] = ["manifest.json", "contents.json"];

impl PackFiles {
    /// The files and the store behind them.
    pub(crate) fn with_store(entries: BTreeMap<PackPath, FileSlot>, store: Store) -> PackFiles {
        PackFiles { entries, store }
    }

    /// Whether the file's bytes are loaded (resident, in a resident archive or brarchive).
    pub(crate) fn is_resident(&self, path: &PackPath) -> bool {
        if self.store.brarchive.get(path).is_some() {
            return true;
        }
        match self.entries.get(path) {
            Some(FileSlot::Resident(_) | FileSlot::Archived) => true,
            Some(FileSlot::Indexed) => false,
            None => self.store.brarchive.contains(path),
        }
    }

    /// The file's bytes as the pack stores them: a brarchive entry, the resident loose bytes or
    /// the inflated archive entry, still encrypted.
    pub(crate) fn read_stored(&self, path: &PackPath) -> Result<Bytes, StorageError> {
        // The brarchive layer is consulted first; an empty entry falls through to the loose file.
        if let Some(bytes) = self.store.brarchive.get(path) {
            return Ok(bytes);
        }
        match self.entries.get(path) {
            Some(FileSlot::Resident(bytes)) => Ok(bytes.clone()),
            Some(FileSlot::Indexed) => Err(StorageError::NotResident),
            Some(FileSlot::Archived) => self.read_archived(path),
            None if self.store.brarchive.contains(path) => Ok(Bytes::new()),
            None => Err(StorageError::NotFound),
        }
    }

    /// The archive entry of `path`, inflated.
    #[cfg(feature = "zip")]
    fn read_archived(&self, path: &PackPath) -> Result<Bytes, StorageError> {
        let archive = self.store.archive.as_ref().ok_or(StorageError::NotFound)?;
        archive.index.read_path(&archive.prefix.join(path))
    }

    /// Without `zip` no pack has archived files.
    #[cfg(not(feature = "zip"))]
    #[allow(clippy::unused_self)]
    fn read_archived(&self, _path: &PackPath) -> Result<Bytes, StorageError> {
        Err(StorageError::NotFound)
    }

    /// The file's bytes, inflated and decrypted.
    pub(crate) fn read(&self, path: &PackPath) -> Result<Bytes, StorageError> {
        let stored = self.read_stored(path)?;
        self.decrypt(path, stored)
    }

    /// `stored` (the bytes of `path` as stored) decrypted when the pack's encryption says so.
    pub(crate) fn decrypt(&self, path: &PackPath, stored: Bytes) -> Result<Bytes, StorageError> {
        match &self.store.encryption {
            Encryption::Plain => Ok(stored),
            #[cfg(feature = "encrypted")]
            Encryption::Table(table) => match table.get(path) {
                Some(cipher) => crate::storage::crypt::decrypt_asset(&stored, cipher),
                None => Ok(stored),
            },
            Encryption::Locked(reason) => {
                if PLAINTEXT.contains(&path.as_str()) {
                    Ok(stored)
                } else {
                    Err(match reason {
                        LockReason::MissingKey => StorageError::MissingContentKey,
                        LockReason::Decrypt => StorageError::Decrypt,
                    })
                }
            }
        }
    }

    /// The store behind the index.
    pub(crate) fn store(&self) -> &Store {
        &self.store
    }

    /// The size of an indexed file, as listed.
    pub(crate) fn size(&self, path: &PackPath) -> Option<u64> {
        self.store.sizes.get(path).copied()
    }

    /// Every indexed file and its slot, bytewise (the brarchive layer's paths not included).
    pub(crate) fn slots(&self) -> impl Iterator<Item = (&PackPath, &FileSlot)> + '_ {
        self.entries.iter()
    }

    /// The paths a brarchive layer adds to the index, beside the loose ones.
    pub(crate) fn brarchive_paths(&self) -> impl Iterator<Item = &PackPath> + '_ {
        self.store
            .brarchive
            .paths()
            .filter(|path| !self.entries.contains_key(*path))
    }
}

/// Two bytewise-sorted path iterators merged into one, each path once.
pub(crate) struct Union<'a, A: Iterator<Item = &'a PackPath>, B: Iterator<Item = &'a PackPath>> {
    a: std::iter::Peekable<A>,
    b: std::iter::Peekable<B>,
}

impl<'a, A: Iterator<Item = &'a PackPath>, B: Iterator<Item = &'a PackPath>> Union<'a, A, B> {
    pub(crate) fn new(a: A, b: B) -> Self {
        Union {
            a: a.peekable(),
            b: b.peekable(),
        }
    }
}

impl<'a, A: Iterator<Item = &'a PackPath>, B: Iterator<Item = &'a PackPath>> Iterator
    for Union<'a, A, B>
{
    type Item = &'a PackPath;

    fn next(&mut self) -> Option<&'a PackPath> {
        match (self.a.peek(), self.b.peek()) {
            (Some(a), Some(b)) if b < a => self.b.next(),
            (Some(a), Some(b)) if a == b => {
                self.b.next();
                self.a.next()
            }
            (Some(_), _) => self.a.next(),
            (None, _) => self.b.next(),
        }
    }
}

impl PackFiles {
    /// The brarchive layer of the files: every `[<prefix>/]__brarchive/<dir>.brarchive` the
    /// index holds, read (and decrypted) through `self`, in path order; a later archive wins for
    /// the same path. A malformed archive is left out with a `ReadError` finding
    /// (`InvalidData`).
    pub(crate) fn brarchive_layer(&self) -> (BrarchiveLayer, Diagnostics) {
        let mut layer = BrarchiveLayer::default();
        let mut findings = Diagnostics::default();
        let archives: Vec<(PackPath, PackPath)> = self
            .entries
            .keys()
            .filter_map(|path| archive_dir(path).map(|dir| (path.clone(), dir)))
            .collect();
        for (path, dir) in archives {
            let decoded = self
                .read(&path)
                .and_then(Brarchive::decode)
                .and_then(|archive| layer.add(&dir, &archive));
            if let Err(error) = decoded {
                findings.push(storage_finding(&path, &error));
            }
        }
        (layer, findings)
    }
}

/// The finding for a file that could not be read from its pack: an encrypted file without its
/// key is an unsupported format, anything else a read error (`InvalidData`).
// 03.
pub(crate) fn storage_finding(path: &PackPath, error: &StorageError) -> Diagnostic {
    let code = match error {
        StorageError::MissingContentKey | StorageError::Decrypt | StorageError::Mcb { .. } => {
            DiagnosticCode::UnsupportedFileFormat
        }
        StorageError::NotResident => DiagnosticCode::FileNotLoaded,
        _ => DiagnosticCode::ReadError {
            kind: std::io::ErrorKind::InvalidData,
            text: None,
        },
    };
    Diagnostic::new(
        code,
        Severity::Error,
        Phase::Read,
        Location::new(None, Some(path.clone()), JsonPath::new(), None),
    )
}

impl Pack {
    /// A pack from files already in memory, paths relative to the pack root; runs a
    /// `PackLoader` with jobs inline. From a path: `bedrock_addon_std::load_dir` /
    /// `load_archive`.
    ///
    /// Fails only when there is no pack to return: unsupported container, no manifest, or a
    /// manifest whose errors leave no identity. Every other problem is a diagnostic in
    /// [`Pack::report`].
    ///
    /// # Examples
    ///
    /// ```
    /// use bedrock_addon::pack::{OpenOptions, Pack};
    /// use bedrock_addon::storage::PackPath;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manifest = br#"{
    /// #   "format_version": 2,
    /// #   "header": { "name": "Ruby", "description": "", "uuid": "00000000-0000-4000-8000-000000000001",
    /// #               "version": [1, 0, 0], "min_engine_version": [1, 21, 0] },
    /// #   "modules": [{ "type": "resources", "uuid": "00000000-0000-4000-8000-000000000002", "version": [1, 0, 0] }]
    /// # }"#;
    /// let pack = Pack::from_files(
    ///     [
    ///         (PackPath::new("manifest.json")?, &manifest[..]),
    ///         (PackPath::new("textures/ruby.png")?, &b"png"[..]),
    ///     ],
    ///     &OpenOptions::default(),
    /// )?;
    /// assert_eq!(pack.read(&PackPath::new("textures/ruby.png")?)?.as_ref(), b"png");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`OpenError`] when the files do not make a pack: no manifest, several manifests at the same
    /// depth, or a path that breaks the pack-path rules.
    pub fn from_files<B: Into<Bytes>>(
        files: impl IntoIterator<Item = (PackPath, B)>,
        options: &OpenOptions,
    ) -> Result<Pack, OpenError> {
        let files: BTreeMap<PackPath, Bytes> = files
            .into_iter()
            .map(|(path, bytes)| (path, bytes.into()))
            .collect();
        let root = Place::root(RootId::MEMORY);
        let mut loader = PackLoader::directory(RootId::MEMORY, options.clone());
        let pack = drive_in_memory(&mut loader, |location| {
            if *location == root {
                Answer::Dir(
                    files
                        .iter()
                        .map(|(path, bytes)| DirEntry {
                            name: path.as_str().into(),
                            lossy: false,
                            kind: EntryKind::File {
                                len: bytes.len() as u64,
                            },
                        })
                        .collect(),
                )
            } else {
                Answer::File(files.get(&location.path).cloned())
            }
        })?;
        // Every file given stays resident; the loader read the manifest and `contents.json` only.
        // Paths are relative to the pack root, the subdirectory holding the manifest if any.
        let prefix = pack.location().path.clone();
        Ok(pack.with_resident(
            files
                .into_iter()
                .filter_map(|(path, bytes)| Some((crate::load::strip(&path, &prefix)?, bytes))),
        ))
    }

    /// One archive already in memory (`.mcpack`, `.zip`, `.mctemplate`), detected by magic
    /// bytes, not by extension.
    ///
    /// # Errors
    ///
    /// [`OpenError`] when the bytes are not a readable archive, or the archive holds no usable
    /// pack (no manifest, several manifests, or a path the pack-path rules reject).
    #[cfg(feature = "zip")]
    #[cfg_attr(docsrs, doc(cfg(feature = "zip")))]
    pub fn from_archive(bytes: impl Into<Bytes>, options: &OpenOptions) -> Result<Pack, OpenError> {
        let bytes: Bytes = bytes.into();
        let mut loader =
            PackLoader::file(RootId::MEMORY, Some(bytes.len() as u64), options.clone());
        drive_in_memory(&mut loader, |_| Answer::File(Some(bytes.clone())))
    }

    /// Reads one file; paths use `/`, are case-sensitive and must not contain `.` / `..`.
    /// Inflates or decrypts on the calling thread; `StorageError::NotResident` when the file
    /// is indexed but not loaded.
    ///
    /// # Errors
    ///
    /// [`StorageError`] when the file is not in the pack, is indexed but not loaded
    /// (`NotResident`), or cannot be inflated or decrypted.
    pub fn read(&self, path: &PackPath) -> Result<Bytes, StorageError> {
        self.inner.files.read(path)
    }

    /// The pack as zip bytes (RP chunk serving): the original archive, else a zip of its
    /// files (all must be resident, `Residency::All`).
    ///
    /// # Examples
    ///
    /// ```
    /// use bedrock_addon::pack::{OpenOptions, Pack};
    /// use bedrock_addon::storage::PackPath;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manifest = br#"{
    /// #   "format_version": 2,
    /// #   "header": { "name": "Ruby", "description": "", "uuid": "00000000-0000-4000-8000-000000000001",
    /// #               "version": [1, 0, 0], "min_engine_version": [1, 21, 0] },
    /// #   "modules": [{ "type": "resources", "uuid": "00000000-0000-4000-8000-000000000002", "version": [1, 0, 0] }]
    /// # }"#;
    /// let options = OpenOptions::default();
    /// let pack = Pack::from_files([(PackPath::new("manifest.json")?, &manifest[..])], &options)?;
    /// let archive = pack.to_archive()?;
    /// let again = Pack::from_archive(archive, &options)?;
    /// assert_eq!(again.id(), pack.id());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`WriteError`](super::WriteError) when the pack has files that are not loaded, or the archive
    /// writer refuses them (`Zip`).
    #[cfg(feature = "zip")]
    #[cfg_attr(docsrs, doc(cfg(feature = "zip")))]
    pub fn to_archive(&self) -> Result<Bytes, super::WriteError> {
        let files = &self.inner.files;
        if let Some(archive) = &files.store.archive {
            // The original archive is served as given. A whole-file encrypted one is served as read
            // even when its pack root is a folder: re-zipping would write its plaintext.
            if let Some(original) = &archive.original {
                return Ok(original.clone());
            }
            if archive.sealed {
                return Err(super::WriteError::Encrypted);
            }
            if archive.own {
                return Ok(archive.index.bytes().clone());
            }
        }
        // The files as stored (still encrypted: content keys are never written), each its own
        // bytes: the brarchive layer is read-through for reads, not part of the files.
        let stored = files
            .entries
            .iter()
            .map(|(path, slot)| {
                let bytes = match slot {
                    FileSlot::Resident(bytes) => Ok(bytes.clone()),
                    FileSlot::Archived => files.read_archived(path),
                    FileSlot::Indexed => Err(StorageError::NotResident),
                };
                bytes.map(|bytes| (path, bytes))
            })
            .collect::<Result<Vec<_>, _>>()?;
        crate::storage::zip::encode(stored.iter().map(|(path, bytes)| (*path, &bytes[..])))
            .map_err(|source| super::WriteError::Zip { source })
    }

    /// The same pack (one `Arc`), not merely an equal one.
    pub(crate) fn same(&self, other: &Pack) -> bool {
        std::sync::Arc::ptr_eq(&self.inner, &other.inner)
    }

    /// The pack's index and store.
    pub(crate) fn files(&self) -> &PackFiles {
        &self.inner.files
    }

    /// This pack with `layer` as its brarchive layer and `findings` added to its report.
    pub(crate) fn with_layer(&self, layer: BrarchiveLayer, findings: Diagnostics) -> Pack {
        let inner = &self.inner;
        let mut files = inner.files.clone();
        files.store.brarchive = layer;
        let mut report = inner.report.clone();
        report.diagnostics.extend(findings);
        Pack::from_inner(PackInner::new(
            inner.manifest.clone(),
            files,
            report,
            inner.location.clone(),
        ))
    }

    /// This pack with `resident` loaded into its index, the brarchive layer kept.
    pub(crate) fn with_loaded(
        &self,
        resident: impl IntoIterator<Item = (PackPath, Bytes)>,
    ) -> Pack {
        let inner = &self.inner;
        let mut files = inner.files.clone();
        for (path, bytes) in resident {
            if let Some(slot) = files.entries.get_mut(&path) {
                *slot = FileSlot::Resident(bytes);
            }
        }
        Pack::from_inner(PackInner::new(
            inner.manifest.clone(),
            files,
            inner.report.clone(),
            inner.location.clone(),
        ))
    }

    /// This pack with `resident` loaded into its index (paths it does not index are ignored)
    /// and its brarchive layer rebuilt from what is now resident; the layer's findings join the
    /// report.
    pub(crate) fn with_resident(
        &self,
        resident: impl IntoIterator<Item = (PackPath, Bytes)>,
    ) -> Pack {
        let inner = &self.inner;
        let mut entries = inner.files.entries.clone();
        for (path, bytes) in resident {
            if let Some(slot) = entries.get_mut(&path) {
                *slot = FileSlot::Resident(bytes);
            }
        }
        let mut files = PackFiles::with_store(entries, inner.files.store.clone());
        let mut report = inner.report.clone();
        if files.entries.keys().any(|path| archive_dir(path).is_some()) {
            let (layer, findings) = files.brarchive_layer();
            files.store.brarchive = layer;
            report.diagnostics.extend(findings);
        }
        Pack::from_inner(PackInner::new(
            inner.manifest.clone(),
            files,
            report,
            inner.location.clone(),
        ))
    }
}

/// How the in-memory driver answers a request.
enum Answer {
    /// A listing.
    Dir(Vec<DirEntry>),
    /// A file's bytes, or `None` for a missing one.
    File(Option<Bytes>),
}

/// Runs `loader` to its result, answering each request with `answer` and running every job
/// inline, in issue order.
fn drive_in_memory(
    loader: &mut PackLoader,
    answer: impl Fn(&Place) -> Answer,
) -> Result<Pack, OpenError> {
    loop {
        let Some(output) = loader.poll() else {
            // A loader that waits with nothing asked is a bug of this crate.
            return Err(OpenError::Internal {
                location: Place::root(RootId::MEMORY),
            });
        };
        let input = match output {
            Output::Done(result) => return result,
            Output::Job(job) => Input::JobDone(job.run()),
            Output::NeedKey(request) => Input::Key {
                identity: request.identity,
                key: None,
            },
            Output::Request(request) => {
                let location = match &request {
                    crate::load::Request::ListDir { location, .. }
                    | crate::load::Request::ReadFile { location, .. } => location.clone(),
                };
                let range = match &request {
                    crate::load::Request::ReadFile { range, .. } => Some(*range),
                    crate::load::Request::ListDir { .. } => None,
                };
                match answer(&location) {
                    Answer::Dir(entries) => Input::Dir { location, entries },
                    Answer::File(Some(bytes)) => Input::File {
                        bytes: prefix(bytes, range),
                        location,
                    },
                    Answer::File(None) => Input::Missing { location },
                }
            }
        };
        if loader.handle(input).is_err() {
            return Err(OpenError::Internal {
                location: Place::root(RootId::MEMORY),
            });
        }
    }
}

/// The part of `bytes` a read of `range` asks for.
fn prefix(bytes: Bytes, range: Option<crate::load::ReadRange>) -> Bytes {
    match range {
        Some(crate::load::ReadRange::Prefix(n)) => {
            let n = (n as usize).min(bytes.len());
            bytes.slice(..n)
        }
        _ => bytes,
    }
}

/// A pack hands its files to the persona piece loader itself:
/// `PersonaPiece::load_all(&pack)`.
// `skin` cannot name `Pack`, so the persona piece loader takes anything that can list and read
// files (`PieceSource`); `pack` may name a content module, as the accessor list does.
#[cfg(feature = "persona")]
impl crate::families::persona::PieceSource for Pack {
    fn id(&self) -> &bedrock_addon_core::pack_identity::PackIdVersion {
        Pack::id(self)
    }

    fn name(&self) -> &str {
        &self.facts().name
    }

    fn paths(&self) -> impl Iterator<Item = &PackPath> + '_ {
        Pack::paths(self)
    }

    fn read(&self, path: &PackPath) -> Result<Bytes, std::io::ErrorKind> {
        use std::io::ErrorKind;
        Pack::read(self, path).map_err(|error| match error {
            StorageError::NotFound => ErrorKind::NotFound,
            StorageError::MissingContentKey => ErrorKind::PermissionDenied,
            StorageError::NotResident => ErrorKind::WouldBlock,
            _ => ErrorKind::InvalidData,
        })
    }
}
