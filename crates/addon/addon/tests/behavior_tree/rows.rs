//! The rows of the fixtures, one test each: the fixture's stack is loaded
//! and compared with its `expected.json` and `expected.toml` (`check`), then the row's own
//! claims are asserted.

use bedrock_addon::diagnostic::Diagnostics;
use bedrock_addon::diagnostic::{DiagnosticCode, Phase, Severity, VanillaText};
use bedrock_addon::document::{Registry, Rejected};
use bedrock_addon::families::behavior_tree::{BehaviorTree, Node, Param};
use bedrock_addon::pack::{Behavior, PackStack};

use super::common::*;

/// The nine trees built into the game, in the order they are registered.
const BUILT_INS: [&str; 9] = [
    "activate_tool",
    "break_block",
    "find_block",
    "fly",
    "get_interaction_position_for_block",
    "look_at",
    "move",
    "move_to",
    "place_block",
];

#[test]
fn bt02_the_empty_stack_holds_the_nine_built_in_trees() {
    let stack = check("bt02_builtins_only");
    let trees = stack.behavior_trees();
    let names: Vec<&str> = trees.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, BUILT_INS);
    for (name, tree) in trees {
        // vanilla's texts: tab-indented, a leading line feed, no final one
        assert!(
            tree.raw.starts_with(
                "\n{\n\t\t\"minecraft:behavior_tree\" : \n\t{\n\t\t\"minecraft:name\": \""
            ),
            "{name}: {:?}",
            tree.raw
        );
        assert!(tree.raw.ends_with("\n\t}\n}"), "{name}: {:?}", tree.raw);
        assert!(
            tree.origin().pack.is_app_data(),
            "{name}: a built-in tree comes from no pack"
        );
    }
    // the four trees that wrap their node in a repeat
    for name in ["activate_tool", "break_block", "find_block", "place_block"] {
        let tree = trees.get(&id(name)).unwrap();
        let Node::RepeatUntilSuccess { child, .. } = &tree.root else {
            panic!("{name}: {:?}", tree.root);
        };
        assert!(child.is_some(), "{name}");
    }
    assert!(matches!(
        trees.get(&id("fly")).unwrap().root,
        Node::Fly { .. }
    ));
    assert_eq!(
        trees.get(&id("fly")).unwrap().raw,
        "\n{\n\t\t\"minecraft:behavior_tree\" : \n\t{\n\t\t\"minecraft:name\": \"fly\",\n\t\t\"minecraft:root\" : \n\t\t{\n\t\t\t\"minecraft:fly\": {}\n\t\t}\n\t}\n}"
    );
    assert_eq!(trees.rejected().len(), 0);
}

#[test]
fn bt06_the_file_name_plays_no_part() {
    let stack = check("bt06_filename_irrelevant");
    let trees = stack.behavior_trees();
    let bar = trees
        .get(&id("bar"))
        .expect("registered under its minecraft:name");
    assert_eq!(bar.origin().path.as_str(), "behavior_trees/foo.json");
    assert_eq!(bar.origin().pack.index(), Some(0));
    assert!(trees.get(&id("foo")).is_none());
    assert!(matches!(bar.root, Node::ConsumeItem { .. }));
    assert_eq!(
        bar.raw,
        fixture_text("bt06_filename_irrelevant/pack/behavior_trees/foo.json")
    );
    // the pack trees are loaded before the built-in ones
    assert_eq!(trees.iter().next().unwrap().0, &id("bar"));
    assert_eq!(trees.rejected().len(), 0);
}

#[test]
fn bt07_a_format_version_member_is_ignored() {
    let stack = check("bt07_format_version_ignored");
    let trees = stack.behavior_trees();
    let tree = trees.get(&id("versioned")).unwrap();
    assert!(matches!(tree.root, Node::PlaceBlock { .. }));
    silent(tree.diagnostics());
    assert!(trees.diagnostics().next().is_none());
    // not read, and kept where it was written
    assert_eq!(
        tree.root_extra
            .get("format_version")
            .and_then(|version| version.as_string())
            .map(|version| version.as_str()),
        Some("1.99.0")
    );
    assert!(tree.extra.is_empty());
    // the family has no version: nothing was written, whatever the member says
    assert_eq!(tree.version().written, None);
}

