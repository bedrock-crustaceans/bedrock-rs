//! The tree a `minecraft:subtree` node of a behaviour tree names.

use super::super::{Findings, LinkCx};
use crate::diagnostic::DiagnosticDetail;
use crate::families::behavior_tree;
use crate::id::BehaviorTreeId;

/// Each `subtree_id` that names no tree is an `UnresolvedReference` finding, and each through
/// which its tree reaches itself a `ReferenceCycle` finding, at the member. The registry holds
/// the trees of the packs and the nine the game has in its code; a host answers for any other
/// tree of the game.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let trees = cx.behavior().behavior_trees();
    for finding in behavior_tree::link_subtrees(trees) {
        // The link raises `UNRESOLVED_REFERENCE` or `REFERENCE_CYCLE`.
        let hosted = finding.code.id() == "core/unresolved_reference"
            && matches!(
                finding.detail.as_deref(),
                Some(DiagnosticDetail::Reference { id, .. })
                    if !cx.behavior_tree(&BehaviorTreeId::parse(id.as_str())).is_missing()
            );
        if !hosted {
            out.push(finding);
        }
    }
}
