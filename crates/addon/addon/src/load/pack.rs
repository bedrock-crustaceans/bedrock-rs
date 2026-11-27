//! Opening packs: the per-candidate machine behind [`PackLoader`] and [`RepositoryLoader`]. A
//! candidate is a folder or a file; a file may be a composite (`.mcaddon`, `.mcworld`) that
//! holds several packs, which open from the outer bytes.
//!
//! [`RepositoryLoader`]: super::RepositoryLoader

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Arc;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Location, PackReport, Phase,
    Severity,
};
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::{ListDepth, Place, ReadRange, RootId};
use bedrock_addon_core::pack_identity::{PackOrigin, PackPath, PackType};
use bytes::Bytes;

use super::Stage as ProgressStage;
use super::io::{Charge, Expect, Io};
use super::job::{JobDone, JobId, JobKind};
use super::{DirEntry, EntryKind, Input, KeyRequest, Loader, LoaderError, Output, Progress, Sans};
use crate::manifest::Manifest;
use crate::pack::open::{Encryption, LockReason, Store};
use crate::pack::stack::Erased;
use crate::pack::{FileSlot, OpenError, OpenOptions, Pack, PackFiles, PackInner};
use crate::storage::brarchive::archive_dir;
use crate::storage::contents::ContentsJson;
use crate::storage::magic::{FileType, PREFIX_LEN, detect};
use crate::storage::{ContentIdentity, ContentKey, StorageError};

/// `manifest.json`.
const MANIFEST: &str = "manifest.json";
/// `contents.json`.
const CONTENTS: &str = "contents.json";
/// The prefix a file is first read with: magic detection looks at 256 bytes.
const PREFIX: u32 = 256;
const _: () = assert!(PREFIX as usize == PREFIX_LEN);

/// Where a candidate sorts among the packs of a repository: its source, its rank in the
/// source's sorted listing, and its path inside a composite: registration follows source order,
/// then each source's entries sorted bytewise.
pub(crate) type UnitKey = (usize, usize, PackPath);

/// The packs of several candidates being opened.
#[derive(Default)]
pub(crate) struct Opener {
    units: Vec<Unit>,
    /// The unit each outstanding request belongs to.
    /// (Two candidates at one place, e.g. two sources on one root, wait in issue order.)
    places: HashMap<Place, VecDeque<usize>>,
    /// The unit each outstanding job belongs to.
    jobs: HashMap<JobId, usize>,
    /// Units waiting for a key, by identity.
    waiting_keys: HashMap<ContentIdentity, Vec<usize>>,
    /// Keys the caller answered, by identity: one answer serves every pack.
    answered_keys: HashMap<ContentIdentity, Option<ContentKey>>,
}

/// One candidate (or one pack of a composite) on its way to a pack.
#[cfg_attr(not(feature = "zip"), allow(dead_code))]
struct Unit {
    key: UnitKey,
    origin: PackOrigin,
    options: Arc<OpenOptions>,
    /// The candidate: a folder, or an archive file (or a member of one).
    location: Place,
    /// The candidate may hold several packs (a repository's archive).
    composite: bool,
    /// Archive nesting depth: 1 for a file of a source.
    depth: u8,
    /// The candidate is an archive file of its own (not a folder inside one).
    archive_file: bool,
    /// Whether the candidate lies inside a decrypted whole-file encrypted composite.
    seal: Seal,
    step: Step,
    /// The pack root: the folder holding the manifest.
    root: Place,
    files: BTreeMap<PackPath, FileSlot>,
    sizes: BTreeMap<PackPath, u64>,
    store: Store,
    findings: Diagnostics,
    manifest: Option<Document<Manifest>>,
    /// An archive pack's `contents.json`, inflated by the archive's `Index` job.
    contents: Option<Result<Bytes, StorageError>>,
    /// `contents.json` is behind the encryption header.
    contents_encrypted: bool,
    /// `Some` once the unit is finished: a pack, or why there is none. A composite that only
    /// spawned other units finishes with `None` inside.
    result: Option<Finished>,
}

/// Whether a unit came out of a whole-file encrypted composite that was decrypted.
#[cfg_attr(not(feature = "zip"), allow(dead_code))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Seal {
    /// Plain bytes, or a whole-file encrypted archive of its own (its ciphertext is kept).
    #[default]
    Open,
    /// Inside a decrypted composite: no ciphertext of its own, so `Pack::to_archive` refuses
    /// it rather than write its plaintext.
    Sealed,
}

