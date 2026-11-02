//! The `StackLoader` over toy collections (a table of toy accessor lines stands in for the
//! families'): reads by walk and residency, the brarchive layer, slots in walk order whatever the
//! completion order, the read window, failed reads, panicked jobs and cancellation.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics};
use bedrock_addon_core::load::{
    AppDataPlace, Limits, ListDepth, LoadContext, LoadOptions, Place, ReadRange, RootId, StackCx,
    Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackOrigin, PackPath, PackRef, PackSide};
use bytes::Bytes;

use super::collection::{ErasedCollection, Loaded};
use super::job::Erased;
use super::{
    DirEntry, EntryKind, Input, JobKind, Loader, Output, PackLoader, Request, Residency, Sans,
    StackLoader, StackOptions,
};
use crate::pack::stack::{PackInstance, PackStack};
use crate::pack::{OpenOptions, Pack, StackSet};

/// What the toy collection keeps: each parsed file's pack index, path and text, in combine
/// order, and how many packs the stack had.
#[derive(Debug, PartialEq)]
struct Toys {
    files: Vec<(Option<usize>, String, String)>,
    packs: usize,
}

fn json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("json"))
}

/// A toy accessor line: entities, bottom-up, on `side`.
struct Toy {
    side: PackSide,
    cache: OnceLock<Loaded<Toys>>,
    /// Combines seen (a panicking combine is simulated by the driver, not here).
    combined: Mutex<usize>,
    /// The directories the walk reads.
    directories: &'static [&'static str],
    /// The keys a second walk stage names, whatever the first parsed; none for no stage.
    then_keys: &'static [&'static str],
    /// How many parsed files the second stage was shown, once asked.
    then_saw: Mutex<Option<usize>>,
    /// The walk's text for a file that cannot be read.
    read_error: Option<&'static str>,
}

impl Toy {
    fn leak(side: PackSide) -> &'static Toy {
        Toy::leak_with(side, &["entities"], &[])
    }

    /// A line that walks `directories` and then reads the files at `then_keys`.
    fn leak_with(
        side: PackSide,
        directories: &'static [&'static str],
        then_keys: &'static [&'static str],
    ) -> &'static Toy {
        Box::leak(Box::new(Toy {
            side,
            cache: OnceLock::new(),
            combined: Mutex::new(0),
            directories,
            then_keys,
            then_saw: Mutex::new(None),
            read_error: None,
        }))
    }
}

impl ErasedCollection for Toy {
    fn collection(&self) -> Option<super::Collection> {
        None
    }
    fn side(&self) -> PackSide {
        self.side
    }
    fn walk(&self) -> Walk {
        Walk {
            directories: self.directories,
            depth: ListDepth::Recursive,
            accept: json,
            order: WalkOrder::BottomUp,
            directory_major: false,
            app_data: &[],
            app_data_place: AppDataPlace::After,
            in_pack: None,
            shadowed: None,
            read_error: self.read_error,
        }
    }
    fn has_then(&self) -> bool {
        !self.then_keys.is_empty()
    }
    fn then(&self, parsed: &[Erased]) -> Vec<PackPath> {
        *self.then_saw.lock().unwrap() = Some(parsed.len());
        self.then_keys
            .iter()
            .map(|key| PackPath::from_static(key))
            .collect()
    }
    fn parse(&self, file: &WalkFile, _cx: &LoadContext) -> Erased {
        Box::new((
            file.pack.index(),
            file.path.to_string(),
            String::from_utf8_lossy(&file.bytes).into_owned(),
        ))
    }
    fn combine(&self, parsed: Vec<Erased>, cx: &StackCx<'_>) -> Erased {
        *self.combined.lock().unwrap() += 1;
        let files = parsed
            .into_iter()
            .map(|value| *value.downcast::<(Option<usize>, String, String)>().unwrap())
            .collect();
        Box::new(Toys {
            files,
            packs: cx.packs.len(),
        })
    }
    fn install(&self, _stack: &dyn std::any::Any, value: Erased, findings: Diagnostics) {
        let _ = self.cache.set(Loaded {
            value: *value.downcast::<Toys>().unwrap(),
            findings,
        });
    }
    fn is_loaded(&self, _stack: &dyn std::any::Any) -> bool {
        self.cache.get().is_some()
    }
    fn load(&self, _stack: &dyn std::any::Any) {}
    fn report<'a>(&self, _stack: &'a dyn std::any::Any, _out: &mut Vec<&'a Diagnostic>) {}
    fn debug(&self, _stack: &dyn std::any::Any) -> Option<String> {
        self.cache.get().map(|loaded| format!("{:?}", loaded.value))
    }
}

