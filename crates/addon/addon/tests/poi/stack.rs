//! The rows of the fixtures that need a stack: the experiment gate (poi11), the directories
//! (poi22) and the cross-document rows (poi30–poi41; the block lookups of poi31 and poi40 are
//! the link pass's, `link.rs`), with the validation rules pinned
//! at both ends.

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticDetail, Severity};
use bedrock_addon::families::poi::{Poi, PoiTag, PoiType};
use bedrock_addon::id::{BlockId, PoiTagId, PoiTypeId};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, PackStack};

use super::common::{
    block_of, context_with, describe, fixture_packs, fixture_stack, load_with, names, pack, stack,
    stack_with, tag_of,
};

/// A type identifier.
fn ty(text: &str) -> PoiTypeId {
    PoiTypeId::parse(text)
}

/// A tag identifier.
fn tg(text: &str) -> PoiTagId {
    PoiTagId::parse(text)
}

/// A block identifier.
fn bl(text: &str) -> BlockId {
    BlockId::parse(text)
}

/// The identifiers of `ids`, sorted.
fn sorted<K: bedrock_addon::id::Kind>(ids: &[bedrock_addon::id::Id<K>]) -> Vec<&str> {
    let mut out: Vec<&str> = ids.iter().map(bedrock_addon::id::Id::as_str).collect();
    out.sort_unstable();
    out
}

/// The findings of the group named `code`.
fn found<'a>(poi: &'a Poi, code: &str) -> Vec<&'a Diagnostic> {
    poi.diagnostics()
        .filter(|d| d.code.name() == code)
        .collect()
}

/// The file of a finding.
fn file(finding: &Diagnostic) -> Option<&str> {
    finding
        .location
        .path
        .as_ref()
        .map(bedrock_addon::storage::PackPath::as_str)
}

/// The name a finding's reference detail carries.
fn referenced(finding: &Diagnostic) -> Option<(&'static str, &str)> {
    match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => Some((kind, id.as_str())),
        _ => None,
    }
}

/// Requires the group to have no finding at all.
#[track_caller]
fn clean(poi: &Poi) {
    assert_eq!(
        poi.diagnostics().count(),
        0,
        "{}",
        describe(poi.diagnostics())
    );
}

/// A stack of one user pack of `files`, the experiment on.
fn one_pack(files: &[(&str, &str)]) -> PackStack<Behavior> {
    stack(vec![pack(10, files)])
}

