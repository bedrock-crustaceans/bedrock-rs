//! Unit tests of the crate-private parts: the name pattern, the walk's file filter, the cycle
//! text's arity, the graph helpers and the link checker.

use super::link::{Targets, link};
use super::load::LOADER;
use super::parse::is_poi_name;
use super::resolve::{TagNode, collect_members, components};
use super::validate::{PART_OF_A_CYCLE, part_of_a_cycle};
use super::{Poi, PoiTag, PoiType};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Phase, Severity,
};
use bedrock_addon_core::document::Content;
use bedrock_addon_core::id::{BlockId, PoiTagId, PoiTypeId};
use bedrock_addon_core::load::{Experiment, Experiments, LoadOptions};
use bedrock_addon_core::load::{LoadContext, StackCx, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide, PackType};

/// A type document of `blocks` (a JSON array).
fn type_document(identifier: &str, blocks: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.40", "minecraft:poi_block": {{"description": {{"identifier": "{identifier}"}}, "blocks": {blocks}}}}}"#
    )
}

/// The options with the `poi` experiment on.
fn options() -> LoadOptions {
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::Poi);
    LoadOptions {
        experiments,
        ..LoadOptions::default()
    }
}

/// The group of `files` (`(path, text)`), loaded as app-data-less files outside any pack.
fn group(files: &[(&str, String)]) -> Poi {
    let options = options();
    let cx = LoadContext::for_pack(
        &options,
        uuid::Uuid::from_u128(7),
        PackType::Behavior,
        None,
        false,
    );
    let loader = LOADER;
    let parsed = files
        .iter()
        .map(|(path, text)| {
            let path = PackPath::new(path).unwrap();
            let file = WalkFile {
                pack: PackRef::app_data(),
                side: PackSide::Behavior,
                path: path.clone(),
                key: path,
                bytes: text.clone().into_bytes().into(),
            };
            (loader.parse)(&file, &cx)
        })
        .collect();
    (loader.combine)(parsed, &StackCx::new(&options, &[]))
}

/// Blocks named `minecraft:…` exist.
struct Vanilla;

impl Targets for Vanilla {
    fn block(&self, id: &BlockId) -> bool {
        id.as_str().starts_with("minecraft:")
    }
}

/// The name a finding's reference detail carries.
fn referenced(finding: &Diagnostic) -> Option<(&'static str, &str)> {
    match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => Some((kind, id.as_str())),
        _ => None,
    }
}

#[test]
fn a_poi_name_is_word_colon_word() {
    for good in ["a:b", "test:forge", "Test_1:Forge_2", "_:_", "0:9"] {
        assert!(is_poi_name(good), "{good}");
    }
    for bad in [
        "", ":", "a:", ":b", "ab", "a:b:c", "a::b", "a b:c", "a:b c", "a-b:c", "a:b-c", "a.b:c",
        "a:b.c", "a:b/c", "\u{e9}:b", "a:\u{e9}", "a:b\n", "\na:b",
    ] {
        assert!(!is_poi_name(bad), "{bad:?}");
    }
}

#[test]
fn the_walk_takes_json_files_directly_in_poi_blocks_and_poi_tags() {
    let walk = LOADER.walk;
    let reads = |path: &str| walk.matches(&PackPath::new(path).unwrap()).is_some();
    for path in [
        "poi/blocks/a.json",
        "poi/tags/a.json",
        "poi/blocks/.json",
        "poi/blocks/a.b.json",
    ] {
        assert!(reads(path), "{path}");
    }
    for path in [
        "poi/a.json",
        "poi/blocks/deep/a.json",
        "poi/tags/v1.2/a.json",
        "poi/types/a.json",
        "poi_types/a.json",
        "poi_tags/a.json",
        "pois/a.json",
        "points_of_interest/a.json",
        "poi/blocks/a.JSON",
        "poi/blocks/a.json5",
        "poi/blocks/a.json.bak",
        "poi/blocks/json",
        "poi/tags/a",
        "poi.json",
        "blocks/poi/a.json",
    ] {
        assert!(!reads(path), "{path}");
    }
}

