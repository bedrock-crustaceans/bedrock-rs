#![allow(clippy::pedantic, missing_docs)]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
//! Pins of present behaviour: what a refused definition in the pack that would win does to a
//! valid definition of the same identifier (or the same path) in the other packs of a behaviour
//! stack. Either another pack's definition is effective ("fallback") or the identifier stays
//! without one ("taken").
//!
//! Families: dimensions, POI types, dialogue scenes, functions, trade tables, voxel shapes,
//! aim-assist presets, behaviour-pack animations and animation controllers, behavior trees and
//! camera presets. Per family one test with two packs and one with three; the three-pack test
//! says which of the two valid definitions is effective. Behavior trees and camera presets
//! have two kinds of refusal that behave differently, so each kind has its pair.
//!
//! The packs are named `top`, `middle` and `low` by their place in the stack; the effective
//! definition is told by the name of its origin pack. Dimensions and dialogue scenes are
//! bottom-wins families: there the refused definition is in the lowest pack, the winner's
//! place.

#![cfg(feature = "full")]

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::{Definition, Registry, Rejected};
use bedrock_addon::id::{
    AimAssistPresetId, AnimationRef, BehaviorTreeId, CameraPresetId, DialogueSceneTag, DimensionId,
    FunctionPath, Id, PoiTypeId, VoxelShapeId,
};
use bedrock_addon::load::{Experiment, Experiments, LoadOptions};
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

// ---- the packs ------------------------------------------------------------------------------

/// A user behaviour pack named `name` holding `text` at `path`.
fn pack(name: &str, path: &str, text: &str) -> Pack {
    let n: u128 = match name {
        "top" => 1,
        "middle" => 2,
        _ => 3,
    };
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 20]}}, "modules": [{{"type": "data", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(n),
        Uuid::from_u128(n + 1000),
    );
    let files = vec![
        (
            PackPath::new("manifest.json").unwrap(),
            manifest.into_bytes(),
        ),
        (PackPath::new(path).unwrap(), text.as_bytes().to_vec()),
    ];
    let options = OpenOptions {
        origin: PackOrigin::User,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).expect("the pack opens")
}

/// The behaviour stack of the packs `top`, `middle` and `low` (those given), top first, each
/// holding its text at `path`.
fn stack_with(path: &str, texts: &[(&str, &str)], options: LoadOptions) -> PackStack<Behavior> {
    let packs: Vec<Pack> = texts
        .iter()
        .map(|(name, text)| pack(name, path, text))
        .collect();
    PackStack::<Behavior>::of(packs, options).expect("a behaviour stack")
}

/// [`stack_with`] the default options.
fn stack(path: &str, texts: &[(&str, &str)]) -> PackStack<Behavior> {
    stack_with(path, texts, LoadOptions::default())
}

// ---- what a test reads ----------------------------------------------------------------------

/// The name of the pack whose definition of `id` is effective.
fn effective<'r, T: Definition>(registry: &'r Registry<T>, id: &Id<T::Kind>) -> Option<&'r str> {
    registry.get(id).map(|d| d.origin().pack.name())
}

/// The code names of the error findings at the file `path` of the pack named `pack`.
fn errors_at<'a>(
    findings: impl Iterator<Item = &'a Diagnostic>,
    pack: &str,
    path: &str,
) -> Vec<&'static str> {
    findings
        .filter(|f| f.severity == Severity::Error)
        .filter(|f| f.location.pack.as_ref().is_some_and(|p| p.name() == pack))
        .filter(|f| f.location.path.as_ref().is_some_and(|p| p.as_str() == path))
        .map(|f| f.code.name())
        .collect()
}

/// The names of the packs whose documents were rejected.
fn rejected_packs(rejected: &[Rejected]) -> Vec<&str> {
    rejected.iter().map(|r| r.origin.pack.name()).collect()
}

