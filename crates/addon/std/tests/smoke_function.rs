//! Function smoke over the game's own files: the behaviour packs of the sample packs, of the
//! server install and of client install B load through `Driver` into `bp.functions()` and
//! `bp.tick_functions()` with zero error-level diagnostics, and every `.mcfunction` file on
//! disk becomes a function. Vanilla ships no function of its own, so the two test asset
//! packs of client install A that carry one (loose, and baked into brarchives) are loaded
//! too. MCB documents (baked documents the loaders do not decode) are counted apart.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_function`; without the variable every test skips.

#[macro_use]
mod common;

use bedrock_addon::diagnostic::codes as core_codes;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::id::FunctionPath;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

use common::game_files;
use common::world::world_list;

/// The pack folders directly below `dir`.
fn pack_roots(dir: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    roots.retain(|path| path.is_dir());
    roots.sort();
    roots
}

/// The loose `.mcfunction` files below `dir`, recursively.
fn function_files(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .map(|path| {
            if path.is_dir() {
                function_files(&path)
            } else {
                usize::from(
                    path.file_name()
                        .is_some_and(|name| name.to_string_lossy().ends_with(".mcfunction")),
                )
            }
        })
        .sum()
}

/// What the packs below `dirs` hold on disk: loose function files, and MCB documents in
/// `functions/` and the `functions` brarchives.
fn on_disk(dirs: &[&Path]) -> (usize, usize) {
    let roots: Vec<PathBuf> = dirs.iter().flat_map(|dir| pack_roots(dir)).collect();
    let loose = roots
        .iter()
        .map(|root| function_files(&root.join("functions")))
        .sum();
    let mcb = roots
        .iter()
        .flat_map(|root| {
            [
                root.join("functions"),
                root.join("__brarchive").join("functions.brarchive"),
                root.join("__brarchive").join("functions"),
            ]
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path).total())
        .sum();
    (loose, mcb)
}

/// The stacks composed from the packs directories `dirs` for a world listing `world_behavior`.
fn load(dirs: &[&Path], origin: PackOrigin, world_behavior: &[u8]) -> StackSet {
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository(
            dirs.iter()
                .map(|dir| DirSource::packs(dir.to_path_buf(), origin)),
        )
        .unwrap();
    driver
        .load_stacks(
            repository.stacks(
                &world_list(&repository, world_behavior, &LoadOptions::default()),
                LoadOptions::default(),
            ),
            StackOptions::default(),
        )
        .unwrap()
}

/// Whether `finding` is about a file below `functions/`.
fn in_functions(finding: &Diagnostic) -> bool {
    finding
        .location
        .path
        .as_ref()
        .is_some_and(|path| path.as_str().starts_with("functions/"))
}

