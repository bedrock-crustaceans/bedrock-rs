//! What the full-stack smoke binaries (`full*.rs`) share: the trees of the game's packs, how one
//! is loaded as a world, what a load came to as numbers ([`Facts`]), and the pin files those
//! numbers are compared with.
//!
//! A binary includes this file with `#[path = "full/support.rs"] mod support;` beside
//! `#[macro_use] mod common;`.
//!
//! # The pin files
//!
//! `tests/full/pins/<tree>.<aspect>.txt` holds one count per line: the number, the key it
//! counts and, after ` # `, a sentence about it. Lines that start with `#` are the file's
//! header. A test builds the same keys from a load and requires the two maps to be equal; the
//! message names every key that differs. With `BEDROCK_ADDON_CORPUS_PIN=write` in the
//! environment the file is written from the load instead (the sentences of the keys that stay
//! are kept, a new key gets none) and the test passes: that is how the numbers are taken
//! again after the game's files or a rule changed. A pin file whose aspect needs a sentence beside
//! every number (the link pass) fails while one is missing.
//!
//! Nothing of the game's files is in a pin file but counts, the ids of diagnostic definitions and
//! of reference kinds, the names of folders, and a sentence per number.

#![allow(
    dead_code,
    unused_macros,
    reason = "each binary uses the part its tree needs"
)]

use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Write as _};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Severity};
use bedrock_addon::id::Kind;
use bedrock_addon::load::{
    BuiltIns, Experiment, Experiments, LoadOptions, RECOMMENDED_JOB_STACK, Residency, StackOptions,
};
use bedrock_addon::pack::{LinkOptions, PackOrigin, PackStack, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy, read_app_data};

use crate::common::game_files;
use crate::common::world::base_game_world;

// ---- the trees ------------------------------------------------------------------------------

/// One tree of the game's packs that is loaded as a world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Tree {
    /// The stem of its pin files.
    pub name: &'static str,
    /// The folder of the game's files it is below.
    pub entry: &'static str,
    /// The folder below the entry that holds `behavior_packs/` and `resource_packs/`, or
    /// `""`.
    pub below: &'static str,
    /// The sample packs: one `behavior_pack/` and one `resource_pack/`, no install.
    pub samples: bool,
    /// A client's install: its default world has the client's base-game resource stack. A
    /// server's has no resource pack of the game on it: a server never lists vanilla, slice,
    /// experimental, platform or library packs.
    pub client: bool,
}

/// The Bedrock Dedicated Server install.
pub const SERVER_INSTALL: Tree = Tree {
    name: "server_install",
    entry: game_files::SERVER_INSTALL,
    below: "",
    samples: false,
    client: false,
};
/// The first client install.
pub const CLIENT_INSTALL_A: Tree = Tree {
    name: "client_install_a",
    entry: game_files::CLIENT_INSTALL_A,
    below: "",
    samples: false,
    client: true,
};
/// The second client install.
pub const CLIENT_INSTALL_B: Tree = Tree {
    name: "client_install_b",
    entry: game_files::CLIENT_INSTALL_B,
    below: "",
    samples: false,
    client: true,
};
/// The sample packs.
pub const SAMPLES: Tree = Tree {
    name: "samples",
    entry: game_files::SAMPLE_PACKS,
    below: "",
    samples: true,
    client: true,
};
/// The server half of the split client pack tree.
pub const SPLIT_SERVER: Tree = Tree {
    name: "split_server",
    entry: game_files::SPLIT_TREE,
    below: "server",
    samples: false,
    client: false,
};
/// The client half of the split client pack tree.
pub const SPLIT_CLIENT: Tree = Tree {
    name: "split_client",
    entry: game_files::SPLIT_TREE,
    below: "client",
    samples: false,
    client: true,
};

impl Tree {
    /// Where the tree is, when the folder has it.
    pub fn root(&self) -> Option<PathBuf> {
        let entry = game_files::source(self.entry)?;
        let root = if self.below.is_empty() {
            entry
        } else {
            entry.join(self.below)
        };
        root.exists().then_some(root)
    }
}

// ---- one load -------------------------------------------------------------------------------

/// How a tree is loaded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent switches of one load, each a column of the pin files"
)]
pub struct Run {
    /// `Strategy::Rayon` in place of `Strategy::Sequential`.
    pub rayon: bool,
    /// Every experiment a world can turn on, the beta APIs and the upcoming creator features.
    pub experiments: bool,
    /// A host that answers for the kinds the crate has no list of ([`TheGameHasIt`]).
    pub host: bool,
    /// `Residency::All` on both stacks: every file is read and kept.
    pub everything_resident: bool,
    /// The world a fresh install has ([`default_world`]) in place of the world that lists
    /// every pack of the install.
    pub default_world: bool,
}

impl Run {
    /// The way a server loads: sequentially here, no experiment, no host, the bytes of a
    /// file released once it is parsed.
    pub const SERVER: Run = Run {
        rayon: false,
        experiments: false,
        host: false,
        everything_resident: false,
        default_world: false,
    };

    /// A word for the pin file of this run's aspect.
    pub fn word(self) -> &'static str {
        if self.default_world {
            return "default_world";
        }
        match (self.experiments, self.host, self.everything_resident) {
            (false, false, false) => "default",
            (true, false, false) => "experiments",
            (false, true, false) => "host",
            (false, false, true) => "resident",
            _ => "other",
        }
    }
}

