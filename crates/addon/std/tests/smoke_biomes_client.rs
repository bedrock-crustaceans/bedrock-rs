//! Vanilla smoke for client biomes: the `biomes/` documents and the
//! `biomes_client.json` of the sample packs' `resource_pack/` and of every resource pack of the
//! two client installs load through the `Driver` with zero error-level diagnostics; MCB
//! documents (baked documents the loaders do not decode) are counted, not failed.
//!
//! Each pack folder is loaded on its own, so every file is read: in a world stack the highest
//! pack's document claims a biome and the lower ones are never applied. `biomes_client.json`
//! is read in a world of base-game version 1.21.30, because from 1.21.40 the game does not
//! read a base-game pack's.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_biomes_client`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use bedrock_addon::diagnostic::codes as core_codes;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Severity, codes};
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, PackRepository};
use bedrock_addon::version::{BaseGameVersion, SemVersion};
use bedrock_addon_std::{Driver, Options};

use common::game_files;
use common::world::base_game_world;

/// What the loads of one source found.
#[derive(Default)]
struct Outcome {
    /// Packs with a `biomes/` directory, and the `.json` files in them.
    biome_packs: usize,
    files: usize,
    client_biomes: usize,
    beta: usize,
    rejected: Vec<String>,
    /// Packs with a `biomes_client.json`, and the entries read from them.
    legacy_packs: usize,
    legacy_entries: usize,
    legacy_defaults: usize,
    /// The same files in a world from 1.21.40: skipped, with the info.
    legacy_skipped: usize,
    mcb: game_files::McbCount,
    mcb_seen: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
    /// How many of the warnings are the client biomes' (the rest are `biomes_client.json`'s).
    biome_warnings: usize,
    /// The warnings counted by code name.
    warnings_by_code: BTreeMap<&'static str, usize>,
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

/// The `.json` files directly below `dir`.
fn json_files(dir: &Path) -> usize {
    std::fs::read_dir(dir).map_or(0, |entries| {
        entries
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().is_some_and(|e| e == "json"))
            .count()
    })
}

/// A world of base-game version 1.21.30: base-game packs' `biomes_client.json` still load.
fn before_1_21_40() -> LoadOptions {
    LoadOptions {
        base_game_version: BaseGameVersion::new(SemVersion::new(1, 21, 30)),
        ..LoadOptions::default()
    }
}

/// Sorts a collection's findings into the outcome; an `UnsupportedFileFormat` finding is an
/// MCB document as long as as many were counted on disk.
fn collect<'a>(
    outcome: &mut Outcome,
    folder: &str,
    on_disk: usize,
    findings: impl Iterator<Item = &'a Diagnostic>,
) {
    for finding in findings {
        let text = format!(
            "{folder}/{} {} {}",
            finding
                .location
                .path
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            finding.location.json_path,
            finding.code,
        );
        if finding.severity == Severity::Warning {
            outcome
                .warnings
                .push(bedrock_addon::diagnostic::testing::legacy_debug(finding));
            *outcome
                .warnings_by_code
                .entry(finding.code.name())
                .or_insert(0) += 1;
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if finding.is(&core_codes::UNSUPPORTED_FILE_FORMAT) && outcome.mcb_seen < on_disk {
            outcome.mcb_seen += 1;
            continue;
        }
        outcome.errors.push(text);
    }
}

/// Loads the resource pack at `root` on its own, as a base-game pack.
fn load_pack(driver: &mut Driver, root: &Path, outcome: &mut Outcome) {
    let has_biomes = root.join("biomes").is_dir();
    let has_legacy = root.join("biomes_client.json").is_file();
    if !has_biomes && !has_legacy {
        return;
    }
    let folder = root.file_name().unwrap().to_string_lossy().into_owned();
    let Ok(pack) = driver.pack_dir(root) else {
        outcome
            .errors
            .push(format!("{folder}: the pack does not open"));
        return;
    };
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package);
    let load = |options: LoadOptions| {
        driver
            .load_stacks(
                repository.stacks(&base_game_world(&repository, &options), options),
                StackOptions::default(),
            )
            .expect("the stacks load")
    };
    let stacks = load(before_1_21_40());
    if has_biomes {
        let mcb = game_files::mcb_on_disk(&root.join("biomes"));
        outcome.mcb = outcome.mcb + mcb;
        outcome.biome_packs += 1;
        outcome.files += json_files(&root.join("biomes"));
        let biomes = stacks.resource.client_biomes();
        outcome.client_biomes += biomes.len();
        outcome.beta += biomes
            .beta()
            .map_or(0, bedrock_addon::document::Registry::len);
        outcome.rejected.extend(
            biomes
                .rejected()
                .iter()
                .map(|rejected| format!("{folder}/{}", rejected.origin.path)),
        );
        collect(outcome, &folder, mcb.total(), biomes.diagnostics());
        outcome.biome_warnings = outcome.warnings.len();
    }
    if has_legacy {
        let legacy = stacks.resource.legacy_client_biomes();
        outcome.legacy_packs += 1;
        outcome.legacy_entries += legacy.biomes.len();
        outcome.legacy_defaults += usize::from(legacy.default.is_some());
        assert_eq!(legacy.layers().len(), 1, "{folder}: one layer");
        collect(outcome, &folder, 0, legacy.diagnostics());
        // From 1.21.40 the same file takes no part.
        let later = load(LoadOptions::default());
        let skipped = later.resource.legacy_client_biomes();
        assert!(skipped.biomes.is_empty(), "{folder}: skipped from 1.21.40");
        outcome.legacy_skipped += skipped
            .diagnostics()
            .filter(|d| d.is(&codes::VANILLA_BIOMES_CLIENT_SKIPPED))
            .count();
    }
}

