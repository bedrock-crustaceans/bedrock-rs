//! A scripted in-memory responder for the loader conformance tests (the loader conformance
//! layer): a virtual tree of `(Place, bytes)` that answers a loader's requests,
//! key requests and jobs in a chosen order, with failures, lossy names and panics on demand,
//! and records what it was asked so the tests can check sequences and windows.

#![allow(dead_code, reason = "each test file uses part of the responder")]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::ErrorKind;

use bedrock_addon::load::{
    DirEntry, EntryKind, Input, JobKind, ListDepth, Loader, Output, Place, ReadRange, Request,
    RootId,
};
use bedrock_addon::storage::{ContentIdentity, ContentKey, PackPath};
use bytes::Bytes;

/// The order in which the responder completes what a loader handed out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    /// The oldest outstanding item first.
    Issue,
    /// The newest outstanding item first.
    Reverse,
    /// A seeded pseudo-random pick each time.
    Seeded(u64),
}

impl Order {
    /// The five orders of the determinism test.
    pub const ALL: [Order; 5] = [
        Order::Issue,
        Order::Reverse,
        Order::Seeded(1),
        Order::Seeded(0x5eed),
        Order::Seeded(0xdead_beef),
    ];
}

/// Something a loader handed out and the responder has not completed yet.
#[derive(Debug)]
enum Item {
    Request(Request),
    Key(ContentIdentity),
    Job(bedrock_addon::load::Job),
}

/// What the responder saw, for assertions.
#[derive(Clone, Debug, Default)]
pub struct Log {
    /// Every request, in issue order.
    pub requests: Vec<Request>,
    /// Every key request's identity, in issue order.
    pub keys: Vec<ContentIdentity>,
    /// Every job's kind, in issue order.
    pub jobs: Vec<JobKind>,
    /// The most `ReadFile` requests outstanding at once.
    pub max_reads_out: usize,
    /// The most bytes of `ReadFile` requests outstanding at once (by the file's size).
    pub max_read_bytes_out: u64,
    /// The most jobs outstanding at once.
    pub max_jobs_out: usize,
}

/// The responder.
#[derive(Clone, Debug)]
pub struct Responder {
    files: BTreeMap<Place, Bytes>,
    dirs: BTreeSet<Place>,
    missing: BTreeSet<Place>,
    failed: BTreeMap<Place, ErrorKind>,
    lossy: BTreeSet<Place>,
    /// Extra raw names listed under a directory (names the loader must reject).
    raw_names: BTreeMap<Place, Vec<(Box<str>, bool)>>,
    keys: HashMap<ContentIdentity, ContentKey>,
    /// Job kinds whose jobs "panic": the responder sends `JobPanicked` for them.
    panics: BTreeSet<String>,
    /// Jobs of a panicking kind to run normally before the panics start.
    panic_skip: BTreeMap<String, usize>,
    /// Hold every `n`-th item back until nothing else is left (`None`: no holding).
    pub order: Order,
    rng: u64,
    pub log: Log,
}

impl Responder {
    /// An empty tree answered in `order`.
    pub fn new(order: Order) -> Responder {
        let rng = match order {
            Order::Seeded(seed) => seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
            _ => 1,
        };
        Responder {
            files: BTreeMap::new(),
            dirs: BTreeSet::new(),
            missing: BTreeSet::new(),
            failed: BTreeMap::new(),
            lossy: BTreeSet::new(),
            raw_names: BTreeMap::new(),
            keys: HashMap::new(),
            panics: BTreeSet::new(),
            panic_skip: BTreeMap::new(),
            order,
            rng,
            log: Log::default(),
        }
    }

    /// The same tree answered in another order, with a fresh log.
    pub fn reordered(&self, order: Order) -> Responder {
        let mut copy = self.clone();
        copy.order = order;
        copy.rng = Responder::new(order).rng;
        copy.log = Log::default();
        copy
    }

    /// Adds a file at `path` below `root`.
    pub fn file(&mut self, root: u32, path: &str, bytes: impl Into<Bytes>) -> &mut Self {
        let place = place(root, path);
        let mut parent = place.path.as_str();
        while let Some((dir, _)) = parent.rsplit_once('/') {
            self.dirs.insert(self::place(root, dir));
            parent = dir;
        }
        self.dirs.insert(Place::root(RootId(root)));
        self.files.insert(place, bytes.into());
        self
    }