// ---- dimensions -----------------------------------------------------------------------------
//
// The file is skipped on a parse error (missing required member, ...) or an `Error` from the
// validator. Nothing is inserted, and the other files and packs continue. An insert happens only
// when the key is absent, so the lowest-priority pack's file wins.
// Present behaviour: fallback. The winner's place is the lowest pack; its refused file claims
// nothing and the lowest valid definition above it is effective.

const DIMENSION_PATH: &str = "dimensions/overworld.json";
const DIMENSION: &str = r#"{"format_version": "1.21.60", "minecraft:dimension": {"description": {"identifier": "minecraft:overworld"}, "components": {"minecraft:dimension_bounds": {"min": -64, "max": 320}, "minecraft:generation": {"generator_type": "void"}}}}"#;
/// No component: the document names the overworld and is refused.
const DIMENSION_REFUSED: &str = r#"{"format_version": "1.21.60", "minecraft:dimension": {"description": {"identifier": "minecraft:overworld"}, "components": {}}}"#;

fn overworld() -> DimensionId {
    DimensionId::new("minecraft:overworld").unwrap()
}

#[test]
fn dimension_two_packs_a_refused_lowest_file_leaves_the_higher_dimension_effective() {
    let stack = stack(
        DIMENSION_PATH,
        &[("top", DIMENSION), ("low", DIMENSION_REFUSED)],
    );
    let dimensions = stack.dimensions();
    assert_eq!(effective(dimensions, &overworld()), Some("top"));
    assert_eq!(rejected_packs(dimensions.rejected()), ["low"]);
    assert_eq!(
        errors_at(dimensions.diagnostics(), "low", DIMENSION_PATH),
        ["MissingRequiredField"]
    );
}

#[test]
fn dimension_three_packs_a_refused_lowest_file_leaves_the_middle_dimension_effective() {
    let stack = stack(
        DIMENSION_PATH,
        &[
            ("top", DIMENSION),
            ("middle", DIMENSION),
            ("low", DIMENSION_REFUSED),
        ],
    );
    let dimensions = stack.dimensions();
    assert_eq!(effective(dimensions, &overworld()), Some("middle"));
    assert_eq!(rejected_packs(dimensions.rejected()), ["low"]);
    assert_eq!(
        errors_at(dimensions.diagnostics(), "low", DIMENSION_PATH),
        ["MissingRequiredField"]
    );
}

// ---- POI types ------------------------------------------------------------------------------
//
// A failing file is skipped and the walk continues. Across packs the last load wins per
// identifier, with the stack walked lowest priority first.
// Present behaviour: fallback. The refused top type registers nothing; the highest valid type
// below it is effective.

const POI_PATH: &str = "poi/blocks/forge.json";
const POI: &str = r#"{"format_version": "1.26.40", "minecraft:poi_block": {"description": {"identifier": "test:forge"}, "blocks": ["minecraft:anvil"]}}"#;
/// No `blocks`: the document names `test:forge` and is refused.
const POI_REFUSED: &str = r#"{"format_version": "1.26.40", "minecraft:poi_block": {"description": {"identifier": "test:forge"}}}"#;

/// The stack with the `poi` experiment on (without it no POI file is read).
fn poi_stack(texts: &[(&str, &str)]) -> PackStack<Behavior> {
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::Poi);
    let options = LoadOptions {
        experiments,
        ..LoadOptions::default()
    };
    stack_with(POI_PATH, texts, options)
}

fn forge() -> PoiTypeId {
    PoiTypeId::parse("test:forge")
}

#[test]
fn poi_two_packs_a_refused_top_type_leaves_the_lower_type_effective() {
    let stack = poi_stack(&[("top", POI_REFUSED), ("low", POI)]);
    let poi = stack.poi();
    assert_eq!(effective(&poi.types, &forge()), Some("low"));
    assert_eq!(rejected_packs(poi.types.rejected()), ["top"]);
    assert_eq!(
        errors_at(poi.diagnostics(), "top", POI_PATH),
        // The cause, then the loader's closing line "Failed to load POI Type".
        ["MissingRequiredField", "DocumentRejected"]
    );
}