/// Every experiment a world can turn on, and the two toggles.
pub fn everything_on() -> (Experiments, bool, bool) {
    let mut experiments = Experiments::new();
    for experiment in Experiment::ALL {
        if experiment.save_name().is_some() {
            experiments.insert(*experiment);
        }
    }
    (experiments, true, true)
}

/// A host for the link pass that answers for the game's own content where the crate has no
/// list: for a kind the crate was generated a list of (`block`, `item`, `entity`, `biome`,
/// `dimension`, `camera preset`, `jigsaw structure`) it answers from that list, as the pass
/// does without a host; for every other kind it says the game has the name.
#[derive(Debug)]
pub struct TheGameHasIt;

impl BuiltIns for TheGameHasIt {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        use bedrock_addon::id::kind as k;
        let ids = bedrock_addon::vanilla::ids();
        let listed = |list: &[&str]| list.contains(&id);
        if kind == k::Block::NAME {
            listed(ids.blocks)
        } else if kind == k::Item::NAME {
            // Every block has an item of its name.
            listed(ids.items) || listed(ids.blocks)
        } else if kind == k::Entity::NAME {
            listed(ids.entities)
        } else if kind == k::Biome::NAME {
            listed(ids.biomes)
        } else if kind == k::Dimension::NAME {
            listed(ids.dimensions)
        } else if kind == k::CameraPreset::NAME {
            listed(ids.camera_presets)
        } else if kind == k::JigsawStructure::NAME {
            listed(ids.features)
        } else {
            true
        }
    }
}

/// The load options of `run` for the tree at `root`.
pub fn load_options(tree: &Tree, root: &Path, run: Run) -> LoadOptions {
    let mut options = LoadOptions::default();
    if !tree.samples && root.join("definitions").is_dir() {
        options.app_data = Some(read_app_data(root).expect("the app data"));
    }
    if run.experiments {
        (
            options.experiments,
            options.beta_apis,
            options.upcoming_creator_features,
        ) = everything_on();
    }
    if run.host {
        options.built_ins = Some(Arc::new(TheGameHasIt));
    }
    // Every Molang field is compiled, as the game compiles it.
    options.molang = Some(Arc::new(bedrock_addon::molang::MolangCheck::new()));
    options
}

/// A driver that opens packs as the app's own.
pub fn driver(rayon: bool) -> Driver {
    #[cfg(feature = "rayon")]
    let strategy = if rayon {
        Strategy::Rayon { threads: None }
    } else {
        Strategy::Sequential
    };
    #[cfg(not(feature = "rayon"))]
    let strategy = {
        assert!(!rayon, "built without the `rayon` feature");
        Strategy::Sequential
    };
    let mut options = Options::default().with_strategy(strategy);
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

/// Both stacks of the tree at `root`, composed the way the game composes them and every
/// collection loaded through a driver: an install's packs registered as the app's own, the
/// world's lists by `base_game_world` (the base-game layer in vanilla's order, every other
/// pack of the install above it) with the install's app data; the sample packs as their two
/// packs.
pub fn stacks(tree: &Tree, root: &Path, run: Run) -> StackSet {
    let mut driver = driver(run.rayon);
    let options = load_options(tree, root, run);
    let set = if tree.samples {
        let mut pack = |name: &str| {
            let dir = root.join(name);
            driver
                .pack_dir(&dir)
                .unwrap_or_else(|error| panic!("{}: {error:?}", dir.display()))
        };
        let (behavior, resource) = (pack("behavior_pack"), pack("resource_pack"));
        StackSet {
            behavior: PackStack::of([behavior], options.clone()).expect("a stack"),
            resource: PackStack::of([resource], options).expect("a stack"),
        }
    } else {
        let repository = driver
            .repository([
                DirSource::packs(root.join("behavior_packs"), PackOrigin::Package),
                DirSource::packs(root.join("resource_packs"), PackOrigin::Package),
            ])
            .expect("the repository");
        assert!(
            repository.invalid().is_empty(),
            "{}: packs that do not open: {:?}",
            tree.name,
            repository.invalid()
        );
        let world = if run.default_world {
            default_world(tree, root, &repository)
        } else {
            without_inactive_experimental_packs(base_game_world(&repository, &options), &options)
        };
        repository.stacks(&world, options)
    };
    let residency = if run.everything_resident {
        Residency::All
    } else {
        Residency::Transient
    };
    driver
        .load_stacks(
            set,
            StackOptions {
                behavior: residency,
                resource: residency,
                ..StackOptions::default()
            },
        )
        .expect("the stacks load")
}

/// `world` without the vanilla experimental packs (`VANILLA_EXPERIMENTAL_PACKS`) whose
/// experiment `options` leaves off. The game puts such a pack on a stack only while its
/// experiment is on (the composer's experiment gate, as for the base-game layer), so with the toggle off none of its documents is read. A run with
/// every experiment on keeps them all, on the layer, with their toggles.
pub fn without_inactive_experimental_packs(
    mut world: bedrock_addon::pack::WorldPacks,
    options: &LoadOptions,
) -> bedrock_addon::pack::WorldPacks {
    let inactive = |selection: &bedrock_addon::pack::PackSelection| {
        bedrock_addon::pack::VANILLA_EXPERIMENTAL_PACKS
            .iter()
            .any(|(uuid, _, experiment)| {
                *uuid == selection.pack_id && !options.experiments.contains(*experiment)
            })
    };
    world.behavior.retain(|selection| !inactive(selection));
    world.resource.retain(|selection| !inactive(selection));
    world
}

/// The world a fresh install of the tree has, by default: what the game loads when nobody
/// chose a pack.
///
/// - The world's own lists are those of the install's world when it has one (the first folder
///   of `worlds/`, its `world_behavior_packs.json` and `world_resource_packs.json`; a missing
///   file is an empty list, as in vanilla), else empty. The server install's `Bedrock level`
///   has neither file, and no other tree has a world: every list is empty.
/// - Below them the composer adds the behaviour base-game layer in the game's order: the packs
///   whose experiment is on (none), the script libraries, the `vanilla` slices, `vanilla`.
///   No education feature is on, so no chemistry pack is on the stack (they
///   join a world's stack only when the world has the education features on), and no editor
///   pack.
/// - A client's install gets the client's base-game resource stack on the resource list
///   (`common::world::client_resource_layer`); a server's world lists no resource pack.
pub fn default_world(
    tree: &Tree,
    root: &Path,
    repository: &bedrock_addon::pack::PackRepository,
) -> bedrock_addon::pack::WorldPacks {
    let mut worlds: Vec<PathBuf> = std::fs::read_dir(root.join("worlds"))
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .filter(|path| path.is_dir())
                .collect()
        })
        .unwrap_or_default();
    worlds.sort();
    let mut world = match worlds.first() {
        Some(dir) => bedrock_addon_std::read_world_packs(dir).expect("the world's pack lists"),
        None => bedrock_addon::pack::WorldPacks {
            behavior: Vec::new(),
            resource: Vec::new(),
        },
    };
    if tree.client {
        world
            .resource
            .extend(crate::common::world::client_resource_layer(repository));
    }
    world
}

