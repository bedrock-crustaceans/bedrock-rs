//! The event-tree walker: the nested lists of an event node in the order the explicit-stack
//! walkers visit them, whether the game ignores the node's `sequence`, and the finding it logs for that.

use bedrock_addon_core::diagnostic::{
    DiagnosticCode, EVENT_BOTH_RANDOMIZE_AND_SEQUENCE, JsonPath, Phase, Severity, VanillaText,
};
use bedrock_addon_core::json::Value;
use bedrock_addon_shared_shapes::event::{self, LISTS};

/// The node `json`.
fn node(json: &str) -> Value {
    bedrock_addon_core::json::from_str_jsonc(json).expect("JSON")
}

/// The keys and lengths of `node`'s lists, in the order given.
fn shape<'a>(
    lists: impl Iterator<Item = (&'static str, &'a [Value])>,
) -> Vec<(&'static str, usize)> {
    lists.map(|(key, list)| (key, list.len())).collect()
}

#[test]
fn a_sequence_node_lists_its_sequence() {
    let value = node(r#"{"sequence": [{"add": {}}, {"trigger": "a"}]}"#);
    let children = event::children(&value);
    assert_eq!(shape(children.lists.iter().copied()), [("sequence", 2)]);
    assert!(children.has_collection);
    assert!(!children.sequence_ignored);
}

#[test]
fn a_randomize_node_lists_its_randomize() {
    let value = node(r#"{"randomize": [{"weight": 1}, {"weight": 3}, {}]}"#);
    let children = event::children(&value);
    assert_eq!(shape(children.lists.iter().copied()), [("randomize", 3)]);
    assert!(children.has_collection);
    assert!(!children.sequence_ignored);
}

#[test]
fn a_first_valid_node_lists_its_first_valid() {
    let value = node(r#"{"first_valid": [{"filters": {}}]}"#);
    let children = event::children(&value);
    assert_eq!(shape(children.lists.iter().copied()), [("first_valid", 1)]);
    assert!(children.has_collection);
    assert!(!children.sequence_ignored);
}

#[test]
fn a_leaf_node_has_no_lists() {
    // The flattened responses of a leaf: every member is a response, none a list.
    let value = node(r#"{"add": {"component_groups": ["a"]}, "trigger": "b", "weight": 2}"#);
    let children = event::children(&value);
    assert!(children.lists.is_empty());
    assert!(!children.has_collection);
    assert!(!children.sequence_ignored);
    // Not an object: nothing at all.
    let number = Value::from(3.0);
    let children = event::children(&number);
    assert!(children.lists.is_empty());
    assert!(!children.has_collection);
}

#[test]
fn the_lists_come_in_the_walkers_order_whatever_the_documents() {
    let value = node(r#"{"first_valid": [{}], "randomize": [{}, {}], "sequence": [{}, {}, {}]}"#);
    let children = event::children(&value);
    assert_eq!(LISTS, ["sequence", "randomize", "first_valid"]);
    assert_eq!(
        shape(children.lists.iter().copied()),
        [("sequence", 3), ("randomize", 2), ("first_valid", 1)]
    );
}

#[test]
fn a_list_that_is_no_array_is_not_listed_but_counts_as_a_collection() {
    let value = node(r#"{"sequence": {"add": {}}, "first_valid": null}"#);
    let children = event::children(&value);
    assert!(children.lists.is_empty());
    assert!(children.has_collection);
}

#[test]
fn a_node_with_both_lists_ignores_its_sequence_but_lists_both() {
    let value = node(r#"{"sequence": [{}], "randomize": [{}, {}]}"#);
    let children = event::children(&value);
    assert!(children.sequence_ignored);
    // Both are listed: entity validates both.
    assert_eq!(
        shape(children.lists.iter().copied()),
        [("sequence", 1), ("randomize", 2)]
    );
    // The test is presence, any JSON type: an empty or non-array sequence still counts.
    let value = node(r#"{"sequence": [], "randomize": {}}"#);
    assert!(event::children(&value).sequence_ignored);
    let value = node(r#"{"sequence": [{}], "first_valid": [{}]}"#);
    assert!(!event::children(&value).sequence_ignored);
}

#[test]
fn the_two_tests_read_alone_agree_with_the_children() {
    for json in [
        r#"{"sequence": [{}], "randomize": [{}, {}]}"#,
        r#"{"sequence": [], "randomize": {}}"#,
        r#"{"sequence": [{}], "first_valid": [{}]}"#,
        r#"{"sequence": {"add": {}}, "first_valid": null}"#,
        r#"{"randomize": [{"weight": 1}]}"#,
        r#"{"add": {"component_groups": ["a"]}, "trigger": "b"}"#,
        "3",
    ] {
        let value = node(json);
        let children = event::children(&value);
        assert_eq!(
            event::is_sequence_ignored(&value),
            children.sequence_ignored,
            "{json}"
        );
        assert_eq!(
            event::has_collection(&value),
            children.has_collection,
            "{json}"
        );
    }
    assert!(event::is_sequence_ignored(&node(
        r#"{"sequence": [], "randomize": {}}"#
    )));
    assert!(!event::has_collection(&node(r#"{"trigger": "b"}"#)));
}

#[test]
fn the_ignored_sequence_is_a_warning_with_the_games_text_at_the_sequence() {
    let mut at = JsonPath::new();
    at.push("events");
    at.push("demo:e");
    let finding = event::sequence_ignored(&at);
    assert_eq!(
        finding.code,
        DiagnosticCode::IgnoredMember {
            text: Some(VanillaText::Fixed(EVENT_BOTH_RANDOMIZE_AND_SEQUENCE)),
        }
    );
    assert_eq!(finding.severity, Severity::Warning);
    assert_eq!(finding.phase, Phase::Parse);
    assert_eq!(finding.location.json_path, at.child("sequence"));
}
