//! Fixture tests of the container readers. They live beside the code because the readers are
//! internal surface (and some of what the tests call is test-only); the fixtures are under
//! `tests/fixtures/`, and the vanilla archives under the folder named by `BEDROCK_ADDON_CORPUS`
//! (skipped when unset). The tests that go through the pack layer, the loader or a family stay
//! in `bedrock_addon` (`src/storage/tests/`).

// Test code reads fixture files; library code never does.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

mod brarchive;
mod contents;
#[cfg(feature = "encrypted")]
mod encrypted;
mod magic;
mod mcb;
#[cfg(feature = "zip")]
mod zip;

use std::path::PathBuf;

/// `tests/fixtures/<relative>`.
pub(super) fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative)
}

/// The bytes of `tests/fixtures/<relative>`.
pub(super) fn fixture(relative: &str) -> Vec<u8> {
    let path = fixture_path(relative);
    std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// Every file whose name ends in `suffix` below `$BEDROCK_ADDON_CORPUS/<top>` for each of
/// `tops`, sorted; `None` (with a message) when the variable is unset or a directory is missing,
/// so the test is skipped.
pub(super) fn game_files(tops: &[&str], suffix: &str) -> Option<Vec<PathBuf>> {
    let Some(root) = std::env::var_os("BEDROCK_ADDON_CORPUS") else {
        // The line of `bedrock_addon_test_support::corpus`, spelled here: this crate's sources
        // name core alone.
        eprintln!("SKIPPED (not a pass): the game's files: BEDROCK_ADDON_CORPUS is not set");
        return None;
    };
    let mut files = Vec::new();
    for top in tops {
        let dir = PathBuf::from(&root).join(top);
        if !dir.is_dir() {
            eprintln!(
                "SKIPPED (not a pass): the game's files: {} is missing",
                dir.display()
            );
            return None;
        }
        let mut stack = vec![dir];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else if path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(suffix))
                {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    Some(files)
}

/// The file names in `tests/fixtures/<relative>`, sorted.
pub(super) fn fixture_names(relative: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(fixture_path(relative))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}