/// A finished unit: a pack, why there is none, or nothing (a composite that only spawned the
/// units of its members).
struct Finished(Option<Result<Pack, OpenError>>);

/// Where a unit is.
#[cfg_attr(not(feature = "zip"), allow(dead_code))]
enum Step {
    /// Waiting for the folder's listing.
    Listing,
    /// Waiting for the manifest's bytes.
    ReadingManifest,
    /// The `Manifest` job is out.
    ParsingManifest,
    /// Waiting for `contents.json`.
    ReadingContents,
    /// Waiting for the key of `contents.json`.
    WaitingKey(Bytes),
    /// The `Index` job of `contents.json` is out.
    ReadingTable,
    /// The `Index` job of the brarchive layer is out.
    IndexingBrarchives,
    /// Waiting for the first 256 bytes of a file.
    Prefix { len_hint: Option<u64> },
    /// Waiting for a whole archive; a key when it is behind the encryption header.
    Whole { key: Option<ContentKey> },
    /// The archive's `Index` job is out.
    IndexingArchive,
    /// Finished.
    Done,
}

/// The outcome of the `Manifest` job.
type ManifestOutcome = Result<Document<Manifest>, Rejected>;

/// The outcome of the `Index` job of `contents.json`.
struct ContentsOutcome {
    encryption: Encryption,
    findings: Diagnostics,
}

impl Opener {
    /// Opens the pack folder at `location`: first request `ListDir { location, Recursive }`.
    pub(crate) fn add_directory(
        &mut self,
        io: &mut Io,
        key: UnitKey,
        location: Place,
        options: Arc<OpenOptions>,
    ) {
        let index = self.push(key, location.clone(), options, false, 1, Step::Listing);
        self.units[index].archive_file = false;
        self.places
            .entry(location.clone())
            .or_default()
            .push_back(index);
        io.list(location, ListDepth::Recursive);
    }

    /// Opens the file at `location`, detected by magic bytes: first request `Prefix(256)`.
    /// A `composite` file may hold several packs.
    pub(crate) fn add_file(
        &mut self,
        io: &mut Io,
        key: UnitKey,
        location: Place,
        len_hint: Option<u64>,
        options: Arc<OpenOptions>,
        composite: bool,
    ) {
        let index = self.push(
            key,
            location.clone(),
            options,
            composite,
            1,
            Step::Prefix { len_hint },
        );
        self.places
            .entry(location.clone())
            .or_default()
            .push_back(index);
        io.read(
            location,
            ReadRange::Prefix(PREFIX),
            len_hint,
            Charge::Release,
        );
    }

    fn push(
        &mut self,
        key: UnitKey,
        location: Place,
        options: Arc<OpenOptions>,
        composite: bool,
        depth: u8,
        step: Step,
    ) -> usize {
        self.units.push(Unit {
            key,
            origin: options.origin,
            options,
            root: location.clone(),
            location,
            composite,
            depth,
            archive_file: true,
            seal: Seal::Open,
            step,
            files: BTreeMap::new(),
            sizes: BTreeMap::new(),
            store: Store::default(),
            findings: Diagnostics::default(),
            manifest: None,
            contents: None,
            contents_encrypted: false,
            result: None,
        });
        self.units.len() - 1
    }

    /// Every unit is finished.
    pub(crate) fn is_finished(&self) -> bool {
        self.units.iter().all(|unit| unit.result.is_some())
    }

    /// The results, sorted by unit key: `(key, origin, candidate location, result)`.
    pub(crate) fn take_results(
        &mut self,
    ) -> Vec<(UnitKey, PackOrigin, Place, Result<Pack, OpenError>)> {
        let mut results: Vec<_> = self
            .units
            .drain(..)
            .filter_map(|unit| {
                let result = unit.result?.0?;
                Some((unit.key, unit.origin, unit.location, result))
            })
            .collect();
        results.sort_by(|a, b| a.0.cmp(&b.0));
        results
    }

