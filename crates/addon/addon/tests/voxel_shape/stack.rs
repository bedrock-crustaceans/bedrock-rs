//! `bp.voxel_shapes()`: rows vs23, vs26, vs27, the walk of
//! `shapes/`, last load wins per identifier, and what stays with the runtime registry.

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::families::voxel_shape::ShapeBox;
use bedrock_addon::id::VoxelShapeId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{
    Behavior, PackOrigin, PackRepository, PackSelection, PackStack, StackSet, WorldPacks,
};

use super::pack_common::{Spec, bp};

/// `a:shape` as the lower half of the block.
const HALF_SLAB: &[u8] = br#"{"format_version": "1.21.110", "minecraft:voxel_shape": {"description": {"identifier": "a:shape"}, "shape": {"boxes": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}}}"#;
/// `a:shape` as a fence post.
const FENCE_POST: &[u8] = br#"{"format_version": "1.21.110", "minecraft:voxel_shape": {"description": {"identifier": "a:shape"}, "shape": {"boxes": [{"min": [6, 0, 6], "max": [10, 16, 10]}]}}}"#;
/// `b:other`, the same boxes as the half slab.
const OTHER: &[u8] = br#"{"format_version": "1.21.110", "minecraft:voxel_shape": {"description": {"identifier": "b:other"}, "shape": {"boxes": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}}}"#;
/// `minecraft:anvil_base`, a name only the base game may use.
const VANILLA: &[u8] = br#"{"format_version": "1.21.110", "minecraft:voxel_shape": {"description": {"identifier": "minecraft:anvil_base"}, "shape": {"boxes": [{"min": [2, 0, 2], "max": [14, 4, 14]}]}}}"#;

const SLAB_BOX: ShapeBox = ShapeBox {
    min: [0.0, 0.0, 0.0],
    max: [16.0, 8.0, 16.0],
};
const POST_BOX: ShapeBox = ShapeBox {
    min: [6.0, 0.0, 6.0],
    max: [10.0, 16.0, 10.0],
};

fn id(text: &str) -> VoxelShapeId {
    VoxelShapeId::new(text).unwrap()
}

/// The behaviour stack of `packs`, top first, none of them the base game's.
fn stack(packs: &[Spec]) -> PackStack<Behavior> {
    PackStack::<Behavior>::of(packs.iter().map(Spec::open), LoadOptions::default()).unwrap()
}

/// The stacks of a world that lists the behaviour packs `packs`, top first, each registered
/// with its origin.
fn world(packs: &[(Spec, PackOrigin)]) -> StackSet {
    let mut repository = PackRepository::new();
    let mut behavior = Vec::new();
    for (spec, origin) in packs {
        repository.add_pack(spec.open(), *origin);
        let id = spec.id();
        behavior.push(PackSelection {
            pack_id: id.uuid,
            version: id.version,
            subpack: None,
        });
    }
    let world = WorldPacks {
        behavior,
        resource: Vec::new(),
    };
    repository.stacks(&world, LoadOptions::default())
}

/// `(code name, severity, pack name, file)` of every finding of the registry.
fn findings(stack: &PackStack<Behavior>) -> Vec<(&'static str, Severity, String, String)> {
    stack
        .voxel_shapes()
        .diagnostics()
        .map(|f| {
            (
                f.code.name(),
                f.severity,
                f.location
                    .pack
                    .as_ref()
                    .map(|pack| pack.name().to_owned())
                    .unwrap_or_default(),
                f.location
                    .path
                    .as_ref()
                    .map(|path| path.as_str().to_owned())
                    .unwrap_or_default(),
            )
        })
        .collect()
}

fn shadowed(pack: &str, path: &str) -> (&'static str, Severity, String, String) {
    (
        "ShadowedDefinition",
        Severity::Info,
        pack.to_owned(),
        path.to_owned(),
    )
}

#[test]
fn vs23_the_higher_pack_s_shape_replaces_the_lower_one_with_an_info_deviation() {
    // Vanilla: silently replaced (`vanilla = "silently replaced"`).
    let stack = stack(&[
        bp(1).file("shapes/shape.json", FENCE_POST),
        bp(2).file("shapes/shape.json", HALF_SLAB),
    ]);
    let shapes = stack.voxel_shapes();
    assert_eq!(shapes.len(), 1);
    assert!(shapes.rejected().is_empty());
    let effective = shapes.get(&id("a:shape")).unwrap();
    assert_eq!(effective.shape.boxes, [POST_BOX]);
    assert_eq!(effective.origin().pack.name(), "bp 1");
    assert_eq!(findings(&stack), [shadowed("bp 2", "shapes/shape.json")]);
    assert!(matches!(
        shapes.diagnostics().next().unwrap().code,
        DiagnosticCode::ShadowedDefinition { text: None }
    ));
    assert!(std::ptr::eq(shapes, stack.voxel_shapes()));
}

