//! `Driver`: the loop that runs a loader to completion, its strategies and its key lookup,
//! driven by small scripted loaders over a temporary directory.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs;
use std::io::ErrorKind;
use std::sync::Arc;

use bedrock_addon::load::{
    Input, KeyRequest, ListDepth, Loader, LoaderError, Output, Place, Progress, ReadRange, Request,
    RootId, Sans,
};
use bedrock_addon::pack::{PackIdVersion, PackOrigin, PackType};
use bedrock_addon::storage::{ContentIdentity, ContentKey, ContentKeys, PackPath};
use bedrock_addon::version::SemVersion;
use bedrock_addon_std::{DirSource, DirSourceKind, Driver, LoadError, Options, Strategy};

/// What a scripted loader recorded for one request.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Answer {
    File(Vec<u8>),
    Dir(Vec<String>),
    Missing,
    Failed(ErrorKind),
}

/// Asks for every request it was given at once, then finishes with the answers sorted by
/// location: the shape of a loader whose result does not depend on completion order.
struct Reader {
    requests: Vec<Request>,
    outstanding: usize,
    answers: Vec<(Place, Answer)>,
    finished: bool,
}

impl Reader {
    fn new(requests: Vec<Request>) -> Reader {
        Reader {
            requests,
            outstanding: 0,
            answers: Vec::new(),
            finished: false,
        }
    }
}

impl Sans for Reader {
    type Input = Input;
    type Output = Output<Vec<(Place, Answer)>>;
    type Error = LoaderError;

    fn handle(&mut self, input: Input) -> Result<(), LoaderError> {
        if self.finished {
            return Err(LoaderError::Finished);
        }
        let (location, answer) = match input {
            Input::File { location, bytes } => (location, Answer::File(bytes.to_vec())),
            Input::Dir { location, entries } => {
                let mut names: Vec<String> = entries.iter().map(|e| e.name.to_string()).collect();
                names.sort();
                (location, Answer::Dir(names))
            }
            Input::Missing { location } => (location, Answer::Missing),
            Input::Failed { location, kind } => (location, Answer::Failed(kind)),
            other => panic!("not asked for: {other:?}"),
        };
        self.outstanding -= 1;
        self.answers.push((location, answer));
        Ok(())
    }

    fn poll(&mut self) -> Option<Self::Output> {
        if let Some(request) = self.requests.pop() {
            self.outstanding += 1;
            return Some(Output::Request(request));
        }
        if self.outstanding == 0 && !self.finished {
            self.finished = true;
            let mut answers = std::mem::take(&mut self.answers);
            answers.sort_by(|a, b| a.0.cmp(&b.0));
            return Some(Output::Done(answers));
        }
        None
    }
}

impl Loader<Vec<(Place, Answer)>> for Reader {
    fn progress(&self) -> Progress {
        Progress::default()
    }
}

/// Asks for one key, then finishes with the answer.
struct KeyAsker {
    request: Option<KeyRequest>,
    #[expect(
        clippy::option_option,
        reason = "`None` is no answer yet, `Some(None)` an answer without a key"
    )]
    answer: Option<Option<ContentKey>>,
}

impl KeyAsker {
    fn new(identity: &str) -> KeyAsker {
        KeyAsker {
            request: Some(key_request(identity)),
            answer: None,
        }
    }
}

impl Sans for KeyAsker {
    type Input = Input;
    type Output = Output<Option<ContentKey>>;
    type Error = LoaderError;

    fn handle(&mut self, input: Input) -> Result<(), LoaderError> {
        match input {
            Input::Key { key, .. } => {
                self.answer = Some(key);
                Ok(())
            }
            other => panic!("not asked for: {other:?}"),
        }
    }

    fn poll(&mut self) -> Option<Self::Output> {
        if let Some(request) = self.request.take() {
            return Some(Output::NeedKey(request));
        }
        self.answer.take().map(Output::Done)
    }
}

impl Loader<Option<ContentKey>> for KeyAsker {
    fn progress(&self) -> Progress {
        Progress::default()
    }
}

/// Never asks for anything and never finishes.
struct Waiter;

