//! Write-back at the Value level: `parse(write(parse(x))) == parse(x)` for
//! every document the row fixtures load; vanilla's own files rewrite byte for byte; members the
//! author left out are never written.

use std::fmt::Debug;

use bedrock_addon::document::{Content, Extra};
use bedrock_addon::families::jigsaw::{
    GenerationStep, HeightProvider, JigsawStructure, ProcessorList, StructureSet, TemplatePool,
    VerticalAnchor,
};
use bedrock_addon::id::{JigsawStructureId, TemplatePoolId};
use bedrock_addon::json::StrictInt;

use super::common::{fixture, loads};

/// Loads `path` of fixture `name`, writes it, reads the text back and requires the same value
/// and a stable rewrite; the written text.
fn round_trip<T: Content + Debug + PartialEq>(name: &str, path: &str) -> String {
    let text = String::from_utf8(fixture(&format!("{name}/{path}"))).unwrap();
    let first = loads::<T>(path, &text);
    let written = first.to_json().expect("written");
    let second = loads::<T>(path, &written);
    assert_eq!(first.value(), second.value(), "{name}/{path}");
    assert_eq!(
        second.to_json().unwrap(),
        written,
        "{name}/{path}: a stable rewrite"
    );
    written
}

#[test]
fn every_row_document_round_trips_at_the_value_level() {
    round_trip::<JigsawStructure>("jig01", "worldgen/structures/ruin.json");
    round_trip::<TemplatePool>("jig01", "worldgen/template_pools/ruin/start.json");
    round_trip::<StructureSet>("jig01", "worldgen/structure_sets/ruins.json");
    round_trip::<ProcessorList>("jig01", "worldgen/processors/ruin_rot.json");
    round_trip::<JigsawStructure>("jig03", "worldgen/structures/ruin.json");
    round_trip::<TemplatePool>("jig03", "worldgen/template_pools/start.json");
    for name in ["a_integer", "b_horizontal", "c_both", "f_absent"] {
        round_trip::<JigsawStructure>(
            "jig05_max_distance_forms",
            &format!("worldgen/structures/{name}.json"),
        );
    }
    for name in ["a_rule", "b_protected", "c_ignore", "d_capped"] {
        round_trip::<ProcessorList>(
            "jig11_processor_types",
            &format!("worldgen/processors/{name}.json"),
        );
    }
    round_trip::<ProcessorList>("jig12_predicates", "worldgen/processors/a_predicates.json");
    round_trip::<TemplatePool>(
        "jig14_pool_elements",
        "worldgen/template_pools/a_single.json",
    );
    round_trip::<TemplatePool>(
        "jig14_pool_elements",
        "worldgen/template_pools/b_feature.json",
    );
    round_trip::<JigsawStructure>(
        "jig16_pool_alias_forms",
        "worldgen/structures/a_untagged.json",
    );
    round_trip::<JigsawStructure>("jig16_pool_alias_forms", "worldgen/structures/b_typed.json");
    round_trip::<TemplatePool>("jig20_trailing_commas", "worldgen/template_pools/pool.json");
}

#[test]
fn vanilla_set_and_pool_rewrite_byte_for_byte() {
    // jig04: the vanilla trail-ruins inputs (BDS 1.26.36.1 `vanilla_1.19.80/worldgen`, with the
    // unit tests' members). The structure and the processor list are left out: vanilla's files
    // write `max_distance_from_center` before `start_height`, `limit` before `delegate` and
    // `output_state` before `block_entity_modifier`, and the writer keeps the bind order (Value
    // level).
    round_trip::<JigsawStructure>(
        "jig04_vanilla_test_inputs",
        "worldgen/structures/trail_ruins.json",
    );
    round_trip::<ProcessorList>(
        "jig04_vanilla_test_inputs",
        "worldgen/processors/trail_ruins_roads_archaeology.json",
    );
    for (path, written) in [
        (
            "worldgen/structure_sets/trail_ruins.json",
            round_trip::<StructureSet>(
                "jig04_vanilla_test_inputs",
                "worldgen/structure_sets/trail_ruins.json",
            ),
        ),
        (
            "worldgen/template_pools/trail_ruins/tower.json",
            round_trip::<TemplatePool>(
                "jig04_vanilla_test_inputs",
                "worldgen/template_pools/trail_ruins/tower.json",
            ),
        ),
    ] {
        let original =
            String::from_utf8(fixture(&format!("jig04_vanilla_test_inputs/{path}"))).unwrap();
        assert_eq!(written, original, "{path}");
    }
}