    /// Feeds an input this opener owns (checked by the caller with `owns_*`).
    pub(crate) fn handle(&mut self, io: &mut Io, input: Input) -> Result<(), LoaderError> {
        match input {
            Input::Dir { location, entries } => {
                io.check_answer(&location, Expect::List)?;
                let unit = self.unit_at(&location)?;
                io.answer(&location, 0);
                self.unplace(&location);
                self.on_listing(io, unit, entries);
            }
            Input::File { location, bytes } => {
                io.check_answer(&location, Expect::Read)?;
                let unit = self.unit_at(&location)?;
                io.answer(&location, bytes.len() as u64);
                self.unplace(&location);
                self.on_file(io, unit, &location, bytes);
            }
            Input::Missing { location } => {
                io.check_answer(&location, Expect::Any)?;
                let unit = self.unit_at(&location)?;
                io.answer(&location, 0);
                self.unplace(&location);
                self.on_unreadable(io, unit, &location, std::io::ErrorKind::NotFound);
            }
            Input::Failed { location, kind } => {
                io.check_answer(&location, Expect::Any)?;
                let unit = self.unit_at(&location)?;
                io.answer(&location, 0);
                self.unplace(&location);
                self.on_unreadable(io, unit, &location, kind);
            }
            Input::Key { identity, key } => {
                io.check_key(&identity)?;
                io.answer_key(&identity);
                self.answered_keys.insert(identity.clone(), key.clone());
                for unit in self.waiting_keys.remove(&identity).unwrap_or_default() {
                    self.on_key(io, unit, key.clone());
                }
            }
            Input::JobDone(done) => {
                let kind = io.check_job(done.id())?;
                let unit = self.unit_of_job(done.id())?;
                io.job_done(done.id());
                self.jobs.remove(&done.id());
                self.on_job(io, unit, kind, Some(done));
            }
            Input::JobPanicked(id) => {
                let kind = io.check_job(id)?;
                let unit = self.unit_of_job(id)?;
                io.job_done(id);
                self.jobs.remove(&id);
                self.on_job(io, unit, kind, None);
            }
        }
        Ok(())
    }

    /// The answered request at `location` is no unit's any more.
    fn unplace(&mut self, location: &Place) {
        if let Some(queue) = self.places.get_mut(location) {
            queue.pop_front();
            if queue.is_empty() {
                self.places.remove(location);
            }
        }
    }

    fn unit_at(&self, location: &Place) -> Result<usize, LoaderError> {
        self.places
            .get(location)
            .and_then(|queue| queue.front())
            .copied()
            .ok_or_else(|| LoaderError::Unrequested {
                location: location.clone(),
            })
    }

    fn unit_of_job(&self, id: JobId) -> Result<usize, LoaderError> {
        self.jobs
            .get(&id)
            .copied()
            .ok_or(LoaderError::UnknownJob { id })
    }

    /// Queues a job for `unit`.
    fn job(
        &mut self,
        io: &mut Io,
        unit: usize,
        kind: JobKind,
        weight: u64,
        work: impl FnOnce() -> Erased + Send + Sync + 'static,
    ) {
        let id = io.job(kind, weight, Box::new(move |_| work()));
        self.jobs.insert(id, unit);
    }

    /// Finishes `unit` with `result` and counts it.
    fn finish(&mut self, io: &mut Io, unit: usize, result: Option<Result<Pack, OpenError>>) {
        match &result {
            Some(Ok(pack)) => {
                io.progress.packs += 1;
                io.progress.count(pack.report().diagnostics());
            }
            Some(Err(_)) => io.progress.errors += 1,
            None => {}
        }
        let unit = &mut self.units[unit];
        unit.step = Step::Done;
        unit.result = Some(Finished(result));
    }