impl Sans for Waiter {
    type Input = Input;
    type Output = Output<()>;
    type Error = LoaderError;

    fn handle(&mut self, _: Input) -> Result<(), LoaderError> {
        Ok(())
    }

    fn poll(&mut self) -> Option<Self::Output> {
        None
    }
}

impl Loader<()> for Waiter {
    fn progress(&self) -> Progress {
        Progress::default()
    }
}

/// Asks for one file and refuses the answer.
struct Refuser {
    asked: bool,
}

impl Sans for Refuser {
    type Input = Input;
    type Output = Output<()>;
    type Error = LoaderError;

    fn handle(&mut self, _: Input) -> Result<(), LoaderError> {
        Err(LoaderError::Finished)
    }

    fn poll(&mut self) -> Option<Self::Output> {
        if self.asked {
            return None;
        }
        self.asked = true;
        Some(Output::Request(Request::ReadFile {
            location: Place::root(RootId(0)),
            range: ReadRange::Whole,
            len_hint: None,
        }))
    }
}

impl Loader<()> for Refuser {
    fn progress(&self) -> Progress {
        Progress::default()
    }
}

const KEY: [u8; 32] = *b"0123456789abcdefghijklmnopqrstuv";

fn key_request(identity: &str) -> KeyRequest {
    KeyRequest {
        pack: PackIdVersion::new(
            uuid::Uuid::from_u128(0x5eed),
            SemVersion::new(1, 0, 0),
            PackType::Resource,
        ),
        identity: ContentIdentity::new(identity),
        location: Place::root(RootId(0)),
    }
}

fn sequential() -> Options {
    Options::default().with_strategy(Strategy::Sequential)
}

/// Writes `count` files under `dir/files/` and returns one request per file, plus a missing
/// file, a directory read as a file and a recursive listing of `dir`, all under root 0.
fn read_requests(dir: &std::path::Path, count: usize) -> Vec<Request> {
    let mut requests = Vec::new();
    for index in 0..count {
        let name = format!("files/{index:04}.json");
        fs::create_dir_all(dir.join("files")).unwrap();
        fs::write(dir.join(&name), format!("{{\"index\":{index}}}")).unwrap();
        requests.push(Request::ReadFile {
            location: place(&name),
            range: ReadRange::Whole,
            len_hint: None,
        });
    }
    requests.push(Request::ReadFile {
        location: place("missing.json"),
        range: ReadRange::Prefix(256),
        len_hint: None,
    });
    requests.push(Request::ReadFile {
        location: place("files"),
        range: ReadRange::Whole,
        len_hint: None,
    });
    requests.push(Request::ListDir {
        location: Place::root(RootId(0)),
        depth: ListDepth::Recursive,
    });
    requests
}

fn place(path: &str) -> Place {
    Place::root(RootId(0)).join(&PackPath::new(path).unwrap())
}

mod strategy {
    use super::*;

    #[cfg(feature = "rayon")]
    #[test]
    fn the_default_is_rayon_with_rayons_thread_count() {
        assert!(matches!(
            Strategy::default(),
            Strategy::Rayon { threads: None }
        ));
    }

    #[cfg(not(feature = "rayon"))]
    #[test]
    fn the_default_is_sequential_without_rayon() {
        assert!(matches!(Strategy::default(), Strategy::Sequential));
    }

    #[test]
    fn default_options_ask_no_provider_and_read_no_sidecar() {
        let options = Options::default();
        assert!(options.keys.is_none());
        #[cfg(feature = "encrypted")]
        assert!(!options.sidecar_keys, "sidecar keys are opt-in");
    }

    #[test]
    fn a_new_driver_has_an_empty_root_table() {
        let driver = Driver::new(sequential()).unwrap();
        assert_eq!(driver.roots().host_path(&Place::root(RootId(0))), None);
    }