/// A behaviour-pack manifest with UUID `n`.
fn manifest(n: u8) -> String {
    format!(
        r#"{{"format_version": 2,
  "header": {{ "name": "bp {n}", "description": "", "uuid": "00000000-0000-4000-8000-0000000000{n:02}",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [{{ "type": "data", "uuid": "00000000-0000-4000-9000-0000000000{n:02}", "version": [1, 0, 0] }}]}}"#
    )
}

/// A tree of files per root, answered in a chosen order.
#[derive(Clone, Default)]
struct Tree {
    files: BTreeMap<Place, Bytes>,
    failed: BTreeMap<Place, std::io::ErrorKind>,
}

/// How the driver picks the next outstanding item.
#[derive(Clone, Copy, Debug)]
enum Order {
    Issue,
    Reverse,
    Seeded(u64),
    /// Every request before any job: a loader that released a read charge at the answer would
    /// have every read answered before the first parse.
    RequestsFirst,
}

const ORDERS: [Order; 5] = [
    Order::Issue,
    Order::Reverse,
    Order::Seeded(1),
    Order::Seeded(77),
    Order::Seeded(4242),
];

/// What the driver saw.
#[derive(Default, Debug)]
struct Seen {
    reads: Vec<Place>,
    max_reads_out: usize,
    max_bytes_out: u64,
    /// The most jobs outstanding at once.
    max_jobs_out: usize,
    /// The most loose reads answered and not parsed yet.
    max_unparsed: usize,
    /// Reads answered minus `Parse` jobs fed back.
    unparsed: usize,
    /// Panic at most this many jobs of the panicking kind (`None`: all).
    panic_limit: Option<usize>,
}

impl Tree {
    fn file(&mut self, root: u32, path: &str, bytes: impl Into<Bytes>) -> &mut Self {
        let place = Place::root(RootId(root)).join(&PackPath::new(path).unwrap());
        self.files.insert(place, bytes.into());
        self
    }

    fn answer(&self, request: &Request) -> Input {
        match request {
            Request::ListDir { location, .. } => {
                let entries = self
                    .files
                    .iter()
                    .filter(|(place, _)| place.root == location.root)
                    .map(|(place, bytes)| DirEntry {
                        name: place.path.as_str().into(),
                        lossy: false,
                        kind: EntryKind::File {
                            len: bytes.len() as u64,
                        },
                    })
                    .collect();
                Input::Dir {
                    location: location.clone(),
                    entries,
                }
            }
            Request::ReadFile {
                location, range, ..
            } => {
                if let Some(kind) = self.failed.get(location) {
                    return Input::Failed {
                        location: location.clone(),
                        kind: *kind,
                    };
                }
                match self.files.get(location) {
                    Some(bytes) => Input::File {
                        location: location.clone(),
                        bytes: match range {
                            ReadRange::Prefix(n) => bytes.slice(..(*n as usize).min(bytes.len())),
                            ReadRange::Whole => bytes.clone(),
                        },
                    },
                    None => Input::Missing {
                        location: location.clone(),
                    },
                }
            }
        }
    }

    /// Drives `loader` to its result in `order`; jobs of `panic` kind are fed back panicked.
    fn drive<T, L: Loader<T>>(
        &self,
        loader: &mut L,
        order: Order,
        panic: Option<JobKind>,
        log: &mut Seen,
    ) -> T {
        let mut pool: Vec<Output<T>> = Vec::new();
        let mut rng = match order {
            Order::Seeded(seed) => seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
            _ => 1,
        };
        loop {
            while let Some(output) = loader.poll() {
                if let Output::Done(result) = output {
                    assert!(pool.is_empty());
                    return result;
                }
                pool.push(output);
            }
            let reads: Vec<&Place> = pool
                .iter()
                .filter_map(|output| match output {
                    Output::Request(Request::ReadFile { location, .. }) => Some(location),
                    _ => None,
                })
                .collect();
            log.max_reads_out = log.max_reads_out.max(reads.len());
            let bytes: u64 = reads
                .iter()
                .map(|place| self.files.get(*place).map_or(0, |b| b.len() as u64))
                .sum();
            log.max_bytes_out = log.max_bytes_out.max(bytes);
            assert!(!pool.is_empty(), "stalled");
            let jobs = pool.iter().filter(|o| matches!(o, Output::Job(_))).count();
            log.max_jobs_out = log.max_jobs_out.max(jobs);
            let index = match order {
                Order::RequestsFirst => pool
                    .iter()
                    .position(|o| matches!(o, Output::Request(_)))
                    .unwrap_or(0),
                Order::Issue => 0,
                Order::Reverse => pool.len() - 1,
                Order::Seeded(_) => {
                    rng ^= rng << 13;
                    rng ^= rng >> 7;
                    rng ^= rng << 17;
                    usize::try_from(rng % pool.len() as u64).unwrap()
                }
            };
            let input = match pool.remove(index) {
                Output::Request(request) => {
                    if let Request::ReadFile { location, .. } = &request {
                        log.reads.push(location.clone());
                        log.unparsed += 1;
                        log.max_unparsed = log.max_unparsed.max(log.unparsed);
                    }
                    self.answer(&request)
                }
                Output::Job(job)
                    if Some(job.kind()) == panic && log.panic_limit.is_none_or(|n| n > 0) =>
                {
                    log.panic_limit = log.panic_limit.map(|n| n - 1);
                    Input::JobPanicked(job.id())
                }
                Output::Job(job) => {
                    if job.kind() == JobKind::Parse {
                        log.unparsed = log.unparsed.saturating_sub(1);
                    }
                    Input::JobDone(job.run())
                }
                Output::NeedKey(request) => Input::Key {
                    identity: request.identity,
                    key: None,
                },
                Output::Done(_) => unreachable!(),
            };
            loader.handle(input).unwrap();
        }
    }