    /// The folder's listing: validate the names, find the manifest, read it.
    fn on_listing(&mut self, io: &mut Io, index: usize, entries: Vec<DirEntry>) {
        let unit = &mut self.units[index];
        let mut files = BTreeMap::new();
        let mut entries = entries;
        entries.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));
        for entry in entries {
            let EntryKind::File { len } = entry.kind else {
                continue;
            };
            // A lossy or rule-breaking name is skipped with `PathRejected`.
            match PackPath::new(&entry.name) {
                Ok(path) if !entry.lossy && !path.is_root() => {
                    files.insert(path, len);
                }
                _ => unit.findings.push(path_rejected(&entry.name)),
            }
        }
        let location = unit.location.clone();
        let prefix = match locate_manifest(&files) {
            Ok(prefix) => prefix,
            Err(error) => {
                let error = match error {
                    Locate::Missing => OpenError::NoManifest { location },
                    Locate::Several(count) => OpenError::MultipleManifests { location, count },
                };
                self.finish(io, index, Some(Err(error)));
                return;
            }
        };
        unit.root = location.join(&prefix);
        for (path, len) in files {
            let Some(relative) = strip(&path, &prefix) else {
                continue;
            };
            unit.sizes.insert(relative.clone(), len);
            unit.files.insert(relative, FileSlot::Indexed);
        }
        let manifest = PackPath::from_static(MANIFEST);
        let place = unit.root.join(&manifest);
        let len = unit.sizes.get(&manifest).copied();
        unit.step = Step::ReadingManifest;
        self.places
            .entry(place.clone())
            .or_default()
            .push_back(index);
        io.read(place, ReadRange::Whole, len, Charge::Release);
    }

    fn on_file(&mut self, io: &mut Io, index: usize, location: &Place, bytes: Bytes) {
        let step = std::mem::replace(&mut self.units[index].step, Step::Done);
        match step {
            Step::ReadingManifest => {
                self.units[index].files.insert(
                    PackPath::from_static(MANIFEST),
                    FileSlot::Resident(bytes.clone()),
                );
                self.parse_manifest(io, index, bytes);
            }
            Step::ReadingContents => self.on_contents(io, index, bytes),
            Step::Prefix { len_hint } => self.on_prefix(io, index, location, len_hint, &bytes),
            Step::Whole { key } => self.index_archive(io, index, bytes, key),
            other => self.units[index].step = other,
        }
    }

    fn on_unreadable(
        &mut self,
        io: &mut Io,
        index: usize,
        location: &Place,
        kind: std::io::ErrorKind,
    ) {
        let step = std::mem::replace(&mut self.units[index].step, Step::Done);
        if let Step::ReadingContents = step {
            // A file that cannot be read is a `ReadError`; the pack opens.
            let finding = read_error(&PackPath::from_static(CONTENTS), kind);
            self.units[index].findings.push(finding);
            self.index_brarchives(io, index);
        } else {
            let error = OpenError::Unreadable {
                location: location.clone(),
                kind,
            };
            self.finish(io, index, Some(Err(error)));
        }
    }

    fn parse_manifest(&mut self, io: &mut Io, index: usize, bytes: Bytes) {
        let unit = &mut self.units[index];
        unit.step = Step::ParsingManifest;
        let options = Arc::clone(&unit.options);
        let weight = bytes.len() as u64;
        self.job(io, index, JobKind::Manifest, weight, move || {
            let outcome: ManifestOutcome = Manifest::parse(
                &PackPath::from_static(MANIFEST),
                &bytes,
                &options.capabilities,
            );
            Box::new(outcome)
        });
    }

    fn on_job(&mut self, io: &mut Io, index: usize, kind: JobKind, done: Option<JobDone>) {
        let step = std::mem::replace(&mut self.units[index].step, Step::Done);
        match step {
            Step::ParsingManifest => match done.and_then(JobDone::take::<ManifestOutcome>) {
                Some(Ok(document)) => self.on_manifest(io, index, document),
                Some(Err(rejected)) => {
                    let unit = &self.units[index];
                    let mut report =
                        PackReport::new(unit.root.clone(), PackType::Invalid, unit.origin);
                    report.diagnostics = unit.findings.clone();
                    report.diagnostics.extend(rejected.diagnostics);
                    let error = OpenError::InvalidManifest {
                        location: unit.root.clone(),
                        report: Box::new(report),
                    };
                    self.finish(io, index, Some(Err(error)));
                }
                // A panicked `Manifest` job leaves no pack.
                None => {
                    let location = self.units[index].root.clone();
                    self.finish(io, index, Some(Err(OpenError::Internal { location })));
                }
            },
            Step::ReadingTable => {
                let unit = &mut self.units[index];
                if let Some(outcome) = done.and_then(JobDone::take::<ContentsOutcome>) {
                    unit.store.encryption = outcome.encryption;
                    unit.findings.extend(outcome.findings);
                } else {
                    if unit.contents_encrypted {
                        unit.store.encryption = Encryption::Locked(LockReason::Decrypt);
                    }
                    unit.findings
                        .push(internal_error(Some(PackPath::from_static(CONTENTS))));
                }
                self.index_brarchives(io, index);
            }
            Step::IndexingBrarchives => {
                let unit = &mut self.units[index];
                if let Some((layer, findings)) = done.and_then(JobDone::take::<BrarchiveOutcome>) {
                    unit.store.brarchive = layer;
                    unit.findings.extend(findings);
                } else {
                    // A panicked brarchive `Index` job leaves the layer absent.
                    let archives: Vec<PackPath> = unit
                        .files
                        .keys()
                        .filter(|path| archive_dir(path).is_some())
                        .cloned()
                        .collect();
                    for path in archives {
                        unit.findings.push(internal_error(Some(path)));
                    }
                }
                self.build(io, index);
            }
            Step::IndexingArchive => self.on_archive_index(io, index, done),
            other => {
                debug_assert!(false, "a {kind:?} job for a unit that waits for none");
                self.units[index].step = other;
            }
        }
    }

    fn on_manifest(&mut self, io: &mut Io, index: usize, document: Document<Manifest>) {
        let unit = &mut self.units[index];
        let contents = PackPath::from_static(CONTENTS);
        let slot = unit.files.get(&contents).cloned();
        unit.manifest = Some(document);
        match slot {
            None => self.index_brarchives(io, index),
            Some(FileSlot::Indexed) => {
                let place = unit.root.join(&contents);
                let len = unit.sizes.get(&contents).copied();
                unit.step = Step::ReadingContents;
                self.places
                    .entry(place.clone())
                    .or_default()
                    .push_back(index);
                io.read(place, ReadRange::Whole, len, Charge::Release);
            }
            Some(FileSlot::Resident(bytes)) => self.on_contents(io, index, bytes),
            Some(FileSlot::Archived) => match unit.contents.take() {
                Some(Ok(bytes)) => self.on_contents(io, index, bytes),
                Some(Err(error)) => {
                    unit.findings
                        .push(crate::pack::open::storage_finding(&contents, &error));
                    self.index_brarchives(io, index);
                }
                None => self.index_brarchives(io, index),
            },
        }
    }

    /// `contents.json`'s bytes: behind the encryption header it needs the pack key.
    fn on_contents(&mut self, io: &mut Io, index: usize, bytes: Bytes) {
        let unit = &mut self.units[index];
        let contents = PackPath::from_static(CONTENTS);
        if matches!(unit.files.get(&contents), Some(FileSlot::Indexed)) {
            unit.files
                .insert(contents, FileSlot::Resident(bytes.clone()));
        }
        match detect(&bytes) {
            FileType::Encrypted(identity) => {
                unit.contents_encrypted = true;
                if !cfg!(feature = "encrypted") {
                    // A rule of the crate: without `encrypted` no key is asked for; the table stays locked.
                    self.read_table(io, index, bytes, None);
                    return;
                }
                if let Some(key) = unit.options.keys.get(&identity).cloned() {
                    self.read_table(io, index, bytes, Some(key));
                    return;
                }
                if let Some(key) = self.answered_keys.get(&identity).cloned() {
                    self.read_table(io, index, bytes, key);
                    return;
                }
                let pack = unit.manifest.as_ref().map_or_else(
                    || {
                        bedrock_addon_core::pack_identity::PackIdVersion::new(
                            uuid::Uuid::nil(),
                            bedrock_addon_core::version::SemVersion::new(0, 0, 0),
                            PackType::Invalid,
                        )
                    },
                    |document| document.value().identity(),
                );
                let request = KeyRequest {
                    pack,
                    identity: identity.clone(),
                    location: unit.location.clone(),
                };
                unit.step = Step::WaitingKey(bytes);
                self.waiting_keys.entry(identity).or_default().push(index);
                // Issued once per identity; a second pack waits for the same
                // answer.
                io.ask_key(request);
            }
            _ => self.read_table(io, index, bytes, None),
        }
    }

    fn on_key(&mut self, io: &mut Io, index: usize, key: Option<ContentKey>) {
        let step = std::mem::replace(&mut self.units[index].step, Step::Done);
        match step {
            Step::WaitingKey(bytes) => self.read_table(io, index, bytes, key),
            other => self.units[index].step = other,
        }
    }

    /// The `Index` job that decrypts and reads `contents.json`.
    fn read_table(&mut self, io: &mut Io, index: usize, bytes: Bytes, key: Option<ContentKey>) {
        self.units[index].step = Step::ReadingTable;
        let weight = bytes.len() as u64;
        self.job(io, index, JobKind::Index, weight, move || {
            Box::new(read_contents(&bytes, key.as_ref()))
        });
    }

    /// The `Index` job of the brarchive layer, when the pack's archives are resident (an
    /// archive pack); a folder pack's are read by the `StackLoader`.
    fn index_brarchives(&mut self, io: &mut Io, index: usize) {
        let unit = &mut self.units[index];
        let resident = unit
            .files
            .iter()
            .any(|(path, slot)| archive_dir(path).is_some() && !matches!(slot, FileSlot::Indexed));
        if !resident {
            self.build(io, index);
            return;
        }
        unit.step = Step::IndexingBrarchives;
        let files = PackFiles::with_store(unit.files.clone(), unit.store.clone());
        self.job(io, index, JobKind::Index, 0, move || {
            let outcome: BrarchiveOutcome = files.brarchive_layer();
            Box::new(outcome)
        });
    }

    /// The pack, from the unit's parts.
    fn build(&mut self, io: &mut Io, index: usize) {
        let unit = &mut self.units[index];
        let Some(document) = unit.manifest.take() else {
            return;
        };
        let manifest_findings = document.diagnostics().clone();
        let upgraded = document.version().upgraded;
        let manifest = document.into_value();
        let mut report = PackReport::new(unit.root.clone(), manifest.pack_type(), unit.origin);
        report.diagnostics = manifest_findings;
        report
            .diagnostics
            .extend(std::mem::take(&mut unit.findings));
        report.was_upgraded = upgraded;
        report.attempted_upgrade = upgraded;
        report.original_name.clone_from(&manifest.header.name);
        report.original_version = manifest.header.version.to_string();
        report.identity = Some(manifest.identity());
        report
            .required_base_game_version
            .clone_from(&manifest.header.base_game_version);
        unit.store.sizes = std::mem::take(&mut unit.sizes);
        let files = PackFiles::with_store(
            std::mem::take(&mut unit.files),
            std::mem::take(&mut unit.store),
        );
        let pack = Pack::from_inner(PackInner::new(manifest, files, report, unit.root.clone()));
        self.finish(io, index, Some(Ok(pack)));
    }

    /// The first bytes of a file: an archive is read whole, anything else is no pack.
    fn on_prefix(
        &mut self,
        io: &mut Io,
        index: usize,
        location: &Place,
        len_hint: Option<u64>,
        prefix: &[u8],
    ) {
        let unit = &self.units[index];
        let limits = *io.limits();
        let fail = |this: &mut Self, io: &mut Io, error: StorageError| {
            this.finish(io, index, Some(Err(OpenError::Storage(error))));
        };
        if cfg!(not(feature = "zip")) {
            // A rule of the crate: without `zip` no archive can be opened.
            return fail(
                self,
                io,
                StorageError::Zip {
                    fault: crate::storage::ZipFault::Unsupported,
                },
            );
        }
        if len_hint.is_some_and(|len| len > limits.max_archive_bytes) {
            return fail(
                self,
                io,
                StorageError::LimitExceeded {
                    limit: crate::storage::ZipLimit::ArchiveBytes,
                },
            );
        }
        let key = match detect(prefix) {
            FileType::Zip => None,
            // A whole-file encrypted archive decrypts with the pack key from the options; it
            // has no plaintext manifest, so there is no identity to ask a key request about.
            FileType::Encrypted(identity) if cfg!(feature = "encrypted") => {
                match unit.options.keys.get(&identity) {
                    Some(key) => Some(key.clone()),
                    None => return fail(self, io, StorageError::MissingContentKey),
                }
            }
            // Any other content is not an archive.
            _ => {
                return fail(
                    self,
                    io,
                    StorageError::Zip {
                        fault: crate::storage::ZipFault::Malformed,
                    },
                );
            }
        };
        self.units[index].step = Step::Whole { key };
        self.places
            .entry(location.clone())
            .or_default()
            .push_back(index);
        io.read(
            location.clone(),
            ReadRange::Whole,
            len_hint,
            Charge::Release,
        );
    }
}

