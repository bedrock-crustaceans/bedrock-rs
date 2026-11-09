//! Write-back: a tree is written from its name and root, read
//! again to the same value, and written again to the same text; a member the author did not
//! write is not written.

#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it; not yet moved to the door"
)]

use bedrock_addon::diagnostic::JsonPath;
use bedrock_addon::document::{Content, Document, Placed};
use bedrock_addon::families::behavior_tree::{BehaviorTree, Node, Param};
use bedrock_addon::json::{EncodeError, Indent, Unrepresentable, Value, WriteStyle};

use super::common::*;

const COMPACT: WriteStyle = WriteStyle {
    indent: Indent::Compact,
    trailing_newline: false,
};

fn compact(tree: &BehaviorTree) -> String {
    let mut out = Vec::new();
    tree.write_json(&mut out, COMPACT).expect("writes");
    String::from_utf8(out).unwrap()
}

/// The root node `node` written in a tree named `t`, as the text of the node alone.
fn written(node: Node) -> String {
    let text = compact(&BehaviorTree::new(id("t"), node));
    let prefix = r#"{"minecraft:behavior_tree":{"minecraft:name":"t","minecraft:root":"#;
    text.strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix("}}"))
        .unwrap_or_else(|| panic!("{text}"))
        .to_owned()
}

/// The root node `{"<type>": <body>}` read and written again, with its unknown members.
fn rewritten(node: &str) -> String {
    let text = compact_document(&loaded(&tree_text("t", node)));
    let prefix = r#"{"minecraft:behavior_tree":{"minecraft:name":"t","minecraft:root":"#;
    text.strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix("}}"))
        .unwrap_or_else(|| panic!("{text}"))
        .to_owned()
}

/// The document written compact, with its unknown members.
fn compact_document(document: &Document<BehaviorTree>) -> String {
    let mut out = Vec::new();
    document.write_json(&mut out, COMPACT).expect("writes");
    String::from_utf8(out).unwrap()
}

/// The path of the root node's body of type `kind`.
fn root_body(kind: &str) -> JsonPath {
    JsonPath::of_keys(["minecraft:behavior_tree", "minecraft:root", kind])
}

/// Every fixture's stack: the name of the fixture and its stack.
fn fixture_names() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with("bt"))
        .collect();
    names.sort();
    names
}

#[test]
fn every_tree_of_every_fixture_reads_back_equal_and_rewrites_identically() {
    let mut trees = 0;
    for fixture in fixture_names() {
        let stack = fixture_stack(&fixture);
        for (name, document) in stack.behavior_trees() {
            let text = document.to_json().expect("writes");
            let again = parse(&text).unwrap_or_else(|r| panic!("{fixture} {name}: {text}\n{r:#?}"));
            assert_eq!(again.name, document.name, "{fixture} {name}");
            assert_eq!(again.root, document.root, "{fixture} {name}:\n{text}");
            assert_eq!(again.unknown(), document.unknown(), "{fixture} {name}");
            // the raw text is the file's: of the rewritten file, the rewritten text
            assert_eq!(again.raw, text);
            assert_eq!(again.to_json().unwrap(), text, "{fixture} {name}");
            // nothing is written that reads with a finding the file did not have
            for finding in again.diagnostics() {
                assert!(
                    document.diagnostics().iter().any(|d| d.code == finding.code
                        && d.location.json_path == finding.location.json_path),
                    "{fixture} {name}: a new finding {finding:?}\n{text}"
                );
            }
            trees += 1;
        }
    }
    // the 22 vanilla trees, the fixtures' own, and the nine built-in ones of every stack
    assert!(trees > 22 + 9 * 20, "{trees}");
}

#[test]
fn a_vanilla_tree_is_written_with_the_members_its_author_wrote() {
    let stack = fixture_stack("bt01_vanilla_22");
    let tree = stack.behavior_trees().get(&id("collect_items")).unwrap();
    assert_eq!(
        compact(tree),
        concat!(
            r#"{"minecraft:behavior_tree":{"minecraft:name":"collect_items","minecraft:root":"#,
            r#"{"minecraft:sequence":{"children":["#,
            r#"{"minecraft:find_entity":{"entity_name":"item"}},"#,
            r#"{"minecraft:move_to":{"block_pos_id":"target_pos","distance_epsilon":1.5}},"#,
            r#"{"minecraft:wait":{"wait_duration":1}}"#,
            r#"]}}}}"#
        )
    );
    // the default style: two spaces and a final line feed
    let pretty = tree.to_json().unwrap();
    assert!(pretty.starts_with(
        "{\n  \"minecraft:behavior_tree\": {\n    \"minecraft:name\": \"collect_items\","
    ));
    assert!(pretty.ends_with("}\n"));
    let mut out = Vec::new();
    tree.write_json(&mut out, WriteStyle::default()).unwrap();
    assert_eq!(out, pretty.as_bytes());
}

