//! Texture list smoke over the game's own files: the resource packs of the client installs load
//! through `Driver` into `rp.texture_lists()`, every pack's list is read, and none has an
//! error-level diagnostic. The sample packs ship the list under a name no client reads.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_textures_list`; without the variable every test skips.

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::Path;

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::load::{Collection, CollectionSet, LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

use common::game_files;
use common::world::base_game_world;

/// The file the smoke is about.
const FILE: &str = "textures/textures_list.json";

/// The pack folders below `dir` that hold `FILE`.
fn packs_with_file(dir: &Path) -> usize {
    std::fs::read_dir(dir).map_or(0, |entries| {
        entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|pack| pack.join(FILE).is_file())
            .count()
    })
}

/// The stacks of every pack below `dir`, with the texture group loaded.
fn load(dir: &Path, origin: PackOrigin) -> StackSet {
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository([DirSource::packs(dir.to_path_buf(), origin)])
        .unwrap();
    let options = LoadOptions::default();
    let world = base_game_world(&repository, &options);
    driver
        .load_stacks(
            repository.stacks(&world, options),
            StackOptions {
                collections: CollectionSet::only([
                    Collection::TextureLists,
                    Collection::TextureFiles,
                ]),
                ..StackOptions::default()
            },
        )
        .unwrap()
}

/// Prints the counts of `source` and its warnings by code, and asserts that none of
/// `findings` is an error.
fn check<'a>(
    source: &str,
    what: &str,
    findings: impl Iterator<Item = &'a Diagnostic>,
    expected: &[(&str, usize)],
) {
    let mut errors = Vec::new();
    let mut warnings: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in findings {
        match finding.severity {
            Severity::Error => errors.push(finding.to_string()),
            Severity::Warning => *warnings.entry(finding.code.id()).or_default() += 1,
            _ => {}
        }
    }
    println!("{source}: {what}; warnings by code: {warnings:?}");
    assert!(
        errors.is_empty(),
        "{source}: {} error(s):\n{}",
        errors.len(),
        errors.join("\n")
    );
    // The tree's warnings by code, exactly: a game file that gains a warning fails the smoke.
    game_files::assert_warnings(source, &warnings, expected);
}

/// The packs of the resource stack that hold `FILE`.
fn stacked(stacks: &StackSet) -> usize {
    stacks.resource.texture_files().packs_with(FILE).count()
}

/// Loads the resource packs below `dir` and checks the texture lists of the stack.
fn smoke(source: &str, dir: &Path, origin: PackOrigin, warnings: &[(&str, usize)]) {
    let on_disk = packs_with_file(dir);
    assert!(
        on_disk > 0,
        "{source}: no pack below {} has {FILE}",
        dir.display()
    );
    let stacks = load(dir, origin);
    let in_stack = stacked(&stacks);
    let textures = stacks.resource.texture_lists();
    let entries: usize = textures.lists.iter().map(|list| list.entries.len()).sum();
    let claims = textures.claims().len();
    check(
        source,
        &format!(
            "{on_disk} pack(s) with {FILE} on disk, {in_stack} in the stack, {} read, {entries} entries, {claims} claims",
            textures.lists.len()
        ),
        textures
            .lists
            .iter()
            .flat_map(|list| list.diagnostics().iter()),
        warnings,
    );
    // Every list of the stack is read, top pack first, and claims something.
    assert!(in_stack > 0, "{source}: the stack has the file");
    assert_eq!(
        textures.lists.len(),
        in_stack,
        "{source}: every list of the stack is read"
    );
    let order: Vec<_> = textures
        .lists
        .iter()
        .map(|list| list.origin().pack.index())
        .collect();
    assert!(order.is_sorted(), "{source}: top pack first: {order:?}");
    assert!(
        entries > 0 && claims >= entries / 2,
        "{source}: {entries} entries, {claims} claims"
    );
}

#[test]
fn sample_packs_ships_no_texture_list() {
    let Some(samples) = game_files::source(game_files::SAMPLE_PACKS) else {
        game_files::skipped(game_files::SAMPLE_PACKS);
        return;
    };
    // The tree holds `textures/texture_list.json`, a name no client reads.
    assert_eq!(packs_with_file(&samples), 0);
    assert!(
        samples
            .join("resource_pack/textures/texture_list.json")
            .is_file()
    );
    eprintln!("The sample packs ship textures/texture_list.json, not {FILE}: nothing to load");
}

#[test]
fn client_install_b_texture_lists_load_without_errors() {
    let Some(client) = game_files::source(game_files::CLIENT_INSTALL_B) else {
        game_files::skipped(game_files::CLIENT_INSTALL_B);
        return;
    };
    smoke(
        "client install B",
        &client.join("resource_packs"),
        PackOrigin::Package,
        // The 130 findings about `.png` entries and duplicates are informations of the crate's
        // own: the game reads the list without a word.
        &[],
    );
}

#[test]
fn client_install_a_texture_lists_load_without_errors() {
    let Some(client) = game_files::source(game_files::CLIENT_INSTALL_A) else {
        game_files::skipped(game_files::CLIENT_INSTALL_A);
        return;
    };
    smoke(
        "client install A",
        &client.join("resource_packs"),
        PackOrigin::Package,
        // The 135 findings about `.png` entries and duplicates are informations of the crate's
        // own: the game reads the list without a word.
        &[],
    );
}
