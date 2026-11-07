//! Vanilla smoke for the crafting item catalogue: the base-game stacks
//! of the server install and of the client installs load `bp.crafting_catalog()` through the
//! `Driver` with zero error-level findings; MCB documents (baked documents the loaders
//! do not decode) are counted, not failed. The sample packs ship no `item_catalog/` in their
//! `behavior_pack/`: they are loaded and must give four empty categories and no finding.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test smoke_catalog`;
//! without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use bedrock_addon::diagnostic::codes as core_codes;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Merged;
use bedrock_addon::families::crafting_catalog::{CategoryName, CraftingCatalog};
use bedrock_addon::load::{Collection, CollectionSet, Experiment, LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet};
use bedrock_addon::version::FormatVersion;
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

/// The MCB documents on disk of the catalogue directory in the packs below `dir` (the loose
/// directory and its brarchive).
fn mcb(dir: &Path) -> usize {
    pack_roots(dir)
        .into_iter()
        .flat_map(|root| {
            [
                root.join("item_catalog"),
                root.join("__brarchive").join("item_catalog.brarchive"),
            ]
        })
        .filter(|path| path.exists())
        .map(|path| game_files::mcb_on_disk(&path).total())
        .sum()
}

/// The stacks of the packs directory `dir` (all `Package` origin) for the world's behaviour
/// list `world_behavior`, with `experiments` on.
fn stacks(dir: &Path, world_behavior: &[u8], experiments: &[Experiment]) -> StackSet {
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository([DirSource::packs(dir.to_path_buf(), PackOrigin::Package)])
        .unwrap();
    let options = LoadOptions {
        experiments: experiments.iter().copied().collect(),
        ..LoadOptions::default()
    };
    driver
        .load_stacks(
            repository.stacks(&world_list(&repository, world_behavior, &options), options),
            // Only the catalogue: its walk reads the catalogue files and the item and block
            // definitions the merge resolves names against, nothing else of the stack.
            StackOptions {
                collections: CollectionSet::only([Collection::CraftingCatalog]),
                ..StackOptions::default()
            },
        )
        .unwrap()
}

/// What a merged catalogue holds: groups and items per category, in vanilla's order.
fn counts(catalog: &CraftingCatalog) -> (usize, usize) {
    let order: Vec<CategoryName> = catalog.categories.iter().map(|c| c.category_name).collect();
    assert_eq!(order, CategoryName::ALL);
    let groups = catalog.categories.iter().map(|c| c.groups.len()).sum();
    let items = catalog
        .categories
        .iter()
        .flat_map(|c| &c.groups)
        .map(|g| g.items.len())
        .sum();
    (groups, items)
}

/// Prints the catalogue's line (layers, groups, items, MCB, warnings by code) and requires no
/// error but MCB documents; returns the group and item counts.
fn check(source: &str, dir: &Path, merged: &Merged<CraftingCatalog>) -> (usize, usize) {
    let on_disk = mcb(dir);
    let mut counted = 0;
    let mut errors = Vec::new();
    let mut warnings: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in merged.diagnostics() {
        if finding.severity == Severity::Warning {
            *warnings.entry(finding.code.id()).or_default() += 1;
        }
        if finding.severity < Severity::Error {
            continue;
        }
        if finding.is(&core_codes::UNSUPPORTED_FILE_FORMAT) && counted < on_disk {
            counted += 1;
            continue;
        }
        errors.push(finding.to_string());
    }
    let (groups, items) = counts(merged.value());
    let layers: Vec<String> = merged
        .layers()
        .iter()
        .map(|layer| {
            format!(
                "{} {}",
                layer.origin().pack.id().version,
                layer
                    .version()
                    .written
                    .map_or_else(String::new, |v| v.to_string())
            )
        })
        .collect();
    eprintln!(
        "{source}: layers (pack version, format_version) {layers:?}, {groups} groups, {items} items, {counted} MCB documents ({on_disk} on disk), {} errors, warnings {warnings:?}",
        errors.len()
    );
    assert!(errors.is_empty(), "{source}: {errors:#?}");
    // Vanilla's own catalogues raise nothing: no unknown item, no duplicate, no unknown member.
    assert!(warnings.is_empty(), "{source}: {warnings:?}");
    (groups, items)
}

#[test]
fn sample_packs_has_no_catalogue() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let world = br#"[{"pack_id": "ee649bcf-256c-4013-9068-6a802b89d756", "version": [0, 0, 1]}]"#;
    let set = stacks(&samples, world, &[]);
    let merged = set.behavior.crafting_catalog();
    assert_eq!(check("sample packs", &samples, merged), (0, 0));
    assert!(merged.layers().is_empty());
    assert_eq!(merged.diagnostics().count(), 0);
}

/// Server install: the topmost vanilla slice with the file is `vanilla_1.26.30` (119 groups, 1700
/// items); the 49 slices below it are not read.
#[test]
fn server_install_catalogue_is_the_topmost_vanilla_slice_s() {
    let server = skip!(game_files::SERVER_INSTALL);
    let dir = server.join("behavior_packs");
    let set = stacks(&dir, b"[]", &[]);
    let merged = set.behavior.crafting_catalog();
    assert_eq!(check("server install", &dir, merged), (119, 1700));
    assert_eq!(merged.layers().len(), 1);
    assert_eq!(
        merged.layers()[0].version().written,
        Some(FormatVersion::new(1, 26, 30))
    );
    assert_eq!(
        merged
            .diagnostics()
            .filter(|f| f.severity > Severity::Info)
            .count(),
        0
    );
}

/// The client installs: install B's base is `vanilla_1.26.40`, install A's `vanilla_1.26.10`;
/// with the experiment on, its experimental pack joins 21 vanilla groups and adds one.
#[test]
fn client_catalogues_load_without_errors() {
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let dir = install_b.join("behavior_packs");
    let set = stacks(&dir, b"[]", &[]);
    assert_eq!(
        check("client install B", &dir, set.behavior.crafting_catalog()),
        (119, 1700)
    );
    let set = stacks(&dir, b"[]", &[Experiment::Y2026Drop3]);
    let merged = set.behavior.crafting_catalog();
    assert_eq!(
        check("client install B + y_2026_drop_3", &dir, merged),
        (120, 1773)
    );
    assert_eq!(merged.layers().len(), 2);
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    let dir = install_a.join("behavior_packs");
    let set = stacks(&dir, b"[]", &[]);
    assert_eq!(
        check("client install A", &dir, set.behavior.crafting_catalog()),
        (119, 1669)
    );
}
