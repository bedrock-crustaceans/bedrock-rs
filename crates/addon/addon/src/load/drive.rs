//! The one loop that runs a loader to its result. Where the work runs and where keys come from
//! is the [`Answerer`]'s business; the loop itself touches no file, thread or clock.

use super::{Input, Job, KeyRequest, Loader, LoaderError, Output, Request};
use crate::storage::ContentKey;

/// One piece of work a loader handed out, for an [`Answerer`] to do.
#[derive(Debug)]
pub enum Work {
    /// A `ListDir` or `ReadFile`, answered with `Dir`, `File`, `Missing` or `Failed`.
    Read(Request),
    /// CPU work, answered with `JobDone` (or `JobPanicked` by an answerer that catches panics).
    Job(Job),
}

/// Where a loader's work runs and where its keys come from; [`drive`] is the loop that asks.
///
/// Every piece of work given to `dispatch` has exactly one answer, which `next` returns now or
/// later, in any order.
pub trait Answerer {
    /// Starts `work`. Its one answer comes back from [`Answerer::next`].
    fn dispatch(&mut self, work: Work);

    /// The answer of some dispatched work, waiting for it if it must; `None` when no answer
    /// will ever come.
    fn next(&mut self) -> Option<Input>;

    /// The key for `request`, or `None` when there is none, decided on the calling thread.
    fn key(&mut self, request: &KeyRequest) -> Option<ContentKey>;
}

/// Why [`drive`] stopped without the loader's result.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DriveError {
    /// The loader waits, but nothing it asked for is outstanding: a bug in the loader, or an
    /// answerer that lost an answer.
    #[error("the loader stalled with no request or job outstanding")]
    Stalled,
    /// The loader refused an answer: a bug in the answerer.
    #[error(transparent)]
    Loader(#[from] LoaderError),
}

