//! The jigsaw record documents run through the loader: a missing `format_version`, duplicate
//! identifiers, and the other places the crate departs from the game (tests named `deviation`),
//! and the metadata step.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Content;
use bedrock_addon::families::jigsaw::history::MetadataFamily;
use bedrock_addon::families::jigsaw::{
    JigsawMetadata, JigsawStructure, StructureSet, TemplatePool,
};
use bedrock_addon::id::{JigsawStructureId, TemplatePoolId};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;

use super::common::{document, load_fixture, loads, names, pack, stack};

/// A minimal structure payload with `identifier`.
fn structure(identifier: &str) -> String {
    format!(
        r#"{{"description": {{"identifier": "{identifier}"}}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {{"type": "constant", "value": {{"absolute": 0}}}}}}"#
    )
}

/// A minimal pool payload with `identifier`.
fn pool(identifier: &str) -> String {
    format!(r#"{{"description": {{"identifier": "{identifier}"}}, "elements": []}}"#)
}

/// A minimal set payload with `identifier`.
fn set(identifier: &str) -> String {
    format!(
        r#"{{"description": {{"identifier": "{identifier}"}}, "placement": {{"type": "minecraft:random_spread", "salt": 1, "separation": 1, "spacing": 4}}, "structures": []}}"#
    )
}

/// A minimal processor list payload with `identifier`.
fn processors(identifier: &str) -> String {
    format!(r#"{{"description": {{"identifier": "{identifier}"}}, "processors": []}}"#)
}

/// A missing `format_version` parses at 1.21.20 with only a `MissingFormatVersion` information
/// (the game says nothing).
fn missing_format_version<T: Content + std::fmt::Debug>(path: &str, key: &str, payload: &str) {
    let text = format!(r#"{{"{key}": {payload}}}"#);
    let loaded = loads::<T>(path, &text);
    assert_eq!(
        names(loaded.diagnostics(), Severity::Info),
        ["MissingFormatVersion"]
    );
    assert_eq!(loaded.version().effective, FormatVersion::new(1, 21, 20));
}

/// Two definitions of one identifier in one pack and in a lower pack: the first in walk order
/// of the top pack stays, each other one is a `ShadowedDefinition` info.
fn duplicate_identifier(directory: &str, key: &str, payload: &str) {
    let file = |name: &str| {
        (
            format!("worldgen/{directory}/{name}.json"),
            document("1.21.20", key, payload).into_bytes(),
        )
    };
    let top = pack(10, vec![file("a"), file("b")]);
    let low = pack(20, vec![file("a")]);
    let stack = stack(vec![top, low]);
    let shadowed: Vec<(Option<usize>, String)> = stack
        .jigsaw()
        .diagnostics()
        // By legacy name: the variant and the definitions that were it.
        .filter(|d| matches!(d.code.name(), "ShadowedDefinition"))
        .map(|d| {
            assert_eq!(d.severity, Severity::Info);
            (
                d.location
                    .pack
                    .as_ref()
                    .and_then(bedrock_addon::pack::PackRef::index),
                d.location.path.as_ref().unwrap().as_str().to_owned(),
            )
        })
        .collect();
    assert_eq!(shadowed.len(), 2, "{shadowed:?}");
    assert!(shadowed.contains(&(Some(0), format!("worldgen/{directory}/b.json"))));
    assert!(shadowed.contains(&(Some(1), format!("worldgen/{directory}/a.json"))));
}

/// The body of a `.dat` file is kept as bytes with a `SkippedCacheDocument` info.
fn metadata_body_skipped() -> Result<(), &'static str> {
    let mut bytes = vec![7];
    bytes.extend_from_slice(b"1.21.90");
    bytes.extend_from_slice(&[0xde, 0xad]);
    let loaded = JigsawMetadata::parse(
        &PackPath::new("worldgen/metadata/a.dat").unwrap(),
        &bytes,
        &super::common::context(),
    )
    .map_err(|_| "the file is rejected")?;
    if loaded.body != [0xde, 0xad] {
        return Err("the body is not kept");
    }
    if names(loaded.diagnostics(), Severity::Info) != ["SkippedCacheDocument"] {
        return Err("no SkippedCacheDocument info");
    }
    Ok(())
}

#[test]
fn jmeta01_typed_use_legacy_rotation_true_upgrades_without_it_with_a_dropped_field_info() {
    use bedrock_addon::json::JsonSlice;
    use bedrock_addon::load::LoadContext;
    let text = r#"{"format_version": "1.21.50", "minecraft:jigsaw_structure_metadata": {"metadata": {"42": {"size": [1, 2, 3], "use_legacy_rotation": true, "markers": [{"name": "a", "pool": "p", "target": "t", "block": "b", "final_block": "f", "joint_type": "rollable", "pos": [0, 1, 2], "selection_priority": 0, "placement_priority": 0}]}}}}"#;
    let outcome = bedrock_addon::versioning::load::<MetadataFamily>(
        JsonSlice::new(text),
        &LoadContext::detached(),
    );
    let Some(bedrock_addon::versioning::Loaded::Stable(registry)) = outcome.value else {
        panic!("{:#?}", outcome.diagnostics);
    };
    assert!(outcome.version.upgraded);
    let entry = &registry.metadata["42"];
    assert_eq!(entry.size, [1, 2, 3]);
    assert_eq!(entry.markers.len(), 1);
    let dropped: Vec<_> = outcome
        .diagnostics
        .iter()
        .filter(|d| d.code.name() == "DroppedField")
        .collect();
    assert_eq!(dropped.len(), 1);
    assert_eq!(dropped[0].severity, Severity::Info);
    assert_eq!(
        dropped[0].location.json_path.to_string(),
        "minecraft:jigsaw_structure_metadata/metadata/42/use_legacy_rotation"
    );
}

#[test]
fn structure_missing_format_version_deviation() {
    missing_format_version::<JigsawStructure>(
        "worldgen/structures/a.json",
        "minecraft:jigsaw",
        &structure("demo:a"),
    );
}

#[test]
fn structure_duplicate_identifier_deviation() {
    duplicate_identifier("structures", "minecraft:jigsaw", &structure("demo:a"));
}

#[test]
fn structure_empty_start_pool_deviation() {
    let payload = structure("demo:a").replace(r#""demo:start""#, r#""""#);
    let loaded = loads::<JigsawStructure>(
        "worldgen/structures/a.json",
        &document("1.21.20", "minecraft:jigsaw", &payload),
    );
    let finding = loaded
        .diagnostics()
        .iter()
        .find(|d| d.code.name() == "InvalidValue")
        .expect("the builder's text");
    assert_eq!(finding.severity, Severity::Info, "the document stays");
    assert_eq!(
        finding.code.to_string(),
        "[Jigsaw Structure Builder] start_pool is empty. Fill it in with a valid template pool e.g. minecraft:example/structure"
    );
}

#[test]
fn pool_missing_format_version_deviation() {
    missing_format_version::<TemplatePool>(
        "worldgen/template_pools/a.json",
        "minecraft:template_pool",
        &pool("demo:a"),
    );
}

#[test]
fn pool_duplicate_identifier_deviation() {
    duplicate_identifier("template_pools", "minecraft:template_pool", &pool("demo:a"));
}

#[test]
fn pool_feature_element_at_every_version() {
    let payload = r#"{"description": {"identifier": "demo:f"}, "elements": [{"element": {"element_type": "minecraft:feature_pool_element", "feature": "minecraft:oak_tree_feature"}}]}"#;
    for version in ["1.21.20", "1.26.20", "1.26.50"] {
        let loaded = loads::<TemplatePool>(
            "worldgen/template_pools/f.json",
            &document(version, "minecraft:template_pool", payload),
        );
        assert!(loaded.diagnostics().is_empty(), "{version}");
    }
}

#[test]
fn pool_element_redefinition_deviation() {
    let stack = load_fixture("jig15_element_redefinition");
    let found = stack
        .jigsaw()
        .diagnostics()
        .find(|d| d.code.name() == "InvalidValue")
        .expect("the redefinition");
    assert_eq!(
        found.code.to_string(),
        "Redefinition of existing element with location 'test_path/test_structure'."
    );
    // The text is the crate's own (no 1.26.60 counterpart): a lint, Info.
    assert_eq!(found.severity, bedrock_addon::diagnostic::Severity::Info);
    assert!(
        stack
            .jigsaw()
            .pools
            .contains(&TemplatePoolId::from_static("test:pool_b"))
    );
}

#[test]
fn pool_unknown_feature_text_deviation() {
    // At load the element is kept without a finding; the text is the link pass's
    // (the family's unit tests pin it).
    let stack = load_fixture("jig22_feature_pool_element_unknown_feature");
    assert_eq!(stack.jigsaw().diagnostics().count(), 0);
}

#[test]
fn set_missing_format_version_deviation() {
    missing_format_version::<StructureSet>(
        "worldgen/structure_sets/a.json",
        "minecraft:structure_set",
        &set("demo:a"),
    );
}

#[test]
fn set_duplicate_identifier_deviation() {
    duplicate_identifier("structure_sets", "minecraft:structure_set", &set("demo:a"));
}

#[test]
fn set_entry_unknown_structure() {
    // At load the entry is kept; the link pass drops it with a warning
    // (`families/jigsaw/tests.rs` `w20_set_entry_naming_no_structure_warns`).
    let stack = load_fixture("jig17_set_entry_unknown_structure");
    let set = stack
        .jigsaw()
        .structure_sets
        .iter()
        .next()
        .map(|(_, s)| s.structures[0].structure.clone())
        .expect("the set");
    assert_eq!(set, JigsawStructureId::from_static("demo:missing"));
}

#[test]
fn processors_missing_format_version_deviation() {
    missing_format_version::<bedrock_addon::families::jigsaw::ProcessorList>(
        "worldgen/processors/a.json",
        "minecraft:processor_list",
        &processors("demo:a"),
    );
}

#[test]
fn processors_duplicate_identifier_deviation() {
    duplicate_identifier(
        "processors",
        "minecraft:processor_list",
        &processors("demo:a"),
    );
}

#[test]
fn metadata_body_skipped_deviation() {
    metadata_body_skipped().unwrap();
}
