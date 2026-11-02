//! Vanilla smoke for blocks: the block documents of the sample packs' `behavior_pack/` and of
//! the server install's behaviour packs load through the `Driver` with zero error-level
//! diagnostics; MCB documents (baked documents the loaders do not decode) are counted,
//! not failed. The behaviour packs of client install B are loaded too and their findings
//! printed.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test smoke_block`;
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

/// What a load found: blocks loaded, rejected files, MCB documents, error-level findings.
struct Outcome {
    blocks: usize,
    rejected: usize,
    mcb: usize,
    errors: Vec<String>,
    /// Warnings by path and code, for the report.
    warnings: Vec<String>,
    /// The warnings counted by code name.
    warnings_by_code: BTreeMap<&'static str, usize>,
}

/// The block MCB documents on disk below the pack roots `roots` (`blocks/` and the `blocks`
/// brarchive), counted with the game-files helper.
fn block_mcb(roots: &[PathBuf]) -> game_files::McbCount {
    roots
        .iter()
        .flat_map(|root| {
            [
                root.join("blocks"),
                root.join("__brarchive").join("blocks.brarchive"),
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

/// Loads the behaviour stack of `repository` with `driver` and collects the block findings; an
/// `UnsupportedFileFormat` finding is an MCB document as long as the helper counted as many
/// on disk below `roots`, an error otherwise.
fn load_blocks(driver: &Driver, repository: &PackRepository, roots: &[PathBuf]) -> Outcome {
    let on_disk = block_mcb(roots).total();
    let stacks = driver
        .load_stacks(
            repository.stacks(
                &base_game_world(repository, &LoadOptions::default()),
                LoadOptions::default(),
            ),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let blocks = stacks.behavior.blocks();
    let mut mcb = 0;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for finding in blocks.diagnostics() {
        if finding.severity == Severity::Warning {
            warnings.push(format!("{} {}", finding.location.json_path, finding.code));
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if matches!(finding.code, DiagnosticCode::UnsupportedFileFormat) && mcb < on_disk {
            mcb += 1;
            continue;
        }
        errors.push(format!(
            "{} {} {} {}",
            finding
                .location
                .path
                .as_ref()
                .map(|p| p.to_string())
                .unwrap_or_default(),
            finding.location.json_path,
            finding.code,
            format_args!("{:?}", finding.code),
        ));
    }
    Outcome {
        blocks: blocks.len(),
        rejected: blocks.rejected().len(),
        mcb,
        errors,
        warnings,
        warnings_by_code: game_files::warnings_by_code(blocks.diagnostics()),
    }
}

/// Prints the source's line and requires no error outside MCB documents; the warnings by
/// code are `warnings`, exactly.
fn report(source: &str, outcome: &Outcome, strict: bool, warnings: &[(&str, usize)]) {
    eprintln!(
        "{source}: {} blocks, {} rejected, {} MCB documents, {} errors, {} warnings",
        outcome.blocks,
        outcome.rejected,
        outcome.mcb,
        outcome.errors.len(),
        outcome.warnings.len()
    );
    let mut kinds: Vec<(&String, usize)> = Vec::new();
    for warning in &outcome.warnings {
        match kinds.iter_mut().find(|(kind, _)| *kind == warning) {
            Some((_, count)) => *count += 1,
            None => kinds.push((warning, 1)),
        }
    }
    for (warning, count) in kinds {
        eprintln!("  {count} × warning: {warning}");
    }
    for error in &outcome.errors {
        eprintln!("  {error}");
    }
    if strict {
        assert!(outcome.errors.is_empty(), "{source}: {:#?}", outcome.errors);
    }
    game_files::assert_warnings(source, &outcome.warnings_by_code, warnings);
}

/// A driver whose packs are base-game packs.
fn base_game_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

#[test]
fn sample_packs_blocks_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let mut driver = base_game_driver();
    let pack: Pack = driver
        .pack_dir(samples.join("behavior_pack"))
        .expect("the behaviour pack opens");
    let resource: Pack = driver
        .pack_dir(samples.join("resource_pack"))
        .expect("its resource pack opens");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package);
    repository.add_pack(resource, PackOrigin::Package);
    let outcome = load_blocks(&driver, &repository, &[samples.join("behavior_pack")]);
    assert!(outcome.blocks > 0, "The sample packs have blocks");
    // No `particle_count` warning is raised: the game's gate is the format version alone, and
    // The sample packs are 1.21.90 or newer.
    report("sample packs", &outcome, true, &[]);
}

#[test]
fn server_install_behavior_packs_blocks_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    let mut driver = base_game_driver();
    let repository = driver
        .repository([DirSource::packs(
            server.join("behavior_packs"),
            PackOrigin::Package,
        )])
        .expect("the repository");
    let outcome = load_blocks(
        &driver,
        &repository,
        &pack_roots(&server.join("behavior_packs")),
    );
    report("server install", &outcome, true, &[]);
}

#[test]
fn client_install_b_behavior_packs_blocks_load() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let mut driver = base_game_driver();
    let repository = driver
        .repository([DirSource::packs(
            client.join("behavior_packs"),
            PackOrigin::Package,
        )])
        .expect("the repository");
    let outcome = load_blocks(
        &driver,
        &repository,
        &pack_roots(&client.join("behavior_packs")),
    );
    report(
        "client install B",
        &outcome,
        false,
        // No `particle_count` warning is raised (the gate is the format version alone); the 48
        // `FieldSinceEngine` findings are informations of the crate's own (the game is silent
        // about them), so no warning is left.
        &[],
    );
}

/// Every block occurrence of the Molang pairs file (the packs of client install B)
/// loads with the pair's text and raw version.
#[test]
fn molang_pairs_block_conditions_load_with_their_text_and_version() {
    use bedrock_addon::json::Value;
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let pairs_path = skip!(game_files::MOLANG_PAIRS);
    let pairs: Value =
        facet_json::from_str(&std::fs::read_to_string(pairs_path).unwrap()).expect("JSON");
    let mut driver = base_game_driver();
    let repository = driver
        .repository([DirSource::packs(
            client.join("behavior_packs"),
            PackOrigin::Package,
        )])
        .expect("the repository");
    let stacks = driver
        .load_stacks(
            repository.stacks(
                &base_game_world(&repository, &LoadOptions::default()),
                LoadOptions::default(),
            ),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let blocks = stacks.behavior.blocks();
    let member =
        |value: &Value, key: &str| -> Option<Value> { value.as_object()?.get(key).cloned() };
    let text =
        |value: Option<Value>| value.and_then(|v| v.as_string().map(|s| s.as_str().to_owned()));
    let mut checked = 0;
    for pair in member(&pairs, "pairs")
        .and_then(|p| p.as_array().cloned())
        .unwrap_or_default()
        .iter()
    {
        let expression = text(member(pair, "expr")).unwrap_or_default();
        for occurrence in member(pair, "occurrences")
            .and_then(|o| o.as_array().cloned())
            .unwrap_or_default()
            .iter()
        {
            if text(member(occurrence, "family")).as_deref() != Some("blocks") {
                continue;
            }
            let file = text(member(occurrence, "file")).unwrap_or_default();
            let pointer = text(member(occurrence, "pointer")).unwrap_or_default();
            let version = member(occurrence, "version_effective")
                .and_then(|v| v.as_number().and_then(|n| n.to_i64()))
                .expect("a version");
            let index: usize = pointer
                .strip_prefix("/minecraft:block/permutations/")
                .and_then(|rest| rest.strip_suffix("/condition"))
                .and_then(|i| i.parse().ok())
                .unwrap_or_else(|| panic!("a condition pointer: {pointer}"));
            let (_, block) = blocks
                .iter()
                .find(|(_, b)| b.origin().path.as_str() == file)
                .unwrap_or_else(|| panic!("{file} is loaded"));
            let source = block.permutations[index]
                .condition
                .source()
                .unwrap_or_else(|| panic!("{file} {pointer}: an expression"));
            assert_eq!(source.as_str(), expression, "{file} {pointer}");
            assert_eq!(
                source.raw_version().map(|v| i64::from(v.0)),
                Some(version),
                "{file} {pointer}"
            );
            checked += 1;
        }
    }
    eprintln!("Molang pairs: {checked} block occurrences checked");
    assert!(checked > 0);
}
