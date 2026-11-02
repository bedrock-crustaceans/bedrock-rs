//! `blocks.json` smoke over the game's own files: the `blocks.json` of every resource pack of
//! the sample packs and of the client installs loads through `Driver` into
//! `rp.client_blocks()` with no error-level diagnostic.
//!
//! The packs are the base game's, so a legacy name at or above its gate is kept at debug
//! level, and no pack sets a `blockshape`. The merged result depends on the order of the
//! resource packs: the world is the shared helper's, in the client's base-game resource order,
//! and the numbers and the one order-dependent value below are pinned for it.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_blocks_json`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::Path;

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

use common::game_files;
use common::world::base_game_world;

/// How many pack folders directly below `dir` hold a `blocks.json`.
fn on_disk(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| entry.path().join("blocks.json").is_file())
                .count()
        })
        .unwrap_or_default()
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

/// What a load of a tree gave.
#[derive(Debug, PartialEq, Eq)]
struct Counts {
    /// The blocks of the registry.
    blocks: usize,
    /// The blocks more than one pack contributed to.
    merged: usize,
    /// The files an entry of which is part of a block.
    files_read: usize,
    /// The `blocks.json` files on disk.
    files_on_disk: usize,
    /// The resource packs in the stack.
    packs: usize,
    /// The pack whose entry is on top of `white_wool`, and the block's `up` texture: both
    /// depend on the order of the resource packs.
    white_wool: (String, String),
}

/// Loads the `blocks.json` of every resource pack below `dir` and requires no error, and
/// the warnings by code to be `warnings`, exactly.
fn check(source: &str, dir: &Path, warnings: &[(&str, usize)]) -> Counts {
    let files = on_disk(dir);
    let set = stacks(dir);
    let blocks = set.resource.client_blocks();
    let mut errors = Vec::new();
    let mut by_code: BTreeMap<(String, &str), usize> = BTreeMap::new();
    for finding in blocks.diagnostics() {
        *by_code
            .entry((finding.severity.to_string(), finding.code.name()))
            .or_default() += 1;
        if finding.severity >= Severity::Error {
            errors.push(finding.to_string());
        }
    }
    // The packs whose entry is part of a merged block: one file each. (A file whose every
    // entry names an old name or nothing at all contributes to no block.)
    let mut read: Vec<usize> = Vec::new();
    for (_, document) in blocks {
        let origin = document.origin();
        for pack in std::iter::once(&origin.pack).chain(&origin.contributors) {
            if let Some(index) = pack.index()
                && !read.contains(&index)
            {
                read.push(index);
            }
        }
    }
    let merged = blocks
        .iter()
        .filter(|(_, d)| !d.origin().contributors.is_empty())
        .count();
    eprintln!(
        "{source}: {} blocks ({merged} merged from more than one pack) from {} of {files} files in {} resource packs, {} rejected, {} errors; findings {by_code:?}",
        blocks.len(),
        read.len(),
        set.resource.packs().len(),
        blocks.rejected().len(),
        errors.len(),
    );
    for error in &errors {
        eprintln!("{source}: {error}");
    }
    assert!(errors.is_empty(), "{source}: {} errors", errors.len());
    game_files::assert_warnings(
        source,
        &game_files::warnings_by_code(blocks.diagnostics()),
        warnings,
    );
    assert!(blocks.rejected().is_empty(), "{source}: every file is JSON");
    assert!(files > 0, "{source}: no blocks.json on disk");
    assert!(!blocks.is_empty(), "{source}: no block loaded");
    let white_wool = blocks
        .get(&bedrock_addon::id::BlockId::parse("minecraft:white_wool"))
        .map(|block| {
            (
                block.origin().pack.name().to_owned(),
                block.face_textures()[0].to_owned(),
            )
        })
        .unwrap_or_default();
    Counts {
        blocks: blocks.len(),
        merged,
        files_read: read.len(),
        files_on_disk: files,
        packs: set.resource.packs().len(),
        white_wool,
    }
}

/// The sample packs file (1,352 entries at
/// `"1.21.40"`) loads as 1,320 blocks: 31 legacy names and `lava_cauldron` apply to nothing.
#[test]
fn sample_packs_blocks_json_loads_without_errors() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    assert_eq!(on_disk(&samples), 1, "resource_pack/blocks.json");
    // `coral_block/sounds` is noted as an information: the game accepts the value without a word.
    let counts = check("sample packs", &samples, &[("InvalidType", 2)]);
    assert_eq!(
        counts,
        Counts {
            blocks: 1320,
            merged: 0,
            files_read: 1,
            files_on_disk: 1,
            packs: 1,
            white_wool: (
                "Vanilla Resource Pack".to_owned(),
                "wool_colored_white".to_owned()
            ),
        }
    );
}

/// Client install B: 34 packs carry a `blocks.json`.
#[test]
fn client_install_b_blocks_json_loads_without_errors() {
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let dir = install_b.join("resource_packs");
    assert_eq!(on_disk(&dir), 34, "the packs with a blocks.json");
    let counts = check("client install B", &dir, &[]);
    // 30 of the 34 files are in the 64 packs of the helper's world. The flattened
    // slices sit above `vanilla` in the client's resource order, so white wool has a slice's texture name,
    // not the `wool:0` that `vanilla`'s legacy entry is split into.
    assert_eq!(
        counts,
        Counts {
            blocks: 1273,
            merged: 376,
            files_read: 30,
            files_on_disk: 34,
            packs: 64,
            white_wool: (
                "resourcePack.vanilla.name".to_owned(),
                "wool_colored_white".to_owned()
            ),
        }
    );
}

/// Client install A is a smoke tree like client install B; skipped with a message when absent.
#[test]
fn client_install_a_blocks_json_loads_without_errors() {
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    let counts = check("client install A", &install_a.join("resource_packs"), &[]);
    // As client install B, with one file and one experimental pack fewer in the stack's world.
    assert_eq!(
        counts,
        Counts {
            blocks: 1199,
            merged: 376,
            files_read: 29,
            files_on_disk: 33,
            packs: 71,
            white_wool: (
                "resourcePack.vanilla.name".to_owned(),
                "wool_colored_white".to_owned()
            ),
        }
    );
}
