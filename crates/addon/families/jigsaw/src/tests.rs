//! Unit tests of the crate-private parts: the identifier check's order, the metadata file's
//! leading string, the walk's file filter and the link checker.

use super::link::{Targets, link};
use super::load::load_group;
use super::validate::identifier_failure;
use super::{Jigsaw, JigsawMetadata};
use bedrock_addon_core::diagnostic::{Diagnostic, Severity};
use bedrock_addon_core::id::{BlockId, FeatureId, LootTablePath, StructureId};
use bedrock_addon_core::load::LoadOptions;
use bedrock_addon_core::load::{LoadContext, StackCx, WalkFile};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::{PackRef, PackSide};

#[test]
fn identifier_failures_come_in_the_games_check_order() {
    assert_eq!(identifier_failure("demo:ruin"), None);
    assert_eq!(
        identifier_failure("Demo:Ruin/Tower.v-2"),
        None,
        "case and punctuation pass"
    );
    assert_eq!(identifier_failure(""), Some(&[][..]), "no text of its own");
    assert_eq!(
        identifier_failure("invalid"),
        Some(&["Identifier must have a namespace"][..])
    );
    for invalid in ["a:b:c", ":b", "a:"] {
        assert_eq!(
            identifier_failure(invalid),
            Some(&["Invalid identifier"][..]),
            "{invalid}"
        );
    }
    assert_eq!(
        identifier_failure("de mo:ruin"),
        Some(
            &[
                "Identifier cannot have illegal characters",
                "Namespace is invalid"
            ][..]
        )
    );
    assert_eq!(
        identifier_failure("demo:ru\tin"),
        Some(
            &[
                "Identifier cannot have illegal characters",
                "Identifier name is invalid"
            ][..]
        )
    );
    assert_eq!(
        identifier_failure("minecraft:ruin"),
        None,
        "the minecraft namespace is allowed for every pack"
    );
}

#[test]
fn a_long_version_string_round_trips_through_a_two_byte_length() {
    let file = JigsawMetadata {
        format_version: "1".repeat(200),
        body: vec![9, 8],
    };
    let bytes = file.to_bytes();
    assert_eq!(&bytes[..2], &[0xc8, 0x01], "200 as LEB128");
    let parsed = super::load::parse_metadata(
        PackRef::app_data(),
        PackSide::Behavior,
        &PackPath::new("worldgen/metadata/a.dat").unwrap(),
        &bytes,
    );
    // A 200-character version is no version: the string itself was read whole.
    let rejected = parsed.expect_err("not a version");
    assert!(
        rejected
            .diagnostics
            .iter()
            .any(|d| d.code.name() == "MalformedFormatVersion"),
        "{:?}",
        rejected.diagnostics
    );
}

#[test]
fn the_walk_reads_json_below_the_four_directories_and_dat_directly_in_metadata() {
    let walk = load_group().walk;
    let reads = |path: &str| walk.matches(&PackPath::new(path).unwrap());
    assert_eq!(reads("worldgen/structures/a.json"), Some(0));
    assert_eq!(reads("worldgen/template_pools/x/y/a.json"), Some(1));
    assert_eq!(reads("worldgen/structure_sets/a.json"), Some(2));
    assert_eq!(reads("worldgen/processors/a.json"), Some(3));
    assert_eq!(reads("worldgen/metadata/a.dat"), Some(4));
    for ignored in [
        "worldgen/structures/a.JSON",
        "worldgen/structures/a.dat",
        "worldgen/metadata/a.json",
        "worldgen/metadata/a.DAT",
        "worldgen/metadata/sub/a.dat",
        "worldgen/features/a.json",
    ] {
        assert_eq!(reads(ignored), None, "{ignored}");
    }
}

/// The other families' contents, by name.
struct Known;

impl Targets for Known {
    fn block(&self, id: &BlockId) -> bool {
        id.as_str().starts_with("minecraft:")
    }
    fn feature(&self, id: &FeatureId) -> bool {
        id.as_str() == "minecraft:oak_tree_feature"
    }
    fn loot_table(&self, path: &LootTablePath) -> bool {
        path.as_str() == "loot_tables/chests/x.json"
    }
    fn structure(&self, location: &StructureId) -> bool {
        location.as_str() == "demo/house"
    }
}

/// The group of `files`, loaded through the group's own loader (walk filter, parse, combine)
/// as the app-data layer.
fn group(files: &[(&str, &str)]) -> Jigsaw {
    let loader = load_group();
    let cx = LoadContext::detached();
    let parsed = files
        .iter()
        .map(|(path, text)| {
            let path = PackPath::new(path).unwrap();
            assert!(loader.walk.matches(&path).is_some(), "{path:?}");
            let file = WalkFile {
                pack: PackRef::app_data(),
                side: PackSide::Behavior,
                path: path.clone(),
                key: path,
                bytes: text.as_bytes().to_vec().into(),
            };
            (loader.parse)(&file, &cx)
        })
        .collect();
    let options = LoadOptions::default();
    (loader.combine)(parsed, &StackCx::new(&options, &[]))
}

