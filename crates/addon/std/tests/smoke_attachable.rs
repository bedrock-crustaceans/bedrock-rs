//! Attachable smoke over the game's own files: every `attachables/` file of the sample packs'
//! resource pack, of the resource packs of the two client installs and of the app's own
//! `definitions/attachables` loads through `Driver` and `attachables()` with no error-level
//! diagnostic with the Molang compile check on (MCB documents, baked documents the
//! loaders do not decode, are counted apart from errors); the
//! whole install, app data below the packs, resolves to one definition per identifier with
//! the item bindings of every layer; and every attachable occurrence of the Molang pairs file
//! reads to its text and version.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_attachable`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[path = "common/pairs.rs"]
mod pairs;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::{JsonDocument as _, Registry};
use bedrock_addon::families::attachable::{Attachable, Condition, ItemBindings};
use bedrock_addon::id::AttachableId;
use bedrock_addon::load::{AppData, LoadOptions, StackOptions};
use bedrock_addon::pack::{OpenOptions, PackOrigin, PackStack, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options, read_app_data};

use common::game_files;
use common::world::base_game_world;

/// The condition of every vanilla `*.player` attachable.
const PLAYER: &str = "query.owner_identifier == 'minecraft:player'";

/// A driver that opens base-game packs.
fn driver() -> Driver {
    Driver::new(Options::default().with_open(OpenOptions {
        origin: PackOrigin::Package,
        ..OpenOptions::default()
    }))
    .unwrap()
}

/// The world options: the Molang compile check on, over `app_data` when given.
fn options(app_data: Option<AppData>) -> LoadOptions {
    LoadOptions {
        molang: Some(std::sync::Arc::new(
            bedrock_addon::molang::MolangCheck::new(),
        )),
        app_data,
        ..LoadOptions::default()
    }
}

/// The MCB documents on disk of the pack `dir`'s attachables.
fn attachable_mcb(dir: &Path) -> usize {
    [
        dir.join("attachables"),
        dir.join("__brarchive").join("attachables.brarchive"),
    ]
    .iter()
    .filter(|path| path.exists())
    .map(|path| game_files::mcb_on_disk(path).total())
    .sum()
}

/// What the packs of one source reported.
#[derive(Default)]
struct Tally {
    attachables: usize,
    bindings: usize,
    rejected: usize,
    errors: Vec<String>,
    mcb: usize,
    findings: BTreeMap<String, usize>,
}

impl Tally {
    /// Adds a registry's findings; `on_disk` is the MCB documents its packs hold.
    fn add(&mut self, at: &Path, registry: &Registry<Attachable>, on_disk: usize) {
        self.attachables += registry.len();
        self.bindings += registry
            .iter()
            .map(|(_, document)| document.bound_items().len())
            .sum::<usize>();
        self.rejected += registry.rejected().len();
        let mut unsupported = Vec::new();
        for diagnostic in registry.diagnostics() {
            let diagnostic: &Diagnostic = diagnostic;
            if diagnostic.code.name() == "UnsupportedFileFormat" {
                unsupported.push(diagnostic.to_string());
            } else if diagnostic.severity >= Severity::Error {
                self.errors.push(format!("{}: {diagnostic}", at.display()));
            } else {
                let key = format!("{:?} {}", diagnostic.severity, diagnostic.code.name());
                if diagnostic.severity == Severity::Warning
                    && self.findings.get(&key).is_none_or(|n| *n < 3)
                {
                    eprintln!("  {diagnostic}");
                }
                *self.findings.entry(key).or_insert(0) += 1;
            }
        }
        self.mcb += unsupported.len();
        if unsupported.len() > on_disk {
            self.errors.extend(unsupported);
        }
    }

    /// The findings at warning level.
    fn warnings(&self) -> usize {
        self.findings
            .iter()
            .filter(|(key, _)| key.starts_with("Warning"))
            .map(|(_, count)| count)
            .sum()
    }

