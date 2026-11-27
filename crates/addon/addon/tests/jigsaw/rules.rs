//! The rules that no fixture row covers on its own: the
//! envelope cases, the structural errors, pool-name collisions
//! (rule 11), an empty metadata version (rule 20) and the metadata precedence (rule 23).

use bedrock_addon::diagnostic::codes;
use std::sync::Arc;

use bedrock_addon::diagnostic::{Diagnostics, Kind, Severity};
use bedrock_addon::families::jigsaw::{JigsawStructure, ProcessorList, StructureSet, TemplatePool};
use bedrock_addon::load::{BuiltIns, LoadOptions};
use bedrock_addon::pack::{Behavior, PackStack};
use bedrock_addon::storage::PackPath;

use super::common::{document, load, loads, names, pack, rejected};

/// A structure payload with `extra` members added (JSON text after the required ones).
fn structure(extra: &str) -> String {
    format!(
        r#"{{"description": {{"identifier": "demo:a"}}, "step": "lakes", "start_pool": "demo:p", "max_depth": 1, "start_height": {{"type": "constant", "value": {{"absolute": 0}}}}{extra}}}"#
    )
}

/// The path every structure here is loaded at.
const PATH: &str = "worldgen/structures/a.json";

/// The code names of the errors.
fn errors(d: &Diagnostics) -> Vec<&'static str> {
    names(d, Severity::Error)
}

/// The finding of `d` with `code`, and its JSON path.
fn at(d: &Diagnostics, code: &str) -> String {
    d.iter()
        .find(|f| f.code.id() == code)
        .unwrap_or_else(|| panic!("no {code}: {d:?}"))
        .location
        .json_path
        .to_string()
}

#[test]
fn rule_3_a_version_that_is_no_version_is_malformed() {
    // A number is a version on the document input (`1` is 1.0.0: `input.rs`).
    for version in [r#""invalid""#, "true", "null", "[1, 21]", r#"{"a": 1}"#] {
        let text = format!(
            r#"{{"format_version": {version}, "minecraft:jigsaw": {}}}"#,
            structure("")
        );
        let d = rejected::<JigsawStructure>(PATH, &text);
        assert_eq!(
            errors(&d),
            ["core/malformed_format_version", "jigsaw/structure_failed"],
            "{version}"
        );
    }
}

#[test]
fn rule_3_an_array_of_three_integers_is_a_version() {
    let text = format!(
        r#"{{"format_version": [1, 21, 20], "minecraft:jigsaw": {}}}"#,
        structure("")
    );
    let loaded = loads::<JigsawStructure>(PATH, &text);
    assert!(loaded.diagnostics().is_empty());
}

#[test]
fn rule_3_a_third_root_member_rejects_the_document() {
    let text = format!(
        r#"{{"format_version": "1.21.20", "minecraft:jigsaw": {}, "demo": 1}}"#,
        structure("")
    );
    let d = rejected::<JigsawStructure>(PATH, &text);
    assert_eq!(
        errors(&d),
        ["core/document_member_count", "jigsaw/structure_failed"]
    );
    assert!(d.iter().any(|f| f.code.is(&codes::DOCUMENT_MEMBER_COUNT)));
}

#[test]
fn rule_5_an_unreadable_file_gets_the_family_line() {
    let d = match load::<JigsawStructure>(PATH, "\u{0}") {
        Ok(_) => panic!("loaded"),
        Err(r) => r.diagnostics,
    };
    assert_eq!(errors(&d).last(), Some(&"jigsaw/structure_failed"));
    let bytes = [0xff_u8, 0xfe, 0x00];
    let d = <JigsawStructure as bedrock_addon::document::Content>::parse(
        &PackPath::new(PATH).unwrap(),
        &bytes,
        &super::common::context(),
    )
    .expect_err("not UTF-8")
    .diagnostics;
    assert_eq!(errors(&d), ["core/syntax", "jigsaw/structure_failed"]);
}