// ---- what a load came to --------------------------------------------------------------------

/// Counts by key, as a pin file holds them.
pub type Counts = BTreeMap<String, usize>;

/// A finding as two loads are compared by: code, severity, file and place in the file.
pub type Finding = (String, String, String, String);

/// What a loaded and linked world came to.
#[derive(Clone, Debug, Default)]
pub struct Facts {
    /// Packs on the behaviour and on the resource stack.
    pub packs: (usize, usize),
    /// What each collection of each stack holds ([`documents`]).
    pub documents: Counts,
    /// The findings of the load by severity, code and the folder of the file ([`load_key`]).
    pub load: Counts,
    /// The findings of the load, sorted.
    pub findings: Vec<Finding>,
    /// The error-level findings of the load, one line each.
    pub errors: Vec<String>,
    /// The error-level findings of the load by code and file ([`error_key`]).
    pub error_counts: Counts,
    /// Every finding of the load and of the link pass, one line each, when the findings are
    /// to be written out ([`DUMP`]); else empty.
    pub lines: (Vec<String>, Vec<String>),
    /// The findings of `link()` by severity, code, reference kind and folder ([`link_key`]).
    pub link: Counts,
    /// `Debug` of what `link()` returned.
    pub link_debug: String,
    /// The findings of `link()` that refuse something, one line each ([`refusing_lines`]): a kept error is none.
    pub link_errors: Vec<String>,
    /// The error-level findings of `link()`, kept ones included, one line each
    /// ([`error_level_lines`]).
    pub link_error_level: Vec<String>,
    /// `Debug` of what `link_with(&LinkOptions { lints: false, ..Default::default() })` returned.
    pub link_without_lints: String,
    /// `Debug` of the findings of `link()` above the information level, in order.
    pub link_above_info: String,
    /// The findings of the load whose text is vanilla's "Filter test not found".
    pub unknown_filter_tests: usize,
    /// The wall time of the load (composition included) and of `link()`.
    pub wall: (Duration, Duration),
}

/// `E`, `W`, `I` or `D`.
fn letter(severity: Severity) -> char {
    match severity {
        Severity::Error => 'E',
        Severity::Warning => 'W',
        Severity::Info => 'I',
        Severity::Debug => 'D',
    }
}

/// The folder a finding's file is in, the family's own: the first segment of its path, the
/// folder of a subpack left out; the file's name for a file at the root of a pack; `-` for a
/// finding that names no file.
fn folder(finding: &Diagnostic) -> String {
    let Some(path) = &finding.location.path else {
        return String::from("-");
    };
    let path = path.as_str();
    let path = path
        .strip_prefix("subpacks/")
        .and_then(|rest| rest.split_once('/'))
        .map_or(path, |(_, rest)| rest);
    path.split('/').next().unwrap_or(path).to_owned()
}

/// The environment variable that points the pin files at another folder than
/// `tests/full/pins`: the scratch run of the pin rename writes there, never into the repository.
const PIN_DIR: &str = "BEDROCK_ADDON_CORPUS_PIN_DIR";

/// The key of a finding of the load: `W core/deprecated_member @ entities`.
pub fn load_key(finding: &Diagnostic) -> String {
    format!(
        "{} {} @ {}",
        letter(finding.severity),
        finding.code.id(),
        folder(finding)
    )
}

/// The key of a finding of the link pass: `W core/unresolved_reference [material] @ particles`. The
/// kind is the kind of the target the reference names, the folder the family that holds the
/// reference: together with the code they name the rule.
pub fn link_key(finding: &Diagnostic) -> String {
    let kind = match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, .. }) => kind,
        _ => "-",
    };
    format!(
        "{} {} [{}] @ {}",
        letter(finding.severity),
        finding.code.id(),
        kind,
        folder(finding)
    )
}