/// The link findings of a group, as `(json path, text)`.
fn linked(jigsaw: &Jigsaw) -> Vec<(String, String)> {
    let found = link(jigsaw, &Known);
    found
        .iter()
        .map(|d: &Diagnostic| {
            // The validator's lines are content-log errors. A block the processors name is the
            // game's own error-level line (a kept error); the structure and feature texts are
            // informations of the crate's own.
            let line = d.code.to_string().contains(":  Missing referenced asset ")
                || d.code.to_string().contains(":  Invalid asset path ")
                || d.code.to_string().starts_with("Cannot find block ");
            let expected = if line {
                Severity::Error
            } else {
                Severity::Info
            };
            assert_eq!(d.severity, expected, "{d:?}");
            assert_eq!(d.code.name(), "UnresolvedReference");
            assert_eq!(d.phase, bedrock_addon_core::diagnostic::Phase::Link);
            (d.location.json_path.to_string(), d.code.to_string())
        })
        .collect()
}

#[test]
fn pools_resolve_inside_the_group_and_minecraft_empty_is_built_in() {
    let s = group(&[
        (
            "worldgen/structures/a.json",
            r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "lakes", "start_pool": "demo:missing", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": 0}}, "pool_aliases": [{"groups": [{"weight": 1, "data": [{"alias": "x", "target": "demo:start"}, {"alias": "y", "targets": [{"weight": 1, "data": "demo:gone"}]}]}]}]}}"#,
        ),
        (
            "worldgen/template_pools/start.json",
            r#"{"format_version": "1.21.20", "minecraft:template_pool": {"description": {"identifier": "demo:start"}, "elements": [], "fallback": "minecraft:empty"}}"#,
        ),
        (
            "worldgen/template_pools/other.json",
            r#"{"format_version": "1.21.20", "minecraft:template_pool": {"description": {"identifier": "demo:other"}, "elements": [], "fallback": "demo:nowhere"}}"#,
        ),
    ]);
    assert_eq!(
        linked(&s),
        [
            (
                "minecraft:jigsaw/start_pool".to_owned(),
                "-> start_pool:  Missing referenced asset demo:missing".to_owned()
            ),
            (
                "minecraft:jigsaw/pool_aliases/0/groups/0/data/1/targets/0/data".to_owned(),
                "-> pool_aliases[0] -> groups[0] -> data[1] -> targets[0] -> data:  Missing referenced asset demo:gone".to_owned()
            ),
            (
                "minecraft:template_pool/fallback".to_owned(),
                "-> fallback:  Missing referenced asset demo:nowhere".to_owned()
            ),
        ]
    );
}

#[test]
fn pool_elements_link_structures_processor_lists_and_features() {
    let s = group(&[(
        "worldgen/template_pools/a.json",
        r#"{"format_version": "1.21.20", "minecraft:template_pool": {"description": {"identifier": "demo:pool"}, "elements": [
                {"element": {"element_type": "minecraft:single_pool_element", "location": "demo:missing", "processors": "demo:none"}},
                {"element": {"element_type": "minecraft:single_pool_element", "location": "demo/house", "processors": ""}},
                {"element": {"element_type": "minecraft:feature_pool_element", "feature": "minecraft:oak_tree_feature"}}
            ]}}"#,
    )]);
    assert_eq!(
        linked(&s),
        [
            (
                "minecraft:template_pool/elements/0/element/location".to_owned(),
                "[JIGSAW GENERATION FAILURE] Structure name [demo:missing] referenced in template pool [demo:pool] does not exist.".to_owned()
            ),
            // And the game's validator line of the file reference.
            (
                "minecraft:template_pool/elements/0/element/location".to_owned(),
                "-> elements[0] -> element -> location:  Invalid asset path demo:missing".to_owned()
            ),
            (
                "minecraft:template_pool/elements/0/element/processors".to_owned(),
                "-> elements[0] -> element -> processors:  Missing referenced asset demo:none".to_owned()
            ),
        ]
    );
}

#[test]
fn an_unknown_feature_gets_the_games_generation_text() {
    let s = group(&[(
        "worldgen/template_pools/a.json",
        r#"{"format_version": "1.21.20", "minecraft:template_pool": {"description": {"identifier": "demo:features"}, "elements": [{"element": {"element_type": "minecraft:feature_pool_element", "feature": "demo:missing"}}]}}"#,
    )]);
    assert_eq!(
        linked(&s),
        [(
            "minecraft:template_pool/elements/0/element/feature".to_owned(),
            "Cannot find 'feature' 'demo:missing' for 'feature_pool_element'. Make sure to include the 'namespace_name:' prefix.".to_owned()
        )]
    );
}

