//! The driver: a root table plus a strategy, and the loop that runs any loader to completion.

use std::fmt;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};

use bedrock_addon::load::{
    Input, Job, KeyRequest, Loader, Output, PackLoader, Place, RepositoryLoader, Request, RootId,
    Source, StackLoader, StackOptions,
};
use bedrock_addon::pack::{OpenError, OpenOptions, Pack, PackOrigin, PackRepository, StackSet};
use bedrock_addon::storage::ContentKey;

use crate::{LoadError, io, keys};

/// Where jobs and reads run.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Strategy {
    /// Everything on the calling thread, in issue order. No threads, fully reproducible.
    /// The calling thread needs no stack that grows with the files: one megabyte holds a
    /// world whose documents nest to the limit (`RECOMMENDED_JOB_STACK` says what the rest
    /// is for).
    Sequential,
    /// Jobs and reads on a rayon pool built with `RECOMMENDED_JOB_STACK`; `threads: None` =
    /// rayon's default count.
    #[cfg(feature = "rayon")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
    Rayon {
        /// The thread count; `None` = rayon's default.
        threads: Option<std::num::NonZeroUsize>,
    },
    /// Jobs and reads on a pool the caller built. Give its threads `RECOMMENDED_JOB_STACK`
    /// (`rayon::ThreadPoolBuilder::stack_size`); no file can overflow a smaller one by its
    /// nesting, down to one megabyte, but the recommended stack is the margin the loaders
    /// are tested with.
    #[cfg(feature = "rayon")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
    RayonPool(Arc<rayon::ThreadPool>),
}

/// `Rayon { threads: None }` with the `rayon` feature, else `Sequential`.
impl Default for Strategy {
    fn default() -> Self {
        #[cfg(feature = "rayon")]
        {
            Strategy::Rayon { threads: None }
        }
        #[cfg(not(feature = "rayon"))]
        {
            Strategy::Sequential
        }
    }
}

/// How the driver opens packs, runs work and finds keys.
///
/// The struct is `#[non_exhaustive]` (the `sidecar_keys` field exists only with the `encrypted`
/// feature, which another crate in the build can turn on), so build it from
/// [`Options::default`] and the `with_*` methods:
///
/// ```
/// use bedrock_addon_std::{Options, Strategy};
///
/// // One thread, the same order every run.
/// let options = Options::default().with_strategy(Strategy::Sequential);
/// # let _ = options;
/// ```
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct Options {
    /// How packs are opened.
    pub open: OpenOptions,
    /// Where jobs and reads run.
    pub strategy: Strategy,
    /// Asked for keys `open.keys` does not hold; `None` answers every `NeedKey` with no key.
    pub keys: Option<Arc<dyn keys::ContentKeyProvider>>,
    /// Opt-in (default `false`): before asking `keys`, look for a `<file>.key` sidecar next
    /// to the pack (`keys::read_sidecar_key`). The driver resolves the host path from its own
    /// root table, which grows with every `pack_dir` / `repository` call; a provider built
    /// before those calls could not see the new roots, so this is a flag and not a provider.
    #[cfg(feature = "encrypted")]
    #[cfg_attr(docsrs, doc(cfg(feature = "encrypted")))]
    pub sidecar_keys: bool,
}

impl Options {
    /// Sets [`Options::open`].
    #[must_use]
    pub fn with_open(mut self, open: OpenOptions) -> Self {
        self.open = open;
        self
    }

    /// Sets [`Options::strategy`].
    #[must_use]
    pub fn with_strategy(mut self, strategy: Strategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Sets [`Options::keys`] to `Some(keys)`.
    #[must_use]
    pub fn with_keys(mut self, keys: Arc<dyn keys::ContentKeyProvider>) -> Self {
        self.keys = Some(keys);
        self
    }

    /// Sets `Options::sidecar_keys`.
    #[cfg(feature = "encrypted")]
    #[cfg_attr(docsrs, doc(cfg(feature = "encrypted")))]
    #[must_use]
    pub fn with_sidecar_keys(mut self, sidecar_keys: bool) -> Self {
        self.sidecar_keys = sidecar_keys;
        self
    }
}

impl fmt::Debug for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Options");
        debug
            .field("open", &self.open)
            .field("strategy", &self.strategy)
            .field("keys", &self.keys.is_some());
        #[cfg(feature = "encrypted")]
        debug.field("sidecar_keys", &self.sidecar_keys);
        debug.finish()
    }
}

/// One packs directory or one archive file, with its origin.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DirSource {
    /// The host path.
    pub path: PathBuf,
    /// The origin of every pack found there.
    pub origin: PackOrigin,
    /// A packs directory or one archive.
    pub kind: DirSourceKind,
}

