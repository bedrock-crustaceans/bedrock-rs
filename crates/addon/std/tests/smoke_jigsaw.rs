//! Vanilla smoke for the jigsaw documents: the `worldgen/` documents of the sample packs'
//! `behavior_pack/` (it ships none today) and of the server install's behaviour packs load
//! through the `Driver` with zero error-level diagnostics; MCB documents (baked documents the
//! loaders do not decode) are counted, not failed. The behaviour packs of client install B (the
//! experimental pack's abandoned camps) are loaded too.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo test -p bedrock_addon_std --test smoke_jigsaw`;
//! without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{Pack, PackOrigin, PackRepository};
use bedrock_addon_std::{DirSource, Driver, Options};

use common::game_files;
use common::world::base_game_world;

/// What a load found.
struct Outcome {
    /// Structures, pools, sets, processor lists and metadata files loaded.
    counts: [usize; 5],
    rejected: usize,
    mcb: usize,
    errors: Vec<String>,
    /// Warnings and infos by path and code, for the report.
    findings: Vec<String>,
    /// The warnings counted by code name.
    warnings: BTreeMap<&'static str, usize>,
}

/// The MCB documents on disk below the pack roots `roots`: `worldgen/` and its brarchives.
fn worldgen_mcb(roots: &[PathBuf]) -> game_files::McbCount {
    roots
        .iter()
        .flat_map(|root| {
            [
                root.join("worldgen"),
                root.join("__brarchive").join("worldgen"),
                root.join("__brarchive").join("worldgen.brarchive"),
            ]
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path))
        .sum()
}

/// The pack folders directly below `dir`.
fn pack_roots(dir: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    roots.retain(|path| path.is_dir());
    roots.sort();
    roots
}

/// Loads the behaviour stack of `repository` with `driver` and collects the jigsaw findings; an
/// `UnsupportedFileFormat` finding is an MCB document as long as the helper counted as many on
/// disk below `roots`, an error otherwise, so an unsupported non-MCB file cannot pass silently.
fn load_jigsaw(driver: &Driver, repository: &PackRepository, roots: &[PathBuf]) -> Outcome {
    let on_disk = worldgen_mcb(roots).total();
    let stacks = driver
        .load_stacks(
            repository.stacks(
                &base_game_world(repository, &LoadOptions::default()),
                LoadOptions::default(),
            ),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let jigsaw = stacks.behavior.jigsaw();
    let mut mcb = 0;
    let mut errors = Vec::new();
    let mut findings = Vec::new();
    for finding in jigsaw.diagnostics() {
        let at = format!(
            "{} {}",
            finding
                .location
                .path
                .as_ref()
                .map(|p| p.to_string())
                .unwrap_or_default(),
            finding.location.json_path
        );
        if finding.severity < Severity::Error {
            findings.push(format!(
                "{} {} ({at})",
                finding.severity,
                finding.code.name()
            ));
            continue;
        }
        if matches!(finding.code, DiagnosticCode::UnsupportedFileFormat) && mcb < on_disk {
            mcb += 1;
            continue;
        }
        errors.push(format!("{at} {} {:?}", finding.code, finding.code));
    }
    Outcome {
        counts: [
            jigsaw.structures.len(),
            jigsaw.pools.len(),
            jigsaw.structure_sets.len(),
            jigsaw.processors.len(),
            jigsaw.metadata.iter().count(),
        ],
        rejected: jigsaw.structures.rejected().len()
            + jigsaw.pools.rejected().len()
            + jigsaw.structure_sets.rejected().len()
            + jigsaw.processors.rejected().len()
            + jigsaw.metadata.rejected().len(),
        mcb,
        errors,
        findings,
        warnings: game_files::warnings_by_code(jigsaw.diagnostics()),
    }
}

/// Prints the source's line and requires no error outside MCB documents.
fn report(source: &str, outcome: &Outcome, on_disk: usize) {
    let [structures, pools, sets, processors, metadata] = outcome.counts;
    eprintln!(
        "{source}: {structures} structures, {pools} pools, {sets} sets, {processors} processor \
         lists, {metadata} metadata files, {} rejected, {} MCB documents ({on_disk} on disk), \
         {} errors",
        outcome.rejected,
        outcome.mcb,
        outcome.errors.len(),
    );
    let mut kinds: Vec<(String, usize)> = Vec::new();
    for finding in &outcome.findings {
        let kind = finding.split(" (").next().unwrap_or(finding).to_owned();
        match kinds.iter_mut().find(|(k, _)| *k == kind) {
            Some((_, count)) => *count += 1,
            None => kinds.push((kind, 1)),
        }
    }
    for (kind, count) in kinds {
        eprintln!("  {count} × {kind}");
    }
    for error in &outcome.errors {
        eprintln!("  {error}");
    }
    assert!(outcome.errors.is_empty(), "{source}: {:#?}", outcome.errors);
    // No tree has a jigsaw warning: the pin is exact.
    game_files::assert_warnings(source, &outcome.warnings, &[]);
}

/// A driver whose packs are base-game packs.
fn base_game_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

#[test]
fn sample_packs_jigsaw_loads_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let mut driver = base_game_driver();
    let pack: Pack = driver
        .pack_dir(samples.join("behavior_pack"))
        .expect("the behaviour pack opens");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package);
    let roots = [samples.join("behavior_pack")];
    let outcome = load_jigsaw(&driver, &repository, &roots);
    report("sample packs", &outcome, worldgen_mcb(&roots).total());
}

#[test]
fn server_install_behavior_packs_jigsaw_loads_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    let mut driver = base_game_driver();
    let repository = driver
        .repository([DirSource::packs(
            server.join("behavior_packs"),
            PackOrigin::Package,
        )])
        .expect("the repository");
    let roots = pack_roots(&server.join("behavior_packs"));
    let outcome = load_jigsaw(&driver, &repository, &roots);
    report("server install", &outcome, worldgen_mcb(&roots).total());
    let [structures, pools, sets, processors, metadata] = outcome.counts;
    assert!(
        structures + pools + sets + processors + metadata + outcome.mcb > 0,
        "the server packs have jigsaw documents"
    );
}

#[test]
fn client_install_b_behavior_packs_jigsaw_loads_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let mut driver = base_game_driver();
    let repository = driver
        .repository([DirSource::packs(
            client.join("behavior_packs"),
            PackOrigin::Package,
        )])
        .expect("the repository");
    let roots = pack_roots(&client.join("behavior_packs"));
    let outcome = load_jigsaw(&driver, &repository, &roots);
    report("client install B", &outcome, worldgen_mcb(&roots).total());
}
