//! The deviations of the POI family: the experiment gate, the load order of types and tags, and
//! the findings vanilla does not make. Every test name ends in `deviation`.

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::families::poi::{Poi, PoiTag, PoiType};
use bedrock_addon::id::{BlockId, PoiTagId, PoiTypeId};
use bedrock_addon::load::LoadOptions;

use super::common::{block_of, context_with, load_with, loads, names, tag_of};
use super::stacks::{fixture_packs, fixture_stack, pack, stack, stack_with};

/// The findings of the group named `code`.
fn found<'a>(poi: &'a Poi, code: &str) -> Vec<&'a Diagnostic> {
    poi.diagnostics().filter(|d| d.code.id() == code).collect()
}

/// A stack of one user pack of `files`.
fn one_pack(
    files: &[(&str, &str)],
) -> bedrock_addon::pack::PackStack<bedrock_addon::pack::Behavior> {
    stack(vec![pack(10, files)])
}

/// The gate of document type `T`: a single-file load without the experiment is one
/// `MissingToggles` info and no value.
fn gate<T: bedrock_addon::document::Content + std::fmt::Debug>(text: &str) {
    let off = context_with(&LoadOptions::default(), false);
    let rejected = load_with::<T>(text, &off).expect_err("the gate is closed");
    assert_eq!(rejected.diagnostics.iter().count(), 1);
    let finding = rejected.diagnostics.iter().next().unwrap();
    assert!(finding.is(&bedrock_addon::diagnostic::codes::MISSING_TOGGLES));
    assert_eq!(finding.severity, Severity::Info);
    loads::<T>(text);
}