/// The key of an error-level finding of the load: `core/unrecognized_field @ entity/husk.entity.json`,
/// the code and the file; `-` for a finding that names no file.
pub fn error_key(finding: &Diagnostic) -> String {
    format!(
        "{} @ {}",
        finding.code.id(),
        finding
            .location
            .path
            .as_ref()
            .map_or("-", |path| path.as_str())
    )
}

/// One line for each finding that refuses something: an error-level finding that is not a
/// kept error (a kept error refuses nothing: the game logs it and goes on). What a check that
/// wants "no error" asks, as against the pinned error counts, which list the kept errors with a
/// sentence saying why each is reported.
pub fn refusing_lines<'a>(findings: impl IntoIterator<Item = &'a Diagnostic>) -> Vec<String> {
    findings
        .into_iter()
        .filter(|finding| finding.rejects())
        .map(line)
        .collect()
}

/// The lines of the error-level findings of `findings`, a kept error included.
pub fn error_level_lines<'a>(findings: impl IntoIterator<Item = &'a Diagnostic>) -> Vec<String> {
    findings
        .into_iter()
        .filter(|finding| finding.severity >= Severity::Error)
        .map(line)
        .collect()
}

/// `findings` counted by `key`.
pub fn tally<'a>(
    findings: impl IntoIterator<Item = &'a Diagnostic>,
    key: fn(&Diagnostic) -> String,
) -> Counts {
    let mut counts = Counts::new();
    for finding in findings {
        *counts.entry(key(finding)).or_insert(0) += 1;
    }
    counts
}

/// One line for a finding: severity and code, its text, the pack (place on its stack and
/// name), the file and the place in the file.
pub fn line(finding: &Diagnostic) -> String {
    let location = &finding.location;
    format!(
        "{} {} | {} | {} | {} | {}",
        letter(finding.severity),
        finding.code.id(),
        finding,
        location
            .pack
            .as_ref()
            .map_or_else(String::new, |pack| format!(
                "{}:{} {}",
                pack.index()
                    .map_or_else(String::new, |index| index.to_string()),
                pack.name(),
                pack.id().version
            )),
        location
            .path
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        location.json_path
    )
}

fn comparable(finding: &Diagnostic) -> Finding {
    (
        finding.code.id().to_owned(),
        format!("{:?}", finding.severity),
        finding
            .location
            .path
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        finding.location.json_path.to_string(),
    )
}

/// How often `Document {` stands in the `Debug` of `collection`: the documents a group type
/// holds, whatever its fields are.
fn documents_in(collection: &impl Debug) -> usize {
    format!("{collection:?}").matches("Document {").count()
}

/// What each collection of both stacks holds, by `<side> <accessor> <what>`:
///
/// - a registry or a path map: `documents` (the entries of `iter()`: one per identifier or
///   path, the winning copy) and `rejected` (the copies refused);
/// - a merged value: `layers` (the files that went into it);
/// - a group type (several registries and merged values behind one accessor): `documents`,
///   the count of `Document {` in its `Debug`;
/// - the scripts: `bundles`, and `plugins` (the bundles whose module starts).
///
/// Every accessor of the stacks is listed here by hand; the test
/// `every_collection_of_the_stacks_is_counted` of `full.rs` requires the list to be the stacks' own.
#[allow(clippy::too_many_lines, reason = "one line an accessor")]
pub fn documents(set: &StackSet) -> Counts {
    let mut out = Counts::new();
    macro_rules! count {
        (@entries $side:ident $name:ident) => {{
            let collection = set.$side.$name();
            let key = concat!(stringify!($side), " ", stringify!($name));
            out.insert(format!("{key} documents"), collection.iter().count());
            out.insert(format!("{key} rejected"), collection.rejected().len());
        }};
        (@merged $side:ident $name:ident) => {{
            let key = concat!(stringify!($side), " ", stringify!($name));
            out.insert(format!("{key} layers"), set.$side.$name().layers().len());
        }};
        (@group $side:ident $name:ident) => {{
            let key = concat!(stringify!($side), " ", stringify!($name));
            out.insert(format!("{key} documents"), documents_in(set.$side.$name()));
        }};
        (@scripts $side:ident $name:ident) => {{
            let key = concat!(stringify!($side), " ", stringify!($name));
            let bundles = set.$side.$name();
            out.insert(format!("{key} bundles"), bundles.len());
            out.insert(
                format!("{key} plugins"),
                bundles.iter().filter(|bundle| bundle.plugin.is_some()).count(),
            );
        }};
        (@translations $side:ident $name:ident) => {{
            let key = concat!(stringify!($side), " ", stringify!($name));
            let languages: Vec<_> = set.$side.languages().value().codes.clone();
            let mut layers = 0;
            for language in &languages {
                if let Some(translations) = set.$side.$name(language) {
                    layers += translations.layers().len();
                }
            }
            out.insert(format!("{key} languages"), languages.len());
            out.insert(format!("{key} layers"), layers);
        }};
        ($($kind:ident $side:ident $name:ident;)*) => {
            $(count!(@$kind $side $name);)*
        };
    }
    count! {
        entries behavior blocks;
        entries behavior items;
        merged behavior crafting_catalog;
        entries behavior entities;
        entries behavior biomes;
        entries resource client_biomes;
        merged resource legacy_client_biomes;
        entries behavior spawn_rules;
        entries behavior spawn_groups;
        entries behavior features;
        group behavior jigsaw;
        group behavior recipes;
        entries behavior animations;
        entries resource animations;
        entries behavior animation_controllers;
        entries resource animation_controllers;
        group behavior feature_rules;
        entries behavior dimensions;
        group behavior poi;
        entries behavior dialogue;
        entries behavior trade_tables;
        entries behavior structures;
        entries behavior loot_tables;
        entries behavior functions;
        merged behavior tick_functions;
        merged behavior languages;
        merged resource languages;
        translations behavior translations;
        translations resource translations;
        scripts behavior scripts;
        scripts resource scripts;
        entries behavior behavior_trees;
        entries behavior voxel_shapes;
        entries resource geometries;
        group behavior cameras;
        group behavior aim_assist;
        merged resource sound_definitions;
        merged resource sounds_json;
        merged resource music;
        group resource sound_files;
        entries behavior server_sound_definitions;
        group resource deferred_settings;
        merged resource ui;
        entries resource client_blocks;
        entries resource block_culling;
        group resource data_driven_ui;
        group resource texture_atlases;
        merged resource flipbooks;
        group resource texture_lists;
        group resource texture_sets;
        group resource texture_files;
        merged resource splashes;
        merged resource loading_messages;
        merged resource credits;
        group resource fonts;
        entries resource ui_texture_infos;
        entries resource render_controllers;
        entries resource fogs;
        entries resource client_entities;
        entries resource atomic_client_entities;
        entries resource attachables;
        group resource materials;
        entries resource particles;
    }
    out
}