    fn report(&self, source: &str) {
        eprintln!(
            "{source}: {} attachables, {} bindings, {} rejected, {} errors, {} MCB documents, findings {:?}",
            self.attachables,
            self.bindings,
            self.rejected,
            self.errors.len(),
            self.mcb,
            self.findings
        );
    }
}

/// Loads every pack in `dirs` alone, as a resource stack of one base-game pack, and requires
/// no error.
fn assert_packs_load(source: &str, dirs: &[PathBuf]) -> Tally {
    let mut tally = Tally::default();
    for dir in dirs {
        if !dir.join("manifest.json").is_file() {
            continue;
        }
        let mut driver = driver();
        let pack = driver
            .pack_dir(dir)
            .unwrap_or_else(|e| panic!("{}: {e:?}", dir.display()));
        let set = StackSet {
            behavior: PackStack::of([], options(None)).unwrap(),
            resource: PackStack::of([pack], options(None)).unwrap(),
        };
        let set = driver.load_stacks(set, StackOptions::default()).unwrap();
        tally.add(dir, set.resource.attachables(), attachable_mcb(dir));
    }
    tally.report(source);
    assert!(tally.errors.is_empty(), "{source}: {:#?}", tally.errors);
    assert_eq!(tally.rejected, 0, "{source}");
    tally
}

/// The pack directories below `packs` that hold a `folder` folder.
fn packs_with(packs: &Path, folder: &str) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(packs) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.join(folder).is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// Loads the app data below `root` (the folder that holds `definitions/`) alone, with no
/// pack, and requires no error.
fn assert_app_data_loads(source: &str, root: &Path) -> Tally {
    let app_data = read_app_data(root).expect("the app data");
    let driver = driver();
    let set = StackSet {
        behavior: PackStack::of([], options(None)).unwrap(),
        resource: PackStack::of([], options(Some(app_data))).unwrap(),
    };
    let set = driver.load_stacks(set, StackOptions::default()).unwrap();
    let mut tally = Tally::default();
    let on_disk = game_files::mcb_on_disk(&root.join("definitions").join("attachables")).total();
    tally.add(root, set.resource.attachables(), on_disk);
    tally.report(source);
    assert!(tally.errors.is_empty(), "{source}: {:#?}", tally.errors);
    assert_eq!(tally.rejected, 0, "{source}");
    tally
}

/// Loads the whole install below `root` as one resource stack, in the base game's order,
/// over the install's app data, and requires no error. The stack set, for what the test
/// asks of it.
fn assert_install_loads(source: &str, root: &Path) -> (Tally, StackSet) {
    let packs = root.join("resource_packs");
    let app_data = read_app_data(root).expect("the app data");
    let mut driver = driver();
    let repository = driver
        .repository([DirSource::packs(packs.clone(), PackOrigin::Package)])
        .expect("the repository");
    let options = options(Some(app_data));
    let world = base_game_world(&repository, &options);
    let set = driver
        .load_stacks(repository.stacks(&world, options), StackOptions::default())
        .expect("the stacks load");
    let on_disk = packs_with(&packs, "attachables")
        .iter()
        .map(|dir| attachable_mcb(dir))
        .sum();
    let mut tally = Tally::default();
    tally.add(&packs, set.resource.attachables(), on_disk);
    tally.report(source);
    assert!(tally.errors.is_empty(), "{source}: {:#?}", tally.errors);
    assert_eq!(tally.rejected, 0, "{source}");
    (tally, set)
}

#[test]
fn sample_packs_resource_pack_loads() {
    // The 55 files of the flattened pack; 25 bindings; 30 attachables drawn for the item of
    // their own identifier only; nothing at warning level.
    let samples = skip!(game_files::SAMPLE_PACKS);
    let tally = assert_packs_load(
        "sample packs resource_pack",
        &[samples.join("resource_pack")],
    );
    assert_eq!((tally.attachables, tally.bindings), (55, 25));
    assert_eq!(tally.warnings(), 0, "{:?}", tally.findings);
}

