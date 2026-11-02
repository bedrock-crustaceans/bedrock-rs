//! `RepositoryLoader`: every pack of several sources, opened by one shared `Opener` and
//! registered in source order.

use std::collections::HashMap;
use std::sync::Arc;

use bedrock_addon_core::load::{ListDepth, Place, RootId};
use bedrock_addon_core::pack_identity::{PackOrigin, PackPath};

use super::Stage;
use super::io::Io;
use super::pack::{Opener, UnitKey};
use super::{DirEntry, EntryKind, Input, Loader, LoaderError, Output, Progress, Sans, Source};
use crate::pack::{InvalidPack, OpenError, OpenOptions, Pack, PackRepository, SourceReport};
use crate::storage::StorageError;

/// Builds a [`PackRepository`] from sources: every pack under a packs directory, every pack
/// in a composite archive, and packs already in memory. Reads **manifests and indexes
/// only**; content is read later by [`StackLoader`](super::StackLoader) for the packs a stack
/// selects.
///
/// A loader is driven by answering its outputs until [`Output::Done`]; the example of [`Loader`]
/// shows the loop.
pub struct RepositoryLoader {
    io: Io,
    opener: Opener,
    sources: Vec<SourceState>,
    /// The packs directories whose listing is outstanding, by source.
    /// (Sources on one root share the listing.)
    listings: HashMap<Place, Vec<usize>>,
    /// Candidates refused before opening (a lossy or rule-breaking name).
    refused: Vec<(UnitKey, PackOrigin, Place, OpenError)>,
    done: bool,
}

/// One source and what it yielded besides its packs.
struct SourceState {
    location: Place,
    origin: PackOrigin,
    unreadable: Option<std::io::ErrorKind>,
    /// The packs of a `Source::Packs`.
    packs: Vec<Pack>,
    options: Arc<OpenOptions>,
    kind: SourceKind,
}

/// Which [`Source`] variant a [`SourceState`] came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceKind {
    /// A `Source::Directory`.
    Directory,
    /// A `Source::Archive`.
    Archive,
    /// A `Source::Packs`.
    Packs,
}

impl RepositoryLoader {
    /// Sources in priority order (first registration of a `(PackIdVersion, PackOrigin)` wins).
    #[must_use]
    pub fn new(sources: Vec<Source>, options: OpenOptions) -> RepositoryLoader {
        let mut io = Io::new(options.limits);
        let options = Arc::new(options);
        let mut opener = Opener::default();
        let mut listings = HashMap::new();
        let mut states = Vec::with_capacity(sources.len());
        for (index, source) in sources.into_iter().enumerate() {
            let (location, origin, packs, kind) = match source {
                Source::Directory { root, origin } => {
                    (Place::root(root), origin, Vec::new(), SourceKind::Directory)
                }
                Source::Archive { root, origin } => {
                    (Place::root(root), origin, Vec::new(), SourceKind::Archive)
                }
                Source::Packs { packs, origin } => (
                    Place::root(RootId::MEMORY),
                    origin,
                    packs,
                    SourceKind::Packs,
                ),
            };
            let source_options = Arc::new(OpenOptions {
                origin,
                ..OpenOptions::clone(&options)
            });
            match kind {
                SourceKind::Directory => {
                    let sharing: &mut Vec<usize> = listings.entry(location.clone()).or_default();
                    if sharing.is_empty() {
                        io.list(location.clone(), ListDepth::Children);
                    }
                    sharing.push(index);
                }
                SourceKind::Archive => opener.add_file(
                    &mut io,
                    (index, 0, PackPath::ROOT),
                    location.clone(),
                    None,
                    Arc::clone(&source_options),
                    true,
                ),
                SourceKind::Packs => {}
            }
            states.push(SourceState {
                location,
                origin,
                unreadable: None,
                packs,
                options: source_options,
                kind,
            });
        }
        RepositoryLoader {
            io,
            opener,
            sources: states,
            listings,
            refused: Vec::new(),
            done: false,
        }
    }

