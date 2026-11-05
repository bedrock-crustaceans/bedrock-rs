//! The event tree of entity and item events: which members of an event node nest further
//! nodes ([`children`]), and the finding the game logs for a node with both `randomize` and
//! `sequence` ([`sequence_ignored`]).
//!
//! An event node is JSON as written: its nested lists, and every other member a response. The
//! families keep the walk itself (an explicit stack, a recursion over every member, the
//! document order of their findings); this module answers what each step asks of one node.
//! Block events are typed and take only the finding; particle events take nothing from here.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, EVENT_BOTH_RANDOMIZE_AND_SEQUENCE, JsonPath, VanillaText,
};
use bedrock_addon_core::json::Value;

/// The members of an event node that hold nested nodes, in the order the explicit-stack
/// walkers visit them (entity's references). The recursive walkers keep their own order: the
/// document's, or entity's expression `collect`, which visits `first_valid` before `randomize`.
pub const LISTS: [&str; 3] = ["sequence", "randomize", "first_valid"];

/// The nested lists of one event node.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Children<'a> {
    /// Each member of [`LISTS`] the node has as an array, in [`LISTS`] order, with its
    /// elements. A list member of another JSON type nests nothing and is left out. Both lists
    /// are here when the sequence is [ignored](Self::sequence_ignored).
    pub lists: Vec<(&'static str, &'a [Value])>,
    /// The node has both `randomize` and `sequence`, of any JSON type: the game logs
    /// [`sequence_ignored`] and loads the `randomize` alone. A family may still read both:
    /// entity validates both lists on purpose (its `node()` in `validate.rs`), while item's
    /// expression walk skips the `sequence` as the game does.
    pub sequence_ignored: bool,
    /// The node has a member of [`LISTS`], of any JSON type: it is a collection, whose other
    /// members are not a response collection.
    pub has_collection: bool,
}

/// The nested lists of `node`; none for a value that is no object.
#[must_use]
pub fn children(node: &Value) -> Children<'_> {
    let Some(object) = node.as_object() else {
        return Children::default();
    };
    let lists = LISTS
        .into_iter()
        .filter_map(|key| {
            let list = object.get(key)?.as_array()?;
            Some((key, list.as_slice()))
        })
        .collect();
    Children {
        lists,
        sequence_ignored: is_sequence_ignored(node),
        has_collection: has_collection(node),
    }
}

/// [`Children::sequence_ignored`] of `node` without building the lists: it has both
/// `randomize` and `sequence`, of any JSON type. `false` for a value that is no object.
#[must_use]
pub fn is_sequence_ignored(node: &Value) -> bool {
    node.as_object()
        .is_some_and(|object| object.contains_key("randomize") && object.contains_key("sequence"))
}

/// [`Children::has_collection`] of `node` without building the lists: it has a member of
/// [`LISTS`], of any JSON type. `false` for a value that is no object.
#[must_use]
pub fn has_collection(node: &Value) -> bool {
    node.as_object()
        .is_some_and(|object| LISTS.iter().any(|key| object.contains_key(key)))
}

/// The game's warning for the event node at `node_at` that has both `randomize` and
/// `sequence`, located at its `sequence`. Told in the parse phase; a validator chains
/// [`Diagnostic::phase`].
#[must_use]
pub fn sequence_ignored(node_at: &JsonPath) -> Diagnostic {
    Diagnostic::warn(DiagnosticCode::IgnoredMember {
        text: Some(VanillaText::Fixed(EVENT_BOTH_RANDOMIZE_AND_SEQUENCE)),
    })
    .at(node_at.child("sequence"))
}