/// The packs of an install that hold an `attachables/` folder, with how many files.
const LAYERS: [(&str, usize); 9] = [
    ("vanilla_1.17.0", 1),
    ("vanilla_1.20.0", 1),
    ("vanilla_1.20.80", 1),
    ("vanilla_1.21.0", 1),
    ("vanilla_1.21.10", 1),
    ("vanilla_1.21.110", 8),
    ("vanilla_1.21.130", 7),
    ("vanilla_1.21.90", 16),
    ("vanilla_1.26.10", 59),
];

/// Each of the nine packs alone: every file is an attachable of its own.
fn assert_layers_load(source: &str, packs: &Path) {
    let dirs = packs_with(packs, "attachables");
    let names: Vec<String> = dirs
        .iter()
        .map(|dir| dir.file_name().unwrap().to_str().unwrap().to_owned())
        .collect();
    assert_eq!(names, LAYERS.map(|(name, _)| name), "{source}");
    let tally = assert_packs_load(source, &dirs);
    assert_eq!(
        tally.attachables,
        LAYERS.iter().map(|(_, files)| files).sum::<usize>()
    );
    assert_eq!(tally.warnings(), 0, "{:?}", tally.findings);
}

#[test]
fn client_install_b_resource_packs_load() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    assert_layers_load(
        "client install B resource_packs",
        &client.join("resource_packs"),
    );
}

#[test]
fn client_install_a_resource_packs_load() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    if !client.join("resource_packs").is_dir() {
        eprintln!("skipped: client install A has no resource_packs");
        return;
    }
    assert_layers_load(
        "client install A resource_packs",
        &client.join("resource_packs"),
    );
}

#[test]
fn no_client_install_b_entity_folder_holds_an_attachable() {
    // The walk reads `entity/` for `minecraft:attachable` payloads; the
    // game's own packs have none there, and the client entities are left alone.
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let dirs: Vec<PathBuf> = packs_with(&client.join("resource_packs"), "entity")
        .into_iter()
        .filter(|dir| !dir.join("attachables").is_dir())
        .collect();
    assert!(dirs.len() > 30, "{}", dirs.len());
    let tally = assert_packs_load("client install B packs with entity/ only", &dirs);
    assert_eq!(tally.attachables, 0);
    assert!(tally.findings.is_empty(), "{:?}", tally.findings);
}

#[test]
fn the_app_data_attachables_load() {
    // The base set: 55 files, the same identifiers as the sample packs' flattened pack.
    for (source, name) in [
        ("client install B definitions", game_files::CLIENT_INSTALL_B),
        ("client install A definitions", game_files::CLIENT_INSTALL_A),
        ("server install definitions", game_files::SERVER_INSTALL),
    ] {
        let Some(root) = game_files::source(name) else {
            eprintln!("skipped: {}", game_files::missing(name));
            continue;
        };
        if !root.join("definitions").join("attachables").is_dir() {
            eprintln!("skipped: {source} has no definitions/attachables");
            continue;
        }
        let tally = assert_app_data_loads(source, &root);
        assert_eq!((tally.attachables, tally.bindings), (55, 25), "{source}");
        assert_eq!(tally.warnings(), 0, "{source}: {:?}", tally.findings);
    }
}