/// What a [`DirSource`] points at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DirSourceKind {
    /// A directory whose children are packs.
    Packs,
    /// One archive file (a pack or a composite).
    Archive,
}

impl DirSource {
    /// A directory whose children are packs (`behavior_packs/`, `development_resource_packs/`,
    /// …).
    pub fn packs(path: impl Into<PathBuf>, origin: PackOrigin) -> DirSource {
        DirSource {
            path: path.into(),
            origin,
            kind: DirSourceKind::Packs,
        }
    }

    /// One archive file: a pack (`.mcpack`, `.zip`) or a composite (`.mcaddon`, `.mcworld`).
    pub fn archive(path: impl Into<PathBuf>, origin: PackOrigin) -> DirSource {
        DirSource {
            path: path.into(),
            origin,
            kind: DirSourceKind::Archive,
        }
    }

    /// The loader's view: the same source with its host path replaced by a root.
    fn to_source(&self, root: RootId) -> Source {
        match self.kind {
            DirSourceKind::Packs => Source::Directory {
                root,
                origin: self.origin,
            },
            DirSourceKind::Archive => Source::Archive {
                root,
                origin: self.origin,
            },
        }
    }
}

/// A root table plus a strategy; keeps the roots between the repository and the stack load,
/// because the packs carry `Place`s into this table.
///
/// The driver keeps no other state between calls: reloading means calling again, and the
/// returned packs, repositories and stacks are values the caller keeps or drops.
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use bedrock_addon::load::{LoadOptions, StackOptions};
/// use bedrock_addon::pack::PackOrigin;
/// use bedrock_addon_std::{DirSource, Driver, Options, read_world_packs};
///
/// let mut driver = Driver::new(Options::default())?;
/// let repository = driver.repository([
///     DirSource::packs("behavior_packs", PackOrigin::World),
///     DirSource::packs("resource_packs", PackOrigin::World),
/// ])?;
/// let world = read_world_packs("worlds/Bedrock level")?;
/// let stacks = driver.load_stacks(
///     repository.stacks(&world, LoadOptions::default()),
///     StackOptions::default(),
/// )?;
/// # let _ = stacks;
/// # Ok(())
/// # }
/// ```
pub struct Driver {
    /// Host paths of every location a loader of this driver was given.
    roots: io::Roots,
    /// How to open packs, where to run work, where keys come from.
    options: Options,
    /// The pool of `Strategy::Rayon` / `Strategy::RayonPool`; `None` runs everything inline.
    #[cfg(feature = "rayon")]
    pool: Option<Arc<rayon::ThreadPool>>,
}

impl fmt::Debug for Driver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Driver");
        debug
            .field("roots", &self.roots)
            .field("options", &self.options);
        #[cfg(feature = "rayon")]
        debug.field("pool", &self.pool.is_some());
        debug.finish()
    }
}

impl Driver {
    /// A driver; builds the rayon pool if `options.strategy` asks for one, with
    /// `RECOMMENDED_JOB_STACK` per thread.
    ///
    /// # Errors
    ///
    /// `LoadError::Pool` when the rayon pool cannot be built.
    pub fn new(options: Options) -> Result<Driver, LoadError> {
        #[cfg(feature = "rayon")]
        let pool = match &options.strategy {
            Strategy::Sequential => None,
            Strategy::Rayon { threads } => Some(Arc::new(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(threads.map_or(0, std::num::NonZeroUsize::get))
                    .stack_size(bedrock_addon::load::RECOMMENDED_JOB_STACK)
                    .thread_name(|index| format!("bedrock-addon-{index}"))
                    .build()?,
            )),
            Strategy::RayonPool(pool) => Some(Arc::clone(pool)),
        };
        Ok(Driver {
            roots: io::Roots::new(),
            options,
            #[cfg(feature = "rayon")]
            pool,
        })
    }

    /// The root table.
    pub fn roots(&self) -> &io::Roots {
        &self.roots
    }

    /// Registers a host path and returns its root, for a loader built by hand and run with
    /// [`Driver::drive`] (`PackLoader::file(root, …)`, `Source::Directory { root, … }`).
    pub fn add_root(&mut self, path: impl Into<PathBuf>) -> RootId {
        self.roots.add(path)
    }

