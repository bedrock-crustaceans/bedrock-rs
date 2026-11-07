//! The rows of 20 that the link pass does not check: a reference only the game can resolve
//! when it runs, places or generates (`run`), and one nothing checks (`—`). Each test builds
//! the reference with a target nothing defines and requires that neither the load nor the
//! link pass tells a reference finding for it.
//!
//! The rows the load checks (C1, C2, M8, M10 and the first word of a function line in M11)
//! are pinned in their families' tests: `camera/stack.rs`
//! (`cam07_invalid_inheritance_discards_the_preset`,
//! `cam09_a_list_must_name_known_presets_once_each`), `poi/stack.rs`
//! (`poi32_unknown_tag_reference_is_dropped_with_a_warning`,
//! `a_parent_naming_no_tag_is_dropped_with_a_warning`,
//! `poi33_type_tag_without_document_is_a_leaf_tag`,
//! `poi34_tag_cycle_keeps_the_tags_and_ignores_the_cycle`), `function/tick.rs`
//! (`f18_an_entry_naming_no_function_is_skipped_with_a_warning`) and `function/lines.rs`
//! (`f10_…`, `f11_…`).

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Kind};
#[allow(unused_imports, reason = "which rows compile depends on the features")]
use bedrock_addon::load::LoadOptions;
#[allow(unused_imports, reason = "which rows compile depends on the features")]
use bedrock_addon::pack::StackSet;

#[allow(unused_imports, reason = "which rows compile depends on the features")]
use super::super::common::stacks;

/// Whether `finding` is a reference finding: one of the codes of a miss, or any finding that
/// carries a reference.
fn is_reference(finding: &Diagnostic) -> bool {
    matches!(
        finding.code.kind(),
        Kind::UnresolvedReference | Kind::InvalidReference | Kind::Cycle
    ) || matches!(
        finding.detail.as_deref(),
        Some(DiagnosticDetail::Reference { .. })
    )
}

/// Requires that the load findings `loaded` and the link findings `linked` hold no reference
/// finding.
#[track_caller]
fn tells_no_reference<'a>(loaded: impl Iterator<Item = &'a Diagnostic>, linked: &Diagnostics) {
    let loaded: Vec<_> = loaded.filter(|finding| is_reference(finding)).collect();
    assert!(loaded.is_empty(), "the load: {loaded:#?}");
    let linked: Vec<_> = linked
        .iter()
        .filter(|finding| is_reference(finding))
        .collect();
    assert!(linked.is_empty(), "the link pass: {linked:#?}");
}

// ---- world generation --------------------------------------------------------------------------

/// W2: the game looks the feature up when it places the filter.
#[cfg(feature = "feature")]
#[test]
fn w2_the_feature_of_a_height_difference_filter_stays_run() {
    let filter = super::feature::feature(
        "1.26.30",
        "height_difference_filter_feature",
        "demo:filter",
        r#", "places_feature": "demo:gone", "search_radius": 2"#,
    );
    let set = stacks(
        &[("features/filter.json", &filter)],
        &[],
        LoadOptions::default(),
    );
    let features = set.behavior.features();
    assert_eq!(features.len(), 1, "{:#?}", features.rejected());
    tells_no_reference(features.diagnostics(), &set.link());
}

/// W7: a tag a descriptor's Molang names, and what a condition asks, are arguments of a
/// query: the game resolves them when the expression runs.
#[cfg(feature = "feature")]
#[test]
fn w7_the_tags_of_a_descriptor_and_a_condition_stay_run() {
    let ore = super::feature::feature(
        "1.13.0",
        "ore_feature",
        "demo:ore",
        r#", "count": 8, "replace_rules": [{"places_block": "minecraft:stone", "may_replace": [{"tags": "query.any_tag('no_such_tag')"}]}]"#,
    );
    let carver = super::feature::feature(
        "1.13.0",
        "cave_carver_feature",
        "demo:carver",
        r#", "width_modifier": "query.noise(1.0, 2.0)""#,
    );
    let set = stacks(
        &[
            ("features/ore.json", &ore),
            ("features/carver.json", &carver),
        ],
        &[],
        LoadOptions::default(),
    );
    let features = set.behavior.features();
    assert_eq!(features.len(), 2, "{:#?}", features.rejected());
    tells_no_reference(features.diagnostics(), &set.link());
}

