//! One test per fixture row, each over its fixture tree under
//! `tests/fixtures/worldgen/jigsaw/<row>/` and that tree's `expected.json`.

use bedrock_addon::families::jigsaw::{
    Axis, BlockEntityModifier, BlockPredicate, CappedDelegate, GenerationStep, HeightProvider,
    HeightmapProjection, IntProvider, LiquidSettings, PoolAlias, PoolElement, PositionPredicate,
    Processor, Projection, SpreadType, TerrainAdaptation, VerticalAnchor,
};
use bedrock_addon::id::{JigsawStructureId, ProcessorListId, StructureSetId, TemplatePoolId};
use bedrock_addon::shared::{Filter, StateValue};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;

use super::common::load_fixture;

/// A structure identifier literal.
fn structure(id: &'static str) -> JigsawStructureId {
    JigsawStructureId::from_static(id)
}

/// A pool identifier literal.
fn pool(id: &'static str) -> TemplatePoolId {
    TemplatePoolId::from_static(id)
}

#[test]
fn jig01_each_payload_at_1_21_20() {
    let stack = load_fixture("jig01");
    let jigsaw = stack.jigsaw();
    let ruin = jigsaw
        .structures
        .get(&structure("demo:ruin"))
        .expect("the structure");
    assert_eq!(ruin.step, GenerationStep::SurfaceStructures);
    assert_eq!(ruin.max_depth.0, 4);
    assert!(matches!(
        ruin.start_height,
        HeightProvider::Constant {
            value: VerticalAnchor::Absolute(a),
            ..
        } if a.0 == 0
    ));
    assert_eq!(
        ruin.version().parsed_as,
        FormatVersion::new(1, 21, 20),
        "the only struct"
    );
    let start = jigsaw
        .pools
        .get(&pool("demo:ruin/start"))
        .expect("the pool");
    assert!(matches!(
        &start.elements[0].element,
        PoolElement::Single { location, .. } if location.as_str() == "ruin/start"
    ));
    let set = jigsaw
        .structure_sets
        .get(&StructureSetId::from_static("demo:ruins"))
        .expect("the set");
    assert_eq!(set.placement.spacing.0, 12);
    let list = jigsaw
        .processors
        .get(&ProcessorListId::from_static("demo:ruin_rot"))
        .expect("the processor list");
    assert!(matches!(
        &list.processors[0],
        Processor::ProtectedBlocks { value, .. } if value.0 == "minecraft:features_cannot_replace"
    ));
}

#[test]
fn jig02_below_1_21_20_is_rejected_with_the_puv_text() {
    let stack = load_fixture("jig02");
    let jigsaw = stack.jigsaw();
    assert_eq!(jigsaw.structures.rejected().len(), 1);
    assert_eq!(jigsaw.pools.rejected().len(), 1);
    assert_eq!(jigsaw.structure_sets.rejected().len(), 1);
    assert_eq!(jigsaw.processors.rejected().len(), 1);
}

#[test]
fn jig03_unknown_members_warn_and_load() {
    let stack = load_fixture("jig03");
    let ruin = stack
        .jigsaw()
        .structures
        .get(&structure("demo:ruin"))
        .expect("the structure");
    assert!(ruin.extra.get("demo_note").is_some());
    assert!(ruin.description.extra.get("demo_flag").is_some());
}

#[test]
fn jmeta01_a_1_21_50_metadata_file_is_kept_with_its_body_skipped() {
    let stack = load_fixture("jmeta01");
    let file = stack
        .jigsaw()
        .metadata
        .get(&PackPath::new("worldgen/metadata/a.dat").unwrap())
        .expect("the metadata file");
    assert_eq!(file.format_version, "1.21.50");
    assert_eq!(file.body, [1, 2, 3]);
    assert_eq!(file.version().parsed_as, FormatVersion::new(1, 21, 50));
    let mut bytes = vec![7];
    bytes.extend_from_slice(b"1.21.50");
    bytes.extend_from_slice(&[1, 2, 3]);
    assert_eq!(file.to_bytes(), bytes, "written back byte for byte");
}

