//! The validation rules and the schema beyond what the fixture rows show: both sides of
//! every condition, on single files.

use std::fmt::Write as _;
use std::time::{Duration, Instant};

use bedrock_addon_core::diagnostic::{Code, JsonPath, Severity, codes};
use bedrock_addon_core::document::Content;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_behavior_tree::codes as bt_codes;
use bedrock_addon_family_behavior_tree::{BehaviorTree, Node, Param};

use super::common::*;

fn param<T>(literal: T, key: &str) -> Param<T> {
    Param {
        literal,
        key: key.to_owned(),
    }
}

fn parse_bytes(
    bytes: &[u8],
) -> Result<
    bedrock_addon_core::document::Document<BehaviorTree>,
    bedrock_addon_core::document::Rejected,
> {
    let path = PackPath::from_static("behavior_trees/test.json");
    BehaviorTree::parse(&path, bytes, &LoadContext::detached())
}

// ---- the text ---------------------------------------------------------------------------------

#[test]
fn a_byte_order_mark_is_skipped_and_stays_in_the_raw_text() {
    let text = format!("\u{feff}{}", tree_text("bom", r#"{"minecraft:wait": {}}"#));
    let document = loaded(&text);
    assert_eq!(document.name, id("bom"));
    assert_eq!(document.raw, text);
    silent(document.diagnostics());
}

#[test]
fn text_that_is_not_utf8_is_malformed() {
    let mut bytes = tree_text("x", r#"{"minecraft:wait": {}}"#).into_bytes();
    bytes.push(0xff);
    let rejected = parse_bytes(&bytes).unwrap_err();
    let finding = one(&rejected.diagnostics, "core/syntax", Severity::Error, "");
    // not a text of vanilla's: its reader does not check the encoding
    assert!(finding.is(&codes::SYNTAX));
    assert_eq!(finding.vanilla(), None);
}

#[test]
fn comments_are_accepted() {
    let document = loaded(
        "// a line comment\n{ /* a block */ \"minecraft:behavior_tree\": {\"minecraft:name\": \"c\", \"minecraft:root\": {\"minecraft:wait\": {}}}}",
    );
    assert_eq!(document.name, id("c"));
    silent(document.diagnostics());
}

#[test]
fn only_a_file_of_no_bytes_is_an_empty_document() {
    // white space is text that is not JSON: vanilla logs its parse error
    let rejected = rejected("  \n");
    let finding = one(
        &rejected.diagnostics,
        "behavior_tree/parse_error",
        Severity::Error,
        "",
    );
    assert!(finding.is(&bt_codes::PARSE_ERROR));
    assert_eq!(rejected.diagnostics.len(), 1);
    // a null document parses: it has no name
    for text in ["null", "5", "\"text\""] {
        let rejected = super::common::rejected(text);
        one(
            &rejected.diagnostics,
            "core/missing_required_field",
            Severity::Error,
            "minecraft:behavior_tree/minecraft:name",
        );
        assert_eq!(rejected.diagnostics.len(), 1, "{text}");
    }
    let rejected = parse_bytes(b"").unwrap_err();
    one(
        &rejected.diagnostics,
        "core/empty_document",
        Severity::Info,
        "",
    );
    assert_eq!(rejected.diagnostics.len(), 1);
}

/// `depth` inverters around a leaf whose body is `leaf_body`.
fn nested(depth: usize, leaf_body: &str) -> String {
    let mut root = String::new();
    for _ in 0..depth {
        root.push_str(r#"{"minecraft:inverter": {"child": "#);
    }
    write!(root, r#"{{"minecraft:consume_item": {leaf_body}}}"#).unwrap();
    for _ in 0..depth {
        root.push_str("}}");
    }
    tree_text("deep", &root)
}

/// A tree is a recursive document: one nested past the JSON depth limit (128 arrays and
/// objects) is rejected before it is parsed, however deep it is.
#[test]
fn a_tree_nested_past_the_limit_is_rejected_quickly() {
    // the root, the payload, two levels per inverter, the leaf and its body: 2 + 2n + 2
    let at_the_limit = loaded(&nested(62, "{}"));
    let mut node = &at_the_limit.root;
    let mut inverters = 0;
    while let Node::Inverter { child, .. } = node {
        node = child.as_deref().expect("every inverter has its child");
        inverters += 1;
    }
    assert_eq!(inverters, 62);
    // one level more (the leaf's body an array: 2 + 2n + 1 for n = 63)
    let just_over = nested(63, "5").replace(": 5}", ": []}");
    let rejected = rejected(&just_over);
    let finding = one(&rejected.diagnostics, "core/syntax", Severity::Error, "");
    assert!(finding.is(&codes::SYNTAX));
    assert_eq!(finding.vanilla(), None);
    assert_eq!(rejected.diagnostics.len(), 1);
    // and far more, quickly: no recursion happens
    let started = Instant::now();
    let huge = nested(200_000, "{}");
    let rejected = super::common::rejected(&huge);
    one(&rejected.diagnostics, "core/syntax", Severity::Error, "");
    // only brackets were counted
    let unbalanced = "{".repeat(1_000_000);
    let rejected = super::common::rejected(&unbalanced);
    one(&rejected.diagnostics, "core/syntax", Severity::Error, "");
    // A guard against a hang, not a cost bound: 23 ms measured in the dev profile,
    // 200 times under the bound.
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
}

// ---- literals and keys (rule 10) ----------------------------------------------------------------

fn wait_ticks(body: &str) -> (Param<i32>, usize) {
    let (Node::WaitTicks { ticks, .. }, findings) =
        &root_of(&format!(r#"{{"minecraft:wait_ticks": {body}}}"#))
    else {
        panic!()
    };
    (ticks.clone(), findings.len())
}

const DEFAULT: &str = "target_num_of_ticks";

#[test]
fn an_integer_literal_fits_an_i32() {
    assert_eq!(
        wait_ticks(r#"{"num_of_ticks": 2147483647}"#),
        (param(i32::MAX, ""), 0)
    );
    assert_eq!(
        wait_ticks(r#"{"num_of_ticks": -2147483648}"#),
        (param(i32::MIN, ""), 0)
    );
    assert_eq!(
        wait_ticks(r#"{"num_of_ticks": 2147483648}"#),
        (param(0, DEFAULT), 1)
    );
    assert_eq!(
        wait_ticks(r#"{"num_of_ticks": -2147483649}"#),
        (param(0, DEFAULT), 1)
    );
    // A bool is integral, and told (ours).
    assert_eq!(wait_ticks(r#"{"num_of_ticks": false}"#), (param(0, ""), 1));
    assert_eq!(
        wait_ticks(r#"{"num_of_ticks": 1e2}"#),
        (param(0, DEFAULT), 1)
    );
    assert_eq!(
        wait_ticks(r#"{"num_of_ticks": "7"}"#),
        (param(0, DEFAULT), 1)
    );
    assert_eq!(
        wait_ticks(r#"{"num_of_ticks": [7]}"#),
        (param(0, DEFAULT), 1)
    );
}

#[test]
fn a_null_literal_is_an_absent_one() {
    assert_eq!(
        wait_ticks(r#"{"num_of_ticks": null}"#),
        (param(0, DEFAULT), 0)
    );
    assert_eq!(
        wait_ticks(r#"{"num_of_ticks": null, "num_of_ticks_id": "k"}"#),
        (param(0, "k"), 0)
    );
}

/// The key member's text as vanilla's string conversion gives it; a value that is no string is told
/// (ours: one `InvalidType` warning).
#[test]
fn a_key_member_that_is_no_string() {
    assert_eq!(
        wait_ticks(r#"{"num_of_ticks_id": null}"#),
        (param(0, DEFAULT), 0)
    );
    assert_eq!(
        wait_ticks(r#"{"num_of_ticks_id": true}"#),
        (param(0, "true"), 1)
    );
    assert_eq!(
        wait_ticks(r#"{"num_of_ticks_id": false}"#),
        (param(0, "false"), 1)
    );
    assert_eq!(wait_ticks(r#"{"num_of_ticks_id": 5}"#), (param(0, ""), 1));
    assert_eq!(
        wait_ticks(r#"{"num_of_ticks_id": ["k"]}"#),
        (param(0, ""), 1)
    );
    assert_eq!(wait_ticks(r#"{"num_of_ticks_id": {}}"#), (param(0, ""), 1));
    assert_eq!(wait_ticks(r#"{"num_of_ticks_id": ""}"#), (param(0, ""), 0));
}

#[test]
fn a_float_literal_takes_any_number() {
    let epsilon = |body: &str| {
        let (
            Node::MoveTo {
                distance_epsilon, ..
            },
            findings,
        ) = &root_of(&format!(r#"{{"minecraft:move_to": {body}}}"#))
        else {
            panic!()
        };
        (distance_epsilon.clone(), findings.len())
    };
    assert_eq!(epsilon(r#"{"distance_epsilon": 2}"#), (param(2.0, ""), 0));
    assert_eq!(
        epsilon(r#"{"distance_epsilon": -0.25}"#),
        (param(-0.25, ""), 0)
    );
    assert_eq!(
        epsilon(r#"{"distance_epsilon": true}"#),
        (param(1.0, ""), 1)
    );
    assert_eq!(
        epsilon(r#"{"distance_epsilon": "1.5"}"#),
        (param(0.0, "target_distance_epsilon"), 1)
    );
}

#[test]
fn a_bool_or_string_literal_takes_only_its_type() {
    let (Node::FindBlock { block_name, .. }, findings) =
        &root_of(r#"{"minecraft:find_block": {"block_name": 5, "block_name_id": 7}}"#)
    else {
        panic!()
    };
    // the literal failed, so the preset key is replaced by the key member's text: a number is ""
    assert_eq!(*block_name, param(String::new(), ""));
    one(
        findings,
        "core/literal_ignored",
        Severity::Info,
        &format!("{ROOT}/minecraft:find_block/block_name"),
    );
    let (Node::FindBlock { block_name, .. }, _) =
        &root_of(r#"{"minecraft:find_block": {"block_name": "", "block_name_id": "k"}}"#)
    else {
        panic!()
    };
    // an empty string is a string: read, and the key is the preset
    assert_eq!(*block_name, param(String::new(), "block_name"));
    let (
        Node::Fly {
            should_be_flying, ..
        },
        findings,
    ) = &root_of(r#"{"minecraft:fly": {"should_be_flying_bool": 1}}"#)
    else {
        panic!()
    };
    assert_eq!(*should_be_flying, param(false, "should_be_flying"));
    one(
        findings,
        "core/literal_ignored",
        Severity::Info,
        &format!("{ROOT}/minecraft:fly/should_be_flying_bool"),
    );
}

#[test]
fn wait_duration_of_another_type_is_an_ignored_literal() {
    let wait = |body: &str| {
        let (Node::Wait { seconds, .. }, findings) =
            &root_of(&format!(r#"{{"minecraft:wait": {body}}}"#))
        else {
            panic!()
        };
        (seconds.clone(), kinds(findings))
    };
    // A bool is integral, and told (ours).
    assert_eq!(
        wait(r#"{"wait_duration": true}"#),
        (param(1, ""), vec!["core/invalid_type"])
    );
    // the same member read as text: a number, an array or an object is ""
    for body in [
        r#"{"wait_duration": 1.5}"#,
        r#"{"wait_duration": [1]}"#,
        r#"{"wait_duration": {}}"#,
    ] {
        assert_eq!(
            wait(body),
            (param(0, ""), vec!["core/literal_ignored"]),
            "{body}"
        );
    }
    assert_eq!(
        wait(r#"{"wait_duration": null}"#),
        (param(0, "duration"), vec![])
    );
    assert_eq!(wait(r#"{"wait_duration": ""}"#), (param(0, ""), vec![]));
}

// ---- [x, y, z] (rules 11 and 12) ----------------------------------------------------------------

fn look_at(block_pos: &str) -> (Param<[i32; 3]>, Vec<String>) {
    let (Node::LookAt { block_pos, .. }, findings) = &root_of(&format!(
        r#"{{"minecraft:look_at": {{"block_pos": {block_pos}}}}}"#
    )) else {
        panic!()
    };
    let found = findings
        .iter()
        .map(|d| format!("{} {}", d.code.id(), d.location.json_path))
        .collect();
    (block_pos.clone(), found)
}

#[test]
fn a_block_position_is_an_array_of_exactly_three() {
    let at = format!("{ROOT}/minecraft:look_at/block_pos");
    let ignored = (
        param([0; 3], "target_block_pos"),
        vec![format!("core/literal_ignored {at}")],
    );
    assert_eq!(look_at("[1, 2, 3]"), (param([1, 2, 3], ""), vec![]));
    assert_eq!(look_at("[1, 2]"), ignored);
    assert_eq!(look_at("[1, 2, 3, 4]"), ignored);
    assert_eq!(look_at("[]"), ignored);
    assert_eq!(look_at(r#"{"x": 1, "y": 2, "z": 3}"#), ignored);
    assert_eq!(look_at("7"), ignored);
}

#[test]
fn an_element_of_a_block_position() {
    let at = |index: usize| format!("core/invalid_type {ROOT}/minecraft:look_at/block_pos/{index}");
    let not_integral = |index: usize| {
        format!("behavior_tree/expected_integral {ROOT}/minecraft:look_at/block_pos/{index}")
    };
    // a null element stays 0 and a bool is integral: each read so, and told (ours)
    assert_eq!(
        look_at("[null, true, -3]"),
        (param([0, 1, -3], ""), vec![at(0), at(1)])
    );
    // a real is not integral, in every position
    assert_eq!(
        look_at("[1.5, 2.0, \"3\"]"),
        (
            param([0, 0, 0], ""),
            vec![not_integral(0), not_integral(1), not_integral(2)]
        )
    );
    assert_eq!(
        look_at("[1, 2, [3]]"),
        (param([1, 2, 0], ""), vec![not_integral(2)])
    );
}

#[test]
fn an_element_of_a_position_of_numbers() {
    let move_to = |block_pos: &str| {
        let (Node::MoveTo { pos, .. }, findings) = &root_of(&format!(
            r#"{{"minecraft:move_to": {{"block_pos": {block_pos}}}}}"#
        )) else {
            panic!()
        };
        (pos.clone(), findings.clone())
    };
    let (pos, findings) = move_to("[1, null, true]");
    assert_eq!(pos, param([1.0, 0.0, 1.0], ""));
    // The `null` and the bool are read so, and told (ours, an information).
    for index in [1, 2] {
        one(
            &findings,
            "core/invalid_type",
            Severity::Info,
            &format!("{ROOT}/minecraft:move_to/block_pos/{index}"),
        );
    }
    assert_eq!(findings.len(), 2);
    let (pos, findings) = move_to(r#"[0.5, "y", 2]"#);
    assert_eq!(pos, param([0.5, 0.0, 2.0], ""));
    // The game's level-3 line, and the tree loads: a kept error.
    let finding = one(
        &findings,
        "behavior_tree/expected_number",
        Severity::Error,
        &format!("{ROOT}/minecraft:move_to/block_pos/1"),
    );
    assert!(finding.kept);
    assert_eq!(finding.code, Code::from(&bt_codes::EXPECTED_NUMBER));
    assert_eq!(finding.code.to_string(), "Expected Number.");
    assert_eq!(findings.len(), 1);
    let (pos, findings) = move_to("[1, 2]");
    assert_eq!(pos, param([0.0; 3], "target_block_pos"));
    assert_eq!(kinds(&findings), ["core/literal_ignored"]);
    let (pos, findings) = move_to("null");
    assert_eq!(pos, param([0.0; 3], "target_block_pos"));
    silent(&findings);
}

// ---- minecraft:fly (rule 10a) -------------------------------------------------------------------

#[test]
fn the_fly_alias_is_reported_once_and_only_when_both_literals_are_read() {
    const FLYING: &str = "should_be_flying_bool";
    const THROW: &str = "should_throw_event_if_no_state_change_necessary_bool";
    const THROW_KEY: &str = "should_throw_event_if_no_state_change_necessary";

    let fly = |body: &str| {
        let (
            Node::Fly {
                should_be_flying,
                throw_if_no_change,
                ..
            },
            findings,
        ) = &root_of(&format!(r#"{{"minecraft:fly": {body}}}"#))
        else {
            panic!()
        };
        let found: Vec<String> = findings
            .iter()
            .map(|d| d.location.json_path.to_string())
            .collect();
        assert!(
            findings
                .iter()
                .all(|d| d.code.id() == "core/literal_ignored")
        );
        (should_be_flying.clone(), throw_if_no_change.clone(), found)
    };
    let at = |key: &str| format!("{ROOT}/minecraft:fly/{key}");
    // both read, the same value: the flying literal is still the overwritten one
    assert_eq!(
        fly(&format!(r#"{{"{FLYING}": true, "{THROW}": true}}"#)),
        (param(true, ""), param(true, ""), vec![at(FLYING)])
    );
    assert_eq!(
        fly(&format!(r#"{{"{FLYING}": false, "{THROW}": true}}"#)),
        (param(true, ""), param(true, ""), vec![at(FLYING)])
    );
    // a flying literal of the wrong type is rule 10's finding, once
    assert_eq!(
        fly(&format!(
            r#"{{"{FLYING}": "yes", "{THROW}": true, "should_be_flying_id": "k"}}"#
        )),
        (param(true, "k"), param(true, ""), vec![at(FLYING)])
    );
    // a throw literal of the wrong type overwrites nothing
    assert_eq!(
        fly(&format!(r#"{{"{FLYING}": true, "{THROW}": 0}}"#)),
        (param(true, ""), param(true, THROW_KEY), vec![at(THROW)])
    );
    assert_eq!(
        fly(
            r#"{"should_throw_event_if_no_state_change_necessary_id": "t", "should_be_flying_id": "f"}"#
        ),
        (param(false, "f"), param(false, "t"), vec![])
    );
}

// ---- children and child (rules 6 to 9) ----------------------------------------------------------

#[test]
fn children_absent_null_or_no_array_is_an_information_and_an_empty_array_is_not() {
    for kind in ["minecraft:sequence", "minecraft:selector"] {
        for body in ["{}", r#"{"children": null}"#, r#"{"children": "x"}"#, "7"] {
            let (node, findings) = &root_of(&format!(r#"{{"{kind}": {body}}}"#));
            let (Node::Sequence { children, .. } | Node::Selector { children, .. }) = node else {
                panic!()
            };
            assert_eq!(*children, None, "{kind} {body}");
            one(
                findings,
                "core/invalid_type",
                Severity::Info,
                &format!("{ROOT}/{kind}/children"),
            );
            // A body that is no object is told as well (ours).
            assert_eq!(findings.len(), 1 + usize::from(body == "7"));
        }
        let (node, findings) = &root_of(&format!(r#"{{"{kind}": {{"children": []}}}}"#));
        let (Node::Sequence { children, .. } | Node::Selector { children, .. }) = node else {
            panic!()
        };
        assert_eq!(*children, Some(Vec::new()));
        silent(findings);
    }
}

#[test]
fn an_element_of_children_is_a_one_member_object() {
    let (Node::Sequence { children, .. }, findings) = &root_of(
        r#"{"minecraft:sequence": {"children": [{}, null, "minecraft:wait", [], {"minecraft:wait": {}}]}}"#,
    ) else {
        panic!()
    };
    assert_eq!(children.as_ref().unwrap().len(), 1);
    assert_eq!(kinds(findings), ["core/invalid_node"; 4]);
    let paths: Vec<String> = findings
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    let at = |index: usize| format!("{ROOT}/minecraft:sequence/children/{index}");
    assert_eq!(paths, [at(0), at(1), at(2), at(3)]);
    assert!(findings.iter().all(|d| d.severity == Severity::Info));
}

#[test]
fn a_child_is_a_one_member_object_of_a_known_type() {
    for kind in [
        "minecraft:inverter",
        "minecraft:repeat_until_failure",
        "minecraft:repeat_until_success",
    ] {
        let child_of = |body: &str| {
            let (node, findings) = root_of(&format!(r#"{{"{kind}": {body}}}"#));
            let (Node::Inverter { child, .. }
            | Node::RepeatUntilFailure { child, .. }
            | Node::RepeatUntilSuccess { child, .. }) = &node
            else {
                panic!()
            };
            let found: Vec<String> = findings
                .iter()
                .map(|d| format!("{} {} {}", d.code.id(), d.severity, d.location.json_path))
                .collect();
            (child.clone(), found)
        };
        let at = format!("{ROOT}/{kind}/child");
        let invalid = vec![format!("core/invalid_node info {at}")];
        for body in [
            "{}",
            r#"{"child": null}"#,
            r#"{"child": []}"#,
            r#"{"child": {}}"#,
            r#"{"child": {"minecraft:wait": {}, "minecraft:attack": {}}}"#,
        ] {
            assert_eq!(child_of(body), (None, invalid.clone()), "{kind} {body}");
        }
        // an unknown child is vanilla's own message, once, and the tree still loads: a kept
        // error
        assert_eq!(
            child_of(r#"{"child": {"minecraft:dance": {}}}"#),
            (
                None,
                vec![format!("core/unknown_node error {at}/minecraft:dance")]
            )
        );
        let (child, found) = child_of(r#"{"child": {"minecraft:consume_item": {}}}"#);
        assert!(matches!(child.as_deref(), Some(Node::ConsumeItem { .. })));
        assert!(found.is_empty());
    }
}

#[test]
fn only_the_root_of_a_tree_makes_an_unknown_type_an_error() {
    let rejected = rejected(&tree_text("r", r#"{"minecraft:dance": {}}"#));
    one(
        &rejected.diagnostics,
        "core/unknown_node",
        Severity::Error,
        &format!("{ROOT}/minecraft:dance"),
    );
    // the node types of older headers are unknown today
    for kind in ["minecraft:set_variable", "minecraft:have_item", "sequence"] {
        let rejected = super::common::rejected(&tree_text("r", &format!(r#"{{"{kind}": {{}}}}"#)));
        assert_eq!(
            kinds(&rejected.diagnostics),
            ["core/unknown_node"],
            "{kind}"
        );
    }
    let (_, findings) = root_of(
        r#"{"minecraft:inverter": {"child": {"minecraft:sequence": {"children": [{"minecraft:dance": 1}]}}}}"#,
    );
    let finding = one(
        &findings,
        "core/unknown_node",
        Severity::Error,
        &format!("{ROOT}/minecraft:inverter/child/minecraft:sequence/children/0/minecraft:dance"),
    );
    assert!(finding.kept);
}

#[test]
fn the_root_is_an_object_with_exactly_one_member() {
    for root in ["null", "[]", "{}", r#""minecraft:wait""#, "5"] {
        let rejected = rejected(&tree_text("r", root));
        one(
            &rejected.diagnostics,
            "core/invalid_root",
            Severity::Error,
            "minecraft:behavior_tree/minecraft:root",
        );
        assert_eq!(rejected.diagnostics.len(), 1, "{root}");
    }
    let rejected = rejected(r#"{"minecraft:behavior_tree": {"minecraft:name": "r"}}"#);
    assert_eq!(kinds(&rejected.diagnostics), ["core/invalid_root"]);
}

// ---- subtree (rule 14) and unknown keys (rule 13) ---------------------------------------------

fn subtree(body: &str) -> (String, Vec<(String, String)>, Vec<String>) {
    let (
        Node::Subtree {
            subtree_id,
            data_items,
            ..
        },
        findings,
    ) = &root_of(&format!(r#"{{"minecraft:subtree": {body}}}"#))
    else {
        panic!()
    };
    let found = findings
        .iter()
        .map(|d| {
            format!(
                "{} {} {}",
                d.code.id(),
                d.severity,
                d.location
                    .json_path
                    .to_string()
                    .trim_start_matches(&format!("{ROOT}/minecraft:subtree/"))
            )
        })
        .collect();
    (subtree_id.as_str().to_owned(), data_items.clone(), found)
}

fn pair(from: &str, to: &str) -> (String, String) {
    (from.to_owned(), to.to_owned())
}

#[test]
fn data_items_is_one_item_or_an_array_of_items() {
    let none: Vec<String> = Vec::new();
    assert_eq!(subtree("{}"), (String::new(), vec![], none.clone()));
    assert_eq!(subtree(r#"{"data_items": null}"#).2, none);
    assert_eq!(subtree(r#"{"data_items": []}"#).2, none);
    assert_eq!(subtree(r#"{"data_items": "a"}"#).1, [pair("a", "a")]);
    assert_eq!(
        subtree(r#"{"data_items": {"from": "a", "to": "b"}}"#),
        (String::new(), vec![pair("a", "b")], none.clone())
    );
    // one item that is no item: reported at the member
    for item in ["7", "true", r#"{"from": "a"}"#, r#"{"to": "b"}"#, "{}"] {
        assert_eq!(
            subtree(&format!(r#"{{"data_items": {item}}}"#)),
            (
                String::new(),
                vec![],
                vec!["core/invalid_type info data_items".to_owned()]
            ),
            "{item}"
        );
    }
    // an array inside the array is no item
    assert_eq!(
        subtree(r#"{"data_items": [["a"], "b"]}"#),
        (
            String::new(),
            vec![pair("b", "b")],
            vec!["core/invalid_type info data_items/0".to_owned()]
        )
    );
    // `from` and `to` are read as text: a bool is its word, a number is empty, null is empty
    assert_eq!(
        subtree(r#"{"data_items": [{"from": true, "to": 5}, {"from": null, "to": "t"}]}"#).1,
        [pair("true", ""), pair("", "t")]
    );
}

#[test]
fn subtree_id_is_read_as_text() {
    assert_eq!(subtree(r#"{"subtree_id": "other"}"#).0, "other");
    assert_eq!(subtree(r#"{"subtree_id": null}"#).0, "");
    assert_eq!(subtree(r#"{"subtree_id": true}"#).0, "true");
    assert_eq!(subtree(r#"{"subtree_id": 5}"#).0, "");
    // there is no `subtree_id_id`: it is an unknown key
    assert_eq!(
        subtree(r#"{"subtree_id_id": "k"}"#).2,
        ["core/unrecognized_field info subtree_id_id"]
    );
}

#[test]
fn every_unknown_key_of_a_body_is_reported_and_kept_in_order() {
    let document = loaded(&tree_text(
        "t",
        r#"{"minecraft:sequence": {"zeta": 1, "children": [{"minecraft:use_entity": {"who": "me"}}], "alpha": [2]}}"#,
    ));
    let (node, findings) = (&document.root, document.diagnostics());
    let Node::Sequence { children } = node else {
        panic!()
    };
    let body = JsonPath::of_keys([
        "minecraft:behavior_tree",
        "minecraft:root",
        "minecraft:sequence",
    ]);
    assert_eq!(
        document
            .unknown()
            .members_at(&body)
            .map(|(key, _)| key)
            .collect::<Vec<_>>(),
        ["zeta", "alpha"]
    );
    let Some([Node::UseEntity {}]) = children.as_deref() else {
        panic!()
    };
    let mut inner = body.child("children");
    inner.push_index(0);
    inner.push("minecraft:use_entity");
    assert_eq!(
        document
            .unknown()
            .members_at(&inner)
            .map(|(key, _)| key)
            .collect::<Vec<_>>(),
        ["who"]
    );
    let mut paths: Vec<String> = findings
        .iter()
        .map(|d| {
            assert_eq!(
                (d.code.id(), d.severity),
                ("core/unrecognized_field", Severity::Info)
            );
            d.location.json_path.to_string()
        })
        .collect();
    paths.sort();
    assert_eq!(
        paths,
        [
            format!("{ROOT}/minecraft:sequence/alpha"),
            format!("{ROOT}/minecraft:sequence/children/0/minecraft:use_entity/who"),
            format!("{ROOT}/minecraft:sequence/zeta"),
        ]
    );
    // the payload's and the root's other members are kept without a word
    let document = loaded(
        r#"{"extra": 1, "minecraft:behavior_tree": {"minecraft:name": "t", "note": "n", "minecraft:root": {"minecraft:wait": {}}}}"#,
    );
    silent(document.diagnostics());
    let (root, payload) = (
        JsonPath::new(),
        JsonPath::of_keys(["minecraft:behavior_tree"]),
    );
    assert!(document.unknown().get(&root, "extra").is_some());
    assert!(document.unknown().get(&payload, "note").is_some());
    assert!(document.unknown().get(&payload, "minecraft:name").is_none());
    assert!(
        document
            .unknown()
            .get(&root, "minecraft:behavior_tree")
            .is_none()
    );
}

/// A `null` fly literal is an absent one, as for every other literal: no finding, the key
/// member is read, and it overwrites nothing.
#[test]
fn a_null_fly_literal_is_an_absent_one() {
    let (
        Node::Fly {
            should_be_flying,
            throw_if_no_change,
            ..
        },
        findings,
    ) = &root_of(
        r#"{"minecraft:fly": {"should_be_flying_bool": null, "should_be_flying_id": "f", "should_throw_event_if_no_state_change_necessary_bool": null}}"#,
    )
    else {
        panic!()
    };
    silent(findings);
    assert_eq!(*should_be_flying, param(false, "f"));
    assert_eq!(
        *throw_if_no_change,
        param(false, "should_throw_event_if_no_state_change_necessary")
    );
    // beside a read flying literal, a null throw literal overwrites nothing
    let (
        Node::Fly {
            should_be_flying, ..
        },
        findings,
    ) = &root_of(
        r#"{"minecraft:fly": {"should_be_flying_bool": true, "should_throw_event_if_no_state_change_necessary_bool": null}}"#,
    )
    else {
        panic!()
    };
    silent(findings);
    assert_eq!(*should_be_flying, param(true, ""));
}
