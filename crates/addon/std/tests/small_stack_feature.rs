#![allow(clippy::pedantic, missing_docs)]
//! A tree feature, the largest document of the crate, loads on a thread with three quarters of
//! a mebibyte of stack, in whatever profile the test is built with.
//!
//! Guards the size of a `Feature`. A tree feature that held every trunk and canopy shape by
//! value would make a `Feature` some 17 KB inline, and the dispatch by payload key keeps one
//! result of that size per feature type in its frame (504 016 bytes for 29 types in an
//! unoptimised build), so the load of one tree feature through a sequential `Driver` would take
//! 1 344 to 1 408 KiB of stack: the base game's load would overflow a 1 MiB thread whenever
//! that thread got a stack of exactly the size it asked for.
//!
//! On purpose its own binary, with ONE test: an overflow aborts the process, and glibc hands a
//! new thread the cached stack of a thread that has ended when that stack is at most four
//! times the size asked for, so a second test whose thread ends first (a test thread has
//! 2 MiB) could give this one more stack than it asks for and hide the overflow.
//!
//! Needs no game files.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs;
use std::path::Path;

use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, StackSet, WorldPacks};
use bedrock_addon_std::{DirSource, Driver, Options, Strategy};

/// The stack of the loading thread: three quarters of the 1 MiB a world load must pass on, so
/// the load of the largest document keeps a quarter of it free.
const SMALL_STACK: usize = 768 * 1024;

const BEHAVIOR: &str = "7a3b2c5e-5c83-4d3f-9a2a-5e3b7d2c8f43";

/// A behaviour pack with one tree feature of vanilla's shape (an acacia trunk and a random
/// spread canopy, as `azalea_tree_feature.json` of the base game has them).
fn tree_world(dir: &Path) {
    let manifest = format!(
        r#"{{
    "format_version": 2,
    "header": {{
        "name": "tree bp",
        "description": "",
        "uuid": "{BEHAVIOR}",
        "version": [1, 0, 0],
        "min_engine_version": [1, 26, 0]
    }},
    "modules": [{{ "type": "data", "uuid": "8b4c3d6f-6d94-4e4a-8b3b-6f4c8e3d9a54", "version": [1, 0, 0] }}]
}}"#
    );
    let feature = r#"{
    "format_version": "1.13.0",
    "minecraft:tree_feature": {
        "description": { "identifier": "demo:azalea_tree_feature" },
        "acacia_trunk": {
            "trunk_width": 1,
            "trunk_height": { "base": 4, "intervals": [2], "min_height_for_canopy": 3 },
            "trunk_block": { "name": "minecraft:log", "states": { "old_log_type": "oak" } },
            "trunk_lean": {
                "allow_diagonal_growth": true,
                "lean_height": { "range_min": 2, "range_max": 3 },
                "lean_steps": { "range_min": 3, "range_max": 4 },
                "lean_length": { "range_min": 1, "range_max": 2 }
            }
        },
        "random_spread_canopy": {
            "canopy_height": 2,
            "canopy_radius": 3,
            "leaf_placement_attempts": 50,
            "leaf_blocks": [["minecraft:azalea_leaves", 3], ["minecraft:azalea_leaves_flowered", 1]]
        },
        "base_block": ["minecraft:dirt_with_roots"],
        "may_grow_on": ["minecraft:dirt", "minecraft:grass"],
        "may_replace": ["minecraft:air"]
    }
}"#;
    for (path, text) in [
        ("tree_bp/manifest.json", manifest.as_str()),
        ("tree_bp/features/azalea_tree_feature.json", feature),
    ] {
        let path = dir.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
}

/// The stacks of the world in `dir`, every collection loaded on the calling thread.
fn load(dir: &Path) -> StackSet {
    let mut driver = Driver::new(Options::default().with_strategy(Strategy::Sequential)).unwrap();
    let repository = driver
        .repository([DirSource::packs(dir.to_path_buf(), PackOrigin::World)])
        .unwrap();
    let world = WorldPacks::parse(
        Some(format!(r#"[{{"pack_id": "{BEHAVIOR}", "version": [1, 0, 0]}}]"#).as_bytes()),
        None,
    )
    .unwrap();
    driver
        .load_stacks(
            repository.stacks(&world, LoadOptions::default()),
            StackOptions::default(),
        )
        .unwrap()
}

#[test]
fn a_tree_feature_loads_on_a_thread_with_three_quarters_of_a_mebibyte_of_stack() {
    let dir = tempfile::tempdir().unwrap();
    tree_world(dir.path());
    let root = dir.path().to_path_buf();
    let (features, rejected) = std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(move || {
            let set = load(&root);
            let features = set.behavior.features();
            (features.len(), features.rejected().len())
        })
        .unwrap()
        .join()
        .expect("the load does not panic");
    assert_eq!((features, rejected), (1, 0), "the tree feature loads");
}