#[test]
fn jmeta02_a_1_21_40_metadata_file_has_no_parser() {
    let stack = load_fixture("jmeta02");
    assert_eq!(stack.jigsaw().metadata.rejected().len(), 1);
}

#[test]
fn jmeta03_bad_leading_string_skips_the_file() {
    let stack = load_fixture("jmeta03_bad_leading_string");
    assert_eq!(stack.jigsaw().metadata.rejected().len(), 1);
}

#[test]
fn jig04_vanilla_test_inputs_load_with_their_defaults() {
    let stack = load_fixture("jig04_vanilla_test_inputs");
    let jigsaw = stack.jigsaw();
    let trail = jigsaw
        .structures
        .get(&structure("minecraft:trail_ruins"))
        .expect("the trail ruins");
    assert_eq!(trail.terrain_adaptation(), TerrainAdaptation::Bury);
    assert_eq!(
        trail.heightmap_projection(),
        HeightmapProjection::WorldSurface
    );
    assert_eq!(trail.liquid_settings(), LiquidSettings::IgnoreWaterlogging);
    assert!(matches!(trail.biome_filters, Some(Filter::List(ref list)) if list.len() == 1));
    // Defaults the document leaves out.
    assert_eq!(trail.start_jigsaw_name(), "");
    assert_eq!(trail.dimension_padding(), (0, 0));
    assert!(trail.pool_aliases().is_empty());
    assert_eq!(trail.max_distance_from_center(), (80, 80));
    let set = jigsaw
        .structure_sets
        .get(&StructureSetId::from_static("minecraft:trail_ruins"))
        .expect("the set");
    assert_eq!(
        (set.placement.separation.0, set.placement.spacing.0),
        (8, 34)
    );
    assert_eq!(set.placement.spread_type(), SpreadType::Triangular);
    let roads = jigsaw
        .processors
        .get(&ProcessorListId::from_static(
            "minecraft:trail_ruins_roads_archaeology",
        ))
        .expect("the processor list");
    let Processor::Capped {
        delegate: CappedDelegate::Rule { rules, .. },
        limit: IntProvider::Value(2),
        ..
    } = &roads.processors[1]
    else {
        panic!("a capped rule processor: {:?}", roads.processors[1]);
    };
    assert!(matches!(
        rules[0].block_entity_modifier,
        Some(BlockEntityModifier::AppendLoot { .. })
    ));
    assert!(matches!(
        rules[0].location_predicate(),
        BlockPredicate::AlwaysTrue { .. }
    ));
    let Processor::Rule { rules, .. } = &roads.processors[0] else {
        panic!("a rule processor");
    };
    assert!(matches!(
        &rules[0].input_predicate,
        Some(BlockPredicate::RandomBlockMatch { probability, .. }) if (probability.0 - 0.2).abs() < 1e-6
    ));
    let tower = jigsaw
        .pools
        .get(&pool("minecraft:trail_ruins/tower"))
        .expect("the pool");
    assert_eq!(tower.fallback().as_str(), "minecraft:fallback");
    assert_eq!(tower.elements.len(), 2);
    assert_eq!(tower.elements[1].weight(), 1);
    assert_eq!(tower.elements[1].element.processors(), None);
    assert_eq!(
        tower.elements[1].element.projection(),
        Some(Projection::Rigid)
    );
}

#[test]
fn jig05_max_distance_forms() {
    let stack = load_fixture("jig05_max_distance_forms");
    let jigsaw = stack.jigsaw();
    let distance = |id| {
        jigsaw
            .structures
            .get(&structure(id))
            .expect("loaded")
            .max_distance_from_center()
    };
    assert_eq!(distance("demo:a_integer"), (7, i32::MAX));
    assert_eq!(distance("demo:b_horizontal"), (7, i32::MAX));
    assert_eq!(distance("demo:c_both"), (7, 7));
    assert_eq!(distance("demo:f_absent"), (80, i32::MAX));
}

#[test]
fn jig06_max_depth() {
    let stack = load_fixture("jig06_max_depth");
    let deep = stack
        .jigsaw()
        .structures
        .get(&structure("demo:deep"))
        .expect("max_depth 20 loads");
    assert_eq!(deep.max_depth.0, 20);
}