    /// The folder pack at `root`, opened by the `PackLoader`.
    fn open(&self, root: u32) -> Pack {
        let mut loader = PackLoader::directory(
            RootId(root),
            OpenOptions {
                origin: PackOrigin::World,
                ..OpenOptions::default()
            },
        );
        self.drive(&mut loader, Order::Issue, None, &mut Seen::default())
            .unwrap()
    }
}

/// Two behaviour packs: 1 (top) with `entities/pig.json` and a texture, 2 with
/// `entities/cow.json` and `entities/a/b.json`.
fn two_packs() -> Tree {
    let mut tree = Tree::default();
    tree.file(1, "manifest.json", manifest(1))
        .file(1, "entities/pig.json", "pig")
        .file(1, "textures/pig.png", "png")
        .file(2, "manifest.json", manifest(2))
        .file(2, "entities/cow.json", "cow")
        .file(2, "entities/a/b.json", "b");
    tree
}

/// A `StackSet` with `packs` (top first) on the behaviour stack.
fn stack_set(packs: Vec<Pack>) -> StackSet {
    let instances = packs
        .into_iter()
        .map(|pack| PackInstance {
            origin: PackOrigin::World,
            pack,
            subpack: None,
            is_dependent: false,
        })
        .collect();
    StackSet {
        behavior: PackStack::new(
            instances,
            Vec::new(),
            LoadOptions::default(),
            Diagnostics::default(),
        ),
        resource: PackStack::of([], LoadOptions::default()).unwrap(),
    }
}

/// Loads the toy collection over `tree`'s packs 1 and 2.
fn load(
    tree: &Tree,
    options: &StackOptions,
    order: Order,
    panic: Option<JobKind>,
) -> (&'static Toy, StackSet, Seen) {
    let toy = Toy::leak(PackSide::Behavior);
    let set = stack_set(vec![tree.open(1), tree.open(2)]);
    let mut loader = StackLoader::with_table(set, options, vec![toy]);
    let mut seen = Seen::default();
    let set = tree.drive(&mut loader, order, panic, &mut seen);
    (toy, set, seen)
}

fn files(toy: &Toy) -> Vec<(Option<usize>, String, String)> {
    toy.cache.get().expect("installed").value.files.clone()
}

#[test]
fn a_collection_reads_only_its_walk_and_combines_bottom_up() {
    let (toy, set, seen) = load(&two_packs(), &StackOptions::default(), Order::Issue, None);
    assert_eq!(
        files(toy),
        [
            (Some(1), "entities/a/b.json".to_owned(), "b".to_owned()),
            (Some(1), "entities/cow.json".to_owned(), "cow".to_owned()),
            (Some(0), "entities/pig.json".to_owned(), "pig".to_owned()),
        ]
    );
    assert_eq!(toy.cache.get().unwrap().value.packs, 2);
    let mut reads: Vec<String> = seen.reads.iter().map(ToString::to_string).collect();
    reads.sort();
    assert_eq!(
        reads,
        [
            "#1/entities/pig.json",
            "#2/entities/a/b.json",
            "#2/entities/cow.json"
        ]
    );
    // `Transient`: the loose bytes are released once parsed.
    let pig = PackPath::from_static("entities/pig.json");
    assert!(!set.behavior.instances()[0].pack.is_resident(&pig));
}

#[test]
fn residency_all_reads_every_file_and_keeps_it() {
    let options = StackOptions {
        behavior: Residency::All,
        ..StackOptions::default()
    };
    let (toy, set, seen) = load(&two_packs(), &options, Order::Reverse, None);
    assert_eq!(files(toy).len(), 3);
    assert_eq!(seen.reads.len(), 4, "{:?}", seen.reads);
    let top = &set.behavior.instances()[0].pack;
    for path in ["entities/pig.json", "textures/pig.png", "manifest.json"] {
        assert!(top.is_resident(&PackPath::from_static(path)), "{path}");
    }
    assert_eq!(
        top.read(&PackPath::from_static("textures/pig.png"))
            .unwrap(),
        &b"png"[..]
    );
}