#[test]
fn vs23_the_pack_order_decides_not_the_file() {
    let stack = stack(&[
        bp(1).file("shapes/z.json", HALF_SLAB),
        bp(2).file("shapes/a.json", FENCE_POST),
    ]);
    let effective = stack.voxel_shapes().get(&id("a:shape")).unwrap();
    assert_eq!(effective.shape.boxes, [SLAB_BOX]);
    assert_eq!(findings(&stack), [shadowed("bp 2", "shapes/a.json")]);
}

#[test]
fn within_one_pack_the_file_walked_last_wins() {
    let stack = stack(&[bp(1)
        .file("shapes/a.json", HALF_SLAB)
        .file("shapes/b.json", FENCE_POST)]);
    let effective = stack.voxel_shapes().get(&id("a:shape")).unwrap();
    assert_eq!(effective.origin().path.as_str(), "shapes/b.json");
    assert_eq!(effective.shape.boxes, [POST_BOX]);
    assert_eq!(findings(&stack), [shadowed("bp 1", "shapes/a.json")]);
}

#[test]
fn the_identifier_is_the_key_not_the_file_name() {
    let stack = stack(&[
        bp(1).file("shapes/shape.json", OTHER),
        bp(2).file("shapes/shape.json", HALF_SLAB),
    ]);
    let shapes = stack.voxel_shapes();
    assert_eq!(shapes.len(), 2);
    assert!(shapes.get(&id("a:shape")).is_some());
    assert!(shapes.get(&id("b:other")).is_some());
    assert_eq!(findings(&stack), []);
}

#[test]
fn vs26_a_sub_folder_of_shapes_is_read() {
    let stack = stack(&[bp(1).file("shapes/sub/deeper/x.json", HALF_SLAB)]);
    let shape = stack.voxel_shapes().get(&id("a:shape")).unwrap();
    assert_eq!(shape.origin().path.as_str(), "shapes/sub/deeper/x.json");
}

#[test]
fn vs27_only_files_ending_in_json_exactly_are_read() {
    let stack = stack(&[bp(1)
        .file("shapes/x.JSON", HALF_SLAB)
        .file("shapes/x.Json", HALF_SLAB)
        .file("shapes/x.txt", HALF_SLAB)
        .file("shapes/x.json5", HALF_SLAB)
        .file("shapes/json", HALF_SLAB)
        .file("shapes/y.json", OTHER)]);
    let shapes = stack.voxel_shapes();
    assert_eq!(shapes.len(), 1);
    assert!(shapes.get(&id("b:other")).is_some());
    assert!(shapes.rejected().is_empty());
    assert_eq!(findings(&stack), []);
}

#[test]
fn only_the_shapes_directory_is_read() {
    let stack = stack(&[bp(1)
        .file("shape/a.json", HALF_SLAB)
        .file("shapes.json", HALF_SLAB)
        .file("blocks/shapes/a.json", HALF_SLAB)]);
    assert!(stack.voxel_shapes().is_empty());
    assert!(stack.voxel_shapes().rejected().is_empty());
}

#[test]
fn the_extension_is_what_follows_the_last_dot() {
    // Mutation pass.
    let stack = stack(&[bp(1)
        .file("shapes/slab.v2.json", HALF_SLAB)
        .file("shapes/other.json.bak", OTHER)]);
    let shapes = stack.voxel_shapes();
    assert_eq!(shapes.len(), 1);
    let slab = shapes.get(&id("a:shape")).unwrap();
    assert_eq!(slab.origin().path.as_str(), "shapes/slab.v2.json");
}

#[test]
fn shapes_load_without_the_voxel_shapes_experiment_deviation() {
    // Vanilla 1.26.20 loads `shapes/` only with the experiment
    // (`vanilla = "1.26.20: shapes/ loaded only with the voxel_shapes experiment"`).
    let options = LoadOptions::default();
    assert!(options.experiments.is_empty());
    let stack =
        PackStack::<Behavior>::of([bp(1).file("shapes/shape.json", HALF_SLAB).open()], options)
            .unwrap();
    assert_eq!(stack.voxel_shapes().len(), 1);
    assert_eq!(findings(&stack), []);
}