#[test]
fn the_cycle_text_has_one_placeholder() {
    assert_eq!(PART_OF_A_CYCLE.matches("{}").count(), 1);
    let code = part_of_a_cycle(&PoiTagId::from_static("test:t1"));
    assert_eq!(code.to_string(), "test:t1 was a part of a cycle");
    assert!(matches!(
        code,
        DiagnosticCode::ReferenceCycle { text: Some(_) }
    ));
}

#[test]
fn components_group_the_nodes_that_reach_one_another() {
    // 0 → 1 → 2 → 0 is one component; 3 → 0 and 2 → 4 hang off it; 5 is alone; 6 ⇄ 7.
    let graph = vec![
        vec![1],
        vec![2],
        vec![0, 4],
        vec![0],
        vec![],
        vec![],
        vec![7],
        vec![6],
    ];
    let c = components(&graph);
    assert!(c[0] == c[1] && c[1] == c[2], "{c:?}");
    assert!(c[6] == c[7], "{c:?}");
    let mut distinct = vec![c[0], c[3], c[4], c[5], c[6]];
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(distinct.len(), 5, "{c:?}");
    assert!(components(&[]).is_empty());
}

/// The tags `t0…` with one type `i<n>` each, for `collect_members`.
fn tags_with_a_type_each(count: usize) -> indexmap::IndexMap<PoiTagId, TagNode> {
    (0..count)
        .map(|i| {
            let mut node = TagNode::default();
            node.direct.insert(PoiTypeId::parse(format!("test:i{i}")));
            (PoiTagId::parse(format!("test:t{i}")), node)
        })
        .collect()
}

#[test]
fn the_member_expansion_ends_on_a_cyclic_graph_by_itself() {
    // No clock in the crate (the sans-IO lints): a run that did not end would hit the test
    // runner's CPU limit, so returning at all is the assertion.
    const RING: usize = 1_000;
    // Handed cycles directly, without the pass that removes them: a tag that is its own
    // narrower tag, two tags naming each other, and a 1 000-tag ring with a chord.
    let mut own = tags_with_a_type_each(1);
    collect_members(&mut own, &[vec![0]]);
    assert_eq!(own[0].members, [PoiTypeId::from_static("test:i0")]);

    let mut pair = tags_with_a_type_each(2);
    collect_members(&mut pair, &[vec![1], vec![0]]);
    assert_eq!(pair[0].members.len(), 2, "the tag entered first holds both");
    assert_eq!(pair[1].members, [PoiTypeId::from_static("test:i1")]);

    let mut ring = tags_with_a_type_each(RING);
    let mut narrower: Vec<Vec<usize>> = (0..RING).map(|i| vec![(i + 1) % RING]).collect();
    narrower[RING / 2].push(0);
    collect_members(&mut ring, &narrower);
    assert_eq!(ring[0].members.len(), RING);
    assert_eq!(ring[RING - 1].members.len(), 1);

    // An acyclic graph is expanded as before: 2 ⊂ 1 ⊂ 0 and 2 ⊂ 0.
    let mut chain = tags_with_a_type_each(3);
    collect_members(&mut chain, &[vec![1, 2], vec![2], vec![]]);
    let names = |node: &TagNode| -> Vec<String> {
        node.members.iter().map(|m| m.as_str().to_owned()).collect()
    };
    assert_eq!(names(&chain[0]), ["test:i0", "test:i1", "test:i2"]);
    assert_eq!(names(&chain[1]), ["test:i1", "test:i2"]);
    assert_eq!(names(&chain[2]), ["test:i2"]);
}