#[test]
fn every_completion_order_fills_the_same_slots() {
    let reference = format!(
        "{:?}",
        files(load(&two_packs(), &StackOptions::default(), Order::Issue, None).0)
    );
    for order in ORDERS {
        let (toy, ..) = load(&two_packs(), &StackOptions::default(), order, None);
        assert_eq!(format!("{:?}", files(toy)), reference, "{order:?}");
    }
}

#[test]
fn a_brarchive_is_read_and_indexed_before_the_files_it_holds_parse() {
    let mut tree = two_packs();
    let archive = brarchive::serialize([
        ("horse.json", &b"horse"[..]),
        ("pig.json", &b"archived pig"[..]),
    ])
    .unwrap();
    tree.file(1, "__brarchive/entities.brarchive", archive);
    for order in ORDERS {
        let (toy, set, seen) = load(&tree, &StackOptions::default(), order, None);
        assert_eq!(
            files(toy),
            [
                (Some(1), "entities/a/b.json".to_owned(), "b".to_owned()),
                (Some(1), "entities/cow.json".to_owned(), "cow".to_owned()),
                (
                    Some(0),
                    "entities/horse.json".to_owned(),
                    "horse".to_owned()
                ),
                (
                    Some(0),
                    "entities/pig.json".to_owned(),
                    "archived pig".to_owned()
                ),
            ],
            "{order:?}"
        );
        assert!(
            seen.reads
                .iter()
                .any(|place| place.path.as_str() == "__brarchive/entities.brarchive")
        );
        // The pack's index is the union, and the archive stays resident.
        let top = &set.behavior.instances()[0].pack;
        assert!(
            top.paths()
                .any(|path| path.as_str() == "entities/horse.json")
        );
        assert!(top.is_resident(&PackPath::from_static("entities/horse.json")));
    }
}

#[test]
fn the_read_window_bounds_reads_and_an_oversized_file_goes_alone() {
    let mut tree = Tree::default();
    tree.file(1, "manifest.json", manifest(1));
    for n in 0..10 {
        tree.file(1, &format!("entities/e{n}.json"), "x".repeat(10));
    }
    tree.file(1, "entities/huge.json", "x".repeat(1000));
    let options = StackOptions {
        limits: Limits {
            max_pending_reads: 3,
            max_pending_read_bytes: 25,
            ..Limits::default()
        },
        ..StackOptions::default()
    };
    let toy = Toy::leak(PackSide::Behavior);
    let set = stack_set(vec![tree.open(1)]);
    let mut loader = StackLoader::with_table(set, &options, vec![toy]);
    let mut seen = Seen::default();
    tree.drive(&mut loader, Order::Reverse, None, &mut seen);
    assert_eq!(files(toy).len(), 11);
    // Reads are charged by their listed size and stay charged until parsed (`Transient`): at
    // most two 10-byte files fit the 25-byte window, and the 1000-byte file goes alone.
    assert_eq!(seen.max_reads_out, 2, "{seen:?}");
    assert_eq!(seen.max_bytes_out, 1000, "{seen:?}");
}

#[test]
fn a_transient_read_stays_charged_until_its_file_is_parsed() {
    let mut tree = Tree::default();
    tree.file(1, "manifest.json", manifest(1));
    for n in 0..10 {
        tree.file(1, &format!("entities/e{n}.json"), "x".repeat(10));
    }
    let limits = Limits {
        max_pending_read_bytes: 25,
        ..Limits::default()
    };
    for (residency, most) in [(Residency::Transient, 2), (Residency::All, 10)] {
        let options = StackOptions {
            behavior: residency,
            limits,
            ..StackOptions::default()
        };
        let toy = Toy::leak(PackSide::Behavior);
        let mut loader =
            StackLoader::with_table(stack_set(vec![tree.open(1)]), &options, vec![toy]);
        let mut seen = Seen::default();
        // Every read is answered before any job runs: only the parse frees a `Transient` charge.
        tree.drive(&mut loader, Order::RequestsFirst, None, &mut seen);
        assert_eq!(files(toy).len(), 10);
        assert_eq!(seen.max_unparsed, most, "{residency:?}: {seen:?}");
    }
}

#[test]
fn outstanding_jobs_stay_within_max_pending_jobs() {
    let mut tree = Tree::default();
    tree.file(1, "manifest.json", manifest(1));
    for n in 0..10 {
        tree.file(1, &format!("entities/e{n}.json"), "x");
    }
    let options = StackOptions {
        limits: Limits {
            max_pending_jobs: 2,
            ..Limits::default()
        },
        ..StackOptions::default()
    };
    let toy = Toy::leak(PackSide::Behavior);
    let mut loader = StackLoader::with_table(stack_set(vec![tree.open(1)]), &options, vec![toy]);
    let mut seen = Seen::default();
    tree.drive(&mut loader, Order::RequestsFirst, None, &mut seen);
    assert_eq!(files(toy).len(), 10);
    assert_eq!(seen.max_jobs_out, 2, "{seen:?}");
}