#[test]
fn poi_three_packs_a_refused_top_type_leaves_the_middle_type_effective() {
    let stack = poi_stack(&[("top", POI_REFUSED), ("middle", POI), ("low", POI)]);
    let poi = stack.poi();
    assert_eq!(effective(&poi.types, &forge()), Some("middle"));
    assert_eq!(rejected_packs(poi.types.rejected()), ["top"]);
    assert_eq!(
        errors_at(poi.diagnostics(), "top", POI_PATH),
        // The cause, then the loader's closing line "Failed to load POI Type".
        ["MissingRequiredField", "DocumentRejected"]
    );
}

// ---- dialogue scenes ------------------------------------------------------------------------
//
// A tag that is already stored is rejected with a message, so across packs the first visited,
// i.e. the lowest-priority pack, keeps the scene. Scene errors skip that scene.
// Present behaviour: fallback. A skipped scene of the lowest pack stores nothing, so the tag
// goes to the next pack above it that has a valid scene; a valid scene above that one gets
// the "already exists" error (`DuplicateIdentifier`).

const DIALOGUE_PATH: &str = "dialogue/a.json";
const DIALOGUE: &str = r#"{"format_version": "1.14.0", "minecraft:npc_dialogue": {"scenes": [{"scene_tag": "a", "text": "x"}]}}"#;
/// No `text`: the scene names the tag `a` and is skipped.
const DIALOGUE_REFUSED: &str =
    r#"{"format_version": "1.14.0", "minecraft:npc_dialogue": {"scenes": [{"scene_tag": "a"}]}}"#;

fn scene_a() -> DialogueSceneTag {
    DialogueSceneTag::parse("a")
}

#[test]
fn dialogue_two_packs_a_skipped_lowest_scene_leaves_the_higher_scene_effective() {
    let stack = stack(
        DIALOGUE_PATH,
        &[("top", DIALOGUE), ("low", DIALOGUE_REFUSED)],
    );
    let scenes = stack.dialogue();
    assert_eq!(effective(scenes, &scene_a()), Some("top"));
    assert_eq!(rejected_packs(scenes.rejected()), ["low"]);
    assert_eq!(
        errors_at(scenes.diagnostics(), "low", DIALOGUE_PATH),
        ["MissingRequiredField"]
    );
}

#[test]
fn dialogue_three_packs_a_skipped_lowest_scene_leaves_the_middle_scene_effective() {
    let stack = stack(
        DIALOGUE_PATH,
        &[
            ("top", DIALOGUE),
            ("middle", DIALOGUE),
            ("low", DIALOGUE_REFUSED),
        ],
    );
    let scenes = stack.dialogue();
    assert_eq!(effective(scenes, &scene_a()), Some("middle"));
    assert_eq!(rejected_packs(scenes.rejected()), ["low"]);
    assert_eq!(
        errors_at(scenes.diagnostics(), "low", DIALOGUE_PATH),
        ["MissingRequiredField"]
    );
    // The top pack's scene is told that the middle one keeps the tag.
    assert_eq!(
        errors_at(scenes.diagnostics(), "top", DIALOGUE_PATH),
        ["DuplicateIdentifier"]
    );
}

// ---- functions ------------------------------------------------------------------------------
//
// Functions combine whole-file, keyed by function name; the highest-priority pack wins and
// lower copies are ignored without a message. A bad line: with command version < 12 the line is
// dropped and the function loads; with 12 or more the whole function is skipped. Nothing is
// known about whether a skipped function still holds its name against the lower packs.
// Present behaviour: fallback. The refused top function does not reserve the name; the highest
// valid function below it is effective.

