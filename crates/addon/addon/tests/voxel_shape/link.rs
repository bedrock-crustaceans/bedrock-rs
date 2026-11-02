//! The rule for a block's
//! `minecraft:geometry.culling_shape` that names no shape of the stack. The check reads two
//! registries, so it runs in the link pass. Its body is
//! written against `StackSet::link`.

use bedrock_addon::diagnostic::{DiagnosticCode, Diagnostics, Severity};
use bedrock_addon::id::BlockId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, StackSet, WorldPacks};

use super::pack_common::{Spec, bp};

/// A 1.26.20 block `demo:<name>` whose geometry culls with `culling_shape`.
fn block(name: &str, culling_shape: &str) -> &'static [u8] {
    format!(
        r#"{{"format_version": "1.26.20", "minecraft:block": {{"description": {{"identifier": "demo:{name}"}}, "components": {{"minecraft:geometry": {{"identifier": "geometry.demo", "culling_shape": "{culling_shape}"}}, "minecraft:material_instances": {{"*": {{"texture": "demo"}}}}}}}}}}"#
    )
    .leak()
    .as_bytes()
}

const SLAB: &[u8] = br#"{"format_version": "1.21.110", "minecraft:voxel_shape": {"description": {"identifier": "demo:slab"}, "shape": {"boxes": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}}}"#;

/// The stacks of a world with the behaviour pack `spec`.
fn stacks(spec: &Spec) -> StackSet {
    let mut repository = PackRepository::new();
    repository.add_pack(spec.open(), PackOrigin::User);
    let id = spec.id();
    let world = WorldPacks {
        behavior: vec![PackSelection {
            pack_id: id.uuid,
            version: id.version,
            subpack: None,
        }],
        resource: Vec::new(),
    };
    repository.stacks(&world, LoadOptions::default())
}

/// The geometry and the terrain tile the blocks of [`block`] name: without them the link pass
/// tells of each block's geometry and texture too.
const RESOURCES: [(&str, &[u8]); 2] = [
    (
        "models/blocks/demo.geo.json",
        br#"{"format_version": "1.16.0", "minecraft:geometry": [{"description": {"identifier": "geometry.demo"}, "bones": [{"name": "body"}]}]}"#,
    ),
    (
        "textures/terrain_texture.json",
        br#"{"texture_name": "atlas.terrain", "texture_data": {"demo": {"textures": "textures/blocks/demo"}}}"#,
    ),
];

/// The stacks of a world with the behaviour pack `spec` and a resource pack of [`RESOURCES`].
fn stacks_with_resources(spec: &Spec) -> StackSet {
    let mut resources = super::pack_common::rp(2);
    for (path, bytes) in RESOURCES {
        resources = resources.file(path, bytes);
    }
    let mut repository = PackRepository::new();
    repository.add_pack(spec.open(), PackOrigin::User);
    repository.add_pack(resources.open(), PackOrigin::User);
    let selection = |spec: &Spec| {
        let id = spec.id();
        PackSelection {
            pack_id: id.uuid,
            version: id.version,
            subpack: None,
        }
    };
    let world = WorldPacks {
        behavior: vec![selection(spec)],
        resource: vec![selection(&resources)],
    };
    repository.stacks(&world, LoadOptions::default())
}

/// The findings about culling shapes among `d`.
fn unresolved(d: &Diagnostics) -> Vec<(Severity, String)> {
    d.iter()
        .filter(|f| matches!(f.code, DiagnosticCode::UnresolvedReference { .. }))
        .map(|f| (f.severity, f.code.to_string()))
        .collect()
}

#[test]
fn the_blocks_of_the_link_test_load() {
    // What the ignored test below builds on: the three blocks and the shape load on their own.
    let pack = bp(1)
        .file("blocks/missing.json", block("missing", "test:missing"))
        .file("blocks/slab.json", block("slab", "demo:slab"))
        .file("blocks/cube.json", block("cube", "minecraft:unit_cube"))
        .file("shapes/slab.json", SLAB);
    let set = stacks(&pack);
    let blocks = set.behavior.blocks();
    assert!(blocks.rejected().is_empty(), "{:#?}", blocks.rejected());
    for name in ["demo:missing", "demo:slab", "demo:cube"] {
        assert!(blocks.get(&BlockId::new(name).unwrap()).is_some(), "{name}");
    }
    assert_eq!(set.behavior.voxel_shapes().len(), 1);
    // Loading alone says nothing about the reference.
    assert_eq!(
        blocks
            .diagnostics()
            .filter(|f| f.severity >= Severity::Warning)
            .count(),
        0
    );
}

