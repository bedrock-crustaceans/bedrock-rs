//! Unit tests of the aim-assist module's private parts: the text arities and the link checker
//! the link pass calls.

use super::link::{self, Targets, entity_key, item_key};
use super::{AimAssist, AimAssistCategories, AimAssistCategory, AimAssistPreset};
use bedrock_addon_core::diagnostic::{DiagnosticDetail, Phase, Severity};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, RegistryBuilder,
};
use bedrock_addon_core::load::{LoadContext, LoadOptions};
use bedrock_addon_core::pack_identity::PackPath;

/// The world's blocks, entities and items: three of each.
struct World;

impl Targets for World {
    fn block(&self, key: &str) -> bool {
        ["minecraft:stone", "minecraft:water"].contains(&key)
    }
    fn entity(&self, key: &str) -> bool {
        ["minecraft:pig", "minecraft:stone_golem"].contains(&key)
    }
    fn item(&self, key: &str) -> bool {
        ["minecraft:wool", "minecraft:bucket"].contains(&key)
    }
    fn category(&self, _: &str) -> bool {
        false
    }
    fn preset(&self, key: &str) -> bool {
        key == "minecraft:aim_assist_default"
    }
}

fn registry<T: Definition>(
    documents: Vec<(bedrock_addon_core::id::Id<T::Kind>, Document<T>)>,
) -> bedrock_addon_core::document::Registry<T> {
    let mut builder = RegistryBuilder::new(
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        &LoadOptions::default(),
    );
    for (id, document) in documents {
        builder.push(Contribution::Stable(id, document));
    }
    builder.finish()
}

/// A group of one preset (`members` after its identifier) and the categories `names`.
fn group(version: &str, members: &str, names: &[&str]) -> AimAssist {
    let cx = LoadContext::detached();
    let preset = AimAssistPreset::parse(
        &PackPath::from_static("aim_assist/presets/p.json"),
        format!(
            r#"{{"format_version": "{version}", "minecraft:aim_assist_preset": {{"identifier": "test:p", {members}}}}}"#
        )
        .as_bytes(),
        &cx,
    )
    .unwrap();
    let categories: Vec<String> = names
        .iter()
        .map(|name| {
            format!(
                r#"{{"name": "{name}", "priorities": {{"entities": {{"minecraft:PIG<minecraft:born>": 1, "test:ghost": 2}}, "blocks": {{"minecraft:Stone": 1, "test:air": 2}}, "block_tags": {{"anything": 1}}, "entity_type_families": {{"anything": 1}}}}}}"#
            )
        })
        .collect();
    let file = AimAssistCategories::parse(
        &PackPath::from_static("aim_assist/categories/c.json"),
        format!(
            r#"{{"format_version": "1.21.50", "minecraft:aim_assist_categories": {{"categories": [{}]}}}}"#,
            categories.join(",")
        )
        .as_bytes(),
        &cx,
    )
    .unwrap();
    let (origin, version) = (file.origin().clone(), file.version().clone());
    let categories: Vec<_> = file
        .into_value()
        .categories
        .into_iter()
        .map(|category: AimAssistCategory| {
            (
                category.name.clone(),
                Document::new(
                    category,
                    origin.clone(),
                    version.clone(),
                    bedrock_addon_core::diagnostic::Diagnostics::default(),
                ),
            )
        })
        .collect();
    AimAssist {
        presets: registry(vec![(preset.identifier.clone(), preset)]),
        categories: registry(categories),
    }
}

/// `(JSON path, detail)` of each finding of the link check.
fn misses(group: &AimAssist) -> Vec<(String, String)> {
    link::link(group, &World)
        .iter()
        .map(|finding| {
            assert_eq!(finding.code.name(), "UnresolvedReference");
            assert_eq!(finding.severity, Severity::Info);
            assert_eq!(finding.phase, Phase::Link);
            assert!(finding.location.path.is_some());
            let Some(DiagnosticDetail::Reference { kind, id, hint }) = finding.detail.as_deref()
            else {
                panic!("a reference detail");
            };
            assert!(hint.is_none());
            (
                finding.location.json_path.to_string(),
                format!("{kind} '{id}'"),
            )
        })
        .collect()
}

