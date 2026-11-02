//! Vanilla smoke for server biomes: the biome documents of
//! The sample packs' `behavior_pack/` and of the server install's behaviour packs load through
//! the `Driver` with zero error-level diagnostics; MCB documents (baked documents the
//! loaders do not decode) are counted, not failed. The behaviour packs of both client installs
//! (with the experimental packs' `"beta"` files) are loaded too and their findings printed, and
//! the biome occurrences of the Molang pairs file load with their text and version.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo test -p bedrock_addon_std --test smoke_biome`;
//! without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, PackRepository};
use bedrock_addon_std::{Driver, Options};

use common::game_files;
use common::world::base_game_world;

/// What a load found.
struct Outcome {
    biomes: usize,
    beta: usize,
    rejected: usize,
    /// The rejected files, by path (prefixed with the pack folder by `load_each`).
    rejected_files: Vec<String>,
    mcb: game_files::McbCount,
    mcb_seen: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
    /// The warnings counted by code name.
    warnings_by_code: BTreeMap<&'static str, usize>,
}

/// The biome MCB documents on disk below the pack roots (`biomes/` and the `biomes`
/// brarchive), counted with the game-files helper.
fn biome_mcb(roots: &[PathBuf]) -> game_files::McbCount {
    roots
        .iter()
        .flat_map(|root| {
            [
                root.join("biomes"),
                root.join("__brarchive").join("biomes.brarchive"),
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

/// Loads the behaviour stack of `repository` and collects the biome findings; an
/// `UnsupportedFileFormat` finding is an MCB document as long as the helper counted as many
/// on disk below `roots`, an error otherwise.
fn load_biomes(driver: &Driver, repository: &PackRepository, roots: &[PathBuf]) -> Outcome {
    let mcb = biome_mcb(roots);
    let on_disk = mcb.total();
    let stacks = driver
        .load_stacks(
            repository.stacks(
                &base_game_world(repository, &LoadOptions::default()),
                LoadOptions::default(),
            ),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let biomes = stacks.behavior.biomes();
    let mut mcb_seen = 0;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for finding in biomes.diagnostics() {
        if finding.severity == Severity::Warning {
            warnings.push(format!("{} {}", finding.location.json_path, finding.code));
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if matches!(finding.code, DiagnosticCode::UnsupportedFileFormat) && mcb_seen < on_disk {
            mcb_seen += 1;
            continue;
        }
        errors.push(format!(
            "{} {} {} {:?}",
            finding
                .location
                .path
                .as_ref()
                .map(|p| p.to_string())
                .unwrap_or_default(),
            finding.location.json_path,
            finding.code,
            finding.code,
        ));
    }
    Outcome {
        biomes: biomes.len(),
        beta: biomes.beta().map_or(0, |beta| beta.len()),
        rejected: biomes.rejected().len(),
        rejected_files: biomes
            .rejected()
            .iter()
            .map(|r| r.origin.path.to_string())
            .collect(),
        mcb,
        mcb_seen,
        errors,
        warnings,
        warnings_by_code: game_files::warnings_by_code(biomes.diagnostics()),
    }
}

/// Loads every pack folder below `dir` on its own (a world stack applies only some of the
/// versioned vanilla slices, and the smoke covers every file), with the resource pack of the
/// same folder name when there is one, and sums what each load found.
fn load_each(driver: &mut Driver, dir: &Path, resources: Option<&Path>) -> Outcome {
    let mut total = Outcome {
        biomes: 0,
        beta: 0,
        rejected: 0,
        rejected_files: Vec::new(),
        mcb: game_files::McbCount::default(),
        mcb_seen: 0,
        errors: Vec::new(),
        warnings: Vec::new(),
        warnings_by_code: BTreeMap::new(),
    };
    for root in pack_roots(dir) {
        if !root.join("biomes").exists() && !root.join("__brarchive/biomes.brarchive").exists() {
            continue;
        }
        let Ok(pack) = driver.pack_dir(&root) else {
            continue;
        };
        let mut repository = PackRepository::new();
        repository.add_pack(pack, PackOrigin::Package);
        if let Some(resource) = resources
            .map(|r| r.join(root.file_name().unwrap()))
            .filter(|r| r.join("manifest.json").exists())
            .and_then(|r| driver.pack_dir(r).ok())
        {
            repository.add_pack(resource, PackOrigin::Package);
        }
        let outcome = load_biomes(driver, &repository, std::slice::from_ref(&root));
        total.biomes += outcome.biomes;
        total.beta += outcome.beta;
        total.rejected += outcome.rejected;
        let folder = root.file_name().unwrap().to_string_lossy().into_owned();
        total.rejected_files.extend(
            outcome
                .rejected_files
                .iter()
                .map(|path| format!("{folder}/{path}")),
        );
        total.mcb = total.mcb + outcome.mcb;
        total.mcb_seen += outcome.mcb_seen;
        total.errors.extend(outcome.errors);
        total.warnings.extend(outcome.warnings);
        for (code, count) in outcome.warnings_by_code {
            *total.warnings_by_code.entry(code).or_insert(0) += count;
        }
    }
    total
}

/// Prints the source's line and, when `strict`, requires no error outside MCB documents;
/// the warnings by code are `warnings`, exactly: a game file that gains a warning fails the
/// smoke.
fn report(source: &str, outcome: &Outcome, strict: bool, warnings: &[(&str, usize)]) {
    eprintln!(
        "{source}: {} biomes, {} beta biomes, {} rejected, MCB {:?} ({} seen), {} errors, {} warnings",
        outcome.biomes,
        outcome.beta,
        outcome.rejected,
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
fn sample_packs_biomes_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let mut driver = base_game_driver();
    let pack = driver
        .pack_dir(samples.join("behavior_pack"))
        .expect("the behaviour pack opens");
    let resource = driver
        .pack_dir(samples.join("resource_pack"))
        .expect("its resource pack opens");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package);
    repository.add_pack(resource, PackOrigin::Package);
    let outcome = load_biomes(&driver, &repository, &[samples.join("behavior_pack")]);
    report("sample packs", &outcome, true, &[]);
    // 89 files, 89 identifiers.
    assert_eq!(outcome.biomes, 89);
}

#[test]
fn server_install_behavior_packs_biomes_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    let mut driver = base_game_driver();
    let outcome = load_each(
        &mut driver,
        &server.join("behavior_packs"),
        Some(&server.join("resource_packs")),
    );
    report("server install", &outcome, true, &[]);
    assert!(outcome.biomes >= 128, "{} biomes", outcome.biomes);
}

#[test]
fn client_install_b_behavior_packs_biomes_load_without_errors() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let mut driver = base_game_driver();
    let outcome = load_each(
        &mut driver,
        &client.join("behavior_packs"),
        Some(&client.join("resource_packs")),
    );
    report("client install B", &outcome, true, &[]);
    // The 17 `"beta"` overrides of the experimental pack load from their base-game pack.
    assert_eq!(outcome.beta, 17);
}

#[test]
fn client_install_a_behavior_packs_biomes_load() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let mut driver = base_game_driver();
    let outcome = load_each(
        &mut driver,
        &client.join("behavior_packs"),
        Some(&client.join("resource_packs")),
    );
    // The 1.26.20 beta `sulfur_caves` of client install A uses the noise gradient vanilla
    // replaced in 1.26.30 (fixture b19): the one file expected to fail. Its four members the
    // 1.26.30 builder does not have are errors, not warnings: the game reads every biome slice,
    // the beta one included, strictly, so an unknown member refuses the biome.
    report("client install A", &outcome, false, &[]);
    assert_eq!(
        outcome.rejected_files,
        ["experimental_y_2026_drop_2/biomes/sulfur_caves.biome.json"],
        "{:#?}",
        outcome.errors
    );
}

/// Every biome occurrence of the Molang pairs file (the packs of client install B)
/// loads with the pair's text and raw version.
#[test]
fn molang_pairs_biome_height_ranges_load_with_their_text_and_version() {
    use bedrock_addon::families::biome::component::SurfaceMaterialAdjustments;
    use bedrock_addon::json::Value;
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let pairs_path = skip!(game_files::MOLANG_PAIRS);
    let pairs: Value =
        facet_json::from_str(&std::fs::read_to_string(pairs_path).unwrap()).expect("JSON");
    let mut driver = base_game_driver();
    let pack = driver
        .pack_dir(client.join("behavior_packs").join("vanilla"))
        .expect("the vanilla pack opens");
    let resource = driver
        .pack_dir(client.join("resource_packs").join("vanilla"))
        .expect("the vanilla resource pack opens");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package);
    repository.add_pack(resource, PackOrigin::Package);
    let stacks = driver
        .load_stacks(
            repository.stacks(
                &base_game_world(&repository, &LoadOptions::default()),
                LoadOptions::default(),
            ),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let biomes = stacks.behavior.biomes();
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
            if text(member(occurrence, "family")).as_deref() != Some("biomes") {
                continue;
            }
            let pack = text(member(occurrence, "pack")).unwrap_or_default();
            let file = text(member(occurrence, "file")).unwrap_or_default();
            let pointer = text(member(occurrence, "pointer")).unwrap_or_default();
            let version = member(occurrence, "version_effective")
                .and_then(|v| v.as_number().and_then(|n| n.to_i64()))
                .expect("a version");
            let rest = pointer
                .strip_prefix(
                    "/minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/",
                )
                .unwrap_or_else(|| panic!("a height_range pointer: {pointer}"));
            let (index, slot) = rest
                .split_once("/height_range/")
                .and_then(|(i, s)| Some((i.parse::<usize>().ok()?, s.parse::<usize>().ok()?)))
                .unwrap_or_else(|| panic!("a height_range pointer: {pointer}"));
            assert_eq!(pack, "vanilla", "the biome pairs are the vanilla pack's");
            let (_, document) = biomes
                .iter()
                .find(|(_, d)| d.origin().path.as_str() == file)
                .unwrap_or_else(|| panic!("{file} is loaded"));
            let adjustments = document
                .components
                .get::<SurfaceMaterialAdjustments>()
                .and_then(|c| c.adjustments.as_ref())
                .unwrap_or_else(|| panic!("{file}: adjustments"));
            let value = &adjustments[index]
                .height_range
                .as_ref()
                .expect("a height range")[slot];
            let source = value
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
    eprintln!("Molang pairs: {checked} biome occurrences checked");
    assert!(checked > 0);
}