#[test]
fn jig07_spacing_separation() {
    let stack = load_fixture("jig07_spacing_separation");
    let jigsaw = stack.jigsaw();
    assert!(
        !jigsaw
            .structure_sets
            .contains(&StructureSetId::from_static("minecraft:trail_ruins"))
    );
    assert!(
        jigsaw
            .structure_sets
            .contains(&StructureSetId::from_static("demo:ok"))
    );
}

#[test]
fn jig08_invalid_identifier() {
    let stack = load_fixture("jig08_invalid_identifier");
    let jigsaw = stack.jigsaw();
    assert_eq!(jigsaw.processors.rejected().len(), 1);
    assert_eq!(jigsaw.structures.rejected().len(), 1);
    assert_eq!(jigsaw.pools.rejected().len(), 1);
    assert_eq!(jigsaw.structure_sets.rejected().len(), 1);
}

#[test]
fn jig09_duplicate_first_wins() {
    let stack = load_fixture("jig09_duplicate_first_wins");
    let jigsaw = stack.jigsaw();
    let ruin = jigsaw
        .structures
        .get(&structure("demo:ruin"))
        .expect("kept");
    assert_eq!(ruin.max_depth.0, 7, "the high-priority pack's");
    assert_eq!(ruin.origin().pack.index(), Some(0));
    let dup = jigsaw.structures.get(&structure("demo:dup")).expect("kept");
    assert_eq!(dup.max_depth.0, 1, "the first file of the pack");
    assert!(
        jigsaw.pools.contains(&pool("demo:ruin")),
        "families are separate"
    );
}

#[test]
fn jig10_file_filter() {
    let stack = load_fixture("jig10_file_filter");
    let jigsaw = stack.jigsaw();
    let b = jigsaw.structures.get(&structure("demo:b")).expect("loaded");
    assert_eq!(b.origin().path.as_str(), "worldgen/structures/sub/b.json");
}

#[test]
fn jig11_processor_types() {
    let stack = load_fixture("jig11_processor_types");
    let capped = stack
        .jigsaw()
        .processors
        .get(&ProcessorListId::from_static("demo:capped"))
        .expect("loaded");
    let limits: Vec<&IntProvider> = capped
        .processors
        .iter()
        .map(|p| match p {
            Processor::Capped { limit, .. } => limit,
            other => panic!("{other:?}"),
        })
        .collect();
    assert!(matches!(limits[0], IntProvider::Value(2)));
    assert!(matches!(
        limits[1],
        IntProvider::Constant { value, .. } if value.0 == 2
    ));
    assert!(matches!(
        limits[2],
        IntProvider::Uniform { min_inclusive, max_inclusive, .. }
            if (min_inclusive.0, max_inclusive.0) == (1, 3)
    ));
}

#[test]
fn jig12_predicates() {
    let stack = load_fixture("jig12_predicates");
    let list = stack
        .jigsaw()
        .processors
        .get(&ProcessorListId::from_static("demo:predicates"))
        .expect("loaded");
    let Processor::Rule { rules, .. } = &list.processors[0] else {
        panic!("a rule processor");
    };
    assert_eq!(rules.len(), 9, "every rule kept");
    let BlockPredicate::BlockstateMatch { block_state, .. } = rules[4].input_predicate() else {
        panic!("blockstate_match");
    };
    assert!(!block_state.object, "the string form");
    let BlockPredicate::RandomBlockstateMatch { block_state, .. } = rules[5].input_predicate()
    else {
        panic!("random_blockstate_match");
    };
    assert_eq!(
        block_state.states.get("pillar_axis"),
        Some(&StateValue::String("y".into()))
    );
    let Some(
        position @ PositionPredicate::AxisAlignedLinearPos {
            min_chance,
            max_chance,
            min_dist,
            max_dist,
            ..
        },
    ) = &rules[6].position_predicate
    else {
        panic!("axis_aligned_linear_pos");
    };
    assert_eq!(position.axis(), Some(Axis::Y));
    assert!(min_chance.is_none() && max_chance.is_none());
    assert!(min_dist.is_none() && max_dist.is_none());
    assert!(
        rules[8].block_entity_modifier.is_none(),
        "the empty loot table's modifier is dropped, the rule kept"
    );
}