/// Loads every pack folder below `dir`.
fn load_each(driver: &mut Driver, dir: &Path) -> Outcome {
    let mut outcome = Outcome::default();
    for root in pack_roots(dir) {
        load_pack(driver, &root, &mut outcome);
    }
    outcome
}

/// Prints the source's line and requires no error outside MCB documents, every file
/// accounted for, and the warnings by code to be `warnings`, exactly.
fn report(source: &str, outcome: &Outcome, warnings: &[(&str, usize)]) {
    eprintln!(
        "{source}: {} packs with biomes/, {} files: {} client biomes, {} beta, {} rejected; \
         {} packs with biomes_client.json: {} entries, {} defaults, {} skipped from 1.21.40; \
         MCB {:?} ({} seen); {} errors, {} warnings",
        outcome.biome_packs,
        outcome.files,
        outcome.client_biomes,
        outcome.beta,
        outcome.rejected.len(),
        outcome.legacy_packs,
        outcome.legacy_entries,
        outcome.legacy_defaults,
        outcome.legacy_skipped,
        outcome.mcb,
        outcome.mcb_seen,
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
    assert!(outcome.errors.is_empty(), "{source}: {:#?}", outcome.errors);
    assert!(
        outcome.rejected.is_empty(),
        "{source}: {:#?}",
        outcome.rejected
    );
    // Every `.json` file is one document (each pack's identifiers are its own).
    assert_eq!(
        outcome.client_biomes + outcome.beta + outcome.mcb_seen,
        outcome.files,
        "{source}: every file is a loaded document"
    );
    assert_eq!(outcome.legacy_skipped, outcome.legacy_packs, "{source}");
    game_files::assert_warnings(source, &outcome.warnings_by_code, warnings);
}

/// A driver whose packs are base-game packs.
fn base_game_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

#[test]
fn sample_packs_client_biomes_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let mut driver = base_game_driver();
    let mut outcome = Outcome::default();
    load_pack(&mut driver, &samples.join("resource_pack"), &mut outcome);
    // The ignored fogs (71 here) are informations of the crate's own: the game ignores such an
    // entry without a word.
    report("sample packs", &outcome, &[]);
    // 89 files, 89 identifiers; no warning (fixture cb17).
    assert_eq!(outcome.client_biomes, 89);
    assert_eq!(outcome.biome_warnings, 0, "{:#?}", outcome.warnings);
    // The file alone: each entry's `fog_identifier` meets no lower entry and names no fog,
    // which is every warning there is.
    assert!(
        outcome.warnings.iter().all(|w| w == "LegacyFogIgnored"),
        "{:#?}",
        outcome.warnings
    );
    assert_eq!(outcome.legacy_packs, 1);
    assert_eq!(outcome.legacy_defaults, 1);
    assert!(outcome.legacy_entries >= 70, "{}", outcome.legacy_entries);
}

#[test]
fn client_install_b_client_biomes_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let mut driver = base_game_driver();
    let outcome = load_each(&mut driver, &client.join("resource_packs"));
    // The ignored fogs (279 here) are informations of the crate's own: the game ignores them without a word.
    report("client install B", &outcome, &[]);
    // 16 packs with `biomes/` (343 files) and 6 with `biomes_client.json`.
    assert_eq!(outcome.biome_packs, 16);
    assert_eq!(outcome.files, 343);
    assert_eq!(outcome.legacy_packs, 6);
}

#[test]
fn client_install_a_client_biomes_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let mut driver = base_game_driver();
    let outcome = load_each(&mut driver, &client.join("resource_packs"));
    // The ignored fogs (279 here) are informations of the crate's own: the game ignores them without a word.
    report("client install A", &outcome, &[]);
    assert!(outcome.biome_packs >= 15, "{}", outcome.biome_packs);
    assert_eq!(outcome.legacy_packs, 6);
}