/// The only rejected file of `trees`.
fn only_rejected(trees: &Registry<BehaviorTree>) -> &Rejected {
    assert_eq!(trees.rejected().len(), 1, "{:#?}", trees.rejected());
    &trees.rejected()[0]
}

#[test]
fn bt08_a_syntax_error_skips_the_file_with_vanillas_text() {
    let stack = check("bt08_parse_error");
    let trees = stack.behavior_trees();
    assert!(trees.get(&id("cut")).is_none());
    let rejected = only_rejected(trees);
    assert_eq!(
        rejected.origin.path.as_str(),
        "behavior_trees/truncated.json"
    );
    let finding = one(&rejected.diagnostics, "MalformedInput", Severity::Error, "");
    let DiagnosticCode::MalformedInput { message, text } = &finding.code else {
        unreachable!()
    };
    // `parse error:\n%s` with the reader's message
    assert!(!message.is_empty());
    assert_eq!(
        text,
        &Some(VanillaText::Formatted {
            template: "parse error:\n{}",
            args: vec![message.clone()],
        })
    );
    assert_eq!(finding.code.to_string(), format!("parse error:\n{message}"));
    assert_eq!(finding.phase, Phase::Parse);
    assert_eq!(
        finding.location.pack.as_ref().and_then(|p| p.index()),
        Some(0)
    );
}

#[test]
fn bt09_an_empty_file_is_skipped_with_an_information() {
    let stack = check("bt09_empty_file");
    let trees = stack.behavior_trees();
    let rejected = only_rejected(trees);
    let finding = one(&rejected.diagnostics, "EmptyDocument", Severity::Info, "");
    assert!(matches!(
        finding.code,
        DiagnosticCode::EmptyDocument { text: None }
    ));
    // an information only (vanilla is silent): nothing here is an error, and the file is still
    // not a tree
    assert!(!rejected.diagnostics.has_errors());
}

#[test]
fn bt10_a_file_without_a_usable_name_is_skipped() {
    let stack = check("bt10_name_missing");
    let trees = stack.behavior_trees();
    let mut files: Vec<&str> = trees
        .rejected()
        .iter()
        .map(|rejected| rejected.origin.path.as_str())
        .collect();
    files.sort_unstable();
    assert_eq!(
        files,
        [
            "behavior_trees/empty_name.json",
            "behavior_trees/no_name.json",
            "behavior_trees/number_name.json"
        ]
    );
    for rejected in trees.rejected() {
        let finding = one(
            &rejected.diagnostics,
            "MissingRequiredField",
            Severity::Error,
            "minecraft:behavior_tree/minecraft:name",
        );
        // vanilla skips the file without a word
        assert!(matches!(
            finding.code,
            DiagnosticCode::MissingRequiredField { text: None }
        ));
        assert_eq!(rejected.diagnostics.len(), 1);
    }
    // an empty name is no key: nothing is registered under it
    assert!(trees.get(&id("")).is_none());
}

#[test]
fn bt27_a_payload_or_root_that_is_not_an_object_has_no_name() {
    let stack = check("bt27_non_object_envelope");
    let trees = stack.behavior_trees();
    assert_eq!(trees.rejected().len(), 2);
    for rejected in trees.rejected() {
        one(
            &rejected.diagnostics,
            "MissingRequiredField",
            Severity::Error,
            "minecraft:behavior_tree/minecraft:name",
        );
        assert_eq!(rejected.diagnostics.len(), 1);
    }
}

#[test]
fn bt11_a_root_with_two_members_is_not_registered() {
    let stack = check("bt11_root_two_members");
    let trees = stack.behavior_trees();
    assert!(trees.get(&id("two")).is_none());
    let rejected = only_rejected(trees);
    one(
        &rejected.diagnostics,
        "InvalidRoot",
        Severity::Error,
        "minecraft:behavior_tree/minecraft:root",
    );
    assert_eq!(rejected.diagnostics.len(), 1);
}

