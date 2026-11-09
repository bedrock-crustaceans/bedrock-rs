//! The fixture rows about one file: aa01–aa17 (aa04b's lookup half
//! is in `stack.rs`).

#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it; not yet moved to the door"
)]

use bedrock_addon::families::aim_assist::{AimAssistPreset, ExclusionSettings};
use bedrock_addon::id::{AimAssistCategoryId, AimAssistPresetId, BlockId, EntityId, ItemId};
use bedrock_addon::version::FormatVersion;

use super::common::*;

const NOT_IN_SCHEMA: &str = "this member was found in the input, but is not present in the Schema";
const NO_VERSION: &str = "no valid version passed by the caller or found in the input";

fn at_preset(path: &str) -> String {
    format!("minecraft:aim_assist_preset/{path}")
}

fn at_category(path: &str) -> String {
    format!("minecraft:aim_assist_categories/categories/{path}")
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_owned()).collect()
}

#[test]
fn aa01_a_1_21_50_exclusion_list_becomes_block_and_entity_exclusions() {
    let document = loads(preset(&preset_json(
        "1.21.50",
        r#""identifier": "test:p", "exclusion_list": ["minecraft:stone", "minecraft:pig", "minecraft:stone"],
        "liquid_targeting_list": ["minecraft:bucket"],
        "item_settings": {"minecraft:iron_sword": "test:melee"},
        "default_item_settings": "test:default", "hand_settings": "test:empty_hand""#,
    )));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert!(document.version().upgraded);
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 21, 50));
    let mut expected = AimAssistPreset::new(AimAssistPresetId::parse("test:p"));
    // Both lists, in the original order, duplicates kept; tags and families empty.
    expected.exclusion_settings = Some(ExclusionSettings {
        blocks: Some(
            ["minecraft:stone", "minecraft:pig", "minecraft:stone"]
                .map(BlockId::parse)
                .to_vec(),
        ),
        entities: Some(
            ["minecraft:stone", "minecraft:pig", "minecraft:stone"]
                .map(EntityId::parse)
                .to_vec(),
        ),
        block_tags: Some(Vec::new()),
        entity_type_families: Some(Vec::new()),
    });
    expected.liquid_targeting_list = Some(vec![ItemId::parse("minecraft:bucket")]);
    expected.item_settings = Some(
        [(
            "minecraft:iron_sword".to_owned(),
            AimAssistCategoryId::parse("test:melee"),
        )]
        .into_iter()
        .collect(),
    );
    expected.default_item_settings = Some(AimAssistCategoryId::parse("test:default"));
    expected.hand_settings = Some(AimAssistCategoryId::parse("test:empty_hand"));
    assert_eq!(document.value(), &expected);
    // The newer member is unknown to the older struct, and the older one to the newer.
    let d = rejects(preset(&preset_json(
        "1.21.110",
        r#""identifier": "test:p", "exclusion_settings": {}"#,
    )));
    assert_eq!(
        errors(&d),
        [finding(
            "core/unrecognized_field_worded",
            &at_preset("exclusion_settings"),
            NOT_IN_SCHEMA
        )]
    );
    let d = rejects(preset(&preset_json(
        "1.21.120",
        r#""identifier": "test:p", "exclusion_list": []"#,
    )));
    assert_eq!(
        errors(&d),
        [finding(
            "core/unrecognized_field_worded",
            &at_preset("exclusion_list"),
            NOT_IN_SCHEMA
        )]
    );
}

#[test]
fn aa02_a_1_21_120_preset_is_read_as_written() {
    let document = loads(preset(&preset_json(
        "1.21.120",
        r#""identifier": "test:p",
        "exclusion_settings": {"blocks": ["minecraft:stone"], "entities": ["minecraft:pig"],
            "block_tags": ["wood"], "entity_type_families": ["monster", "undead"]},
        "hand_settings": "test:empty_hand""#,
    )));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert!(!document.version().upgraded);
    let mut expected = AimAssistPreset::new(AimAssistPresetId::parse("test:p"));
    expected.exclusion_settings = Some(ExclusionSettings {
        blocks: Some(vec![BlockId::parse("minecraft:stone")]),
        entities: Some(vec![EntityId::parse("minecraft:pig")]),
        block_tags: Some(strings(&["wood"])),
        entity_type_families: Some(strings(&["monster", "undead"])),
    });
    expected.hand_settings = Some(AimAssistCategoryId::parse("test:empty_hand"));
    assert_eq!(document.value(), &expected);
    // Absent members stay absent: nothing is invented on the way back.
    let bare = loads(preset(&preset_json(
        "1.21.120",
        r#""identifier": "test:p", "exclusion_settings": {"blocks": []}"#,
    )));
    let settings = bare.exclusion_settings.as_ref().unwrap();
    assert_eq!(settings.blocks, Some(Vec::new()));
    assert_eq!(
        (settings.entities.as_ref(), settings.block_tags.as_ref()),
        (None, None)
    );
    let written = bare.to_json().unwrap();
    assert_eq!(
        written,
        "{\n  \"format_version\": \"1.21.120\",\n  \"minecraft:aim_assist_preset\": {\n    \"identifier\": \"test:p\",\n    \"exclusion_settings\": {\n      \"blocks\": []\n    }\n  }\n}\n"
    );
    // The run-time defaults are the accessors', never the document's.
    assert_eq!(bare.default_item_settings, None);
    assert_eq!(bare.default_item_settings().as_str(), "minecraft:default");
    assert_eq!(bare.hand_settings().as_str(), "minecraft:empty_hand");
    assert_eq!(document.hand_settings().as_str(), "test:empty_hand");
}

