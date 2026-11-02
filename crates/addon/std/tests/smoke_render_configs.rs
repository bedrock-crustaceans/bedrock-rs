//! Vanilla smoke for the deferred renderer's settings: the `lighting/`,
//! `atmospherics/`, `water/`, `color_grading/`, `cubemaps/`, `shadows/`, `pbr/` and
//! `local_lighting/` files of the sample packs' `resource_pack/` and of every resource pack of
//! client installs B and A load through the `Driver` with zero error-level diagnostics; MCB
//! documents (baked documents the loaders do not decode) are counted, not failed.
//!
//! Each pack folder is loaded on its own, so every file is read: in a world stack the highest
//! pack's definition hides the lower ones. A folder no pack of a source has is reported in
//! the source's line with a count of 0 (`cubemaps/` is in no game tree; `shadows/`, `pbr/` and
//! `local_lighting/` are not in the sample packs).
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_render_configs`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, PackRepository};
use bedrock_addon_std::{Driver, Options};

use common::game_files;
use common::world::base_game_world;

/// The folders, in the order the game loads them.
const FOLDERS: [&str; 8] = [
    "lighting",
    "atmospherics",
    "water",
    "color_grading",
    "pbr",
    "shadows",
    "cubemaps",
    "local_lighting",
];

/// What the loads of one source found.
#[derive(Default)]
struct Outcome {
    /// Packs with at least one of the folders.
    packs: usize,
    /// The `.json` files on disk, per folder.
    files: [usize; 8],
    /// The documents loaded, per folder (for local lights: the files merged).
    loaded: [usize; 8],
    /// The block entries of the local-light files.
    local_lights: usize,
    /// The packs whose default file named a default identifier, per identifier folder.
    defaults: [usize; 4],
    rejected: Vec<String>,
    mcb: game_files::McbCount,
    mcb_seen: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
    infos: Vec<String>,
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

/// Loads the resource pack at `root` on its own, as a base-game pack.
fn load_pack(driver: &mut Driver, root: &Path, outcome: &mut Outcome) {
    if !FOLDERS.iter().any(|folder| root.join(folder).is_dir()) {
        return;
    }
    let name = root.file_name().unwrap().to_string_lossy().into_owned();
    let Ok(pack) = driver.pack_dir(root) else {
        outcome
            .errors
            .push(format!("{name}: the pack does not open"));
        return;
    };
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::Package);
    let options = LoadOptions::default();
    let stacks = driver
        .load_stacks(
            repository.stacks(&base_game_world(&repository, &options), options),
            StackOptions::default(),
        )
        .expect("the stacks load");
    let settings = stacks.resource.deferred_settings();
    outcome.packs += 1;
    let mut on_disk = 0;
    for (index, folder) in FOLDERS.iter().enumerate() {
        outcome.files[index] += json_files(&root.join(folder));
        let mcb = game_files::mcb_on_disk(&root.join(folder));
        on_disk += mcb.total();
        outcome.mcb = outcome.mcb + mcb;
    }
    let loaded = [
        settings.lighting.len(),
        settings.atmosphere.len(),
        settings.water.len(),
        settings.color_grading.len(),
        usize::from(settings.pbr_fallback_settings().is_some()),
        usize::from(settings.shadow_settings().is_some()),
        settings.cubemaps.len(),
        settings.local_lighting.layers().len(),
    ];
    for (total, count) in outcome.loaded.iter_mut().zip(loaded) {
        *total += count;
    }
    outcome.local_lights += settings.local_lighting.lights.len();
    let defaults = [
        settings.default_lighting().is_some(),
        settings.default_atmosphere().is_some(),
        settings.default_water().is_some(),
        settings.default_color_grading().is_some(),
    ];
    for (total, named) in outcome.defaults.iter_mut().zip(defaults) {
        *total += usize::from(named);
    }
    let rejected = settings
        .lighting
        .rejected()
        .iter()
        .chain(settings.atmosphere.rejected())
        .chain(settings.water.rejected())
        .chain(settings.color_grading.rejected())
        .chain(settings.cubemaps.rejected())
        .chain(settings.shadows.rejected())
        .chain(settings.pbr_fallback.rejected());
    outcome
        .rejected
        .extend(rejected.map(|rejected| format!("{name}/{}", rejected.origin.path)));
    for finding in settings.diagnostics() {
        let text = format!(
            "{name}/{} {} {}",
            finding
                .location
                .path
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            finding.location.json_path,
            finding.code,
        );
        match finding.severity {
            Severity::Warning => outcome.warnings.push(finding.code.name().to_owned()),
            Severity::Info => outcome.infos.push(finding.code.name().to_owned()),
            _ => {}
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if matches!(finding.code, DiagnosticCode::UnsupportedFileFormat)
            && outcome.mcb_seen < on_disk
        {
            outcome.mcb_seen += 1;
            continue;
        }
        outcome.errors.push(text);
    }
}

/// `count × name` of each distinct name, in first-seen order.
fn tally(names: &[String]) -> Vec<String> {
    let mut kinds: Vec<(&String, usize)> = Vec::new();
    for name in names {
        match kinds.iter_mut().find(|(kind, _)| *kind == name) {
            Some((_, count)) => *count += 1,
            None => kinds.push((name, 1)),
        }
    }
    kinds
        .into_iter()
        .map(|(name, count)| format!("{count} × {name}"))
        .collect()
}

/// Prints the source's line and requires no error outside MCB documents, and every file
/// accounted for.
fn report(source: &str, outcome: &Outcome) {
    let per_folder: Vec<String> = FOLDERS
        .iter()
        .enumerate()
        .map(|(index, folder)| {
            format!(
                "{folder} {}/{}",
                outcome.loaded[index], outcome.files[index]
            )
        })
        .collect();
    eprintln!(
        "{source}: {} packs with settings; loaded/files: {}; {} local lights; defaults \
         (lighting, atmosphere, water, color grading): {:?}; {} rejected; MCB {:?} ({} seen); \
         {} errors, {} warnings, {} infos",
        outcome.packs,
        per_folder.join(", "),
        outcome.local_lights,
        outcome.defaults,
        outcome.rejected.len(),
        outcome.mcb,
        outcome.mcb_seen,
        outcome.errors.len(),
        outcome.warnings.len(),
        outcome.infos.len()
    );
    for (index, folder) in FOLDERS.iter().enumerate() {
        if outcome.files[index] == 0 {
            eprintln!("  skipped: no pack of {source} has {folder}/");
        }
    }
    for line in tally(&outcome.warnings) {
        eprintln!("  warning: {line}");
    }
    for line in tally(&outcome.infos) {
        eprintln!("  info: {line}");
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
    // Every `.json` file is one loaded document: each pack's identifiers are its own, and
    // the single-file folders hold their one file.
    assert_eq!(
        outcome.loaded.iter().sum::<usize>() + outcome.mcb_seen,
        outcome.files.iter().sum::<usize>(),
        "{source}: every file is a loaded document"
    );
    assert!(outcome.packs > 0, "{source}: no pack has a settings folder");
}

/// Requires the source's exact counts: its packs with a settings folder, the documents loaded
/// per folder (in `FOLDERS` order), its local lights, and the warnings and infos by code. A
/// vanilla pack has no file below a settings folder that the game does not read: an
/// `IgnoredFile` info would show here.
fn counts(
    outcome: &Outcome,
    packs: usize,
    loaded: [usize; 8],
    local_lights: usize,
    warnings: &[&str],
    infos: &[&str],
) {
    assert_eq!(outcome.packs, packs);
    assert_eq!(outcome.loaded, loaded);
    assert_eq!(outcome.files, loaded);
    assert_eq!(outcome.local_lights, local_lights);
    assert_eq!(tally(&outcome.warnings), warnings);
    assert_eq!(tally(&outcome.infos), infos);
}

/// A driver whose packs are base-game packs.
fn base_game_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

/// Loads every pack folder below `dir`.
fn load_each(dir: &Path) -> Outcome {
    let mut driver = base_game_driver();
    let mut outcome = Outcome::default();
    for root in pack_roots(dir) {
        load_pack(&mut driver, &root, &mut outcome);
    }
    outcome
}

#[test]
fn sample_packs_settings_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let mut driver = base_game_driver();
    let mut outcome = Outcome::default();
    load_pack(&mut driver, &samples.join("resource_pack"), &mut outcome);
    report("sample packs", &outcome);
    // The four identifier folders the samples ship; the default file of each names its
    // default identifier.
    assert_eq!(outcome.defaults, [1, 1, 1, 1]);
    for folder in [0, 1, 2, 3] {
        assert!(outcome.files[folder] > 0, "{}", FOLDERS[folder]);
    }
    counts(&outcome, 1, [15, 17, 1, 14, 0, 0, 0, 0], 0, &[], &[]);
}

#[test]
fn client_install_b_settings_load_without_errors() {
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let outcome = load_each(&install_b.join("resource_packs"));
    report("client install B", &outcome);
    // `vanilla_1.21.90` ships the two single files; its 1.21.80 water file still has
    // `waves.sampleWidth`, which the 1.21.120 step drops.
    assert_eq!((outcome.loaded[4], outcome.loaded[5]), (1, 1));
    counts(
        &outcome,
        2,
        [15, 17, 2, 14, 1, 1, 0, 0],
        0,
        &[],
        &["1 × DroppedField"],
    );
}

#[test]
fn client_install_a_settings_load_without_errors() {
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    let outcome = load_each(&install_a.join("resource_packs"));
    report("client install A", &outcome);
    assert_eq!((outcome.loaded[4], outcome.loaded[5]), (1, 1));
    // As client install B, plus the local lights that install A has.
    counts(
        &outcome,
        3,
        [15, 17, 2, 14, 1, 1, 0, 1],
        96,
        &[],
        &["1 × DroppedField"],
    );
}
