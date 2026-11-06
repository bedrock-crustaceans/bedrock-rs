//! How loading happens, beside what loading depends on: the sans-IO loaders (`PackLoader`,
//! `RepositoryLoader`, `StackLoader`), the `Input` / `Output` vocabulary they speak, the jobs
//! they hand out, progress and limits. The core's `LoadOptions`, `LoadContext`, `RootId`,
//! `Place`, `ListDepth`, `ReadRange`, `Limits`, … are re-exported here, their one documented path.

pub use bedrock_addon_core::load::{
    AppData, BiomeRenderIdentifiers, BuiltIns, DocumentSource, Experiment, Experiments,
    JSON_DOCUMENT_WARNING_BYTES, JsonDocumentSize, Limits, ListDepth, ListedFile, LoadContext,
    LoadOptions, LoadRequirement, MAX_JSON_DOCUMENT_BYTES, NamedFiles, Place,
    RECOMMENDED_JOB_STACK, ReadRange, RootId, SaveName, json_document_size,
    json_document_size_against,
};

mod io;
mod job;
mod pack;
mod repository;
mod stack;
#[cfg(test)]
mod tests;

pub use crate::pack::stack::{Collection, CollectionSet};
pub use job::{Job, JobDone, JobId, JobKind};
pub use pack::PackLoader;
pub(crate) use pack::strip;
pub use repository::RepositoryLoader;
pub use stack::StackLoader;

use bedrock_addon_core::pack_identity::{PackIdVersion, PackOrigin};
use bytes::Bytes;

use crate::pack::Pack;
use crate::storage::{ContentIdentity, ContentKey};

/// A state machine that performs no I/O and starts no threads: it is fed inputs and yields
/// outputs. The same shape as `raknet::sans::Sans` and `nethernet::sans::Sans`.
pub trait Sans {
    /// What the caller feeds in: file contents, directory listings, keys, finished jobs.
    type Input;
    /// What the machine asks for or produces.
    type Output;
    /// A protocol violation by the caller.
    type Error;

    /// Feeds one input. `Err` means the input answers nothing the machine asked for; the
    /// machine's state is then unchanged, and the input is dropped.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when the input answers nothing the machine asked for (an unrequested
    /// location or job, a wrong kind of answer, or an input after the machine finished); the
    /// machine is left unchanged.
    fn handle(&mut self, input: Self::Input) -> Result<(), Self::Error>;

    /// The next output, or `None` while the machine waits for inputs it has asked for.
    fn poll(&mut self) -> Option<Self::Output>;
}

/// A loader: a [`Sans`] machine that speaks the shared [`Input`] / [`Output`] vocabulary and
/// finishes with a `T` ([`Output::Done`]).
///
/// Every loader is `Send + Sync + 'static`, so it can be a Bevy `Resource`, live in a tokio
/// task or cross threads between steps.
///
/// # Examples
///
/// Driving a [`PackLoader`] over files in memory: answer every request, run every job, feed
/// the key request an empty answer, and stop at [`Output::Done`]. [`RepositoryLoader`] and
/// [`StackLoader`] are driven the same way.
///
/// ```
/// use std::collections::BTreeMap;
///
/// use bedrock_addon::load::{
///     DirEntry, EntryKind, Input, Loader, Output, PackLoader, ReadRange, Request, RootId, Sans,
///     Stage,
/// };
/// use bedrock_addon::pack::OpenOptions;
/// use bytes::Bytes;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let manifest: &'static [u8] = br#"{
///   "format_version": 2,
///   "header": { "name": "Ruby", "description": "", "uuid": "00000000-0000-4000-8000-000000000001",
///               "version": [1, 0, 0], "min_engine_version": [1, 21, 0] },
///   "modules": [{ "type": "resources", "uuid": "00000000-0000-4000-8000-000000000002", "version": [1, 0, 0] }]
/// }"#;
/// let files: BTreeMap<&str, &'static [u8]> =
///     BTreeMap::from([("manifest.json", manifest), ("textures/ruby.png", b"png")]);
///
/// let mut loader = PackLoader::directory(RootId::MEMORY, OpenOptions::default());
/// let pack = loop {
///     let Some(output) = loader.poll() else {
///         return Err("the loader waits for an answer nobody asked for".into());
///     };
///     match output {
///         Output::Done(result) => break result?,
///         Output::Job(job) => loader.handle(Input::JobDone(job.run()))?,
///         Output::NeedKey(request) => loader.handle(Input::Key {
///             identity: request.identity,
///             key: None,
///         })?,
///         Output::Request(Request::ListDir { location, .. }) => {
///             let entries = files
///                 .iter()
///                 .map(|(name, bytes)| DirEntry {
///                     name: (*name).into(),
///                     lossy: false,
///                     kind: EntryKind::File { len: bytes.len() as u64 },
///                 })
///                 .collect();
///             loader.handle(Input::Dir { location, entries })?;
///         }
///         Output::Request(Request::ReadFile { location, range, .. }) => {
///             let input = match files.get(location.path.as_str()) {
///                 Some(bytes) => {
///                     let len = match range {
///                         ReadRange::Whole => bytes.len(),
///                         ReadRange::Prefix(n) => bytes.len().min(n as usize),
///                     };
///                     Input::File { location, bytes: Bytes::from_static(&bytes[..len]) }
///                 }
///                 None => Input::Missing { location },
///             };
///             loader.handle(input)?;
///         }
///     }
/// };
/// assert_eq!(pack.id().uuid.to_string(), "00000000-0000-4000-8000-000000000001");
/// assert_eq!(loader.progress().stage, Stage::Finished);
/// # Ok(())
/// # }
/// ```
pub trait Loader<T>:
    Sans<Input = Input, Output = Output<T>, Error = LoaderError> + Send + Sync + 'static
{
    /// Counters for a progress bar or a log line; cheap to call every tick.
    fn progress(&self) -> Progress;
}