#[test]
fn b3_an_unknown_culling_shape_is_unresolved_deviation() {
    // Retail 1.26.45 wording with the default shape's identifier
    // (`vanilla = "1.26.20 wording: Defaulting to empty shape"`).
    let pack = bp(1)
        .file("blocks/missing.json", block("missing", "test:missing"))
        .file("blocks/slab.json", block("slab", "demo:slab"))
        .file("blocks/cube.json", block("cube", "minecraft:unit_cube"))
        .file("blocks/empty.json", block("empty", "minecraft:empty"))
        .file("shapes/slab.json", SLAB);
    let found = stacks_with_resources(&pack).link();
    // The defined shape and the two built-in ones resolve; only `test:missing` is reported,
    // at level 3 and the block keeps the default: a kept error.
    assert_eq!(
        unresolved(&found),
        [(
            Severity::Error,
            "Voxel shape with name \"test:missing\" doesn't exist in the registry. Defaulting to minecraft:empty for identifier."
                .to_owned()
        )]
    );
}

/// A 1.26.20 block `demo:<name>` with geometry `geometry` culling with `culling_shape`, and
/// `more` further components (each with its leading comma).
fn block_with(name: &str, geometry: &str, culling_shape: &str, more: &str) -> &'static [u8] {
    format!(
        r#"{{"format_version": "1.26.20", "minecraft:block": {{"description": {{"identifier": "demo:{name}"}}, "components": {{"minecraft:geometry": {{"identifier": "{geometry}", "culling_shape": "{culling_shape}"}}, "minecraft:material_instances": {{"*": {{"texture": "demo"}}}}{more}}}}}}}"#
    )
    .leak()
    .as_bytes()
}

/// The findings at warning level or above of the blocks and shapes of `pack`, and how many of
/// each loaded.
fn load_findings(pack: &Spec) -> (usize, usize, Vec<String>) {
    let set = stacks(pack);
    let blocks = set.behavior.blocks();
    let shapes = set.behavior.voxel_shapes();
    let found = blocks
        .diagnostics()
        .chain(shapes.diagnostics())
        .filter(|f| f.severity >= Severity::Warning)
        .map(std::string::ToString::to_string)
        .collect();
    (blocks.len(), shapes.len(), found)
}

#[test]
fn b4_a_full_block_geometry_with_a_culling_shape_loads_without_a_finding() {
    // "A full block geometry cannot have a custom culling shape … the unit cube shape will be
    // used instead" is the runtime registry's warning when it finalises the block;
    // loading keeps what was written.
    let pack = bp(1)
        .file(
            "blocks/full.json",
            block_with("full", "minecraft:geometry.full_block", "demo:slab", ""),
        )
        .file("shapes/slab.json", SLAB);
    assert_eq!(load_findings(&pack), (1, 1, Vec::new()));
}

#[test]
fn b3a_b5_b6_a_transformed_block_naming_a_defined_shape_loads_without_a_finding() {
    // The transformed copy of the shape (B5), its face shapes (B6) and the invalid handle of a
    // shape a full registry dropped (B3a) exist only in the runtime registry: nothing
    // of them is a load finding, with or without the shape in the stack.
    let rotated = r#", "minecraft:transformation": {"rotation": [0, 90, 0]}"#;
    let pack = bp(1)
        .file(
            "blocks/rotated.json",
            block_with("rotated", "geometry.demo", "demo:slab", rotated),
        )
        .file(
            "blocks/dangling.json",
            block_with("dangling", "geometry.demo", "demo:dropped", rotated),
        )
        .file("shapes/slab.json", SLAB);
    assert_eq!(load_findings(&pack), (2, 1, Vec::new()));
}
