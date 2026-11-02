//! Atomic client entity smoke over the game's own files. No vanilla pack ships an
//! `atomic_client_entities/` folder (none in the client installs, the server install or the
//! The sample packs), so there is nothing to load: the test proves the folder is absent from every
//! tree and says so. The day a tree gains the
//! folder the test fails, and the smoke proper is written against real files. Until then the
//! family is covered by the fixtures of `crates/addon/addon/tests/atomic_client_entity`.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo test -p bedrock_addon_std --test
//! smoke_atomic_client_entity`; without the variable the test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::path::{Path, PathBuf};

use common::game_files;

/// Every `atomic_client_entities` directory below `root`, at most `depth` levels down.
fn atomic_directories(root: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("atomic_client_entities") {
            found.push(path);
        } else if depth > 0 {
            atomic_directories(&path, depth - 1, found);
        }
    }
}

#[test]
fn no_game_tree_ships_atomic_client_entities() {
    let _ = skip!(game_files::SAMPLE_PACKS);
    let mut found = Vec::new();
    let mut searched = Vec::new();
    for name in [
        game_files::SAMPLE_PACKS,
        game_files::SERVER_INSTALL,
        game_files::CLIENT_INSTALL_B,
        game_files::CLIENT_INSTALL_A,
    ] {
        match game_files::source(name) {
            // Pack roots are at most three levels below a tree's root
            // (`resource_packs/<pack>/subpacks/<name>`).
            Some(root) => {
                atomic_directories(&root, 4, &mut found);
                searched.push(name);
            }
            None => eprintln!("skipped: {}", game_files::missing(name)),
        }
    }
    eprintln!(
        "skipped: no `atomic_client_entities/` folder in {searched:?}; the family has fixtures only"
    );
    assert!(
        found.is_empty(),
        "a game tree now ships atomic client entities; write the smoke over them: {found:#?}"
    );
}