/// The brarchive layer and its findings.
type BrarchiveOutcome = (crate::storage::brarchive::BrarchiveLayer, Diagnostics);

impl Progress {
    /// Counts the errors and warnings of `diagnostics` into the counters.
    pub(crate) fn count(&mut self, diagnostics: &Diagnostics) {
        for diagnostic in diagnostics {
            match diagnostic.severity {
                Severity::Error => self.errors += 1,
                Severity::Warning => self.warnings += 1,
                _ => {}
            }
        }
    }
}

/// `path` relative to `prefix`, if it lies below it.
pub(crate) fn strip(path: &PackPath, prefix: &PackPath) -> Option<PackPath> {
    if prefix.is_root() {
        return Some(path.clone());
    }
    let rest = path
        .as_str()
        .strip_prefix(prefix.as_str())?
        .strip_prefix('/')?;
    PackPath::new(rest).ok()
}

/// Why a folder has no manifest to use.
enum Locate {
    Missing,
    Several(usize),
}

/// The folder holding the manifest: the root, else the single immediate subdirectory holding
/// one.
fn locate_manifest(files: &BTreeMap<PackPath, u64>) -> Result<PackPath, Locate> {
    if files.contains_key(&PackPath::from_static(MANIFEST)) {
        return Ok(PackPath::ROOT);
    }
    let holders: Vec<PackPath> = files
        .keys()
        .filter_map(|path| {
            let (folder, name) = path.as_str().split_once('/')?;
            (name == MANIFEST)
                .then(|| PackPath::new(folder).ok())
                .flatten()
        })
        .collect();
    match holders.len() {
        0 => Err(Locate::Missing),
        1 => Ok(holders.into_iter().next().unwrap_or(PackPath::ROOT)),
        count => Err(Locate::Several(count)),
    }
}