    #[test]
    fn add_root_grows_the_table() {
        let mut driver = Driver::new(sequential()).unwrap();
        let first = driver.add_root("/srv/behavior_packs");
        let second = driver.add_root("/srv/ruby.mcpack");
        assert_ne!(first, second);
        assert_eq!(
            driver.roots().host_path(&Place::root(second)),
            Some(std::path::PathBuf::from("/srv/ruby.mcpack"))
        );
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn every_strategy_builds_a_driver() {
        for strategy in [
            Strategy::Sequential,
            Strategy::Rayon { threads: None },
            Strategy::Rayon {
                threads: std::num::NonZeroUsize::new(2),
            },
            Strategy::RayonPool(Arc::new(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(1)
                    .build()
                    .unwrap(),
            )),
        ] {
            Driver::new(Options::default().with_strategy(strategy)).unwrap();
        }
    }
}

mod dir_source {
    use super::*;

    #[test]
    fn packs_points_at_a_packs_directory() {
        let source = DirSource::packs("worlds/a/behavior_packs", PackOrigin::World);
        assert_eq!(
            source.path,
            std::path::PathBuf::from("worlds/a/behavior_packs")
        );
        assert_eq!(source.origin, PackOrigin::World);
        assert_eq!(source.kind, DirSourceKind::Packs);
    }

    #[test]
    fn archive_points_at_one_file() {
        let source = DirSource::archive("downloads/ruby.mcaddon", PackOrigin::Package);
        assert_eq!(
            source.path,
            std::path::PathBuf::from("downloads/ruby.mcaddon")
        );
        assert_eq!(source.origin, PackOrigin::Package);
        assert_eq!(source.kind, DirSourceKind::Archive);
    }
}

mod drive {
    use super::*;

    /// A driver whose root 0 is `dir`.
    fn driver_over(dir: &std::path::Path, options: Options) -> Driver {
        let mut driver = Driver::new(options).unwrap();
        assert_eq!(driver.add_root(dir), RootId(0));
        driver
    }

    fn expected(count: usize) -> Vec<(Place, Answer)> {
        let mut answers: Vec<(Place, Answer)> = (0..count)
            .map(|index| {
                (
                    place(&format!("files/{index:04}.json")),
                    Answer::File(format!("{{\"index\":{index}}}").into_bytes()),
                )
            })
            .collect();
        answers.push((place("missing.json"), Answer::Missing));
        answers.push((place("files"), Answer::Failed(ErrorKind::IsADirectory)));
        let mut names: Vec<String> = (0..count)
            .map(|index| format!("files/{index:04}.json"))
            .collect();
        names.push("files".to_owned());
        names.sort();
        answers.push((Place::root(RootId(0)), Answer::Dir(names)));
        answers.sort_by(|a, b| a.0.cmp(&b.0));
        answers
    }

