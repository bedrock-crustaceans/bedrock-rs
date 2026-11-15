//! The unknown members of the four jigsaw documents that the load keeps for the write-back, at
//! every depth the documents have (the structs, the enums' variants, a weighted list's elements):
//! each document has a member no struct binds at each level, and the table is what is kept.

use bedrock_addon_core::document::{Content, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_jigsaw::{JigsawStructure, ProcessorList, StructureSet, TemplatePool};

/// The members the load of `text` as the `T` at `path` keeps, one `at/key` line each.
fn kept<T: Content>(path: &str, text: &str) -> Vec<String> {
    let read: Result<_, Rejected> = T::parse(
        &PackPath::new(path).expect("a path"),
        text.as_bytes(),
        &LoadContext::detached(),
    );
    let document = read.unwrap_or_else(|rejected| {
        panic!(
            "the document is refused: {:?}",
            rejected.diagnostics.iter().collect::<Vec<_>>()
        )
    });
    document
        .unknown()
        .iter()
        .map(|(at, key, _)| format!("{at}/{key}"))
        .collect()
}

#[test]
fn a_structure_keeps_its_unknown_members_at_every_depth() {
    let found = kept::<JigsawStructure>(
        "worldgen/structures/a.json",
        r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"zz": 1, "description": {"identifier": "demo:a", "zz": 2}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": 0}, "zz": 3}, "dimension_padding": {"top": 1, "zz": 4}, "max_distance_from_center": {"horizontal": 10, "vertical": 10, "zz": 5}, "pool_aliases": [{"alias": "x", "targets": [{"weight": 1, "data": "demo:p", "zz": 6}], "zz": 7}]}}"#,
    );
    assert_eq!(
        found,
        [
            "minecraft:jigsaw/zz",
            "minecraft:jigsaw/description/zz",
            "minecraft:jigsaw/dimension_padding/zz",
            "minecraft:jigsaw/max_distance_from_center/zz",
            "minecraft:jigsaw/pool_aliases/0/zz",
            "minecraft:jigsaw/pool_aliases/0/targets/0/zz",
            "minecraft:jigsaw/start_height/zz",
        ]
    );
}

#[test]
fn a_template_pool_keeps_its_unknown_members_at_every_depth() {
    let found = kept::<TemplatePool>(
        "worldgen/template_pools/a.json",
        r#"{"format_version": "1.21.20", "minecraft:template_pool": {"zz": 1, "description": {"identifier": "demo:p", "zz": 2}, "elements": [{"weight": 1, "zz": 3, "element": {"element_type": "minecraft:empty_pool_element", "zz": 4}}]}}"#,
    );
    assert_eq!(
        found,
        [
            "minecraft:template_pool/zz",
            "minecraft:template_pool/description/zz",
            "minecraft:template_pool/elements/0/zz",
            "minecraft:template_pool/elements/0/element/zz",
        ]
    );
}

#[test]
fn a_structure_set_keeps_its_unknown_members_at_every_depth() {
    let found = kept::<StructureSet>(
        "worldgen/structure_sets/a.json",
        r#"{"format_version": "1.21.20", "minecraft:structure_set": {"zz": 1, "description": {"identifier": "demo:s", "zz": 2}, "placement": {"type": "minecraft:random_spread", "salt": 1, "separation": 1, "spacing": 4, "zz": 3}, "structures": [{"structure": "demo:a", "weight": 1, "zz": 4}]}}"#,
    );
    assert_eq!(
        found,
        [
            "minecraft:structure_set/zz",
            "minecraft:structure_set/description/zz",
            "minecraft:structure_set/placement/zz",
            "minecraft:structure_set/structures/0/zz",
        ]
    );
}

#[test]
fn a_processor_list_keeps_its_unknown_members_at_every_depth() {
    let found = kept::<ProcessorList>(
        "worldgen/processors/a.json",
        r#"{"format_version": "1.21.20", "minecraft:processor_list": {"zz": 1, "description": {"identifier": "demo:l", "zz": 2}, "processors": [{"processor_type": "minecraft:protected_blocks", "value": "demo:tag", "zz": 3}, {"processor_type": "minecraft:rule", "rules": [{"zz": 4, "input_predicate": {"predicate_type": "minecraft:always_true", "zz": 5}, "location_predicate": {"predicate_type": "minecraft:always_true"}, "output_state": "demo:b"}]}]}}"#,
    );
    assert_eq!(
        found,
        [
            "minecraft:processor_list/zz",
            "minecraft:processor_list/description/zz",
            "minecraft:processor_list/processors/0/zz",
            "minecraft:processor_list/processors/1/rules/0/zz",
            "minecraft:processor_list/processors/1/rules/0/input_predicate/zz",
        ]
    );
}

#[test]
fn a_metadata_registry_keeps_the_unknown_members_of_its_entries() {
    use bedrock_addon_core::json::JsonSlice;
    use bedrock_addon_core::versioning::load;
    use bedrock_addon_family_jigsaw::history::MetadataFamily;
    for version in ["1.21.50", "1.21.80"] {
        let text = format!(
            r#"{{"format_version": "{version}", "minecraft:jigsaw_structure_metadata": {{"metadata": {{"42": {{"size": [1, 2, 3], "demo_entry": 1, "markers": [{{"name": "a", "pool": "p", "target": "t", "block": "b", "final_block": "f", "joint_type": "aligned", "pos": [0, 1, 2], "selection_priority": 0, "placement_priority": 0, "demo_marker": true}}]}}}}}}}}"#
        );
        let outcome = load::<MetadataFamily>(JsonSlice::new(&text), &LoadContext::detached());
        let kept: Vec<String> = outcome
            .diagnostics
            .unknown()
            .iter()
            .map(|(at, key, _)| format!("{at}/{key}"))
            .collect();
        assert_eq!(
            kept,
            [
                "minecraft:jigsaw_structure_metadata/metadata/42/demo_entry",
                "minecraft:jigsaw_structure_metadata/metadata/42/markers/0/demo_marker",
            ],
            "{version}"
        );
    }
}