/// Checks the functions and the tick list of `stacks`: no error but MCB documents, nothing
/// rejected, and prints the counts and the warnings by code. Returns the number of functions.
fn check_functions(source: &str, stacks: &StackSet, dirs: &[&Path]) -> usize {
    let (loose, mcb_on_disk) = on_disk(dirs);
    let functions = stacks.behavior.functions();
    let tick = stacks.behavior.tick_functions();
    let report = stacks.behavior.report();
    let mut mcb = 0;
    let mut errors = Vec::new();
    let mut warnings: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in report.iter().filter(|finding| in_functions(finding)) {
        if finding.severity == Severity::Warning {
            *warnings.entry(finding.code.id()).or_default() += 1;
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if finding.is(&core_codes::UNSUPPORTED_FILE_FORMAT) {
            mcb += 1;
            continue;
        }
        errors.push(finding.to_string());
    }
    let lines: usize = functions
        .iter()
        .map(|(_, function)| function.commands.len())
        .sum();
    eprintln!(
        "{source}: {} functions ({lines} command lines, {loose} loose files on disk) and {} tick entries over {} packs, {} rejected files, {mcb} MCB findings ({mcb_on_disk} MCB documents on disk), {} errors, warnings {warnings:?}",
        functions.len(),
        tick.values.len(),
        stacks.behavior.packs().len(),
        functions.rejected().len(),
        errors.len(),
    );
    for error in errors.iter().take(50) {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    // No tree has a warning below `functions/`: the pin is exact, so a game file that gains
    // one fails the smoke.
    game_files::assert_warnings(source, &warnings, &[]);
    assert!(
        functions.rejected().is_empty(),
        "{source}: a function file was rejected"
    );
    // A finding per MCB document at most: the tick loader does not walk `functions/` itself
    // (it asks the stack's function registry), so each loader finding exists once.
    assert!(
        mcb <= mcb_on_disk,
        "{source}: {mcb} MCB findings for {mcb_on_disk} MCB documents on disk"
    );
    functions.len()
}

#[test]
fn sample_packs_functions_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    // The sample pack, selected by the world from the vanilla (`Package`) location.
    let world = br#"[{"pack_id": "ee649bcf-256c-4013-9068-6a802b89d756", "version": [0, 0, 1]}]"#;
    let dirs = [samples.as_path()];
    let stacks = load(&dirs, PackOrigin::Package, world);
    let count = check_functions("sample packs", &stacks, &dirs);
    assert_eq!(count, on_disk(&dirs).0, "every function file loads");
}

#[test]
fn server_install_functions_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    let packs = server.join("behavior_packs");
    let dirs = [packs.as_path()];
    let stacks = load(&dirs, PackOrigin::Package, b"[]");
    let count = check_functions("server install", &stacks, &dirs);
    assert_eq!(count, on_disk(&dirs).0, "every function file loads");
}

#[test]
fn client_install_b_functions_load_without_errors() {
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let packs = install_b.join("behavior_packs");
    let dirs = [packs.as_path()];
    let stacks = load(&dirs, PackOrigin::Package, b"[]");
    let count = check_functions("client install B", &stacks, &dirs);
    assert_eq!(count, on_disk(&dirs).0, "every function file loads");
}

/// The behaviour pack of both test asset folders of client install A.
const TEST_PACK: &[u8] =
    br#"[{"pack_id": "eab7dbf0-2980-4231-acf5-df7ad9d995f0", "version": [1, 0, 0]}]"#;

/// Loads the test asset folder `folder` of client install A and checks its one function,
/// `scriptevent/tick`, which its tick list runs.
fn check_test_assets(install_a: &Path, folder: &str) {
    let assets = install_a.join("test").join("assets").join(folder);
    if !assets.is_dir() {
        eprintln!("skipped: {} is not in the game files", assets.display());
        return;
    }
    let dirs = [assets.as_path()];
    let stacks = load(&dirs, PackOrigin::User, TEST_PACK);
    let count = check_functions(folder, &stacks, &dirs);
    assert_eq!(count, 1, "{folder}");
    let functions = stacks.behavior.functions();
    let function = functions
        .get(&FunctionPath::parse("scriptevent/tick"))
        .expect("scriptevent/tick");
    // min_engine_version 1.19.0.
    assert_eq!(function.command_version, Some(20));
    let lines: Vec<&str> = function
        .commands
        .iter()
        .map(bedrock_addon::families::function::FunctionLine::command)
        .collect();
    assert_eq!(lines, ["scriptevent demo:tickjson message from tick.json"]);
    assert!(function.diagnostics().is_empty());
    assert!(function.is_auto_completed());
    let tick = stacks.behavior.tick_functions();
    let names: Vec<&str> = tick.values.iter().map(|e| e.function.as_str()).collect();
    assert_eq!(names, ["scriptevent/tick"], "{folder}");
    assert_eq!(tick.values[0].command_version, 20);
}

/// The test pack with loose files: `functions/scriptevent/tick.mcfunction` and
/// `functions/tick.json`.
#[test]
fn the_client_test_pack_function_and_tick_list_load() {
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    check_test_assets(&install_a, "ArchivedPackLoadingUnbakedAssets");
}

/// The same pack baked: the function in `__brarchive/functions/scriptevent.brarchive`, the tick
/// list in `__brarchive/functions.brarchive`.
#[test]
fn the_client_archived_test_pack_function_and_tick_list_load() {
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    check_test_assets(&install_a, "ArchivedPackLoadingFunctionalTests");
}