#[test]
fn a_long_chain_of_tags_resolves_without_recursion() {
    // 20 000 tags, each the child of the next, and one type at the bottom.
    const LENGTH: usize = 20_000;
    let chain: Vec<Vec<usize>> = (0..LENGTH)
        .map(|i| {
            if i + 1 < LENGTH {
                vec![i + 1]
            } else {
                Vec::new()
            }
        })
        .collect();
    let c = components(&chain);
    let mut distinct = c.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        LENGTH,
        "no two tags of a chain share a component"
    );

    let mut files = vec![(
        "poi/blocks/type.json".to_owned(),
        r#"{"format_version": "1.26.40", "minecraft:poi_block": {"description": {"identifier": "test:bottom"}, "blocks": ["minecraft:anvil"], "tags": ["test:t0"]}}"#.to_owned(),
    )];
    for i in 0..2_000 {
        files.push((
            format!("poi/tags/t{i:05}.json"),
            format!(
                r#"{{"format_version": "1.26.40", "minecraft:poi_tag": {{"description": {{"identifier": "test:t{i}"}}, "parents": ["test:t{}"]}}}}"#,
                i + 1
            ),
        ));
    }
    files.push((
        "poi/tags/top.json".to_owned(),
        r#"{"format_version": "1.26.40", "minecraft:poi_tag": {"description": {"identifier": "test:t2000"}}}"#.to_owned(),
    ));
    let files: Vec<(&str, String)> = files.iter().map(|(p, t)| (p.as_str(), t.clone())).collect();
    let poi = group(&files);
    assert_eq!(poi.diagnostics().count(), 0);
    assert_eq!(
        poi.members(&PoiTagId::from_static("test:t2000")),
        [PoiTypeId::from_static("test:bottom")]
    );
}