const FUNCTION_PATH: &str = "functions/x.mcfunction";
const FUNCTION: &str = "say hi\n";
/// No such command: the file names the function `x` and is refused.
const FUNCTION_REFUSED: &str = "notacommand hi\n";

fn function_x() -> FunctionPath {
    FunctionPath::parse("x")
}

#[test]
fn function_two_packs_a_refused_top_function_leaves_the_lower_function_effective() {
    let stack = stack(
        FUNCTION_PATH,
        &[("top", FUNCTION_REFUSED), ("low", FUNCTION)],
    );
    let functions = stack.functions();
    assert_eq!(effective(functions, &function_x()), Some("low"));
    assert_eq!(rejected_packs(functions.rejected()), ["top"]);
    assert_eq!(
        errors_at(functions.diagnostics(), "top", FUNCTION_PATH),
        ["DocumentRejected", "UnresolvedReference"]
    );
}

#[test]
fn function_three_packs_a_refused_top_function_leaves_the_middle_function_effective() {
    let stack = stack(
        FUNCTION_PATH,
        &[
            ("top", FUNCTION_REFUSED),
            ("middle", FUNCTION),
            ("low", FUNCTION),
        ],
    );
    let functions = stack.functions();
    assert_eq!(effective(functions, &function_x()), Some("middle"));
    assert_eq!(rejected_packs(functions.rejected()), ["top"]);
    assert_eq!(
        errors_at(functions.diagnostics(), "top", FUNCTION_PATH),
        ["DocumentRejected", "UnresolvedReference"]
    );
}

// ---- trade tables ---------------------------------------------------------------------------
//
// Nothing is known of how the packs' copies of a path combine, nor of what a failed copy does
// to a lower one.
// Present behaviour: taken. The top pack's copy of the path is the only one looked at; when
// it is refused the path has no table, whatever the lower packs hold.

const TRADE_PATH: &str = "trading/farmer_trades.json";
const TRADE: &str = r#"{"format_version": "1.21.30", "tiers": [{"total_exp_required": 0, "groups": [{"trades": [{"wants": [{"item": "minecraft:carrot"}], "gives": [{"item": "minecraft:emerald"}]}]}]}]}"#;
/// A tier without a trade group: the copy is refused.
const TRADE_REFUSED: &str = r#"{"format_version": "1.21.30", "tiers": [{"groups": []}]}"#;

/// The name of the pack whose copy of the trade table is effective.
fn effective_table(stack: &PackStack<Behavior>) -> Option<&str> {
    stack
        .trade_tables()
        .get(&PackPath::new(TRADE_PATH).unwrap())
        .map(|d| d.origin().pack.name())
}

#[test]
fn trading_two_packs_a_refused_top_copy_leaves_the_path_without_a_table() {
    let stack = stack(TRADE_PATH, &[("top", TRADE_REFUSED), ("low", TRADE)]);
    let tables = stack.trade_tables();
    assert_eq!(effective_table(&stack), None);
    assert_eq!(rejected_packs(tables.rejected()), ["top"]);
    assert_eq!(
        errors_at(tables.diagnostics(), "top", TRADE_PATH),
        ["MissingRequiredField"]
    );
}

#[test]
fn trading_three_packs_a_refused_top_copy_leaves_the_path_without_a_table() {
    let stack = stack(
        TRADE_PATH,
        &[("top", TRADE_REFUSED), ("middle", TRADE), ("low", TRADE)],
    );
    let tables = stack.trade_tables();
    assert_eq!(effective_table(&stack), None);
    assert_eq!(rejected_packs(tables.rejected()), ["top"]);
    assert_eq!(
        errors_at(tables.diagnostics(), "top", TRADE_PATH),
        ["MissingRequiredField"]
    );
}

// ---- voxel shapes ---------------------------------------------------------------------------
//
// A parse failure skips the file and nothing is registered; the walk continues with the next
// file. Per identifier, the last load wins.
// Present behaviour: fallback. The refused top shape binds nothing; the highest valid shape
// below it is effective.