/// W11: a biome filter tests tags and values when the game decorates a chunk.
#[cfg(all(feature = "feature_rules", feature = "feature"))]
#[test]
fn w11_the_tag_of_a_biome_filter_is_not_checked() {
    let rule = r#"{"format_version": "1.13.0", "minecraft:feature_rules": {"description": {"identifier": "demo:r", "places_feature": "demo:vein"}, "conditions": {"placement_pass": "surface_pass", "minecraft:biome_filter": {"test": "has_biome_tag", "value": "no_such_tag"}}}}"#;
    let vein = super::feature::vein("demo:vein");
    let set = stacks(
        &[
            ("feature_rules/r.json", rule),
            ("features/vein.json", &vein),
        ],
        &[],
        LoadOptions::default(),
    );
    assert_eq!(set.behavior.feature_rules().len(), 1);
    tells_no_reference(set.behavior.feature_rules().diagnostics(), &set.link());
}

/// W23: the jigsaw block a structure starts at is in a template, read when the game
/// generates the structure.
#[cfg(feature = "jigsaw")]
#[test]
fn w23_the_start_jigsaw_name_of_a_structure_stays_run() {
    let pool = r#"{"format_version": "1.21.20", "minecraft:template_pool": {"description": {"identifier": "demo:start"}, "elements": []}}"#;
    let structure = r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:ruin"}, "step": "surface_structures", "start_pool": "demo:start", "start_jigsaw_name": "demo:no_such_anchor", "max_depth": 4, "start_height": {"type": "constant", "value": {"absolute": 0}}}}"#;
    let set = stacks(
        &[
            ("worldgen/template_pools/start.json", pool),
            ("worldgen/structures/ruin.json", structure),
        ],
        &[],
        LoadOptions::default(),
    );
    let jigsaw = set.behavior.jigsaw();
    assert_eq!((jigsaw.structures.len(), jigsaw.pools.len()), (1, 1));
    tells_no_reference(jigsaw.diagnostics(), &set.link());
}

/// W24: the tags of a processor are plain strings.
#[cfg(feature = "jigsaw")]
#[test]
fn w24_the_block_tags_of_a_processor_are_not_checked() {
    let processors = r#"{"format_version": "1.21.20", "minecraft:processor_list": {"description": {"identifier": "demo:weather"}, "processors": [{"processor_type": "minecraft:protected_blocks", "value": "no_such_tag"}, {"processor_type": "minecraft:rule", "rules": [{"input_predicate": {"predicate_type": "minecraft:tag_match", "tag": "no_such_tag"}, "output_state": {"name": "minecraft:stone"}}]}]}}"#;
    let set = stacks(
        &[("worldgen/processors/weather.json", processors)],
        &[],
        LoadOptions::default(),
    );
    let jigsaw = set.behavior.jigsaw();
    assert_eq!(
        jigsaw.processors.len(),
        1,
        "{:#?}",
        jigsaw.processors.rejected()
    );
    tells_no_reference(jigsaw.diagnostics(), &set.link());
}

/// W26: the game copies the entities of a structure as they are.
#[cfg(feature = "structure")]
#[test]
fn w26_the_entities_of_a_structure_are_not_checked() {
    use bedrock_addon::families::structure::Structure;
    use nbtx::{Compound, Value};

    use super::super::common::{behavior_pack_of, stacks_of};
    let mut structure = Structure::new([1, 1, 1]);
    structure.structure.entities = vec![Compound::from_iter([(
        "identifier".into(),
        Value::String("demo:no_such_entity".into()),
    )])];
    let bytes: &'static [u8] = Box::leak(
        structure
            .to_bytes()
            .expect("a structure encodes")
            .into_boxed_slice(),
    );
    let pack = behavior_pack_of(1, &[("structures/demo/hut.mcstructure", bytes)]);
    let set: StackSet = stacks_of(vec![pack], Vec::new(), LoadOptions::default());
    let structures = set.behavior.structures();
    assert_eq!(structures.iter().count(), 1);
    assert_eq!(
        structures
            .iter()
            .map(|(_, document)| document.value().structure.entities.len())
            .sum::<usize>(),
        1
    );
    tells_no_reference(structures.diagnostics(), &set.link());
}