#[test]
fn bt12_an_unknown_root_type_is_not_registered_deviation() {
    // vanilla: registered with a null root and never sent; ours: not registered
    let stack = check("bt12_unknown_root");
    let trees = stack.behavior_trees();
    assert!(trees.get(&id("craft")).is_none());
    let rejected = only_rejected(trees);
    let finding = one(
        &rejected.diagnostics,
        "UnknownNode",
        Severity::Error,
        "minecraft:behavior_tree/minecraft:root/minecraft:craft_item",
    );
    assert_eq!(
        finding.code,
        DiagnosticCode::UnknownNode {
            name: "minecraft:craft_item".to_owned()
        }
    );
    // vanilla's text, without its trailing line feed
    assert_eq!(
        finding.code.to_string(),
        "Unknown behavior minecraft:craft_item"
    );
}

/// The root node of tree `name` of `stack`, and the tree's findings.
fn root<'s>(stack: &'s PackStack<Behavior>, name: &str) -> (&'s Node, &'s Diagnostics) {
    let tree = stack
        .behavior_trees()
        .get(&id(name))
        .unwrap_or_else(|| panic!("no tree `{name}`"));
    (&tree.root, tree.diagnostics())
}

fn param<T: Clone>(literal: T, key: &str) -> Param<T> {
    Param {
        literal,
        key: key.to_owned(),
    }
}

#[test]
fn bt16_wait_duration_is_a_literal_or_the_key() {
    let stack = check("bt16_literal_vs_key");
    let seconds = |name: &str| {
        let (Node::Wait { seconds, .. }, findings) = root(&stack, name) else {
            panic!("{name}")
        };
        // the string form is the documented key form: no finding
        silent(findings);
        seconds.clone()
    };
    assert_eq!(seconds("wait_literal"), param(3, ""));
    assert_eq!(seconds("wait_key"), param(0, "secs"));
    assert_eq!(seconds("wait_default"), param(0, "duration"));
}

#[test]
fn bt26_a_bool_is_integral_and_numeric_and_a_real_is_no_integer() {
    let stack = check("bt26_bool_and_real_literals");
    let (Node::WaitTicks { ticks, .. }, findings) = root(&stack, "bool_int") else {
        panic!()
    };
    assert_eq!(ticks, &param(1, ""));
    // Read as 1, and told (ours, an information).
    one(
        findings,
        "InvalidType",
        Severity::Info,
        &format!("{ROOT}/minecraft:wait_ticks/num_of_ticks"),
    );
    let (Node::WaitTicks { ticks, .. }, findings) = root(&stack, "real_int") else {
        panic!()
    };
    assert_eq!(ticks, &param(0, "target_num_of_ticks"));
    one(
        findings,
        "LiteralIgnored",
        Severity::Info,
        &format!("{ROOT}/minecraft:wait_ticks/num_of_ticks"),
    );
    let (
        Node::MoveTo {
            distance_epsilon, ..
        },
        findings,
    ) = root(&stack, "bool_float")
    else {
        panic!()
    };
    assert_eq!(distance_epsilon, &param(0.0, ""));
    one(
        findings,
        "InvalidType",
        Severity::Info,
        &format!("{ROOT}/minecraft:move_to/distance_epsilon"),
    );
}

#[test]
fn bt18_interacts_literal_default_is_one() {
    let stack = check("bt18_interact_default");
    let (Node::Interact { ticks, .. }, findings) = root(&stack, "press") else {
        panic!()
    };
    assert_eq!(ticks, &param(1, "target_num_of_ticks"));
    silent(findings);
}

#[test]
fn bt17_a_failed_search_radius_is_32() {
    let stack = check("bt17_search_radius_fallback");
    for name in ["far", "bare"] {
        let (
            Node::FindBlock {
                block_name,
                search_radius,
                ..
            },
            findings,
        ) = root(&stack, name)
        else {
            panic!("{name}")
        };
        assert_eq!(search_radius, &param(32, "target_search_radius"), "{name}");
        assert_eq!(block_name, &param(String::new(), "block_name"), "{name}");
        if name == "far" {
            one(
                findings,
                "LiteralIgnored",
                Severity::Info,
                &format!("{ROOT}/minecraft:find_block/search_radius"),
            );
            assert_eq!(findings.len(), 1);
        } else {
            // an absent literal is not an ignored one
            silent(findings);
        }
    }
}