/// A type document of `blocks` (a JSON array) carrying `tags` (a JSON array).
fn tagged(identifier: &str, blocks: &str, tags: &str) -> String {
    block_of(identifier, "", &format!(r#"{blocks}, "tags": {tags}"#))
}

// ---- E7: the experiment gate ----------------------------------------------------------------

#[test]
fn poi11_no_experiment_parses_nothing_and_says_so_once() {
    let stack = stack_with(fixture_packs("poi11_no_experiment"), LoadOptions::default());
    let poi = stack.poi();
    assert_eq!(
        (poi.types.len(), poi.tags.len()),
        (0, 0),
        "nothing is loaded"
    );
    assert_eq!(poi.rejected().count(), 0, "nothing is parsed");
    let all: Vec<&Diagnostic> = poi.diagnostics().collect();
    assert_eq!(all.len(), 1, "{}", describe(poi.diagnostics()));
    assert!(
        matches!(all[0].code, DiagnosticCode::MissingToggles { .. }),
        "{:?}",
        all[0].code
    );
    assert_eq!(all[0].severity, Severity::Info);
    assert_eq!(file(all[0]), Some("poi/blocks/forge.json"));
    assert_eq!(
        all[0]
            .location
            .pack
            .as_ref()
            .and_then(bedrock_addon::pack::PackRef::index),
        Some(0)
    );
}

#[test]
fn the_gate_reports_once_per_pack_at_its_first_file() {
    let forge = block_of("test:forge", "", r#"["minecraft:anvil"]"#);
    let civic = tag_of("test:civic", "");
    let top = pack(
        10,
        &[
            ("poi/blocks/b.json", &forge),
            ("poi/tags/a.json", &civic),
            ("poi/blocks/sub/c.json", &forge),
        ],
    );
    let low = pack(20, &[("poi/tags/z.json", &civic)]);
    let bare = pack(30, &[("blocks/x.json", "{}")]);
    let stack = stack_with(vec![top, low, bare], LoadOptions::default());
    let poi = stack.poi();
    let infos = found(poi, "MissingToggles");
    assert_eq!(
        poi.diagnostics().count(),
        2,
        "{}",
        describe(poi.diagnostics())
    );
    let mut places: Vec<(Option<usize>, Option<&str>)> = infos
        .iter()
        .map(|d| {
            (
                d.location
                    .pack
                    .as_ref()
                    .and_then(bedrock_addon::pack::PackRef::index),
                file(d),
            )
        })
        .collect();
    places.sort();
    assert_eq!(
        places,
        [
            (Some(0), Some("poi/blocks/b.json")),
            (Some(1), Some("poi/tags/z.json"))
        ],
        "one per pack with POI files, at the first in walk order"
    );
    assert!(infos.iter().all(|d| d.severity == Severity::Info));
}

#[test]
fn without_the_experiment_a_broken_file_is_not_even_read() {
    let stack = stack_with(
        vec![pack(
            10,
            &[("poi/blocks/broken.json", "{ this is not JSON")],
        )],
        LoadOptions::default(),
    );
    let poi = stack.poi();
    assert_eq!(names(poi.diagnostics(), Severity::Error), [] as [&str; 0]);
    assert_eq!(names(poi.diagnostics(), Severity::Info), ["MissingToggles"]);
    // With the experiment the same file is an error.
    let stack = one_pack(&[("poi/blocks/broken.json", "{ this is not JSON")]);
    assert_eq!(
        names(stack.poi().diagnostics(), Severity::Error),
        ["MalformedInput"]
    );
    assert_eq!(found(stack.poi(), "MissingToggles").len(), 0);
}

#[test]
fn a_single_file_load_without_the_experiment_is_rejected_unparsed() {
    let cx = context_with(&LoadOptions::default(), false);
    for text in [
        block_of("test:forge", "", r#"["minecraft:anvil"]"#),
        "{ this is not JSON".to_owned(),
    ] {
        let rejected = load_with::<PoiType>(&text, &cx).expect_err("the gate is closed");
        assert_eq!(rejected.diagnostics.iter().count(), 1);
        let finding = rejected.diagnostics.iter().next().unwrap();
        assert!(matches!(
            finding.code,
            DiagnosticCode::MissingToggles { .. }
        ));
        assert_eq!(finding.severity, Severity::Info);
    }
    let rejected =
        load_with::<PoiTag>(&tag_of("test:civic", ""), &cx).expect_err("the gate is closed");
    assert_eq!(
        names(rejected.diagnostics.iter(), Severity::Info),
        ["MissingToggles"]
    );
}

// ---- the directories ------------------------------------------------------------------------

/// The type loader walks `poi/blocks` and the tag loader walks
/// `poi/tags`, each without sub-folders; the directories (`poi`, `poi_types`, `poi_tags`, `pois`,
/// `points_of_interest`) and `poi/types` are read by nothing.
#[test]
fn poi22_types_load_from_poi_blocks_and_tags_from_poi_tags_only() {
    let stack = fixture_stack("poi22_blocks_and_tags_only");
    let poi = stack.poi();
    assert!(poi.types.contains(&ty("test:x")), "poi/blocks/x.json");
    assert!(poi.tags.contains(&tg("test:z")), "poi/tags/z.json");
    for (id, path) in [
        ("test:w", "poi/blocks/sub/w.json"),
        ("test:y", "poi/blocks/y.json"),
        ("test:t", "poi/types/t.json"),
        ("test:v", "poi_types/v.json"),
        ("test:u", "points_of_interest/u.json"),
    ] {
        assert!(!poi.types.contains(&ty(id)), "{path} is not read");
    }
    assert_eq!((poi.types.len(), poi.tags.len()), (1, 1));
    assert_eq!(poi.rejected().count(), 0);
    clean(poi);
}

/// The directory decides the loader, not the payload key: a tag file in `poi/blocks` is a type
/// file without its payload, and a type file in `poi/tags` a tag file without its own.
#[test]
fn a_document_in_the_other_document_s_directory_is_rejected_by_that_directory_s_loader() {
    let forge = block_of("test:forge", "", r#"["minecraft:anvil"]"#);
    let civic = tag_of("test:civic", "");
    let stack = one_pack(&[
        ("poi/blocks/civic.json", &civic),
        ("poi/tags/forge.json", &forge),
    ]);
    let poi = stack.poi();
    assert_eq!((poi.types.len(), poi.tags.len()), (0, 0));
    let types: Vec<&str> = poi
        .types
        .rejected()
        .iter()
        .map(|r| r.origin.path.as_str())
        .collect();
    let tags: Vec<&str> = poi
        .tags
        .rejected()
        .iter()
        .map(|r| r.origin.path.as_str())
        .collect();
    assert_eq!(types, ["poi/blocks/civic.json"]);
    assert_eq!(tags, ["poi/tags/forge.json"]);
    for rejected in poi.rejected() {
        let missing: Vec<&Diagnostic> = rejected
            .diagnostics
            .iter()
            .filter(|f| {
                f.severity == Severity::Error
                    && matches!(f.code, DiagnosticCode::MissingPayload { .. })
            })
            .collect();
        assert_eq!(
            missing.len(),
            1,
            "{}",
            describe(rejected.diagnostics.iter())
        );
    }
}

#[test]
fn only_files_with_the_extension_json_in_the_poi_directories_are_read() {
    let forge = block_of("test:forge", "", r#"["minecraft:anvil"]"#);
    let stack = one_pack(&[
        ("poi/blocks/readme.txt", "not a document"),
        ("poi/blocks/upper.JSON", &forge),
        ("poi/blocks/double.json.bak", &forge),
        ("poi/blocks/json", &forge),
        ("blocks/forge.json", &forge),
        ("poi.json", &forge),
        ("other/poi/forge.json", &forge),
    ]);
    let poi = stack.poi();
    assert_eq!((poi.types.len(), poi.tags.len()), (0, 0));
    assert_eq!(poi.rejected().count(), 0);
    clean(poi);
}

// ---- the cross-document rows ----------------------------------------------------------------

#[test]
fn poi30_vanilla_1_26_20_set_loads_and_resolves_its_tags() {
    let stack = stack(super::common::fixture_packs_from(
        "poi30_vanilla_1_26_20_set",
        bedrock_addon::pack::PackOrigin::Package,
    ));
    let poi = stack.poi();
    assert_eq!((poi.types.len(), poi.tags.len()), (19, 3));
    clean(poi);
    let job_sites = [
        "minecraft:armorer",
        "minecraft:butcher",
        "minecraft:cartographer",
        "minecraft:cleric",
        "minecraft:farmer",
        "minecraft:fisherman",
        "minecraft:fletcher",
        "minecraft:leatherworker",
        "minecraft:librarian",
        "minecraft:mason",
        "minecraft:shepherd",
        "minecraft:toolsmith",
        "minecraft:weaponsmith",
    ];
    assert_eq!(
        sorted(poi.members(&tg("minecraft:acquirable_job_site"))),
        job_sites
    );
    let mut village: Vec<&str> = job_sites.to_vec();
    village.extend(["minecraft:home", "minecraft:meeting"]);
    village.sort_unstable();
    assert_eq!(sorted(poi.members(&tg("minecraft:village"))), village);
    assert_eq!(
        sorted(poi.members(&tg("minecraft:bee_home"))),
        ["minecraft:bee_nest", "minecraft:beehive"]
    );
    assert_eq!(
        sorted(&poi.tag_names().cloned().collect::<Vec<_>>()),
        [
            "minecraft:acquirable_job_site",
            "minecraft:bee_home",
            "minecraft:village"
        ]
    );
    // The values of the game's built-in point-of-interest types.
    let meeting = poi.types.get(&ty("minecraft:meeting")).unwrap();
    assert_eq!((meeting.tickets(), meeting.usable_distance()), (32, 6));
    let beehive = poi.types.get(&ty("minecraft:beehive")).unwrap();
    assert_eq!((beehive.tickets(), beehive.usable_distance()), (0, 1));
    assert_eq!(
        poi.type_of_block(&bl("minecraft:stonecutter_block")),
        Some(&ty("minecraft:mason"))
    );
    assert_eq!(poi.type_of_block(&bl("minecraft:stonecutter")), None);
    assert!(poi.has_tag(&ty("minecraft:mason"), &tg("minecraft:village")));
    assert!(!poi.has_tag(&ty("minecraft:mason"), &tg("minecraft:bee_home")));
    assert!(!poi.has_tag(&ty("minecraft:lodestone"), &tg("minecraft:village")));
}

#[test]
fn poi31_unknown_block_loads_the_type_as_written() {
    // The lookup of the block is the link pass's (`link.rs`).
    let stack = fixture_stack("poi31_unknown_block");
    let poi = stack.poi();
    let forge = poi.types.get(&ty("test:forge")).expect("the type loads");
    assert_eq!(forge.blocks, [bl("test:not_a_block"), bl("minecraft:bell")]);
    assert_eq!(
        poi.type_of_block(&bl("minecraft:bell")),
        Some(&ty("test:forge"))
    );
    clean(poi);
}

#[test]
fn poi32_unknown_tag_reference_is_dropped_with_a_warning() {
    let stack = fixture_stack("poi32_unknown_tag_reference");
    let poi = stack.poi();
    let a = tg("test:a");
    assert!(poi.tags.contains(&a), "the tag loads");
    assert!(poi.children(&a).is_empty(), "the reference is dropped");
    assert!(
        !poi.contains_tag(&tg("test:missing")),
        "the missing tag is not created"
    );
    assert_eq!(
        sorted(&poi.tag_names().cloned().collect::<Vec<_>>()),
        ["test:a"]
    );
    let all: Vec<&Diagnostic> = poi.diagnostics().collect();
    assert_eq!(all.len(), 1, "{}", describe(poi.diagnostics()));
    assert!(matches!(
        all[0].code,
        DiagnosticCode::UnresolvedReference { .. }
    ));
    assert_eq!(all[0].severity, Severity::Info);
    assert_eq!(file(all[0]), Some("poi/tags/a.json"));
    assert_eq!(
        all[0].location.json_path.to_string(),
        "minecraft:poi_tag/children/0"
    );
    assert_eq!(referenced(all[0]), Some(("poi tag", "test:missing")));
}

#[test]
fn a_parent_naming_no_tag_is_dropped_with_a_warning() {
    let stack = one_pack(&[
        (
            "poi/tags/a.json",
            &tag_of("test:a", r#", "parents": ["test:b", "test:missing"]"#),
        ),
        ("poi/tags/b.json", &tag_of("test:b", "")),
    ]);
    let poi = stack.poi();
    assert_eq!(
        poi.parents(&tg("test:a")),
        [tg("test:b")],
        "the known parent stays"
    );
    assert!(!poi.contains_tag(&tg("test:missing")));
    let all: Vec<&Diagnostic> = poi.diagnostics().collect();
    assert_eq!(all.len(), 1, "{}", describe(poi.diagnostics()));
    assert!(matches!(
        all[0].code,
        DiagnosticCode::UnresolvedReference { .. }
    ));
    assert_eq!(all[0].severity, Severity::Info);
    assert_eq!(
        all[0].location.json_path.to_string(),
        "minecraft:poi_tag/parents/1"
    );
    assert_eq!(referenced(all[0]), Some(("poi tag", "test:missing")));
}

#[test]
fn poi33_type_tag_without_document_is_a_leaf_tag() {
    let stack = fixture_stack("poi33_type_tag_without_document");
    let poi = stack.poi();
    let smithy = tg("test:smithy");
    assert!(poi.types.contains(&ty("test:forge")));
    assert!(poi.contains_tag(&smithy), "created on first use");
    assert!(poi.tags.get(&smithy).is_none(), "it has no document");
    assert_eq!(poi.members(&smithy), [ty("test:forge")]);
    assert!(poi.has_tag(&ty("test:forge"), &smithy));
    assert!(poi.parents(&smithy).is_empty() && poi.children(&smithy).is_empty());
    assert_eq!(poi.tag_names().cloned().collect::<Vec<_>>(), [smithy]);
    clean(poi);
}

#[test]
fn an_edge_to_a_tag_only_a_type_names_resolves() {
    // The tag exists once a type names it, so the edge has something to point at.
    let stack = one_pack(&[
        (
            "poi/blocks/forge.json",
            &tagged("test:forge", r#"["minecraft:anvil"]"#, r#"["test:smithy"]"#),
        ),
        (
            "poi/tags/craft.json",
            &tag_of("test:craft", r#", "children": ["test:smithy"]"#),
        ),
    ]);
    let poi = stack.poi();
    clean(poi);
    assert_eq!(poi.children(&tg("test:craft")), [tg("test:smithy")]);
    assert_eq!(poi.parents(&tg("test:smithy")), [tg("test:craft")]);
    assert_eq!(poi.members(&tg("test:craft")), [ty("test:forge")]);
}

#[test]
fn the_tags_of_a_type_that_did_not_load_are_not_created() {
    let stack = one_pack(&[
        // Rejected: no blocks.
        (
            "poi/blocks/broken.json",
            &super::common::block(
                r#"{"description": {"identifier": "test:broken"}, "tags": ["test:ghost"]}"#,
            ),
        ),
    ]);
    let poi = stack.poi();
    assert!(!poi.contains_tag(&tg("test:ghost")));
    assert_eq!(poi.tag_names().count(), 0);
}

#[test]
fn poi34_tag_cycle_keeps_the_tags_and_ignores_the_cycle() {
    let stack = fixture_stack("poi34_tag_cycle");
    let poi = stack.poi();
    let cycles = found(poi, "ReferenceCycle");
    assert_eq!(
        poi.diagnostics().count(),
        3,
        "{}",
        describe(poi.diagnostics())
    );
    let mut texts: Vec<String> = cycles.iter().map(|d| d.code.to_string()).collect();
    texts.sort();
    assert_eq!(
        texts,
        [
            "test:t1 was a part of a cycle",
            "test:t2 was a part of a cycle",
            "test:t3 was a part of a cycle"
        ]
    );
    assert!(cycles.iter().all(|d| d.severity == Severity::Info));
    let mut files: Vec<Option<&str>> = cycles.iter().map(|d| file(d)).collect();
    files.sort();
    assert_eq!(
        files,
        [
            Some("poi/tags/t1.json"),
            Some("poi/tags/t2.json"),
            Some("poi/tags/t3.json")
        ],
        "each at its tag's document"
    );
    for cycle in &cycles {
        assert_eq!(
            cycle.location.json_path.to_string(),
            "minecraft:poi_tag/description/identifier"
        );
        // Each names its own tag: the one whose document it is at.
        let own = file(cycle)
            .unwrap()
            .trim_start_matches("poi/tags/")
            .trim_end_matches(".json");
        assert_eq!(
            cycle.code.to_string(),
            format!("test:{own} was a part of a cycle")
        );
        assert_eq!(
            cycle
                .location
                .pack
                .as_ref()
                .and_then(bedrock_addon::pack::PackRef::index),
            Some(0)
        );
    }
    for tag in ["test:t1", "test:t2", "test:t3"] {
        assert!(poi.contains_tag(&tg(tag)), "{tag} still exists");
        assert!(
            poi.parents(&tg(tag)).is_empty(),
            "{tag} inherits nothing through the cycle"
        );
    }
    // `test:ok` registers, and its edge into the cycle is kept.
    let ok = tg("test:ok");
    assert!(poi.contains_tag(&ok));
    assert_eq!(poi.parents(&ok), [tg("test:t1")]);
    assert_eq!(poi.children(&tg("test:t1")), [ok]);
    assert!(poi.children(&tg("test:t2")).is_empty() && poi.children(&tg("test:t3")).is_empty());
}

#[test]
fn an_edge_into_a_cycle_member_is_kept_but_does_not_travel_round_the_cycle() {
    let stack = one_pack(&[
        (
            "poi/tags/t1.json",
            &tag_of("test:t1", r#", "parents": ["test:t2", "test:top"]"#),
        ),
        (
            "poi/tags/t2.json",
            &tag_of("test:t2", r#", "parents": ["test:t1"]"#),
        ),
        ("poi/tags/top.json", &tag_of("test:top", "")),
        (
            "poi/tags/ok.json",
            &tag_of("test:ok", r#", "parents": ["test:t1"]"#),
        ),
        (
            "poi/blocks/i.json",
            &tagged("test:i", r#"["minecraft:anvil"]"#, r#"["test:ok"]"#),
        ),
        (
            "poi/blocks/j.json",
            &tagged("test:j", r#"["minecraft:bell"]"#, r#"["test:t1"]"#),
        ),
    ]);
    let poi = stack.poi();
    let mut texts: Vec<String> = found(poi, "ReferenceCycle")
        .iter()
        .map(|d| d.code.to_string())
        .collect();
    texts.sort();
    assert_eq!(
        texts,
        [
            "test:t1 was a part of a cycle",
            "test:t2 was a part of a cycle"
        ],
        "a two-tag cycle; the tags outside it are not reported"
    );
    assert_eq!(poi.diagnostics().count(), 2);
    assert_eq!(
        sorted(poi.members(&tg("test:t1"))),
        ["test:i", "test:j"],
        "its own type and its child's"
    );
    assert!(
        poi.members(&tg("test:t2")).is_empty(),
        "nothing through the cycle"
    );
    assert_eq!(
        sorted(poi.members(&tg("test:top"))),
        ["test:i", "test:j"],
        "the edge out of the cycle is kept too"
    );
    assert_eq!(poi.parents(&tg("test:t1")), [tg("test:top")]);
}

#[test]
fn a_cycle_through_a_tag_without_a_document_is_reported_for_both() {
    let stack = one_pack(&[
        (
            "poi/tags/a.json",
            &tag_of(
                "test:a",
                r#", "children": ["test:leaf"], "parents": ["test:leaf"]"#,
            ),
        ),
        (
            "poi/blocks/i.json",
            &tagged("test:i", r#"["minecraft:anvil"]"#, r#"["test:leaf"]"#),
        ),
    ]);
    let poi = stack.poi();
    let cycles = found(poi, "ReferenceCycle");
    let mut texts: Vec<String> = cycles.iter().map(|d| d.code.to_string()).collect();
    texts.sort();
    assert_eq!(
        texts,
        [
            "test:a was a part of a cycle",
            "test:leaf was a part of a cycle"
        ]
    );
    assert_eq!(poi.members(&tg("test:leaf")), [ty("test:i")]);
    assert!(poi.members(&tg("test:a")).is_empty());
}

#[test]
fn poi35_parent_child_equivalence() {
    let job = tg("test:job");
    let town = tg("test:town");
    for side in ["a", "b"] {
        let stack = fixture_stack(&format!("poi35_parent_child_equivalence/{side}"));
        let poi = stack.poi();
        clean(poi);
        assert_eq!(poi.parents(&job), std::slice::from_ref(&town), "{side}");
        assert_eq!(poi.children(&town), std::slice::from_ref(&job), "{side}");
        assert!(
            poi.children(&job).is_empty() && poi.parents(&town).is_empty(),
            "{side}"
        );
    }
}

#[test]
fn an_edge_said_from_both_sides_is_one_edge() {
    let stack = one_pack(&[
        (
            "poi/tags/job.json",
            &tag_of("test:job", r#", "parents": ["test:town", "test:town"]"#),
        ),
        (
            "poi/tags/town.json",
            &tag_of("test:town", r#", "children": ["test:job"]"#),
        ),
    ]);
    let poi = stack.poi();
    clean(poi);
    assert_eq!(poi.parents(&tg("test:job")), [tg("test:town")]);
    assert_eq!(poi.children(&tg("test:town")), [tg("test:job")]);
}

#[test]
fn poi36_shadowed_type_the_higher_pack_wins() {
    let stack = fixture_stack("poi36_shadowed_type");
    let poi = stack.poi();
    let forge = poi.types.get(&ty("test:forge")).expect("the type loads");
    assert_eq!(forge.tickets(), 2, "the high-priority pack's");
    assert_eq!(forge.origin().pack.index(), Some(0));
    assert_eq!(poi.types.len(), 1);
    let all: Vec<&Diagnostic> = poi.diagnostics().collect();
    assert_eq!(all.len(), 1, "{}", describe(poi.diagnostics()));
    assert!(matches!(
        all[0].code,
        DiagnosticCode::ShadowedDefinition { .. }
    ));
    assert_eq!(all[0].severity, Severity::Info);
    assert_eq!(
        all[0]
            .location
            .pack
            .as_ref()
            .and_then(bedrock_addon::pack::PackRef::index),
        Some(1),
        "at the definition that lost"
    );
}

#[test]
fn two_definitions_in_one_pack_the_later_file_wins_with_the_same_info() {
    let stack = one_pack(&[
        (
            "poi/blocks/a.json",
            &block_of("test:forge", r#""tickets": 1,"#, r#"["minecraft:anvil"]"#),
        ),
        (
            "poi/blocks/b.json",
            &block_of("test:forge", r#""tickets": 2,"#, r#"["minecraft:anvil"]"#),
        ),
    ]);
    let poi = stack.poi();
    assert_eq!(poi.types.get(&ty("test:forge")).unwrap().tickets(), 2);
    let all: Vec<&Diagnostic> = poi.diagnostics().collect();
    assert_eq!(all.len(), 1, "{}", describe(poi.diagnostics()));
    assert!(matches!(
        all[0].code,
        DiagnosticCode::ShadowedDefinition { .. }
    ));
    assert_eq!(all[0].severity, Severity::Info);
    assert_eq!(file(all[0]), Some("poi/blocks/a.json"));
}

#[test]
fn a_shadowed_types_blocks_and_tags_are_released_deviation() {
    // Vanilla's 1.26.20 registry keeps the earlier registration's blocks mapped to the reused
    // id; ours releases them with the shadowed document.
    let low = pack(
        20,
        &[(
            "poi/blocks/forge.json",
            &tagged("test:forge", r#"["minecraft:anvil"]"#, r#"["test:old"]"#),
        )],
    );
    let top = pack(
        10,
        &[(
            "poi/blocks/forge.json",
            &tagged("test:forge", r#"["minecraft:bell"]"#, r#"["test:new"]"#),
        )],
    );
    let stack = stack(vec![top, low]);
    let poi = stack.poi();
    assert_eq!(poi.type_of_block(&bl("minecraft:anvil")), None, "released");
    assert_eq!(
        poi.type_of_block(&bl("minecraft:bell")),
        Some(&ty("test:forge"))
    );
    assert!(!poi.contains_tag(&tg("test:old")), "its tags go with it");
    assert_eq!(poi.members(&tg("test:new")), [ty("test:forge")]);
}

#[test]
fn poi37_block_claimed_twice_the_later_type_takes_it() {
    let stack = fixture_stack("poi37_block_claimed_twice");
    let poi = stack.poi();
    let anvil = bl("minecraft:anvil");
    assert_eq!(poi.type_of_block(&anvil), Some(&ty("test:b")));
    assert!(poi.types.contains(&ty("test:a")) && poi.types.contains(&ty("test:b")));
    let all: Vec<&Diagnostic> = poi.diagnostics().collect();
    assert_eq!(all.len(), 1, "{}", describe(poi.diagnostics()));
    assert!(matches!(all[0].code, DiagnosticCode::PoiBlockClaimedTwice));
    assert_eq!(all[0].severity, Severity::Info);
    // It names both types: it is at the one that took the block and carries the other.
    assert_eq!(file(all[0]), Some("poi/blocks/b.json"));
    assert_eq!(
        all[0]
            .location
            .pack
            .as_ref()
            .and_then(bedrock_addon::pack::PackRef::index),
        Some(0)
    );
    assert_eq!(
        all[0].location.json_path.to_string(),
        "minecraft:poi_block/blocks/0"
    );
    assert_eq!(referenced(all[0]), Some(("poi type", "test:a")));
}

#[test]
fn a_block_goes_to_the_type_loaded_last_whatever_its_name() {
    let anvil = r#"["minecraft:anvil"]"#;
    // The higher pack is loaded last: its `test:a` takes the block from the lower `test:b`.
    let top = pack(10, &[("poi/blocks/a.json", &block_of("test:a", "", anvil))]);
    let low = pack(20, &[("poi/blocks/b.json", &block_of("test:b", "", anvil))]);
    let stack = stack(vec![top, low]);
    assert_eq!(
        stack.poi().type_of_block(&bl("minecraft:anvil")),
        Some(&ty("test:a"))
    );
    let claimed = found(stack.poi(), "PoiBlockClaimedTwice");
    assert_eq!(claimed.len(), 1);
    assert_eq!(file(claimed[0]), Some("poi/blocks/a.json"));
    assert_eq!(referenced(claimed[0]), Some(("poi type", "test:b")));
    // Inside one pack the later file.
    let stack = one_pack(&[
        ("poi/blocks/1.json", &block_of("test:z", "", anvil)),
        ("poi/blocks/2.json", &block_of("test:y", "", anvil)),
    ]);
    assert_eq!(
        stack.poi().type_of_block(&bl("minecraft:anvil")),
        Some(&ty("test:y"))
    );
}

#[test]
fn an_overriding_definition_claims_its_blocks_when_it_is_loaded() {
    let anvil = r#"["minecraft:anvil"]"#;
    // Low: `a` then `b` (b holds the anvil). High: `a` again, loaded after both.
    let low = pack(
        20,
        &[
            ("poi/blocks/a.json", &block_of("test:a", "", anvil)),
            ("poi/blocks/b.json", &block_of("test:b", "", anvil)),
        ],
    );
    let top = pack(10, &[("poi/blocks/a.json", &block_of("test:a", "", anvil))]);
    let stack = stack(vec![top, low]);
    let poi = stack.poi();
    assert_eq!(
        poi.type_of_block(&bl("minecraft:anvil")),
        Some(&ty("test:a"))
    );
    let claimed = found(poi, "PoiBlockClaimedTwice");
    assert_eq!(claimed.len(), 1, "{}", describe(poi.diagnostics()));
    assert_eq!(
        claimed[0]
            .location
            .pack
            .as_ref()
            .and_then(bedrock_addon::pack::PackRef::index),
        Some(0)
    );
    assert_eq!(referenced(claimed[0]), Some(("poi type", "test:b")));
}

#[test]
fn types_with_different_blocks_claim_nothing_twice() {
    let stack = one_pack(&[
        (
            "poi/blocks/a.json",
            &block_of("test:a", "", r#"["minecraft:anvil", "minecraft:bell"]"#),
        ),
        (
            "poi/blocks/b.json",
            &block_of("test:b", "", r#"["minecraft:lectern"]"#),
        ),
    ]);
    let poi = stack.poi();
    clean(poi);
    assert_eq!(
        poi.type_of_block(&bl("minecraft:bell")),
        Some(&ty("test:a"))
    );
    assert_eq!(
        poi.type_of_block(&bl("minecraft:lectern")),
        Some(&ty("test:b"))
    );
    assert_eq!(poi.type_of_block(&bl("minecraft:barrel")), None);
}

#[test]
fn poi38_tag_union_across_packs() {
    let stack = fixture_stack("poi38_tag_union");
    let poi = stack.poi();
    let t = tg("test:t");
    let document = poi.tags.get(&t).expect("the tag loads");
    assert_eq!(
        document.parents(),
        [tg("test:p2"), tg("test:p1")],
        "both packs' parents, the higher pack's first"
    );
    assert_eq!(sorted(poi.parents(&t)), ["test:p1", "test:p2"]);
    assert_eq!(poi.children(&tg("test:p1")), std::slice::from_ref(&t));
    assert_eq!(poi.children(&tg("test:p2")), [t]);
    clean(poi);
}

#[test]
fn tag_documents_of_one_pack_are_unioned_without_a_finding() {
    let stack = one_pack(&[
        (
            "poi/tags/t_a.json",
            &tag_of(
                "test:t",
                r#", "parents": ["test:p1"], "children": ["test:c1"]"#,
            ),
        ),
        (
            "poi/tags/t_b.json",
            &tag_of(
                "test:t",
                r#", "parents": ["test:p1", "test:p2"], "children": ["test:c2"]"#,
            ),
        ),
        ("poi/tags/p1.json", &tag_of("test:p1", "")),
        ("poi/tags/p2.json", &tag_of("test:p2", "")),
        ("poi/tags/c1.json", &tag_of("test:c1", "")),
        ("poi/tags/c2.json", &tag_of("test:c2", "")),
    ]);
    let poi = stack.poi();
    clean(poi);
    let document = poi.tags.get(&tg("test:t")).unwrap();
    assert_eq!(
        sorted(document.parents()),
        ["test:p1", "test:p2"],
        "duplicates removed"
    );
    assert_eq!(sorted(document.children()), ["test:c1", "test:c2"]);
    assert_eq!(sorted(poi.parents(&tg("test:t"))), ["test:p1", "test:p2"]);
    assert_eq!(sorted(poi.children(&tg("test:t"))), ["test:c1", "test:c2"]);
    assert_eq!(poi.tags.len(), 5);
}

#[test]
fn poi39_inherited_membership() {
    let stack = fixture_stack("poi39_inherited_membership");
    let poi = stack.poi();
    clean(poi);
    let forge = ty("test:forge");
    for tag in ["test:smithy", "test:craft", "test:town"] {
        assert!(poi.has_tag(&forge, &tg(tag)), "{tag}");
        assert_eq!(poi.members(&tg(tag)), std::slice::from_ref(&forge), "{tag}");
    }
    assert!(!poi.has_tag(&forge, &tg("test:other")));
}

#[test]
fn a_type_reached_twice_is_a_member_once() {
    // A diamond: `forge` is in `a` and `b`, both children of `top`, and names `top` itself.
    let stack = one_pack(&[
        (
            "poi/blocks/forge.json",
            &tagged(
                "test:forge",
                r#"["minecraft:anvil"]"#,
                r#"["test:a", "test:b", "test:top", "test:a"]"#,
            ),
        ),
        (
            "poi/tags/a.json",
            &tag_of("test:a", r#", "parents": ["test:top"]"#),
        ),
        (
            "poi/tags/b.json",
            &tag_of("test:b", r#", "parents": ["test:top"]"#),
        ),
        ("poi/tags/top.json", &tag_of("test:top", "")),
    ]);
    let poi = stack.poi();
    assert_eq!(poi.members(&tg("test:top")), [ty("test:forge")]);
    assert_eq!(poi.members(&tg("test:a")), [ty("test:forge")]);
}

#[test]
fn poi40_all_blocks_unknown_loads_the_type_without_a_no_blocks_warning() {
    // The lookup of the block is the link pass's (`link.rs`).
    let stack = fixture_stack("poi40_all_blocks_unknown");
    let poi = stack.poi();
    let forge = poi.types.get(&ty("test:forge")).expect("the type loads");
    assert_eq!(forge.blocks, [bl("test:not_a_block")]);
    assert_eq!(
        found(poi, "PoiNoBlocks").len(),
        0,
        "the document's list was not empty"
    );
    clean(poi);
}

#[test]
fn poi41_cycle_keeps_direct_types() {
    let stack = fixture_stack("poi41_cycle_keeps_direct_types");
    let poi = stack.poi();
    for n in 1..=3 {
        assert_eq!(
            poi.members(&tg(&format!("test:t{n}"))),
            [ty(&format!("test:i{n}"))],
            "test:t{n} holds only its own type"
        );
    }
    let cycles = found(poi, "ReferenceCycle");
    assert_eq!(cycles.len(), 3, "{}", describe(poi.diagnostics()));
    assert_eq!(poi.diagnostics().count(), 3);
    assert!(cycles.iter().all(|d| d.severity == Severity::Info));
}

#[test]
fn the_minecraft_namespace_is_allowed_in_a_user_pack() {
    let stack = fixture_stack("poi03_vanilla_meeting");
    let poi = stack.poi();
    assert!(poi.types.contains(&ty("minecraft:meeting")));
    clean(poi);
    let tag = one_pack(&[("poi/tags/village.json", &tag_of("minecraft:village", ""))]);
    assert!(tag.poi().tags.contains(&tg("minecraft:village")));
    clean(tag.poi());
}

#[test]
fn the_determinism_fragment_is_one_valid_document_per_root_type() {
    let read =
        |name: &str| super::common::fixture(&format!("determinism/behavior_pack/poi/{name}.json"));
    let stack = one_pack(&[
        (
            "poi/blocks/determinism_type.json",
            &read("blocks/determinism_type"),
        ),
        (
            "poi/tags/determinism_tag.json",
            &read("tags/determinism_tag"),
        ),
    ]);
    let poi = stack.poi();
    assert_eq!((poi.types.len(), poi.tags.len()), (1, 1));
    assert_eq!(
        poi.members(&tg("determinism:smithy")),
        [ty("determinism:forge")]
    );
    clean(poi);
}

#[test]
fn rejected_files_are_listed_by_the_group_and_by_their_registry() {
    let stack = one_pack(&[
        (
            "poi/blocks/bad_type.json",
            &super::common::block(r#"{"description": {"identifier": "test:t"}}"#),
        ),
        (
            "poi/tags/bad_tag.json",
            &super::common::tag(r#"{"description": {}}"#),
        ),
        (
            "poi/blocks/neither.json",
            r#"{"format_version": "1.26.40", "minecraft:poi": {}}"#,
        ),
        ("poi/blocks/empty.json", ""),
    ]);
    let poi = stack.poi();
    // A file of `poi/blocks` that holds no type is the type loader's rejection.
    assert_eq!(poi.types.rejected().len(), 3);
    assert_eq!(poi.tags.rejected().len(), 1);
    let mut all: Vec<&str> = poi.rejected().map(|r| r.origin.path.as_str()).collect();
    all.sort_unstable();
    assert_eq!(
        all,
        [
            "poi/blocks/bad_type.json",
            "poi/blocks/empty.json",
            "poi/blocks/neither.json",
            "poi/tags/bad_tag.json"
        ]
    );
}