    /// Adds every file of `files` below `root/prefix`.
    pub fn pack(&mut self, root: u32, prefix: &str, files: &[(&str, &[u8])]) -> &mut Self {
        for (path, bytes) in files {
            let full = if prefix.is_empty() {
                (*path).to_owned()
            } else {
                format!("{prefix}/{path}")
            };
            self.file(root, &full, Bytes::copy_from_slice(bytes));
        }
        self
    }

    /// Answers requests for `path` with `Missing`.
    pub fn missing(&mut self, root: u32, path: &str) -> &mut Self {
        self.missing.insert(place(root, path));
        self
    }

    /// Answers requests for `path` with `Failed { kind }`.
    pub fn failed(&mut self, root: u32, path: &str, kind: ErrorKind) -> &mut Self {
        self.failed.insert(place(root, path), kind);
        self
    }

    /// Lists the file at `path` as lossy (its host name was not UTF-8).
    pub fn lossy(&mut self, root: u32, path: &str) -> &mut Self {
        self.lossy.insert(place(root, path));
        self
    }

    /// Lists an extra raw name under `dir` (the loader must reject it).
    pub fn raw_name(&mut self, root: u32, dir: &str, name: &str) -> &mut Self {
        self.raw_entry(root, dir, name, false)
    }

    /// Lists an extra file entry `name` under `dir`, `lossy` as the driver says.
    pub fn raw_entry(&mut self, root: u32, dir: &str, name: &str, lossy: bool) -> &mut Self {
        self.raw_names
            .entry(place(root, dir))
            .or_default()
            .push((name.into(), lossy));
        self
    }

    /// Makes the root `root` itself a file (an archive source).
    pub fn root_file(&mut self, root: u32, bytes: impl Into<Bytes>) -> &mut Self {
        self.files.insert(Place::root(RootId(root)), bytes.into());
        self
    }

    /// Answers `NeedKey` for `identity` with `key`.
    pub fn key(&mut self, identity: &str, key: ContentKey) -> &mut Self {
        self.keys.insert(ContentIdentity::new(identity), key);
        self
    }

    /// Answers every job of `kind` with `JobPanicked`.
    pub fn panic_on(&mut self, kind: JobKind) -> &mut Self {
        self.panics.insert(format!("{kind:?}"));
        self
    }

    /// Runs the first `skip` jobs of `kind`, then answers every later one with `JobPanicked`.
    pub fn panic_after(&mut self, kind: JobKind, skip: usize) -> &mut Self {
        self.panic_skip.insert(format!("{kind:?}"), skip);
        self.panic_on(kind)
    }

    /// Drives `loader` to its result.
    pub fn drive<T, L: Loader<T>>(&mut self, loader: &mut L) -> T {
        let mut pool: Vec<Item> = Vec::new();
        loop {
            while let Some(output) = loader.poll() {
                match output {
                    Output::Done(result) => {
                        assert!(pool.is_empty(), "Done while {pool:?} is outstanding");
                        assert!(loader.poll().is_none(), "poll after Done");
                        return result;
                    }
                    Output::Request(request) => {
                        self.log.requests.push(request.clone());
                        pool.push(Item::Request(request));
                    }
                    Output::NeedKey(request) => {
                        self.log.keys.push(request.identity.clone());
                        pool.push(Item::Key(request.identity));
                    }
                    Output::Job(job) => {
                        self.log.jobs.push(job.kind());
                        pool.push(Item::Job(job));
                    }
                }
            }
            self.measure(&pool);
            assert!(
                !pool.is_empty(),
                "the loader waits with nothing outstanding"
            );
            let item = pool.remove(self.pick(pool.len()));
            let input = self.answer(item);
            loader
                .handle(input)
                .expect("the loader accepts every answer");
        }
    }

    /// Records the windows the pool shows.
    fn measure(&mut self, pool: &[Item]) {
        let mut reads = 0;
        let mut bytes = 0;
        let mut jobs = 0;
        for item in pool {
            match item {
                Item::Request(Request::ReadFile {
                    location, range, ..
                }) => {
                    reads += 1;
                    let len = self.files.get(location).map_or(0, |b| b.len() as u64);
                    bytes += match range {
                        ReadRange::Prefix(n) => u64::from(*n).min(len),
                        ReadRange::Whole => len,
                    };
                }
                Item::Job(_) => jobs += 1,
                _ => {}
            }
        }
        self.log.max_reads_out = self.log.max_reads_out.max(reads);
        self.log.max_read_bytes_out = self.log.max_read_bytes_out.max(bytes);
        self.log.max_jobs_out = self.log.max_jobs_out.max(jobs);
    }