/// The merged `biomes_client.json` of the stack vanilla builds from the install's resource
/// packs, in a world below 1.21.40: how many packs gave a layer, how many entries the merge
/// left, and the entries whose `fog_identifier` meets no lower entry (`LegacyFogIgnored`), each
/// with the version of the pack that wrote it, sorted.
///
/// The world's resource list is the packs that have the file in the client's base-game order:
/// the `vanilla_1.x` slices newest first, `vanilla` (pack version 0.0.1) at the bottom. The
/// world helper orders its resource stack that way, not by folder name.
fn stacked(driver: &mut Driver, packs: &[PathBuf]) -> (usize, usize, Vec<String>) {
    let mut repository = PackRepository::new();
    for root in packs {
        if !root.join("biomes_client.json").is_file() {
            continue;
        }
        if let Ok(pack) = driver.pack_dir(root) {
            repository.add_pack(pack, PackOrigin::Package);
        }
    }
    let world = base_game_world(&repository, &before_1_21_40());
    let stacks = driver
        .load_stacks(
            repository.stacks(&world, before_1_21_40()),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let legacy = stacks.resource.legacy_client_biomes();
    let errors: Vec<_> = legacy
        .diagnostics()
        .filter(|d| d.severity >= Severity::Error)
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
    let order: Vec<String> = legacy
        .layers()
        .iter()
        .map(|layer| layer.origin.pack.id().version.to_string())
        .collect();
    eprintln!("  layers, top first, by pack version: {order:?}");
    let mut ignored: Vec<String> = legacy
        .diagnostics()
        .filter(|d| d.is(&codes::LEGACY_FOG_IGNORED))
        .map(|d| {
            let path = d.location.json_path.to_string();
            let biome = path
                .trim_start_matches("biomes/")
                .trim_end_matches("/fog_identifier");
            format!(
                "{biome} ({})",
                d.location
                    .pack
                    .as_ref()
                    .map_or_else(|| "?".to_owned(), |pack| pack.id().version.to_string())
            )
        })
        .collect();
    ignored.sort();
    // The merged file has no warning (its notes are informations of the crate's own: the game
    // merges the copies without a word); the ignored fogs the tests pin are every one of them.
    game_files::assert_warnings(
        "stacked biomes_client.json",
        &game_files::warnings_by_code(legacy.diagnostics()),
        &[],
    );
    (legacy.layers().len(), legacy.biomes.len(), ignored)
}

/// The resource-pack folders of an install.
fn install(tree: &Path) -> Vec<PathBuf> {
    pack_roots(&tree.join("resource_packs"))
}

/// The stacked count beside the per-file one: read alone, each file's entries are all first
/// entries (279 `LegacyFogIgnored` over the six files of client install B); merged in the base-game order
/// only an entry no lower file has is one.
#[test]
fn client_install_b_stacked_biomes_client_json() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let mut driver = base_game_driver();
    let (layers, entries, ignored) = stacked(&mut driver, &install(&client));
    eprintln!(
        "client install B stacked: {layers} layers, {entries} entries, {} LegacyFogIgnored: {ignored:?}",
        ignored.len()
    );
    assert_eq!(layers, 6);
    assert_eq!(entries, 70);
    assert_the_two_new_biomes(&ignored);
}

#[test]
fn client_install_a_stacked_biomes_client_json() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let mut driver = base_game_driver();
    let (layers, entries, ignored) = stacked(&mut driver, &install(&client));
    eprintln!(
        "client install A stacked: {layers} layers, {entries} entries, {} LegacyFogIgnored: {ignored:?}",
        ignored.len()
    );
    assert_eq!(layers, 6);
    assert_eq!(entries, 70);
    assert_the_two_new_biomes(&ignored);
}

/// The sample packs have one resource pack: its stack is that pack's file alone.
#[test]
fn sample_packs_stacked_biomes_client_json() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let mut driver = base_game_driver();
    let packs = [samples.join("behavior_pack"), samples.join("resource_pack")];
    let (layers, entries, ignored) = stacked(&mut driver, &packs);
    eprintln!(
        "sample packs stacked: {layers} layers, {entries} entries, {} LegacyFogIgnored",
        ignored.len()
    );
    assert_eq!(layers, 1);
    assert_eq!(entries, 70);
    assert_eq!(ignored.len(), 71);
}

/// The `LegacyFogIgnored` findings of the base-game stacks of both client installs: two. Each slice's
/// entry meets the entry of a file below it, so its `fog_identifier` becomes
/// `fog_ids_to_merge`; only the two biomes a slice introduced have no lower entry:
/// `minecraft:mangrove_swamp` (first in the 1.19.0 slice) and `minecraft:cherry_grove` (first
/// in the 1.20.0 slice). Read alone, the six files give 279.
fn assert_the_two_new_biomes(ignored: &[String]) {
    assert_eq!(
        ignored,
        [
            "minecraft:cherry_grove (1.20.0)",
            "minecraft:mangrove_swamp (1.19.0)"
        ]
    );
}
