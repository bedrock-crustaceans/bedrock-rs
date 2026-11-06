//! The single-document rows of the fixtures (poi01–poi21; poi11 and poi22 need a stack and are
//! in `stack.rs`), each loaded from its fixture file with the `poi` experiment on.

use bedrock_addon::diagnostic::codes;
use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Severity};
use bedrock_addon::families::poi::{PoiDescription, PoiTag, PoiType};
use bedrock_addon::id::{BlockId, PoiTagId, PoiTypeId};
use bedrock_addon::pack::PackOrigin;
use bedrock_addon::version::ENGINE_VERSION;

use super::common::{
    context_with, describe, fixture, fixture_packs_from, fixture_stack, is_closing, load_with,
    loads, names, poi_options, rejected, stack,
};

/// The one error of `findings`.
#[track_caller]
fn only_error(findings: &Diagnostics) -> &Diagnostic {
    let errors: Vec<&Diagnostic> = findings
        .iter()
        .filter(|d| d.severity == Severity::Error && !is_closing(d))
        .collect();
    assert_eq!(errors.len(), 1, "one error: {}", describe(findings.iter()));
    errors[0]
}

#[test]
fn poi01_block_minimal_loads_with_the_defaults() {
    let forge = loads::<PoiType>(&fixture("poi01_block_minimal/poi/blocks/forge.json"));
    assert_eq!(
        forge.description.identifier,
        PoiTypeId::from_static("test:forge")
    );
    assert_eq!(forge.blocks, [BlockId::from_static("minecraft:anvil")]);
    assert_eq!(forge.tickets(), 0, "tickets defaults to 0");
    assert_eq!(forge.usable_distance(), 1, "usable_distance defaults to 1");
    assert!(forge.tags().is_empty(), "tags defaults to none");
    assert_eq!(
        (forge.tickets, forge.usable_distance, forge.tags.clone()),
        (None, None, None),
        "the members the file leaves out stay absent"
    );
    assert!(
        forge.diagnostics().is_empty(),
        "{}",
        describe(forge.diagnostics().iter())
    );
}

#[test]
fn poi02_block_full_loads_with_every_member_as_given() {
    let hall = loads::<PoiType>(&fixture("poi02_block_full/poi/blocks/guild_hall.json"));
    assert_eq!(
        *hall.value(),
        PoiType {
            description: PoiDescription {
                identifier: PoiTypeId::from_static("test:guild_hall"),
            },
            tickets: Some(3),
            usable_distance: Some(4),
            blocks: vec![
                BlockId::from_static("minecraft:lectern"),
                BlockId::from_static("minecraft:bell"),
            ],
            tags: Some(vec![PoiTagId::from_static("test:civic")]),
        }
    );
    assert_eq!(hall.tickets(), 3);
    assert_eq!(hall.usable_distance(), 4);
    assert_eq!(hall.tags(), [PoiTagId::from_static("test:civic")]);
    assert!(hall.diagnostics().is_empty());
}

#[test]
fn poi03_vanilla_meeting_loads_from_a_base_game_pack() {
    // The single-file load with a base-game context.
    let text = fixture("poi03_vanilla_meeting/poi/blocks/meeting.json");
    let meeting = load_with::<PoiType>(&text, &context_with(&poi_options(), true))
        .unwrap_or_else(|r| panic!("rejected: {}", describe(r.diagnostics.iter())));
    assert_eq!(meeting.tickets(), 32);
    assert_eq!(meeting.usable_distance(), 6);
    assert_eq!(meeting.blocks, [BlockId::from_static("minecraft:bell")]);
    assert_eq!(meeting.tags(), [PoiTagId::from_static("minecraft:village")]);
    // And through a base-game pack of a stack.
    let stack = stack(fixture_packs_from(
        "poi03_vanilla_meeting",
        PackOrigin::Package,
    ));
    let poi = stack.poi();
    let id = PoiTypeId::from_static("minecraft:meeting");
    let loaded = poi.types.get(&id).expect("minecraft:meeting loads");
    assert_eq!((loaded.tickets(), loaded.usable_distance()), (32, 6));
    assert_eq!(
        poi.diagnostics().count(),
        0,
        "{}",
        describe(poi.diagnostics())
    );
}

#[test]
fn poi04_tag_minimal_loads_without_parents_or_children() {
    let civic = loads::<PoiTag>(&fixture("poi04_tag_minimal/poi/tags/civic.json"));
    assert_eq!(
        civic.description.identifier,
        PoiTagId::from_static("test:civic")
    );
    assert!(civic.parents().is_empty(), "parents defaults to none");
    assert!(civic.children().is_empty(), "children defaults to none");
    assert_eq!(
        (civic.parents.clone(), civic.children.clone()),
        (None, None),
        "the members the file leaves out stay absent"
    );
    assert!(civic.diagnostics().is_empty());
}

