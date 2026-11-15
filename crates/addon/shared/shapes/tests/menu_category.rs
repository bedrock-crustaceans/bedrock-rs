//! `MenuCategory` and `CreativeCategory`: the object round trips; the five
//! category spellings; and the per-family constraints are not the type's.

use bedrock_addon_test_support::value as common;

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::Unknown;
use bedrock_addon_core::json::{self, StrictBool, Value};
use bedrock_addon_shared_shapes::{CreativeCategory, MenuCategory};
use common::{compact, read, round_trip};

/// [`compact`] with a document's unknown members.
fn compact_with(value: &MenuCategory, unknown: &Unknown) -> String {
    let mut out = Vec::new();
    let style = json::WriteStyle {
        indent: json::Indent::Compact,
        trailing_newline: false,
    };
    json::write_json_with(value, unknown, &mut out, style).unwrap();
    String::from_utf8(out).unwrap()
}

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
            read::<CreativeCategory>(json).is_none(),
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
            read::<MenuCategory>(&text).is_none(),
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

/// `text` read as a `MenuCategory` by the document reading: the unknown members it keeps.
fn kept_unknown(text: &str) -> (usize, Unknown) {
    use bedrock_addon_core::json::kit::{Node, Read};
    let parsed = json::parse::parse(text, json::parse::Options::default()).unwrap();
    let mut d = bedrock_addon_core::diagnostic::Diagnostics::default();
    {
        let mut read = Read::new(bedrock_addon_core::versioning::Reading::DOCUMENT, &mut d);
        let _ = bedrock_addon_core::schema::read_document::<MenuCategory>(
            &Node::parsed(&parsed, JsonPath::new()),
            &mut read,
        );
    }
    let told = d
        .iter()
        .filter(|f| f.is(&bedrock_addon_core::diagnostic::codes::UNRECOGNIZED_FIELD))
        .count();
    (told, d.unknown().clone())
}

#[test]
fn unknown_members_are_kept_by_the_family_walk() {
    let text = r#"{"category": "items", "note": 1}"#;
    let m: MenuCategory = read(text).unwrap();
    let (unknown, kept) = kept_unknown(text);
    assert_eq!(unknown, 1);
    assert_eq!(kept.get(&JsonPath::new(), "note"), Some(&Value::from(1i64)));
    assert_eq!(compact_with(&m, &kept), r#"{"category":"items","note":1}"#);
}

/// A member named like the skipped `extra` field is an unknown member.
#[test]
fn a_member_named_extra_is_an_unknown_member() {
    let text = r#"{"category": "items", "extra": {"group": "x"}}"#;
    let m: MenuCategory = read(text).unwrap();
    assert_eq!(
        compact(&m),
        r#"{"category":"items"}"#,
        "the member's object is not read into the side-car"
    );
    let (unknown, kept) = kept_unknown(text);
    assert_eq!(unknown, 1);
    assert_eq!(
        compact_with(&m, &kept),
        r#"{"category":"items","extra":{"group":"x"}}"#
    );
}

#[test]
fn construction_is_the_construction_tab_without_a_group() {
    let m = MenuCategory::construction();
    assert_eq!(m.category, Some(CreativeCategory::Construction));
    assert_eq!(compact(&m), r#"{"category":"construction"}"#);
}
