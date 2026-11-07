//! Client entity smoke over the game's own files: every `entity/` file of the sample packs'
//! resource pack and of the resource packs of the client installs loads through `Driver` and
//! `client_entities()` with no error-level diagnostic with the Molang compile check on (MCB
//! documents, baked documents the loaders do not decode, are counted apart); the whole
//! install resolves to one definition per identifier by `description.min_engine_version`; and
//! every client entity occurrence of the Molang pairs file reads to its text and version.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_client_entity`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[path = "common/pairs.rs"]
mod pairs;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::{JsonDocument as _, Registry};
use bedrock_addon::families::client_entity::ClientEntity;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{OpenOptions, PackOrigin, PackStack, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options};

use common::game_files;
use common::world::base_game_world;

/// A driver that opens base-game packs.
fn driver() -> Driver {
    Driver::new(Options::default().with_open(OpenOptions {
        origin: PackOrigin::Package,
        ..OpenOptions::default()
    }))
    .unwrap()
}

/// The world options: the Molang compile check on.
fn options() -> LoadOptions {
    LoadOptions {
        molang: Some(std::sync::Arc::new(
            bedrock_addon::molang::MolangCheck::new(),
        )),
        ..LoadOptions::default()
    }
}

/// The MCB documents on disk of the pack `dir`'s client entities.
fn entity_mcb(dir: &Path) -> usize {
    [
        dir.join("entity"),
        dir.join("__brarchive").join("entity.brarchive"),
    ]
    .iter()
    .filter(|path| path.exists())
    .map(|path| game_files::mcb_on_disk(path).total())
    .sum()
}

/// What the packs of one source reported.
#[derive(Default)]
struct Tally {
    entities: usize,
    rejected: usize,
    errors: Vec<String>,
    mcb: usize,
    findings: BTreeMap<String, usize>,
}