#[test]
fn poi05_tag_full_loads_with_its_parent() {
    let sites = loads::<PoiTag>(&fixture("poi05_tag_full/poi/tags/acquirable_job_site.json"));
    assert_eq!(
        sites.description.identifier,
        PoiTagId::from_static("minecraft:acquirable_job_site")
    );
    assert_eq!(
        sites.parents(),
        [PoiTagId::from_static("minecraft:village")]
    );
    assert!(sites.children().is_empty());
    assert!(sites.diagnostics().is_empty());
}

#[test]
fn poi06_below_floor_has_no_parser() {
    let d = rejected::<PoiType>(&fixture("poi06_below_floor/poi/blocks/forge.json"));
    let error = only_error(&d);
    assert!(
        error.code == DiagnosticCode::from(codes::NO_PARSER_FOR_VERSION.with(["1.26.30"])),
        "{:?}",
        error.code
    );
    assert_eq!(
        error.code.to_string(),
        "unsupported version: no parser available for version 1.26.30"
    );
    // Through a stack the file is not loaded.
    let stack = fixture_stack("poi06_below_floor");
    assert_eq!(stack.poi().types.len(), 0);
    assert_eq!(stack.poi().types.rejected().len(), 1);
}

#[test]
fn poi07_above_engine_has_no_parser() {
    let above = format!(
        "{}.{}.{}",
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 10
    );
    let text = fixture("poi07_above_engine/poi/blocks/forge.json");
    assert!(
        text.contains(&format!(r#""format_version": "{above}""#)),
        "the fixture is at ENGINE + 0.0.10"
    );
    let d = rejected::<PoiType>(&text);
    let error = only_error(&d);
    assert!(
        error.code == DiagnosticCode::from(codes::NO_PARSER_FOR_VERSION.with([above.clone()])),
        "{:?}",
        error.code
    );
}

#[test]
fn poi08_missing_version_is_rejected_with_the_puv_text() {
    let d = rejected::<PoiType>(&fixture("poi08_missing_version/poi/blocks/forge.json"));
    let error = only_error(&d);
    assert!(error.code.is(&codes::NO_VALID_VERSION), "{:?}", error.code);
    assert_eq!(
        error.code.to_string(),
        "no valid version passed by the caller or found in the input"
    );
}

#[test]
fn poi09_third_root_member_is_rejected() {
    let d = rejected::<PoiType>(&fixture("poi09_third_root_member/poi/blocks/forge.json"));
    let error = only_error(&d);
    assert!(
        error.code.is(&codes::DOCUMENT_MEMBER_COUNT),
        "{:?}",
        error.code
    );
}

#[test]
fn poi10_unknown_payload_is_a_missing_payload_error() {
    let text = fixture("poi10_unknown_payload/poi/blocks/forge.json");
    for d in [rejected::<PoiType>(&text), rejected::<PoiTag>(&text)] {
        let error = only_error(&d);
        assert!(
            // By legacy name: the variant and the definitions that were it.
            matches!(error.code.name(), "MissingPayload"),
            "{:?}",
            error.code
        );
    }
    // Through a stack the file is neither kind: skipped with the error at the file.
    let stack = fixture_stack("poi10_unknown_payload");
    let poi = stack.poi();
    assert_eq!((poi.types.len(), poi.tags.len()), (0, 0));
    let skipped: Vec<_> = poi.rejected().collect();
    assert_eq!(skipped.len(), 1, "the file is rejected once");
    assert_eq!(skipped[0].origin.path.as_str(), "poi/blocks/forge.json");
    let errors: Vec<&Diagnostic> = poi
        .diagnostics()
        .filter(|d| d.severity == Severity::Error && !is_closing(d))
        .collect();
    assert_eq!(errors.len(), 1, "{}", describe(poi.diagnostics()));
    // By legacy name: the variant and the definitions that were it.
    assert!(matches!(errors[0].code.name(), "MissingPayload"));
    assert_eq!(
        errors[0]
            .location
            .path
            .as_ref()
            .map(bedrock_addon::storage::PackPath::as_str),
        Some("poi/blocks/forge.json")
    );
}

#[test]
fn poi12_missing_blocks_is_rejected() {
    let d = rejected::<PoiType>(&fixture("poi12_missing_blocks/poi/blocks/forge.json"));
    let error = only_error(&d);
    assert!(
        // By legacy name: the variant and the definitions that were it.
        matches!(error.code.name(), "MissingRequiredField"),
        "{:?}",
        error.code
    );
    assert!(
        error
            .location
            .json_path
            .to_string()
            .starts_with("minecraft:poi_block"),
        "{}",
        error.location.json_path
    );
}

#[test]
fn poi13_bad_identifier_is_rejected_for_each_spelling() {
    for file in ["no_namespace", "hyphen"] {
        let d = rejected::<PoiType>(&fixture(&format!(
            "poi13_bad_identifier/poi/blocks/{file}.json"
        )));
        let error = only_error(&d);
        assert!(
            // By legacy name: the variant and the definitions that were it.
            matches!(error.code.name(), "InvalidValue"),
            "{file}: {:?}",
            error.code
        );
        assert_eq!(
            error.location.json_path.to_string(),
            "minecraft:poi_block/description/identifier",
            "{file}"
        );
    }
}

#[test]
fn poi14_bad_block_name_is_a_pattern_error() {
    let d = rejected::<PoiType>(&fixture("poi14_bad_block_name/poi/blocks/forge.json"));
    let error = only_error(&d);
    assert!(
        // By legacy name: the variant and the definitions that were it.
        matches!(error.code.name(), "InvalidValue"),
        "{:?}",
        error.code
    );
    assert_eq!(
        error.location.json_path.to_string(),
        "minecraft:poi_block/blocks/0"
    );
}

#[test]
fn poi15_tickets_range_rejects_256_minus_1_and_1_5() {
    for (file, code) in [
        ("over", "InvalidValue"),
        ("negative", "InvalidValue"),
        ("fraction", "InvalidType"),
    ] {
        let d = rejected::<PoiType>(&fixture(&format!(
            "poi15_tickets_range/poi/blocks/{file}.json"
        )));
        let error = only_error(&d);
        assert_eq!(error.code.name(), code, "{file}");
        assert_eq!(
            error.location.json_path.to_string(),
            "minecraft:poi_block/tickets",
            "{file}"
        );
    }
}

#[test]
fn poi16_usable_distance_range_rejects_300() {
    let d = rejected::<PoiType>(&fixture(
        "poi16_usable_distance_range/poi/blocks/forge.json",
    ));
    let error = only_error(&d);
    assert!(
        // By legacy name: the variant and the definitions that were it.
        matches!(error.code.name(), "InvalidValue"),
        "{:?}",
        error.code
    );
    assert_eq!(
        error.location.json_path.to_string(),
        "minecraft:poi_block/usable_distance"
    );
}

#[test]
fn poi17_unknown_member_is_an_error() {
    let d = rejected::<PoiType>(&fixture("poi17_unknown_member/poi/blocks/forge.json"));
    let error = only_error(&d);
    assert!(
        // By legacy name: the variant and the definitions that were it.
        matches!(error.code.name(), "UnrecognizedField"),
        "{:?}",
        error.code
    );
    assert_eq!(
        error.location.json_path.to_string(),
        "minecraft:poi_block/radius"
    );
}

#[test]
fn poi18_empty_blocks_loads_with_an_information() {
    let forge = loads::<PoiType>(&fixture("poi18_empty_blocks/poi/blocks/forge.json"));
    assert!(forge.blocks.is_empty());
    // ours, an information
    assert_eq!(
        names(forge.diagnostics().iter(), Severity::Info),
        ["PoiNoBlocks"]
    );
    assert_eq!(forge.diagnostics().iter().count(), 1);
    let warning = forge.diagnostics().iter().next().unwrap();
    // By legacy name: the variant and the definitions that were it.
    assert!(matches!(warning.code.name(), "PoiNoBlocks"));
    assert_eq!(
        warning.location.json_path.to_string(),
        "minecraft:poi_block/blocks"
    );
}

#[test]
fn poi19_duplicate_block_in_list_loads_with_one_entry() {
    let forge = loads::<PoiType>(&fixture(
        "poi19_duplicate_block_in_list/poi/blocks/forge.json",
    ));
    assert_eq!(forge.blocks, [BlockId::from_static("minecraft:bell")]);
    assert_eq!(forge.diagnostics().iter().count(), 1);
    let info = forge.diagnostics().iter().next().unwrap();
    assert!(
        // By legacy name: the variant and the definitions that were it.
        matches!(info.code.name(), "NoEffect"),
        "{:?}",
        info.code
    );
    assert_eq!(info.severity, Severity::Info);
    assert_eq!(
        info.location.json_path.to_string(),
        "minecraft:poi_block/blocks/1",
        "at the repeated entry"
    );
}

#[test]
fn poi20_tag_bad_parent_is_a_pattern_error() {
    let d = rejected::<PoiTag>(&fixture("poi20_tag_bad_parent/poi/tags/tag.json"));
    let error = only_error(&d);
    assert!(
        // By legacy name: the variant and the definitions that were it.
        matches!(error.code.name(), "InvalidValue"),
        "{:?}",
        error.code
    );
    assert_eq!(
        error.location.json_path.to_string(),
        "minecraft:poi_tag/parents/0"
    );
}

#[test]
fn poi21_tag_self_parent_loads_without_the_edge() {
    let a = loads::<PoiTag>(&fixture("poi21_tag_self_parent/poi/tags/a.json"));
    assert!(a.parents().is_empty(), "the edge to itself is dropped");
    assert_eq!(a.diagnostics().iter().count(), 1);
    let warning = a.diagnostics().iter().next().unwrap();
    assert_eq!(warning.code.name(), "ReferenceCycle");
    assert_eq!(warning.severity, Severity::Info);
    assert_eq!(warning.code.to_string(), "test:a was a part of a cycle");
    assert_eq!(
        warning.location.json_path.to_string(),
        "minecraft:poi_tag/parents/0"
    );
    // In a stack the tag exists and has no parent.
    let stack = fixture_stack("poi21_tag_self_parent");
    let id = PoiTagId::from_static("test:a");
    assert!(stack.poi().contains_tag(&id));
    assert!(stack.poi().parents(&id).is_empty());
}