#[test]
fn bt20_find_entitys_key_is_read_from_entity_type() {
    let stack = check("bt20_find_entity_key");
    let (
        Node::FindEntity {
            entity_name,
            search_radius,
            ..
        },
        findings,
    ) = root(&stack, "who")
    else {
        panic!()
    };
    assert_eq!(entity_name, &param(String::new(), "who"));
    assert_eq!(search_radius, &param(32, "target_search_radius"));
    silent(findings);
}

#[test]
fn bt19_position_arrays() {
    let stack = check("bt19_blockpos_arrays");
    let (Node::LookAt { block_pos, .. }, findings) = root(&stack, "bad_element") else {
        panic!()
    };
    // the bad element stays 0 and the value still counts as read
    assert_eq!(block_pos, &param([1, 0, 3], ""));
    // The game's level-3 line, and the tree loads: a kept error.
    let finding = one(
        findings,
        "InvalidType",
        Severity::Error,
        &format!("{ROOT}/minecraft:look_at/block_pos/1"),
    );
    assert!(finding.kept);
    assert_eq!(
        finding.code,
        DiagnosticCode::InvalidType {
            text: Some(VanillaText::Fixed("Expected Integral Number."))
        }
    );
    assert_eq!(findings.len(), 1);
    let (Node::LookAt { block_pos, .. }, findings) = root(&stack, "short") else {
        panic!()
    };
    assert_eq!(block_pos, &param([0, 0, 0], "target_block_pos"));
    one(
        findings,
        "LiteralIgnored",
        Severity::Info,
        &format!("{ROOT}/minecraft:look_at/block_pos"),
    );
    assert_eq!(findings.len(), 1);
    let (Node::MoveTo { pos, .. }, findings) = root(&stack, "vec3") else {
        panic!()
    };
    assert_eq!(pos, &param([0.5, 4.0, 0.5], ""));
    silent(findings);
}

#[test]
fn bt28_both_fly_literals_are_one_value() {
    let stack = check("bt28_fly_literal_alias");
    let fly = |name: &str| {
        let (
            Node::Fly {
                should_be_flying,
                throw_if_no_change,
                ..
            },
            findings,
        ) = root(&stack, name)
        else {
            panic!("{name}")
        };
        (
            should_be_flying.clone(),
            throw_if_no_change.clone(),
            findings,
        )
    };
    const THROW: &str = "should_throw_event_if_no_state_change_necessary";
    // the throw literal, read last, replaces the flying literal
    let (flying, throw, findings) = fly("both");
    assert_eq!((flying, throw), (param(false, ""), param(false, "")));
    one(
        findings,
        "LiteralIgnored",
        Severity::Info,
        &format!("{ROOT}/minecraft:fly/should_be_flying_bool"),
    );
    assert_eq!(findings.len(), 1);
    let (flying, throw, findings) = fly("throw_only");
    assert_eq!(
        (flying, throw),
        (param(true, "should_be_flying"), param(true, ""))
    );
    silent(findings);
    let (flying, throw, findings) = fly("flying_only");
    assert_eq!((flying, throw), (param(true, ""), param(true, THROW)));
    silent(findings);
}

#[test]
fn bt23_an_unknown_body_key_is_reported_and_kept() {
    let stack = check("bt23_unknown_body_key");
    let (Node::WaitTicks { ticks, extra }, findings) = root(&stack, "ticks") else {
        panic!()
    };
    assert_eq!(ticks, &param(2, ""));
    let finding = one(
        findings,
        "UnrecognizedField",
        Severity::Info,
        &format!("{ROOT}/minecraft:wait_ticks/ticks"),
    );
    assert!(matches!(
        finding.code,
        DiagnosticCode::UnrecognizedField { text: None }
    ));
    assert_eq!(findings.len(), 1);
    assert_eq!(
        extra
            .get("ticks")
            .and_then(|v| v.as_number())
            .and_then(|n| n.to_i64()),
        Some(3)
    );
    assert_eq!(extra.iter().count(), 1);
}