#[test]
fn structural_unknown_enum_names_are_invalid_enum_values() {
    let cases = [
        (
            structure("").replace(r#""lakes""#, r#""nope""#),
            "minecraft:jigsaw/step",
        ),
        (
            structure(r#", "terrain_adaptation": "Bury""#),
            "minecraft:jigsaw/terrain_adaptation",
        ),
        (
            structure(r#", "heightmap_projection": "surface""#),
            "minecraft:jigsaw/heightmap_projection",
        ),
        (
            structure(r#", "liquid_settings": "none""#),
            "minecraft:jigsaw/liquid_settings",
        ),
        (
            structure("").replace(r#""constant""#, r#""exponential""#),
            "minecraft:jigsaw/start_height",
        ),
    ];
    for (payload, path) in cases {
        let d =
            rejected::<JigsawStructure>(PATH, &document("1.21.20", "minecraft:jigsaw", &payload));
        assert_eq!(
            errors(&d),
            ["core/invalid_enum_value", "jigsaw/structure_failed"],
            "{path}"
        );
        assert_eq!(at(&d, "core/invalid_enum_value"), path);
    }
}

#[test]
fn structural_wrong_json_types_are_invalid_types() {
    for (extra, path) in [
        (
            r#", "dimension_padding": "7""#,
            "minecraft:jigsaw/dimension_padding",
        ),
        (
            r#", "max_distance_from_center": 7.5"#,
            "minecraft:jigsaw/max_distance_from_center",
        ),
        (
            r#", "start_jigsaw_name": 3"#,
            "minecraft:jigsaw/start_jigsaw_name",
        ),
    ] {
        let d = rejected::<JigsawStructure>(
            PATH,
            &document("1.21.20", "minecraft:jigsaw", &structure(extra)),
        );
        assert_eq!(
            errors(&d),
            ["core/invalid_type", "jigsaw/structure_failed"],
            "{extra}"
        );
        assert_eq!(at(&d, "core/invalid_type"), path);
    }
    let d = rejected::<JigsawStructure>(
        PATH,
        &document(
            "1.21.20",
            "minecraft:jigsaw",
            &structure("").replace(r#""max_depth": 1"#, r#""max_depth": "1""#),
        ),
    );
    assert_eq!(errors(&d), ["core/invalid_type", "jigsaw/structure_failed"]);
}

#[test]
fn structural_numbers_outside_the_storage_type_reject_the_file() {
    // `weight` is a u8, anchors an i16 (the storage type's integer range).
    let pool = r#"{"description": {"identifier": "demo:p"}, "elements": [{"element": {"element_type": "minecraft:empty_pool_element"}, "weight": 300}]}"#;
    let d = rejected::<TemplatePool>(
        "worldgen/template_pools/p.json",
        &document("1.21.20", "minecraft:template_pool", pool),
    );
    assert_eq!(errors(&d), ["core/invalid_type", "jigsaw/pool_failed"]);
    let d = rejected::<JigsawStructure>(
        PATH,
        &document(
            "1.21.20",
            "minecraft:jigsaw",
            &structure("").replace(r#""absolute": 0"#, r#""absolute": 40000"#),
        ),
    );
    assert_eq!(errors(&d), ["core/invalid_type", "jigsaw/structure_failed"]);
}

#[test]
fn structural_missing_members_are_missing_required_fields() {
    for (payload, path) in [
        (
            structure("").replace(r#""step": "lakes", "#, ""),
            "minecraft:jigsaw/step",
        ),
        (
            structure("").replace(r#""start_pool": "demo:p", "#, ""),
            "minecraft:jigsaw/start_pool",
        ),
        (
            structure(r#", "pool_aliases": [{"alias": "x"}]"#),
            "minecraft:jigsaw/pool_aliases/0/target",
        ),
    ] {
        let d =
            rejected::<JigsawStructure>(PATH, &document("1.21.20", "minecraft:jigsaw", &payload));
        assert_eq!(
            errors(&d),
            ["core/missing_required_field", "jigsaw/structure_failed"],
            "{path}"
        );
        assert_eq!(at(&d, "core/missing_required_field"), path);
    }
}

#[test]
fn structural_a_limit_object_whose_type_names_the_other_form() {
    let list = r#"{"description": {"identifier": "demo:l"}, "processors": [{"processor_type": "minecraft:capped", "limit": {"type": "uniform", "value": 2}, "delegate": {"processor_type": "minecraft:rule", "rules": []}}]}"#;
    let d = rejected::<ProcessorList>(
        "worldgen/processors/l.json",
        &document("1.21.20", "minecraft:processor_list", list),
    );
    assert_eq!(
        errors(&d),
        ["core/invalid_enum_value", "jigsaw/processors_failed"]
    );
    assert_eq!(
        at(&d, "core/invalid_enum_value"),
        "minecraft:processor_list/processors/0/limit/type"
    );
    let string = list.replace(r#"{"type": "uniform", "value": 2}"#, r#""2""#);
    let d = rejected::<ProcessorList>(
        "worldgen/processors/l.json",
        &document("1.21.20", "minecraft:processor_list", &string),
    );
    assert_eq!(
        errors(&d),
        ["core/invalid_type", "jigsaw/processors_failed"]
    );
}

#[test]
fn structural_file_references_hold_at_most_256_bytes() {
    let long = "a".repeat(257);
    let pool = format!(
        r#"{{"description": {{"identifier": "demo:p"}}, "elements": [{{"element": {{"element_type": "minecraft:single_pool_element", "location": "{long}"}}}}]}}"#
    );
    let d = rejected::<TemplatePool>(
        "worldgen/template_pools/p.json",
        &document("1.21.20", "minecraft:template_pool", &pool),
    );
    assert_eq!(errors(&d), ["core/invalid_value", "jigsaw/pool_failed"]);
    let ok = pool.replace(&long, &"a".repeat(256));
    loads::<TemplatePool>(
        "worldgen/template_pools/p.json",
        &document("1.21.20", "minecraft:template_pool", &ok),
    );
}

#[test]
fn structural_shared_members_report_their_unknown_members() {
    let list = r#"{"description": {"identifier": "demo:l"}, "processors": [{"processor_type": "minecraft:rule", "rules": [{"output_state": {"name": "minecraft:stone", "demo": 1}}]}]}"#;
    let loaded = loads::<ProcessorList>(
        "worldgen/processors/l.json",
        &document("1.21.20", "minecraft:processor_list", list),
    );
    assert_eq!(
        names(loaded.diagnostics(), Severity::Warning),
        ["core/unrecognized_field"]
    );
    assert_eq!(
        at(loaded.diagnostics(), "core/unrecognized_field"),
        "minecraft:processor_list/processors/0/rules/0/output_state/demo"
    );
}

/// Every template pool name of the built-in set.
struct BuiltInPools;

impl BuiltIns for BuiltInPools {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        kind == "template pool" && id == "minecraft:village/plains/houses"
    }
}

#[test]
fn rule_11_pool_names_collide_after_lowercasing_or_with_a_built_in() {
    let file = |name: &str, id: &str| {
        (
            format!("worldgen/template_pools/{name}.json"),
            document(
                "1.21.20",
                "minecraft:template_pool",
                &format!(r#"{{"description": {{"identifier": "{id}"}}, "elements": []}}"#),
            )
            .into_bytes(),
        )
    };
    let options = LoadOptions {
        built_ins: Some(Arc::new(BuiltInPools)),
        ..LoadOptions::default()
    };
    let stack = PackStack::<Behavior>::of(
        [pack(
            10,
            vec![
                file("a", "demo:Ruin"),
                file("b", "demo:ruin"),
                file("c", "minecraft:village/plains/houses"),
            ],
        )],
        options,
    )
    .unwrap();
    let found: Vec<(String, String)> = stack
        .jigsaw()
        .diagnostics()
        .filter(|d| d.code.id() == "jigsaw/pool_name_registered")
        .map(|d| {
            assert_eq!(d.severity, Severity::Info);
            (
                d.location.path.as_ref().unwrap().as_str().to_owned(),
                d.code.to_string(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            (
                "worldgen/template_pools/b.json".to_owned(),
                "Jigsaw pool name `demo:ruin` already registered".to_owned()
            ),
            (
                "worldgen/template_pools/c.json".to_owned(),
                "Jigsaw pool name `minecraft:village/plains/houses` already registered".to_owned()
            ),
        ]
    );
    assert_eq!(stack.jigsaw().pools.len(), 3, "every pool stays loaded");
}

#[test]
fn rule_20_an_empty_leading_string_has_no_version() {
    let d = <bedrock_addon::families::jigsaw::JigsawMetadata as bedrock_addon::document::Content>::parse(
        &PackPath::new("worldgen/metadata/a.dat").unwrap(),
        &[0, 1, 2],
        &super::common::context(),
    )
    .expect_err("no version")
    .diagnostics;
    assert_eq!(errors(&d), ["core/no_valid_version"]);
    let d = <bedrock_addon::families::jigsaw::JigsawMetadata as bedrock_addon::document::Content>::parse(
        &PackPath::new("worldgen/metadata/a.dat").unwrap(),
        &[9, b'1'],
        &super::common::context(),
    )
    .expect_err("the stream ends inside the string")
    .diagnostics;
    assert_eq!(errors(&d), ["jigsaw/bad_format_version_string"]);
}

#[test]
fn rule_23_the_highest_pack_metadata_file_wins() {
    let dat = |version: &str| {
        let mut bytes = vec![u8::try_from(version.len()).unwrap()];
        bytes.extend_from_slice(version.as_bytes());
        bytes
    };
    let top = pack(
        10,
        vec![("worldgen/metadata/m.dat".to_owned(), dat("1.21.90"))],
    );
    let low = pack(
        20,
        vec![("worldgen/metadata/m.dat".to_owned(), dat("1.21.50"))],
    );
    let stack = super::common::stack(vec![top, low]);
    let file = stack
        .jigsaw()
        .metadata
        .get(&PackPath::new("worldgen/metadata/m.dat").unwrap())
        .expect("the file");
    assert_eq!(file.format_version, "1.21.90");
}

/// A structure set with `separation` and `spacing`.
fn spaced_set(separation: u16, spacing: u16) -> String {
    format!(
        r#"{{"description": {{"identifier": "minecraft:trail_ruins"}}, "placement": {{"type": "minecraft:random_spread", "salt": 1, "separation": {separation}, "spacing": {spacing}}}, "structures": [{{"structure": "minecraft:trail_ruins", "weight": 1}}]}}"#
    )
}

#[test]
fn rule_9_spacing_equal_to_twice_the_separation_is_rejected() {
    // `cmp spacing, separation, lsl #1; b.ls`: spacing <= 2 x separation fails.
    let d = rejected::<StructureSet>(
        "worldgen/structure_sets/s.json",
        &document("1.21.20", "minecraft:structure_set", &spaced_set(4, 8)),
    );
    assert_eq!(errors(&d), ["jigsaw/spacing_separation"]);
    assert_eq!(
        d.iter()
            .find(|f| f.code.id() == "jigsaw/spacing_separation")
            .unwrap()
            .code
            .to_string(),
        "Invalid spacing and separation combination for structure set 'minecraft:trail_ruins'. Value 'separation' must be less than half of 'spacing'."
    );
}

#[test]
fn rule_9_spacing_above_twice_the_separation_is_accepted() {
    let loaded = loads::<StructureSet>(
        "worldgen/structure_sets/s.json",
        &document("1.21.20", "minecraft:structure_set", &spaced_set(4, 9)),
    );
    assert!(
        loaded.diagnostics().is_empty(),
        "{:?}",
        loaded.diagnostics()
    );
}

#[test]
fn a_third_root_member_below_1_21_20_gets_the_member_count_text_deviation() {
    // Vanilla checks the member count from 1.21.20 only and answers "no parser
    // available" below it; ours checks it at every version. The file is skipped either way.
    let text = format!(
        r#"{{"format_version": "1.21.10", "minecraft:jigsaw": {}, "demo": 1}}"#,
        structure("")
    );
    let d = rejected::<JigsawStructure>(PATH, &text);
    assert_eq!(
        errors(&d),
        ["core/document_member_count", "jigsaw/structure_failed"]
    );
    assert!(d.iter().any(|f| f.code.is(&codes::DOCUMENT_MEMBER_COUNT)));
    assert!(
        !d.iter().any(|f| f.kind() == Kind::UnsupportedVersion),
        "{d:?}"
    );
}

/// A stack of structure-set files at one path, top first: `(separation, spacing)` of each,
/// `None` for a file whose payload does not parse.
fn set_stack(sets: &[Option<(u16, u16)>]) -> PackStack<Behavior> {
    let packs = sets
        .iter()
        .enumerate()
        .map(|(i, set)| {
            let payload = match set {
                Some((separation, spacing)) => spaced_set(*separation, *spacing),
                None => r#"{"description": {"identifier": "minecraft:trail_ruins"}}"#.to_owned(),
            };
            let text = document("1.21.20", "minecraft:structure_set", &payload);
            pack(
                10 * (u128::try_from(i).unwrap() + 1),
                vec![(
                    "worldgen/structure_sets/s.json".to_owned(),
                    text.into_bytes(),
                )],
            )
        })
        .collect();
    super::common::stack(packs)
}

/// The pack index and spacing of the effective `minecraft:trail_ruins` set.
fn effective_set(stack: &PackStack<Behavior>) -> Option<(Option<usize>, u16)> {
    let id = bedrock_addon::id::Id::parse("minecraft:trail_ruins");
    let set = stack.jigsaw().structure_sets.get(&id)?;
    Some((set.origin().pack.index(), set.placement.spacing))
}

/// The pack indexes of the findings named `code` at `severity`.
fn packs_of(stack: &PackStack<Behavior>, code: &str, severity: Severity) -> Vec<Option<usize>> {
    stack
        .jigsaw()
        .diagnostics()
        .filter(|d| d.code.id() == code && d.severity == severity)
        .map(|d| {
            d.location
                .pack
                .as_ref()
                .and_then(bedrock_addon::pack::PackRef::index)
        })
        .collect()
}

// The game compares the new document's `description.identifier` byte for byte with every
// document already in the list and drops a match silently; the registration checks run
// after all packs are loaded, and a set that fails them is not
// recorded. So the top pack's set holds the identifier when it fails rule 9: the lower pack's
// set was dropped before the check ran.
#[test]
fn rule_9_two_packs_a_set_refused_at_registration_keeps_the_lower_set_out() {
    let stack = set_stack(&[Some((4, 8)), Some((4, 9))]);
    assert_eq!(effective_set(&stack), None);
    assert_eq!(
        packs_of(&stack, "jigsaw/spacing_separation", Severity::Error),
        [Some(0)],
        "rule 9 is reported on the top pack's set"
    );
    assert_eq!(
        packs_of(&stack, "core/shadowed_definition", Severity::Info),
        [Some(1)],
        "the lower set is reported as shadowed (ours; vanilla drops it silently)"
    );
}

#[test]
fn rule_9_three_packs_a_set_refused_at_registration_keeps_both_lower_sets_out() {
    let stack = set_stack(&[Some((4, 8)), Some((4, 10)), Some((4, 12))]);
    assert_eq!(effective_set(&stack), None);
    assert_eq!(
        packs_of(&stack, "jigsaw/spacing_separation", Severity::Error),
        [Some(0)]
    );
    assert_eq!(
        packs_of(&stack, "core/shadowed_definition", Severity::Info),
        [Some(1), Some(2)]
    );
    // A refused set in the middle changes nothing for the top pack's set.
    let stack = set_stack(&[Some((4, 9)), Some((4, 8)), Some((4, 12))]);
    assert_eq!(effective_set(&stack), Some((Some(0), 9)));
}

// A file that fails to parse is "skipped": its document never reaches the
// vector, so the next pack's document of that identifier is appended and registered.
#[test]
fn a_set_that_fails_to_parse_leaves_the_next_pack_s_set_effective() {
    let stack = set_stack(&[None, Some((4, 9))]);
    assert_eq!(effective_set(&stack), Some((Some(1), 9)));
    let stack = set_stack(&[None, Some((4, 10)), Some((4, 12))]);
    assert_eq!(effective_set(&stack), Some((Some(1), 10)));
    // Below a set refused at registration, a file that does not parse is still skipped, and
    // the set below it is still kept out.
    let stack = set_stack(&[Some((4, 8)), None, Some((4, 12))]);
    assert_eq!(effective_set(&stack), None);
}
