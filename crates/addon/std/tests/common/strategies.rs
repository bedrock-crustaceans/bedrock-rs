//! The comparison of `Strategy::Sequential` with `Strategy::Rayon` over one world of the game's
//! packs, for `full_strategies.rs` (the server install and the sample packs),
//! `full_strategies_client_install_a.rs` and `full_strategies_client_install_b.rs` (one client
//! install each): three targets, because a test binary may use 600 s of CPU and the two client
//! installs in one took 436 s of it on a loaded machine. A target that passes 450 s is split,
//! the limit never raised. A test file includes
//! this with
//! `#[path = "common/strategies.rs"] mod strategies;` beside `common` and `collections`.

#![allow(
    dead_code,
    unused_imports,
    reason = "each test file loads its own kind of world; without `rayon` nothing is loaded"
)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use bedrock_addon::load::{CollectionSet, LoadOptions, RECOMMENDED_JOB_STACK, StackOptions};
use bedrock_addon::pack::{PackOrigin, PackStack, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy, read_app_data};

use crate::collections::{self, Snapshot};
use crate::common::world::base_game_world;

// What a `Debug` leaves out, for the same comparison (`unprinted.rs`). The driver crate's
// tests compile every content module of `bedrock_addon` in: every family's block is kept.
macro_rules! family {
    ($feature:literal => $body:block) => {
        $body
    };
    ($feature:literal => $item:item) => {
        $item
    };
}
#[path = "../../../addon/tests/common/unprinted.rs"]
mod unprinted;

/// The accessor lines the driver crate's tests compile in (its dev-dependency turns every
/// content feature on): the variants `CollectionSet::ALL` prints.
const ACCESSORS: usize = 58;
/// The collections of the two stacks: the 53 one-sided accessors, and the five of
/// both sides (animations, animation controllers, languages, translations, scripts) on each.
const COLLECTIONS: usize = 63;

/// One install at a time: a load of a client install and its text are some gigabytes.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// What of the game's files a test loads.
#[derive(Clone)]
pub enum World {
    /// An install: `behavior_packs/`, `resource_packs/` and the app data below `root`.
    Install(PathBuf),
    /// The sample packs: `behavior_pack/` and `resource_pack/` below `root`.
    SamplePacks(PathBuf),
}

