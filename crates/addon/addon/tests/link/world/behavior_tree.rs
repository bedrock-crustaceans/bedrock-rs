//! 20 M5: the tree a `minecraft:subtree` node names. M6 (the block and entity names of other
//! nodes) is the game's at run time and is audited in `audit.rs`.

#![allow(unused_imports, reason = "which rows compile depends on the features")]

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link, link_with};
use super::{finding, said, unresolved_info};

const KIND: &str = "behavior tree";
const AT: &str = "minecraft:behavior_tree/minecraft:root/minecraft:subtree/subtree_id";

/// A tree `name` whose root is a subtree node naming `target`.
fn tree(name: &str, target: &str) -> String {
    format!(
        r#"{{"minecraft:behavior_tree": {{"minecraft:name": "{name}", "minecraft:root": {{"minecraft:subtree": {{"subtree_id": "{target}"}}}}}}}}"#
    )
}

/// A tree `name` without a subtree node.
fn leaf(name: &str) -> String {
    format!(
        r#"{{"minecraft:behavior_tree": {{"minecraft:name": "{name}", "minecraft:root": {{"minecraft:wait": {{}}}}}}}}"#
    )
}

#[test]
fn m5_a_subtree_a_pack_defines_or_the_game_has_resolves() {
    let (patrol, rest, built_in) = (
        tree("patrol", "rest"),
        leaf("rest"),
        // `move_to` is one of the trees the game has in its code.
        tree("dig", "move_to"),
    );
    let found = link(
        &[
            ("behavior_trees/patrol.json", patrol.as_str()),
            ("behavior_trees/rest.json", rest.as_str()),
            ("behavior_trees/dig.json", built_in.as_str()),
        ],
        &[],
    );
    assert_eq!(said(&found, &[KIND]), []);
}

#[test]
fn m5_a_subtree_nothing_defines_is_a_warning_at_the_node() {
    let patrol = tree("patrol", "nowhere");
    let files = [("behavior_trees/patrol.json", patrol.as_str())];
    let found = link(&files, &[]);
    assert_eq!(
        said(&found, &[KIND]),
        [unresolved_info(
            "behavior_trees/patrol.json",
            AT,
            KIND,
            "nowhere"
        )]
    );
    // An information of ours: the lints off, the pass tells none.
    let found = link_with(
        &files,
        &[],
        LoadOptions::default(),
        LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert_eq!(said(&found, &[KIND]).len(), 0);
}

/// A host that has a tree of the game the crate does not know answers for it; one it does not
/// have stays missing.
#[test]
fn m5_a_host_answers_for_a_tree_of_the_game() {
    let (patrol, other) = (tree("patrol", "hosted_tree"), tree("other", "nowhere"));
    let found = link_with(
        &[
            ("behavior_trees/patrol.json", patrol.as_str()),
            ("behavior_trees/other.json", other.as_str()),
        ],
        &[],
        hosted(&[("behavior tree", "hosted_tree")]),
        LinkOptions::default(),
    );
    assert_eq!(
        said(&found, &[KIND]),
        [unresolved_info(
            "behavior_trees/other.json",
            AT,
            KIND,
            "nowhere"
        )]
    );
}

/// Each node of a cycle is told once, at its own member, with the tree it names.
#[test]
fn m5_a_subtree_through_which_a_tree_reaches_itself_is_a_cycle_warning() {
    let (a, b, into) = (tree("a", "b"), tree("b", "a"), tree("into", "a"));
    let found = link(
        &[
            ("behavior_trees/a.json", a.as_str()),
            ("behavior_trees/b.json", b.as_str()),
            // A tree that only leads into the cycle is on none.
            ("behavior_trees/into.json", into.as_str()),
        ],
        &[],
    );
    assert_eq!(
        said(&found, &[KIND]),
        [
            finding(
                "core/reference_cycle",
                Severity::Info,
                "behavior_trees/a.json",
                AT,
                KIND,
                "b"
            ),
            finding(
                "core/reference_cycle",
                Severity::Info,
                "behavior_trees/b.json",
                AT,
                KIND,
                "a"
            ),
        ]
    );
}
