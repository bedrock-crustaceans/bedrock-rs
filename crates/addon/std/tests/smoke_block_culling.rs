//! Block-culling smoke over the game's own files: every `block_culling/*.json` of the
//! resource packs of the two client installs loads through `Driver` into `rp.block_culling()`
//! with no error-level diagnostic. The sample packs ship no `block_culling/` and are skipped
//! with a message.
//!
//! Vanilla's files are `<name>.culling.json` at the top of the directory, named
//! `minecraft:culling.<name>`, in base-game packs (the only packs that may use the namespace).
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo test -p bedrock_addon_std --test
//! smoke_block_culling`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::Path;

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Content;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

use common::game_files;
use common::world::base_game_world;

/// The `.json` files directly in the `block_culling/` of each pack folder below `dir`.
fn on_disk(dir: &Path) -> usize {
    let Ok(packs) = std::fs::read_dir(dir) else {
        return 0;
    };
    packs
        .filter_map(Result::ok)
        .filter_map(|pack| std::fs::read_dir(pack.path().join("block_culling")).ok())
        .flat_map(|files| files.filter_map(Result::ok))
        .filter(|file| file.path().extension().is_some_and(|e| e == "json"))
        .count()
}

/// The stacks of every pack below `dir` (all `Package` origin), each resource pack listed.
fn stacks(dir: &Path) -> StackSet {
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository([DirSource::packs(dir.to_path_buf(), PackOrigin::Package)])
        .unwrap();
    let options = LoadOptions::default();
    driver
        .load_stacks(
            repository.stacks(&base_game_world(&repository, &options), options),
            StackOptions::default(),
        )
        .unwrap()
}

/// Loads the culling rules of every resource pack below `dir` and requires that every file
/// on disk is an effective document, without a finding; the documents, their rules and the
/// resource packs of the stack.
fn check(source: &str, dir: &Path) -> (usize, usize, usize) {
    let files = on_disk(dir);
    let set = stacks(dir);
    let culling = set.resource.block_culling();
    let mut by_code: BTreeMap<(String, &str), usize> = BTreeMap::new();
    let mut errors = Vec::new();
    for finding in culling.diagnostics() {
        *by_code
            .entry((finding.severity.to_string(), finding.code.name()))
            .or_default() += 1;
        if finding.severity >= Severity::Error {
            errors.push(finding.to_string());
        }
    }
    let upgraded = culling.iter().filter(|(_, d)| d.version().upgraded).count();
    let rules: usize = culling.iter().map(|(_, d)| d.rules.len()).sum();
    eprintln!(
        "{source}: {} rules documents ({upgraded} from 1.20.60, {rules} rules) of {files} files in {} resource packs, {} rejected, {} errors; findings {by_code:?}",
        culling.len(),
        set.resource.packs().len(),
        culling.rejected().len(),
        errors.len(),
    );
    for error in &errors {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    assert!(culling.rejected().is_empty(), "{source}");
    assert_eq!(culling.len(), files, "{source}: one identifier per file");
    for (id, document) in culling {
        assert_eq!(id.namespace(), Some("minecraft"), "{source}: {id:?}");
        assert!(
            document.origin().path.as_str().ends_with(".culling.json"),
            "{source}: {}",
            document.origin().path
        );
        // Every vanilla file writes text that reads back.
        document.value().to_json().expect("it writes");
    }
    assert_eq!(
        upgraded,
        culling.len(),
        "{source}: every file is at 1.20.60"
    );
    assert!(by_code.is_empty(), "{source}: {by_code:?}");
    (culling.len(), rules, set.resource.packs().len())
}

/// Client install B: four files, all at 1.20.60, in `experimental_y_2026_drop_3`.
#[test]
fn client_install_b_block_culling_loads_without_errors() {
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let dir = install_b.join("resource_packs");
    assert_eq!(on_disk(&dir), 4, "experimental_y_2026_drop_3/block_culling");
    assert_eq!(check("client install B", &dir), (4, 41, 64));
}

/// Client install A is a smoke tree like client install B; skipped with a message when absent.
/// It ships 32 files in `experimental_vanilla_block_geometry`.
#[test]
fn client_install_a_block_culling_loads_without_errors() {
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    let dir = install_a.join("resource_packs");
    assert_eq!(on_disk(&dir), 32, "experimental_vanilla_block_geometry");
    assert_eq!(check("client install A", &dir), (32, 153, 71));
}

/// The sample packs have no `block_culling/`; the test says so instead of passing silently.
#[test]
fn sample_packs_has_no_block_culling() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    assert_eq!(
        on_disk(&samples),
        0,
        "The sample packs ship no block_culling/"
    );
    eprintln!("sample packs: no block_culling/ directory; nothing to load");
}
