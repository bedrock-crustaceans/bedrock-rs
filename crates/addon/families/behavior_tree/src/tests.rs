//! Unit tests of the module's private parts: the link checker of the
//! built-in texts and the arity of the one formatted text.

use bedrock_addon_core::load::{LoadContext, LoadOptions, StackCx, WalkFile};
use bedrock_addon_core::pack_identity::{PackIdVersion, PackPath, PackRef, PackSide, PackType};
use bytes::Bytes;
use uuid::Uuid;

use super::{BehaviorTree, LOADER, Node, builtin, link, read};
use bedrock_addon_core::diagnostic::{DiagnosticDetail, Phase, Severity};
use bedrock_addon_core::document::Registry;
use bedrock_addon_core::version::SemVersion;

/// The registry of one pack holding `trees`: `(name, root node as JSON text)`, each in the file
/// `behavior_trees/<name>.json`.
fn registry(trees: &[(&str, &str)]) -> Registry<BehaviorTree> {
    let id = PackIdVersion::new(Uuid::nil(), SemVersion::new(1, 0, 0), PackType::Behavior);
    let pack = PackRef::new(0, id, "p");
    let loader = LOADER;
    let cx = LoadContext::detached();
    let parsed = trees
        .iter()
        .map(|(name, root)| {
            let path = PackPath::new(&format!("behavior_trees/{name}.json")).unwrap();
            let text = format!(
                r#"{{"minecraft:behavior_tree": {{"minecraft:name": "{name}", "minecraft:root": {root}}}}}"#
            );
            let file = WalkFile {
                pack: pack.clone(),
                side: PackSide::Behavior,
                path: path.clone(),
                key: path,
                bytes: Bytes::from(text),
            };
            (loader.parse)(&file, &cx)
        })
        .collect();
    (loader.combine)(parsed, &StackCx::new(&LoadOptions::default(), &[]))
}