#[test]
fn read_and_brarchive_failures_report_the_same_in_every_order() {
    let mut tree = two_packs();
    let archive = brarchive::serialize([("horse.json", &b"horse"[..])]).unwrap();
    tree.file(1, "__brarchive/entities.brarchive", archive)
        .file(2, "__brarchive/a.brarchive", "not a brarchive")
        .file(2, "__brarchive/b.brarchive", "unreadable");
    tree.failed.insert(
        Place::root(RootId(2)).join(&PackPath::from_static("__brarchive/b.brarchive")),
        std::io::ErrorKind::PermissionDenied,
    );
    tree.failed.insert(
        Place::root(RootId(2)).join(&PackPath::from_static("entities/cow.json")),
        std::io::ErrorKind::PermissionDenied,
    );
    let describe = |order: Order| {
        let (toy, set, _) = load(&tree, &StackOptions::default(), order, None);
        let reports: Vec<String> = set
            .behavior
            .instances()
            .iter()
            .map(|instance| format!("{:?}", instance.pack.report().diagnostics()))
            .collect();
        (format!("{:?}", toy.cache.get().unwrap().findings), reports)
    };
    let reference = describe(Order::Issue);
    assert!(
        reference.0.contains("bp 2"),
        "the read error names its pack: {}",
        reference.0
    );
    assert!(reference.1[1].contains("a.brarchive") && reference.1[1].contains("b.brarchive"));
    for order in [
        Order::Reverse,
        Order::Seeded(0),
        Order::Seeded(5),
        Order::Seeded(77),
        Order::RequestsFirst,
    ] {
        assert_eq!(describe(order), reference, "{order:?}");
    }
}

#[test]
fn a_failed_read_skips_the_file_with_a_read_error() {
    let mut tree = two_packs();
    tree.failed.insert(
        Place::root(RootId(2)).join(&PackPath::from_static("entities/cow.json")),
        std::io::ErrorKind::PermissionDenied,
    );
    let (toy, ..) = load(&tree, &StackOptions::default(), Order::Seeded(5), None);
    assert_eq!(files(toy).len(), 2);
    let findings = &toy.cache.get().unwrap().findings;
    let codes: Vec<_> = findings.iter().map(|d| &d.code).collect();
    assert!(matches!(
        codes[..],
        [DiagnosticCode::ReadError {
            kind: std::io::ErrorKind::PermissionDenied,
            ..
        }]
    ));
    let finding = findings.iter().next().unwrap();
    assert_eq!(
        finding.location.pack.as_ref().and_then(PackRef::index),
        Some(1)
    );
    assert_eq!(
        finding.location.path.as_ref().map(PackPath::as_str),
        Some("entities/cow.json")
    );
}

/// A walk with a text for an unreadable file (`Walk::read_error`): the driver's `ReadError`
/// carries it with the file's path in its pack, in every completion order; the code,
/// severity, pack and path are the ones a walk without a text gets.
#[test]
fn a_failed_read_carries_the_walk_s_text_for_the_file() {
    let mut tree = two_packs();
    tree.failed.insert(
        Place::root(RootId(2)).join(&PackPath::from_static("entities/cow.json")),
        std::io::ErrorKind::PermissionDenied,
    );
    for order in ORDERS {
        let toy: &'static Toy = Box::leak(Box::new(Toy {
            side: PackSide::Behavior,
            cache: OnceLock::new(),
            combined: Mutex::new(0),
            directories: &["entities"],
            then_keys: &[],
            then_saw: Mutex::new(None),
            read_error: Some("Failed to load the contents of Toy file {}; it will be skipped."),
        }));
        let set = stack_set(vec![tree.open(1), tree.open(2)]);
        let mut loader = StackLoader::with_table(set, &StackOptions::default(), vec![toy]);
        tree.drive(&mut loader, order, None, &mut Seen::default());
        let findings = &toy.cache.get().unwrap().findings;
        let found: Vec<_> = findings
            .iter()
            .map(|d| {
                (
                    d.code.to_string(),
                    matches!(
                        d.code,
                        DiagnosticCode::ReadError {
                            kind: std::io::ErrorKind::PermissionDenied,
                            ..
                        }
                    ),
                    d.severity,
                    d.location.pack.as_ref().and_then(PackRef::index),
                    d.location.path.as_ref().map(|p| p.as_str().to_owned()),
                )
            })
            .collect();
        assert_eq!(
            found,
            [(
                "Failed to load the contents of Toy file entities/cow.json; it will be skipped."
                    .to_owned(),
                true,
                bedrock_addon_core::diagnostic::Severity::Error,
                Some(1),
                Some("entities/cow.json".to_owned()),
            )],
            "{order:?}"
        );
    }
}