    /// The children of a packs directory: each folder or file is a candidate, in bytewise
    /// order; a lossy or rule-breaking name is refused.
    fn on_listing(&mut self, source: usize, mut entries: Vec<DirEntry>) {
        entries.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));
        let state = &self.sources[source];
        let (root, origin, options) = (
            state.location.clone(),
            state.origin,
            Arc::clone(&state.options),
        );
        for (rank, entry) in entries.into_iter().enumerate() {
            let key = (source, rank, PackPath::ROOT);
            let path = match PackPath::new(&entry.name) {
                Ok(path) if !path.is_root() => path,
                Ok(_) => continue,
                Err(reason) => {
                    // A rule-breaking name is skipped; in a repository it is an
                    // invalid pack, because a source report has no list of findings to hold
                    // a finding about a candidate.
                    let error = OpenError::Storage(StorageError::PathRejected {
                        path: entry.name.clone(),
                        reason,
                    });
                    self.refused.push((key, origin, root.clone(), error));
                    continue;
                }
            };
            let location = root.join(&path);
            if entry.lossy {
                // The host name is not UTF-8: the driver cannot be asked for it by this name.
                let error = OpenError::Unreadable {
                    location: location.clone(),
                    kind: std::io::ErrorKind::InvalidData,
                };
                self.refused.push((key, origin, location, error));
                continue;
            }
            match entry.kind {
                EntryKind::Dir => {
                    self.opener
                        .add_directory(&mut self.io, key, location, Arc::clone(&options));
                }
                EntryKind::File { len } => self.opener.add_file(
                    &mut self.io,
                    key,
                    location,
                    Some(len),
                    Arc::clone(&options),
                    true,
                ),
            }
        }
    }

    /// The repository: sources in order, each source's candidates in listing order.
    fn build(&mut self) -> PackRepository {
        let mut repository = PackRepository::new();
        let mut results: Vec<(UnitKey, PackOrigin, Place, Result<Pack, OpenError>)> =
            self.opener.take_results();
        results.extend(
            self.refused
                .drain(..)
                .map(|(key, origin, location, error)| (key, origin, location, Err(error))),
        );
        results.sort_by(|a, b| a.0.cmp(&b.0));
        let mut results = results.into_iter().peekable();
        for (index, state) in self.sources.iter_mut().enumerate() {
            let mut report = SourceReport::new(state.location.clone(), state.origin);
            report.unreadable = state.unreadable;
            for pack in std::mem::take(&mut state.packs) {
                report
                    .packs
                    .insert(pack.id().clone(), pack.report().clone());
                repository.add_pack(pack, state.origin);
            }
            while let Some((_, origin, location, result)) =
                results.next_if(|(key, ..)| key.0 == index)
            {
                // An archive source whose file cannot be read is an unreadable
                // root, as a packs directory's is, and not an invalid pack.
                if let Err(OpenError::Unreadable { location: at, kind }) = &result
                    && state.kind == SourceKind::Archive
                    && *at == state.location
                {
                    report.unreadable = Some(*kind);
                    continue;
                }
                match result {
                    Ok(pack) => {
                        report
                            .packs
                            .insert(pack.id().clone(), pack.report().clone());
                        repository.add_pack(pack, origin);
                    }
                    Err(error) => {
                        report.invalid.push(location.clone());
                        repository.push_invalid(InvalidPack::new(location, origin, error));
                    }
                }
            }
            repository.push_source_report(report);
        }
        repository
    }
}

impl Sans for RepositoryLoader {
    type Input = Input;
    type Output = Output<PackRepository>;
    type Error = LoaderError;

    fn handle(&mut self, input: Input) -> Result<(), LoaderError> {
        if self.done {
            return Err(LoaderError::Finished);
        }
        let location = match &input {
            Input::Dir { location, .. }
            | Input::File { location, .. }
            | Input::Missing { location }
            | Input::Failed { location, .. } => Some(location),
            _ => None,
        };
        // The answer is a source's listing when the place's outstanding request is one.
        let listing = location
            .filter(|location| self.io.is_listing(location))
            .cloned();
        let Some(location) = listing.filter(|location| self.listings.contains_key(location)) else {
            return self.opener.handle(&mut self.io, input);
        };
        if let Input::File { .. } = input {
            return Err(LoaderError::WrongAnswer { location });
        }
        self.io.answer(&location, 0);
        let sources = self.listings.remove(&location).unwrap_or_default();
        match input {
            Input::Dir { entries, .. } => {
                for source in sources {
                    self.on_listing(source, entries.clone());
                }
            }
            Input::Missing { .. } => {
                for source in sources {
                    self.sources[source].unreadable = Some(std::io::ErrorKind::NotFound);
                }
            }
            Input::Failed { kind, .. } => {
                for source in sources {
                    self.sources[source].unreadable = Some(kind);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn poll(&mut self) -> Option<Self::Output> {
        if let Some(ask) = self.io.poll() {
            return Some(ask.into_output());
        }
        if self.done
            || !self.listings.is_empty()
            || !self.opener.is_finished()
            || !self.io.is_idle()
        {
            return None;
        }
        self.done = true;
        let repository = self.build();
        self.io.progress.packs = u32::try_from(repository.packs().count()).unwrap_or(u32::MAX);
        self.io.progress.stage = Stage::Finished;
        Some(Output::Done(repository))
    }
}

impl Loader<PackRepository> for RepositoryLoader {
    fn progress(&self) -> Progress {
        self.io.progress
    }
}

/// Prints the progress counters only.
impl std::fmt::Debug for RepositoryLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RepositoryLoader")
            .field("progress", &Loader::progress(self))
            .finish_non_exhaustive()
    }
}
