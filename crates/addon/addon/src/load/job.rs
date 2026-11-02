//! Jobs: the CPU work a loader hands out, run by the caller anywhere and fed back.

use std::any::Any;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// One unit of CPU work a loader hands out. Owns everything it needs (shared `Bytes`, an
/// `Arc` of the load context and the Molang check), so it is `Send + 'static` and can run on
/// any thread, in any order, at any time before the loader is dropped.
#[must_use = "a job that is never run and fed back stalls the loader"]
pub struct Job {
    pub(crate) id: JobId,
    kind: JobKind,
    weight: u64,
    cancel: Cancel,
    work: Work,
}

/// The work of a job: a closure over owned inputs whose result the loader downcasts.
pub(crate) type Work = Box<dyn FnOnce(&Cancel) -> Erased + Send + Sync>;

/// A job's result or a parsed value, type-erased: the loader that issued the job downcasts it.
pub(crate) type Erased = Box<dyn Any + Send + Sync>;

impl Job {
    /// A job of `kind` over `weight` input bytes.
    pub(crate) fn new(id: JobId, kind: JobKind, weight: u64, cancel: Cancel, work: Work) -> Job {
        Job {
            id,
            kind,
            weight,
            cancel,
            work,
        }
    }

    /// The job's id, matched by the loader when it is fed back.
    pub fn id(&self) -> JobId {
        self.id
    }

    /// What kind of work.
    pub fn kind(&self) -> JobKind {
        self.kind
    }

    /// Input bytes the job processes: a scheduling hint (run big jobs first, or bound
    /// in-flight bytes).
    pub fn weight(&self) -> u64 {
        self.weight
    }

    /// Does the work. Never blocks, never performs I/O, and returns promptly once the loader
    /// it came from has been dropped (cancellation).
    pub fn run(self) -> JobDone {
        let Job {
            id, cancel, work, ..
        } = self;
        // A job of a dropped loader returns at once; nobody reads its result.
        let outcome = if cancel.is_cancelled() {
            None
        } else {
            Some(work(&cancel))
        };
        JobDone { id, outcome }
    }
}

/// Prints id, kind and weight only.
impl fmt::Debug for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Job")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("weight", &self.weight)
            .finish_non_exhaustive()
    }
}

/// The result of [`Job::run`]; opaque, fed back with `Input::JobDone`.
pub struct JobDone {
    pub(crate) id: JobId,
    /// `None` when the job saw its loader dropped and returned early.
    pub(crate) outcome: Option<Erased>,
}

impl JobDone {
    /// The id of the job that produced it.
    pub fn id(&self) -> JobId {
        self.id
    }

    /// The outcome as a `T`, or `None` for a cancelled job (or a mismatch, which the loader
    /// treats like a panic).
    pub(crate) fn take<T: 'static>(self) -> Option<T> {
        self.outcome?.downcast::<T>().ok().map(|boxed| *boxed)
    }
}

/// Prints the id only.
impl fmt::Debug for JobDone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobDone")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

/// The id of a [`Job`], unique within its loader.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JobId(pub(crate) u64);

/// What kind of work a [`Job`] does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum JobKind {
    /// Parse a manifest.
    Manifest,
    /// Index an archive or a brarchive, or decrypt and read `contents.json`.
    Index,
    /// Parse one content file.
    Parse,
    /// Combine one collection in walk order.
    Combine,
}

/// The flag a loader sets when it is dropped; every job it handed out shares it.
#[derive(Clone, Debug, Default)]
pub(crate) struct Cancel(Arc<AtomicBool>);

impl Cancel {
    /// The loader is gone: return early.
    pub(crate) fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    /// Marks the loader gone.
    pub(crate) fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}