#[test]
fn a_panicked_parse_rejects_its_file_and_a_panicked_combine_installs_it_empty() {
    let (toy, ..) = load(
        &two_packs(),
        &StackOptions::default(),
        Order::Issue,
        Some(JobKind::Parse),
    );
    assert!(files(toy).is_empty());
    let findings = &toy.cache.get().unwrap().findings;
    assert_eq!(findings.len(), 3);
    assert!(
        findings
            .iter()
            .all(|d| d.code == DiagnosticCode::InternalError)
    );

    // The first `Combine` panics; the replacement over no files installs the empty collection.
    let toy = Toy::leak(PackSide::Behavior);
    let tree = two_packs();
    let set = stack_set(vec![tree.open(1), tree.open(2)]);
    let mut loader = StackLoader::with_table(set, &StackOptions::default(), vec![toy]);
    let mut seen = Seen {
        panic_limit: Some(1),
        ..Seen::default()
    };
    tree.drive(&mut loader, Order::Issue, Some(JobKind::Combine), &mut seen);
    assert!(files(toy).is_empty());
    let codes: Vec<_> = toy
        .cache
        .get()
        .unwrap()
        .findings
        .iter()
        .map(|d| &d.code)
        .collect();
    assert_eq!(codes, [&DiagnosticCode::InternalError]);
}

#[test]
fn an_indexed_file_of_a_memory_pack_is_not_loaded() {
    let pack = crate::pack::tests::pack_from(
        &manifest(3),
        &[("entities/a.json", Some(b"a")), ("entities/b.json", None)],
        PackOrigin::World,
    );
    let toy = Toy::leak(PackSide::Behavior);
    let mut loader =
        StackLoader::with_table(stack_set(vec![pack]), &StackOptions::default(), vec![toy]);
    let mut seen = Seen::default();
    Tree::default().drive(&mut loader, Order::Issue, None, &mut seen);
    assert!(seen.reads.is_empty(), "a memory pack is never read");
    assert_eq!(files(toy).len(), 1);
    let codes: Vec<_> = toy
        .cache
        .get()
        .unwrap()
        .findings
        .iter()
        .map(|d| &d.code)
        .collect();
    assert_eq!(codes, [&DiagnosticCode::FileNotLoaded]);
}

#[test]
fn an_mcb_document_is_an_unsupported_file_format() {
    let pack = crate::pack::tests::pack_from(
        &manifest(3),
        &[(
            "entities/a.json",
            Some(b"\x7fMCB\x01\x00\x1a\x00\x1e\x00\x00\x00\x0eminecraft:item"),
        )],
        PackOrigin::World,
    );
    let toy = Toy::leak(PackSide::Behavior);
    let mut loader =
        StackLoader::with_table(stack_set(vec![pack]), &StackOptions::default(), vec![toy]);
    Tree::default().drive(&mut loader, Order::Issue, None, &mut Seen::default());
    let codes: Vec<_> = toy
        .cache
        .get()
        .unwrap()
        .findings
        .iter()
        .map(|d| &d.code)
        .collect();
    assert_eq!(codes, [&DiagnosticCode::UnsupportedFileFormat]);
}

#[test]
fn a_panicked_brarchive_index_job_leaves_the_archive_absent() {
    let mut tree = two_packs();
    let archive = brarchive::serialize([("horse.json", &b"horse"[..])]).unwrap();
    tree.file(1, "__brarchive/entities.brarchive", archive);
    let (toy, set, _) = load(
        &tree,
        &StackOptions::default(),
        Order::Issue,
        Some(JobKind::Index),
    );
    assert_eq!(files(toy).len(), 3, "the horse is not walked");
    let top = &set.behavior.instances()[0].pack;
    let codes: Vec<_> = top.report().diagnostics().iter().map(|d| &d.code).collect();
    assert_eq!(codes, [&DiagnosticCode::InternalError]);
}

#[test]
fn a_dropped_stack_loader_cancels_its_parse_jobs() {
    let tree = two_packs();
    let toy = Toy::leak(PackSide::Behavior);
    let set = stack_set(vec![tree.open(1), tree.open(2)]);
    let mut loader = StackLoader::with_table(set, &StackOptions::default(), vec![toy]);
    let job = loop {
        match loader.poll() {
            Some(Output::Request(request)) => {
                let input = tree.answer(&request);
                loader.handle(input).unwrap();
            }
            Some(Output::Job(job)) => break job,
            other => panic!("{other:?}"),
        }
    };
    assert_eq!(job.kind(), JobKind::Parse);
    drop(loader);
    let done = job.run();
    assert!(done.outcome.is_none(), "the job returned without working");
    assert!(toy.cache.get().is_none());
}