#[test]
fn aa03_below_1_21_50_nothing_loads() {
    for version in ["1.21.40", "1.0.0"] {
        let expected = [finding(
            "core/no_parser_for_version",
            "format_version",
            &format!("unsupported version: no parser available for version {version}"),
        )];
        let d = rejects(preset(&preset_json(version, r#""identifier": "test:p""#)));
        assert_eq!(errors(&d), expected, "{version}");
        let d = rejects(categories(&categories_json(version, "")));
        assert_eq!(errors(&d), expected, "{version}");
    }
    loads(preset(&preset_json("1.21.50", r#""identifier": "test:p""#)));
    loads(categories(&categories_json("1.21.50", "")));
}

#[test]
fn aa04_an_unknown_member_fails_the_file() {
    let d = rejects(preset(&preset_json(
        "1.21.120",
        r#""identifier": "test:p", "foo": 1"#,
    )));
    assert_eq!(
        errors(&d),
        [finding(
            "core/unrecognized_field_worded",
            &at_preset("foo"),
            NOT_IN_SCHEMA
        )]
    );
    let d = rejects(preset(&preset_json(
        "1.21.120",
        r#""identifier": "test:p", "exclusion_settings": {"items": []}"#,
    )));
    assert_eq!(
        errors(&d),
        [finding(
            "core/unrecognized_field_worded",
            &at_preset("exclusion_settings/items"),
            NOT_IN_SCHEMA
        )]
    );
    let d = rejects(categories(&categories_json(
        "1.21.50",
        &category("test:c", r#""items": {}"#),
    )));
    assert_eq!(
        errors(&d),
        [finding(
            "core/unrecognized_field_worded",
            &at_category("0/priorities/items"),
            NOT_IN_SCHEMA
        )]
    );
    let text = categories_json("1.21.50", "")
        .replace(r#""categories": []"#, r#""categories": [], "foo": 1"#);
    let d = rejects(categories(&text));
    assert_eq!(
        errors(&d),
        [finding(
            "core/unrecognized_field_worded",
            "minecraft:aim_assist_categories/foo",
            NOT_IN_SCHEMA
        )]
    );
}

#[test]
fn aa04b_reference_texts_are_kept_as_written() {
    let document = loads(preset(&preset_json(
        "1.21.120",
        r#""identifier": "test:p", "item_settings": {"minecraft:WOOL:3": "test:c"},
        "liquid_targeting_list": ["minecraft:Bucket:1"],
        "exclusion_settings": {"entities": ["minecraft:pig<minecraft:entity_born>"]}"#,
    )));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let keys: Vec<&String> = document.item_settings.as_ref().unwrap().keys().collect();
    assert_eq!(keys, ["minecraft:WOOL:3"]);
    assert_eq!(
        document.liquid_targeting_list.as_ref().unwrap()[0].as_str(),
        "minecraft:Bucket:1"
    );
    let entities = document
        .exclusion_settings
        .as_ref()
        .unwrap()
        .entities
        .as_ref()
        .unwrap();
    assert_eq!(entities[0].as_str(), "minecraft:pig<minecraft:entity_born>");
    let written = document.to_json().unwrap();
    assert!(
        written.contains(r#""minecraft:WOOL:3": "test:c""#),
        "{written}"
    );
    assert!(
        written.contains(r#""minecraft:pig<minecraft:entity_born>""#),
        "{written}"
    );
    assert_eq!(loads(preset(&written)).value(), document.value());
}

#[test]
fn aa05_the_docs_object_form_of_the_lists_is_a_type_error() {
    for (members, path) in [
        (r#""exclusion_list": {}"#, "exclusion_list"),
        // The vanilla unit test's input (`InvalidPresetV1HavingFormatOfV2_LoadPreset_ThrowError`).
        (
            r#""exclusion_list": {"blocks": ["minecraft:grass_block"], "entities": ["minecraft:arrow"]}"#,
            "exclusion_list",
        ),
        (r#""liquid_targeting_list": {}"#, "liquid_targeting_list"),
    ] {
        let d = rejects(preset(&preset_json(
            "1.21.50",
            &format!(r#""identifier": "test:p", {members}"#),
        )));
        assert_eq!(
            errors(&d),
            [finding(
                "core/invalid_type_worded",
                &at_preset(path),
                "expected an array"
            )],
            "{members}"
        );
    }
    // facet would read `[]` as an empty map: vanilla expects an object.
    for (members, path, id) in [
        (
            r#""item_settings": []"#,
            "item_settings",
            "core/expected_object",
        ),
        (
            r#""exclusion_settings": {"blocks": {}}"#,
            "exclusion_settings/blocks",
            "core/invalid_type_worded",
        ),
    ] {
        let d = rejects(preset(&preset_json(
            "1.21.120",
            &format!(r#""identifier": "test:p", {members}"#),
        )));
        let found = errors(&d);
        assert_eq!(found.len(), 1, "{members}: {found:?}");
        assert_eq!((found[0].0, found[0].1.clone()), (id, at_preset(path)));
    }
    for members in [
        r#""identifier": 5"#,
        r#""identifier": "test:p", "hand_settings": null"#,
        r#""identifier": "test:p", "item_settings": {"minecraft:stick": 1}"#,
        r#""identifier": "test:p", "liquid_targeting_list": [1]"#,
    ] {
        let d = rejects(preset(&preset_json("1.21.120", members)));
        assert_eq!(errors(&d).len(), 1, "{members}");
        assert_eq!(errors(&d)[0].0, "core/invalid_type", "{members}");
    }
}

#[test]
fn aa06_categories_load_unchanged_and_default_to_priority_1() {
    let text = categories_json(
        "1.21.50",
        &format!(
            "{}, {}",
            category(
                "test:full",
                r#""entities": {"minecraft:pig": 60, "minecraft:cow": 0},
                "blocks": {"minecraft:stone": 100}, "block_tags": {"wood": 5},
                "entity_type_families": {"monster": 90},
                "entity_default": 30, "block_default": 20"#
            ),
            category("test:bare", "")
        ),
    );
    let document = loads(categories(&text));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let full = &document.categories[0];
    assert_eq!(full.name.as_str(), "test:full");
    let pairs = |map: &Option<indexmap::IndexMap<String, i32>>| -> Vec<(String, i32)> {
        map.as_ref()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    };
    assert_eq!(
        pairs(&full.priorities.entities),
        [
            ("minecraft:pig".to_owned(), 60),
            ("minecraft:cow".to_owned(), 0)
        ]
    );
    assert_eq!(
        pairs(&full.priorities.blocks),
        [("minecraft:stone".to_owned(), 100)]
    );
    assert_eq!(pairs(&full.priorities.block_tags), [("wood".to_owned(), 5)]);
    assert_eq!(
        pairs(&full.priorities.entity_type_families),
        [("monster".to_owned(), 90)]
    );
    assert_eq!(
        (
            full.priorities.entity_default(),
            full.priorities.block_default()
        ),
        (30, 20)
    );
    // Without the defaults the game uses 1; the document still has none.
    let bare = &document.categories[1];
    assert_eq!(
        (
            bare.priorities.entity_default,
            bare.priorities.block_default
        ),
        (None, None)
    );
    assert_eq!(
        (
            bare.priorities.entity_default(),
            bare.priorities.block_default()
        ),
        (1, 1)
    );
    assert_eq!(bare.priorities.entities, None);
    // Written back, the document is the file's.
    let written = document.to_json().unwrap();
    let a: bedrock_addon::json::Value = facet_json::from_str(&written).unwrap();
    let b: bedrock_addon::json::Value = facet_json::from_str(&text).unwrap();
    assert_eq!(a, b);
    assert_eq!(loads(categories(&written)).value(), document.value());
}

#[test]
fn aa07_a_missing_format_version_is_an_error() {
    let d = rejects(preset(
        r#"{"minecraft:aim_assist_preset": {"identifier": "test:p"}}"#,
    ));
    assert_eq!(
        errors(&d),
        [finding("core/no_valid_version", "", NO_VERSION)]
    );
    let d = rejects(categories(
        r#"{"minecraft:aim_assist_categories": {"categories": []}}"#,
    ));
    assert_eq!(
        errors(&d),
        [finding("core/no_valid_version", "", NO_VERSION)]
    );
}

#[test]
fn aa07b_envelope_edges() {
    let malformed = [finding(
        "core/malformed_format_version",
        "format_version",
        "Malformed format version",
    )];
    for version in [r#""abc""#, "true", "null", "{}", "[1, 21]"] {
        let d = rejects(preset(&format!(
            r#"{{"format_version": {version}, "minecraft:aim_assist_preset": {{"identifier": "test:p"}}}}"#
        )));
        assert_eq!(errors(&d), malformed, "{version}");
        let d = rejects(categories(&format!(
            r#"{{"format_version": {version}, "minecraft:aim_assist_categories": {{"categories": []}}}}"#
        )));
        assert_eq!(errors(&d), malformed, "{version}");
    }
    // A third root member is an unknown member of the root struct: no "exactly two members".
    let text = preset_json("1.21.120", r#""identifier": "test:p""#);
    let d = rejects(preset(&text.replacen('{', r#"{"extra": 1, "#, 1)));
    assert_eq!(
        errors(&d),
        [finding(
            "core/unrecognized_field_worded",
            "extra",
            NOT_IN_SCHEMA
        )]
    );
    let text = categories_json("1.21.50", "");
    let d = rejects(categories(&text.replacen('{', r#"{"extra": 1, "#, 1)));
    assert_eq!(
        errors(&d),
        [finding(
            "core/unrecognized_field_worded",
            "extra",
            NOT_IN_SCHEMA
        )]
    );
    // Three integers route as a version, then fail the string bind of `format_version`.
    let d = rejects(preset(
        r#"{"format_version": [1, 21, 120], "minecraft:aim_assist_preset": {"identifier": "test:p"}}"#,
    ));
    let found = errors(&d);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(
        (found[0].0, found[0].1.as_str()),
        ("core/invalid_type", "format_version")
    );
    // A number is read as its text (`MinecraftDocumentInput`):
    // it routes as a version, then fails the string bind too.
    let d = rejects(preset(
        r#"{"format_version": 1.22, "minecraft:aim_assist_preset": {"identifier": "test:p"}}"#,
    ));
    let found = errors(&d);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(
        (found[0].0, found[0].1.as_str()),
        ("core/invalid_type", "format_version")
    );
    let d = rejects(categories(
        r#"{"format_version": 1.22, "minecraft:aim_assist_categories": {"categories": []}}"#,
    ));
    let found = errors(&d);
    assert_eq!(
        (found[0].0, found[0].1.as_str()),
        ("core/invalid_type", "format_version")
    );
    let d = rejects(categories(
        r#"{"format_version": 1.20, "minecraft:aim_assist_categories": {"categories": []}}"#,
    ));
    assert_eq!(errors(&d)[0].0, "core/no_parser_for_version");
    // The payload key is required by the root struct.
    let d = rejects(preset(r#"{"format_version": "1.21.120"}"#));
    assert_eq!(
        errors(&d),
        [finding(
            "core/missing_required_field_worded",
            "minecraft:aim_assist_preset",
            "missing required field"
        )]
    );
    let d = rejects(categories(r#"{"format_version": "1.21.120"}"#));
    assert_eq!(
        errors(&d),
        [finding(
            "core/missing_required_field_worded",
            "minecraft:aim_assist_categories",
            "missing required field"
        )]
    );
}

#[test]
fn aa08_above_the_engine_nothing_loads() {
    let engine = bedrock_addon::version::ENGINE_VERSION;
    let above = format!("{}.{}.{}", engine.major, engine.minor, engine.patch + 10);
    let expected = [finding(
        "core/no_parser_for_version",
        "format_version",
        &format!("unsupported version: no parser available for version {above}"),
    )];
    let d = rejects(preset(&preset_json(&above, r#""identifier": "test:p""#)));
    assert_eq!(errors(&d), expected);
    let d = rejects(categories(&categories_json(&above, "")));
    assert_eq!(errors(&d), expected);
    loads(preset(&preset_json(
        &engine.to_string(),
        r#""identifier": "test:p""#,
    )));
    loads(categories(&categories_json(&engine.to_string(), "")));
}

#[test]
fn aa09_a_preset_without_identifier_misses_a_required_field() {
    // The vanilla unit test's input.
    let d = rejects(preset(
        r#"{"format_version": "1.21.120", "minecraft:aim_assist_preset": {}}"#,
    ));
    assert_eq!(
        errors(&d),
        [finding(
            "core/missing_required_field_worded",
            &at_preset("identifier"),
            "missing required field"
        )]
    );
    let d = rejects(categories(&categories_json(
        "1.21.50",
        r#"{"name": "test:c"}, {"priorities": {}}"#,
    )));
    assert_eq!(
        errors(&d),
        [
            finding(
                "core/missing_required_field_worded",
                &at_category("0/priorities"),
                "missing required field"
            ),
            finding(
                "core/missing_required_field_worded",
                &at_category("1/name"),
                "missing required field"
            )
        ]
    );
    let d = rejects(categories(
        r#"{"format_version": "1.21.50", "minecraft:aim_assist_categories": {}}"#,
    ));
    assert_eq!(
        errors(&d),
        [finding(
            "core/missing_required_field_worded",
            "minecraft:aim_assist_categories/categories",
            "missing required field"
        )]
    );
}

#[test]
fn aa10_a_priority_above_100_rejects_the_file() {
    const RANGE: &str = "the input is outside of the accepted range [0, 100]";
    for (priorities, path) in [
        (
            r#""entities": {"minecraft:pig": 200}"#,
            "entities/minecraft:pig",
        ),
        (
            r#""blocks": {"minecraft:stone": 101}"#,
            "blocks/minecraft:stone",
        ),
        (r#""block_tags": {"wood": 101}"#, "block_tags/wood"),
        (
            r#""entity_type_families": {"monster": 101}"#,
            "entity_type_families/monster",
        ),
        (r#""entity_default": 101"#, "entity_default"),
        (r#""block_default": 101"#, "block_default"),
    ] {
        let d = rejects(categories(&categories_json(
            "1.21.50",
            &format!(
                "{}, {}",
                category("test:ok", ""),
                category("test:c", priorities)
            ),
        )));
        assert_eq!(
            errors(&d),
            [finding(
                "core/outside_accepted_range",
                &at_category(&format!("1/priorities/{path}")),
                RANGE
            )],
            "{priorities}"
        );
    }
    // Both ends load.
    loads(categories(&categories_json(
        "1.21.50",
        &category(
            "test:c",
            r#""entities": {"a": 0, "b": 100}, "blocks": {"a": 100}, "block_tags": {"a": 100},
            "entity_type_families": {"a": 100}, "entity_default": 100, "block_default": 0"#,
        ),
    )));
}

#[test]
fn aa11_a_negative_priority_rejects_the_file() {
    for (priorities, path) in [
        (
            r#""entities": {"minecraft:pig": -1}"#,
            "entities/minecraft:pig",
        ),
        (
            r#""blocks": {"minecraft:stone": -1}"#,
            "blocks/minecraft:stone",
        ),
        (r#""block_tags": {"wood": -1}"#, "block_tags/wood"),
        (
            r#""entity_type_families": {"monster": -1}"#,
            "entity_type_families/monster",
        ),
        (r#""entity_default": -1"#, "entity_default"),
        (r#""block_default": -1"#, "block_default"),
    ] {
        let d = rejects(categories(&categories_json(
            "1.21.50",
            &category("test:c", priorities),
        )));
        assert_eq!(
            errors(&d),
            [finding(
                "core/invalid_value_worded",
                &at_category(&format!("0/priorities/{path}")),
                "invalid numeric value"
            )],
            "{priorities}"
        );
    }
    for priorities in [
        r#""entities": {"minecraft:pig": 1.5}"#,
        r#""entities": {"minecraft:pig": "1"}"#,
        r#""entities": []"#,
        r#""entity_default": null"#,
    ] {
        let d = rejects(categories(&categories_json(
            "1.21.50",
            &category("test:c", priorities),
        )));
        assert_eq!(errors(&d).len(), 1, "{priorities}");
        // facet reads `[]` as an empty map: vanilla expects an object.
        let id = if priorities == r#""entities": []"# {
            "core/expected_object"
        } else {
            "core/invalid_type"
        };
        assert_eq!(errors(&d)[0].0, id, "{priorities}");
    }
}

#[test]
fn aa12_one_category_without_a_namespace_drops_the_whole_file() {
    let d = rejects(categories(&categories_json(
        "1.21.50",
        &format!(
            "{}, {}",
            category("test:melee", ""),
            category("default", "")
        ),
    )));
    assert_eq!(
        errors(&d),
        [finding(
            "aim_assist/category_namespace",
            &at_category("1/name"),
            "Namespace required for category: default"
        )]
    );
    for (name, text) in [
        (":melee", "Namespace required for category: :melee"),
        ("", "Namespace required for category: "),
        ("a:b:c", "Invalid namespace format for category: a:b:c"),
        // The namespace is the text before the last colon: not empty here.
        (":a:b", "Invalid namespace format for category: :a:b"),
        ("test:", "Invalid namespace format for category: test:"),
    ] {
        let d = rejects(categories(&categories_json("1.21.50", &category(name, ""))));
        assert_eq!(
            errors(&d),
            [finding(
                if text.starts_with("Namespace required") {
                    "aim_assist/category_namespace"
                } else {
                    "aim_assist/category_format"
                },
                &at_category("0/name"),
                text
            )],
            "{name}"
        );
    }
    // No pattern beyond the namespace: any characters pass.
    loads(categories(&categories_json(
        "1.21.50",
        &category("Test-1:Melee.2", ""),
    )));
}

#[test]
fn aa13_a_preset_identifier_needs_one_namespace() {
    for (identifier, text) in [
        (
            "nonamespace",
            "Namespace required for preset Id: nonamespace",
        ),
        (":p", "Namespace required for preset Id: :p"),
        // The namespace is the text before the last colon: not empty here.
        (
            ":a:b",
            "Failed to add aim assist preset with invalid namespace Id: :a:b",
        ),
        (
            "a:b:c",
            "Failed to add aim assist preset with invalid namespace Id: a:b:c",
        ),
        (
            "test:",
            "Failed to add aim assist preset with invalid namespace Id: test:",
        ),
    ] {
        for version in ["1.21.50", "1.21.120"] {
            let d = rejects(preset(&preset_json(
                version,
                &format!(r#""identifier": "{identifier}""#),
            )));
            assert_eq!(
                errors(&d),
                [finding(
                    if text.starts_with("Namespace required") {
                        "aim_assist/preset_namespace"
                    } else {
                        "aim_assist/preset_format"
                    },
                    &at_preset("identifier"),
                    text
                )],
                "{identifier} at {version}"
            );
        }
    }
    loads(preset(&preset_json(
        "1.21.120",
        r#""identifier": "Test-1:P.2""#,
    )));
}

#[test]
fn aa14_a_category_reference_needs_one_namespace() {
    for (members, path, text) in [
        (
            r#""hand_settings": "empty_hand""#,
            "hand_settings",
            "Namespace required for category Id: empty_hand",
        ),
        (
            r#""item_settings": {"minecraft:stick": "a:b:c"}"#,
            "item_settings/minecraft:stick",
            "Invalid namespace format for category Id: a:b:c",
        ),
        (
            r#""default_item_settings": "default""#,
            "default_item_settings",
            "Namespace required for category Id: default",
        ),
        (
            r#""default_item_settings": "a:b:c""#,
            "default_item_settings",
            "Invalid namespace format for category Id: a:b:c",
        ),
        (
            r#""hand_settings": "test:""#,
            "hand_settings",
            "Invalid namespace format for category Id: test:",
        ),
        (
            r#""item_settings": {"minecraft:stick": "test:c", "minecraft:bow": "bow"}"#,
            "item_settings/minecraft:bow",
            "Namespace required for category Id: bow",
        ),
    ] {
        let d = rejects(preset(&preset_json(
            "1.21.120",
            &format!(r#""identifier": "test:p", {members}"#),
        )));
        assert_eq!(
            errors(&d),
            [finding(
                if text.starts_with("Namespace required") {
                    "aim_assist/category_id_namespace"
                } else {
                    "aim_assist/category_id_format"
                },
                &at_preset(path),
                text
            )],
            "{members}"
        );
    }
    // The preset is skipped at the first failure: default, then hand, then the item settings.
    let d = rejects(preset(&preset_json(
        "1.21.120",
        r#""identifier": "test:p", "item_settings": {"minecraft:stick": "stick"},
        "hand_settings": "hand", "default_item_settings": "default""#,
    )));
    assert_eq!(
        errors(&d),
        [finding(
            "aim_assist/category_id_namespace",
            &at_preset("default_item_settings"),
            "Namespace required for category Id: default"
        )]
    );
    let d = rejects(preset(&preset_json(
        "1.21.120",
        r#""identifier": "test:p", "item_settings": {"minecraft:stick": "stick"}, "hand_settings": "hand""#,
    )));
    assert_eq!(errors(&d)[0].1, at_preset("hand_settings"));
    // The identifier is checked before the references.
    let d = rejects(preset(&preset_json(
        "1.21.120",
        r#""identifier": "p", "hand_settings": "hand""#,
    )));
    assert_eq!(errors(&d).len(), 1);
    assert_eq!(errors(&d)[0].1, at_preset("identifier"));
}

#[test]
fn aa15_the_docs_categories_example_is_rejected() {
    // learn.microsoft.com … camerasystem/aimassistpresets, with the missing comma restored.
    let d = rejects(categories(
        r#"{
  "format_version": "1.21.50",
  "minecraft:aim_assist_categories": {
    "categories": [
      {
        "name": "test:bucket",
        "entity_default": 30,
        "block_dafault": 20,
        "priorities": {
          "blocks": { "minecraft:cauldron": 60 }
        }
      },
      {
        "name": "default",
        "priorities": {
          "entities": { "minecraft:pig": 100 }
        }
      }
    ]
  }
}"#,
    ));
    assert_eq!(
        errors(&d),
        [
            finding(
                "core/unrecognized_field_worded",
                &at_category("0/entity_default"),
                NOT_IN_SCHEMA
            ),
            finding(
                "core/unrecognized_field_worded",
                &at_category("0/block_dafault"),
                NOT_IN_SCHEMA
            )
        ]
    );
}

#[test]
fn aa16_a_1_21_120_exclusion_settings_array_is_a_type_error() {
    // The vanilla unit test's input (`InvalidPresetV2HavingFormatOfV1_LoadPreset_ThrowError`).
    let d = rejects(preset(&preset_json(
        "1.21.120",
        r#""identifier": "test:p", "exclusion_settings": ["minecraft:grass_block", "minecraft:arrow"]"#,
    )));
    assert_eq!(
        errors(&d),
        [finding(
            "core/expected_object",
            &at_preset("exclusion_settings"),
            "expected an object"
        )]
    );
}

#[test]
fn aa17_a_1_21_50_preset_without_exclusion_list_gets_four_empty_lists() {
    // The vanilla unit test's input (`NoExclusionInPresetV1_LoadPreset_ConvertV1toV2`).
    let document = loads(preset(&preset_json(
        "1.21.50",
        r#""identifier": "test:missing_exclusion_preset_identifier", "default_item_settings": "test:default_setting""#,
    )));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(
        document.exclusion_settings,
        Some(ExclusionSettings {
            blocks: Some(Vec::new()),
            entities: Some(Vec::new()),
            block_tags: Some(Vec::new()),
            entity_type_families: Some(Vec::new()),
        })
    );
    assert_eq!(
        document.default_item_settings.as_ref().unwrap().as_str(),
        "test:default_setting"
    );
    assert_eq!(
        document.identifier.as_str(),
        "test:missing_exclusion_preset_identifier"
    );
}

#[test]
fn every_member_of_both_documents_reads_into_its_field() {
    use bedrock_addon::families::aim_assist::{AimAssistCategories, AimAssistCategory, Priorities};

    // The member tables and the structs agree key for key: a document with every member equals
    // the value built by hand.
    let document = loads(preset(&preset_json(
        "1.21.120",
        r#""identifier": "test:all",
        "exclusion_settings": {"blocks": ["minecraft:stone"], "entities": ["minecraft:pig"],
            "block_tags": ["wood"], "entity_type_families": ["monster"]},
        "liquid_targeting_list": ["minecraft:bucket"],
        "item_settings": {"minecraft:bow": "test:ranged"},
        "default_item_settings": "test:default", "hand_settings": "test:hand""#,
    )));
    let mut expected = AimAssistPreset::new(AimAssistPresetId::parse("test:all"));
    expected.exclusion_settings = Some(ExclusionSettings {
        blocks: Some(vec![BlockId::parse("minecraft:stone")]),
        entities: Some(vec![EntityId::parse("minecraft:pig")]),
        block_tags: Some(strings(&["wood"])),
        entity_type_families: Some(strings(&["monster"])),
    });
    expected.liquid_targeting_list = Some(vec![ItemId::parse("minecraft:bucket")]);
    expected.item_settings = Some(
        [(
            "minecraft:bow".to_owned(),
            AimAssistCategoryId::parse("test:ranged"),
        )]
        .into_iter()
        .collect(),
    );
    expected.default_item_settings = Some(AimAssistCategoryId::parse("test:default"));
    expected.hand_settings = Some(AimAssistCategoryId::parse("test:hand"));
    assert_eq!(document.value(), &expected);

    let map = |key: &str, value: i32| {
        Some(
            [(key.to_owned(), value)]
                .into_iter()
                .collect::<indexmap::IndexMap<_, _>>(),
        )
    };
    let document = loads(categories(&categories_json(
        "1.21.50",
        &category(
            "test:all",
            r#""entities": {"minecraft:pig": 60}, "blocks": {"minecraft:stone": 50},
            "block_tags": {"wood": 40}, "entity_type_families": {"monster": 30},
            "entity_default": 20, "block_default": 10"#,
        ),
    )));
    assert_eq!(
        document.value(),
        &AimAssistCategories {
            categories: vec![AimAssistCategory::new(
                AimAssistCategoryId::parse("test:all"),
                Priorities {
                    entities: map("minecraft:pig", 60),
                    blocks: map("minecraft:stone", 50),
                    block_tags: map("wood", 40),
                    entity_type_families: map("monster", 30),
                    entity_default: Some(20),
                    block_default: Some(10),
                }
            )],
        }
    );
}

#[test]
fn a_duplicate_member_keeps_its_last_value_whole() {
    // Of a member written twice the last value stands, whole; two objects of one key
    // are not merged member by member.
    let document = loads(preset(&preset_json(
        "1.21.120",
        r#""identifier": "test:p",
        "exclusion_settings": {"blocks": ["minecraft:stone"], "block_tags": ["wood"]},
        "exclusion_settings": {"entities": ["minecraft:pig"]},
        "hand_settings": "nonamespace", "hand_settings": "test:hand""#,
    )));
    assert_eq!(
        document.exclusion_settings,
        Some(ExclusionSettings {
            blocks: None,
            entities: Some(vec![EntityId::parse("minecraft:pig")]),
            block_tags: None,
            entity_type_families: None,
        })
    );
    assert_eq!(document.hand_settings().as_str(), "test:hand");
    // The last value is the one checked.
    let d = rejects(categories(&categories_json(
        "1.21.50",
        &category("test:c", r#""entity_default": 5, "entity_default": 200"#),
    )));
    assert_eq!(errors(&d).len(), 1);
    assert_eq!(errors(&d)[0].0, "core/outside_accepted_range");
    // Two payloads at the root: the last one is the document.
    let d = rejects(preset(
        r#"{"format_version": "1.21.120",
        "minecraft:aim_assist_preset": {"identifier": "test:first"},
        "minecraft:aim_assist_preset": {"identifier": "test:last", "bogus": 1}}"#,
    ));
    assert_eq!(
        errors(&d),
        [finding(
            "core/unrecognized_field_worded",
            &at_preset("bogus"),
            NOT_IN_SCHEMA
        )]
    );
    let document = loads(preset(
        r#"{"format_version": "1.21.120",
        "minecraft:aim_assist_preset": {"identifier": "test:first", "bogus": 1},
        "minecraft:aim_assist_preset": {"identifier": "test:last"}}"#,
    ));
    assert_eq!(document.identifier.as_str(), "test:last");
}

#[test]
fn an_error_only_the_typed_parse_finds_is_reported_the_same_beside_a_duplicate_member() {
    // The second read's own failure is the finding, as without the duplicate.
    let plain = rejects(categories(&categories_json(
        "1.21.50",
        &category("test:c", r#""entity_default": 2, "block_default": 1.0"#),
    )));
    for priorities in [
        r#""entity_default": 1, "entity_default": 2, "block_default": 1.0"#,
        r#""block_default": 1.0, "entity_default": 1, "entity_default": 2"#,
        r#""entities": {"a": 1, "a": 2}, "block_default": 1.0"#,
    ] {
        let text = categories_json("1.21.50", &category("test:c", priorities));
        let doubled = rejects(categories(&text));
        assert_eq!(errors(&doubled), errors(&plain), "{priorities}");
        assert_eq!(
            errors(&doubled),
            [(
                "core/invalid_type",
                at_category("0/priorities/block_default"),
                errors(&plain)[0].2.clone()
            )]
        );
        let span = doubled
            .iter()
            .next()
            .unwrap()
            .location
            .span
            .expect("a span");
        assert_eq!(
            &text[span.offset as usize..(span.offset + span.len) as usize],
            "1.0",
            "{priorities}"
        );
    }
    // In the second category of the file, and in a map: the path and the span follow.
    let text = categories_json(
        "1.21.50",
        &format!(
            "{}, {}",
            category("test:a", r#""block_default": 3, "block_default": 4"#),
            category("test:b", r#""blocks": {"minecraft:stone": 2.5}"#)
        ),
    );
    let d = rejects(categories(&text));
    assert_eq!(errors(&d).len(), 1);
    assert_eq!(
        (errors(&d)[0].0, errors(&d)[0].1.clone()),
        (
            "core/invalid_type",
            at_category("1/priorities/blocks/minecraft:stone")
        )
    );
    let span = d.iter().next().unwrap().location.span.unwrap();
    assert_eq!(
        &text[span.offset as usize..(span.offset + span.len) as usize],
        "2.5"
    );
}

#[test]
fn a_bad_first_value_only_the_typed_parse_refuses_is_replaced_by_a_good_last_one() {
    // Of a member written twice the last value is the one read, also when the
    // first one would fail only the typed parse (a real where an integer is wanted): the file
    // loads, with the last value, and no finding about the first.
    for (members, entity_default, block_default) in [
        (r#""block_default": 1.0, "block_default": 2"#, None, Some(2)),
        (
            r#""entity_default": 2.5, "entity_default": 7"#,
            Some(7),
            None,
        ),
        (
            r#""block_default": "1", "block_default": 3, "entity_default": 4"#,
            Some(4),
            Some(3),
        ),
    ] {
        let text = categories_json("1.21.50", &category("test:c", members));
        let document = loads(categories(&text));
        assert!(
            document.diagnostics().is_empty(),
            "{members}: {:#?}",
            document.diagnostics()
        );
        let priorities = &document.categories[0].priorities;
        assert_eq!(priorities.entity_default, entity_default, "{members}");
        assert_eq!(priorities.block_default, block_default, "{members}");
    }
}

#[test]
fn the_span_of_a_member_written_twice_with_the_bad_value_last_is_the_last_value() {
    // The finding of a member written twice whose last value is the bad one
    // is at the member, with the span of that last value in the file, not of the first.
    for (members, bad) in [
        (r#""block_default": 2, "block_default": 1.0"#, "1.0"),
        (r#""block_default": 1.5, "block_default": 2.5"#, "2.5"),
    ] {
        let text = categories_json("1.21.50", &category("test:c", members));
        let d = rejects(categories(&text));
        assert_eq!(errors(&d).len(), 1, "{members}");
        assert_eq!(
            (errors(&d)[0].0, errors(&d)[0].1.clone()),
            (
                "core/invalid_type",
                at_category("0/priorities/block_default")
            ),
            "{members}"
        );
        let span = d.iter().next().unwrap().location.span.expect("a span");
        let start = span.offset as usize;
        assert_eq!(&text[start..start + span.len as usize], bad, "{members}");
        assert_eq!(
            start,
            text.rfind(bad).unwrap(),
            "{members}: the span is the last value's"
        );
    }
}
