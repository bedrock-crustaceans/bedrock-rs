#![allow(clippy::pedantic, missing_docs)]
//! `MenuCategory` and `CreativeCategory`: the object round trips; the five
//! category spellings; and the per-family constraints are not the type's.

#[path = "shared/common.rs"]
mod common;

use bedrock_addon_core::json::{self, StrictBool, Value};
use bedrock_addon_core::shared::{CreativeCategory, MenuCategory};
use common::{compact, read, round_trip};

#[test]
fn every_member_round_trips() {
    let m: MenuCategory = round_trip(
        r#"{"category": "nature", "group": "minecraft:itemGroup.name.planks", "is_hidden_in_commands": true}"#,
    );
    assert_eq!(m.category, Some(CreativeCategory::Nature));
    assert_eq!(m.group.as_deref(), Some("minecraft:itemGroup.name.planks"));
    assert_eq!(m.is_hidden_in_commands, Some(StrictBool(true)));
}

#[test]
fn an_empty_object_has_every_member_absent() {
    let m: MenuCategory = round_trip("{}");
    assert_eq!(m, MenuCategory::default());
}

#[test]
fn the_category_spellings() {
    for (name, category) in [
        ("construction", CreativeCategory::Construction),
        ("nature", CreativeCategory::Nature),
        ("equipment", CreativeCategory::Equipment),
        ("items", CreativeCategory::Items),
        ("none", CreativeCategory::ItemCommandOnly),
    ] {
        assert_eq!(
            round_trip::<CreativeCategory>(&format!(r#""{name}""#)),
            category
        );
    }
    for json in [r#""Nature""#, r#""all""#, r#""undefined""#, "1", "null"] {
        assert!(
            read::<CreativeCategory>(json).is_err(),
            "{json} must be rejected"
        );
    }
}

#[test]
fn undefined_is_written_as_none_and_all_cannot_be_written() {
    let m = MenuCategory {
        category: Some(CreativeCategory::Undefined),
        ..MenuCategory::default()
    };
    assert_eq!(compact(&m), r#"{"category":"none"}"#);
    let m = MenuCategory {
        category: Some(CreativeCategory::All),
        ..MenuCategory::default()
    };
    assert!(json::to_json(&m).is_err());
    assert_eq!(CreativeCategory::All.name(), None);
}

#[test]
fn is_hidden_in_commands_takes_a_boolean_only() {
    for json in ["1", r#""true""#] {
        let text = format!(r#"{{"is_hidden_in_commands": {json}}}"#);
        assert!(
            read::<MenuCategory>(&text).is_err(),
            "{json} must be rejected"
        );
    }
}

#[test]
fn d7_the_family_constraints_are_not_the_types() {
    // A block requires `category` and a namespaced `group`; an item limits `group` to 256
    // characters. The shared type takes all of these; each family checks its own rules.
    let long = "a".repeat(300);
    let m: MenuCategory = read(&format!(r#"{{"group": "{long}"}}"#)).unwrap();
    assert_eq!(m.category, None);
    assert_eq!(m.group.as_deref(), Some(long.as_str()));
    let m: MenuCategory = read(r#"{"category": "items", "group": "no namespace"}"#).unwrap();
    assert_eq!(m.group.as_deref(), Some("no namespace"));
}

#[test]
fn unknown_members_are_kept_by_the_family_walk() {
    let text = r#"{"category": "items", "note": 1}"#;
    let mut m: MenuCategory = facet_json::from_str_jsonc(text).unwrap();
    let value: Value = facet_json::from_str_jsonc(text).unwrap();
    let unknown = json::keep_unknown_members(&mut m, &value);
    assert_eq!(unknown.len(), 1);
    assert_eq!(m.extra.get("note"), Some(&Value::from(1i64)));
    assert_eq!(compact(&m), r#"{"category":"items","note":1}"#);
}

/// A member named like the skipped `extra` field is an unknown member.
#[test]
fn a_member_named_extra_is_an_unknown_member() {
    let text = r#"{"category": "items", "extra": {"group": "x"}}"#;
    let mut m: MenuCategory = facet_json::from_str_jsonc(text).unwrap();
    assert_eq!(
        m.extra.get("group"),
        None,
        "the member's object is not read into the side-car"
    );
    let value: Value = facet_json::from_str_jsonc(text).unwrap();
    let unknown = json::keep_unknown_members(&mut m, &value);
    assert_eq!(unknown.len(), 1);
    assert_eq!(compact(&m), r#"{"category":"items","extra":{"group":"x"}}"#);
}

#[test]
fn construction_is_the_construction_tab_without_a_group() {
    let m = MenuCategory::construction();
    assert_eq!(m.category, Some(CreativeCategory::Construction));
    assert_eq!(compact(&m), r#"{"category":"construction"}"#);
}