#[test]
fn progress_counts_the_whole_load() {
    let tree = two_packs();
    let toy = Toy::leak(PackSide::Behavior);
    let set = stack_set(vec![tree.open(1), tree.open(2)]);
    let mut loader = StackLoader::with_table(set, &StackOptions::default(), vec![toy]);
    assert_ne!(loader.progress().stage, super::Stage::Finished);
    tree.drive(&mut loader, Order::Issue, None, &mut Seen::default());
    let progress = loader.progress();
    assert_eq!(progress.stage, super::Stage::Finished);
    assert_eq!(progress.requests_issued, 3);
    assert_eq!(progress.requests_answered, 3);
    assert_eq!(progress.bytes_received, 3 + 3 + 1);
    assert_eq!(progress.jobs_issued, 4);
    assert_eq!(progress.jobs_done, 4);
}

// ---- the second walk stage (`CollectionLoader::then`) ----

/// `two_packs` with, in both packs, a file a second stage names and one it does not.
fn two_packs_with_extras() -> Tree {
    let mut tree = two_packs();
    tree.file(1, "extra/x.json", "x1")
        .file(1, "extra/unnamed.json", "u1")
        .file(2, "extra/x.json", "x2")
        .file(2, "extra/unnamed.json", "u2");
    tree
}

/// The keys the toy's second stage names: a file both packs have, one the walk read
/// already, one no pack has, and the first again.
const NAMED: &[&str] = &[
    "extra/x.json",
    "entities/pig.json",
    "extra/missing.json",
    "extra/x.json",
];

#[test]
fn a_second_stage_reads_the_named_files_after_the_walk_s_in_every_order() {
    let tree = two_packs_with_extras();
    let expected = [
        (Some(1), "entities/a/b.json", "b"),
        (Some(1), "entities/cow.json", "cow"),
        (Some(0), "entities/pig.json", "pig"),
        // The second stage, lowest pack first; the file the walk read is not read again.
        (Some(1), "extra/x.json", "x2"),
        (Some(0), "extra/x.json", "x1"),
    ]
    .map(|(pack, path, text)| (pack, path.to_owned(), text.to_owned()));
    for order in ORDERS.into_iter().chain([Order::RequestsFirst]) {
        for residency in [Residency::Transient, Residency::All] {
            let toy = Toy::leak_with(PackSide::Behavior, &["entities"], NAMED);
            let set = stack_set(vec![tree.open(1), tree.open(2)]);
            let options = StackOptions {
                behavior: residency,
                ..StackOptions::default()
            };
            let mut loader = StackLoader::with_table(set, &options, vec![toy]);
            let mut seen = Seen::default();
            tree.drive(&mut loader, order, None, &mut seen);
            assert_eq!(files(toy), expected, "{order:?} {residency:?}");
            // The stage was asked once, with every parsed file of the walk.
            assert_eq!(*toy.then_saw.lock().unwrap(), Some(3), "{order:?}");
            assert_eq!(*toy.combined.lock().unwrap(), 1, "{order:?}");
            assert!(toy.cache.get().unwrap().findings.is_empty(), "{order:?}");
            // Each file is asked for once; an unnamed file only where every file is read.
            let asked = |path: &str| {
                let count = |root| {
                    let place = Place::root(RootId(root)).join(&PackPath::new(path).unwrap());
                    seen.reads.iter().filter(|read| **read == place).count()
                };
                (count(1), count(2))
            };
            assert_eq!(asked("extra/x.json"), (1, 1), "{order:?} {residency:?}");
            assert_eq!(
                asked("entities/pig.json"),
                (1, 0),
                "{order:?} {residency:?}"
            );
            let unnamed = usize::from(residency == Residency::All);
            assert_eq!(
                asked("extra/unnamed.json"),
                (unnamed, unnamed),
                "{order:?} {residency:?}"
            );
        }
    }
}

// A file the second stage names may be one another collection walked: read while the loose
// files were queued, and released once that collection parsed it. The late slot gets the same
// bytes all the same.
#[test]
fn a_second_stage_file_another_collection_read_and_released_is_still_parsed() {
    let tree = two_packs_with_extras();
    for order in ORDERS.into_iter().chain([Order::RequestsFirst]) {
        for residency in [Residency::Transient, Residency::All] {
            let walks_extra = Toy::leak_with(PackSide::Behavior, &["extra"], &[]);
            let names_extra = Toy::leak_with(PackSide::Behavior, &["entities"], NAMED);
            let set = stack_set(vec![tree.open(1), tree.open(2)]);
            let options = StackOptions {
                behavior: residency,
                ..StackOptions::default()
            };
            let table: Vec<&'static dyn ErasedCollection> = vec![walks_extra, names_extra];
            let mut loader = StackLoader::with_table(set, &options, table);
            tree.drive(&mut loader, order, None, &mut Seen::default());
            let texts = |toy: &Toy| -> Vec<String> {
                files(toy)
                    .into_iter()
                    .filter(|(_, path, _)| path == "extra/x.json")
                    .map(|(_, _, text)| text)
                    .collect()
            };
            assert_eq!(texts(walks_extra), ["x2", "x1"], "{order:?} {residency:?}");
            assert_eq!(texts(names_extra), ["x2", "x1"], "{order:?} {residency:?}");
            for toy in [walks_extra, names_extra] {
                let findings = &toy.cache.get().unwrap().findings;
                assert!(findings.is_empty(), "{order:?} {residency:?}: {findings:?}");
            }
        }
    }
}

