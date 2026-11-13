//! The stack helpers of the behavior-tree tests: packs from memory or from a fixture, the
//! stacks of them, and the check of a fixture's stack against its golden files.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use std::path::PathBuf;

use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;
use bytes::Bytes;
use uuid::Uuid;

use super::common::*;

/// A behaviour pack `n`, opened from memory, holding `files` (pack-relative paths).
pub fn pack(n: u128, files: Vec<(String, Vec<u8>)>) -> Pack {
    let uuid = Uuid::from_u128(n);
    let module = Uuid::from_u128(n ^ (0xabcd_u128 << 64));
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "bp {n}", "description": "", "uuid": "{uuid}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}}, "modules": [{{"type": "data", "uuid": "{module}", "version": [1, 0, 0]}}]}}"#
    );
    let mut all: Vec<(PackPath, Bytes)> = vec![(
        PackPath::from_static("manifest.json"),
        Bytes::from(manifest),
    )];
    all.extend(
        files
            .into_iter()
            .map(|(path, bytes)| (PackPath::new(&path).unwrap(), Bytes::from(bytes))),
    );
    let options = OpenOptions {
        origin: PackOrigin::Unknown,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("the pack opens")
}

/// A pack holding the files `(path below behavior_trees/, text)`.
pub fn tree_pack(n: u128, files: &[(&str, &str)]) -> Pack {
    pack(
        n,
        files
            .iter()
            .map(|(path, text)| (format!("behavior_trees/{path}"), text.as_bytes().to_vec()))
            .collect(),
    )
}

/// The packs of fixture `name`, top first.
pub fn fixture_packs(name: &str) -> Vec<Pack> {
    let fixture = fixtures().join(name);
    let single = fixture.join("pack");
    if single.is_dir() {
        return vec![pack(1, files_below(&single))];
    }
    let stack = fixture.join("stack");
    if !stack.is_dir() {
        return Vec::new();
    }
    let mut roots: Vec<PathBuf> = std::fs::read_dir(&stack)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    roots.sort();
    roots
        .iter()
        .enumerate()
        .map(|(index, root)| {
            let folder = root.file_name().unwrap().to_string_lossy().into_owned();
            assert!(
                folder.starts_with(&format!("{index}_")),
                "{folder}: stack folders are <index>_<pack>, index 0 the highest priority"
            );
            pack(index as u128 + 1, files_below(root))
        })
        .collect()
}

/// A behaviour stack of `packs`, top first.
pub fn stack_of(packs: Vec<Pack>) -> PackStack<Behavior> {
    PackStack::<Behavior>::of(packs, LoadOptions::default()).unwrap()
}

/// The behaviour stack of fixture `name`.
pub fn fixture_stack(name: &str) -> PackStack<Behavior> {
    stack_of(fixture_packs(name))
}

/// Loads fixture `name` and requires its registry and findings to equal `expected.json` and
/// `expected.toml`; returns the stack for the row's own assertions.
pub fn check(name: &str) -> PackStack<Behavior> {
    let stack = fixture_stack(name);
    let registry = stack.behavior_trees();
    assert_registry(name, registry);
    assert_findings(name, registry);
    stack
}