/// Reads `contents.json`: decrypted with `key` behind the encryption header, else plain.
fn read_contents(bytes: &[u8], key: Option<&ContentKey>) -> ContentsOutcome {
    let path = PackPath::from_static(CONTENTS);
    let mut findings = Diagnostics::default();
    let warn = |code: Code| {
        Diagnostic::new(
            code,
            // The game has no content-log line for either case (a missing key is an internal
            // assertion there, and a table that does not parse has no message), so these are
            // the crate's own findings and informations.
            Severity::Info,
            Phase::Read,
            Location::new(None, Some(path.clone()), JsonPath::new(), None),
        )
    };
    let encryption = if let FileType::Encrypted(_) = detect(bytes) {
        match decrypt_table(bytes, key) {
            Ok((encryption, current)) => {
                if !current {
                    findings.push(version_finding(&path));
                }
                encryption
            }
            Err(reason) => {
                // Without a usable key every encrypted entry is `unsupported_file_format`.
                findings.push(warn(Code::from(&core_codes::UNSUPPORTED_FILE_FORMAT)));
                Encryption::Locked(reason)
            }
        }
    } else {
        match ContentsJson::parse(bytes) {
            Ok(contents) if !contents.is_current() => findings.push(version_finding(&path)),
            Ok(_) => {}
            Err(_) => findings.push(warn(bedrock_addon_core::diagnostic::builders::syntax(
                "contents.json is not a contents table",
            ))),
        }
        Encryption::Plain
    };
    ContentsOutcome {
        encryption,
        findings,
    }
}