const SHAPE_PATH: &str = "shapes/shape.json";
const SHAPE: &str = r#"{"format_version": "1.21.110", "minecraft:voxel_shape": {"description": {"identifier": "a:shape"}, "shape": {"boxes": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}}}"#;
/// No `shape`: the document names `a:shape` and is refused.
const SHAPE_REFUSED: &str = r#"{"format_version": "1.21.110", "minecraft:voxel_shape": {"description": {"identifier": "a:shape"}}}"#;

fn a_shape() -> VoxelShapeId {
    VoxelShapeId::new("a:shape").unwrap()
}

#[test]
fn voxel_shape_two_packs_a_refused_top_shape_leaves_the_lower_shape_effective() {
    let stack = stack(SHAPE_PATH, &[("top", SHAPE_REFUSED), ("low", SHAPE)]);
    let shapes = stack.voxel_shapes();
    assert_eq!(effective(shapes, &a_shape()), Some("low"));
    assert_eq!(rejected_packs(shapes.rejected()), ["top"]);
    assert_eq!(
        errors_at(shapes.diagnostics(), "top", SHAPE_PATH),
        ["MissingRequiredField", "DocumentRejected"]
    );
}

#[test]
fn voxel_shape_three_packs_a_refused_top_shape_leaves_the_middle_shape_effective() {
    let stack = stack(
        SHAPE_PATH,
        &[("top", SHAPE_REFUSED), ("middle", SHAPE), ("low", SHAPE)],
    );
    let shapes = stack.voxel_shapes();
    assert_eq!(effective(shapes, &a_shape()), Some("middle"));
    assert_eq!(rejected_packs(shapes.rejected()), ["top"]);
    assert_eq!(
        errors_at(shapes.diagnostics(), "top", SHAPE_PATH),
        ["MissingRequiredField", "DocumentRejected"]
    );
}

// ---- aim-assist presets ---------------------------------------------------------------------
//
// A preset file carries exactly one preset; any failure skips that preset only, and the failed
// file registers nothing. A later load replaces an earlier one with the same identifier.
// Present behaviour: fallback. The refused top preset registers nothing; the highest valid
// preset below it is effective.

const AIM_PATH: &str = "aim_assist/presets/p.json";
const AIM: &str =
    r#"{"format_version": "1.21.120", "minecraft:aim_assist_preset": {"identifier": "test:p"}}"#;
/// An unknown member: the document names `test:p` and is refused.
const AIM_REFUSED: &str = r#"{"format_version": "1.21.120", "minecraft:aim_assist_preset": {"identifier": "test:p", "foo": 1}}"#;

fn preset_p() -> AimAssistPresetId {
    AimAssistPresetId::parse("test:p")
}

#[test]
fn aim_assist_two_packs_a_refused_top_preset_leaves_the_lower_preset_effective() {
    let stack = stack(AIM_PATH, &[("top", AIM_REFUSED), ("low", AIM)]);
    let aim = stack.aim_assist();
    assert_eq!(effective(&aim.presets, &preset_p()), Some("low"));
    assert_eq!(rejected_packs(aim.presets.rejected()), ["top"]);
    assert_eq!(
        errors_at(aim.diagnostics(), "top", AIM_PATH),
        ["UnrecognizedField"]
    );
}

#[test]
fn aim_assist_three_packs_a_refused_top_preset_leaves_the_middle_preset_effective() {
    let stack = stack(
        AIM_PATH,
        &[("top", AIM_REFUSED), ("middle", AIM), ("low", AIM)],
    );
    let aim = stack.aim_assist();
    assert_eq!(effective(&aim.presets, &preset_p()), Some("middle"));
    assert_eq!(rejected_packs(aim.presets.rejected()), ["top"]);
    assert_eq!(
        errors_at(aim.diagnostics(), "top", AIM_PATH),
        ["UnrecognizedField"]
    );
}