#[test]
fn block_experiment_gate_deviation() {
    gate::<PoiType>(&block_of("test:forge", "", r#"["minecraft:anvil"]"#));
    let stack = stack_with(fixture_packs("poi11_no_experiment"), LoadOptions::default());
    assert_eq!(stack.poi().types.len(), 0);
    assert_eq!(
        names(stack.poi().diagnostics(), Severity::Info),
        ["core/missing_toggles"]
    );
    assert_eq!(stack.poi().diagnostics().count(), 1);
}

#[test]
fn block_last_load_wins_deviation() {
    let stack = fixture_stack("poi36_shadowed_type");
    let poi = stack.poi();
    let forge = poi
        .types
        .get(&PoiTypeId::from_static("test:forge"))
        .unwrap();
    assert_eq!(forge.tickets(), 2);
    let shadowed = found(poi, "core/shadowed_definition");
    assert_eq!(shadowed.len(), 1);
    assert_eq!(shadowed[0].severity, Severity::Info);
}

#[test]
fn block_shadowed_blocks_released_deviation() {
    let low = pack(
        20,
        &[(
            "poi/blocks/forge.json",
            &block_of("test:forge", "", r#"["minecraft:anvil"]"#),
        )],
    );
    let top = pack(
        10,
        &[(
            "poi/blocks/forge.json",
            &block_of("test:forge", "", r#"["minecraft:bell"]"#),
        )],
    );
    let stack = stack(vec![top, low]);
    let poi = stack.poi();
    assert_eq!(
        poi.type_of_block(&BlockId::from_static("minecraft:anvil")),
        None
    );
    assert_eq!(
        poi.type_of_block(&BlockId::from_static("minecraft:bell")),
        Some(&PoiTypeId::from_static("test:forge"))
    );
}

#[test]
fn block_claimed_twice_deviation() {
    let stack = fixture_stack("poi37_block_claimed_twice");
    let poi = stack.poi();
    assert_eq!(
        poi.type_of_block(&BlockId::from_static("minecraft:anvil")),
        Some(&PoiTypeId::from_static("test:b"))
    );
    let claimed = found(poi, "core/poi_block_claimed_twice");
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].severity, Severity::Info);
}

#[test]
fn block_unknown_block_deviation() {
    // The load keeps the entry and says nothing; the warning is the link pass's (20 M7),
    // pinned on the crate-private checker in src/poi/tests.rs.
    let stack = fixture_stack("poi31_unknown_block");
    let poi = stack.poi();
    let forge = poi
        .types
        .get(&PoiTypeId::from_static("test:forge"))
        .unwrap();
    assert_eq!(forge.blocks.len(), 2);
    assert_eq!(poi.diagnostics().count(), 0);
}

#[test]
fn block_no_blocks_deviation() {
    let forge = loads::<PoiType>(&block_of("test:forge", "", "[]"));
    assert_eq!(
        names(forge.diagnostics().iter(), Severity::Info),
        ["core/poi_no_blocks"]
    );
}

#[test]
fn block_duplicate_block_deviation() {
    let forge = loads::<PoiType>(&block_of(
        "test:forge",
        "",
        r#"["minecraft:bell", "minecraft:bell"]"#,
    ));
    assert_eq!(forge.blocks.len(), 1);
    assert_eq!(
        names(forge.diagnostics().iter(), Severity::Info),
        ["core/no_effect"]
    );
}

#[test]
fn minecraft_namespace_allowed_deviation() {
    let stack = one_pack(&[
        (
            "poi/blocks/meeting.json",
            &block_of("minecraft:meeting", "", r#"["minecraft:bell"]"#),
        ),
        ("poi/tags/village.json", &tag_of("minecraft:village", "")),
    ]);
    let poi = stack.poi();
    assert_eq!((poi.types.len(), poi.tags.len()), (1, 1));
    assert_eq!(poi.diagnostics().count(), 0);
}

#[test]
fn tag_experiment_gate_deviation() {
    gate::<PoiTag>(&tag_of("test:civic", ""));
    let stack = stack_with(
        vec![pack(
            10,
            &[("poi/tags/civic.json", &tag_of("test:civic", ""))],
        )],
        LoadOptions::default(),
    );
    assert_eq!(stack.poi().tags.len(), 0);
    assert_eq!(
        names(stack.poi().diagnostics(), Severity::Info),
        ["core/missing_toggles"]
    );
}

#[test]
fn tag_union_deviation() {
    let stack = fixture_stack("poi38_tag_union");
    let poi = stack.poi();
    let t = poi.tags.get(&PoiTagId::from_static("test:t")).unwrap();
    assert_eq!(t.parents().len(), 2);
    assert_eq!(poi.diagnostics().count(), 0);
}

#[test]
fn tag_unknown_tag_deviation() {
    let stack = fixture_stack("poi32_unknown_tag_reference");
    let poi = stack.poi();
    let missing = found(poi, "core/unresolved_reference");
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].severity, Severity::Info);
    assert!(!poi.contains_tag(&PoiTagId::from_static("test:missing")));
}

#[test]
fn tag_on_type_creates_leaf_deviation() {
    let stack = fixture_stack("poi33_type_tag_without_document");
    let poi = stack.poi();
    assert!(poi.contains_tag(&PoiTagId::from_static("test:smithy")));
    assert_eq!(poi.diagnostics().count(), 0);
}

#[test]
fn tag_cycle_deviation() {
    let stack = fixture_stack("poi34_tag_cycle");
    let cycles = found(stack.poi(), "poi/part_of_a_cycle");
    assert_eq!(cycles.len(), 3);
    assert!(cycles.iter().all(|d| d.severity == Severity::Info));
    assert!(
        cycles
            .iter()
            .all(|d| d.code.to_string().ends_with(" was a part of a cycle"))
    );
}

#[test]
fn tag_self_edge_deviation() {
    let a = loads::<PoiTag>(&tag_of("test:a", r#", "parents": ["test:a"]"#));
    assert!(a.parents().is_empty());
    assert_eq!(
        names(a.diagnostics().iter(), Severity::Info),
        ["poi/part_of_a_cycle"]
    );
}