/// The decrypted table and whether its `version` is 1. Without `encrypted` no table can be
/// read.
fn decrypt_table(bytes: &[u8], key: Option<&ContentKey>) -> Result<(Encryption, bool), LockReason> {
    #[cfg(feature = "encrypted")]
    {
        use crate::storage::contents::ContentsTable;
        match crate::storage::crypt::decrypt_contents(bytes, key) {
            Ok(contents) => Ok((
                Encryption::Table(ContentsTable::from_json(&contents)),
                contents.is_current(),
            )),
            Err(StorageError::MissingContentKey) => Err(LockReason::MissingKey),
            Err(_) => Err(LockReason::Decrypt),
        }
    }
    #[cfg(not(feature = "encrypted"))]
    {
        let _ = (bytes, key);
        Err(LockReason::MissingKey)
    }
}

/// `contents.json` with a `version` other than 1 (or none, which is how the game's own packs
/// ship it): the game has no message for it, so this finding is the crate's own and an
/// information. A warning would flag every pack the game ships.
fn version_finding(path: &PackPath) -> Diagnostic {
    let mut json_path = JsonPath::new();
    json_path.push("version");
    Diagnostic::new(
        Code::from(&core_codes::INVALID_VALUE),
        Severity::Info,
        Phase::Read,
        Location::new(None, Some(path.clone()), json_path, None),
    )
}