#[test]
fn a_set_entry_naming_no_structure_is_reported() {
    let s = group(&[(
        "worldgen/structure_sets/a.json",
        r#"{"format_version": "1.21.20", "minecraft:structure_set": {"description": {"identifier": "demo:set"}, "placement": {"type": "minecraft:random_spread", "salt": 1, "separation": 1, "spacing": 4}, "structures": [{"structure": "demo:missing", "weight": 1}]}}"#,
    )]);
    assert_eq!(
        linked(&s),
        [(
            "minecraft:structure_set/structures/0/structure".to_owned(),
            "-> structures[0] -> structure:  Missing referenced asset demo:missing".to_owned()
        )]
    );
}

#[test]
fn processor_blocks_and_loot_tables_are_reported_once_per_member() {
    let s = group(&[(
        "worldgen/processors/a.json",
        r#"{"format_version": "1.21.20", "minecraft:processor_list": {"description": {"identifier": "demo:list"}, "processors": [
            {"processor_type": "minecraft:rule", "rules": [
                {"input_predicate": {"predicate_type": "minecraft:block_match", "block": "test:in"},
                 "location_predicate": {"predicate_type": "minecraft:blockstate_match", "block_state": {"name": "test:at"}},
                 "output_state": {"name": "test:invalid"},
                 "block_entity_modifier": {"type": "minecraft:append_loot", "loot_table": "loot_tables/none.json"}}
            ]},
            {"processor_type": "minecraft:capped", "limit": 1, "delegate": {"processor_type": "minecraft:block_ignore", "blocks": ["minecraft:air", "test:ignored"]}}
        ]}}"#,
    )]);
    assert_eq!(
        linked(&s),
        [
            // Each rule block with the game's validator line beside it (the
            // block of a state object at its `name`).
            (
                "minecraft:processor_list/processors/0/rules/0/input_predicate/block".to_owned(),
                "Cannot find block 'test:in'".to_owned()
            ),
            (
                "minecraft:processor_list/processors/0/rules/0/input_predicate/block".to_owned(),
                "-> processors[0] -> rules[0] -> input_predicate -> block:  Missing referenced asset test:in".to_owned()
            ),
            (
                "minecraft:processor_list/processors/0/rules/0/location_predicate/block_state".to_owned(),
                "Cannot find block 'test:at'".to_owned()
            ),
            (
                "minecraft:processor_list/processors/0/rules/0/location_predicate/block_state/name".to_owned(),
                "-> processors[0] -> rules[0] -> location_predicate -> block_state -> name:  Missing referenced asset test:at".to_owned()
            ),
            (
                "minecraft:processor_list/processors/0/rules/0/output_state".to_owned(),
                "Cannot find block 'test:invalid'".to_owned()
            ),
            (
                "minecraft:processor_list/processors/0/rules/0/output_state/name".to_owned(),
                "-> processors[0] -> rules[0] -> output_state -> name:  Missing referenced asset test:invalid".to_owned()
            ),
            (
                "minecraft:processor_list/processors/0/rules/0/block_entity_modifier/loot_table".to_owned(),
                "-> processors[0] -> rules[0] -> block_entity_modifier -> loot_table:  Invalid asset path loot_tables/none.json".to_owned()
            ),
            (
                "minecraft:processor_list/processors/1/delegate/blocks/1".to_owned(),
                "Cannot find block 'test:ignored'".to_owned()
            ),
        ]
    );
}

#[test]
fn the_four_json_families_read_a_document_input_under_the_unstrict_mask() {
    use super::history::{PoolFamily, ProcessorListFamily, StructureFamily, StructureSetFamily};
    use bedrock_addon_core::json::NullRule;
    use bedrock_addon_core::versioning::{ErrorMask, Family, InputKind, StructIndex};
    // The game registers each of the four slices as not strict: unknown members are
    // warnings.
    let slice = StructIndex::Stable(0);
    let declared = [
        (
            StructureFamily::INPUT,
            StructureFamily::null_rule(slice),
            StructureFamily::error_mask(slice),
        ),
        (
            PoolFamily::INPUT,
            PoolFamily::null_rule(slice),
            PoolFamily::error_mask(slice),
        ),
        (
            StructureSetFamily::INPUT,
            StructureSetFamily::null_rule(slice),
            StructureSetFamily::error_mask(slice),
        ),
        (
            ProcessorListFamily::INPUT,
            ProcessorListFamily::null_rule(slice),
            ProcessorListFamily::error_mask(slice),
        ),
    ];
    for (input, rule, mask) in declared {
        assert_eq!(input, InputKind::Document);
        assert_eq!(rule, Some(NullRule::Rejected));
        assert_eq!(mask, ErrorMask::Unstrict);
    }
}
