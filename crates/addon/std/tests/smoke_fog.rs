//! Fog smoke over the game's own files: every `fogs/` file of the resource packs of the sample
//! packs and the client installs (the server install ships no `fogs/`) loads through `Driver`
//! and `rp.fogs()` with no error-level diagnostic (MCB documents, baked documents the loaders
//! do not decode, are counted apart).
//! Fog settings hold no Molang, so the Molang pairs file has no fog occurrence to check.
//!
//! Each tree is loaded as one stack: the install's packs registered as the app's own (the only
//! kind that may use the `minecraft` namespace), the world's lists built by
//! `common::world::base_game_world`, so every pack of the tree is on the resource stack and
//! every file is read; a definition a higher pack shadows is counted by its
//! `ShadowedDefinition` info. The counts are pinned for the game's files.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo test -p bedrock_addon_std --test smoke_fog`;
//! without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, PackRepository};
use bedrock_addon_std::{DirSource, Driver, Options};

use common::game_files;
use common::world::base_game_world;

const DIRECTORY: &str = "fogs";

/// A driver that opens packs as the app's own.
fn base_game_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

/// The `.json` files directly in `dir`.
fn json_files(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|e| e == "json"))
        .count()
}

/// What one tree reported.
#[derive(Debug, Default)]
struct Tally {
    /// Packs with the directory.
    packs: usize,
    /// `.json` files in those directories.
    files: usize,
    /// Definitions the registry holds (one per identifier).
    fogs: usize,
    /// Definitions a higher pack, or an earlier file of their pack, shadows.
    shadowed: usize,
    errors: Vec<String>,
    mcb: usize,
    warnings: BTreeMap<&'static str, usize>,
    infos: BTreeMap<&'static str, usize>,
}

/// Loads the resource stack of `repository` (every pack of it, by the shared world list) and
/// requires no error; `roots` are the pack folders, for the counts on disk.
fn assert_loads(
    source: &str,
    driver: &Driver,
    repository: &PackRepository,
    roots: &[PathBuf],
) -> Tally {
    let mut tally = Tally::default();
    let mut on_disk = 0;
    for dir in roots {
        if !dir.join("manifest.json").is_file() || !dir.join(DIRECTORY).is_dir() {
            continue;
        }
        tally.packs += 1;
        tally.files += json_files(&dir.join(DIRECTORY));
        on_disk += game_files::mcb_on_disk(&dir.join(DIRECTORY)).total()
            + game_files::mcb_on_disk(
                &dir.join("__brarchive")
                    .join(format!("{DIRECTORY}.brarchive")),
            )
            .total();
    }
    let options = LoadOptions::default();
    let stacks = driver
        .load_stacks(
            repository.stacks(&base_game_world(repository, &options), options),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let registry = stacks.resource.fogs();
    tally.fogs = registry.len();
    let mut unsupported = Vec::new();
    for diagnostic in registry.diagnostics() {
        if diagnostic.code.name() == "UnsupportedFileFormat" {
            unsupported.push(diagnostic.to_string());
        } else if diagnostic.severity >= Severity::Error {
            tally.errors.push(diagnostic.to_string());
        } else if diagnostic.severity == Severity::Warning {
            *tally.warnings.entry(diagnostic.code.name()).or_insert(0) += 1;
        } else {
            *tally.infos.entry(diagnostic.code.name()).or_insert(0) += 1;
        }
    }
    tally.shadowed = tally.infos.get("ShadowedDefinition").copied().unwrap_or(0);
    tally.mcb = unsupported.len();
    if unsupported.len() > on_disk {
        tally.errors.extend(unsupported);
    }
    eprintln!(
        "{source}: {} packs, {} files, {} fogs ({} more shadowed), {} errors, {} MCB documents, warnings {:?}, infos {:?}",
        tally.packs,
        tally.files,
        tally.fogs,
        tally.shadowed,
        tally.errors.len(),
        tally.mcb,
        tally.warnings,
        tally.infos
    );
    assert!(tally.errors.is_empty(), "{source}: {:#?}", tally.errors);
    assert!(registry.rejected().is_empty(), "{source}: rejected files");
    assert!(tally.warnings.is_empty(), "{source}: {:?}", tally.warnings);
    tally
}

/// Every directory of a packs directory, sorted.
fn packs(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// The tree of an install: its behaviour and resource packs as one repository.
fn install(root: &Path, source: &str) -> Tally {
    let mut driver = base_game_driver();
    let repository = driver
        .repository([
            DirSource::packs(root.join("behavior_packs"), PackOrigin::Package),
            DirSource::packs(root.join("resource_packs"), PackOrigin::Package),
        ])
        .expect("the repository");
    assert_loads(
        source,
        &driver,
        &repository,
        &packs(&root.join("resource_packs")),
    )
}

#[test]
fn sample_packs_resource_pack_loads() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let mut driver = base_game_driver();
    let pack = driver
        .pack_dir(samples.join("resource_pack"))
        .expect("the resource pack opens");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package);
    let tally = assert_loads(
        "sample packs resource_pack",
        &driver,
        &repository,
        &[samples.join("resource_pack")],
    );
    // Every file of the vanilla pack defines one fog of its own.
    assert_eq!(
        (tally.packs, tally.files, tally.fogs, tally.shadowed),
        (1, 80, 80, 0)
    );
}

#[test]
fn client_install_b_resource_packs_load() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let tally = install(&client, "client install B resource_packs");
    assert_eq!(
        (tally.packs, tally.files, tally.fogs, tally.shadowed),
        CLIENT_INSTALL_B_COUNTS
    );
    // Every file was read: each is a definition or a shadowed one.
    assert_eq!(tally.fogs + tally.shadowed, tally.files);
}

#[test]
fn client_install_a_resource_packs_load() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let tally = install(&client, "client install A resource_packs");
    assert_eq!(
        (tally.packs, tally.files, tally.fogs, tally.shadowed),
        CLIENT_INSTALL_A_COUNTS
    );
    assert_eq!(tally.fogs + tally.shadowed, tally.files);
}

/// Packs with the directory, files, fogs by identifier, fogs shadowed.
const CLIENT_INSTALL_B_COUNTS: (usize, usize, usize, usize) = (12, 248, 80, 168);
const CLIENT_INSTALL_A_COUNTS: (usize, usize, usize, usize) = (10, 246, 78, 168);