/// A request for host I/O. Answer every request with exactly one [`Input`] for the same
/// `location`: `Dir` / `File`, `Missing` or `Failed`. A loader never has two requests for
/// the same `location` outstanding at once, so an answer is matched by its location alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// List the entries below `location`.
    ListDir {
        /// The directory.
        location: Place,
        /// How deep.
        depth: ListDepth,
    },
    /// Read `location`, or its first bytes. `len_hint` is the size the listing reported, for
    /// preallocation.
    ReadFile {
        /// The file.
        location: Place,
        /// How much.
        range: ReadRange,
        /// The size the listing reported.
        len_hint: Option<u64>,
    },
}

/// One entry of a listing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirEntry {
    /// Relative to the listed location, `/`-separated. The driver converts the host name and
    /// sets `lossy` when it had to (non-UTF-8); the loader validates it as a `PackPath`.
    pub name: Box<str>,
    /// The host name was not UTF-8.
    pub lossy: bool,
    /// File or directory.
    pub kind: EntryKind,
}

/// What a listing entry is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    /// A file of `len` bytes.
    File {
        /// Size in bytes.
        len: u64,
    },
    /// A directory.
    Dir,
}

/// The pack and key identity an [`Output::NeedKey`] asks about.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyRequest {
    /// The pack's identity, from its plaintext manifest.
    pub pack: PackIdVersion,
    /// The `ContentIdentity` string of the `contents.json` header.
    pub identity: ContentIdentity,
    /// Where the pack is, so a driver can look for a sidecar next to it.
    pub location: Place,
}

/// What a loader asks for or produces. Deliberately **not** `#[non_exhaustive]`: a new
/// output must break every driver at compile time, because a driver that ignores an output
/// would wait forever.
#[derive(Debug)]
pub enum Output<T> {
    /// Host I/O to perform; answer with an [`Input`].
    Request(Request),
    /// A content key the loader has not been given (`OpenOptions::keys`); answer with
    /// [`Input::Key`], with `key: None` if there is none. Issued at most once per
    /// `identity` per loader; the one answer serves every pack waiting for that identity.
    NeedKey(KeyRequest),
    /// CPU work to run anywhere, in any order; answer with [`Input::JobDone`].
    Job(Job),
    /// The result. Emitted only when no request, key or job of this loader is outstanding,
    /// so no answer can arrive after it. Polled exactly once; afterwards `poll` returns `None`
    /// and `handle` returns `LoaderError::Finished`.
    Done(T),
}

/// What the caller feeds into a loader.
#[derive(Debug)]
#[non_exhaustive]
pub enum Input {
    /// The answer to `ListDir`. The order is irrelevant: the loader sorts bytewise.
    Dir {
        /// The listed directory.
        location: Place,
        /// Its entries.
        entries: Vec<DirEntry>,
    },
    /// The answer to `ReadFile`: exactly the requested range.
    File {
        /// The read file.
        location: Place,
        /// Its bytes.
        bytes: Bytes,
    },
    /// The requested file or directory does not exist (any more).
    Missing {
        /// The requested location.
        location: Place,
    },
    /// The request failed for another reason; becomes a `Phase::Read` diagnostic.
    Failed {
        /// The requested location.
        location: Place,
        /// Why.
        kind: std::io::ErrorKind,
    },
    /// The answer to `NeedKey`. `ContentKey`'s `Debug` prints `ContentKey(..)`, never the key,
    /// so logging an `Input` does not leak key material.
    Key {
        /// The identity asked about.
        identity: ContentIdentity,
        /// The key, if the caller has one.
        key: Option<ContentKey>,
    },
    /// A job's result.
    JobDone(JobDone),
    /// A job whose `run` panicked; sent only by drivers that catch panics. The file or
    /// collection the job was for gets an `InternalError` diagnostic, and loading continues.
    JobPanicked(JobId),
}

