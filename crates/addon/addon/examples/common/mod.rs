//! The two synthetic packs the examples read: a behaviour pack and a resource pack, every file a
//! literal written for these examples. Nothing here is read from disk.

#![allow(dead_code, reason = "each example uses part of the fixture")]

use bedrock_addon::document::Rejected;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{
    OpenError, OpenOptions, Pack, PackOrigin, PackRepository, StackSet, WorldPacks,
};
use bedrock_addon::storage::PackPath;

/// The behaviour pack's `manifest.json`.
pub const BEHAVIOR_MANIFEST: &str = r#"{
    "format_version": 2,
    "header": {
        "name": "Ruby behaviour",
        "description": "The behaviour pack of the bedrock_addon examples",
        "uuid": "3b2c7a52-8f0e-4b7d-9a35-1c6e5d0f4a11",
        "version": [1, 0, 0],
        "min_engine_version": [1, 26, 20]
    },
    "modules": [
        { "type": "data", "uuid": "8d4f1e63-2a9b-4c0d-b1e7-5f3a6c2d9e22", "version": [1, 0, 0] }
    ]
}"#;

/// The resource pack's `manifest.json`.
pub const RESOURCE_MANIFEST: &str = r#"{
    "format_version": 2,
    "header": {
        "name": "Ruby resources",
        "description": "The resource pack of the bedrock_addon examples",
        "uuid": "c5e8a914-6d3b-4f2a-8c70-9b1d4e7f3a33",
        "version": [1, 0, 0],
        "min_engine_version": [1, 26, 20]
    },
    "modules": [
        { "type": "resources", "uuid": "f1a2b3c4-5d6e-4f70-8192-a3b4c5d6e744", "version": [1, 0, 0] }
    ]
}"#;

/// What the world's `world_behavior_packs.json` says: the behaviour pack above.
pub const WORLD_BEHAVIOR_PACKS: &str =
    r#"[{ "pack_id": "3b2c7a52-8f0e-4b7d-9a35-1c6e5d0f4a11", "version": [1, 0, 0] }]"#;

/// What the world's `world_resource_packs.json` says: the resource pack above.
pub const WORLD_RESOURCE_PACKS: &str =
    r#"[{ "pack_id": "c5e8a914-6d3b-4f2a-8c70-9b1d4e7f3a33", "version": [1, 0, 0] }]"#;

/// `blocks/ruby_lamp.json`: a block with one state of its own and one permutation that lights
/// it while the state is set.
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

/// `entities/walker.json`: an entity written in an older format, with two component groups
/// that set its speed.
pub const WALKER: &str = r#"{
    "format_version": "1.16.0",
    "minecraft:entity": {
        "description": {
            "identifier": "demo:walker",
            "is_spawnable": true,
            "is_summonable": true
        },
        "component_groups": {
            "demo:slow": { "minecraft:movement": { "value": 0.1 } },
            "demo:fast": { "minecraft:movement": { "value": 0.35 } }
        },
        "components": {
            "minecraft:type_family": { "family": ["demo", "walker"] },
            "minecraft:collision_box": { "width": 0.5, "height": 1.2 },
            "minecraft:physics": {},
            "minecraft:health": { "value": 12, "max": 12 }
        },
        "events": {
            "demo:hurry": { "add": { "component_groups": ["demo:fast"] } }
        }
    }
}"#;

/// The files of the behaviour pack, by their path in the pack.
pub const BEHAVIOR_FILES: &[(&str, &[u8])] = &[
    ("manifest.json", BEHAVIOR_MANIFEST.as_bytes()),
    ("blocks/ruby_lamp.json", RUBY_LAMP.as_bytes()),
    ("entities/walker.json", WALKER.as_bytes()),
];

/// `entity/walker.entity.json`: how the client draws `demo:walker`, written in an older format.
pub const WALKER_CLIENT: &str = r#"{
    "format_version": "1.10.0",
    "minecraft:client_entity": {
        "description": {
            "identifier": "demo:walker",
            "materials": { "default": "entity_alphatest" },
            "textures": { "default": "textures/entity/walker" },
            "geometry": { "default": "geometry.walker" },
            "render_controllers": ["controller.render.default"]
        }
    }
}"#;

/// `entity/ghost.entity.json`: a client entity whose geometry and texture no pack has, and
/// which no behaviour pack defines.
pub const GHOST_CLIENT: &str = r#"{
    "format_version": "1.10.0",
    "minecraft:client_entity": {
        "description": {
            "identifier": "demo:ghost",
            "materials": { "default": "entity_alphatest" },
            "textures": { "default": "textures/entity/ghost" },
            "geometry": { "default": "geometry.ghost" },
            "render_controllers": ["controller.render.default"]
        }
    }
}"#;