// ---- behaviour-pack animations --------------------------------------------------------------
//
// Every failure skips the whole file (invalid JSON, a version rejection, a failed upgrade, any
// schema error including an unknown member); nothing loads with defaults, and an earlier
// pack's entry with the same identifier survives.
// Present behaviour: fallback. The highest valid animation below the refused file is
// effective.

const ANIMATION_PATH: &str = "animations/x.json";
const ANIMATION: &str =
    r#"{"format_version": "1.8.0", "animations": {"animation.x": {"animation_length": 1.0}}}"#;
/// An unknown member of the animation: the whole file is refused.
const ANIMATION_REFUSED: &str = r#"{"format_version": "1.8.0", "animations": {"animation.x": {"animation_length": 1.0, "foo": 1}}}"#;

fn animation_x() -> AnimationRef {
    AnimationRef::parse("animation.x")
}

#[test]
fn animation_two_packs_a_refused_top_file_leaves_the_lower_animation_effective() {
    let stack = stack(
        ANIMATION_PATH,
        &[("top", ANIMATION_REFUSED), ("low", ANIMATION)],
    );
    let animations = stack.animations();
    assert_eq!(effective(animations, &animation_x()), Some("low"));
    assert_eq!(rejected_packs(animations.rejected()), ["top"]);
    assert_eq!(
        errors_at(animations.diagnostics(), "top", ANIMATION_PATH),
        ["UnrecognizedField"]
    );
}

#[test]
fn animation_three_packs_a_refused_top_file_leaves_the_middle_animation_effective() {
    let stack = stack(
        ANIMATION_PATH,
        &[
            ("top", ANIMATION_REFUSED),
            ("middle", ANIMATION),
            ("low", ANIMATION),
        ],
    );
    let animations = stack.animations();
    assert_eq!(effective(animations, &animation_x()), Some("middle"));
    assert_eq!(rejected_packs(animations.rejected()), ["top"]);
    assert_eq!(
        errors_at(animations.diagnostics(), "top", ANIMATION_PATH),
        ["UnrecognizedField"]
    );
}

// ---- behaviour-pack animation controllers ---------------------------------------------------
//
// As for animations (both loaders): every failure skips the whole file, and an earlier pack's
// entry with the same identifier survives.
// Present behaviour: fallback. The highest valid controller below the refused file is
// effective.

const CONTROLLER_PATH: &str = "animation_controllers/x.json";
const CONTROLLER: &str = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.x": {"states": {"default": {}}}}}"#;
/// An unknown member of the controller: the whole file is refused.
const CONTROLLER_REFUSED: &str = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.x": {"states": {"default": {}}, "foo": 1}}}"#;

fn controller_x() -> AnimationRef {
    AnimationRef::parse("controller.animation.x")
}

#[test]
fn animation_controller_two_packs_a_refused_top_file_leaves_the_lower_controller_effective() {
    let stack = stack(
        CONTROLLER_PATH,
        &[("top", CONTROLLER_REFUSED), ("low", CONTROLLER)],
    );
    let controllers = stack.animation_controllers();
    assert_eq!(effective(controllers, &controller_x()), Some("low"));
    assert_eq!(rejected_packs(controllers.rejected()), ["top"]);
    assert_eq!(
        errors_at(controllers.diagnostics(), "top", CONTROLLER_PATH),
        ["UnrecognizedField"]
    );
}

#[test]
fn animation_controller_three_packs_a_refused_top_file_leaves_the_middle_controller_effective() {
    let stack = stack(
        CONTROLLER_PATH,
        &[
            ("top", CONTROLLER_REFUSED),
            ("middle", CONTROLLER),
            ("low", CONTROLLER),
        ],
    );
    let controllers = stack.animation_controllers();
    assert_eq!(effective(controllers, &controller_x()), Some("middle"));
    assert_eq!(rejected_packs(controllers.rejected()), ["top"]);
    assert_eq!(
        errors_at(controllers.diagnostics(), "top", CONTROLLER_PATH),
        ["UnrecognizedField"]
    );
}