impl Tally {
    /// Adds a registry's findings; `on_disk` is the MCB documents its packs hold.
    fn add(&mut self, at: &Path, registry: &Registry<ClientEntity>, on_disk: usize) {
        self.entities += registry.len();
        self.rejected += registry.rejected().len();
        let mut unsupported = Vec::new();
        for diagnostic in registry.diagnostics() {
            let diagnostic: &Diagnostic = diagnostic;
            if diagnostic.code.id() == "core/unsupported_file_format" {
                unsupported.push(diagnostic.to_string());
            } else if diagnostic.severity >= Severity::Error {
                self.errors.push(format!("{}: {diagnostic}", at.display()));
            } else {
                let key = format!("{:?} {}", diagnostic.severity, diagnostic.code.id());
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

    /// The findings at warning level, by code name.
    fn warnings(&self) -> BTreeMap<&str, usize> {
        self.findings
            .iter()
            .filter_map(|(key, count)| Some((key.strip_prefix("Warning ")?, *count)))
            .collect()
    }

    fn report(&self, source: &str) {
        eprintln!(
            "{source}: {} client entities, {} rejected, {} errors, {} MCB documents, findings {:?}",
            self.entities,
            self.rejected,
            self.errors.len(),
            self.mcb,
            self.findings
        );
    }
}

/// Loads every pack in `dirs` alone, as a resource stack of one base-game pack.
fn load_packs(source: &str, dirs: &[PathBuf]) -> Tally {
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
            behavior: PackStack::of([], options()).unwrap(),
            resource: PackStack::of([pack], options()).unwrap(),
        };
        let set = driver.load_stacks(set, StackOptions::default()).unwrap();
        tally.add(dir, set.resource.client_entities(), entity_mcb(dir));
    }
    tally.report(source);
    // No tree has a client entity warning: the pin is exact, so a game file that gains one
    // fails the smoke.
    game_files::assert_warnings(source, &tally.warnings(), &[]);
    tally
}

/// [`load_packs`], and requires no error.
fn assert_packs_load(source: &str, dirs: &[PathBuf]) -> Tally {
    let tally = load_packs(source, dirs);
    assert!(tally.errors.is_empty(), "{source}: {:#?}", tally.errors);
    assert_eq!(tally.rejected, 0, "{source}");
    tally
}

/// The pack directories below `packs` that hold an `entity/` folder.
fn entity_packs(packs: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(packs) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.join("entity").is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// Loads the whole install below `packs` as one resource stack, in the base game's order, and
/// requires no error.
fn assert_install_loads(source: &str, packs: &Path) -> Tally {
    let mut driver = driver();
    let repository = driver
        .repository([DirSource::packs(packs.to_path_buf(), PackOrigin::Package)])
        .expect("the repository");
    let world = base_game_world(&repository, &options());
    let set = driver
        .load_stacks(
            repository.stacks(&world, options()),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let on_disk = entity_packs(packs).iter().map(|dir| entity_mcb(dir)).sum();
    let mut tally = Tally::default();
    tally.add(packs, set.resource.client_entities(), on_disk);
    tally.report(source);
    assert!(tally.errors.is_empty(), "{source}: {:#?}", tally.errors);
    assert_eq!(tally.rejected, 0, "{source}");
    // Nor has an install as one stack.
    game_files::assert_warnings(source, &tally.warnings(), &[]);
    tally
}

#[test]
fn sample_packs_resource_pack_loads() {
    // The sample packs' pack is the game's layered packs merged member by member. Seven of its
    // files join an old slice's 1.8.0 `animation_controllers` list to a newer slice's 1.10.0 or
    // 1.26.0 body, a document no game pack holds: in the packs of client install B the same list is only in
    // 1.8.0 files (which load, above). Vanilla's schema for 1.10.0 and up does not declare the
    // member, and neither does the crate, so the game rejects those files too.
    const MERGED: [&str; 7] = [
        "creeper",
        "husk",
        "iron_golem",
        "phantom",
        "zombie",
        "zombie_pigman",
        "zombie_villager_v2",
    ];
    let samples = skip!(game_files::SAMPLE_PACKS);
    let mut tally = load_packs(
        "sample packs resource_pack",
        &[samples.join("resource_pack")],
    );
    tally.errors.retain(|error| {
        let merged = MERGED.iter().any(|name| {
            error.contains(&format!("| entity/{name}.entity.json |"))
                && error.ends_with("child 'animation_controllers' not valid here.")
        });
        !merged
    });
    assert!(tally.errors.is_empty(), "{:#?}", tally.errors);
    assert_eq!(tally.rejected, MERGED.len());
    assert!(tally.entities > 0);
}

#[test]
fn client_install_b_resource_packs_load() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let dirs = entity_packs(&client.join("resource_packs"));
    // 40 packs of client install B hold an `entity/` folder.
    assert_eq!(dirs.len(), 40, "{dirs:#?}");
    let tally = assert_packs_load("client install B resource_packs", &dirs);
    assert!(tally.entities > 0);
}

#[test]
fn client_install_a_resource_packs_load() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let dirs = entity_packs(&client.join("resource_packs"));
    if dirs.is_empty() {
        eprintln!("skipped: client install A has no resource pack with an entity/ folder");
        return;
    }
    assert_packs_load("client install A resource_packs", &dirs);
}

#[test]
fn the_client_install_b_resolves_one_definition_per_identifier() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let tally = assert_install_loads("client install B install", &client.join("resource_packs"));
    assert!(tally.entities > 0);
}

#[test]
fn the_client_install_a_resolves_one_definition_per_identifier() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    if !client.join("resource_packs").is_dir() {
        eprintln!("skipped: client install A has no resource_packs");
        return;
    }
    assert_install_loads("client install A install", &client.join("resource_packs"));
}

/// How many of the checked occurrences are `initialize` / `pre_animation` elements of files the
/// 1.26.0 struct reads.
const CONTEXT_VERSION_OCCURRENCES: usize = 436;

/// Whether `pointer` is an element of `scripts.initialize` or `scripts.pre_animation`.
fn is_plain_script_element(pointer: &str) -> bool {
    const SCRIPTS: &str = "/minecraft:client_entity/description/scripts/";
    pointer.strip_prefix(SCRIPTS).is_some_and(|rest| {
        rest.split_once('/').is_some_and(|(name, index)| {
            matches!(name, "initialize" | "pre_animation") && index.parse::<usize>().is_ok()
        })
    })
}