#[test]
fn members_left_out_are_not_written() {
    let structure = JigsawStructure::new(
        JigsawStructureId::from_static("demo:ruin"),
        GenerationStep::SurfaceStructures,
        TemplatePoolId::from_static("demo:start"),
        3,
        HeightProvider::Constant {
            value: VerticalAnchor::AboveBottom(StrictInt(5)),
            extra: Extra::default(),
        },
    );
    assert_eq!(
        structure.to_json().unwrap(),
        r#"{
  "format_version": "1.21.20",
  "minecraft:jigsaw": {
    "description": {
      "identifier": "demo:ruin"
    },
    "step": "surface_structures",
    "start_pool": "demo:start",
    "max_depth": 3,
    "start_height": {
      "type": "constant",
      "value": {
        "above_bottom": 5
      }
    }
  }
}
"#
    );
    let pool = TemplatePool::new(TemplatePoolId::from_static("demo:start"));
    assert_eq!(
        pool.to_json().unwrap(),
        "{\n  \"format_version\": \"1.21.20\",\n  \"minecraft:template_pool\": {\n    \"description\": {\n      \"identifier\": \"demo:start\"\n    },\n    \"elements\": []\n  }\n}\n"
    );
}

#[test]
fn a_built_document_validates_through_the_loader() {
    let mut structure = JigsawStructure::new(
        JigsawStructureId::from_static("demo:ruin"),
        GenerationStep::SurfaceStructures,
        TemplatePoolId::from_static("demo:start"),
        21,
        HeightProvider::Constant {
            value: VerticalAnchor::Absolute(StrictInt(0)),
            extra: Extra::default(),
        },
    );
    let d = structure.validate(&super::common::context());
    assert!(d.iter().any(|f| f.code.name() == "InvalidValue"), "{d:?}");
    structure.max_depth = StrictInt(20);
    assert!(structure.validate(&super::common::context()).is_empty());
}

#[test]
fn a_metadata_entrys_unknown_members_are_reported_and_written_back() {
    use bedrock_addon::diagnostic::Severity;
    use bedrock_addon::families::jigsaw::history::MetadataFamily;
    use bedrock_addon::json::JsonSlice;
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::versioning::{Loaded, load};
    // The typed model's map level: each entry is read from its own slice.
    for version in ["1.21.50", "1.21.80"] {
        let text = format!(
            r#"{{"format_version": "{version}", "minecraft:jigsaw_structure_metadata": {{"metadata": {{"42": {{"size": [1, 2, 3], "demo_entry": 1, "markers": [{{"name": "a", "pool": "p", "target": "t", "block": "b", "final_block": "f", "joint_type": "aligned", "pos": [0, 1, 2], "selection_priority": 0, "placement_priority": 0, "demo_marker": true}}]}}}}}}}}"#
        );
        let outcome = load::<MetadataFamily>(JsonSlice::new(&text), &LoadContext::detached());
        let warnings: Vec<String> = outcome
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .map(|d| format!("{} {}", d.code.name(), d.location.json_path))
            .collect();
        assert_eq!(
            warnings,
            [
                "UnrecognizedField minecraft:jigsaw_structure_metadata/metadata/42/demo_entry",
                "UnrecognizedField minecraft:jigsaw_structure_metadata/metadata/42/markers/0/demo_marker",
            ],
            "{version}"
        );
        let Some(Loaded::Stable(registry)) = outcome.value else {
            panic!("{version}: {:#?}", outcome.diagnostics);
        };
        let _ = &registry.metadata["42"];
        let entry = bedrock_addon::diagnostic::JsonPath::of_keys([
            "minecraft:jigsaw_structure_metadata",
            "metadata",
            "42",
        ]);
        let unknown = outcome.diagnostics.unknown();
        assert!(unknown.get(&entry, "demo_entry").is_some(), "{version}");
        assert!(
            unknown
                .get(&entry.child("markers").index(0), "demo_marker")
                .is_some(),
            "{version}"
        );
        let payload = bedrock_addon::versioning::VersionStruct::write(&registry).unwrap();
        // With the unknown members the load kept, at their objects below the payload key.
        let mut root = facet_value::VObject::new();
        root.insert("minecraft:jigsaw_structure_metadata", payload);
        let mut root = bedrock_addon::json::Value::from(root);
        assert!(bedrock_addon::json::merge_unknown(&mut root, unknown).is_empty());
        let written = bedrock_addon::json::to_json(&root).unwrap();
        assert!(
            written.contains("\"demo_entry\": 1") && written.contains("\"demo_marker\": true"),
            "{version}: {written}"
        );
    }
}