/// What `set` comes to: every collection loaded, the report, and the link pass with and
/// without its lints, linked as `tree`'s side (a server tree links as a dedicated server,
/// where the rules for what only a client reads, such as a client entity's texture, say
/// nothing). `started` is when the load began.
pub fn facts(tree: &Tree, set: &StackSet, started: Instant) -> Facts {
    let report = set.load_all();
    let load_wall = started.elapsed();
    let mut findings: Vec<Finding> = report.iter().map(comparable).collect();
    findings.sort();
    let errors = report
        .iter()
        .filter(|finding| finding.severity >= Severity::Error)
        .map(line)
        .collect();
    let error_counts = tally(
        report
            .iter()
            .filter(|finding| finding.severity >= Severity::Error),
        error_key,
    );
    let load = tally(report.iter(), load_key);
    let documents = documents(set);
    let link_started = Instant::now();
    let side = LinkOptions {
        client: tree.client,
        ..LinkOptions::default()
    };
    let linked: Diagnostics = set.link_with(&side);
    let link_wall = link_started.elapsed();
    let without_lints = set.link_with(&LinkOptions {
        lints: false,
        ..side
    });
    let lines = if std::env::var_os(DUMP).is_some() {
        (
            report.iter().map(line).collect(),
            linked.iter().map(line).collect(),
        )
    } else {
        (Vec::new(), Vec::new())
    };
    let above_info: Vec<&Diagnostic> = linked
        .iter()
        .filter(|finding| finding.severity > Severity::Info)
        .collect();
    Facts {
        packs: (set.behavior.packs().count(), set.resource.packs().count()),
        documents,
        load,
        findings,
        errors,
        error_counts,
        lines,
        link: tally(linked.iter(), link_key),
        link_debug: format!("{linked:?}"),
        link_errors: refusing_lines(linked.iter()),
        link_error_level: error_level_lines(linked.iter()),
        link_without_lints: format!("{:?}", without_lints.iter().collect::<Vec<_>>()),
        link_above_info: format!("{above_info:?}"),
        unknown_filter_tests: report
            .iter()
            .filter(|finding| finding.to_string().contains("Filter test not found"))
            .count(),
        wall: (load_wall, link_wall),
    }
}

/// The environment variable naming a folder every finding of every load is written into,
/// one line each, as `<tree>.<run>.load.txt` and `<tree>.<run>.link.txt`: for reading what a
/// count is made of. Nothing is written without it.
pub const DUMP: &str = "BEDROCK_ADDON_CORPUS_DUMP";

/// Writes the lines of `facts` into the folder [`DUMP`] names, if it names one.
pub fn dump(name: &str, facts: &Facts) {
    let Some(folder) = std::env::var_os(DUMP) else {
        return;
    };
    let folder = PathBuf::from(folder);
    std::fs::create_dir_all(&folder).expect("the dump folder");
    for (what, lines) in [("load", &facts.lines.0), ("link", &facts.lines.1)] {
        let path = folder.join(format!("{name}.{what}.txt"));
        std::fs::write(&path, lines.join("\n") + "\n")
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    }
}

/// One world at a time: a loaded client install is some gigabytes.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// The loads of this binary so far: a tree loaded with a run is loaded once, whichever tests
/// ask for it.
static LOADED: Mutex<Option<Loads>> = Mutex::new(None);

/// The facts of each tree and run loaded so far.
type Loads = HashMap<(Tree, Run), Arc<Facts>>;

/// Runs `work` on a thread with `stack` bytes of stack and gives its result.
pub fn on_a_thread<T: Send + 'static>(
    stack: usize,
    work: impl FnOnce() -> T + Send + 'static,
) -> T {
    std::thread::Builder::new()
        .stack_size(stack)
        .spawn(work)
        .expect("a thread")
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}