/// Runs `loader` to its result: takes every output it has, dispatches its reads and jobs to
/// `answerer` and answers its key requests at once, then feeds it one answer and repeats.
///
/// # Examples
///
/// Every read answered from a map in memory, every job run inline:
///
/// ```
/// use std::collections::{BTreeMap, VecDeque};
///
/// use bedrock_addon::load::{
///     Answerer, DirEntry, EntryKind, Input, KeyRequest, PackLoader, Request, RootId, Work, drive,
/// };
/// use bedrock_addon::pack::OpenOptions;
/// use bedrock_addon::storage::ContentKey;
/// use bytes::Bytes;
///
/// struct Files(BTreeMap<&'static str, Bytes>, VecDeque<Input>);
///
/// impl Answerer for Files {
///     fn dispatch(&mut self, work: Work) {
///         let input = match work {
///             Work::Job(job) => Input::JobDone(job.run()),
///             Work::Read(Request::ListDir { location, .. }) => Input::Dir {
///                 location,
///                 entries: self.0.iter().map(|(name, bytes)| DirEntry {
///                     name: (*name).into(),
///                     lossy: false,
///                     kind: EntryKind::File { len: bytes.len() as u64 },
///                 }).collect(),
///             },
///             Work::Read(Request::ReadFile { location, .. }) => {
///                 match self.0.get(location.path.as_str()) {
///                     Some(bytes) => Input::File { location, bytes: bytes.clone() },
///                     None => Input::Missing { location },
///                 }
///             }
///         };
///         self.1.push_back(input);
///     }
///     fn next(&mut self) -> Option<Input> {
///         self.1.pop_front()
///     }
///     fn key(&mut self, _: &KeyRequest) -> Option<ContentKey> {
///         None
///     }
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let manifest = Bytes::from_static(br#"{
///   "format_version": 2,
///   "header": { "name": "Ruby", "description": "", "uuid": "00000000-0000-4000-8000-000000000001",
///               "version": [1, 0, 0], "min_engine_version": [1, 21, 0] },
///   "modules": [{ "type": "resources", "uuid": "00000000-0000-4000-8000-000000000002", "version": [1, 0, 0] }]
/// }"#);
/// let mut files = Files(BTreeMap::from([("manifest.json", manifest)]), VecDeque::new());
/// let mut loader = PackLoader::directory(RootId::MEMORY, OpenOptions::default());
/// let pack = drive(&mut loader, &mut files)??;
/// assert_eq!(pack.id().uuid.to_string(), "00000000-0000-4000-8000-000000000001");
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// [`DriveError::Stalled`] when the loader waits with nothing dispatched still unanswered, or
/// the answerer has no answer left to give; [`DriveError::Loader`] when the loader refuses an
/// answer.
pub fn drive<T, L: Loader<T>>(
    loader: &mut L,
    answerer: &mut impl Answerer,
) -> Result<T, DriveError> {
    // Dispatched work not yet answered.
    let mut in_flight = 0usize;
    loop {
        while let Some(output) = loader.poll() {
            match output {
                Output::Request(request) => {
                    answerer.dispatch(Work::Read(request));
                    in_flight += 1;
                }
                Output::Job(job) => {
                    answerer.dispatch(Work::Job(job));
                    in_flight += 1;
                }
                Output::NeedKey(request) => {
                    let key = answerer.key(&request);
                    loader.handle(Input::Key {
                        identity: request.identity,
                        key,
                    })?;
                }
                Output::Done(value) => return Ok(value),
            }
        }
        if in_flight == 0 {
            return Err(DriveError::Stalled);
        }
        let input = answerer.next().ok_or(DriveError::Stalled)?;
        in_flight -= 1;
        loader.handle(input)?;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use bedrock_addon_core::load::{Place, ReadRange, RootId};
    use bedrock_addon_core::pack_identity::{PackIdVersion, PackPath, PackType};
    use bedrock_addon_core::version::SemVersion;
    use bytes::Bytes;

    use super::{Answerer, DriveError, Work, drive};
    use crate::load::{Input, KeyRequest, Loader, LoaderError, Output, Progress, Request, Sans};
    use crate::storage::{ContentIdentity, ContentKey};

    /// Waits for an answer it never asked for.
    struct Waiter;

    impl Sans for Waiter {
        type Input = Input;
        type Output = Output<()>;
        type Error = LoaderError;

        fn handle(&mut self, _: Input) -> Result<(), LoaderError> {
            Ok(())
        }

        fn poll(&mut self) -> Option<Output<()>> {
            None
        }
    }

    impl Loader<()> for Waiter {
        fn progress(&self) -> Progress {
            Progress::default()
        }
    }

    fn place(path: &str) -> Place {
        Place {
            root: RootId(0),
            path: PackPath::new(path).unwrap(),
        }
    }

    /// Asks for `a`, `b` and a key at once, and finishes with the answers in the order they
    /// arrived.
    struct Asker {
        asks: VecDeque<Output<Vec<String>>>,
        seen: Vec<String>,
    }

    impl Asker {
        fn new() -> Asker {
            let read = |path: &str| {
                Output::Request(Request::ReadFile {
                    location: place(path),
                    range: ReadRange::Whole,
                    len_hint: None,
                })
            };
            let key = Output::NeedKey(KeyRequest {
                pack: PackIdVersion::new(
                    uuid::Uuid::from_u128(0x5eed),
                    SemVersion::new(1, 0, 0),
                    PackType::Resource,
                ),
                identity: ContentIdentity::new("abc"),
                location: Place::root(RootId(0)),
            });
            Asker {
                asks: VecDeque::from([read("a"), read("b"), key]),
                seen: Vec::new(),
            }
        }
    }

    impl Sans for Asker {
        type Input = Input;
        type Output = Output<Vec<String>>;
        type Error = LoaderError;

        fn handle(&mut self, input: Input) -> Result<(), LoaderError> {
            let seen = match input {
                Input::File { location, .. } => location.path.as_str().to_owned(),
                Input::Key { identity, .. } => format!("key {}", identity.as_str()),
                _ => return Err(LoaderError::Finished),
            };
            self.seen.push(seen);
            Ok(())
        }

        fn poll(&mut self) -> Option<Output<Vec<String>>> {
            if let Some(ask) = self.asks.pop_front() {
                return Some(ask);
            }
            (self.seen.len() == 3).then(|| Output::Done(std::mem::take(&mut self.seen)))
        }
    }

    impl Loader<Vec<String>> for Asker {
        fn progress(&self) -> Progress {
            Progress::default()
        }
    }

    /// Answers every read with empty bytes, in dispatch order, and has no keys.
    #[derive(Default)]
    struct Queue {
        answers: VecDeque<Input>,
        keys_asked: usize,
    }

    impl Answerer for Queue {
        fn dispatch(&mut self, work: Work) {
            let input = match work {
                Work::Read(Request::ReadFile { location, .. }) => Input::File {
                    location,
                    bytes: Bytes::new(),
                },
                Work::Read(Request::ListDir { location, .. }) => Input::Missing { location },
                Work::Job(job) => Input::JobDone(job.run()),
            };
            self.answers.push_back(input);
        }

        fn next(&mut self) -> Option<Input> {
            self.answers.pop_front()
        }

        fn key(&mut self, _: &KeyRequest) -> Option<ContentKey> {
            self.keys_asked += 1;
            None
        }
    }

    #[test]
    fn drive_reports_a_loader_that_waits_on_nothing() {
        let result = drive(&mut Waiter, &mut Queue::default());
        assert!(matches!(result, Err(DriveError::Stalled)), "{result:?}");
    }

    #[test]
    fn drive_answers_keys_inline_and_reads_in_dispatch_order() {
        let mut answerer = Queue::default();
        let seen = drive(&mut Asker::new(), &mut answerer).unwrap();
        assert_eq!(seen, ["key abc", "a", "b"]);
        assert_eq!(answerer.keys_asked, 1);
    }
}
