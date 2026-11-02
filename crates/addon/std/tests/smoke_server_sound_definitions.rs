//! Server sound definitions smoke: no game tree ships the file (no `behavior_packs/*/sounds/`
//! in the client installs or the server install, no `behavior_pack/sounds/` in the sample
//! packs), so the family is covered by its fixtures
//! (`crates/addon/addon/tests/server_sound_definitions.rs`). This test proves the absence on
//! the game files at hand, so a release that starts shipping the file is noticed, and skips with
//! a message otherwise.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_server_sound_definitions`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::path::{Path, PathBuf};

use common::game_files;

/// The behaviour-pack folders below `dir` that have the family's file.
fn packs_with_the_file(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .map(|entry| entry.unwrap().path())
        .filter(|pack| pack.join("sounds/sound_definitions.json").is_file())
        .collect();
    found.sort();
    found
}

#[test]
fn no_game_tree_ships_server_sound_definitions() {
    if game_files::root().is_none() {
        eprintln!("skipped: BEDROCK_ADDON_CORPUS is not set; the family has fixtures only");
        return;
    }
    let mut checked = 0;
    for (name, packs) in [
        (game_files::CLIENT_INSTALL_B, "behavior_packs"),
        (game_files::CLIENT_INSTALL_A, "behavior_packs"),
        (game_files::SERVER_INSTALL, "behavior_packs"),
    ] {
        let Some(root) = game_files::source(name) else {
            eprintln!("skipped: {}", game_files::missing(name));
            continue;
        };
        let found = packs_with_the_file(&root.join(packs));
        assert!(found.is_empty(), "{name} ships the file: {found:#?}");
        checked += 1;
    }
    if let Some(samples) = game_files::source(game_files::SAMPLE_PACKS) {
        assert!(
            !samples
                .join("behavior_pack/sounds/sound_definitions.json")
                .is_file(),
            "The sample packs ship the file"
        );
        checked += 1;
    }
    eprintln!(
        "skipped: none of the {checked} game trees ships a behaviour-pack sounds/sound_definitions.json; fixtures only"
    );
}