    /// Runs any loader to completion: answers its requests with [`io::respond`] and runs its
    /// jobs, both on the strategy's threads, and answers its key requests on the calling
    /// thread (a sidecar first when `Options::sidecar_keys` is set, then `Options::keys`).
    ///
    /// A job that panics is fed back as `Input::JobPanicked`, so the loader reports it and
    /// finishes. Under a rayon strategy the calling thread waits for answers; calling this
    /// from a thread of the same pool can therefore wait forever when the pool has no other
    /// thread free.
    ///
    /// # Errors
    ///
    /// `LoadError::Loader` when the loader reports a protocol violation, and
    /// `LoadError::Stalled` when it waits with no request or job outstanding.
    pub fn drive<T, L: Loader<T>>(&self, mut loader: L) -> Result<T, LoadError> {
        let (answers, received) = mpsc::channel();
        #[cfg(feature = "rayon")]
        if let Some(pool) = &self.pool {
            let roots = &self.roots;
            return pool.in_place_scope(|scope| {
                self.run(&mut loader, &received, |work| {
                    let answers = answers.clone();
                    scope.spawn(move |_| {
                        // The receiver is gone only if the loop already returned an error.
                        let _ = answers.send(work.perform(roots));
                    });
                })
            });
        }
        self.run(&mut loader, &received, |work| {
            Self::answer_inline(&answers, work.perform(&self.roots));
        })
    }

    fn answer_inline(answers: &Sender<Input>, input: Input) {
        // The receiver lives on the caller's stack for the whole loop.
        let _ = answers.send(input);
    }

    /// Drains every output, then waits for one answer, until the loader is done.
    // One channel per call: every read and job reports its answer here, and the loop feeds them
    // to the loader until it finishes, stalls or errors.
    fn run<T, L: Loader<T>>(
        &self,
        loader: &mut L,
        received: &Receiver<Input>,
        mut dispatch: impl FnMut(Work),
    ) -> Result<T, LoadError> {
        let mut in_flight = 0usize;
        loop {
            while let Some(output) = loader.poll() {
                match output {
                    Output::Request(request) => {
                        dispatch(Work::Read(request));
                        in_flight += 1;
                    }
                    Output::Job(job) => {
                        dispatch(Work::Job(job));
                        in_flight += 1;
                    }
                    Output::NeedKey(request) => {
                        let key = self.key_for(&request);
                        loader.handle(Input::Key {
                            identity: request.identity,
                            key,
                        })?;
                    }
                    Output::Done(value) => return Ok(value),
                }
            }
            if in_flight == 0 {
                return Err(LoadError::Stalled);
            }
            // Every dispatched piece of work sends exactly one answer, and the sender outlives
            // this loop, so `recv` only fails if that promise is broken.
            let input = received.recv().map_err(|_| LoadError::Stalled)?;
            in_flight -= 1;
            loader.handle(input)?;
        }
    }

    /// The answer to a `NeedKey`: the sidecar when opted in, then the provider.
    fn key_for(&self, request: &KeyRequest) -> Option<ContentKey> {
        #[cfg(feature = "encrypted")]
        if self.options.sidecar_keys {
            let sidecar = self
                .roots
                .host_path(&request.location)
                .and_then(|path| keys::read_sidecar_key(&path));
            if sidecar.is_some() {
                return sidecar;
            }
        }
        self.options
            .keys
            .as_ref()
            .and_then(|provider| provider.key_for(request))
    }

    /// Opens a pack folder (manifest at the root or in its single subdirectory holding one).
    ///
    /// # Errors
    ///
    /// `LoadError::Open` when the pack cannot be opened (no manifest, several, an unreadable or
    /// invalid one), and the errors of [`Driver::drive`].
    pub fn pack_dir(&mut self, path: impl AsRef<Path>) -> Result<Pack, LoadError> {
        let path = path.as_ref();
        let root = self.roots.add(path);
        let opened = self.drive(PackLoader::directory(root, self.options.open.clone()))?;
        opened.map_err(|source| self.open_error(path, source))
    }

    /// Opens a pack archive (`.mcpack`, `.zip`, `.mctemplate`), detected by magic bytes.
    ///
    /// # Errors
    ///
    /// `LoadError::Open` when the archive cannot be read or holds no valid pack, and the errors
    /// of [`Driver::drive`].
    #[cfg(feature = "zip")]
    #[cfg_attr(docsrs, doc(cfg(feature = "zip")))]
    pub fn pack_archive(&mut self, path: impl AsRef<Path>) -> Result<Pack, LoadError> {
        let path = path.as_ref();
        let root = self.roots.add(path);
        // The size the loader charges the read with; unknown when the file cannot be stat'ed,
        // in which case the read itself reports why.
        let len_hint = std::fs::metadata(path).ok().map(|metadata| metadata.len());
        let opened = self.drive(PackLoader::file(root, len_hint, self.options.open.clone()))?;
        opened.map_err(|source| self.open_error(path, source))
    }

