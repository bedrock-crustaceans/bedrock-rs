//! A synthetic server folder for the examples: two packs and a world that selects them, every
//! file a string literal written for these examples, written into a directory the caller owns.

#![allow(dead_code, reason = "each example uses part of the fixture")]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// `behavior_packs/ruby/manifest.json`.
pub const BEHAVIOR_MANIFEST: &str = r#"{
    "format_version": 2,
    "header": {
        "name": "Ruby behaviour",
        "description": "The behaviour pack of the bedrock_addon_std examples",
        "uuid": "3b2c7a52-8f0e-4b7d-9a35-1c6e5d0f4a11",
        "version": [1, 0, 0],
        "min_engine_version": [1, 26, 20]
    },
    "modules": [
        { "type": "data", "uuid": "8d4f1e63-2a9b-4c0d-b1e7-5f3a6c2d9e22", "version": [1, 0, 0] }
    ]
}"#;

/// `behavior_packs/ruby/blocks/ruby_lamp.json`: a block with one state of its own and one
/// permutation that lights it while the state is set.
pub const RUBY_LAMP: &str = r#"{
    "format_version": "1.26.20",
    "minecraft:block": {
        "description": {
            "identifier": "demo:ruby_lamp",
            "menu_category": { "category": "construction" },
            "states": { "demo:lit": [false, true] }
        },
        "components": {
            "minecraft:destructible_by_mining": { "seconds_to_destroy": 1.5 },
            "minecraft:light_emission": 0
        },
        "permutations": [
            {
                "condition": "q.block_state('demo:lit')",
                "components": { "minecraft:light_emission": 15 }
            }
        ]
    }
}"#;

/// `resource_packs/ruby/manifest.json`.
pub const RESOURCE_MANIFEST: &str = r#"{
    "format_version": 2,
    "header": {
        "name": "Ruby resources",
        "description": "The resource pack of the bedrock_addon_std examples",
        "uuid": "c5e8a914-6d3b-4f2a-8c70-9b1d4e7f3a33",
        "version": [1, 0, 0],
        "min_engine_version": [1, 26, 20]
    },
    "modules": [
        { "type": "resources", "uuid": "f1a2b3c4-5d6e-4f70-8192-a3b4c5d6e744", "version": [1, 0, 0] }
    ]
}"#;

/// `worlds/Bedrock level/world_behavior_packs.json`: the behaviour pack above.
pub const WORLD_BEHAVIOR_PACKS: &str =
    r#"[{ "pack_id": "3b2c7a52-8f0e-4b7d-9a35-1c6e5d0f4a11", "version": [1, 0, 0] }]"#;

/// `worlds/Bedrock level/world_resource_packs.json`: the resource pack above.
pub const WORLD_RESOURCE_PACKS: &str =
    r#"[{ "pack_id": "c5e8a914-6d3b-4f2a-8c70-9b1d4e7f3a33", "version": [1, 0, 0] }]"#;

/// Every file of the server folder, by its path below the folder.
pub const FILES: &[(&str, &str)] = &[
    ("behavior_packs/ruby/manifest.json", BEHAVIOR_MANIFEST),
    ("behavior_packs/ruby/blocks/ruby_lamp.json", RUBY_LAMP),
    ("resource_packs/ruby/manifest.json", RESOURCE_MANIFEST),
    (
        "worlds/Bedrock level/world_behavior_packs.json",
        WORLD_BEHAVIOR_PACKS,
    ),
    (
        "worlds/Bedrock level/world_resource_packs.json",
        WORLD_RESOURCE_PACKS,
    ),
];

/// Where a server keeps its packs and its world, as a dedicated server lays them out.
pub struct Config {
    /// `behavior_packs/`: one folder per behaviour pack.
    pub behavior_packs_directory: PathBuf,
    /// `resource_packs/`: one folder per resource pack.
    pub resource_packs_directory: PathBuf,
    /// `worlds/<level name>/`: the world, with its two pack lists.
    pub world_dir: PathBuf,
}

/// Writes the server folder below `root` and says where its parts are.
pub fn write_server(root: &Path) -> io::Result<Config> {
    for (path, text) in FILES {
        let path = path
            .split('/')
            .fold(root.to_path_buf(), |path, segment| path.join(segment));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, text)?;
    }
    Ok(Config {
        behavior_packs_directory: root.join("behavior_packs"),
        resource_packs_directory: root.join("resource_packs"),
        world_dir: root.join("worlds").join("Bedrock level"),
    })
}