#[test]
fn a_built_in_tree_is_written_without_a_member_of_its_own() {
    let stack = fixture_stack("bt02_builtins_only");
    let trees = stack.behavior_trees();
    assert_eq!(
        compact(trees.get(&id("find_block")).unwrap()),
        concat!(
            r#"{"minecraft:behavior_tree":{"minecraft:name":"find_block","minecraft:root":"#,
            r#"{"minecraft:repeat_until_success":{"child":{"minecraft:find_block":{}}}}}}"#
        )
    );
    assert_eq!(
        compact(trees.get(&id("fly")).unwrap()),
        r#"{"minecraft:behavior_tree":{"minecraft:name":"fly","minecraft:root":{"minecraft:fly":{}}}}"#
    );
}

#[test]
fn a_literal_is_written_when_the_key_is_empty_and_the_key_when_it_is_not_the_default() {
    let ticks = |ticks: Param<i32>| written(Node::WaitTicks { ticks });
    assert_eq!(
        ticks(Param::literal(0)),
        r#"{"minecraft:wait_ticks":{"num_of_ticks":0}}"#
    );
    assert_eq!(
        ticks(Param::literal(-7)),
        r#"{"minecraft:wait_ticks":{"num_of_ticks":-7}}"#
    );
    assert_eq!(
        ticks(Param::key("elsewhere")),
        r#"{"minecraft:wait_ticks":{"num_of_ticks_id":"elsewhere"}}"#
    );
    // the default key is what an absent member reads as: nothing is written
    assert_eq!(
        ticks(Param::key("target_num_of_ticks")),
        r#"{"minecraft:wait_ticks":{}}"#
    );
    // the empty key is the one a read literal leaves: it stands for the literal
    assert_eq!(
        ticks(Param::key("")),
        r#"{"minecraft:wait_ticks":{"num_of_ticks":0}}"#
    );
    // a literal beside a key cannot be read back: vanilla reads the key only when the literal
    // is absent, so the key is written
    assert_eq!(
        ticks(Param {
            literal: 5,
            key: "elsewhere".to_owned()
        }),
        r#"{"minecraft:wait_ticks":{"num_of_ticks_id":"elsewhere"}}"#
    );
}

#[test]
fn every_member_form_reads_and_writes_back_as_written() {
    for node in [
        r#"{"minecraft:sequence":{"children":[]}}"#,
        r#"{"minecraft:sequence":{}}"#,
        r#"{"minecraft:selector":{"children":[{"minecraft:use_entity":{}},{"minecraft:place_block":{}}]}}"#,
        r#"{"minecraft:repeat_until_success":{"desired_max_number_of_attempts":3,"child":{"minecraft:consume_item":{}}}}"#,
        r#"{"minecraft:repeat_until_success":{"max_number_of_attempts_id":"tries"}}"#,
        r#"{"minecraft:repeat_until_failure":{"child":{"minecraft:attack":{"num_of_ticks":4}}}}"#,
        r#"{"minecraft:inverter":{}}"#,
        r#"{"minecraft:subtree":{}}"#,
        r#"{"minecraft:subtree":{"subtree_id":"move","data_items":["a",{"from":"b","to":"c"}]}}"#,
        r#"{"minecraft:activate_tool":{"block_pos":[1,-2,3]}}"#,
        r#"{"minecraft:break_block":{"block_pos_id":"where","num_of_ticks":20}}"#,
        r#"{"minecraft:find_block":{"block_name":"log","search_radius":8}}"#,
        r#"{"minecraft:find_block":{"block_name_id":"what","search_radius_id":"far"}}"#,
        r#"{"minecraft:find_entity":{"entity_name":"item","search_radius":8}}"#,
        r#"{"minecraft:find_entity":{"entity_type":"who"}}"#,
        r#"{"minecraft:fly":{"should_be_flying_bool":true}}"#,
        r#"{"minecraft:fly":{"should_be_flying_id":"f","should_throw_event_if_no_state_change_necessary_bool":false}}"#,
        r#"{"minecraft:fly":{"should_throw_event_if_no_state_change_necessary_id":"t"}}"#,
        r#"{"minecraft:get_interaction_position_for_block":{"target_block_pos":[1,2,3],"anchor_block_pos_id":"a","facing_string":"north","max_search_distance":6}}"#,
        r#"{"minecraft:interact":{"num_of_ticks":2}}"#,
        r#"{"minecraft:look_at":{"block_pos":[0,64,0]}}"#,
        r#"{"minecraft:look_at_entity":{"entity_name":"cow","search_radius_id":"r"}}"#,
        r#"{"minecraft:move":{"desired_direction":"forward","second_direction_id":"side","max_number_of_ticks":100}}"#,
        r#"{"minecraft:move_to":{"block_pos":[0.5,-4.0,2.25],"distance_epsilon":0.1}}"#,
        r#"{"minecraft:shoot_bow":{"num_of_shots":3}}"#,
        r#"{"minecraft:wait":{"wait_duration":3}}"#,
        r#"{"minecraft:wait":{"wait_duration":"secs"}}"#,
        r#"{"minecraft:wait":{}}"#,
        r#"{"minecraft:wait_ticks":{"num_of_ticks":2,"ticks":3,"note":{"a":[1]}}}"#,
    ] {
        assert_eq!(rewritten(node), node);
    }
}

