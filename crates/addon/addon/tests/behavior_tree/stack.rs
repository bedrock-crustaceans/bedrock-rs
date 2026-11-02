//! `bp.behavior_trees()` beyond the fixture rows: which file's tree a name gets, the walk, and the accessor.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::behavior_tree::Node;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, PackStack};

use super::common::*;

const WAIT: &str = r#"{"minecraft:wait": {}}"#;
const ATTACK: &str = r#"{"minecraft:attack": {}}"#;
/// A root with two members: the file names its tree and does not load.
const TWO: &str = r#"{"minecraft:wait": {}, "minecraft:attack": {}}"#;

/// The codes and severities of the registry's findings, sorted.
fn notices(stack: &PackStack<Behavior>) -> Vec<(&'static str, Severity)> {
    let mut found: Vec<_> = stack
        .behavior_trees()
        .diagnostics()
        .map(|d| (d.code.name(), d.severity))
        .collect();
    found.sort_by_key(|(name, _)| *name);
    found
}

/// Vanilla collects the names first, the last file walked overwriting an earlier one, and
/// loads each name's file afterwards: a higher pack's file that names the tree and then fails
/// leaves the name without a tree.
#[test]
fn a_failed_higher_file_still_takes_its_name() {
    let stack = stack_of(vec![
        tree_pack(1, &[("x.json", &tree_text("x", TWO))]),
        tree_pack(2, &[("x.json", &tree_text("x", WAIT))]),
    ]);
    let trees = stack.behavior_trees();
    assert!(trees.get(&id("x")).is_none());
    assert_eq!(trees.len(), 9);
    assert_eq!(trees.rejected().len(), 1);
    assert_eq!(trees.rejected()[0].origin.pack.index(), Some(0));
    // the lower tree is told that it is replaced
    assert_eq!(
        notices(&stack),
        [
            ("InvalidRoot", Severity::Error),
            ("ShadowedDefinition", Severity::Info)
        ]
    );
}

#[test]
fn a_failed_last_file_of_a_pack_still_takes_its_name() {
    let stack = stack_of(vec![tree_pack(
        1,
        &[
            ("a.json", &tree_text("x", WAIT)),
            ("b.json", &tree_text("x", TWO)),
        ],
    )]);
    let trees = stack.behavior_trees();
    assert!(trees.get(&id("x")).is_none());
    assert_eq!(
        notices(&stack),
        [
            // ours, an information
            ("DuplicateIdentifier", Severity::Info),
            ("InvalidRoot", Severity::Error)
        ]
    );
}

/// A file without a usable name, or one that is not JSON, names no tree: the lower pack's
/// tree stays.
#[test]
fn a_higher_file_without_a_name_takes_nothing() {
    let stack = stack_of(vec![
        tree_pack(
            1,
            &[(
                "x.json",
                r#"{"minecraft:behavior_tree": {"minecraft:name": "x""#,
            )],
        ),
        tree_pack(2, &[("x.json", &tree_text("x", WAIT))]),
    ]);
    let trees = stack.behavior_trees();
    assert_eq!(trees.get(&id("x")).unwrap().origin().pack.index(), Some(1));
    assert_eq!(notices(&stack), [("MalformedInput", Severity::Error)]);
}

#[test]
fn a_lower_pack_never_replaces_a_higher_one() {
    // three packs, the middle one without the tree: the top one wins over the bottom one
    let stack = stack_of(vec![
        tree_pack(1, &[("x.json", &tree_text("x", ATTACK))]),
        tree_pack(2, &[("y.json", &tree_text("y", WAIT))]),
        tree_pack(3, &[("x.json", &tree_text("x", WAIT))]),
    ]);
    let trees = stack.behavior_trees();
    let x = trees.get(&id("x")).unwrap();
    assert_eq!(x.origin().pack.index(), Some(0));
    assert!(matches!(x.root, Node::Attack { .. }));
    assert_eq!(notices(&stack), [("ShadowedDefinition", Severity::Info)]);
    let finding = trees.diagnostics().next().unwrap();
    assert_eq!(
        finding.location.pack.as_ref().and_then(|p| p.index()),
        Some(2)
    );
}

#[test]
fn a_built_in_tree_wins_over_a_file_that_failed_under_its_name() {
    let stack = stack_of(vec![tree_pack(1, &[("fly.json", &tree_text("fly", TWO))])]);
    let trees = stack.behavior_trees();
    assert!(trees.get(&id("fly")).unwrap().origin().pack.is_app_data());
    // the failed file is rejected already: nothing of a pack was replaced
    assert_eq!(notices(&stack), [("InvalidRoot", Severity::Error)]);
}

#[test]
fn a_pack_tree_that_is_no_built_in_is_not_shadowed_by_one() {
    let stack = stack_of(vec![tree_pack(1, &[("m.json", &tree_text("mover", WAIT))])]);
    assert!(notices(&stack).is_empty());
    assert_eq!(stack.behavior_trees().len(), 10);
}

