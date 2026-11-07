//! `BlockDescriptor`, `BlockSpecifier`, `StateValue`: the JSON forms round trip;
//! `Mixed` and `Invalid` are kept and reported with vanilla's texts as `InvalidValue` warnings.

mod common;

use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::json::Value;
use bedrock_addon_shared_shapes::{BlockDescriptor, BlockSpecifier, StateValue};
use common::{compact, read, round_trip};

/// The one finding of `descriptor`'s check, as the vanilla text it renders.
fn text_of(descriptor: &BlockDescriptor) -> String {
    let found = descriptor.check(&JsonPath::new(), Severity::Warning);
    assert_eq!(found.len(), 1, "{descriptor:?}");
    let finding = found.iter().next().unwrap();
    // The game logs these at level 3 and goes on: a kept error.
    assert_eq!(finding.severity, Severity::Error);
    assert!(finding.kept);
    // By legacy name: the variant and the definitions that were it.
    assert_eq!(finding.code.name(), "InvalidValue", "{:?}", finding.code);
    finding.code.vanilla().expect("a vanilla text")
}

#[test]
fn a_string_names_one_block() {
    let d: BlockDescriptor = round_trip(r#""minecraft:stone""#);
    let BlockDescriptor::Block {
        name,
        states,
        object,
        extra,
    } = &d
    else {
        panic!("not a block")
    };
    assert_eq!(name, &BlockId::new("minecraft:stone").unwrap());
    assert!(states.is_empty() && !object && extra.is_empty());
    assert!(d.check(&JsonPath::new(), Severity::Warning).is_empty());
}

#[test]
fn the_object_form_carries_states_of_three_types() {
    let d: BlockDescriptor = round_trip(
        r#"{"name": "minecraft:wool", "states": {"color": "red", "age": 3, "open_bit": true, "id": "1"}}"#,
    );
    let BlockDescriptor::Block { states, object, .. } = &d else {
        panic!("not a block")
    };
    assert!(object);
    assert_eq!(states["color"], StateValue::String("red".into()));
    assert_eq!(states["age"], StateValue::Int(3));
    assert_eq!(states["open_bit"], StateValue::Bool(true));
    // `"1"` stays a string: vanilla's `Compound::Type` keeps them apart.
    assert_eq!(states["id"], StateValue::String("1".into()));
    let BlockDescriptor::Block { .. } = round_trip::<BlockDescriptor>(r#"{"name": "a:b"}"#) else {
        panic!("not a block")
    };
}

#[test]
fn tags_is_a_molang_expression() {
    let d: BlockDescriptor = round_trip(r#"{"tags": "query.any_tag('stone')"}"#);
    let BlockDescriptor::Tags { tags, .. } = &d else {
        panic!("not tags")
    };
    assert_eq!(tags.as_str(), "query.any_tag('stone')");
    assert!(d.check(&JsonPath::new(), Severity::Warning).is_empty());
}

#[test]
fn a_name_or_states_beside_tags_is_mixed_and_reported() {
    let d: BlockDescriptor = round_trip(r#"{"name": "a:b", "tags": "q.any_tag('x')"}"#);
    assert!(matches!(&d, BlockDescriptor::Mixed { name: Some(_), .. }));
    assert_eq!(
        text_of(&d),
        "Cannot have both a block name and tag expression in a block descriptor for block: a:b"
            .to_owned()
    );
    let d: BlockDescriptor = round_trip(r#"{"states": {"age": 1}, "tags": "q.any_tag('x')"}"#);
    assert!(matches!(&d, BlockDescriptor::Mixed { name: None, .. }));
    assert_eq!(
        text_of(&d),
        "Cannot have both a tag expression and block states in a block descriptor for block: "
            .to_owned()
    );
}

#[test]
fn a_wrong_shape_is_kept_with_the_json_readers_text() {
    for json in [
        "5",
        "true",
        "[]",
        "{}",
        r#"{"states": {"age": 1}}"#,
        r#"{"tags": 1}"#,
    ] {
        let d: BlockDescriptor = round_trip(json);
        assert!(matches!(d, BlockDescriptor::Invalid(_)), "{json}");
        assert_eq!(
            text_of(&d),
            format!("BlockDescriptor requires a string, block object, or any_tag object. Cannot parse {}", common::normalised(json))
            .to_string(),
            "{json}"
        );
    }
}

#[test]
fn a_name_that_is_not_a_string_is_kept_with_its_text() {
    let d: BlockDescriptor = round_trip(r#"{"name": 7}"#);
    assert!(matches!(d, BlockDescriptor::Invalid(_)));
    assert_eq!(
        text_of(&d),
        "Only strings are valid for name. The invalid name value: 7\n".to_owned()
    );
}

#[test]
fn a_state_of_another_type_is_kept_with_its_text() {
    // A real such as `1.0` is invalid: jsoncpp type 3.
    let d: BlockDescriptor = round_trip(r#"{"name": "a:b", "states": {"age": 1.0}}"#);
    assert!(matches!(d, BlockDescriptor::Invalid(_)));
    assert_eq!(
        text_of(&d),
        "Invalid state type during block parsing: block name a:b, state name age, state type 3. State type can only be bool, int, or string.".to_owned()
    );
    let d: BlockDescriptor = read(r#"{"name": "a:b", "states": {"x": null}}"#).unwrap();
    assert!(text_of(&d).contains(", state type 0. "), "{}", text_of(&d));
}

#[test]
fn unknown_members_are_kept_at_their_level() {
    let d: BlockDescriptor = round_trip(r#"{"name": "a:b", "weight": 2}"#);
    let BlockDescriptor::Block { extra, .. } = &d else {
        panic!("not a block")
    };
    assert_eq!(extra.get("weight"), Some(&Value::from(2i64)));
    let d: BlockDescriptor = round_trip(r#"{"tags": "1", "note": "x"}"#);
    assert!(matches!(&d, BlockDescriptor::Tags { extra, .. } if !extra.is_empty()));
}

#[test]
fn a_list_member_holds_descriptors_and_keeps_bad_entries() {
    let list: Vec<BlockDescriptor> = read(r#"[5, "minecraft:grass_block"]"#).unwrap();
    assert!(matches!(list[0], BlockDescriptor::Invalid(_)));
    assert!(matches!(list[1], BlockDescriptor::Block { .. }));
}

#[test]
fn block_specifier_takes_a_name_or_an_object() {
    let s: BlockSpecifier = round_trip(r#""minecraft:dirt""#);
    assert_eq!(s.name, BlockId::new("minecraft:dirt").unwrap());
    assert!(!s.object && s.states.is_empty());
    let s: BlockSpecifier =
        round_trip(r#"{"name": "minecraft:log", "states": {"pillar_axis": "y"}, "x": 1}"#);
    assert!(s.object);
    assert_eq!(s.states["pillar_axis"], StateValue::String("y".into()));
    assert_eq!(s.extra.get("x"), Some(&Value::from(1i64)));
}

#[test]
fn block_specifier_has_no_tag_form() {
    for json in [
        r#"{"tags": "q.any_tag('a')"}"#,
        "{}",
        "5",
        r#"{"name": 1}"#,
        r#"{"name": "a:b", "states": {"s": 1.5}}"#,
    ] {
        assert!(
            read::<BlockSpecifier>(json).is_err(),
            "{json} must be rejected"
        );
    }
}

#[test]
fn state_value_reads_bool_int_and_string_only() {
    assert_eq!(round_trip::<StateValue>("true"), StateValue::Bool(true));
    assert_eq!(round_trip::<StateValue>("-3"), StateValue::Int(-3));
    assert_eq!(
        round_trip::<StateValue>(r#""1""#),
        StateValue::String("1".into())
    );
    for json in ["1.0", "null", "[]", "{}", "3000000000"] {
        assert!(read::<StateValue>(json).is_err(), "{json} must be rejected");
    }
}

#[test]
fn constructors_write_the_plain_forms() {
    let id = BlockId::new("minecraft:stone").unwrap();
    assert_eq!(
        compact(&BlockDescriptor::block(id.clone())),
        r#""minecraft:stone""#
    );
    assert_eq!(compact(&BlockSpecifier::new(id)), r#""minecraft:stone""#);
}