// ---- the small behaviour-pack families -------------------------------------------------------

/// M6: the block and the entity a node looks for are names the game reads when the tree runs.
#[cfg(feature = "behavior_tree")]
#[test]
fn m6_the_block_and_the_entity_of_a_node_stay_run() {
    let tree = r#"{"minecraft:behavior_tree": {"minecraft:name": "seek", "minecraft:root": {"minecraft:sequence": {"children": [{"minecraft:find_block": {"block_name": "demo:no_such_block"}}, {"minecraft:find_entity": {"entity_name": "demo:no_such_entity"}}, {"minecraft:look_at_entity": {"entity_name": "demo:no_such_entity"}}]}}}}"#;
    let set = stacks(
        &[("behavior_trees/seek.json", tree)],
        &[],
        LoadOptions::default(),
    );
    let trees = set.behavior.behavior_trees();
    assert!(
        trees.iter().any(|(name, _)| name.as_str() == "seek"),
        "{:#?}",
        trees.rejected()
    );
    tells_no_reference(trees.diagnostics(), &set.link());
}

/// M9: the commands of a dialogue are compiled when the action runs, and nothing looks a
/// translation key up.
#[cfg(feature = "dialogue")]
#[test]
fn m9_the_commands_and_the_keys_of_a_dialogue_stay_run() {
    let dialogue = r#"{"format_version": "1.17.0", "minecraft:npc_dialogue": {"scenes": [{"scene_tag": "demo_scene", "npc_name": {"rawtext": [{"translate": "demo.no.such.key"}]}, "text": {"rawtext": [{"translate": "demo.no.such.key"}]}, "on_open_commands": ["/no_such_command"], "on_close_commands": ["/function demo/gone"], "buttons": [{"name": {"rawtext": [{"translate": "demo.no.such.key"}]}, "commands": ["/no_such_command now"]}]}]}}"#;
    let set = stacks(
        &[("dialogue/scene.json", dialogue)],
        &[],
        LoadOptions::default(),
    );
    let scenes = set.behavior.dialogue();
    assert_eq!(scenes.len(), 1, "{:#?}", scenes.rejected());
    tells_no_reference(scenes.diagnostics(), &set.link());
}

/// M11, the second half: the function a `function` line runs is looked up when the line runs.
#[cfg(feature = "function")]
#[test]
fn m11_the_function_a_function_line_runs_stays_run() {
    let set = stacks(
        &[("functions/start.mcfunction", "function demo/gone\n")],
        &[],
        LoadOptions::default(),
    );
    let functions = set.behavior.functions();
    assert_eq!(functions.len(), 1, "{:#?}", functions.rejected());
    tells_no_reference(functions.diagnostics(), &set.link());
}

// ---- cameras ---------------------------------------------------------------------------------

/// C6: block tags and entity families are plain strings.
#[cfg(feature = "aim_assist")]
#[test]
fn c6_the_block_tags_and_the_entity_families_of_aim_assist_are_not_checked() {
    let preset = r#"{"format_version": "1.21.120", "minecraft:aim_assist_preset": {"identifier": "demo:aim", "exclusion_settings": {"block_tags": ["no_such_tag"], "entity_type_families": ["no_such_family"]}}}"#;
    let categories = r#"{"format_version": "1.21.50", "minecraft:aim_assist_categories": {"categories": [{"name": "demo:targets", "priorities": {"block_tags": {"no_such_tag": 1}, "entity_type_families": {"no_such_family": 2}}}]}}"#;
    let set = stacks(
        &[
            ("aim_assist/presets/p.json", preset),
            ("aim_assist/categories/c.json", categories),
        ],
        &[],
        LoadOptions::default(),
    );
    let aim = set.behavior.aim_assist();
    assert_eq!((aim.presets.len(), aim.categories.len()), (1, 1));
    tells_no_reference(aim.diagnostics(), &set.link());
}