/// The facts of `tree` at `root` loaded with `run`, on a thread of `RECOMMENDED_JOB_STACK`
/// (the sequential strategy parses on it). Loaded once per binary.
pub fn loaded(tree: Tree, root: &Path, run: Run) -> Arc<Facts> {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let held = |loads: &Mutex<Option<Loads>>| {
        loads
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_or_insert_with(HashMap::new)
            .get(&(tree, run))
            .cloned()
    };
    if let Some(facts) = held(&LOADED) {
        return facts;
    }
    let at = root.to_path_buf();
    let found = on_a_thread(RECOMMENDED_JOB_STACK, move || {
        let started = Instant::now();
        let set = stacks(&tree, &at, run);
        facts(&tree, &set, started)
    });
    eprintln!(
        "{} ({}, {}): {} + {} packs, {} findings of the load in {:.1?}, {} of link() in {:.1?}",
        tree.name,
        run.word(),
        if run.rayon { "Rayon" } else { "Sequential" },
        found.packs.0,
        found.packs.1,
        found.findings.len(),
        found.wall.0,
        found.link.values().sum::<usize>(),
        found.wall.1,
    );
    dump(
        &format!(
            "{}.{}{}",
            tree.name,
            run.word(),
            if run.rayon { ".rayon" } else { "" }
        ),
        &found,
    );
    let found = Arc::new(found);
    LOADED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_or_insert_with(HashMap::new)
        .insert((tree, run), Arc::clone(&found));
    found
}

// ---- the pin files --------------------------------------------------------------------------

/// The environment variable that makes a pin test write its file.
const PIN: &str = "BEDROCK_ADDON_CORPUS_PIN";

fn pin_path(name: &str) -> PathBuf {
    std::env::var_os(PIN_DIR)
        .map_or_else(
            || Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/full/pins"),
            PathBuf::from,
        )
        .join(format!("{name}.txt"))
}

/// A pin file: its header lines, and per key the count and the sentence.
#[derive(Debug, Default)]
struct Pins {
    header: Vec<String>,
    counts: BTreeMap<String, (usize, String)>,
}