/// `keep_shadowed` lists what a name's effective tree replaced, top first.
#[test]
fn the_replaced_trees_are_the_layers_of_their_name() {
    let options = LoadOptions {
        keep_shadowed: true,
        ..LoadOptions::default()
    };
    let packs = vec![
        tree_pack(1, &[("move.json", &tree_text("move", ATTACK))]),
        tree_pack(2, &[("move.json", &tree_text("move", WAIT))]),
    ];
    let stack = PackStack::<Behavior>::of(packs, options).unwrap();
    let layers: Vec<_> = stack
        .behavior_trees()
        .layers(&id("move"))
        .map(|tree| tree.origin().pack.index())
        .collect();
    assert_eq!(layers, [None, Some(0), Some(1)]);
    assert_eq!(
        notices(&stack),
        [
            ("ShadowedByBuiltin", Severity::Info),
            ("ShadowedDefinition", Severity::Info)
        ]
    );
}

#[test]
fn only_the_extension_json_in_lower_case_is_a_tree_file() {
    let stack = stack_of(vec![tree_pack(
        1,
        &[
            ("upper.JSON", &tree_text("upper", WAIT)),
            ("no_extension", &tree_text("bare", WAIT)),
            ("x.json.bak", &tree_text("bak", WAIT)),
            ("ok.json", &tree_text("ok", WAIT)),
        ],
    )]);
    let trees = stack.behavior_trees();
    assert_eq!(trees.len(), 10);
    assert!(trees.get(&id("ok")).is_some());
    assert_eq!(notices(&stack), [("IgnoredFile", Severity::Info); 3]);
}

#[test]
fn files_outside_behavior_trees_are_not_read() {
    let stack = stack_of(vec![pack(
        1,
        vec![
            (
                "behavior_trees_old/x.json".to_owned(),
                tree_text("x", WAIT).into_bytes(),
            ),
            ("x.json".to_owned(), tree_text("x", WAIT).into_bytes()),
        ],
    )]);
    assert_eq!(stack.behavior_trees().len(), 9);
    assert!(notices(&stack).is_empty());
}

#[test]
fn the_accessor_caches_the_collection() {
    let stack = fixture_stack("bt06_filename_irrelevant");
    assert!(std::ptr::eq(stack.behavior_trees(), stack.behavior_trees()));
}

/// A file that failed is replaced without a notice: it is rejected already, and nothing of
/// it was a tree.
#[test]
fn a_failed_file_that_is_replaced_gets_no_notice() {
    let same_pack = stack_of(vec![tree_pack(
        1,
        &[
            ("a.json", &tree_text("x", TWO)),
            ("b.json", &tree_text("x", WAIT)),
        ],
    )]);
    assert_eq!(
        same_pack
            .behavior_trees()
            .get(&id("x"))
            .unwrap()
            .origin()
            .path
            .as_str(),
        "behavior_trees/b.json"
    );
    assert_eq!(notices(&same_pack), [("InvalidRoot", Severity::Error)]);
    let two_packs = stack_of(vec![
        tree_pack(1, &[("x.json", &tree_text("x", WAIT))]),
        tree_pack(2, &[("x.json", &tree_text("x", TWO))]),
    ]);
    assert_eq!(
        two_packs
            .behavior_trees()
            .get(&id("x"))
            .unwrap()
            .origin()
            .pack
            .index(),
        Some(0)
    );
    assert_eq!(notices(&two_packs), [("InvalidRoot", Severity::Error)]);
}

// A file below `behavior_trees/` that is no `.json` file is reported from the pack's
// file list; a stack load never asks for its bytes.
#[test]
fn a_stack_load_asks_for_the_json_files_only() {
    use bedrock_addon::load::Collection;

    use crate::responder::Order;

    let tree = tree_text("x", WAIT);
    let top: &[(&str, &[u8])] = &[
        ("behavior_trees/x.json", tree.as_bytes()),
        ("behavior_trees/x.JSON", tree.as_bytes()),
        ("behavior_trees/deep/note.txt", b"a note"),
        ("behavior_trees_old/y.txt", b"elsewhere"),
    ];
    let base: &[(&str, &[u8])] = &[("behavior_trees/picture.png", b"\x89PNG")];
    for order in Order::ALL {
        let (set, asked) =
            crate::requests::stack_load("data", &[top, base], &[Collection::BehaviorTrees], order);
        assert_eq!(asked, ["p0/behavior_trees/x.json"], "{order:?}");
        let trees = set.behavior.behavior_trees();
        assert!(trees.get(&id("x")).is_some(), "{order:?}");
        let ignored: Vec<(Option<usize>, &str)> = trees
            .diagnostics()
            .filter(|f| f.code.name() == "IgnoredFile")
            .map(|f| {
                assert_eq!(f.severity, Severity::Info);
                let pack = f.location.pack.as_ref().and_then(|pack| pack.index());
                (pack, f.location.path.as_ref().unwrap().as_str())
            })
            .collect();
        assert_eq!(
            ignored,
            [
                (Some(1), "behavior_trees/picture.png"),
                (Some(0), "behavior_trees/deep/note.txt"),
                (Some(0), "behavior_trees/x.JSON"),
            ],
            "{order:?}"
        );
    }
}