/// The same check on an install: the app's base set below the packs, `vanilla_1.26.10`
/// declaring the armour again (merged: neither side sets `description.min_engine_version`),
/// so each re-declared `*.player` file binds its item a second time.
fn assert_install_layers(source: &str, root: &Path) {
    let (tally, set) = assert_install_loads(source, root);
    let registry = set.resource.attachables();
    assert_eq!(tally.warnings(), 0, "{:?}", tally.findings);
    // No vanilla attachable sets a `min_engine_version`: nothing is ignored or replaced.
    assert!(
        !tally.findings.contains_key("Info AttachableShadowed"),
        "{:?}",
        tally.findings
    );
    assert!(!tally.findings.contains_key("Info ItemBindingShadowed"));
    let index = ItemBindings::from_registry(registry);
    let boots = index.get("minecraft:chainmail_boots");
    let shown: Vec<String> = boots
        .iter()
        .map(|entry| {
            assert!(
                matches!(&entry.condition, Condition::Molang(source) if source.as_str() == PLAYER)
            );
            assert!(!entry.shadowed);
            format!("{} @{}", entry.attachable, entry.pack.name())
        })
        .collect();
    assert_eq!(boots.len(), 2, "{shown:?}");
    assert!(boots[0].pack.is_app_data(), "{shown:?}");
    assert!(
        shown[1].contains("1.26.10") || !boots[1].pack.is_app_data(),
        "{shown:?}"
    );
    assert!(boots[0].sequence < boots[1].sequence);
    for entry in boots {
        assert_eq!(
            entry.attachable.as_str(),
            "minecraft:chainmail_boots.player"
        );
    }
    // The merged definition is the pack's document with the app data's contributing.
    let player = registry
        .get(&AttachableId::parse("minecraft:chainmail_boots.player"))
        .unwrap();
    assert!(!player.origin().pack.is_app_data());
    assert_eq!(player.bound_items().len(), 2);
    // Every item with a binding also has the attachable of its own identifier.
    for item in index.items() {
        assert!(registry.get(&AttachableId::parse(item)).is_some(), "{item}");
    }
    eprintln!(
        "{source}: {} identifiers, {} items with a binding, {} bindings",
        registry.len(),
        index.len(),
        tally.bindings
    );
    // The base set of 55 and what the versioned packs add (spyglass, harnesses, copper
    // armour, spears, …); each of the 29 armour items is bound by the app data and again by
    // the pack that declares its `*.player` file again.
    assert_eq!((registry.len(), index.len(), tally.bindings), (89, 29, 58));
}

#[test]
fn the_client_install_b_layers_its_packs_over_the_app_data() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    assert_install_layers("client install B install", &client);
}

#[test]
fn the_client_install_a_layers_its_packs_over_the_app_data() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    if !client.join("resource_packs").is_dir() {
        eprintln!("skipped: client install A has no resource_packs");
        return;
    }
    assert_install_layers("client install A install", &client);
}

/// Every attachable occurrence of the Molang pairs file (the packs of client install B, family
/// `attachables`) reads, in the document as written, to the pair's text at the pack's Molang
/// version, `item` conditions among them.
#[test]
fn every_attachable_molang_occurrence_loads_with_its_text_and_version() {
    use bedrock_addon::families::attachable::history::Any;

    let pairs_file = skip!(game_files::MOLANG_PAIRS);
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let mut newest = 0;
    let checked = pairs::check(&pairs_file, &install_b, "attachables", |file| {
        let written = Attachable::parse_as_written(&file.at, file.text.as_bytes(), &file.cx)
            .map_err(|rejected| rejected.diagnostics)?;
        let diagnostics = written.diagnostics().clone();
        let mut members = pairs::Members::new();
        match written.into_value() {
            Any::V1_8_0(mut attachable) => pairs::members(&mut attachable, None, &mut members),
            Any::V1_10_0(mut attachable) => pairs::members(&mut attachable, None, &mut members),
            Any::V1_26_0(mut attachable) => {
                // From 1.26.0 `initialize` / `pre_animation` hold plain strings, which carry
                // no Molang version (the load context supplies it, not the document): were a
                // vanilla file to use them, its
                // occurrences would not be found below, and the count would say so.
                newest += 1;
                pairs::members(&mut attachable, None, &mut members);
            }
        }
        Ok((members, diagnostics))
    });
    eprintln!(
        "Molang pairs: {} attachable occurrences checked in {} files, {newest} of them 1.26.0 files",
        checked.compared, checked.files
    );
    assert!(
        checked.failures.is_empty(),
        "{} failures: {:#?}",
        checked.failures.len(),
        &checked.failures[..checked.failures.len().min(20)]
    );
    // The counts of the Molang pairs file for the family `attachables`.
    assert_eq!((checked.compared, checked.files), (152, 71));
    assert_eq!(newest, 0);
}