/// What vanilla ignores is not written back: the tree reads the same without it.
#[test]
fn members_vanilla_ignores_are_left_out() {
    for (node, rewritten_as) in [
        // an ignored literal, an explicit default key, a key member beside a read literal
        (
            r#"{"minecraft:wait_ticks":{"num_of_ticks":"soon"}}"#,
            r#"{"minecraft:wait_ticks":{}}"#,
        ),
        (
            r#"{"minecraft:wait_ticks":{"num_of_ticks_id":"target_num_of_ticks"}}"#,
            r#"{"minecraft:wait_ticks":{}}"#,
        ),
        (
            r#"{"minecraft:wait_ticks":{"num_of_ticks":1,"num_of_ticks_id":"k"}}"#,
            r#"{"minecraft:wait_ticks":{"num_of_ticks":1}}"#,
        ),
        // a bool is the integer it reads as
        (
            r#"{"minecraft:wait_ticks":{"num_of_ticks":true}}"#,
            r#"{"minecraft:wait_ticks":{"num_of_ticks":1}}"#,
        ),
        // children that is no array, a child that is no node
        (
            r#"{"minecraft:sequence":{"children":{}}}"#,
            r#"{"minecraft:sequence":{}}"#,
        ),
        (
            r#"{"minecraft:inverter":{"child":[]}}"#,
            r#"{"minecraft:inverter":{}}"#,
        ),
        // an empty block name reads as the absent one
        (
            r#"{"minecraft:find_block":{"block_name":""}}"#,
            r#"{"minecraft:find_block":{}}"#,
        ),
        // both fly literals are the one value vanilla keeps
        (
            r#"{"minecraft:fly":{"should_be_flying_bool":true,"should_throw_event_if_no_state_change_necessary_bool":false}}"#,
            r#"{"minecraft:fly":{"should_be_flying_bool":false,"should_throw_event_if_no_state_change_necessary_bool":false}}"#,
        ),
        // one data item is written as an array of one; an item that is none is gone
        (
            r#"{"minecraft:subtree":{"data_items":"a"}}"#,
            r#"{"minecraft:subtree":{"data_items":["a"]}}"#,
        ),
        (
            r#"{"minecraft:subtree":{"subtree_id":"","data_items":[7]}}"#,
            r#"{"minecraft:subtree":{}}"#,
        ),
        // a body that is no object
        (
            r#"{"minecraft:place_block":5}"#,
            r#"{"minecraft:place_block":{}}"#,
        ),
    ] {
        assert_eq!(rewritten(node), rewritten_as, "{node}");
        // and the rewritten text reads to the same node
        assert_eq!(root_of(node).0, root_of(rewritten_as).0, "{node}");
    }
}