fn subtree(target: &str) -> String {
    format!(r#"{{"minecraft:subtree": {{"subtree_id": "{target}"}}}}"#)
}

/// `(code, file, JSON path)` of every link finding, sorted.
fn linked(trees: &[(&str, &str)]) -> Vec<(&'static str, String, String)> {
    let registry = registry(trees);
    let mut found: Vec<_> = link::link(&registry)
        .iter()
        .map(|finding| {
            assert_eq!(finding.severity, Severity::Info, "{finding:?}");
            assert_eq!(finding.phase, Phase::Link, "{finding:?}");
            assert_eq!(
                finding.location.pack.as_ref().and_then(PackRef::index),
                Some(0)
            );
            (
                finding.code.name(),
                finding.location.path.as_ref().unwrap().as_str().to_owned(),
                finding.location.json_path.to_string(),
            )
        })
        .collect();
    found.sort();
    found
}

const ROOT_SUBTREE: &str = "minecraft:behavior_tree/minecraft:root/minecraft:subtree/subtree_id";

fn at(code: &'static str, name: &str) -> (&'static str, String, String) {
    (
        code,
        format!("behavior_trees/{name}.json"),
        ROOT_SUBTREE.to_owned(),
    )
}

/// Fixture bt22, the first half.
#[test]
fn bt22_a_subtree_id_that_names_no_tree_is_unresolved() {
    let found = linked(&[("orphan", &subtree("nope"))]);
    assert_eq!(found, [at("UnresolvedReference", "orphan")]);
    // the finding names what it could not find
    let registry = registry(&[("orphan", &subtree("nope"))]);
    let findings = link::link(&registry);
    let finding = findings.iter().next().unwrap();
    assert_eq!(
        finding.detail.as_deref(),
        Some(&DiagnosticDetail::Reference {
            kind: "behavior tree",
            id: "nope".to_owned(),
            hint: None,
        })
    );
}

/// Fixture bt22, the second half: one finding per subtree node on the cycle.
#[test]
fn bt22_subtrees_that_form_a_cycle() {
    let found = linked(&[("a", &subtree("b")), ("b", &subtree("a"))]);
    assert_eq!(
        found,
        [at("ReferenceCycle", "a"), at("ReferenceCycle", "b")]
    );
    // a tree that is its own subtree
    assert_eq!(
        linked(&[("me", &subtree("me"))]),
        [at("ReferenceCycle", "me")]
    );
    // a longer cycle, and a tree that only leads into it
    let found = linked(&[
        ("a", &subtree("b")),
        ("b", &subtree("c")),
        ("c", &subtree("a")),
        ("into", &subtree("a")),
    ]);
    assert_eq!(
        found,
        [
            at("ReferenceCycle", "a"),
            at("ReferenceCycle", "b"),
            at("ReferenceCycle", "c")
        ]
    );
}

#[test]
fn a_subtree_that_resolves_is_no_finding() {
    // a pack tree, a built-in tree, a chain, a tree used twice
    assert!(linked(&[("a", &subtree("b")), ("b", r#"{"minecraft:wait": {}}"#)]).is_empty());
    assert!(linked(&[("a", &subtree("move"))]).is_empty());
    let twice = format!(
        r#"{{"minecraft:sequence": {{"children": [{}, {}]}}}}"#,
        subtree("leaf"),
        subtree("leaf")
    );
    assert!(linked(&[("a", &twice), ("leaf", r#"{"minecraft:wait": {}}"#)]).is_empty());
    assert!(linked(&[]).is_empty());
}

#[test]
fn every_subtree_node_of_a_tree_is_checked_at_its_own_path() {
    let nested = format!(
        r#"{{"minecraft:sequence": {{"children": [{{"minecraft:wait": {{}}}}, {{"minecraft:inverter": {{"child": {}}}}}, {}]}}}}"#,
        subtree("nope"),
        r#"{"minecraft:subtree": {}}"#
    );
    let found = linked(&[("t", &nested)]);
    let children = "minecraft:behavior_tree/minecraft:root/minecraft:sequence/children";
    assert_eq!(
        found,
        [
            (
                "UnresolvedReference",
                "behavior_trees/t.json".to_owned(),
                format!("{children}/1/minecraft:inverter/child/minecraft:subtree/subtree_id")
            ),
            // an absent subtree_id names no tree either
            (
                "UnresolvedReference",
                "behavior_trees/t.json".to_owned(),
                format!("{children}/2/minecraft:subtree/subtree_id")
            ),
        ]
    );
}

/// A cycle through a node deep inside a tree, beside a subtree that does not resolve: each
/// node gets its own finding, and a walk that meets a missing tree ends there.
#[test]
fn a_cycle_beside_a_missing_tree() {
    let both = format!(
        r#"{{"minecraft:selector": {{"children": [{}, {}]}}}}"#,
        subtree("gone"),
        subtree("a")
    );
    let found = linked(&[("a", &both)]);
    let children = "minecraft:behavior_tree/minecraft:root/minecraft:selector/children";
    assert_eq!(
        found,
        [
            (
                "ReferenceCycle",
                "behavior_trees/a.json".to_owned(),
                format!("{children}/1/minecraft:subtree/subtree_id")
            ),
            (
                "UnresolvedReference",
                "behavior_trees/a.json".to_owned(),
                format!("{children}/0/minecraft:subtree/subtree_id")
            ),
        ]
    );
}

#[test]
fn nodes_are_a_composites_children_and_a_decorators_child() {
    let registry = registry(&[(
        "t",
        r#"{"minecraft:sequence": {"children": [{"minecraft:inverter": {"child": {"minecraft:wait": {}}}}, {"minecraft:inverter": {}}, {"minecraft:use_entity": {}}]}}"#,
    )]);
    let tree = registry.iter().next().unwrap().1;
    let names = |nodes: &[Node]| nodes.iter().map(Node::type_name).collect::<Vec<_>>();
    assert_eq!(
        names(tree.root.nodes()),
        [
            "minecraft:inverter",
            "minecraft:inverter",
            "minecraft:use_entity"
        ]
    );
    assert_eq!(names(tree.root.nodes()[0].nodes()), ["minecraft:wait"]);
    assert!(tree.root.nodes()[1].nodes().is_empty());
    assert!(tree.root.nodes()[2].nodes().is_empty());
    let empty = registry_root(r#"{"minecraft:selector": {}}"#);
    assert!(empty.nodes().is_empty());
}

fn registry_root(root: &str) -> Node {
    let registry = registry(&[("t", root)]);
    registry.iter().next().unwrap().1.root.clone()
}

#[test]
fn every_built_in_text_names_its_tree_and_loads_without_a_finding() {
    assert_eq!(builtin::TREES.len(), 9);
    let mut names: Vec<&str> = builtin::TREES.iter().map(|(name, _)| *name).collect();
    names.dedup();
    assert!(names.is_sorted(), "{names:?}");
    assert_eq!(names.len(), 9);
    for (name, text) in builtin::TREES {
        let parsed = read::parse_document(text.as_bytes());
        assert_eq!(
            parsed.name.as_ref().map(bedrock_addon_core::id::Id::as_str),
            Some(name)
        );
        assert!(
            parsed.diagnostics.is_empty(),
            "{name}: {:?}",
            parsed.diagnostics
        );
        assert_eq!(parsed.tree.unwrap().raw, text);
    }
}

/// The findings of one tree come in document order.
#[test]
fn the_findings_of_a_tree_are_in_document_order() {
    let three = format!(
        r#"{{"minecraft:sequence": {{"children": [{}, {}, {}]}}}}"#,
        subtree("first"),
        subtree("second"),
        subtree("third")
    );
    let registry = registry(&[("t", &three)]);
    let ids: Vec<String> = link::link(&registry)
        .iter()
        .map(|finding| match finding.detail.as_deref() {
            Some(DiagnosticDetail::Reference { id, .. }) => id.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(ids, ["first", "second", "third"]);
}

/// 600 trees, each the subtree of the one before. The chain has
/// no finding; closed into a cycle, every tree's node is on it. The crate reads no clock, so
/// the bound on this test is the 10 minutes of CPU every test binary runs under.
#[test]
fn a_long_chain_and_a_long_cycle() {
    const TREES: usize = 600;
    let name = |index: usize| format!("t{index:03}");
    let chain: Vec<(String, String)> = (0..TREES)
        .map(|index| {
            let root = if index + 1 < TREES {
                subtree(&name(index + 1))
            } else {
                r#"{"minecraft:wait": {}}"#.to_owned()
            };
            (name(index), root)
        })
        .collect();
    let borrowed: Vec<(&str, &str)> = chain
        .iter()
        .map(|(name, root)| (name.as_str(), root.as_str()))
        .collect();
    assert!(linked(&borrowed).is_empty());
    let cycle: Vec<(String, String)> = (0..TREES)
        .map(|index| (name(index), subtree(&name((index + 1) % TREES))))
        .collect();
    let borrowed: Vec<(&str, &str)> = cycle
        .iter()
        .map(|(name, root)| (name.as_str(), root.as_str()))
        .collect();
    let found = linked(&borrowed);
    assert_eq!(found.len(), TREES);
    for (index, finding) in found.iter().enumerate() {
        assert_eq!(finding, &at("ReferenceCycle", &name(index)));
    }
    // two cycles joined by one edge: the joining node is on neither
    let found = linked(&[
        ("a", &subtree("b")),
        (
            "b",
            &format!(
                r#"{{"minecraft:sequence": {{"children": [{}, {}]}}}}"#,
                subtree("a"),
                subtree("c")
            ),
        ),
        ("c", &subtree("d")),
        ("d", &subtree("c")),
    ]);
    let codes: Vec<(&str, &str)> = found
        .iter()
        .map(|(code, file, _)| (*code, file.as_str()))
        .collect();
    assert_eq!(
        codes,
        [
            ("ReferenceCycle", "behavior_trees/a.json"),
            ("ReferenceCycle", "behavior_trees/b.json"),
            ("ReferenceCycle", "behavior_trees/c.json"),
            ("ReferenceCycle", "behavior_trees/d.json"),
        ]
    );
}

/// Each row of the node table: a tree whose root is the row's type with an empty body reads
/// as a node of that type.
#[test]
fn every_kinds_row_reads_an_empty_body_as_its_own_type() {
    use bedrock_addon_core::document::Content;
    for row in super::kinds::KINDS {
        let text = format!(
            r#"{{"minecraft:behavior_tree": {{"minecraft:name": "t", "minecraft:root": {{"{}": {{}}}}}}}}"#,
            row.name
        );
        let path = PackPath::new("behavior_trees/t.json").unwrap();
        let document = BehaviorTree::parse(&path, text.as_bytes(), &LoadContext::detached())
            .unwrap_or_else(|rejected| panic!("{}: {:?}", row.name, rejected.diagnostics));
        assert_eq!(document.root.type_name(), row.name);
    }
}
