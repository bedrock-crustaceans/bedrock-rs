//! Behavior-tree smoke over the game's own files: the 22 trees of the vanilla behaviour pack
//! (`behavior_packs/vanilla/behavior_trees/` and its brarchive) of the server install and of the
//! two client installs load through `Driver` into `bp.behavior_trees()` with zero error-level
//! diagnostics, beside the nine built-in trees. The sample packs ship no behavior trees: their
//! stack holds the built-in ones only. MCB documents (baked documents the loaders do
//! not decode) are counted apart from errors.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_behavior_tree`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use bedrock_addon::diagnostic::codes as core_codes;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Severity;
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

/// The behavior-tree MCB documents on disk in the packs below `dirs` (`behavior_trees/` and
/// the `behavior_trees` brarchive).
fn tree_mcb(dirs: &[&Path]) -> usize {
    dirs.iter()
        .flat_map(|dir| pack_roots(dir))
        .flat_map(|root| {
            [
                root.join("behavior_trees"),
                root.join("__brarchive").join("behavior_trees.brarchive"),
            ]
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path).total())
        .sum()
}

/// Loads the base-game behaviour stack composed from the packs directories `dirs` and checks
/// its behavior trees: `pack_trees` trees from packs beside the nine built-in ones, warnings
/// by code, and no error but MCB documents.
fn check_trees(source: &str, dirs: &[&Path], world_behavior: &[u8], pack_trees: usize) {
    let on_disk = tree_mcb(dirs);
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
    let trees = stacks.behavior.behavior_trees();
    let mut mcb = 0;
    let mut errors = Vec::new();
    let mut warnings: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in trees.diagnostics() {
        if finding.severity == Severity::Warning {
            *warnings.entry(finding.code.id()).or_default() += 1;
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if finding.is(&core_codes::UNSUPPORTED_FILE_FORMAT) && mcb < on_disk {
            mcb += 1;
            continue;
        }
        errors.push(finding.to_string());
    }
    let from_packs = trees
        .iter()
        .filter(|(_, tree)| !tree.origin().pack.is_app_data())
        .count();
    eprintln!(
        "{source}: {} trees ({from_packs} from packs, {} built in) over {} packs, {} rejected files, {mcb} MCB documents ({on_disk} on disk), {} errors, warnings {warnings:?}",
        trees.len(),
        trees.len() - from_packs,
        stacks.behavior.packs().len(),
        trees.rejected().len(),
        errors.len(),
    );
    for error in errors.iter().take(50) {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    assert_eq!(
        trees.rejected().len(),
        mcb,
        "{source}: only MCB documents are rejected"
    );
    assert!(warnings.is_empty(), "{source}: {warnings:?}");
    assert_eq!(from_packs, pack_trees, "{source}");
    assert_eq!(trees.len(), pack_trees + 9, "{source}");
}

/// The sample packs have no `behavior_trees/` directory (the family is not documented): the
/// stack holds the nine built-in trees.
#[test]
fn sample_packs_hold_only_the_built_in_trees() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    assert!(
        !samples
            .join("behavior_pack")
            .join("behavior_trees")
            .exists()
    );
    let world = br#"[{"pack_id": "ee649bcf-256c-4013-9068-6a802b89d756", "version": [0, 0, 1]}]"#;
    check_trees("sample packs", &[&samples], world, 0);
}

#[test]
fn server_install_behavior_trees_load_without_errors() {
    let server = skip!(game_files::SERVER_INSTALL);
    check_trees(
        "server install",
        &[&server.join("behavior_packs")],
        b"[]",
        22,
    );
}

#[test]
fn client_install_b_behavior_trees_load_without_errors() {
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    check_trees(
        "client install B",
        &[&install_b.join("behavior_packs")],
        b"[]",
        22,
    );
}

/// Client install A holds the same 22 trees.
#[test]
fn client_install_a_behavior_trees_load_without_errors() {
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    check_trees(
        "client install A",
        &[&install_a.join("behavior_packs")],
        b"[]",
        22,
    );
}

/// Every loose behavior-tree file of every pack of the three installs parses on its own
/// without a finding, and writes back to a text that reads to the same tree.
#[test]
fn every_vanilla_behavior_tree_file_parses_and_round_trips() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::behavior_tree::BehaviorTree;
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::storage::PackPath;

    let cx = LoadContext::detached();
    let mut files = 0;
    for source in [
        game_files::SERVER_INSTALL,
        game_files::CLIENT_INSTALL_B,
        game_files::CLIENT_INSTALL_A,
    ] {
        let Some(root) = game_files::source(source) else {
            eprintln!("skipped: {}", game_files::missing(source));
            continue;
        };
        for pack in pack_roots(&root.join("behavior_packs")) {
            let Ok(entries) = std::fs::read_dir(pack.join("behavior_trees")) else {
                continue;
            };
            let mut paths: Vec<PathBuf> =
                entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
            paths.sort();
            for path in paths {
                let bytes = std::fs::read(&path).unwrap();
                if game_files::is_mcb(&bytes) {
                    continue;
                }
                files += 1;
                let name = format!(
                    "behavior_trees/{}",
                    path.file_name().unwrap().to_string_lossy()
                );
                let pack_path = PackPath::new(&name).unwrap();
                let tree = BehaviorTree::parse(&pack_path, &bytes, &cx)
                    .unwrap_or_else(|r| panic!("{}: {}", path.display(), r.diagnostics.summary()));
                assert!(tree.diagnostics().is_empty(), "{}", path.display());
                assert_eq!(tree.raw.as_bytes(), bytes, "{}", path.display());
                let text = tree.to_json().unwrap();
                let again = BehaviorTree::parse(&pack_path, text.as_bytes(), &cx).unwrap();
                assert_eq!(again.root, tree.root, "{}", path.display());
                assert_eq!(again.name, tree.name, "{}", path.display());
                assert_eq!(again.to_json().unwrap(), text, "{}", path.display());
            }
        }
    }
    eprintln!("{files} behavior-tree files parsed and written back");
    if game_files::root().is_some() {
        assert!(files >= 22, "{files} files");
    }
}
