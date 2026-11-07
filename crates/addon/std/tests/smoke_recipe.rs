//! Recipe smoke over the game's own files: the vanilla behaviour-pack recipes of the sample
//! packs and of the server install load through `Driver` into `bp.recipes()` with zero
//! error-level diagnostics. MCB documents (baked documents the loaders do not decode) in the
//! server install's brarchives are counted apart.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test smoke_recipe`;
//! without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use bedrock_addon::diagnostic::codes as core_codes;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::recipe::RecipeNamespace;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::PackOrigin;
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

/// The recipe MCB documents on disk in the packs below `dirs` (`recipes/` and the `recipes`
/// brarchive).
fn recipe_mcb(dirs: &[&Path]) -> usize {
    dirs.iter()
        .flat_map(|dir| pack_roots(dir))
        .flat_map(|root| {
            [
                root.join("recipes"),
                root.join("__brarchive").join("recipes.brarchive"),
            ]
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path).total())
        .sum()
}

/// Loads the base-game behaviour stack composed from the packs directories `dirs` and checks
/// its recipes: counts per namespace, warnings by code, and no error but MCB documents.
///
/// `flattened`: the source merges vanilla's versioned packs into one folder (sample packs),
/// so a recipe that two of them define is a same-pack duplicate there (11 identifiers are
/// defined by two files); such `DuplicateIdentifier` errors are counted apart, and every
/// rejected file must be one of them. In the real layout (a server install) they are
/// cross-pack overrides and none may occur.
fn check_recipes(source: &str, dirs: &[&Path], world_behavior: &[u8], flattened: bool) {
    let on_disk = recipe_mcb(dirs);
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository(
            dirs.iter()
                .map(|dir| DirSource::packs(dir.to_path_buf(), PackOrigin::Package)),
        )
        .unwrap();
    let stacks = driver
        .load_stacks(
            repository.stacks(
                &world_list(&repository, world_behavior, &LoadOptions::default()),
                LoadOptions::default(),
            ),
            StackOptions::default(),
        )
        .unwrap();
    let recipes = stacks.behavior.recipes();
    let mut mcb = 0;
    let mut merged = 0;
    let mut errors = Vec::new();
    let mut warnings: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in recipes.diagnostics() {
        if finding.severity == Severity::Warning {
            *warnings.entry(finding.code.name()).or_default() += 1;
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if finding.is(&core_codes::UNSUPPORTED_FILE_FORMAT) && mcb < on_disk {
            mcb += 1;
            continue;
        }
        // By legacy name: the definitions that were `DuplicateIdentifier` and the variant.
        if flattened && finding.code.name() == "DuplicateIdentifier" {
            merged += 1;
            continue;
        }
        errors.push(finding.to_string());
    }
    let per_namespace: Vec<String> = RecipeNamespace::ALL
        .iter()
        .map(|namespace| format!("{namespace:?} {}", recipes.registry(*namespace).len()))
        .collect();
    eprintln!(
        "{source}: {} recipes ({}) over {} packs, {} rejected files, {mcb} MCB documents ({on_disk} on disk), {merged} duplicates of the merged folder, {} errors, warnings {warnings:?}",
        recipes.len(),
        per_namespace.join(", "),
        stacks.behavior.packs().len(),
        recipes.rejected().count(),
        errors.len(),
    );
    for error in errors.iter().take(50) {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    // No tree has a recipe warning: the pin is exact.
    game_files::assert_warnings(source, &warnings, &[]);
    assert_eq!(
        recipes.rejected().count(),
        mcb + merged,
        "{source}: only MCB documents and merged duplicates are rejected"
    );
    assert!(!recipes.is_empty(), "{source}: no recipe loaded");
}

#[test]
fn sample_packs_recipes_load_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    // the sample pack, selected by the world from the vanilla (`Package`) location, loads as a
    // base-game pack
    let world = br#"[{"pack_id": "ee649bcf-256c-4013-9068-6a802b89d756", "version": [0, 0, 1]}]"#;
    check_recipes("sample packs", &[&samples], world, true);
}

#[test]
fn server_install_recipes_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    check_recipes(
        "server install",
        &[&server.join("behavior_packs")],
        b"[]",
        false,
    );
}

/// The behaviour packs of client install B, whose `vanilla_1.26.30` and chemistry recipes are
/// the newest vanilla ships (`format_version` 1.26.30, `//` comments).
#[test]
fn client_install_b_recipes_load_without_errors() {
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    check_recipes(
        "client install B",
        &[&install_b.join("behavior_packs")],
        b"[]",
        false,
    );
}

/// Every loose recipe file of every behaviour pack of client install B (3932 files, chemistry and
/// experimental packs included) parses on its own as a base-game file of a world of the engine
/// version, without an error.
#[test]
fn every_client_install_b_recipe_file_parses() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::recipe::Recipe;
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::storage::PackPath;

    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = true;
    let (mut files, mut errors) = (0, Vec::new());
    let mut warnings: BTreeMap<&str, usize> = BTreeMap::new();
    for root in pack_roots(&install_b.join("behavior_packs")) {
        let Ok(entries) = std::fs::read_dir(root.join("recipes")) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
        paths.sort();
        for path in paths {
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            files += 1;
            let bytes = std::fs::read(&path).unwrap();
            let name = format!("recipes/{}", path.file_name().unwrap().to_string_lossy());
            match Recipe::parse(&PackPath::new(&name).unwrap(), &bytes, &cx) {
                Ok(recipe) => {
                    for (code, count) in game_files::warnings_by_code(recipe.diagnostics().iter()) {
                        *warnings.entry(code).or_insert(0) += count;
                    }
                }
                Err(rejected) => errors.push(format!(
                    "{}: {}",
                    path.display(),
                    rejected.diagnostics.summary()
                )),
            }
        }
    }
    eprintln!(
        "client install B: {files} recipe files, {} rejected",
        errors.len()
    );
    for error in errors.iter().take(20) {
        eprintln!("{error}");
    }
    assert!(files > 3000, "{files} files");
    assert!(errors.is_empty());
    // The loose files' own warnings, by code: the pin is exact.
    game_files::assert_warnings("client install B loose recipe files", &warnings, &[]);
}