#[test]
fn bt01_the_22_vanilla_files_load_without_a_finding() {
    let stack = check("bt01_vanilla_22");
    let trees = stack.behavior_trees();
    // 22 pack trees and the 9 built-in ones
    assert_eq!(trees.len(), 31);
    assert_eq!(trees.rejected().len(), 0);
    assert!(trees.diagnostics().next().is_none());
    let (Node::Sequence { children, .. }, _) = root(&stack, "collect_items") else {
        panic!()
    };
    let Some(
        [
            _,
            Node::MoveTo {
                pos,
                distance_epsilon,
                ..
            },
            _,
        ],
    ) = children.as_deref()
    else {
        panic!("{children:?}")
    };
    assert_eq!(pos, &param([0.0, 0.0, 0.0], "target_pos"));
    assert_eq!(distance_epsilon, &param(1.5, ""));
    // the raw text is the file, byte for byte: vanilla's files have CRLF line ends
    let collect = trees.get(&id("collect_items")).unwrap();
    assert_eq!(
        collect.raw,
        fixture_text("bt01_vanilla_22/pack/behavior_trees/collect_items.json")
    );
    assert!(collect.raw.contains("\r\n"));
}

#[test]
fn bt13_an_unknown_child_is_dropped_and_the_rest_loads() {
    let stack = check("bt13_unknown_child");
    let (Node::Sequence { children, .. }, findings) = root(&stack, "dance") else {
        panic!()
    };
    // order kept, the unknown one left out
    let Some([Node::Wait { seconds, .. }, Node::WaitTicks { ticks, .. }]) = children.as_deref()
    else {
        panic!("{children:?}")
    };
    assert_eq!((seconds, ticks), (&param(1, ""), &param(3, "")));
    // vanilla's level-3 text, and the tree loads without the child: a kept error.
    let finding = one(
        findings,
        "UnknownNode",
        Severity::Error,
        &format!("{ROOT}/minecraft:sequence/children/1/minecraft:dance"),
    );
    assert!(finding.kept);
    assert_eq!(finding.code.to_string(), "Unknown behavior minecraft:dance");
    assert_eq!(findings.len(), 1);
    assert_eq!(stack.behavior_trees().rejected().len(), 0);
}

#[test]
fn bt14_children_that_are_no_nodes() {
    let stack = check("bt14_bad_children");
    let (Node::Sequence { children, .. }, findings) = root(&stack, "object_children") else {
        panic!()
    };
    assert_eq!(children, &None);
    let finding = one(
        findings,
        "InvalidType",
        Severity::Info,
        &format!("{ROOT}/minecraft:sequence/children"),
    );
    assert!(matches!(
        finding.code,
        DiagnosticCode::InvalidType { text: None }
    ));
    assert_eq!(findings.len(), 1);
    let (Node::Selector { children, .. }, findings) = root(&stack, "mixed_children") else {
        panic!()
    };
    let Some([Node::Attack { ticks, .. }]) = children.as_deref() else {
        panic!("{children:?}")
    };
    assert_eq!(ticks, &param(0, "target_num_of_ticks"));
    for index in [0, 1] {
        one(
            findings,
            "InvalidNode",
            Severity::Info,
            &format!("{ROOT}/minecraft:selector/children/{index}"),
        );
    }
    assert_eq!(findings.len(), 2);
}

#[test]
fn bt15_a_decorator_without_a_child() {
    let stack = check("bt15_decorator_no_child");
    let (Node::Inverter { child, .. }, findings) = root(&stack, "lonely") else {
        panic!()
    };
    assert_eq!(child, &None);
    one(
        findings,
        "InvalidNode",
        Severity::Info,
        &format!("{ROOT}/minecraft:inverter/child"),
    );
    assert_eq!(findings.len(), 1);
}