/// The pins of the text of a pin file.
fn parse_pins(name: &str, text: &str) -> Pins {
    let mut pins = Pins::default();
    for line in text.lines() {
        if line.starts_with('#') {
            pins.header.push(line.to_owned());
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        let (entry, note) = line.split_once(" # ").unwrap_or((line, ""));
        let (count, key) = entry
            .trim()
            .split_once(char::is_whitespace)
            .unwrap_or_else(|| panic!("{name}: the line {line:?} is no `<count> <key>`"));
        let count = count
            .parse()
            .unwrap_or_else(|_| panic!("{name}: the line {line:?} starts with no count"));
        pins.counts
            .insert(key.trim().to_owned(), (count, note.trim().to_owned()));
    }
    pins
}

/// The pin file `name`; `None` when there is no such file.
fn read_pins(name: &str) -> Option<Pins> {
    let text = std::fs::read_to_string(pin_path(name)).ok()?;
    Some(parse_pins(name, &text))
}

/// What a comparison with the pin file `pins` of `name` fails with before any count is
/// looked at: a pin file that does not exist pins nothing, and a load that found nothing
/// would agree with it.
pub fn missing_pin_file(name: &str, exists: bool) -> Option<String> {
    (!exists).then(|| {
        format!(
            "{name}: there is no pin file {} (write it with {PIN}=write)",
            pin_path(name).display()
        )
    })
}

fn write_pins(name: &str, pins: &Pins, found: &Counts) {
    let mut text = String::new();
    for line in &pins.header {
        text.push_str(line);
        text.push('\n');
    }
    let width = found.keys().map(String::len).max().unwrap_or(0);
    for (key, count) in found {
        let note = pins.counts.get(key).map_or("", |(_, note)| note.as_str());
        if note.is_empty() {
            writeln!(text, "{count:>7}  {key}").unwrap();
        } else {
            writeln!(text, "{count:>7}  {key:<width$} # {note}").unwrap();
        }
    }
    let path = pin_path(name);
    std::fs::create_dir_all(path.parent().expect("a folder")).expect("the pins folder");
    std::fs::write(&path, text).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    eprintln!("{name}: written, {} keys", found.len());
}

/// How `found` differs from the pinned counts, one line a key: a count that is another, a
/// key that is not pinned, a pinned key that was not found.
pub fn differences(pins: &BTreeMap<String, (usize, String)>, found: &Counts) -> Vec<String> {
    let mut differences = Vec::new();
    for (key, count) in found {
        match pins.get(key) {
            Some((pinned, _)) if pinned == count => {}
            Some((pinned, _)) => {
                differences.push(format!("  {key}: pinned {pinned}, found {count}"));
            }
            None => differences.push(format!("  {key}: not pinned, found {count}")),
        }
    }
    for (key, (pinned, _)) in pins {
        if !found.contains_key(key) {
            differences.push(format!("  {key}: pinned {pinned}, found none"));
        }
    }
    differences
}

/// The pinned keys `needs_a_sentence` asks a sentence for that have none.
pub fn bare(
    pins: &BTreeMap<String, (usize, String)>,
    needs_a_sentence: fn(&str) -> bool,
) -> Vec<&str> {
    pins.iter()
        .filter(|(key, (_, note))| note.is_empty() && needs_a_sentence(key))
        .map(|(key, _)| key.as_str())
        .collect()
}

/// No key needs a sentence.
pub fn no_key(_: &str) -> bool {
    false
}

/// Every key needs a sentence.
pub fn every_key(_: &str) -> bool {
    true
}

/// Requires `found` to be the counts the pin file `name` holds, key by key and with no key
/// more or less, and every key `needs_a_sentence` names to have its sentence in the file.
#[track_caller]
pub fn assert_pinned(name: &str, found: &Counts, needs_a_sentence: fn(&str) -> bool) {
    let pins = read_pins(name);
    if std::env::var_os(PIN).is_some_and(|value| value == "write") {
        write_pins(name, &pins.unwrap_or_default(), found);
        return;
    }
    if let Some(missing) = missing_pin_file(name, pins.is_some()) {
        panic!("{missing}");
    }
    let pins = pins.unwrap_or_default();
    let differences = differences(&pins.counts, found);
    assert!(
        differences.is_empty(),
        "{name}: {} counts differ from {} (take them again with {PIN}=write):\n{}",
        differences.len(),
        pin_path(name).display(),
        differences.join("\n")
    );
    let bare = bare(&pins.counts, needs_a_sentence);
    assert!(
        bare.is_empty(),
        "{name}: counts without a sentence saying what raises them: {bare:#?}"
    );
}

// ---- the checks of a tree -------------------------------------------------------------------

/// The tree's folder, or `None` after printing which entry of the game's files is missing: the test that
/// asked returns, which is a skip and no pass. The `SKIPPED (not a pass):` line goes to standard
/// error, which the test harness shows for a passing test only with `--show-output` (or
/// `--nocapture`), as with the other smokes: a run without the game's files is told apart from a
/// pass by those lines, never by the result line.
pub fn root_or_skip(tree: &Tree) -> Option<PathBuf> {
    let root = tree.root();
    if root.is_none() {
        game_files::skipped(tree.entry);
    }
    root
}

/// The checks of one run of a tree: the error-level findings are the pinned ones, each by its
/// code and file with a sentence beside the number saying why the game reports it; every
/// collection's count and every finding's count are as pinned; no filter test is one the table
/// lacks; and the link pass has no rejecting error (a kept error is pinned, not refused: the
/// game logs it and goes on).
pub fn check_load(tree: Tree, run: Run) {
    let Some(root) = root_or_skip(&tree) else {
        return;
    };
    let facts = loaded(tree, &root, run);
    let what = format!("{} ({})", tree.name, run.word());
    let suffix = if run.experiments { "_experiments" } else { "" };
    assert_pinned(
        &format!("{}.errors{suffix}", tree.name),
        &facts.error_counts,
        every_key,
    );
    assert_pinned(
        &format!("{}.documents{suffix}", tree.name),
        &facts.documents,
        no_key,
    );
    assert_pinned(&format!("{}.load{suffix}", tree.name), &facts.load, no_key);
    assert_eq!(
        facts.unknown_filter_tests, 0,
        "{what}: \"Filter test not found\" warnings"
    );
    assert!(
        facts.link_errors.is_empty(),
        "{what}: errors of the link pass:\n{}",
        facts.link_errors.join("\n")
    );
}

/// The link pass: no finding of `link()` refuses (a kept error is pinned like a warning),
/// every count of it is pinned by code, kind and folder with a sentence beside it, and
/// `link_with` without the lints returns exactly the findings above the information level.
pub fn check_link(tree: Tree) {
    let Some(root) = root_or_skip(&tree) else {
        return;
    };
    let facts = loaded(tree, &root, Run::SERVER);
    assert!(
        facts.link_errors.is_empty(),
        "{}: errors of the link pass:\n{}",
        tree.name,
        facts.link_errors.join("\n")
    );
    assert_eq!(
        facts.link_without_lints, facts.link_above_info,
        "{}: link_with(lints: false) against the findings of link() above Info",
        tree.name
    );
    assert_pinned(&format!("{}.link", tree.name), &facts.link, every_key);
    // The game's own `blocks.json` names no block the stack and the lists lack (R27), and no
    // loot table names an item that is none (L1).
    for (key, count) in &facts.link {
        assert!(
            !(key.contains("[block] @ blocks.json") || key.contains("[item] @ loot_tables")),
            "{}: {count} × {key}",
            tree.name
        );
    }
    eprintln!(
        "{}: link() took {:.1?} for {} findings",
        tree.name,
        facts.wall.1,
        facts.link.values().sum::<usize>()
    );
}

/// The link pass with a host that answers for the kinds the crate has no list of: what
/// remains is pinned, and none of it is an error.
pub fn check_host(tree: Tree) {
    let Some(root) = root_or_skip(&tree) else {
        return;
    };
    let facts = loaded(
        tree,
        &root,
        Run {
            host: true,
            ..Run::SERVER
        },
    );
    assert!(
        facts.link_errors.is_empty(),
        "{}: errors of the link pass with a host:\n{}",
        tree.name,
        facts.link_errors.join("\n")
    );
    assert_pinned(&format!("{}.link_host", tree.name), &facts.link, no_key);
}

/// The strategies: the load and the link pass on rayon are the sequential ones.
pub fn check_rayon(tree: Tree) {
    #[cfg(not(feature = "rayon"))]
    {
        eprintln!(
            "skipped: {}: built without the `rayon` feature, so there is no second strategy",
            tree.name
        );
    }
    #[cfg(feature = "rayon")]
    {
        let Some(root) = root_or_skip(&tree) else {
            return;
        };
        let sequential = loaded(tree, &root, Run::SERVER);
        let parallel = loaded(
            tree,
            &root,
            Run {
                rayon: true,
                ..Run::SERVER
            },
        );
        assert_eq!(parallel.packs, sequential.packs, "{}", tree.name);
        assert_eq!(parallel.documents, sequential.documents, "{}", tree.name);
        assert_eq!(parallel.load, sequential.load, "{}", tree.name);
        assert!(
            parallel.findings == sequential.findings,
            "{}: the findings of the two loads differ",
            tree.name
        );
        assert!(
            parallel.link_debug == sequential.link_debug,
            "{}: link() differs between the strategies",
            tree.name
        );
        assert!(
            parallel.link_without_lints == sequential.link_without_lints,
            "{}: link_with(lints: false) differs between the strategies",
            tree.name
        );
    }
}

/// The load and the link pass of `tree` on a thread with `stack` bytes of stack, the
/// sequential strategy parsing on it: the counts are the pinned ones.
pub fn check_on_a_small_stack(tree: Tree, stack: usize) {
    let Some(root) = root_or_skip(&tree) else {
        return;
    };
    let facts = {
        let _one = ONE_AT_A_TIME
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        on_a_thread(stack, move || {
            let started = Instant::now();
            let set = stacks(&tree, &root, Run::SERVER);
            facts(&tree, &set, started)
        })
    };
    assert_pinned(
        &format!("{}.documents", tree.name),
        &facts.documents,
        no_key,
    );
    assert_pinned(&format!("{}.load", tree.name), &facts.load, no_key);
    assert_pinned(&format!("{}.link", tree.name), &facts.link, no_key);
}

/// The default world of `tree` ([`default_world`]): loaded and linked, its findings by
/// severity pinned (`<tree>.default_world`). With `clean`, the load and the link pass must
/// have NO error-level finding, a kept link error included: the game's own packs load with
/// zero errors, asserted and not pinned. Without it (an install whose documents are of another
/// shape than the crate follows) the errors are pinned by code and file with a sentence each
/// (`<tree>.default_world_errors`), and the link pass must still have no rejecting error.
pub fn check_default_world(tree: Tree, clean: bool) {
    let Some(root) = root_or_skip(&tree) else {
        return;
    };
    let facts = loaded(
        tree,
        &root,
        Run {
            default_world: true,
            ..Run::SERVER
        },
    );
    assert!(
        facts.link_errors.is_empty(),
        "{}: errors of the link pass over the default world:\n{}",
        tree.name,
        facts.link_errors.join("\n")
    );
    if clean {
        assert!(
            facts.errors.is_empty(),
            "{}: errors of the load of the default world:\n{}",
            tree.name,
            facts.errors.join("\n")
        );
        assert!(
            facts.link_error_level.is_empty(),
            "{}: error-level findings of the link pass over the default world, kept ones included:\n{}",
            tree.name,
            facts.link_error_level.join("\n")
        );
    } else {
        assert_pinned(
            &format!("{}.default_world_errors", tree.name),
            &facts.error_counts,
            every_key,
        );
    }
    let mut summary = Counts::new();
    summary.insert(String::from("packs on the behaviour stack"), facts.packs.0);
    summary.insert(String::from("packs on the resource stack"), facts.packs.1);
    for (pass, counts) in [("load", &facts.load), ("link", &facts.link)] {
        for (letter, severity) in [('E', "errors"), ('W', "warnings"), ('I', "infos")] {
            let count = counts
                .iter()
                .filter(|(key, _)| key.starts_with(letter))
                .map(|(_, count)| count)
                .sum();
            summary.insert(format!("{pass} {severity}"), count);
        }
    }
    assert_pinned(&format!("{}.default_world", tree.name), &summary, no_key);
}

/// The test of the default world for one tree: `clean` says whether it must load without
/// any error.
macro_rules! default_world_test {
    ($tree:ident, clean) => {
        #[test]
        fn the_default_world_loads_and_links_with_no_error_at_all() {
            crate::support::check_default_world(crate::support::$tree, true);
        }
    };
    ($tree:ident, pinned) => {
        #[test]
        fn the_default_world_loads_with_the_pinned_errors_and_links_with_none() {
            crate::support::check_default_world(crate::support::$tree, false);
        }
    };
}

/// The tests of the load of one tree, in a module of its name.
macro_rules! load_tests {
    ($module:ident, $tree:ident) => {
        mod $module {
            use crate::support::{Run, check_load, $tree};

            #[test]
            fn every_family_loads_and_only_the_known_errors_are_errors() {
                check_load($tree, Run::SERVER);
            }

            #[test]
            fn every_family_loads_with_every_experiment_on() {
                check_load(
                    $tree,
                    Run {
                        experiments: true,
                        ..Run::SERVER
                    },
                );
            }
        }
    };
}

/// The tests of the link pass and of the strategies for one tree, in a module of its name.
macro_rules! link_tests {
    ($module:ident, $tree:ident) => {
        mod $module {
            use crate::support::{check_host, check_link, check_rayon, $tree};

            #[test]
            fn the_link_pass_finds_no_error_and_what_it_finds_is_pinned() {
                check_link($tree);
            }

            #[test]
            fn a_host_that_answers_for_the_game_leaves_the_pinned_findings() {
                check_host($tree);
            }

            #[test]
            fn rayon_loads_and_links_what_the_sequential_strategy_does() {
                check_rayon($tree);
            }
        }
    };
}

/// The tests of the writer for one tree, in a module of its name.
macro_rules! write_tests {
    ($module:ident, $tree:ident) => {
        mod $module {
            use crate::support::$tree;
            use crate::write::{check_copy, check_rewrite};

            #[test]
            fn every_pack_is_copied_exactly_and_the_copies_load_as_the_same_world() {
                check_copy($tree);
            }

            #[test]
            fn every_typed_document_is_written_back_and_reads_as_its_value() {
                check_rewrite($tree);
            }
        }
    };
}