// The job that names the second stage's files holds the walk's parsed files; if it panics
// they are gone, and the collection is installed empty with an internal error, as after a
// panicked `Combine`.
#[test]
fn a_panicked_second_stage_installs_the_collection_empty() {
    // No file of the walk: the stage's job is the only one of its kind.
    let toy = Toy::leak_with(PackSide::Behavior, &["nothing"], NAMED);
    let tree = two_packs_with_extras();
    let set = stack_set(vec![tree.open(1), tree.open(2)]);
    let mut loader = StackLoader::with_table(set, &StackOptions::default(), vec![toy]);
    let mut seen = Seen::default();
    tree.drive(&mut loader, Order::Issue, Some(JobKind::Parse), &mut seen);
    assert!(files(toy).is_empty());
    assert_eq!(*toy.then_saw.lock().unwrap(), None);
    let codes: Vec<_> = toy
        .cache
        .get()
        .unwrap()
        .findings
        .iter()
        .map(|d| &d.code)
        .collect();
    assert_eq!(codes, [&DiagnosticCode::InternalError]);
    // Nothing of the second stage was read.
    assert!(seen.reads.is_empty(), "{:?}", seen.reads);
}

// A second stage that names nothing combines the walk's files as they are.
#[test]
fn a_second_stage_without_a_file_combines_the_walk() {
    let toy = Toy::leak_with(PackSide::Behavior, &["entities"], &["extra/missing.json"]);
    let tree = two_packs_with_extras();
    let set = stack_set(vec![tree.open(1), tree.open(2)]);
    let mut loader = StackLoader::with_table(set, &StackOptions::default(), vec![toy]);
    tree.drive(&mut loader, Order::Issue, None, &mut Seen::default());
    assert_eq!(files(toy).len(), 3);
    assert_eq!(*toy.then_saw.lock().unwrap(), Some(3));
}

/// A stack keeps its caches on the heap. With one inline cache per accessor line a
/// `PackStack` is about 20 KB, a `StackSet` 40 KB and a `StackLoader` 40 KB, and every frame
/// a set is moved through (`PackRepository::stacks`, the loader, the driver, the caller)
/// carries a copy, which overflows a 2 MiB thread stack on the sequential load of a
/// base-game install. A new accessor line must not grow these sizes.
#[test]
fn a_stack_set_and_its_loader_are_small_whatever_the_number_of_accessors() {
    use crate::pack::{Behavior, Resource};
    use std::mem::size_of;

    let sizes = [
        // 240, 240, 480 and 1192 bytes with the caches boxed.
        ("PackStack<Behavior>", size_of::<PackStack<Behavior>>(), 512),
        ("PackStack<Resource>", size_of::<PackStack<Resource>>(), 512),
        ("StackSet", size_of::<StackSet>(), 1024),
        ("StackLoader", size_of::<StackLoader>(), 2048),
    ];
    for (name, size, bound) in sizes {
        assert!(
            size <= bound,
            "{name} is {size} bytes, over {bound}: a cache or a registry is inline again"
        );
    }
}

/// Every loader and the source vocabulary can be printed (the API guideline for `Debug`).
#[test]
fn loaders_and_sources_implement_debug() {
    let pack = PackLoader::directory(RootId::MEMORY, OpenOptions::default());
    let text = format!("{pack:?}");
    assert!(
        text.starts_with("PackLoader") && text.contains("progress"),
        "{text}"
    );
    let repository = super::RepositoryLoader::new(Vec::new(), OpenOptions::default());
    let text = format!("{repository:?}");
    assert!(
        text.starts_with("RepositoryLoader") && text.contains("progress"),
        "{text}"
    );
    let stack = StackLoader::new(stack_set(Vec::new()), StackOptions::default());
    let text = format!("{stack:?}");
    assert!(
        text.starts_with("StackLoader") && text.contains("progress"),
        "{text}"
    );
    let source = super::Source::Packs {
        packs: Vec::new(),
        origin: PackOrigin::World,
    };
    assert!(format!("{:?}", source.clone()).starts_with("Packs"));
}