    #[test]
    fn sequential_answers_every_request() {
        let dir = tempfile::tempdir().unwrap();
        let requests = read_requests(dir.path(), 8);
        let driver = driver_over(dir.path(), sequential());
        assert_eq!(driver.drive(Reader::new(requests)).unwrap(), expected(8));
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn rayon_answers_like_sequential() {
        let dir = tempfile::tempdir().unwrap();
        let requests = read_requests(dir.path(), 300);
        let sequential = driver_over(dir.path(), sequential())
            .drive(Reader::new(requests.clone()))
            .unwrap();
        assert_eq!(sequential, expected(300));
        for strategy in [
            Strategy::Rayon { threads: None },
            Strategy::Rayon {
                threads: std::num::NonZeroUsize::new(3),
            },
            Strategy::RayonPool(Arc::new(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(2)
                    .build()
                    .unwrap(),
            )),
        ] {
            let options = Options::default().with_strategy(strategy.clone());
            let parallel = driver_over(dir.path(), options)
                .drive(Reader::new(requests.clone()))
                .unwrap();
            assert_eq!(parallel, sequential, "{strategy:?}");
        }
    }

    #[test]
    fn a_driver_can_drive_several_loaders() {
        let dir = tempfile::tempdir().unwrap();
        let requests = read_requests(dir.path(), 2);
        let driver = driver_over(dir.path(), Options::default());
        let first = driver.drive(Reader::new(requests.clone())).unwrap();
        let second = driver.drive(Reader::new(requests)).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn a_loader_waiting_on_nothing_is_stalled() {
        for options in [sequential(), Options::default()] {
            let driver = Driver::new(options).unwrap();
            assert!(matches!(driver.drive(Waiter), Err(LoadError::Stalled)));
        }
    }

    #[test]
    fn a_refused_answer_is_a_loader_error() {
        for options in [sequential(), Options::default()] {
            let driver = Driver::new(options).unwrap();
            assert!(matches!(
                driver.drive(Refuser { asked: false }),
                Err(LoadError::Loader(LoaderError::Finished))
            ));
        }
    }

    #[test]
    fn need_key_is_answered_by_the_provider() {
        let keys: ContentKeys = [(ContentIdentity::new("abc"), ContentKey::new(KEY))]
            .into_iter()
            .collect();
        let driver = Driver::new(sequential().with_keys(Arc::new(keys))).unwrap();
        assert_eq!(
            driver.drive(KeyAsker::new("abc")).unwrap(),
            Some(ContentKey::new(KEY))
        );
        assert_eq!(driver.drive(KeyAsker::new("other")).unwrap(), None);
    }

    #[test]
    fn need_key_without_a_provider_is_answered_with_no_key() {
        let driver = Driver::new(sequential()).unwrap();
        assert_eq!(driver.drive(KeyAsker::new("abc")).unwrap(), None);
    }

    #[cfg(feature = "encrypted")]
    #[test]
    fn a_sidecar_key_is_read_only_when_opted_in() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("pack.mcpack"), b"PK\x03\x04").unwrap();
        fs::write(dir.path().join("pack.mcpack.key"), KEY).unwrap();
        let archive = dir.path().join("pack.mcpack");

        let mut without = Driver::new(sequential()).unwrap();
        without.add_root(&archive);
        assert_eq!(without.drive(KeyAsker::new("abc")).unwrap(), None);

        let mut with = Driver::new(sequential().with_sidecar_keys(true)).unwrap();
        with.add_root(&archive);
        assert_eq!(
            with.drive(KeyAsker::new("abc")).unwrap(),
            Some(ContentKey::new(KEY))
        );
    }

    #[cfg(feature = "encrypted")]
    #[test]
    fn the_sidecar_is_asked_before_the_provider() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("pack.mcpack"), b"PK\x03\x04").unwrap();
        fs::write(dir.path().join("pack.mcpack.key"), KEY).unwrap();
        let provider: ContentKeys = [(ContentIdentity::new("abc"), ContentKey::new([b'z'; 32]))]
            .into_iter()
            .collect();
        let mut driver = Driver::new(
            sequential()
                .with_sidecar_keys(true)
                .with_keys(Arc::new(provider)),
        )
        .unwrap();
        driver.add_root(dir.path().join("pack.mcpack"));
        assert_eq!(
            driver.drive(KeyAsker::new("abc")).unwrap(),
            Some(ContentKey::new(KEY))
        );
    }

    #[cfg(feature = "encrypted")]
    #[test]
    fn without_a_sidecar_the_provider_answers() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("pack.mcpack"), b"PK\x03\x04").unwrap();
        let provider: ContentKeys = [(ContentIdentity::new("abc"), ContentKey::new(KEY))]
            .into_iter()
            .collect();
        let mut driver = Driver::new(
            sequential()
                .with_sidecar_keys(true)
                .with_keys(Arc::new(provider)),
        )
        .unwrap();
        driver.add_root(dir.path().join("pack.mcpack"));
        assert_eq!(
            driver.drive(KeyAsker::new("abc")).unwrap(),
            Some(ContentKey::new(KEY))
        );
    }
}

mod errors {
    use super::*;

    #[cfg(feature = "rayon")]
    #[test]
    fn a_pool_build_failure_is_a_pool_error() {
        // The only build failure a test can provoke: the global pool built twice.
        let _ = rayon::ThreadPoolBuilder::new().build_global();
        let error = rayon::ThreadPoolBuilder::new().build_global().unwrap_err();
        assert!(matches!(LoadError::from(error), LoadError::Pool(_)));
    }

    #[test]
    fn a_loader_error_converts() {
        assert!(matches!(
            LoadError::from(LoaderError::Finished),
            LoadError::Loader(LoaderError::Finished)
        ));
    }
}