/// Every client entity occurrence of the Molang pairs file (the packs of client install B, family
/// `entity`) reads, in the document as written, to the pair's text at the pack's Molang
/// version. The family's keys (`scripts.variables` names) and the geometry files
/// vanilla keeps under `entity/` are counted, not read.
#[test]
fn every_client_entity_molang_occurrence_loads_with_its_text_and_version() {
    use bedrock_addon::diagnostic::Diagnostics;
    use bedrock_addon::families::client_entity::history::ClientEntityHistory;

    let pairs_file = skip!(game_files::MOLANG_PAIRS);
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    // The files read by the 1.26.0 struct.
    let mut plain: std::collections::BTreeSet<PathBuf> = std::collections::BTreeSet::new();
    let rules = pairs::Rules {
        // The names of `scripts.variables`: counted, not read.
        keys: pairs::Keys::Counted,
        // A geometry file kept under `entity/` is the geometry family's.
        foreign: |pointer| !pointer.starts_with("/minecraft:client_entity/"),
        ..pairs::Rules::as_written(pairs::Keys::Counted)
    };
    let found = pairs::check_with_keys(&pairs_file, &install_b, "entity", &rules, |file| {
        let written = ClientEntity::parse_as_written(&file.at, file.text.as_bytes(), &file.cx)
            .map_err(|rejected| rejected.diagnostics)?;
        let mut members = pairs::Members::new();
        if matches!(written.value(), ClientEntityHistory::V1_26_0(_)) {
            plain.insert(file.path.to_path_buf());
        }
        match written.into_value() {
            ClientEntityHistory::V1_8_0(mut entity) => {
                pairs::members(&mut entity, None, &mut members);
            }
            ClientEntityHistory::V1_10_0(mut entity) => {
                pairs::members(&mut entity, None, &mut members);
            }
            ClientEntityHistory::V1_26_0(mut entity) => {
                // From 1.26.0 the elements are plain strings (`visual::Scripts`): the loaded
                // document holds no Molang version for them, so none the loader assigned can
                // be read back. The text is the document's; the version is the load
                // context's, which is the one the loader compiles the joined text with. These
                // occurrences are counted apart below.
                let scripts = entity.description.slots.scripts.clone().unwrap_or_default();
                for (name, list) in [
                    ("initialize", &scripts.initialize),
                    ("pre_animation", &scripts.pre_animation),
                ] {
                    for (index, element) in list.iter().flatten().enumerate() {
                        members.insert(
                            format!("/minecraft:client_entity/description/scripts/{name}/{index}"),
                            (element.clone(), Some(file.cx.molang_version)),
                        );
                    }
                }
                pairs::members(&mut entity, None, &mut members);
            }
        }
        Ok((members, Diagnostics::default()))
    });
    let (checked, keys, geometry) = (found.compared, found.keys, found.foreign);
    let context_version = found
        .matched
        .iter()
        .filter(|(path, pointer)| is_plain_script_element(pointer) && plain.contains(path))
        .count();
    eprintln!(
        "Molang pairs: {checked} client entity occurrences checked in {} files, {keys} keys, {geometry} geometry occurrences",
        found.files
    );
    assert!(
        found.failures.is_empty(),
        "{} failures: {:#?}",
        found.failures.len(),
        &found.failures[..found.failures.len().min(20)]
    );
    // The counts of the Molang pairs file for the family `entity`: 2549 occurrences, of which 32
    // are keys and 10 are in geometry files.
    assert_eq!((checked, keys, geometry), (2507, 32, 10));
    // Of the 2507, the `initialize` / `pre_animation` elements of the 1.26.0 files: their
    // version is the context's, not one read from the document (the load context supplies it).
    eprintln!("Molang pairs: {context_version} of them elements of 1.26.0 script lists");
    assert_eq!(context_version, CONTEXT_VERSION_OCCURRENCES);
}
