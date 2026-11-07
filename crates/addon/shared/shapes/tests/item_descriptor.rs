//! `ItemDescriptor`: the five JSON forms round trip with their `form`; a wrong shape is kept as
//! `Invalid` with vanilla's `InvalidType` text; several keys follow the interim rule of taking one
//! and keeping the rest.

mod common;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{MolangSource, SourceForm};
use bedrock_addon_shared_shapes::{ItemDescriptor, ItemDescriptorForm};
use common::{compact, read, round_trip};

/// The id and form of an `Item` descriptor.
fn item(descriptor: &ItemDescriptor) -> (&str, ItemDescriptorForm) {
    match descriptor {
        ItemDescriptor::Item { id, form, .. } => (id.as_str(), *form),
        other => panic!("not an item: {other:?}"),
    }
}

#[test]
fn a_string_names_one_item() {
    let d: ItemDescriptor = round_trip(r#""minecraft:stick""#);
    assert_eq!(item(&d), ("minecraft:stick", ItemDescriptorForm::String));
    assert!(d.check(&JsonPath::new()).is_empty());
}

#[test]
fn the_item_and_name_objects_name_one_item() {
    let d: ItemDescriptor = round_trip(r#"{"item": "minecraft:stick"}"#);
    assert_eq!(item(&d), ("minecraft:stick", ItemDescriptorForm::Item));
    // The cereal writer's form (re-serialised `storage_item.banned_items`).
    let d: ItemDescriptor = round_trip(r#"{"name": "minecraft:shulker_box"}"#);
    assert_eq!(
        item(&d),
        ("minecraft:shulker_box", ItemDescriptorForm::Name)
    );
}

#[test]
fn a_legacy_aux_item_is_kept_as_written() {
    let d: ItemDescriptor = round_trip(r#""minecraft:wool:3""#);
    let ItemDescriptor::Item { id, .. } = &d else {
        panic!("not an item")
    };
    assert_eq!(id, &ItemId::new("minecraft:wool").unwrap());
    assert_eq!(id.aux(), Some(3));
}

#[test]
fn tags_is_a_molang_expression() {
    let d: ItemDescriptor = round_trip(r#"{"tags": "query.any_tag('minecraft:planks')"}"#);
    let ItemDescriptor::Tags { tags, extra } = &d else {
        panic!("not tags")
    };
    assert_eq!(tags.as_str(), "query.any_tag('minecraft:planks')");
    assert!(matches!(tags.form(), SourceForm::String { .. }));
    assert!(extra.is_empty());
    // The object form is read (and written back); pass 1 of the Molang check rejects it.
    let d: ItemDescriptor =
        round_trip(r#"{"tags": {"expression": "q.any_tag('a')", "version": 12}}"#);
    let ItemDescriptor::Tags { tags, .. } = &d else {
        panic!("not tags")
    };
    assert_eq!(tags, &MolangSource::object("q.any_tag('a')", 12));
}

#[test]
fn item_tag_names_every_item_with_the_tag() {
    let d: ItemDescriptor = round_trip(r#"{"item_tag": "minecraft:planks"}"#);
    assert!(
        matches!(&d, ItemDescriptor::ItemTag { item_tag, .. } if item_tag == "minecraft:planks")
    );
}

#[test]
fn a_wrong_shape_is_kept_and_reported_as_invalid_type() {
    for json in [
        "5",
        "true",
        "[]",
        "{}",
        r#"{"item": 5}"#,
        r#"{"tags": 3}"#,
        r#"{"item_tag": ["a"]}"#,
        // A number is no tag: the string members are strict.
        r#"{"item_tag": 5}"#,
        r#"{"name": 5}"#,
        r#"{"other": "minecraft:stick"}"#,
    ] {
        let d: ItemDescriptor = round_trip(json);
        assert!(matches!(d, ItemDescriptor::Invalid(_)), "{json}");
        let found = d.check(&JsonPath::new());
        assert_eq!(found.len(), 1, "{json}");
        let finding = found.iter().next().unwrap();
        // The game logs an error at level 3 and builds an empty descriptor, so the document loads
        // on: a kept error.
        assert_eq!(finding.severity, Severity::Error);
        assert!(finding.kept);
        assert_eq!(
            finding.code.id(),
            "shapes/item_descriptor_shape",
            "{json}: {:?}",
            finding.code
        );
        assert_eq!(
            finding.code.vanilla(),
            Some(
                format!("ItemDescriptor requires a string, {{\"item\": string}}, or {{\"tags\": string}} or {{\"item_tag\": string}}. ItemDescriptor cannot parse {}\n", common::normalised(json))
                .to_string()
            ),
            "{json}"
        );
    }
}

#[test]
fn null_in_a_list_is_a_wrong_shape_and_absent_in_an_optional_member() {
    let list: Vec<ItemDescriptor> = read(r#"[null, "minecraft:stick"]"#).unwrap();
    assert_eq!(list[0], ItemDescriptor::Invalid(Value::NULL));
    let member: Option<ItemDescriptor> = read("null").unwrap();
    assert_eq!(member, None);
}

#[test]
fn unknown_members_are_kept_at_their_level() {
    let d: ItemDescriptor = round_trip(r#"{"item": "minecraft:stick", "count": 2}"#);
    let ItemDescriptor::Item { extra, .. } = &d else {
        panic!("not an item")
    };
    assert_eq!(extra.get("count"), Some(&Value::from(2i64)));
    // Kept, and reported.
    let found = d.check(&JsonPath::new());
    assert_eq!(found.len(), 1);
    assert!(
        found
            .iter()
            .next()
            .unwrap()
            .is(&bedrock_addon_core::diagnostic::codes::UNRECOGNIZED_FIELD)
    );
}

#[test]
fn o4_several_keys_take_item_name_tags_item_tag_in_that_order_with_an_information() {
    let d: ItemDescriptor =
        read(r#"{"tags": "q.any_tag('a')", "item_tag": "b", "name": "c:d"}"#).unwrap();
    assert_eq!(item(&d), ("c:d", ItemDescriptorForm::Name));
    let found = d.check(&JsonPath::new());
    assert_eq!(found.len(), 1);
    let finding = found.iter().next().unwrap();
    assert!(finding.is(&core_codes::INVALID_VALUE));
    // The game's constructor logs nothing for several keys, so the crate reports an information.
    assert_eq!(finding.severity, Severity::Info);
    // The other keys are kept and written after the chosen one.
    assert_eq!(
        compact(&d),
        r#"{"name":"c:d","tags":"q.any_tag('a')","item_tag":"b"}"#
    );

    let d: ItemDescriptor = read(r#"{"item_tag": "b", "tags": "x"}"#).unwrap();
    assert!(matches!(&d, ItemDescriptor::Tags { .. }));
}

#[test]
fn a_finding_is_located_under_the_path_given() {
    let d: ItemDescriptor = read("5").unwrap();
    let mut at = JsonPath::new();
    at.push("repair_items");
    at.push_index(0);
    let found = d.check(&at);
    assert_eq!(found.iter().next().unwrap().location.json_path, at);
}

#[test]
fn constructors_write_the_plain_forms() {
    let id = ItemId::new("minecraft:apple").unwrap();
    assert_eq!(compact(&ItemDescriptor::item(id)), r#""minecraft:apple""#);
    assert_eq!(
        compact(&ItemDescriptor::item_tag("minecraft:logs")),
        r#"{"item_tag":"minecraft:logs"}"#
    );
    let tags = MolangSource::string_without_context("q.any_tag('x')");
    assert_eq!(
        compact(&ItemDescriptor::tags(tags)),
        r#"{"tags":"q.any_tag('x')"}"#
    );
}