#[test]
fn m7_a_block_no_registry_knows_is_an_unresolved_reference_at_its_entry() {
    let poi = group(&[
        (
            "poi/blocks/forge.json",
            type_document(
                "test:forge",
                r#"["test:not_a_block", "minecraft:bell", "test:other"]"#,
            ),
        ),
        (
            "poi/blocks/fine.json",
            type_document("test:fine", r#"["minecraft:anvil"]"#),
        ),
    ]);
    let found = link(&poi, &Vanilla);
    let at: Vec<(String, Option<(&str, &str)>)> = found
        .iter()
        .map(|d| {
            assert!(
                matches!(
                    d.code,
                    DiagnosticCode::UnresolvedReference { text: Some(_) }
                ),
                "{:?}",
                d.code
            );
            // The registry's log is error level; the rest of the type is kept.
            assert_eq!(d.severity, Severity::Error);
            assert_eq!(d.phase, Phase::Link);
            assert_eq!(
                d.location.path.as_ref().map(PackPath::as_str),
                Some("poi/blocks/forge.json")
            );
            (d.location.json_path.to_string(), referenced(d))
        })
        .collect();
    assert_eq!(
        at,
        [
            (
                "minecraft:poi_block/blocks/0".to_owned(),
                Some(("block", "test:not_a_block"))
            ),
            (
                "minecraft:poi_block/blocks/2".to_owned(),
                Some(("block", "test:other"))
            ),
        ]
    );
}

#[test]
fn m7_known_blocks_and_shadowed_types_are_not_reported() {
    // Every block known: nothing.
    let poi = group(&[(
        "poi/blocks/fine.json",
        type_document("test:fine", r#"["minecraft:anvil"]"#),
    )]);
    assert!(link(&poi, &Vanilla).is_empty());
    // The definition that lost is not looked at: its blocks went with it.
    let poi = group(&[
        (
            "poi/blocks/a.json",
            type_document("test:forge", r#"["test:not_a_block"]"#),
        ),
        (
            "poi/blocks/b.json",
            type_document("test:forge", r#"["minecraft:anvil"]"#),
        ),
    ]);
    assert!(link(&poi, &Vanilla).is_empty());
    // A type whose every block is unknown gets the reference warning, never `PoiNoBlocks`
    // (poi40: the document's list was not empty).
    let poi = group(&[(
        "poi/blocks/forge.json",
        type_document("test:forge", r#"["test:not_a_block"]"#),
    )]);
    let found = link(&poi, &Vanilla);
    assert_eq!(found.len(), 1);
    assert!(found.iter().all(|d| d.code.name() == "UnresolvedReference"));
    assert_eq!(poi.diagnostics().count(), 0);
}

#[test]
fn the_documents_are_the_content_types_of_the_behaviour_side() {
    assert_eq!(<PoiType as Content>::SIDES, [PackSide::Behavior]);
    assert_eq!(<PoiTag as Content>::SIDES, [PackSide::Behavior]);
    assert_eq!(<PoiType as Content>::NAME, "poi type");
    assert_eq!(<PoiTag as Content>::NAME, "poi tag");
}

#[test]
fn a_value_the_writer_refuses_is_an_error_of_validate_not_silence() {
    use bedrock_addon_core::diagnostic::JsonPath;
    use bedrock_addon_core::json::{EncodeError, Unrepresentable};
    // No POI value can be refused at present, so the finding is built from the writer's error.
    let mut member = JsonPath::new();
    member.push("tickets");
    let error = EncodeError::NotRepresentable {
        at: member,
        what: Unrepresentable::NaN,
    };
    let finding = super::load::unwritable(&error, "minecraft:poi_block");
    assert!(matches!(finding.code, DiagnosticCode::InvalidValue { .. }));
    assert_eq!(finding.severity, Severity::Error);
    assert_eq!(finding.phase, Phase::Validate);
    assert_eq!(
        finding.location.json_path.to_string(),
        "minecraft:poi_block/tickets"
    );
    // And every value the types can hold is written: the extremes validate cleanly.
    let options = options();
    let cx = LoadContext::for_pack(
        &options,
        uuid::Uuid::from_u128(7),
        PackType::Behavior,
        None,
        false,
    );
    let extreme = PoiType {
        tickets: Some(bedrock_addon_core::json::StrictInt(u8::MAX)),
        usable_distance: Some(bedrock_addon_core::json::StrictInt(0)),
        tags: Some(Vec::new()),
        ..PoiType::new(
            PoiTypeId::from_static("test:forge"),
            vec![BlockId::from_static("minecraft:anvil")],
        )
    };
    assert!(extreme.validate(&cx).is_empty());
}

#[test]
fn both_documents_read_a_minecraft_document_input_under_the_strict_mask() {
    use super::history::{PoiTagFamily, PoiTypeFamily};
    use bedrock_addon_core::json::NullRule;
    use bedrock_addon_core::versioning::{ErrorMask, Family, InputKind, StructIndex};
    // Both loaders read a document input, and the one slice of each is read strictly.
    let slice = StructIndex::Stable(0);
    assert_eq!(PoiTypeFamily::INPUT, InputKind::Document);
    assert_eq!(PoiTagFamily::INPUT, InputKind::Document);
    assert_eq!(PoiTypeFamily::null_rule(slice), Some(NullRule::Rejected));
    assert_eq!(PoiTagFamily::null_rule(slice), Some(NullRule::Rejected));
    assert_eq!(PoiTypeFamily::error_mask(slice), ErrorMask::Strict);
    assert_eq!(PoiTagFamily::error_mask(slice), ErrorMask::Strict);
}

#[test]
fn m7_the_unknown_block_carries_the_registry_s_text() {
    // The game looks each name up in the block registry with logging on, and the registry's
    // log is this text.
    let poi = group(&[(
        "poi/blocks/forge.json",
        type_document("test:forge", r#"["test:not_a_block"]"#),
    )]);
    let found = link(&poi, &Vanilla);
    let found: Vec<&Diagnostic> = found.iter().collect();
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].code.to_string(),
        "BlockTypeRegistry::lookupByName() failed to find block \"test:not_a_block\""
    );
    assert!(matches!(
        found[0].code,
        DiagnosticCode::UnresolvedReference { text: Some(_) }
    ));
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:poi_block/blocks/0"
    );
}

#[test]
fn a_walked_file_implements_debug() {
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<super::load::PoiFile>();
}