#[test]
fn a_rejected_file_is_listed_and_binds_nothing() {
    let broken: &[u8] = br#"{"format_version": "1.21.110", "minecraft:voxel_shape": {"description": {"identifier": "a:shape"}, "shape": {"boxes": []}}}"#;
    let stack = stack(&[
        bp(1).file("shapes/shape.json", broken),
        bp(2).file("shapes/shape.json", HALF_SLAB),
    ]);
    let shapes = stack.voxel_shapes();
    let effective = shapes.get(&id("a:shape")).unwrap();
    assert_eq!(effective.origin().pack.name(), "bp 2");
    assert_eq!(shapes.rejected().len(), 1);
    assert_eq!(shapes.rejected()[0].origin.pack.name(), "bp 1");
    let found = findings(&stack);
    assert_eq!(
        found,
        [
            (
                "InvalidValue",
                Severity::Error,
                "bp 1".to_owned(),
                "shapes/shape.json".to_owned()
            ),
            (
                "DocumentRejected",
                Severity::Error,
                "bp 1".to_owned(),
                "shapes/shape.json".to_owned()
            ),
        ]
    );
}

#[test]
fn the_namespace_rule_follows_the_pack_s_origin() {
    // The base game's pack may name `minecraft:` shapes and nothing else; a user's pack the
    // other way round.
    let set = world(&[
        (
            bp(1)
                .file("shapes/vanilla.json", VANILLA)
                .file("shapes/custom.json", HALF_SLAB),
            PackOrigin::User,
        ),
        (
            bp(2)
                .file("shapes/vanilla.json", VANILLA)
                .file("shapes/custom.json", OTHER),
            PackOrigin::Package,
        ),
    ]);
    let shapes = set.behavior.voxel_shapes();
    assert_eq!(shapes.len(), 2);
    let vanilla = shapes.get(&id("minecraft:anvil_base")).unwrap();
    assert_eq!(vanilla.origin().pack.name(), "bp 2");
    let custom = shapes.get(&id("a:shape")).unwrap();
    assert_eq!(custom.origin().pack.name(), "bp 1");
    let rejected: Vec<(String, String)> = shapes
        .rejected()
        .iter()
        .map(|r| {
            (
                r.origin.pack.name().to_owned(),
                r.diagnostics.iter().next().unwrap().code.to_string(),
            )
        })
        .collect();
    assert_eq!(
        rejected,
        [
            (
                "bp 2".to_owned(),
                "The `minecraft` namespace is required for Vanilla shape: `b:other`".to_owned()
            ),
            (
                "bp 1".to_owned(),
                "The `minecraft` namespace is not valid for custom voxel shape: `minecraft:anvil_base`"
                    .to_owned()
            ),
        ]
    );
}

#[test]
fn a_base_game_pack_s_sub_folders_are_read_too_deviation() {
    // Vanilla walks an app-packaged pack flat; ours reads every
    // pack's `shapes/` recursively.
    let set = world(&[(
        bp(1).file("shapes/sub/vanilla.json", VANILLA),
        PackOrigin::Package,
    )]);
    assert_eq!(set.behavior.voxel_shapes().len(), 1);
}

#[test]
fn shapes_of_equal_geometry_stay_two_definitions() {
    // De-duplication, the cube and empty short-cuts and the handles are the runtime registry's.
    let stack = stack(&[bp(1)
        .file("shapes/a.json", HALF_SLAB)
        .file("shapes/b.json", OTHER)]);
    let shapes = stack.voxel_shapes();
    assert_eq!(shapes.len(), 2);
    assert_eq!(
        shapes.get(&id("a:shape")).unwrap().shape,
        shapes.get(&id("b:other")).unwrap().shape
    );
    assert_eq!(findings(&stack), []);
}

#[test]
fn v7_more_than_4500_unique_shapes_load_without_a_finding() {
    // "Shape registry full" (V7) is the runtime registry's warning, raised when it builds the
    // unique shapes; loading reports nothing and keeps every document.
    let mut pack = bp(1);
    for n in 0..4501_u32 {
        let path: &'static str = format!("shapes/s{n:04}.json").leak();
        let x = 1.0 + f64::from(n) * 0.003;
        let text: &'static str = format!(
            r#"{{"format_version": "1.21.110", "minecraft:voxel_shape": {{"description": {{"identifier": "many:s{n}"}}, "shape": {{"boxes": [{{"min": [0, 0, 0], "max": [{x}, 8, 16]}}]}}}}}}"#
        )
        .leak();
        pack = pack.file(path, text.as_bytes());
    }
    let stack = stack(&[pack]);
    let shapes = stack.voxel_shapes();
    assert_eq!(shapes.len(), 4501);
    assert!(shapes.rejected().is_empty());
    assert_eq!(shapes.diagnostics().count(), 0);
}