// ---- behavior trees -------------------------------------------------------------------------
//
// The tree's name is assigned into a map of name to path and pack, overwriting the path and
// pack unconditionally; for every map entry the file is read again and loaded as a tree. A
// `minecraft:root` that is missing, not an object, or not exactly one member leaves the tree
// not registered, with no message; a JSON syntax error skips the file.
// Present behaviour, two kinds. A top file that names its tree and then fails: taken (the
// name has no tree, whatever the lower packs hold). A top file that is refused before it
// names a tree (not JSON): fallback (the highest valid tree below it is effective).

const TREE_PATH: &str = "behavior_trees/x.json";
const TREE: &str = r#"{"minecraft:behavior_tree": {"minecraft:name": "x", "minecraft:root": {"minecraft:wait": {}}}}"#;
/// A root of two nodes: the file names the tree `x` and is refused.
const TREE_REFUSED: &str = r#"{"minecraft:behavior_tree": {"minecraft:name": "x", "minecraft:root": {"minecraft:wait": {}, "minecraft:attack": {}}}}"#;
/// Not JSON: the file is refused before it names a tree.
const TREE_UNREADABLE: &str = r#"{"minecraft:behavior_tree": {"minecraft:name": "x""#;

fn tree_x() -> BehaviorTreeId {
    BehaviorTreeId::parse("x")
}

#[test]
fn behavior_tree_two_packs_a_refused_top_tree_takes_the_name_from_the_lower_tree() {
    let stack = stack(TREE_PATH, &[("top", TREE_REFUSED), ("low", TREE)]);
    let trees = stack.behavior_trees();
    assert_eq!(effective(trees, &tree_x()), None);
    assert_eq!(rejected_packs(trees.rejected()), ["top"]);
    assert_eq!(
        errors_at(trees.diagnostics(), "top", TREE_PATH),
        ["InvalidRoot"]
    );
}

#[test]
fn behavior_tree_three_packs_a_refused_top_tree_takes_the_name_from_both_lower_trees() {
    let stack = stack(
        TREE_PATH,
        &[("top", TREE_REFUSED), ("middle", TREE), ("low", TREE)],
    );
    let trees = stack.behavior_trees();
    assert_eq!(effective(trees, &tree_x()), None);
    assert_eq!(rejected_packs(trees.rejected()), ["top"]);
    assert_eq!(
        errors_at(trees.diagnostics(), "top", TREE_PATH),
        ["InvalidRoot"]
    );
}

#[test]
fn behavior_tree_two_packs_an_unreadable_top_file_leaves_the_lower_tree_effective() {
    let stack = stack(TREE_PATH, &[("top", TREE_UNREADABLE), ("low", TREE)]);
    let trees = stack.behavior_trees();
    assert_eq!(effective(trees, &tree_x()), Some("low"));
    assert_eq!(rejected_packs(trees.rejected()), ["top"]);
    assert_eq!(
        errors_at(trees.diagnostics(), "top", TREE_PATH),
        ["MalformedInput"]
    );
}

#[test]
fn behavior_tree_three_packs_an_unreadable_top_file_leaves_the_middle_tree_effective() {
    let stack = stack(
        TREE_PATH,
        &[("top", TREE_UNREADABLE), ("middle", TREE), ("low", TREE)],
    );
    let trees = stack.behavior_trees();
    assert_eq!(effective(trees, &tree_x()), Some("middle"));
    assert_eq!(rejected_packs(trees.rejected()), ["top"]);
    assert_eq!(
        errors_at(trees.diagnostics(), "top", TREE_PATH),
        ["MalformedInput"]
    );
}