/// A driver with `strategy` that opens packs as the app's own.
fn driver(strategy: Strategy) -> Driver {
    let mut options = Options::default().with_strategy(strategy);
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

/// Both stacks of `world`, composed and every collection loaded through a driver with
/// `strategy`.
fn stacks(world: &World, strategy: Strategy) -> StackSet {
    let mut driver = driver(strategy);
    let set = match world {
        World::Install(root) => {
            let repository = driver
                .repository([
                    DirSource::packs(root.join("behavior_packs"), PackOrigin::Package),
                    DirSource::packs(root.join("resource_packs"), PackOrigin::Package),
                ])
                .expect("the repository");
            let options = LoadOptions {
                app_data: Some(read_app_data(root).expect("the app data")),
                ..LoadOptions::default()
            };
            repository.stacks(&base_game_world(&repository, &options), options)
        }
        World::SamplePacks(root) => {
            let mut pack = |name: &str| {
                let dir = root.join(name);
                driver
                    .pack_dir(&dir)
                    .unwrap_or_else(|error| panic!("{}: {error:?}", dir.display()))
            };
            let (behavior, resource) = (pack("behavior_pack"), pack("resource_pack"));
            StackSet {
                behavior: PackStack::of([behavior], LoadOptions::default()).expect("a stack"),
                resource: PackStack::of([resource], LoadOptions::default()).expect("a stack"),
            }
        }
    };
    driver
        .load_stacks(set, StackOptions::default())
        .expect("the stacks load")
}

/// What one load came to, for the line a test prints.
struct Loaded {
    snapshot: Snapshot,
    /// From the driver's creation to the last collection loaded.
    wall: Duration,
    /// Packs on the behaviour and on the resource stack.
    packs: (usize, usize),
    findings: usize,
}

/// Loads `world` with `strategy` on a thread of `RECOMMENDED_JOB_STACK`.
fn load(world: &World, strategy: Strategy) -> Loaded {
    let world = world.clone();
    std::thread::Builder::new()
        .stack_size(RECOMMENDED_JOB_STACK)
        .spawn(move || {
            let started = Instant::now();
            let set = stacks(&world, strategy);
            let report = set.load_all();
            let wall = started.elapsed();
            Loaded {
                snapshot: Snapshot::of(&set, &report).with_unprinted(unprinted::of(&set)),
                wall,
                packs: (set.behavior.packs().count(), set.resource.packs().count()),
                findings: report.iter().count(),
            }
        })
        .expect("a thread")
        .join()
        .expect("the load does not panic")
}

/// Loads `world` sequentially and on rayon and requires the two loads to be the same. Built
/// without the `rayon` feature there is one strategy and nothing to compare: the test says
/// `skipped:` and loads nothing, which is not a pass.
pub fn assert_the_strategies_agree(source: &str, world: &World) {
    #[cfg(not(feature = "rayon"))]
    {
        let _ = world;
        eprintln!(
            "skipped: {source}: built without the `rayon` feature, so there is no second strategy to compare the sequential load with"
        );
    }
    #[cfg(feature = "rayon")]
    compare_the_strategies(source, world);
}

/// The comparison of [`assert_the_strategies_agree`].
#[cfg(feature = "rayon")]
fn compare_the_strategies(source: &str, world: &World) {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let sequential = load(world, Strategy::Sequential);
    let collections = sequential.snapshot.collections();
    let text: usize = collections.iter().map(|entry| entry.text.len()).sum();
    eprintln!(
        "{source}: Sequential {:.1?}: {} + {} packs, {} collections ({} bytes of Debug), {} findings",
        sequential.wall,
        sequential.packs.0,
        sequential.packs.1,
        collections.len(),
        text,
        sequential.findings,
    );
    assert!(sequential.packs.0 > 0 && sequential.packs.1 > 0, "{source}");
    // Something is compared: the stacks list every collection the test crate compiles in, a
    // both-sides accessor on each stack, and the largest hold documents.
    let all = format!("{:?}", CollectionSet::ALL);
    let all: BTreeSet<&str> = all.trim_matches(['{', '}']).split(", ").collect();
    let listed: BTreeSet<&str> = collections.iter().map(|entry| entry.name).collect();
    assert_eq!(listed, all, "{source}: the collections of the two stacks");
    assert_eq!(
        (all.len(), collections.len()),
        (ACCESSORS, COLLECTIONS),
        "{source}"
    );
    assert!(text > 1_000_000, "{source}: {text} bytes of Debug");
    eprintln!(
        "{source}: {} bytes of what linking says",
        sequential.snapshot.link.len()
    );
    // And what a `Debug` leaves out. Which kinds an install has differs (a server ships no
    // materials, the sample packs no structures): the line says. The last kind is one value
    // per registry and path map: its whole table of layers, the beta registry's included.
    let unprinted = &sequential.snapshot.unprinted;
    let of_kind = |kind: &str| {
        unprinted
            .iter()
            .filter(|value| value.what.starts_with(kind))
            .count()
    };
    let counts = unprinted::KINDS.map(of_kind);
    eprintln!(
        "{source}: {} values whose Debug leaves a part out ({} bytes), by kind {counts:?}",
        unprinted.len(),
        unprinted
            .iter()
            .map(|value| value.content.len())
            .sum::<usize>(),
    );
    assert_eq!(unprinted.len(), counts.iter().sum::<usize>(), "{source}");
    let beta = unprinted
        .iter()
        .filter(|value| value.what.starts_with(unprinted::LAYERS))
        .map(|value| {
            String::from_utf8_lossy(&value.content)
                .lines()
                .filter(|line| line.starts_with("beta Id<"))
                .count()
        })
        .sum::<usize>();
    eprintln!("{source}: {beta} identifiers of beta registries in them");
    let parallel = load(world, Strategy::Rayon { threads: None });
    eprintln!(
        "{source}: Rayon {:.1?} on {} threads",
        parallel.wall,
        rayon::current_num_threads()
    );
    collections::assert_same(source, &parallel.snapshot, &sequential.snapshot);
    eprintln!(
        "{source}: Sequential and Rayon agree on the report and on {} collections",
        collections.len()
    );
}

pub fn install(root: &Path) -> World {
    World::Install(root.to_path_buf())
}