#[test]
fn the_lookup_keys_are_vanillas_registered_names_deviation() {
    // vanilla = "linked-asset key normalisation": lower-cased; an item of three or more parts
    // without its last one; an entity without its `<event>`.
    assert_eq!(item_key("minecraft:WOOL:3"), "minecraft:wool");
    assert_eq!(item_key("minecraft:Wool"), "minecraft:wool");
    assert_eq!(item_key("Wool"), "wool");
    assert_eq!(item_key("a:b:c:d"), "a:b:c");
    assert_eq!(
        entity_key("minecraft:Pig<minecraft:entity_born>"),
        "minecraft:pig"
    );
    assert_eq!(entity_key("minecraft:PIG"), "minecraft:pig");
    assert_eq!(entity_key("a<b<c"), "a");
}

#[test]
fn unknown_references_are_link_warnings_deviation() {
    // vanilla = "linked-asset validator only" / "no diagnostic": each miss is an information of the
    // link pass, at its member, with the name as written.
    let group = group(
        "1.21.120",
        r#""item_settings": {"minecraft:WOOL:3": "test:known", "test:Nothing:1": "test:unknown"},
        "liquid_targeting_list": ["minecraft:Bucket", "test:pail"],
        "default_item_settings": "test:known", "hand_settings": "test:no_hand",
        "exclusion_settings": {"blocks": ["minecraft:STONE", "test:block"],
            "entities": ["minecraft:pig<minecraft:entity_born>", "test:entity<e>"],
            "block_tags": ["anything"], "entity_type_families": ["anything"]}"#,
        &["test:known"],
    );
    let preset = "minecraft:aim_assist_preset";
    let category = "minecraft:aim_assist_categories/categories/0/priorities";
    assert_eq!(
        misses(&group),
        [
            (
                format!("{category}/entities/test:ghost"),
                "entity 'test:ghost'".to_owned()
            ),
            (
                format!("{category}/blocks/test:air"),
                "block 'test:air'".to_owned()
            ),
            (
                format!("{preset}/exclusion_settings/blocks/1"),
                "block 'test:block'".to_owned()
            ),
            (
                format!("{preset}/exclusion_settings/entities/1"),
                "entity 'test:entity<e>'".to_owned()
            ),
            (
                format!("{preset}/liquid_targeting_list/1"),
                "item 'test:pail'".to_owned()
            ),
            (
                format!("{preset}/item_settings/test:Nothing:1"),
                "item 'test:Nothing:1'".to_owned()
            ),
            (
                format!("{preset}/item_settings/test:Nothing:1"),
                "aim assist category 'test:unknown'".to_owned()
            ),
            (
                format!("{preset}/hand_settings"),
                "aim assist category 'test:no_hand'".to_owned()
            ),
        ]
    );
}

#[test]
fn the_runtime_default_categories_are_not_references_of_the_document() {
    // A preset that names no category uses `minecraft:default` / `minecraft:empty_hand` at
    // run time; the document does not reference them, so nothing is reported.
    let group = group("1.21.120", r#""item_settings": {}"#, &[]);
    assert_eq!(misses(&group), []);
}

#[test]
fn an_upgraded_exclusion_list_is_checked_as_block_or_entity_deviation() {
    // The step copies each entry of the 1.21.50 list into both lists, so a block is also looked up as an entity
    // and an entity as a block: vanilla registers the entry once, as block-or-entity.
    let group = group(
        "1.21.50",
        r#""exclusion_list": ["minecraft:stone", "minecraft:pig", "test:neither"]"#,
        &[],
    );
    let preset = "minecraft:aim_assist_preset/exclusion_settings";
    assert_eq!(
        misses(&group),
        [
            (
                format!("{preset}/blocks/1"),
                "block 'minecraft:pig'".to_owned()
            ),
            (
                format!("{preset}/blocks/2"),
                "block 'test:neither'".to_owned()
            ),
            (
                format!("{preset}/entities/0"),
                "entity 'minecraft:stone'".to_owned()
            ),
            (
                format!("{preset}/entities/2"),
                "entity 'test:neither'".to_owned()
            ),
        ]
    );
}

#[test]
fn a_walked_file_implements_debug() {
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<super::AimAssistFile>();
}