/// `models/entity/walker.geo.json`: one bone with one cube.
pub const WALKER_GEOMETRY: &str = r#"{
    "format_version": "1.12.0",
    "minecraft:geometry": [
        {
            "description": {
                "identifier": "geometry.walker",
                "texture_width": 16,
                "texture_height": 16
            },
            "bones": [
                {
                    "name": "body",
                    "pivot": [0, 0, 0],
                    "cubes": [{ "origin": [-4, 0, -4], "size": [8, 8, 8], "uv": [0, 0] }]
                }
            ]
        }
    ]
}"#;

/// `textures/entity/walker.png`: a PNG of one pixel.
pub const WALKER_TEXTURE: &[u8] =
    b"\x89\x50\x4e\x47\x0d\x0a\x1a\x0a\x00\x00\x00\x0d\x49\x48\x44\x52\x00\x00\x00\x01\
    \x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89\x00\x00\x00\x0d\x49\x44\x41\
    \x54\x78\x9c\x63\x38\xa1\x60\xf0\x1f\x00\x04\xe4\x02\x18\xcc\xde\xb5\x07\x00\x00\
    \x00\x00\x49\x45\x4e\x44\xae\x42\x60\x82";

/// The files of the resource pack, by their path in the pack.
pub const RESOURCE_FILES: &[(&str, &[u8])] = &[
    ("manifest.json", RESOURCE_MANIFEST.as_bytes()),
    ("entity/walker.entity.json", WALKER_CLIENT.as_bytes()),
    ("entity/ghost.entity.json", GHOST_CLIENT.as_bytes()),
    ("models/entity/walker.geo.json", WALKER_GEOMETRY.as_bytes()),
    ("textures/entity/walker.png", WALKER_TEXTURE),
];

/// A pack of `files`, opened from memory.
fn open(files: &[(&'static str, &'static [u8])]) -> Result<Pack, OpenError> {
    Pack::from_files(
        files
            .iter()
            .map(|(path, bytes)| (PackPath::from_static(path), *bytes)),
        &OpenOptions::default(),
    )
}

/// The behaviour pack, opened from memory.
pub fn behavior_pack() -> Result<Pack, OpenError> {
    open(BEHAVIOR_FILES)
}

/// The resource pack, opened from memory.
pub fn resource_pack() -> Result<Pack, OpenError> {
    open(RESOURCE_FILES)
}

/// The world's choice of packs: both of them.
pub fn world_packs() -> Result<WorldPacks, Rejected> {
    WorldPacks::parse(
        Some(WORLD_BEHAVIOR_PACKS.as_bytes()),
        Some(WORLD_RESOURCE_PACKS.as_bytes()),
    )
}

/// Both stacks of a world that uses the two packs, built without touching a file: the packs
/// are registered in a repository and the world's lists select them.
pub fn stack_set() -> Result<StackSet, Box<dyn std::error::Error>> {
    let mut repository = PackRepository::new();
    repository.add_pack(behavior_pack()?, PackOrigin::World);
    repository.add_pack(resource_pack()?, PackOrigin::World);
    Ok(repository.stacks(&world_packs()?, LoadOptions::default()))
}

/// `blocks/broken.json`: a block without a `description`, so without an identifier. It cannot
/// load: an error.
pub const BROKEN_BLOCK: &str =
    r#"{ "format_version": "1.26.20", "minecraft:block": { "components": {} } }"#;

/// `entities/keeper.json`: an entity that loads, and names two things no pack has: the block
/// of its trail (the validator's error, the entity kept) and its loot table (the game
/// keeps the path and logs nothing: a lint of the link pass).
pub const KEEPER: &str = r#"{
    "format_version": "1.26.20",
    "minecraft:entity": {
        "description": {
            "identifier": "demo:keeper",
            "is_spawnable": true,
            "is_summonable": true
        },
        "components": {
            "minecraft:physics": {},
            "minecraft:trail": { "block_type": "demo:missing_block" },
            "minecraft:loot": { "table": "loot_tables/entities/keeper.json" }
        }
    }
}"#;

/// The files of the behaviour pack the `diagnostics_gate` example checks: one sound block and
/// the two documents with the deliberate faults.
pub const GATE_FILES: &[(&str, &[u8])] = &[
    ("manifest.json", BEHAVIOR_MANIFEST.as_bytes()),
    ("blocks/ruby_lamp.json", RUBY_LAMP.as_bytes()),
    ("blocks/broken.json", BROKEN_BLOCK.as_bytes()),
    ("entities/keeper.json", KEEPER.as_bytes()),
];

/// Both stacks of a world that uses the one behaviour pack of [`GATE_FILES`] and no resource
/// pack.
pub fn gate_stack_set() -> Result<StackSet, Box<dyn std::error::Error>> {
    let mut repository = PackRepository::new();
    repository.add_pack(open(GATE_FILES)?, PackOrigin::World);
    let world = WorldPacks::parse(Some(WORLD_BEHAVIOR_PACKS.as_bytes()), None)?;
    Ok(repository.stacks(&world, LoadOptions::default()))
}
