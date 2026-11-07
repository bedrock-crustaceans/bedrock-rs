//! The validation rules that judge one document (E1–E6, B1–B11, T1–T5), each pinned at
//! both ends of its range and on each half of its condition, and the constructors.

use bedrock_addon::diagnostic::codes;
use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Kind, Severity};
use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::poi::{PoiTag, PoiType};
use bedrock_addon::id::{BlockId, PoiTagId, PoiTypeId};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::version::ENGINE_VERSION;

use super::common::{
    block, block_document, block_of, context, context_with, describe, is_closing, loads, names,
    rejected, tag, tag_of,
};

/// The errors of `findings` as `(code, JSON path)`.
fn errors_at(findings: &Diagnostics) -> Vec<(&'static str, String)> {
    findings
        .iter()
        .filter(|d| d.severity == Severity::Error && !is_closing(d))
        .map(|d| (d.code.id(), d.location.json_path.to_string()))
        .collect()
}

/// Requires `text` to be rejected as a type with exactly the error `code` at `path`.
#[track_caller]
fn type_error(text: &str, code: &str, path: &str) {
    let d = rejected::<PoiType>(text);
    assert_eq!(errors_at(&d), [(code, path.to_owned())], "{text}");
}

/// Requires `text` to be rejected as a tag with exactly the error `code` at `path`.
#[track_caller]
fn tag_error(text: &str, code: &str, path: &str) {
    let d = rejected::<PoiTag>(text);
    assert_eq!(errors_at(&d), [(code, path.to_owned())], "{text}");
}

/// A type document with `member` (`"key": value`) beside the required members.
fn forge_with(member: &str) -> String {
    block_of(
        "test:forge",
        &format!("{member},"),
        r#"["minecraft:anvil"]"#,
    )
}

/// A minimal type payload.
const FORGE: &str =
    r#"{"description": {"identifier": "test:forge"}, "blocks": ["minecraft:anvil"]}"#;

// ---- E1–E6: envelope and version -------------------------------------------------------------

#[test]
fn e1_the_floor_is_1_26_40_for_both_documents() {
    for version in ["1.26.39", "1.26.0", "1.0.0", "1.26.40-beta.1"] {
        let d = rejected::<PoiType>(&block_document(version, FORGE));
        let errors: Vec<&Diagnostic> = d
            .iter()
            .filter(|f| f.severity == Severity::Error && !is_closing(f))
            .collect();
        assert_eq!(errors.len(), 1, "{version}: {}", describe(d.iter()));
        assert!(
            errors[0].code == codes::NO_PARSER_FOR_VERSION.with([version]),
            "{version}: {:?}",
            errors[0].code
        );
    }
    let loaded = loads::<PoiType>(&block_document("1.26.40", FORGE));
    assert!(
        loaded.diagnostics().is_empty(),
        "the floor itself loads cleanly"
    );
    // The tag document has the same floor.
    let old_tag = tag_of("test:civic", "").replace("1.26.40", "1.26.39");
    let d = rejected::<PoiTag>(&old_tag);
    assert_eq!(
        d.iter()
            .find(|f| f.severity == Severity::Error && !is_closing(f))
            .map(|f| f.code.to_string())
            .as_deref(),
        Some("unsupported version: no parser available for version 1.26.39")
    );
    loads::<PoiTag>(&tag_of("test:civic", ""));
}

#[test]
fn e2_the_ceiling_is_the_engine_version() {
    let engine = ENGINE_VERSION.to_string();
    let loaded = loads::<PoiType>(&block_document(&engine, FORGE));
    assert!(
        loaded.diagnostics().is_empty(),
        "the engine version itself loads"
    );
    assert_eq!(loaded.version().written, Some(ENGINE_VERSION));
    assert!(!loaded.version().upgraded, "one struct, no upgrade");
    let above = format!(
        "{}.{}.{}",
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 1
    );
    let d = rejected::<PoiType>(&block_document(&above, FORGE));
    assert!(
        d.iter()
            .any(|f| f.code == codes::NO_PARSER_FOR_VERSION.with([above.clone()])),
        "{}",
        describe(d.iter())
    );
    let above_tag = tag_of("test:civic", "").replace("1.26.40", &above);
    assert_eq!(
        names(rejected::<PoiTag>(&above_tag).iter(), Severity::Error),
        ["core/no_parser_for_version"]
    );
}

