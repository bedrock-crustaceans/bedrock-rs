//! What fails a file once its envelope is accepted, and what only
//! warns. Each test builds the smallest document for one rule.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, Severity};

use super::common::*;

const GROUP_NAME_TEXT: &str = "string must be prefixed with a namespace (eg. namespace:value)";
const ITEM_NAME_TEXT: &str = "expected string to match the \"^(?:.)+:(?:.)+$\" regex";
const ROOT: &str = "minecraft:crafting_items_catalog";

/// A 1.21.60 document whose payload is `payload` (JSON text).
fn document(payload: &str) -> String {
    format!(r#"{{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {payload}}}"#)
}

/// A document with one category whose members are `members`.
fn with_category(members: &str) -> String {
    document(&format!(r#"{{"categories": [{{{members}}}]}}"#))
}

/// A document with one group, in `items`, whose members are `members`.
fn with_group(members: &str) -> String {
    with_category(&format!(
        r#""category_name": "items", "groups": [{{{members}}}]"#
    ))
}

/// A document with one item, `item` (JSON text).
fn with_item(item: &str) -> String {
    with_group(&format!(r#""items": [{item}]"#))
}

/// A document with one named group whose `group_identifier` is `identifier` (JSON text).
fn with_identifier(identifier: &str) -> String {
    with_group(&format!(r#""group_identifier": {identifier}, "items": []"#))
}

/// The one error of the rejection of `json`: its code name, path and text.
#[track_caller]
fn error(json: &str) -> (&'static str, String, String) {
    let found: Diagnostics = rejects(json);
    let error = one_error(&found);
    (
        error.code.name(),
        error.location.json_path.to_string(),
        error.code.to_string(),
    )
}

#[track_caller]
fn missing(json: &str, path: &str) {
    let (code, at, _) = error(json);
    assert_eq!(
        (code, at.as_str()),
        ("MissingRequiredField", path),
        "{json}"
    );
}

#[track_caller]
fn wrong_type(json: &str, path: &str) {
    let (code, at, _) = error(json);
    assert_eq!((code, at.as_str()), ("MalformedInput", path), "{json}");
}

#[test]
fn l5_every_required_member_is_required() {
    missing(&document("{}"), &format!("{ROOT}/categories"));
    missing(
        &with_category(r#""groups": []"#),
        &format!("{ROOT}/categories/0/category_name"),
    );
    missing(
        &with_category(r#""category_name": "items""#),
        &format!("{ROOT}/categories/0/groups"),
    );
    missing(
        &with_group(""),
        &format!("{ROOT}/categories/0/groups/0/items"),
    );
    missing(
        &with_identifier(r#"{"icon": "a:b"}"#),
        &format!("{ROOT}/categories/0/groups/0/group_identifier/name"),
    );
    missing(
        &with_item("{}"),
        &format!("{ROOT}/categories/0/groups/0/items/0/name"),
    );
    missing(
        &with_identifier(r#"{"name": "a:b", "icon": {}}"#),
        &format!("{ROOT}/categories/0/groups/0/group_identifier/icon/name"),
    );
}

#[test]
fn l5_the_optional_members_are_optional() {
    let document = loads(&with_group(r#""items": []"#));
    assert!(document.categories[0].groups[0].group_identifier.is_none());
    let document = loads(&with_identifier(r#"{"name": "a:b"}"#));
    let identifier = document.categories[0].groups[0]
        .group_identifier
        .as_ref()
        .unwrap();
    assert!(identifier.icon.is_none());
    assert!(document.diagnostics().is_empty());
}

#[test]
fn l6_the_four_category_names_are_read_with_case() {
    for name in ["construction", "nature", "equipment", "items"] {
        let document = loads(&with_category(&format!(
            r#""category_name": "{name}", "groups": []"#
        )));
        assert_eq!(document.categories[0].category_name.name(), name);
    }
    for name in ["Construction", "ITEMS", "none", "all", "", "items "] {
        let (code, at, _) = error(&with_category(&format!(
            r#""category_name": "{name}", "groups": []"#
        )));
        assert_eq!(
            (code, at.as_str()),
            (
                "InvalidValue",
                "minecraft:crafting_items_catalog/categories/0/category_name"
            ),
            "{name}"
        );
    }
}

#[test]
fn l7_a_group_name_needs_a_namespace() {
    for name in [
        "a:b",
        ":::",
        "a:b:c",
        "MINECRAFT:Group",
        "minecraft:itemGroup.name.planks",
        "a b:c d",
    ] {
        loads(&with_identifier(&format!(r#"{{"name": "{name}"}}"#)));
    }
    for name in ["group", "", ":", "::", ":b", "a:", "a:b\\nc", "a\\r:b"] {
        let (code, at, text) = error(&with_identifier(&format!(r#"{{"name": "{name}"}}"#)));
        assert_eq!(code, "InvalidValue", "{name:?}");
        assert_eq!(
            at,
            format!("{ROOT}/categories/0/groups/0/group_identifier/name")
        );
        assert_eq!(text, GROUP_NAME_TEXT);
    }
}

#[test]
fn l8_an_item_name_needs_a_namespace_in_both_forms_and_as_an_icon() {
    for name in ["a:b", ":::", "a:b:1", "Custom:Item", "a:b:c:d"] {
        loads(&with_item(&format!(r#""{name}""#)));
        loads(&with_item(&format!(r#"{{"name": "{name}"}}"#)));
        loads(&with_identifier(&format!(
            r#"{{"name": "g:g", "icon": "{name}"}}"#
        )));
    }
    for name in ["item", "", ":", "::", ":b", "a:", "a:b\\nc", "a\\r:b"] {
        let items = format!("{ROOT}/categories/0/groups/0/items/0");
        let (code, at, text) = error(&with_item(&format!(r#""{name}""#)));
        assert_eq!(
            (code, at.as_str(), text.as_str()),
            ("InvalidValue", items.as_str(), ITEM_NAME_TEXT),
            "{name:?}"
        );
        let (code, at, text) = error(&with_item(&format!(r#"{{"name": "{name}"}}"#)));
        assert_eq!(
            (code, at, text.as_str()),
            ("InvalidValue", format!("{items}/name"), ITEM_NAME_TEXT),
            "{name:?}"
        );
        let (code, at, text) = error(&with_identifier(&format!(
            r#"{{"name": "g:g", "icon": "{name}"}}"#
        )));
        assert_eq!(
            (code, at, text.as_str()),
            (
                "InvalidValue",
                format!("{ROOT}/categories/0/groups/0/group_identifier/icon"),
                ITEM_NAME_TEXT
            ),
            "{name:?}"
        );
    }
}

/// facet reads `null` as an absent option, `[]` as an empty map and coerces scalars; vanilla's
/// reader fails the file on each, so every level checks its token.
#[test]
fn l9_a_wrong_json_type_fails_the_file() {
    for payload in ["[]", "null", "\"x\"", "5", "true"] {
        wrong_type(&document(payload), ROOT);
    }
    for categories in ["{}", "null", "\"x\"", "5"] {
        wrong_type(
            &document(&format!(r#"{{"categories": {categories}}}"#)),
            &format!("{ROOT}/categories"),
        );
    }
    for category in ["[]", "null", "\"items\"", "5"] {
        wrong_type(
            &document(&format!(r#"{{"categories": [{category}]}}"#)),
            &format!("{ROOT}/categories/0"),
        );
    }
    for name in ["3", "null", "true", "[\"items\"]", "{}"] {
        wrong_type(
            &with_category(&format!(r#""category_name": {name}, "groups": []"#)),
            &format!("{ROOT}/categories/0/category_name"),
        );
    }
    for groups in ["{}", "null", "\"x\"", "0"] {
        wrong_type(
            &with_category(&format!(r#""category_name": "items", "groups": {groups}"#)),
            &format!("{ROOT}/categories/0/groups"),
        );
    }
    for group in ["[]", "null", "\"x\"", "1"] {
        wrong_type(
            &with_category(&format!(r#""category_name": "items", "groups": [{group}]"#)),
            &format!("{ROOT}/categories/0/groups/0"),
        );
    }
    for identifier in ["null", "[]", "\"a:b\"", "1", "false"] {
        wrong_type(
            &with_identifier(identifier),
            &format!("{ROOT}/categories/0/groups/0/group_identifier"),
        );
    }
    for name in ["null", "1", "1.5", "true", "[]", "{}"] {
        wrong_type(
            &with_identifier(&format!(r#"{{"name": {name}}}"#)),
            &format!("{ROOT}/categories/0/groups/0/group_identifier/name"),
        );
        wrong_type(
            &with_item(&format!(r#"{{"name": {name}}}"#)),
            &format!("{ROOT}/categories/0/groups/0/items/0/name"),
        );
    }
    for items in ["{}", "null", "\"a:b\"", "1"] {
        wrong_type(
            &with_group(&format!(r#""items": {items}"#)),
            &format!("{ROOT}/categories/0/groups/0/items"),
        );
    }
    for item in ["null", "1", "1.5", "true", "[]", "[\"a:b\"]"] {
        wrong_type(
            &with_item(item),
            &format!("{ROOT}/categories/0/groups/0/items/0"),
        );
        wrong_type(
            &with_identifier(&format!(r#"{{"name": "a:b", "icon": {item}}}"#)),
            &format!("{ROOT}/categories/0/groups/0/group_identifier/icon"),
        );
    }
}

#[test]
fn l9_every_failure_of_a_file_is_reported() {
    let found = rejects(&document(
        r#"{"categories": [{"category_name": "blocks", "groups": [{"items": ["stone", 5]}]}, 7]}"#,
    ));
    let codes: Vec<&str> = errors(&found).iter().map(|f| f.code.name()).collect();
    assert_eq!(
        codes,
        [
            "InvalidValue",
            "InvalidValue",
            "MalformedInput",
            "MalformedInput"
        ]
    );
}

#[test]
fn l10_an_unknown_member_at_any_level_is_a_warning() {
    let text = document(
        r#"{"a": 1, "categories": [{"b": 1, "category_name": "items", "groups": [{"c": 1,
            "group_identifier": {"d": 1, "name": "g:g", "icon": {"e": 1, "name": "i:i"}},
            "items": [{"f": 1, "name": "i:i"}]}]}]}"#,
    );
    let document = loads(&text);
    let paths: Vec<String> = document
        .diagnostics()
        .iter()
        .map(|f| {
            assert!(matches!(
                f.code,
                DiagnosticCode::UnrecognizedField { text: None }
            ));
            assert_eq!(f.severity, Severity::Warning);
            f.location.json_path.to_string()
        })
        .collect();
    assert_eq!(
        paths,
        [
            format!("{ROOT}/a"),
            format!("{ROOT}/categories/0/b"),
            format!("{ROOT}/categories/0/groups/0/c"),
            format!("{ROOT}/categories/0/groups/0/group_identifier/d"),
            format!("{ROOT}/categories/0/groups/0/group_identifier/icon/e"),
            format!("{ROOT}/categories/0/groups/0/items/0/f"),
        ]
    );
    // Told, never kept.
    assert!(document.unknown().is_empty());
}

/// A member literally named `extra` is an unknown member like any other (this reader has
/// no skipped fields).
#[test]
fn l10_a_member_named_extra_is_an_unknown_member() {
    let document = loads(&with_group(r#""items": [], "extra": {"x": 1}"#));
    assert_eq!(document.diagnostics().iter().count(), 1);
    assert!(document.unknown().is_empty());
}

/// The constraints vanilla does not have: no size limits, no
/// uniqueness at parse time, no rejection of the `minecraft` namespace.
#[test]
fn what_is_not_a_rule_raises_nothing() {
    let document = loads(&document(
        r#"{"categories": [
            {"category_name": "items", "groups": [
                {"group_identifier": {"name": "minecraft:mine"}, "items": ["a:b", "a:b"]},
                {"group_identifier": {"name": "minecraft:mine"}, "items": []}]},
            {"category_name": "items", "groups": []}]}"#,
    ));
    assert!(document.diagnostics().is_empty());
    assert_eq!(document.categories.len(), 2);
    assert_eq!(document.categories[0].groups[0].items.len(), 2);
}
