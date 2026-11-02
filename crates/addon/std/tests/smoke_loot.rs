//! Vanilla smoke for loot tables: the loot tables of the sample packs'
//! `behavior_pack/`, of every behaviour pack of the server install and of every behaviour pack
//! of the two client installs load through the `Driver`, one pack at a time, with zero
//! error-level diagnostics; MCB documents (baked documents the loaders do not decode) are
//! counted, not failed.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test smoke_loot`;
//! without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{Behavior, OpenOptions, PackOrigin, PackStack, StackSet};
use bedrock_addon_std::{Driver, Options};

use common::game_files;

/// The behaviour stack of the base-game pack `dir` alone, its files read by the driver that
/// opened it. Each pack is loaded on its own, so every copy of every table is parsed and no
/// pack is left out by the stack's dependency rules.
fn stack_of(dir: &Path) -> PackStack<Behavior> {
    let mut driver = Driver::new(Options::default().with_open(OpenOptions {
        origin: PackOrigin::Package,
        ..OpenOptions::default()
    }))
    .expect("a driver");
    let pack = driver
        .pack_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e:?}", dir.display()));
    let options = LoadOptions::default();
    let set = StackSet {
        behavior: PackStack::of([pack], options.clone()).expect("a stack of one pack"),
        resource: PackStack::of([], options).expect("an empty stack"),
    };
    driver
        .load_stacks(set, StackOptions::default())
        .expect("the stacks load")
        .behavior
}

/// What a load found.
#[derive(Default)]
struct Outcome {
    tables: usize,
    rejected: usize,
    mcb: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
    infos: Vec<String>,
}

/// The loot table MCB documents on disk below the pack `dir`: `loot_tables/` and the
/// `loot_tables` brarchive.
fn loot_mcb(dir: &Path) -> usize {
    [
        dir.join("loot_tables"),
        dir.join("__brarchive").join("loot_tables.brarchive"),
    ]
    .iter()
    .filter(|path| path.exists())
    .map(|path| game_files::mcb_on_disk(path).total())
    .sum()
}

/// The pack folders directly below `dir` (those with a manifest).
fn pack_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    dirs.retain(|path| path.join("manifest.json").is_file());
    dirs.sort();
    dirs
}

/// Loads each pack of `dirs` and collects the loot table findings. An `UnsupportedFileFormat`
/// finding is an MCB document as long as the game-files helper counted as many on disk
///, an error otherwise.
fn load_tables(dirs: &[PathBuf]) -> Outcome {
    let mut outcome = Outcome::default();
    for dir in dirs {
        let on_disk = loot_mcb(dir);
        let stack = stack_of(dir);
        let tables = stack.loot_tables();
        let mut mcb = 0;
        for finding in tables.diagnostics() {
            match finding.severity {
                Severity::Warning => outcome.warnings.push(finding.code.name().to_owned()),
                Severity::Info => outcome.infos.push(finding.code.name().to_owned()),
                _ => {}
            }
            if finding.severity < Severity::Error {
                continue;
            }
            if matches!(finding.code, DiagnosticCode::UnsupportedFileFormat) && mcb < on_disk {
                mcb += 1;
                continue;
            }
            outcome.errors.push(format!("{}: {finding}", dir.display()));
        }
        outcome.tables += tables.iter().count();
        outcome.rejected += tables.rejected().len();
        outcome.mcb += mcb;
    }
    outcome
}

/// Counts of each name in `names`, in first-seen order.
fn by_kind(names: &[String]) -> Vec<(&String, usize)> {
    let mut kinds: Vec<(&String, usize)> = Vec::new();
    for name in names {
        match kinds.iter_mut().find(|(kind, _)| *kind == name) {
            Some((_, count)) => *count += 1,
            None => kinds.push((name, 1)),
        }
    }
    kinds
}

/// Prints the source's line and its findings by code; requires no error outside MCB documents
/// and the warnings by code to be `warnings`, exactly.
fn report(source: &str, outcome: &Outcome, warnings: &[(&str, usize)]) {
    eprintln!(
        "{source}: {} loot tables, {} rejected, {} MCB documents, {} errors, {} warnings, {} infos",
        outcome.tables,
        outcome.rejected,
        outcome.mcb,
        outcome.errors.len(),
        outcome.warnings.len(),
        outcome.infos.len()
    );
    for (warning, count) in by_kind(&outcome.warnings) {
        eprintln!("  {count} × warning: {warning}");
    }
    for (info, count) in by_kind(&outcome.infos) {
        eprintln!("  {count} × info: {info}");
    }
    for error in &outcome.errors {
        eprintln!("  {error}");
    }
    // The game's own content-log error for its pools `{}` (it logs "There must be an `entries`
    // array inside the loot table pool." and keeps the table): vanilla's `armor_stand.json` and
    // `boat.json`, once each, and the tables are kept.
    let entries: Vec<&String> = outcome
        .errors
        .iter()
        .filter(|error| error.ends_with(ENTRIES_ERROR))
        .collect();
    assert_eq!(entries.len(), 2, "{source}: {entries:#?}");
    assert!(
        entries
            .iter()
            .any(|e| e.contains("loot_tables/entities/armor_stand.json"))
            && entries
                .iter()
                .any(|e| e.contains("loot_tables/entities/boat.json")),
        "{source}: {entries:#?}"
    );
    let others: Vec<&String> = outcome
        .errors
        .iter()
        .filter(|error| !error.ends_with(ENTRIES_ERROR))
        .collect();
    assert!(others.is_empty(), "{source}: {others:#?}");
    assert_eq!(outcome.rejected, 0, "{source}: no table is rejected");
    game_files::assert_warnings(
        source,
        &game_files::count_names(&outcome.warnings),
        warnings,
    );
}

/// The text of the game's content-log error for a pool without an `entries` array.
const ENTRIES_ERROR: &str = "There must be an \"entries\" array inside the loot table pool.";

/// The behaviour packs of an install at `root`.
fn install(root: &Path, source: &str, warnings: &[(&str, usize)]) {
    let outcome = load_tables(&pack_dirs(&root.join("behavior_packs")));
    report(source, &outcome, warnings);
    assert!(outcome.tables > 0, "{source} has loot tables");
}

#[test]
fn sample_packs_loot_tables_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let outcome = load_tables(&[samples.join("behavior_pack")]);
    report("sample packs", &outcome, &[]);
    assert!(outcome.tables > 0, "The sample packs have loot tables");
}

#[test]
fn server_install_loot_tables_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    install(&server, "server install", &[]);
}

#[test]
fn client_install_a_loot_tables_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    install(&client, "client install A", &[]);
}

#[test]
fn client_install_b_loot_tables_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    install(&client, "client install B", &[]);
}