/// A listed entry skipped for its name.
pub(crate) fn path_rejected(name: &str) -> Diagnostic {
    Diagnostic::new(
        core_codes::PATH_REJECTED.with([name.to_owned()]),
        // A listing rule of the crate, with no line of the game's, so an information.
        Severity::Info,
        Phase::Read,
        Location::default(),
    )
}

/// A file the driver could not read.
pub(crate) fn read_error(path: &PackPath, kind: std::io::ErrorKind) -> Diagnostic {
    Diagnostic::new(
        core_codes::READ_ERROR.with([kind.to_string()]),
        Severity::Error,
        Phase::Read,
        Location::new(None, Some(path.clone()), JsonPath::new(), None),
    )
    .detail(DiagnosticDetail::Io(kind))
}

/// A job that panicked.
pub(crate) fn internal_error(path: Option<PackPath>) -> Diagnostic {
    Diagnostic::new(
        Code::from(&core_codes::INTERNAL_ERROR),
        Severity::Error,
        Phase::Read,
        Location::new(None, path, JsonPath::new(), None),
    )
}

/// The archive opener: the archive `Index` job (the central directory, the zip limits, the
/// manifest rule and, in a composite, the packs it holds) and the `Opener` steps around it.
/// Without `zip` the same two steps only fail.
#[cfg_attr(feature = "zip", path = "pack/archive.rs")]
#[cfg_attr(not(feature = "zip"), path = "pack/no_archive.rs")]
mod archive;

/// Loads one pack: a folder or a single archive. The vanilla rules apply: magic-byte detection,
/// the manifest at the root or in the single subdirectory holding one, the shallowest manifest
/// in an archive, subpacks, the brarchive layer and encryption.
///
/// Finishes with `Ok(Pack)`, whose files are **indexed** (every path and size known) with the
/// manifest, `contents.json` and archive bytes resident; or with `Err(OpenError)` when there
/// is no pack.
///
/// A loader is driven by answering its outputs until [`Output::Done`]; the example of [`Loader`]
/// shows the loop.
pub struct PackLoader {
    io: Io,
    opener: Opener,
    done: bool,
}

impl PackLoader {
    /// A pack folder at `root`. First request: `ListDir { Place::root(root), Recursive }`.
    pub fn directory(root: RootId, options: OpenOptions) -> PackLoader {
        let mut io = Io::new(options.limits);
        let mut opener = Opener::default();
        opener.add_directory(
            &mut io,
            (0, 0, PackPath::ROOT),
            Place::root(root),
            Arc::new(options),
        );
        PackLoader {
            io,
            opener,
            done: false,
        }
    }

    /// A single file at `root`, detected by magic bytes. First request:
    /// `ReadFile { range: Prefix(256) }`, then `Whole` if it is an archive.
    pub fn file(root: RootId, len_hint: Option<u64>, options: OpenOptions) -> PackLoader {
        let mut io = Io::new(options.limits);
        io.progress.stage = ProgressStage::Index;
        let mut opener = Opener::default();
        opener.add_file(
            &mut io,
            (0, 0, PackPath::ROOT),
            Place::root(root),
            len_hint,
            Arc::new(options),
            false,
        );
        PackLoader {
            io,
            opener,
            done: false,
        }
    }
}

impl Sans for PackLoader {
    type Input = Input;
    type Output = Output<Result<Pack, OpenError>>;
    type Error = LoaderError;

    fn handle(&mut self, input: Input) -> Result<(), LoaderError> {
        if self.done {
            return Err(LoaderError::Finished);
        }
        self.opener.handle(&mut self.io, input)
    }

    fn poll(&mut self) -> Option<Self::Output> {
        if let Some(ask) = self.io.poll() {
            return Some(ask.into_output());
        }
        if self.done || !self.opener.is_finished() || !self.io.is_idle() {
            return None;
        }
        self.done = true;
        self.io.progress.stage = ProgressStage::Finished;
        let result = self
            .opener
            .take_results()
            .into_iter()
            .next()
            .map(|(_, _, _, result)| result);
        Some(Output::Done(result.unwrap_or_else(|| {
            Err(OpenError::NoManifest {
                location: Place::root(RootId::MEMORY),
            })
        })))
    }
}

impl Loader<Result<Pack, OpenError>> for PackLoader {
    fn progress(&self) -> Progress {
        self.io.progress
    }
}

/// Prints the progress counters only.
impl std::fmt::Debug for PackLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackLoader")
            .field("progress", &Loader::progress(self))
            .finish_non_exhaustive()
    }
}