// ---- camera presets -------------------------------------------------------------------------
//
// Per identifier, the last loaded wins: the later document replaces the earlier one whole. An
// invalid preset is discarded by a pass that runs once, after the presets of every pack are
// loaded; a file that fails to load stores no preset.
// Present behaviour, two kinds. A top file refused when it is read stores nothing: fallback
// (the highest valid preset below it is effective). A top preset that loads and is discarded
// by the pass over the merged presets had already replaced the lower ones: taken (the
// identifier has no preset).

const CAMERA_PATH: &str = "cameras/presets/same.json";
const CAMERA: &str = r#"{"format_version": "1.21.60", "minecraft:camera_preset": {"identifier": "ex:same", "inherit_from": "minecraft:free"}}"#;
/// An unknown member: the file names `ex:same` and is refused when it is read.
const CAMERA_REFUSED_FILE: &str = r#"{"format_version": "1.21.60", "minecraft:camera_preset": {"identifier": "ex:same", "inherit_from": "minecraft:free", "unexpected_field": 1.5}}"#;
/// An unknown parent: the file loads, and the pass over the merged presets discards it.
const CAMERA_DISCARDED: &str = r#"{"format_version": "1.21.60", "minecraft:camera_preset": {"identifier": "ex:same", "inherit_from": "ex:missing"}}"#;

fn same() -> CameraPresetId {
    CameraPresetId::parse("ex:same")
}

#[test]
fn camera_two_packs_a_refused_top_file_leaves_the_lower_preset_effective() {
    let stack = stack(
        CAMERA_PATH,
        &[("top", CAMERA_REFUSED_FILE), ("low", CAMERA)],
    );
    let cameras = stack.cameras();
    assert_eq!(effective(&cameras.presets, &same()), Some("low"));
    assert_eq!(rejected_packs(cameras.presets.rejected()), ["top"]);
    assert_eq!(
        errors_at(cameras.diagnostics(), "top", CAMERA_PATH),
        ["UnrecognizedField"]
    );
}

#[test]
fn camera_three_packs_a_refused_top_file_leaves_the_middle_preset_effective() {
    let stack = stack(
        CAMERA_PATH,
        &[
            ("top", CAMERA_REFUSED_FILE),
            ("middle", CAMERA),
            ("low", CAMERA),
        ],
    );
    let cameras = stack.cameras();
    assert_eq!(effective(&cameras.presets, &same()), Some("middle"));
    assert_eq!(rejected_packs(cameras.presets.rejected()), ["top"]);
    assert_eq!(
        errors_at(cameras.diagnostics(), "top", CAMERA_PATH),
        ["UnrecognizedField"]
    );
}

#[test]
fn camera_two_packs_a_discarded_top_preset_takes_the_identifier_from_the_lower_preset() {
    let stack = stack(CAMERA_PATH, &[("top", CAMERA_DISCARDED), ("low", CAMERA)]);
    let cameras = stack.cameras();
    assert_eq!(effective(&cameras.presets, &same()), None);
    assert_eq!(rejected_packs(cameras.presets.rejected()), ["top"]);
    // The camera loader's line and the linked-asset validator's line beside it.
    assert_eq!(
        errors_at(cameras.diagnostics(), "top", CAMERA_PATH),
        ["UnresolvedReference", "UnresolvedReference"]
    );
}

#[test]
fn camera_three_packs_a_discarded_top_preset_takes_the_identifier_from_both_lower_presets() {
    let stack = stack(
        CAMERA_PATH,
        &[
            ("top", CAMERA_DISCARDED),
            ("middle", CAMERA),
            ("low", CAMERA),
        ],
    );
    let cameras = stack.cameras();
    assert_eq!(effective(&cameras.presets, &same()), None);
    assert_eq!(rejected_packs(cameras.presets.rejected()), ["top"]);
    // The camera loader's line and the linked-asset validator's line beside it.
    assert_eq!(
        errors_at(cameras.diagnostics(), "top", CAMERA_PATH),
        ["UnresolvedReference", "UnresolvedReference"]
    );
}