/// Where packs come from. The origin decides `is_base_game_pack`; a host path is a `RootId`
/// the driver maps.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Source {
    /// A directory whose children are packs (`behavior_packs/`): each child folder or file is
    /// one pack candidate. The first request is `ListDir { root, Children }`.
    Directory {
        /// The directory.
        root: RootId,
        /// The origin of every pack found.
        origin: PackOrigin,
    },
    /// One file: a pack archive or a composite (`.mcaddon`, `.mcworld`), nesting as the
    /// repository rules say.
    Archive {
        /// The file.
        root: RootId,
        /// The origin of every pack found.
        origin: PackOrigin,
    },
    /// Packs already in memory, e.g. server-generated ones (`PackWriter::into_pack`).
    Packs {
        /// The packs.
        packs: Vec<Pack>,
        /// Their origin.
        origin: PackOrigin,
    },
}

/// What a stack load reads, parses and keeps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackOptions {
    /// The collections to parse now. Default: every collection whose feature is compiled in.
    /// Others parse lazily on first accessor call, from resident bytes.
    pub collections: CollectionSet,
    /// Which behaviour-pack bytes to keep.
    pub behavior: Residency,
    /// Which resource-pack bytes to keep.
    pub resource: Residency,
    /// Read and job windows for this load; the zip limits are those the packs
    /// were opened with.
    pub limits: Limits,
}

/// All collections, `Transient`, `Transient`, `Limits::default()`.
impl Default for StackOptions {
    fn default() -> Self {
        Self {
            collections: CollectionSet::ALL,
            behavior: Residency::Transient,
            resource: Residency::Transient,
            limits: Limits::default(),
        }
    }
}

/// Which file bytes a stacked pack keeps after loading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Residency {
    /// Read the files the selected collections walk, and release loose bytes once parsed.
    /// Archive packs keep their archive.
    #[default]
    Transient,
    /// Read every file and keep it: chunk serving (`Pack::to_archive`), `PackWriter::from_pack`.
    All,
}

const _: () = {
    const fn send_static<T: Send + 'static>() {}
    send_static::<Job>();
    send_static::<JobDone>();
    send_static::<Input>(); // answers are produced on I/O threads and sent to the loader's thread
};

/// A snapshot of a loader's counters. State, not an event: read it when you want to draw.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Progress {
    /// Where the loader is.
    pub stage: Stage,
    /// Requests issued so far.
    pub requests_issued: u32,
    /// Requests answered so far.
    pub requests_answered: u32,
    /// Bytes received so far.
    pub bytes_received: u64,
    /// Jobs issued so far.
    pub jobs_issued: u32,
    /// Jobs fed back so far.
    pub jobs_done: u32,
    /// Packs opened so far.
    pub packs: u32,
    /// Findings so far, by severity (the findings themselves are in the result).
    pub errors: u32,
    /// Warnings so far.
    pub warnings: u32,
}

/// Where a loader is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Stage {
    /// Listing sources and locating manifests.
    #[default]
    Discover,
    /// Indexing archives.
    Index,
    /// Reading content.
    Read,
    /// Parsing content.
    Parse,
    /// Combining collections.
    Combine,
    /// Done.
    Finished,
}

/// A caller fed a loader something it did not ask for. Pack, file and document failures are
/// never `LoaderError`; they are values in the result.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoaderError {
    /// An input for a location that was not requested or was already answered.
    #[error("{location}: input for a location that was not requested or was already answered")]
    Unrequested {
        /// The location.
        location: Place,
    },
    /// `File` for a `ListDir` request, or `Dir` for a `ReadFile` request.
    #[error("{location}: `File` for a `ListDir` request, or `Dir` for a `ReadFile` request")]
    WrongAnswer {
        /// The location.
        location: Place,
    },
    /// A `JobDone` for a job that is not outstanding.
    #[error("job {id:?} is not outstanding")]
    UnknownJob {
        /// The job.
        id: JobId,
    },
    /// A key for an identity that was not requested.
    #[error("key for {identity}, which was not requested")]
    UnrequestedKey {
        /// The identity.
        identity: ContentIdentity,
    },
    /// The loader has already produced its result.
    #[error("the loader has already produced its result")]
    Finished,
}