    fn pick(&mut self, len: usize) -> usize {
        match self.order {
            Order::Issue => 0,
            Order::Reverse => len - 1,
            Order::Seeded(_) => {
                // xorshift64
                self.rng ^= self.rng << 13;
                self.rng ^= self.rng >> 7;
                self.rng ^= self.rng << 17;
                (self.rng % len as u64) as usize
            }
        }
    }

    /// The input that answers `item`.
    pub fn answer_item(&mut self, request: Request) -> Input {
        self.answer(Item::Request(request))
    }

    fn answer(&mut self, item: Item) -> Input {
        match item {
            Item::Key(identity) => Input::Key {
                key: self.keys.get(&identity).cloned(),
                identity,
            },
            Item::Job(job) => {
                let kind = format!("{:?}", job.kind());
                let skipping = self.panic_skip.get_mut(&kind).filter(|skip| **skip > 0);
                if let Some(skip) = skipping {
                    *skip -= 1;
                    Input::JobDone(job.run())
                } else if self.panics.contains(&kind) {
                    Input::JobPanicked(job.id())
                } else {
                    Input::JobDone(job.run())
                }
            }
            Item::Request(request) => self.respond(&request),
        }
    }

    /// The answer to one request.
    pub fn respond(&self, request: &Request) -> Input {
        let location = match request {
            Request::ListDir { location, .. } | Request::ReadFile { location, .. } => location,
        };
        if let Some(kind) = self.failed.get(location) {
            return Input::Failed {
                location: location.clone(),
                kind: *kind,
            };
        }
        if self.missing.contains(location) {
            return Input::Missing {
                location: location.clone(),
            };
        }
        match request {
            Request::ReadFile {
                location, range, ..
            } => match self.files.get(location) {
                Some(bytes) => {
                    let bytes = match range {
                        ReadRange::Prefix(n) => bytes.slice(..(*n as usize).min(bytes.len())),
                        ReadRange::Whole => bytes.clone(),
                    };
                    Input::File {
                        location: location.clone(),
                        bytes,
                    }
                }
                None => Input::Missing {
                    location: location.clone(),
                },
            },
            Request::ListDir { location, depth } => {
                if !self.dirs.contains(location) {
                    return Input::Missing {
                        location: location.clone(),
                    };
                }
                Input::Dir {
                    location: location.clone(),
                    entries: self.list(location, *depth),
                }
            }
        }
    }

    /// The entries below `dir`, in reverse order (the loader sorts them).
    fn list(&self, dir: &Place, depth: ListDepth) -> Vec<DirEntry> {
        let below = |place: &Place| -> Option<String> {
            if place.root != dir.root || place == dir {
                return None;
            }
            if dir.path.is_root() {
                return Some(place.path.as_str().to_owned());
            }
            place
                .path
                .as_str()
                .strip_prefix(dir.path.as_str())?
                .strip_prefix('/')
                .map(str::to_owned)
        };
        let mut entries = Vec::new();
        for (place, bytes) in &self.files {
            let Some(name) = below(place) else { continue };
            if depth == ListDepth::Children && name.contains('/') {
                continue;
            }
            entries.push(DirEntry {
                name: name.into(),
                lossy: self.lossy.contains(place),
                kind: EntryKind::File {
                    len: bytes.len() as u64,
                },
            });
        }
        for place in &self.dirs {
            let Some(name) = below(place) else { continue };
            if depth == ListDepth::Children && name.contains('/') {
                continue;
            }
            entries.push(DirEntry {
                name: name.into(),
                lossy: false,
                kind: EntryKind::Dir,
            });
        }
        for (name, lossy) in self.raw_names.get(dir).into_iter().flatten() {
            entries.push(DirEntry {
                name: name.clone(),
                lossy: *lossy,
                kind: EntryKind::File { len: 1 },
            });
        }
        entries.reverse();
        entries
    }
}

/// The place of `path` below `root`.
pub fn place(root: u32, path: &str) -> Place {
    if path.is_empty() {
        Place::root(RootId(root))
    } else {
        Place::root(RootId(root)).join(&PackPath::new(path).expect("a valid test path"))
    }
}

/// A format-2 manifest with UUID `n` and one module of `module` type.
pub fn manifest(n: u128, module: &str) -> String {
    format!(
        r#"{{
  "format_version": 2,
  "header": {{ "name": "pack {n}", "description": "", "uuid": "{}",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [{{ "type": "{module}", "uuid": "{}", "version": [1, 0, 0] }}]
}}"#,
        uuid::Uuid::from_u128(n),
        uuid::Uuid::from_u128(n | 0xffff_0000_0000_0000_0000),
    )
}