#[test]
fn jig13_unknown_block_loads_and_keeps_the_rule() {
    let stack = load_fixture("jig13_unknown_block");
    let list = stack
        .jigsaw()
        .processors
        .get(&ProcessorListId::from_static("test:unknown_block"))
        .expect("loaded");
    let Processor::Rule { rules, .. } = &list.processors[0] else {
        panic!("a rule processor");
    };
    assert_eq!(rules[0].output_state.name.as_str(), "test:invalid");
}

#[test]
fn jig14_pool_elements() {
    let stack = load_fixture("jig14_pool_elements");
    let jigsaw = stack.jigsaw();
    let single = jigsaw.pools.get(&pool("demo:single")).expect("loaded");
    assert_eq!(single.elements[0].element.processors(), None);
    assert_eq!(
        single.elements[0].element.projection(),
        Some(Projection::Rigid)
    );
    let feature = jigsaw.pools.get(&pool("demo:feature")).expect("loaded");
    assert!(matches!(
        &feature.elements[0].element,
        PoolElement::Feature { feature, .. } if feature.as_str() == "minecraft:oak_tree_feature"
    ));
}

#[test]
fn jig15_element_redefinition() {
    let stack = load_fixture("jig15_element_redefinition");
    let jigsaw = stack.jigsaw();
    assert!(jigsaw.pools.contains(&pool("test:pool_a")));
    assert!(jigsaw.pools.contains(&pool("test:pool_b")));
}

#[test]
fn jig16_pool_alias_forms() {
    let stack = load_fixture("jig16_pool_alias_forms");
    let jigsaw = stack.jigsaw();
    for id in ["demo:untagged", "demo:typed"] {
        let s = jigsaw.structures.get(&structure(id)).expect("loaded");
        let aliases = s.pool_aliases();
        assert!(matches!(aliases[0], PoolAlias::Direct { .. }), "{id}");
        assert!(matches!(aliases[1], PoolAlias::Random { .. }), "{id}");
        assert!(matches!(aliases[2], PoolAlias::RandomGroup { .. }), "{id}");
    }
}

#[test]
fn jig17_set_entry_without_a_namespace_is_skipped() {
    let stack = load_fixture("jig17_set_entry_unknown_structure");
    let set = stack
        .jigsaw()
        .structure_sets
        .get(&StructureSetId::from_static("demo:set"))
        .expect("loaded");
    let entries: Vec<&str> = set
        .structures
        .iter()
        .map(|e| e.structure.as_str())
        .collect();
    assert_eq!(entries, ["demo:missing"], "`missing` is skipped");
}

#[test]
fn jig18_missing_format_version_parses_at_the_floor() {
    let stack = load_fixture("jig18_missing_format_version");
    let ruin = stack
        .jigsaw()
        .structures
        .get(&structure("demo:ruin"))
        .expect("loaded");
    assert_eq!(ruin.version().written, None);
    assert_eq!(ruin.version().effective, FormatVersion::new(1, 21, 20));
}

#[test]
fn jig19_above_engine_is_rejected() {
    let stack = load_fixture("jig19_above_engine");
    assert_eq!(stack.jigsaw().structures.rejected().len(), 1);
}

#[test]
fn jig20_trailing_commas_load() {
    let stack = load_fixture("jig20_trailing_commas");
    let p = stack
        .jigsaw()
        .pools
        .get(&pool("test:pool"))
        .expect("loaded");
    assert_eq!(p.elements[0].weight(), 1);
}

#[test]
fn jig21_generation_cases_load_cleanly() {
    load_fixture("jig21_generation_texts");
}

#[test]
fn jig22_unknown_feature_loads_cleanly() {
    load_fixture("jig22_feature_pool_element_unknown_feature");
}

#[test]
fn jig23_generation_start_pool_cases_load_cleanly() {
    let stack = load_fixture("jig23_generation_start_pool_texts");
    let c = stack
        .jigsaw()
        .structures
        .get(&structure("demo:c"))
        .expect("loaded");
    assert_eq!(c.start_jigsaw_name(), "nope");
}