#[test]
fn unknown_members_are_written_after_the_known_ones_of_their_level() {
    let text = r#"{"format_version":"1.99.0","minecraft:behavior_tree":{"note":1,"minecraft:name":"t","minecraft:root":{"minecraft:wait":{"zeta":true,"wait_duration":2}}},"other":[]}"#;
    let document = loaded(text);
    assert_eq!(
        compact_document(&document),
        r#"{"minecraft:behavior_tree":{"minecraft:name":"t","minecraft:root":{"minecraft:wait":{"wait_duration":2,"zeta":true}},"note":1},"format_version":"1.99.0","other":[]}"#
    );
    // an unknown member named like one the writer writes does not appear twice
    let mut document = loaded(&tree_text(
        "t",
        r#"{"minecraft:wait_ticks": {"num_of_ticks": 4}}"#,
    ));
    assert_eq!(
        document.root,
        Node::WaitTicks {
            ticks: Param::literal(4),
        }
    );
    let body = root_body("minecraft:wait_ticks");
    document
        .unknown_mut()
        .put(&body, "num_of_ticks", Value::from("kept out"));
    document
        .unknown_mut()
        .put(&body, "why", Value::from("kept"));
    let text = compact_document(&document);
    let prefix = r#"{"minecraft:behavior_tree":{"minecraft:name":"t","minecraft:root":"#;
    assert_eq!(
        text.strip_prefix(prefix)
            .and_then(|rest| rest.strip_suffix("}}")),
        Some(r#"{"minecraft:wait_ticks":{"num_of_ticks":4,"why":"kept"}}"#)
    );
}

#[test]
fn a_number_without_a_json_spelling_is_not_written() {
    for (value, what) in [
        (f32::NAN, Unrepresentable::NaN),
        (f32::INFINITY, Unrepresentable::Infinity),
        (f32::NEG_INFINITY, Unrepresentable::NegativeInfinity),
    ] {
        let tree = BehaviorTree::new(
            id("t"),
            Node::MoveTo {
                pos: Param::literal([0.0, value, 0.0]),
                distance_epsilon: Param::key("e"),
            },
        );
        let mut out = b"before".to_vec();
        let error = tree.write_json(&mut out, COMPACT).unwrap_err();
        let EncodeError::NotRepresentable { at, what: found } = error else {
            panic!()
        };
        assert_eq!(found, what);
        assert_eq!(
            at.to_string(),
            "minecraft:behavior_tree/minecraft:root/minecraft:move_to/block_pos/1"
        );
        // nothing was appended
        assert_eq!(out, b"before");
        assert!(tree.to_json().is_err());
    }
    let tree = BehaviorTree::new(
        id("t"),
        Node::MoveTo {
            pos: Param::key("p"),
            distance_epsilon: Param::literal(f32::NAN),
        },
    );
    let EncodeError::NotRepresentable { at, .. } = tree.to_json().unwrap_err() else {
        panic!()
    };
    assert_eq!(
        at.to_string(),
        "minecraft:behavior_tree/minecraft:root/minecraft:move_to/distance_epsilon"
    );
    // a float behind a key is not written, so it cannot fail
    let tree = BehaviorTree::new(
        id("t"),
        Node::MoveTo {
            pos: Param::key("p"),
            distance_epsilon: Param {
                literal: f32::NAN,
                key: "e".to_owned(),
            },
        },
    );
    assert!(tree.to_json().is_ok());
}

#[test]
fn a_tree_built_in_memory_is_placed_and_written() {
    let tree = BehaviorTree::new(
        id("chop_wood"),
        Node::Sequence {
            children: Some(vec![
                Node::FindBlock {
                    block_name: Param::literal("log".to_owned()),
                    search_radius: Param::key("target_search_radius"),
                },
                Node::BreakBlock {
                    block_pos: Param::key("target_block_pos"),
                    ticks: Param::key("target_num_of_ticks"),
                },
            ]),
        },
    );
    assert_eq!(tree.raw, "");
    assert_eq!(
        tree.default_path().as_str(),
        "behavior_trees/chop_wood.json"
    );
    assert_eq!(
        compact(&tree),
        r#"{"minecraft:behavior_tree":{"minecraft:name":"chop_wood","minecraft:root":{"minecraft:sequence":{"children":[{"minecraft:find_block":{"block_name":"log"}},{"minecraft:break_block":{}}]}}}}"#
    );
    assert_eq!(
        Param::<i32>::key("k"),
        Param {
            literal: 0,
            key: "k".to_owned()
        }
    );
    assert_eq!(
        Param::literal(3),
        Param {
            literal: 3,
            key: String::new()
        }
    );
}

/// The node read from `node`, and the node read from what it is written as.
fn read_twice(node: &str) -> (Node, Node, String) {
    let first = root_of(node).0;
    let text = written(first.clone());
    let second = root_of(&text).0;
    (first, second, text)
}

/// The key member's text is the empty string for a key member that is no
/// string, and for `find_block`'s block name that is not the preset key a read literal has.
#[test]
fn a_block_name_whose_key_is_empty_reads_back_equal() {
    for body in [
        r#"{"block_name_id": 5}"#,
        r#"{"block_name_id": ""}"#,
        r#"{"block_name": 5, "block_name_id": 5}"#,
        r#"{"block_name_id": []}"#,
    ] {
        let (first, second, text) = read_twice(&format!(r#"{{"minecraft:find_block": {body}}}"#));
        let Node::FindBlock { block_name, .. } = &first else {
            panic!()
        };
        assert_eq!(block_name, &Param::literal(String::new()), "{body}");
        assert_eq!(second, first, "{body} was written as {text}");
    }
}

// (node type, literal member, a literal of its type, key member)
const PAIRS: [(&str, &str, &str, &str); 27] = [
    (
        "minecraft:repeat_until_success",
        "desired_max_number_of_attempts",
        "4",
        "max_number_of_attempts_id",
    ),
    (
        "minecraft:activate_tool",
        "block_pos",
        "[1, -2, 3]",
        "block_pos_id",
    ),
    ("minecraft:attack", "num_of_ticks", "7", "num_of_ticks_id"),
    (
        "minecraft:break_block",
        "block_pos",
        "[1, -2, 3]",
        "block_pos_id",
    ),
    (
        "minecraft:break_block",
        "num_of_ticks",
        "7",
        "num_of_ticks_id",
    ),
    (
        "minecraft:find_block",
        "block_name",
        "\"log\"",
        "block_name_id",
    ),
    (
        "minecraft:find_block",
        "search_radius",
        "5",
        "search_radius_id",
    ),
    (
        "minecraft:find_entity",
        "entity_name",
        "\"item\"",
        "entity_type",
    ),
    (
        "minecraft:find_entity",
        "search_radius",
        "5",
        "search_radius_id",
    ),
    (
        "minecraft:fly",
        "should_be_flying_bool",
        "true",
        "should_be_flying_id",
    ),
    (
        "minecraft:fly",
        "should_throw_event_if_no_state_change_necessary_bool",
        "true",
        "should_throw_event_if_no_state_change_necessary_id",
    ),
    (
        "minecraft:get_interaction_position_for_block",
        "target_block_pos",
        "[1, -2, 3]",
        "target_block_pos_id",
    ),
    (
        "minecraft:get_interaction_position_for_block",
        "anchor_block_pos",
        "[4, 5, 6]",
        "anchor_block_pos_id",
    ),
    (
        "minecraft:get_interaction_position_for_block",
        "facing_string",
        "\"north\"",
        "facing_string_id",
    ),
    (
        "minecraft:get_interaction_position_for_block",
        "max_search_distance",
        "9",
        "max_search_distance_id",
    ),
    ("minecraft:interact", "num_of_ticks", "7", "num_of_ticks_id"),
    (
        "minecraft:look_at",
        "block_pos",
        "[1, -2, 3]",
        "block_pos_id",
    ),
    (
        "minecraft:look_at_entity",
        "entity_name",
        "\"item\"",
        "entity_type",
    ),
    (
        "minecraft:look_at_entity",
        "search_radius",
        "5",
        "search_radius_id",
    ),
    (
        "minecraft:move",
        "desired_direction",
        "\"forward\"",
        "direction_id",
    ),
    (
        "minecraft:move",
        "desired_second_direction",
        "\"left\"",
        "second_direction_id",
    ),
    (
        "minecraft:move",
        "max_number_of_ticks",
        "100",
        "number_of_ticks_id",
    ),
    (
        "minecraft:move_to",
        "block_pos",
        "[0.5, -4, 2.25]",
        "block_pos_id",
    ),
    (
        "minecraft:move_to",
        "distance_epsilon",
        "1.5",
        "distance_epsilon_id",
    ),
    (
        "minecraft:shoot_bow",
        "num_of_shots",
        "3",
        "num_of_shots_id",
    ),
    ("minecraft:wait", "wait_duration", "3", "wait_duration"),
    (
        "minecraft:wait_ticks",
        "num_of_ticks",
        "7",
        "num_of_ticks_id",
    ),
];
// a value no literal type takes, and every form of a key member
const NO_LITERAL: &str = r#"{"no": "literal"}"#;
const KEYS: [&str; 9] = [
    "\"elsewhere\"",
    "\"\"",
    "5",
    "1.5",
    "true",
    "false",
    "null",
    "[\"k\"]",
    "{}",
];

/// Every literal / key pair of every node type, in every state a file can leave it in: the
/// key member absent, a string, empty, the default key, and of each wrong type, with the
/// literal absent, read, and of a wrong type.
#[test]
fn every_state_of_every_pair_reads_back_equal() {
    let mut states = 0;
    for (kind, literal_key, literal, id_key) in PAIRS {
        let mut bodies = vec![
            "{}".to_owned(),
            format!(r#"{{"{literal_key}": {literal}}}"#),
            format!(r#"{{"{literal_key}": null}}"#),
            format!(r#"{{"{literal_key}": {NO_LITERAL}}}"#),
        ];
        for key in KEYS {
            if literal_key == id_key {
                // `wait`: the one member is the literal or the key
                bodies.push(format!(r#"{{"{id_key}": {key}}}"#));
                continue;
            }
            bodies.push(format!(r#"{{"{id_key}": {key}}}"#));
            bodies.push(format!(
                r#"{{"{literal_key}": {literal}, "{id_key}": {key}}}"#
            ));
            bodies.push(format!(
                r#"{{"{literal_key}": {NO_LITERAL}, "{id_key}": {key}}}"#
            ));
        }
        for body in bodies {
            let (first, second, text) = read_twice(&format!(r#"{{"{kind}": {body}}}"#));
            assert_eq!(second, first, "{kind} {body} was written as {text}");
            // and the rewritten text is stable
            assert_eq!(written(second), text, "{kind} {body}");
            states += 1;
        }
    }
    assert_eq!(states, 26 * (4 + 9 * 3) + 4 + 9);
}

/// The default key written out by the author, for every pair: the same state as the absent
/// member, written as nothing.
#[test]
fn an_explicit_default_key_reads_back_equal() {
    for (kind, id_key, default_key) in [
        ("minecraft:find_block", "block_name_id", "block_name"),
        (
            "minecraft:find_block",
            "search_radius_id",
            "target_search_radius",
        ),
        ("minecraft:find_entity", "entity_type", "entity_name"),
        (
            "minecraft:interact",
            "num_of_ticks_id",
            "target_num_of_ticks",
        ),
        ("minecraft:wait", "wait_duration", "duration"),
        (
            "minecraft:move_to",
            "distance_epsilon_id",
            "target_distance_epsilon",
        ),
        ("minecraft:fly", "should_be_flying_id", "should_be_flying"),
    ] {
        let node = format!(r#"{{"{kind}": {{"{id_key}": "{default_key}"}}}}"#);
        let (first, second, text) = read_twice(&node);
        assert_eq!(second, first, "{node}");
        assert_eq!(text, format!(r#"{{"{kind}":{{}}}}"#), "{node}");
    }
}

/// `node` written and read again.
fn read_back(node: &Node) -> Node {
    root_of(&written(node.clone())).0
}

/// A node for each state of a parameter that is built rather than read.
fn nodes_built_in_memory() -> Vec<Node> {
    let mut nodes = nodes_built_for_the_repeat_and_the_finds();
    nodes.extend(nodes_built_for_the_actions());
    nodes
}

/// The first nodes of [`nodes_built_in_memory`]: repeat, activate, attack, break and the finds.
fn nodes_built_for_the_repeat_and_the_finds() -> Vec<Node> {
    let radius = |key: &str| Param::key_with(key, Node::FAILED_SEARCH_RADIUS);
    let block_name = |name: &str| Param {
        literal: name.to_owned(),
        key: Node::BLOCK_NAME_KEY.to_owned(),
    };
    vec![
        Node::RepeatUntilSuccess {
            child: None,
            max_attempts: Param::key("tries"),
        },
        Node::RepeatUntilSuccess {
            child: Some(Box::new(Node::ConsumeItem {})),
            max_attempts: Param::default(),
        },
        Node::ActivateTool {
            block_pos: Param::key("where"),
        },
        Node::ActivateTool {
            block_pos: Param::default(),
        },
        Node::Attack {
            ticks: Param::key("t"),
        },
        Node::BreakBlock {
            block_pos: Param::literal([1, 2, 3]),
            ticks: Param::key("target_num_of_ticks"),
        },
        Node::FindBlock {
            block_name: block_name("log"),
            search_radius: radius("target_search_radius"),
        },
        Node::FindBlock {
            block_name: Param::key("what"),
            search_radius: radius("far"),
        },
        Node::FindBlock {
            block_name: Param::default(),
            search_radius: Param::literal(0),
        },
        Node::FindEntity {
            entity_name: Param::literal("item".to_owned()),
            search_radius: radius("r"),
        },
        Node::FindEntity {
            entity_name: Param::default(),
            search_radius: Param::default(),
        },
    ]
}

/// The last nodes of [`nodes_built_in_memory`]: fly, interact, look, move, shoot and wait.
fn nodes_built_for_the_actions() -> Vec<Node> {
    let radius = |key: &str| Param::key_with(key, Node::FAILED_SEARCH_RADIUS);
    vec![
        Node::Fly {
            should_be_flying: Param::literal(true),
            throw_if_no_change: Param::key_with("t", true),
        },
        Node::Fly {
            should_be_flying: Param::key("f"),
            throw_if_no_change: Param::key("t"),
        },
        Node::GetInteractionPositionForBlock {
            target: Param::key("a"),
            anchor: Param::literal([4, 5, 6]),
            facing: Param::key("f"),
            max_search_distance: Param::default(),
        },
        Node::Interact {
            ticks: Param::key_with("t", Node::INTERACT_DEFAULT_TICKS),
        },
        Node::Interact {
            ticks: Param::default(),
        },
        Node::LookAt {
            block_pos: Param::key("p"),
        },
        Node::LookAtEntity {
            entity_name: Param::key("who"),
            search_radius: radius("r"),
        },
        Node::Move {
            direction: Param::literal("forward".to_owned()),
            second_direction: Param::key("side"),
            ticks: Param::default(),
        },
        Node::MoveTo {
            pos: Param::key("target_pos"),
            distance_epsilon: Param::literal(1.5),
        },
        Node::MoveTo {
            pos: Param::default(),
            distance_epsilon: Param::key("e"),
        },
        Node::ShootBow {
            shots: Param::key("n"),
        },
        Node::Wait {
            seconds: Param::key("secs"),
        },
        Node::Wait {
            seconds: Param::default(),
        },
        Node::WaitTicks {
            ticks: Param::key("t"),
        },
    ]
}

/// A parameter built in memory reads back equal. `Param::literal`
/// and `Param::default` do for every pair but `find_block`'s block name, whose read literal
/// has the key `Node::BLOCK_NAME_KEY`; `Param::key` does where the literal a failed read
/// leaves is the type's default, and `Param::key_with` carries the two that are not
/// (`Node::FAILED_SEARCH_RADIUS`, `Node::INTERACT_DEFAULT_TICKS`).
#[test]
fn a_parameter_built_in_memory_reads_back_equal() {
    assert_eq!(Node::FAILED_SEARCH_RADIUS, 32);
    assert_eq!(Node::INTERACT_DEFAULT_TICKS, 1);
    assert_eq!(Node::BLOCK_NAME_KEY, "block_name");
    assert_eq!(
        Param::key_with("k", 32),
        Param {
            literal: 32,
            key: "k".to_owned()
        }
    );
    let built = nodes_built_in_memory();
    for node in &built {
        assert_eq!(
            &read_back(node),
            node,
            "written as {}",
            written(node.clone())
        );
    }
}

/// `depth` inverters around a leaf, built without recursion.
fn inverters(depth: usize) -> BehaviorTree {
    let mut node = Node::ConsumeItem {};
    for _ in 0..depth {
        node = Node::Inverter {
            child: Some(Box::new(node)),
        };
    }
    BehaviorTree::new(id("deep"), node)
}

/// `depth` sequences of one child around a leaf.
fn sequences(depth: usize) -> BehaviorTree {
    let mut node = Node::ConsumeItem {};
    for _ in 0..depth {
        node = Node::Sequence {
            children: Some(vec![node]),
        };
    }
    BehaviorTree::new(id("deep"), node)
}

/// A tree built in memory may nest without bound. The writer
/// writes no text the reader's depth limit (128 arrays and objects) would reject, and says so
/// before it walks the tree: no recursion past the limit, no output.
#[test]
fn a_tree_nested_past_the_readers_limit_is_not_written() {
    // the same two ends as the reader: 62 nested decorators are 128 levels
    let text = inverters(62).to_json().expect("128 levels are written");
    assert_eq!(loaded(&text).root, inverters(62).root);
    let refused = |tree: &BehaviorTree| {
        let mut out = b"before".to_vec();
        let error = tree.write_json(&mut out, COMPACT).unwrap_err();
        assert_eq!(out, b"before");
        let EncodeError::NotRepresentable { at, what } = error else {
            panic!()
        };
        assert_eq!(what, Unrepresentable::Unsupported);
        assert_eq!(at.to_string(), "minecraft:behavior_tree/minecraft:root");
    };
    refused(&inverters(63));
    // a composite adds its `children` array: whatever is written reads back, at every depth
    let mut written_depths = 0;
    for depth in 0..=64 {
        let tree = sequences(depth);
        match tree.to_json() {
            Ok(text) => {
                assert_eq!(loaded(&text).root, tree.root, "{depth}");
                written_depths += 1;
            }
            Err(_) => refused(&tree),
        }
    }
    // the root, the payload, three levels per sequence, the leaf and its body: 2 + 3n + 2
    assert_eq!(written_depths, 42);
    // an unknown member's own nesting counts
    let mut deep_extra = String::new();
    for _ in 0..125 {
        deep_extra.push('[');
    }
    for _ in 0..125 {
        deep_extra.push(']');
    }
    let value: Value = facet_json::from_str(&deep_extra).unwrap();
    let with_extra = |value: Value| {
        let mut document = loaded(&tree_text("t", r#"{"minecraft:consume_item": {}}"#));
        document
            .unknown_mut()
            .put(&root_body("minecraft:consume_item"), "deep", value);
        document
    };
    let refused_document = |document: &Document<BehaviorTree>| {
        let mut out = b"before".to_vec();
        let error = document.write_json(&mut out, COMPACT).unwrap_err();
        assert_eq!(out, b"before");
        let EncodeError::NotRepresentable { at, what } = error else {
            panic!()
        };
        assert_eq!(what, Unrepresentable::Unsupported);
        assert_eq!(at.to_string(), "minecraft:behavior_tree/minecraft:root");
    };
    // 4 levels down to the body, and 125 more
    refused_document(&with_extra(value));
    let value: Value = facet_json::from_str(&deep_extra[1..deep_extra.len() - 1]).unwrap();
    assert!(with_extra(value).to_json().is_ok());
    // and far past the limit, quickly and without a deep stack
    let started = std::time::Instant::now();
    let huge = inverters(10_000);
    refused(&huge);
    refused(&sequences(10_000));
    // A guard against a hang, not a cost bound: 6 ms measured in the dev profile,
    // 800 times under the bound.
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
    // such a tree is still walked by `nodes` without recursion
    let mut node = &huge.root;
    let mut depth = 0;
    while let [child] = node.nodes() {
        node = child;
        depth += 1;
    }
    assert_eq!(depth, 10_000);
}

/// A tree built in memory, 200 000 levels deep, on a test thread's default stack: the writer
/// refuses it and it is dropped, neither with a stack frame per level. Nothing here clones,
/// compares or prints the tree: those do recurse per level (see `Node`).
#[test]
fn a_very_deep_tree_is_refused_and_dropped_without_a_deep_stack() {
    const DEPTH: usize = 200_000;
    for tree in [inverters(DEPTH), sequences(DEPTH)] {
        let mut out = Vec::new();
        let refused = matches!(
            tree.write_json(&mut out, COMPACT),
            Err(EncodeError::NotRepresentable {
                what: Unrepresentable::Unsupported,
                ..
            })
        );
        assert!(refused);
        assert!(out.is_empty());
        let mut node = &tree.root;
        let mut depth = 0;
        while let [child] = node.nodes() {
            node = child;
            depth += 1;
        }
        assert_eq!(depth, DEPTH);
        drop(tree);
    }
}

/// `depth` inverters around `leaf`.
fn inverters_around(depth: usize, leaf: Node) -> BehaviorTree {
    let mut node = leaf;
    for _ in 0..depth {
        node = Node::Inverter {
            child: Some(Box::new(node)),
        };
    }
    BehaviorTree::new(id("deep"), node)
}

/// The writer's exact measuring pass: a leaf whose body holds an array (or an array of
/// objects) is one (or two) levels deeper than its body, which the walk over the node bodies
/// does not see. Under 61 decorators the leaf's body is level 126.
#[test]
fn the_members_of_a_leaf_count_towards_the_limit() {
    let look_at = || Node::LookAt {
        block_pos: Param::literal([1, 2, 3]),
    };
    let subtree = || Node::Subtree {
        subtree_id: id("move"),
        data_items: vec![("from".to_owned(), "to".to_owned())],
    };
    let string_items = || Node::Subtree {
        subtree_id: id("move"),
        data_items: vec![("same".to_owned(), "same".to_owned())],
    };
    let refused = |tree: &BehaviorTree| {
        matches!(
            tree.to_json(),
            Err(EncodeError::NotRepresentable {
                what: Unrepresentable::Unsupported,
                ..
            })
        )
    };
    // `[1, 2, 3]` is level 127; the array of `data_items` 127 and its object 128
    for leaf in [look_at(), subtree(), string_items()] {
        let tree = inverters_around(61, leaf);
        let text = tree.to_json().expect("within 128 levels");
        // the reader takes it, and reads the same leaf at the bottom
        let again = loaded(&text);
        let (mut written, mut read) = (&tree.root, &again.root);
        while let ([below_written], [below_read]) = (written.nodes(), read.nodes()) {
            written = below_written;
            read = below_read;
        }
        assert_eq!(read, written);
    }
    // one decorator more: the array is level 129, the object of `data_items` 130
    assert!(refused(&inverters_around(62, look_at())));
    assert!(refused(&inverters_around(62, subtree())));
    // `data_items` of strings is an array only: 129
    assert!(refused(&inverters_around(62, string_items())));
    // while the same leaf without an array member is written at 62: its body is level 128
    let bare = Node::LookAt {
        block_pos: Param::key("where"),
    };
    assert!(inverters_around(62, bare).to_json().is_ok());
}

/// A literal that failed beside a key member that is
/// no string leaves the state a read default literal leaves (the default literal, the key
/// `""`). It has one text: the literal member with the value vanilla holds. No member name
/// the author did not write appears, the key member that gave `""` is left out, and the text
/// reads back to the same node and rewrites to itself.
#[test]
fn a_failed_literal_beside_a_key_that_is_no_string_is_rewritten_as_the_literal_member() {
    for (node, rewritten_as) in [
        (
            r#"{"minecraft:wait_ticks":{"num_of_ticks":"soon","num_of_ticks_id":5}}"#,
            r#"{"minecraft:wait_ticks":{"num_of_ticks":0}}"#,
        ),
        // the key member alone gives the same state: the literal member is then one the
        // author did not write, and it is the only member that reads back to the key `""`
        (
            r#"{"minecraft:wait_ticks":{"num_of_ticks_id":5}}"#,
            r#"{"minecraft:wait_ticks":{"num_of_ticks":0}}"#,
        ),
        (
            r#"{"minecraft:wait_ticks":{"num_of_ticks_id":[]}}"#,
            r#"{"minecraft:wait_ticks":{"num_of_ticks":0}}"#,
        ),
        // a key member that is a string is the key, whatever the literal was
        (
            r#"{"minecraft:wait_ticks":{"num_of_ticks":"soon","num_of_ticks_id":"k"}}"#,
            r#"{"minecraft:wait_ticks":{"num_of_ticks_id":"k"}}"#,
        ),
        // and the empty string is a key member's text too
        (
            r#"{"minecraft:wait_ticks":{"num_of_ticks":"soon","num_of_ticks_id":""}}"#,
            r#"{"minecraft:wait_ticks":{"num_of_ticks":0}}"#,
        ),
    ] {
        assert_eq!(rewritten(node), rewritten_as, "{node}");
        assert_eq!(root_of(node).0, root_of(rewritten_as).0, "{node}");
        assert_eq!(rewritten(rewritten_as), rewritten_as, "{node}");
    }
}

#[test]
fn a_dropped_child_before_one_with_unknown_members_leaves_them_at_the_kept_index() {
    let document = loaded(&tree_text(
        "t",
        r#"{"minecraft:sequence": {"children": [{"minecraft:nope": {}}, {"minecraft:consume_item": {"extra": 2}}]}}"#,
    ));
    let kept = root_body("minecraft:sequence")
        .child("children")
        .index(0)
        .child("minecraft:consume_item");
    let rows: Vec<String> = document
        .unknown()
        .iter()
        .map(|(at, key, _)| format!("{at}/{key}"))
        .collect();
    assert_eq!(rows, [format!("{kept}/extra")]);
    let text = compact_document(&document);
    assert!(
        text.contains(r#""children":[{"minecraft:consume_item":{"extra":2}}]"#),
        "{text}"
    );
}