#[test]
fn e3_a_tag_without_format_version_is_rejected() {
    let d = rejected::<PoiTag>(
        r#"{"minecraft:poi_tag": {"description": {"identifier": "test:civic"}}}"#,
    );
    let errors: Vec<&Diagnostic> = d
        .iter()
        .filter(|f| f.severity == Severity::Error && !is_closing(f))
        .collect();
    assert_eq!(errors.len(), 1, "{}", describe(d.iter()));
    assert!(errors[0].code.is(&codes::NO_VALID_VERSION));
}

#[test]
fn e4_a_malformed_format_version_is_rejected() {
    for version in [r#""abc""#, "true", "{}", "null", "[1, 26]", r#""""#] {
        let text = format!(r#"{{"format_version": {version}, "minecraft:poi_block": {FORGE}}}"#);
        let d = rejected::<PoiType>(&text);
        let errors: Vec<&Diagnostic> = d
            .iter()
            .filter(|f| f.severity == Severity::Error && !is_closing(f))
            .collect();
        assert_eq!(errors.len(), 1, "{version}: {}", describe(d.iter()));
        assert!(
            // By legacy name: the variant and the definitions that were it.
            matches!(errors[0].code.id(), "core/malformed_format_version"),
            "{version}: {:?}",
            errors[0].code
        );
        assert_eq!(
            errors[0].code.to_string(),
            "Malformed format version",
            "{version}"
        );
    }
    // The array form is a version.
    loads::<PoiType>(&format!(
        r#"{{"format_version": [1, 26, 40], "minecraft:poi_block": {FORGE}}}"#
    ));
}

/// The game's document input reads a number as its text: `5` is
/// 5.0.0, above the engine, and `1.26` is 1.26.0, below the floor; both are versions without a
/// slice, not malformed ones. The "no parser" text prints the version as parsed.
#[test]
fn e4_a_number_is_read_as_its_text() {
    for (version, text) in [("5", "5.0.0"), ("1.26", "1.26.0")] {
        let json = format!(r#"{{"format_version": {version}, "minecraft:poi_block": {FORGE}}}"#);
        let d = rejected::<PoiType>(&json);
        let errors: Vec<&Diagnostic> = d
            .iter()
            .filter(|f| f.severity == Severity::Error && !is_closing(f))
            .collect();
        assert_eq!(errors.len(), 1, "{version}: {}", describe(d.iter()));
        assert!(
            // By legacy name: the variant and the definitions that were it.
            matches!(errors[0].code.id(), "core/no_parser_for_version"),
            "{version}: {:?}",
            errors[0].code
        );
        assert_eq!(errors[0].location.json_path.to_string(), "format_version");
        assert_eq!(
            errors[0].code.to_string(),
            format!("unsupported version: no parser available for version {text}"),
        );
    }
}

#[test]
fn e5_a_tag_with_a_third_root_member_is_rejected() {
    let text = r#"{"format_version": "1.26.40", "minecraft:poi_tag": {"description": {"identifier": "test:civic"}}, "more": {}}"#;
    let d = rejected::<PoiTag>(text);
    assert!(
        d.iter().any(|f| f.code.is(&codes::DOCUMENT_MEMBER_COUNT)),
        "{}",
        describe(d.iter())
    );
}

#[test]
fn e6_each_document_type_takes_only_its_own_payload_key() {
    let d = rejected::<PoiTag>(&block(FORGE));
    assert_eq!(
        names(d.iter(), Severity::Error),
        ["core/payload_under_other_key"]
    );
    let d = rejected::<PoiType>(&tag_of("test:civic", ""));
    assert_eq!(
        names(d.iter(), Severity::Error),
        ["core/payload_under_other_key"]
    );
}

#[test]
fn the_beta_literal_has_no_struct() {
    // No beta struct: no parser for `beta`, with or without the toggle.
    let d = rejected::<PoiType>(&block_document("beta", FORGE));
    assert_eq!(
        names(d.iter(), Severity::Error),
        ["core/no_parser_for_version"]
    );
    let mut options = super::common::poi_options();
    options.beta_apis = true;
    let text = block_document("beta", FORGE);
    let d = super::common::load_with::<PoiType>(&text, &context_with(&options, false))
        .expect_err("no beta struct")
        .diagnostics;
    assert_eq!(
        names(d.iter(), Severity::Error),
        ["core/no_parser_for_version"]
    );
}

#[test]
fn an_empty_or_malformed_file_is_rejected() {
    assert_eq!(
        names(rejected::<PoiType>("").iter(), Severity::Error),
        ["core/empty_document"]
    );
    assert_eq!(
        names(rejected::<PoiTag>(r#""a string""#).iter(), Severity::Error),
        ["core/document_json"]
    );
    let mut deep = String::new();
    for _ in 0..200 {
        deep.push('[');
    }
    assert_eq!(
        names(rejected::<PoiType>(&deep).iter(), Severity::Error),
        ["core/syntax"]
    );
    let d = PoiType::parse(
        &bedrock_addon::storage::PackPath::new("poi/latin1.json").unwrap(),
        b"{\"format_version\": \"1.26.40\", \"minecraft:poi_block\": \xff}",
        &context(),
    )
    .expect_err("not UTF-8")
    .diagnostics;
    assert_eq!(names(d.iter(), Severity::Error), ["core/syntax"]);
}

#[test]
fn a_byte_order_mark_and_comments_are_read_through() {
    let text = format!("\u{feff}// a type\n{}", block(FORGE));
    let forge = loads::<PoiType>(&text);
    assert_eq!(
        forge.description.identifier,
        PoiTypeId::from_static("test:forge")
    );
}

// ---- B1–B11: minecraft:poi_block -------------------------------------------------------------

#[test]
fn b1_description_is_required() {
    let d = rejected::<PoiType>(&block(r#"{"blocks": ["minecraft:anvil"]}"#));
    assert_eq!(
        names(d.iter(), Severity::Error),
        ["core/missing_required_field"]
    );
}

#[test]
fn b2_identifier_is_required() {
    let d = rejected::<PoiType>(&block(
        r#"{"description": {}, "blocks": ["minecraft:anvil"]}"#,
    ));
    assert_eq!(
        names(d.iter(), Severity::Error),
        ["core/missing_required_field"]
    );
    assert!(
        d.iter()
            .next()
            .unwrap()
            .location
            .json_path
            .to_string()
            .starts_with("minecraft:poi_block/description")
    );
}

/// Identifiers `^\w+:\w+$` takes.
const GOOD_NAMES: &[&str] = &[
    "test:forge",
    "Test_1:Forge_2",
    "a:b",
    "_:_",
    "9:9",
    "minecraft:oak_log",
];
/// Identifiers it refuses.
const BAD_NAMES: &[&str] = &[
    "",
    "forge",
    "test:",
    ":forge",
    "a:b:c",
    "te st:forge",
    "test:for ge",
    "test:forge-2",
    "te-st:forge",
    "test:forge.x",
    "te.st:forge",
    "test:forge/x",
    "t\u{e9}st:forge",
    "test:forg\u{e9}",
    "test:forge\n",
    " test:forge",
];

#[test]
fn b3_the_identifier_is_word_colon_word() {
    for name in GOOD_NAMES {
        let loaded = loads::<PoiType>(&block_of(name, "", r#"["minecraft:anvil"]"#));
        assert_eq!(loaded.description.identifier.as_str(), *name);
    }
    for name in BAD_NAMES {
        let text = block_of(&name.replace('\n', "\\n"), "", r#"["minecraft:anvil"]"#);
        type_error(
            &text,
            "core/invalid_value",
            "minecraft:poi_block/description/identifier",
        );
    }
}

#[test]
fn b3_an_identifier_that_is_not_a_string_is_a_type_error() {
    for value in ["5", "true", "null", "[]", "{}"] {
        let text = block(&format!(
            r#"{{"description": {{"identifier": {value}}}, "blocks": ["minecraft:anvil"]}}"#
        ));
        let d = rejected::<PoiType>(&text);
        assert_eq!(
            names(d.iter(), Severity::Error),
            ["core/invalid_type"],
            "{value}"
        );
    }
}

#[test]
fn b4_blocks_must_be_an_array() {
    for value in [r#""minecraft:anvil""#, "5", "{}", "null"] {
        let text = block(&format!(
            r#"{{"description": {{"identifier": "test:forge"}}, "blocks": {value}}}"#
        ));
        let d = rejected::<PoiType>(&text);
        assert_eq!(
            names(d.iter(), Severity::Error),
            ["core/invalid_type"],
            "{value}"
        );
    }
}

#[test]
fn b5_every_block_is_a_string_matching_the_pattern() {
    for value in ["5", "true", "null", r#"["minecraft:anvil"]"#, "{}"] {
        let text = block_of(
            "test:forge",
            "",
            &format!(r#"["minecraft:anvil", {value}]"#),
        );
        let d = rejected::<PoiType>(&text);
        assert_eq!(
            names(d.iter(), Severity::Error),
            ["core/invalid_type"],
            "{value}"
        );
    }
    for name in GOOD_NAMES {
        loads::<PoiType>(&block_of("test:forge", "", &format!(r#"["{name}"]"#)));
    }
    for name in BAD_NAMES {
        let name = name.replace('\n', "\\n");
        let text = block_of(
            "test:forge",
            "",
            &format!(r#"["minecraft:anvil", "{name}"]"#),
        );
        type_error(&text, "core/invalid_value", "minecraft:poi_block/blocks/1");
    }
}

#[test]
fn b6_every_tag_is_a_string_matching_the_pattern() {
    for value in ["5", "false", "null", "{}"] {
        let d = rejected::<PoiType>(&forge_with(&format!(r#""tags": [{value}]"#)));
        assert_eq!(
            names(d.iter(), Severity::Error),
            ["core/invalid_type"],
            "{value}"
        );
    }
    for value in [r#""test:civic""#, "null", "5", "{}"] {
        type_error(
            &forge_with(&format!(r#""tags": {value}"#)),
            "core/invalid_type",
            "minecraft:poi_block/tags",
        );
    }
    for name in GOOD_NAMES {
        let loaded = loads::<PoiType>(&forge_with(&format!(r#""tags": ["{name}"]"#)));
        assert_eq!(loaded.tags(), [PoiTagId::parse(*name)]);
    }
    for name in BAD_NAMES {
        let name = name.replace('\n', "\\n");
        type_error(
            &forge_with(&format!(r#""tags": ["test:civic", "{name}"]"#)),
            "core/invalid_value",
            "minecraft:poi_block/tags/1",
        );
    }
}

#[test]
fn b7_tickets_is_an_integer_from_0_to_255() {
    for (value, expected) in [("0", 0), ("1", 1), ("255", 255)] {
        let loaded = loads::<PoiType>(&forge_with(&format!(r#""tickets": {value}"#)));
        assert_eq!(loaded.tickets(), expected);
        assert_eq!(loaded.tickets, Some(expected));
        assert!(loaded.diagnostics().is_empty());
    }
    for value in ["256", "-1", "1000", "-256", "4294967296"] {
        type_error(
            &forge_with(&format!(r#""tickets": {value}"#)),
            "core/invalid_value",
            "minecraft:poi_block/tickets",
        );
    }
    for value in ["1.5", r#""3""#, "true", "null", "[1]", "{}"] {
        type_error(
            &forge_with(&format!(r#""tickets": {value}"#)),
            "core/invalid_type",
            "minecraft:poi_block/tickets",
        );
    }
}

#[test]
fn b8_usable_distance_is_an_integer_from_0_to_255() {
    for (value, expected) in [("0", 0), ("1", 1), ("255", 255)] {
        let loaded = loads::<PoiType>(&forge_with(&format!(r#""usable_distance": {value}"#)));
        assert_eq!(loaded.usable_distance(), expected);
        assert_eq!(loaded.usable_distance, Some(expected));
        assert!(loaded.diagnostics().is_empty());
    }
    for value in ["256", "300", "-1"] {
        type_error(
            &forge_with(&format!(r#""usable_distance": {value}"#)),
            "core/invalid_value",
            "minecraft:poi_block/usable_distance",
        );
    }
    for value in ["2.5", r#""1""#, "false", "null"] {
        type_error(
            &forge_with(&format!(r#""usable_distance": {value}"#)),
            "core/invalid_type",
            "minecraft:poi_block/usable_distance",
        );
    }
}

#[test]
fn b9_an_unknown_member_anywhere_rejects_the_type() {
    type_error(
        &block(
            r#"{"description": {"identifier": "test:forge", "name": "x"}, "blocks": ["minecraft:anvil"]}"#,
        ),
        "core/unrecognized_field",
        "minecraft:poi_block/description/name",
    );
    // A member named like nothing the struct has, `extra` included.
    type_error(
        &forge_with(r#""extra": 1"#),
        "core/unrecognized_field",
        "minecraft:poi_block/extra",
    );
    type_error(
        &forge_with(r#""children": []"#),
        "core/unrecognized_field",
        "minecraft:poi_block/children",
    );
    // Two unknown members are two errors.
    let d = rejected::<PoiType>(&forge_with(r#""radius": 2, "range": 3"#));
    assert_eq!(
        errors_at(&d),
        [
            (
                "core/unrecognized_field",
                "minecraft:poi_block/radius".to_owned()
            ),
            (
                "core/unrecognized_field",
                "minecraft:poi_block/range".to_owned()
            )
        ]
    );
}

#[test]
fn b10_one_block_is_enough() {
    let loaded = loads::<PoiType>(&block(FORGE));
    assert_eq!(
        names(loaded.diagnostics().iter(), Severity::Warning),
        [] as [&str; 0]
    );
}

#[test]
fn b11_each_repeat_of_a_block_is_reported_and_removed() {
    let loaded = loads::<PoiType>(&block_of(
        "test:forge",
        "",
        r#"["minecraft:bell", "minecraft:anvil", "minecraft:bell", "minecraft:bell", "minecraft:anvil"]"#,
    ));
    assert_eq!(
        loaded.blocks,
        [
            BlockId::from_static("minecraft:bell"),
            BlockId::from_static("minecraft:anvil")
        ],
        "the first of each stays, in order"
    );
    let infos: Vec<String> = loaded
        .diagnostics()
        .iter()
        .map(|d| {
            assert!(
                // By legacy name: the variant and the definitions that were it.
                matches!(d.code.id(), "core/no_effect"),
                "{:?}",
                d.code
            );
            assert_eq!(d.severity, Severity::Info);
            d.location.json_path.to_string()
        })
        .collect();
    assert_eq!(
        infos,
        [
            "minecraft:poi_block/blocks/2",
            "minecraft:poi_block/blocks/3",
            "minecraft:poi_block/blocks/4"
        ]
    );
    let distinct = loads::<PoiType>(&block_of(
        "test:forge",
        "",
        r#"["minecraft:bell", "minecraft:anvil"]"#,
    ));
    assert!(distinct.diagnostics().is_empty());
    assert_eq!(distinct.blocks.len(), 2);
}

#[test]
fn every_failed_pattern_of_a_document_is_reported() {
    let d = rejected::<PoiType>(&block(
        r#"{"description": {"identifier": "forge"}, "blocks": ["anvil", "minecraft:bell", "b-ell"], "tags": ["Village"]}"#,
    ));
    assert_eq!(
        errors_at(&d),
        [
            (
                "core/invalid_value",
                "minecraft:poi_block/description/identifier".to_owned()
            ),
            (
                "core/invalid_value",
                "minecraft:poi_block/blocks/0".to_owned()
            ),
            (
                "core/invalid_value",
                "minecraft:poi_block/blocks/2".to_owned()
            ),
            (
                "core/invalid_value",
                "minecraft:poi_block/tags/0".to_owned()
            ),
        ]
    );
}

#[test]
fn an_empty_tags_list_keeps_its_presence() {
    let loaded = loads::<PoiType>(&forge_with(r#""tags": []"#));
    assert_eq!(loaded.tags, Some(Vec::new()));
    assert!(loaded.tags().is_empty());
    assert!(loaded.diagnostics().is_empty());
}

// ---- T1–T5: minecraft:poi_tag ----------------------------------------------------------------

#[test]
fn t1_description_and_identifier_are_required() {
    let d = rejected::<PoiTag>(&tag("{}"));
    assert_eq!(
        names(d.iter(), Severity::Error),
        ["core/missing_required_field"]
    );
    let d = rejected::<PoiTag>(&tag(r#"{"description": {}}"#));
    assert_eq!(
        names(d.iter(), Severity::Error),
        ["core/missing_required_field"]
    );
    let d = rejected::<PoiTag>(&tag(r#"{"parents": []}"#));
    assert_eq!(
        names(d.iter(), Severity::Error),
        ["core/missing_required_field"]
    );
}

#[test]
fn t2_the_tag_identifier_is_word_colon_word() {
    for name in GOOD_NAMES {
        let loaded = loads::<PoiTag>(&tag_of(name, ""));
        assert_eq!(loaded.description.identifier.as_str(), *name);
    }
    for name in BAD_NAMES {
        tag_error(
            &tag_of(&name.replace('\n', "\\n"), ""),
            "core/invalid_value",
            "minecraft:poi_tag/description/identifier",
        );
    }
    let d = rejected::<PoiTag>(&tag(r#"{"description": {"identifier": 5}}"#));
    assert_eq!(names(d.iter(), Severity::Error), ["core/invalid_type"]);
}

#[test]
fn t3_every_parent_and_child_is_a_string_matching_the_pattern() {
    for member in ["parents", "children"] {
        for value in ["5", "true", "null", "{}"] {
            let d = rejected::<PoiTag>(&tag_of(
                "test:civic",
                &format!(r#", "{member}": [{value}]"#),
            ));
            assert_eq!(
                names(d.iter(), Severity::Error),
                ["core/invalid_type"],
                "{member} {value}"
            );
        }
        for value in [r#""test:a""#, "null", "5", "{}"] {
            tag_error(
                &tag_of("test:civic", &format!(r#", "{member}": {value}"#)),
                "core/invalid_type",
                &format!("minecraft:poi_tag/{member}"),
            );
        }
        for name in GOOD_NAMES {
            loads::<PoiTag>(&tag_of(
                "test:civic",
                &format!(r#", "{member}": ["{name}"]"#),
            ));
        }
        for name in BAD_NAMES {
            let name = name.replace('\n', "\\n");
            tag_error(
                &tag_of(
                    "test:civic",
                    &format!(r#", "{member}": ["test:a", "{name}"]"#),
                ),
                "core/invalid_value",
                &format!("minecraft:poi_tag/{member}/1"),
            );
        }
    }
}

#[test]
fn t4_an_unknown_member_anywhere_rejects_the_tag() {
    tag_error(
        &tag_of("test:civic", r#", "blocks": []"#),
        "core/unrecognized_field",
        "minecraft:poi_tag/blocks",
    );
    tag_error(
        &tag(r#"{"description": {"identifier": "test:civic", "note": 1}}"#),
        "core/unrecognized_field",
        "minecraft:poi_tag/description/note",
    );
    tag_error(
        &tag_of("test:civic", r#", "extra": {}"#),
        "core/unrecognized_field",
        "minecraft:poi_tag/extra",
    );
}

#[test]
fn t5_a_tag_naming_itself_loses_that_edge_only() {
    // As a child.
    let a = loads::<PoiTag>(&tag_of("test:a", r#", "children": ["test:a"]"#));
    assert!(a.children().is_empty());
    assert_eq!(
        a.children,
        Some(Vec::new()),
        "the member the author wrote stays"
    );
    let warnings: Vec<&Diagnostic> = a.diagnostics().iter().collect();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].code.id(), "poi/part_of_a_cycle");
    assert_eq!(warnings[0].severity, Severity::Info);
    assert_eq!(
        warnings[0].location.json_path.to_string(),
        "minecraft:poi_tag/children/0"
    );
    // Among others, on both sides: the other entries stay, each self-entry is reported where
    // it was written.
    let a = loads::<PoiTag>(&tag_of(
        "test:a",
        r#", "children": ["test:c", "test:a"], "parents": ["test:b", "test:a", "test:d"]"#,
    ));
    assert_eq!(a.children(), [PoiTagId::from_static("test:c")]);
    assert_eq!(
        a.parents(),
        [
            PoiTagId::from_static("test:b"),
            PoiTagId::from_static("test:d")
        ]
    );
    let at: Vec<String> = a
        .diagnostics()
        .iter()
        .map(|d| {
            assert_eq!(d.code.id(), "poi/part_of_a_cycle");
            assert_eq!(d.code.to_string(), "test:a was a part of a cycle");
            d.location.json_path.to_string()
        })
        .collect();
    assert_eq!(
        at,
        [
            "minecraft:poi_tag/children/1",
            "minecraft:poi_tag/parents/1"
        ]
    );
    // Naming others is no cycle of one.
    let clean = loads::<PoiTag>(&tag_of(
        "test:a",
        r#", "parents": ["test:b"], "children": ["test:c"]"#,
    ));
    assert!(clean.diagnostics().is_empty());
}

// ---- constructors, defaults and placement ----------------------------------------------------

#[test]
fn a_new_type_has_the_documented_defaults() {
    let forge = PoiType::new(
        PoiTypeId::from_static("test:forge"),
        vec![BlockId::from_static("minecraft:anvil")],
    );
    assert_eq!(forge.tickets(), 0);
    assert_eq!(forge.usable_distance(), 1);
    assert!(forge.tags().is_empty());
    assert_eq!(
        (forge.tickets, forge.usable_distance, forge.tags.clone()),
        (None, None, None)
    );
    let set = PoiType {
        tickets: Some(7),
        usable_distance: Some(0),
        tags: Some(vec![PoiTagId::from_static("test:civic")]),
        ..forge
    };
    assert_eq!((set.tickets(), set.usable_distance()), (7, 0));
    assert_eq!(set.tags(), [PoiTagId::from_static("test:civic")]);
}

#[test]
fn a_new_tag_has_no_edges() {
    let civic = PoiTag::new(PoiTagId::from_static("test:civic"));
    assert!(civic.parents().is_empty() && civic.children().is_empty());
    assert_eq!(
        (civic.parents.clone(), civic.children.clone()),
        (None, None)
    );
    let wired = PoiTag {
        parents: Some(vec![PoiTagId::from_static("test:town")]),
        children: Some(vec![PoiTagId::from_static("test:job")]),
        ..civic
    };
    assert_eq!(wired.parents(), [PoiTagId::from_static("test:town")]);
    assert_eq!(wired.children(), [PoiTagId::from_static("test:job")]);
}

#[test]
fn types_and_tags_are_placed_directly_in_their_own_directories() {
    // The walks read no sub-folder: `<directory>/<namespace>_<name>.json`, and
    // vanilla's own `<directory>/<name>.json`.
    let forge = PoiType::new(PoiTypeId::from_static("test:forge"), Vec::new());
    assert_eq!(forge.default_path().as_str(), "poi/blocks/test_forge.json");
    let tag = PoiTag::new(PoiTagId::from_static("test:forge"));
    assert_eq!(tag.default_path().as_str(), "poi/tags/test_forge.json");
    let cleric = PoiType::new(PoiTypeId::from_static("minecraft:cleric"), Vec::new());
    assert_eq!(cleric.default_path().as_str(), "poi/blocks/cleric.json");
}

#[test]
fn validate_runs_the_rules_on_a_built_value() {
    let cx = context();
    let good = PoiType::new(
        PoiTypeId::from_static("test:forge"),
        vec![BlockId::from_static("minecraft:anvil")],
    );
    assert!(good.validate(&cx).is_empty());
    let empty = PoiType::new(PoiTypeId::from_static("test:forge"), Vec::new());
    assert_eq!(
        names(empty.validate(&cx).iter(), Severity::Info),
        ["core/poi_no_blocks"]
    );
    let bad = PoiType::new(
        PoiTypeId::parse("forge"),
        vec![BlockId::parse("minecraft:oak-log")],
    );
    assert_eq!(
        errors_at(&bad.validate(&cx)),
        [
            (
                "core/invalid_value",
                "minecraft:poi_block/description/identifier".to_owned()
            ),
            (
                "core/invalid_value",
                "minecraft:poi_block/blocks/0".to_owned()
            ),
        ]
    );
    let looped = PoiTag {
        parents: Some(vec![PoiTagId::from_static("test:a")]),
        ..PoiTag::new(PoiTagId::from_static("test:a"))
    };
    assert_eq!(
        names(looped.validate(&cx).iter(), Severity::Info),
        ["poi/part_of_a_cycle"]
    );
    assert!(
        PoiTag::new(PoiTagId::from_static("test:a"))
            .validate(&cx)
            .is_empty()
    );
    // Without the experiment the value would not load.
    let off = context_with(&LoadOptions::default(), false);
    assert_eq!(
        names(good.validate(&off).iter(), Severity::Info),
        ["core/missing_toggles"]
    );
}

// ---- the closing line -----------------------------------------------------------------------

/// The closing error a failed file of either loader ends with (the type loader
/// and the tag loader both use the one string).
#[track_caller]
fn closes_with_failed_to_load(d: &Diagnostics) {
    let last = d.iter().last().expect("a finding");
    assert!(
        // By legacy name: the variant and the definitions that were it.
        matches!(last.code.id(), "poi/failed_to_load"),
        "{}",
        describe(d.iter())
    );
    assert_eq!(last.severity, Severity::Error);
    assert_eq!(last.location.json_path.to_string(), "");
    assert_eq!(last.code.to_string(), "Failed to load POI Type");
    let closing = d
        .iter()
        // By legacy name: the variant and the definitions that were it.
        .filter(|f| matches!(f.code.id(), "poi/failed_to_load"))
        .count();
    assert_eq!(closing, 1, "one closing line");
}

#[test]
fn a_type_file_that_fails_closes_with_failed_to_load_poi_type() {
    closes_with_failed_to_load(&rejected::<PoiType>(&forge_with(r#""radius": 2"#)));
    closes_with_failed_to_load(&rejected::<PoiType>(&block_document("1.26.39", FORGE)));
    closes_with_failed_to_load(&rejected::<PoiType>(r#"{"minecraft:poi_block": {}}"#));
}

#[test]
fn a_tag_file_that_fails_closes_with_the_same_type_text() {
    closes_with_failed_to_load(&rejected::<PoiTag>(&tag_of("test:civic", r#", "x": 1"#)));
    closes_with_failed_to_load(&rejected::<PoiTag>(&tag_of("Civic", "")));
}

#[test]
fn a_file_that_loads_or_is_gated_has_no_closing_line() {
    let loaded = loads::<PoiType>(&block(FORGE));
    assert!(
        !loaded
            .diagnostics()
            .iter()
            // By kind: the definitions that were `DocumentRejected` and their neighbours.
            .any(|f| f.code.kind() == Kind::Closing)
    );
    let off = context_with(&LoadOptions::default(), false);
    let gated = super::common::load_with::<PoiType>(&block(FORGE), &off).expect_err("gated");
    assert!(
        !gated
            .diagnostics
            .iter()
            // By kind: the definitions that were `DocumentRejected` and their neighbours.
            .any(|f| f.code.kind() == Kind::Closing)
    );
}
