//! Terrain atlas smoke over the game's own files: the resource packs of the sample packs and of
//! the client installs load through `Driver` into `rp.texture_atlases()`, the terrain atlas of
//! each stack is built, and it has no error-level diagnostic.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo test -p bedrock_addon_std --test
//! smoke_terrain_texture`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::load::{Collection, CollectionSet, LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

use common::game_files;
use common::world::base_game_world;

/// The file the smoke is about.
const FILE: &str = "textures/terrain_texture.json";

/// The pack folders below `dir` that hold `FILE`.
fn packs_with_file(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .filter(|pack| pack.join(FILE).is_file())
                .count()
        })
        .unwrap_or(0)
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
                    Collection::TextureAtlases,
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
            Severity::Warning => *warnings.entry(finding.code.name()).or_default() += 1,
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

/// Loads the resource packs below `dir` and checks the terrain atlas of the stack.
fn smoke(source: &str, dir: PathBuf, origin: PackOrigin, warnings: &[(&str, usize)]) {
    let on_disk = packs_with_file(&dir);
    assert!(
        on_disk > 0,
        "{source}: no pack below {} has {FILE}",
        dir.display()
    );
    let stacks = load(&dir, origin);
    let in_stack = stacked(&stacks);
    let textures = stacks.resource.texture_atlases();
    let atlas = textures
        .terrain()
        .expect("the stack builds a terrain atlas");
    assert_eq!(atlas.texture_name, "atlas.terrain");
    assert!(atlas.len() > 1000, "{source}: {} tiles", atlas.len());
    let layers = textures.terrain_atlas.layers().len();
    check(
        source,
        &format!(
            "{on_disk} pack(s) with {FILE} on disk, {in_stack} in the stack, {layers} merged, {} tiles",
            atlas.len()
        ),
        textures.terrain_atlas.diagnostics(),
        warnings,
    );
    // Every copy of the stack is a layer of the merge.
    assert!(in_stack > 0, "{source}: the stack has the file");
    assert_eq!(
        layers, in_stack,
        "{source}: every copy of the stack is merged"
    );
}

#[test]
fn sample_packs_terrain_atlas_loads_without_errors() {
    let Some(samples) = game_files::source(game_files::SAMPLE_PACKS) else {
        eprintln!("{}", game_files::missing(game_files::SAMPLE_PACKS));
        return;
    };
    smoke(
        "sample packs",
        samples,
        PackOrigin::Package,
        // An atlas layer image that no pack has is an information of the crate's own: the game
        // accepts the entry without a word.
        &[],
    );
}

#[test]
fn client_install_b_terrain_atlas_loads_without_errors() {
    let Some(client) = game_files::source(game_files::CLIENT_INSTALL_B) else {
        eprintln!("{}", game_files::missing(game_files::CLIENT_INSTALL_B));
        return;
    };
    smoke(
        "client install B",
        client.join("resource_packs"),
        PackOrigin::Package,
        // An atlas layer image that no pack has is an information of the crate's own: the game
        // accepts the entry without a word.
        &[],
    );
}

#[test]
fn client_install_a_terrain_atlas_loads_without_errors() {
    let Some(client) = game_files::source(game_files::CLIENT_INSTALL_A) else {
        eprintln!("{}", game_files::missing(game_files::CLIENT_INSTALL_A));
        return;
    };
    smoke(
        "client install A",
        client.join("resource_packs"),
        PackOrigin::Package,
        // An atlas layer image that no pack has is an information of the crate's own: the game
        // accepts the entry without a word.
        &[],
    );
}