    /// Sources in priority order (the first registration of a pack identity and origin wins).
    /// Reads manifests and archive indexes only; an unreadable source or an invalid pack is
    /// recorded in the repository's `source_reports` and `invalid`, not returned as an error.
    ///
    /// # Errors
    ///
    /// The errors of [`Driver::drive`].
    pub fn repository(
        &mut self,
        sources: impl IntoIterator<Item = DirSource>,
    ) -> Result<PackRepository, LoadError> {
        let sources = sources
            .into_iter()
            .map(|source| {
                let root = self.roots.add(source.path.clone());
                source.to_source(root)
            })
            .collect();
        self.drive(RepositoryLoader::new(sources, self.options.open.clone()))
    }

    /// Drives a `StackLoader` over stacks composed by `PackRepository::stacks`, whose packs
    /// this driver opened (their locations are roots of this table).
    ///
    /// # Errors
    ///
    /// The errors of [`Driver::drive`].
    pub fn load_stacks(
        &self,
        stacks: StackSet,
        options: StackOptions,
    ) -> Result<StackSet, LoadError> {
        self.drive(StackLoader::new(stacks, options))
    }

    /// `LoadError::Open` for `source`, at the host path of its location (the pack root when
    /// the error names none).
    fn open_error(&self, root: &Path, source: OpenError) -> LoadError {
        let path = open_error_location(&source)
            .and_then(|location| self.roots.host_path(location))
            .unwrap_or_else(|| root.to_path_buf());
        LoadError::Open { path, source }
    }
}

/// The location an `OpenError` names, if any.
fn open_error_location(error: &OpenError) -> Option<&Place> {
    match error {
        OpenError::Unreadable { location, .. }
        | OpenError::NoManifest { location }
        | OpenError::MultipleManifests { location, .. }
        | OpenError::InvalidManifest { location, .. }
        | OpenError::Internal { location } => Some(location),
        _ => None,
    }
}

/// One piece of work the loop hands to the strategy.
enum Work {
    Read(Request),
    Job(Job),
}

impl Work {
    /// Does the work and returns the input that answers it. Never panics: a job's panic is
    /// caught and reported as `Input::JobPanicked`.
    fn perform(self, roots: &io::Roots) -> Input {
        match self {
            Work::Read(request) => io::respond(roots, request),
            Work::Job(job) => {
                let id = job.id();
                panic::catch_unwind(AssertUnwindSafe(|| job.run()))
                    .map_or(Input::JobPanicked(id), Input::JobDone)
            }
        }
    }
}

/// Opens a pack folder with a fresh [`Driver`].
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use bedrock_addon_std::{Options, load_dir};
///
/// let pack = load_dir("behavior_packs/ruby_lamps", &Options::default())?;
/// println!("{:?}", pack.id());
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// `LoadError::Pool` when the rayon pool cannot be built, and the errors of
/// [`Driver::pack_dir`].
pub fn load_dir(path: impl AsRef<Path>, options: &Options) -> Result<Pack, LoadError> {
    Driver::new(options.clone())?.pack_dir(path)
}

/// Opens a pack archive with a fresh [`Driver`].
///
/// # Errors
///
/// `LoadError::Pool` when the rayon pool cannot be built, and the errors of
/// [`Driver::pack_archive`].
#[cfg(feature = "zip")]
#[cfg_attr(docsrs, doc(cfg(feature = "zip")))]
pub fn load_archive(path: impl AsRef<Path>, options: &Options) -> Result<Pack, LoadError> {
    Driver::new(options.clone())?.pack_archive(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_are_built_with_chaining_methods() {
        let options = Options::default()
            .with_open(OpenOptions {
                origin: PackOrigin::World,
                ..OpenOptions::default()
            })
            .with_strategy(Strategy::Sequential)
            .with_keys(Arc::new(bedrock_addon::storage::ContentKeys::default()));
        #[cfg(feature = "encrypted")]
        let options = options.with_sidecar_keys(true);
        assert_eq!(options.open.origin, PackOrigin::World);
        assert!(matches!(options.strategy, Strategy::Sequential));
        assert!(options.keys.is_some());
        #[cfg(feature = "encrypted")]
        assert!(options.sidecar_keys);
    }

    #[test]
    fn dir_sources_compare_and_hash_by_value() {
        use std::collections::HashSet;

        let packs = DirSource::packs("behavior_packs", PackOrigin::World);
        let archive = DirSource::archive("behavior_packs", PackOrigin::World);
        assert_eq!(packs, packs.clone());
        assert_ne!(packs, archive);
        let set: HashSet<_> = [packs.clone(), packs, archive].into_iter().collect();
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn options_and_driver_debug_name_their_fields_without_the_provider() {
        let options = Options::default();
        let text = format!("{options:?}");
        assert!(text.contains("keys: false"), "{text}");
        let driver = Driver::new(Options {
            strategy: Strategy::Sequential,
            ..Options::default()
        })
        .unwrap();
        let text = format!("{driver:?}");
        assert!(text.contains("roots") && text.contains("options"), "{text}");
    }
}
