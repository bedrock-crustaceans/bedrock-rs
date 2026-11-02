//! The subtree references the link pass resolves:
//! the checker the stack's link pass calls with the stack's registry.
//! Vanilla checks nothing when it loads: a missing tree is only found when the node runs
//! (`SubtreeNode: Could not get definition for tree id {}.`).

use std::collections::HashMap;

use super::parse::{PAYLOAD, ROOT, child, element_of};
use super::{BehaviorTree, Node};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{Document, Registry};
use bedrock_addon_core::id::BehaviorTreeId;

/// The `minecraft:subtree` nodes of `tree` in document order: each node's target and the
/// path of its `subtree_id`.
fn subtrees(tree: &BehaviorTree) -> Vec<(&BehaviorTreeId, JsonPath)> {
    let root_at = child(&child(&JsonPath::new(), PAYLOAD), ROOT);
    let mut found = Vec::new();
    // An explicit stack: a tree built in memory may nest deeper than a parsed one.
    let mut pending = vec![(&tree.root, root_at)];
    while let Some((node, at)) = pending.pop() {
        let body_at = child(&at, node.type_name());
        match node {
            Node::Subtree { subtree_id, .. } => {
                found.push((subtree_id, child(&body_at, "subtree_id")));
            }
            Node::Sequence { .. } | Node::Selector { .. } => {
                let children_at = child(&body_at, "children");
                // Pushed last to first, so the first child is visited first.
                for (index, node) in node.nodes().iter().enumerate().rev() {
                    pending.push((node, element_of(&children_at, index)));
                }
            }
            _ => {
                for node in node.nodes() {
                    pending.push((node, child(&body_at, "child")));
                }
            }
        }
    }
    found
}

/// The strongly connected component of every tree of a graph given as `edges[tree]` (the
/// trees its subtree nodes resolve to): two trees are in one component when each reaches the
/// other. One pass forward and one backward (Kosaraju), each with an explicit stack and a
/// visited mark per tree, so every tree and every edge is handled a bounded number of times
/// on any graph.
fn components(edges: &[Vec<usize>]) -> Vec<usize> {
    let count = edges.len();
    // Forward: the trees in the order their walks finish.
    let mut finished = Vec::with_capacity(count);
    let mut visited = vec![false; count];
    for start in 0..count {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        // `(tree, the next of its edges to follow)`.
        let mut walk = vec![(start, 0_usize)];
        while let Some((tree, next)) = walk.last_mut() {
            if let Some(&target) = edges[*tree].get(*next) {
                *next += 1;
                if !visited[target] {
                    visited[target] = true;
                    walk.push((target, 0));
                }
            } else {
                finished.push(*tree);
                walk.pop();
            }
        }
    }
    // Backward, latest finished first: what reaches a tree against the edges is its component.
    let mut reversed = vec![Vec::new(); count];
    for (tree, targets) in edges.iter().enumerate() {
        for &target in targets {
            reversed[target].push(tree);
        }
    }
    let mut component = vec![usize::MAX; count];
    for &start in finished.iter().rev() {
        if component[start] != usize::MAX {
            continue;
        }
        component[start] = start;
        let mut pending = vec![start];
        while let Some(tree) = pending.pop() {
            for &source in &reversed[tree] {
                if component[source] == usize::MAX {
                    component[source] = start;
                    pending.push(source);
                }
            }
        }
    }
    component
}

/// A link finding at `at` of `tree`'s file.
fn finding(tree: &Document<BehaviorTree>, at: JsonPath, code: DiagnosticCode) -> Diagnostic {
    let origin = tree.origin();
    Diagnostic::new(
        code,
        // Nothing at load; "SubtreeNode: Could not get definition for
        // tree id {}." is a run-time line.
        Severity::Info,
        Phase::Link,
        Location::new(
            Some(origin.pack.clone()),
            Some(origin.path.clone()),
            at,
            None,
        ),
    )
}

/// The findings about the `minecraft:subtree` nodes of every effective tree of `trees`: a
/// `subtree_id` that names no tree of the registry (a pack's or a built-in one) is an
/// `UnresolvedReference` warning, and one through which the tree reaches itself a
/// `ReferenceCycle` warning; one finding per node, in registry and document order.
pub fn link(trees: &Registry<BehaviorTree>) -> Diagnostics {
    // Every tree's subtree nodes, read once.
    let index: HashMap<&BehaviorTreeId, usize> = trees
        .iter()
        .enumerate()
        .map(|(index, (name, _))| (name, index))
        .collect();
    let nodes: Vec<_> = trees
        .iter()
        .map(|(_, tree)| (tree, subtrees(tree)))
        .collect();
    let edges: Vec<Vec<usize>> = nodes
        .iter()
        .map(|(_, subtrees)| {
            subtrees
                .iter()
                .filter_map(|(target, _)| index.get(target).copied())
                .collect()
        })
        .collect();
    // A node is on a cycle when its tree and its target reach each other.
    let component = components(&edges);
    let mut out = Diagnostics::default();
    for (source, (tree, subtrees)) in nodes.into_iter().enumerate() {
        for (target, at) in subtrees {
            let code = match index.get(target) {
                None => DiagnosticCode::UnresolvedReference { text: None },
                Some(&found) if component[found] == component[source] => {
                    DiagnosticCode::ReferenceCycle { text: None }
                }
                Some(_) => continue,
            };
            // Either finding names the tree the node names, as written.
            let mut told = finding(tree, at, code);
            told.detail = Some(Box::new(DiagnosticDetail::Reference {
                kind: "behavior tree",
                id: target.as_str().to_owned(),
                hint: None,
            }));
            out.push(told);
        }
    }
    out
}