#[test]
fn bt21_a_subtree_and_its_data_items() {
    let stack = check("bt21_subtree");
    let (
        Node::Subtree {
            subtree_id,
            data_items,
            ..
        },
        findings,
    ) = root(&stack, "sub")
    else {
        panic!()
    };
    assert_eq!(subtree_id, &id("move"));
    let pair = |from: &str, to: &str| (from.to_owned(), to.to_owned());
    assert_eq!(data_items, &[pair("a", "a"), pair("b", "c")]);
    for index in [2, 3] {
        let finding = one(
            findings,
            "InvalidType",
            Severity::Info,
            &format!("{ROOT}/minecraft:subtree/data_items/{index}"),
        );
        assert!(matches!(
            finding.code,
            DiagnosticCode::InvalidType { text: None }
        ));
    }
    assert_eq!(findings.len(), 2);
    // `move` is a built-in tree: the reference resolves, and loading says nothing of it
    assert!(stack.behavior_trees().get(subtree_id).is_some());
    assert!(
        !stack
            .behavior_trees()
            .diagnostics()
            .any(|d| d.code.name() == "UnresolvedReference")
    );
}

#[test]
fn bt03_the_higher_pack_wins_and_the_lower_tree_is_shadowed() {
    let stack = check("bt03_stack_override");
    let trees = stack.behavior_trees();
    let tree = trees.get(&id("chop_wood")).unwrap();
    assert_eq!(tree.origin().pack.index(), Some(0));
    assert!(matches!(tree.root, Node::WaitTicks { .. }));
    let notices: Vec<_> = trees.diagnostics().collect();
    assert_eq!(notices.len(), 1);
    assert!(matches!(
        notices[0].code,
        DiagnosticCode::ShadowedDefinition { text: None }
    ));
    assert_eq!(notices[0].severity, Severity::Info);
    // at the replaced tree: p1's
    assert_eq!(
        notices[0].location.pack.as_ref().and_then(|p| p.index()),
        Some(1)
    );
}

#[test]
fn bt04_the_last_path_of_a_pack_wins_deviation() {
    // vanilla: silent; ours: paths sorted, with an information
    let stack = check("bt04_same_pack_duplicate");
    let trees = stack.behavior_trees();
    let tree = trees.get(&id("x")).unwrap();
    assert_eq!(tree.origin().path.as_str(), "behavior_trees/b.json");
    assert!(matches!(tree.root, Node::UseEntity { .. }));
    let notices: Vec<_> = trees.diagnostics().collect();
    assert_eq!(notices.len(), 1);
    assert!(matches!(
        notices[0].code,
        DiagnosticCode::DuplicateIdentifier { text: None }
    ));
    assert_eq!(notices[0].severity, Severity::Info);
    assert_eq!(
        notices[0].location.path.as_ref().map(|p| p.as_str()),
        Some("behavior_trees/a.json")
    );
}

#[test]
fn bt05_a_built_in_tree_replaces_a_pack_tree_of_its_name() {
    let stack = check("bt05_builtin_wins");
    let trees = stack.behavior_trees();
    let tree = trees.get(&id("move")).unwrap();
    assert!(tree.origin().pack.is_app_data());
    assert!(matches!(tree.root, Node::Move { .. }));
    let notices: Vec<_> = trees.diagnostics().collect();
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].code, DiagnosticCode::ShadowedByBuiltin);
    assert_eq!(notices[0].severity, Severity::Info);
    assert_eq!(
        notices[0].location.path.as_ref().map(|p| p.as_str()),
        Some("behavior_trees/move.json")
    );
    assert_eq!(
        notices[0].location.pack.as_ref().and_then(|p| p.index()),
        Some(0)
    );
    // the built-in trees keep their place: still nine trees, in their order
    let names: Vec<&str> = trees.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names.len(), 9);
}

#[test]
fn bt25_a_file_that_is_not_json_is_ignored_deviation() {
    // vanilla reads every file of the folder, so `notes.txt` would log "parse error"
    let stack = check("bt25_non_json_file");
    let trees = stack.behavior_trees();
    // sub-folders are scanned
    let tree = trees.get(&id("x")).unwrap();
    assert_eq!(tree.origin().path.as_str(), "behavior_trees/sub/x.json");
    assert_eq!(trees.rejected().len(), 0);
    let notices: Vec<_> = trees.diagnostics().collect();
    assert_eq!(notices.len(), 1);
    assert!(matches!(
        notices[0].code,
        DiagnosticCode::IgnoredFile { text: None }
    ));
    assert_eq!(notices[0].severity, Severity::Info);
    assert_eq!(
        notices[0].location.path.as_ref().map(|p| p.as_str()),
        Some("behavior_trees/notes.txt")
    );
}
