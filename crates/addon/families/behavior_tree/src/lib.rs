//! Behavior trees: `BehaviorTree`, `Node`.
//!
//! Automation behavior trees: `behavior_trees/**/*.json` of a behaviour pack,
//! `minecraft:behavior_tree`. Not a version chain: no file has a `format_version` and no loader
//! reads one, so the family is one unversioned struct read by one hand-written walker.
//! Vanilla loads these files in development builds only; retail 1.26.45 has no loader.

#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(hidden)]
pub use link::link as link_subtrees;

mod builtin;
mod link;
mod load;
mod new;
mod parse;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::BehaviorTreeId;

#[doc(hidden)]
pub use load::load_registry;

/// A block position literal: `[x, y, z]` of integers.
pub type BlockPos = [i32; 3];

/// A position literal: `[x, y, z]` of numbers.
pub type Vec3 = [f32; 3];

/// A behavior-tree file (`minecraft:behavior_tree`): a named tree of action nodes that
/// vanilla's automation test framework runs on the local player.
///
/// JSON: `{"minecraft:behavior_tree": {"minecraft:name": "chop_wood", "minecraft:root":
/// {"minecraft:sequence": {"children": [...]}}}}`. The tree is keyed by `minecraft:name`; the
/// file name plays no part. The family has no `format_version`: vanilla reads only
/// `minecraft:behavior_tree` from the document root and only `minecraft:name` and
/// `minecraft:root` from it.
///
/// Vanilla loads these files in development builds only (the retail game has no loader), and
/// only the development commands `/bhave` and `/automate` run a tree.
#[derive(Clone, Debug, PartialEq)]
pub struct BehaviorTree {
    /// `minecraft:name`: the key of the tree, a non-empty string. A file without one is not
    /// loaded.
    pub name: BehaviorTreeId,
    /// `minecraft:root`: an object with exactly one member, the root node.
    pub root: Node,
    /// The whole file text as read (byte order mark, comments and line ends included): what a
    /// development server sends to a joining client as the payload of `AddBehaviorTreePacket`,
    /// one packet per tree. Empty for a tree built in memory. Writing a tree writes
    /// [`name`](Self::name) and [`root`](Self::root), never this text.
    pub raw: String,
    /// The members of `minecraft:behavior_tree` other than `minecraft:name` and
    /// `minecraft:root`. Vanilla reads none of them.
    pub extra: Extra,
    /// The members of the document root other than `minecraft:behavior_tree` (a
    /// `format_version`, for one). Vanilla reads none of them.
    pub root_extra: Extra,
}

/// A node parameter as vanilla keeps it: a literal and the name of a blackboard entry.
///
/// A leaf reads each parameter from a pair of members, a literal (`num_of_ticks`) and a key
/// (`num_of_ticks_id`). When the literal is present and of the right type, it is the value and
/// the key stays empty. Otherwise the literal keeps its default and the key is the `…_id`
/// member's text, or the parameter's default key when that member is absent. When the tree
/// runs, the blackboard entry named by `key` wins if it exists with the parameter's type;
/// the literal is used otherwise.
///
/// A literal of the wrong JSON type is ignored, not an error. An integer literal takes an
/// integer or a bool (`true` is 1), never a real such as `1.0`; a float literal takes any
/// number or a bool.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Param<T> {
    /// The literal: the value as written, or the parameter's default when the literal member
    /// is absent or of the wrong type.
    pub literal: T,
    /// The blackboard key: empty when the literal was read; otherwise the `…_id` member's
    /// text, or the parameter's default key.
    pub key: String,
}

/// One node of a behavior tree: a JSON object with exactly one member, whose name is the
/// node type (`minecraft:sequence`) and whose value is the node's body.
///
/// There are 24 node types. A body that is not an object reads as an empty one. Each
/// variant's `extra` holds the body members the node type does not read.
///
/// A tree read from a file nests at most as deep as the reader's limit of 128 JSON levels
/// (62 decorators around a leaf). A tree built in memory may nest deeper: it is dropped and
/// walked ([`nodes`](Self::nodes)) without a stack frame per level, and the writer refuses it,
/// but `Clone`, `PartialEq` and `Debug` recurse once per level, as derived impls do.
///
/// Because a node has its own `Drop`, a field cannot be moved out of a `Node` by pattern:
/// match on a reference, or take the field with `std::mem::take`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Node {
    /// `minecraft:sequence`: a composite over its children, in order.
    Sequence {
        /// `children`: an array of nodes. `None` when the member is absent or not an array;
        /// the node then has no children. An element that is not a one-member object, or
        /// whose type is unknown, is left out.
        children: Option<Vec<Node>>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:selector`: a composite over its children, in order.
    Selector {
        /// `children`: an array of nodes. `None` when the member is absent or not an array;
        /// the node then has no children. An element that is not a one-member object, or
        /// whose type is unknown, is left out.
        children: Option<Vec<Node>>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:repeat_until_success`: a decorator that repeats its child.
    RepeatUntilSuccess {
        /// `child`: one node. `None` when the member is absent, not a one-member object, or
        /// of an unknown type.
        child: Option<Box<Node>>,
        /// `desired_max_number_of_attempts` (default 0), or the key
        /// `max_number_of_attempts_id` (default key `max_number_of_attempts`).
        max_attempts: Param<i32>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:repeat_until_failure`: a decorator that repeats its child.
    RepeatUntilFailure {
        /// `child`: one node. `None` when the member is absent, not a one-member object, or
        /// of an unknown type.
        child: Option<Box<Node>>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:inverter`: a decorator that inverts its child's result.
    Inverter {
        /// `child`: one node. `None` when the member is absent, not a one-member object, or
        /// of an unknown type.
        child: Option<Box<Node>>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:subtree`: runs another tree.
    Subtree {
        /// `subtree_id`: the `minecraft:name` of the other tree, resolved when the node runs;
        /// empty when the member is absent. Loading checks neither that the tree exists nor
        /// that subtrees form no cycle.
        subtree_id: BehaviorTreeId,
        /// `data_items`: the blackboard entries copied into the subtree, as `(from, to)`. A
        /// string `s` is `(s, s)`; an object with both `from` and `to` is that pair; the
        /// member is one such item or an array of them. Any other item is left out.
        data_items: Vec<(String, String)>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:activate_tool`.
    ActivateTool {
        /// `block_pos` (default `[0, 0, 0]`), or the key `block_pos_id` (default key
        /// `target_block_pos`).
        block_pos: Param<BlockPos>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:attack`.
    Attack {
        /// `num_of_ticks` (default 0), or the key `num_of_ticks_id` (default key
        /// `target_num_of_ticks`).
        ticks: Param<i32>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:break_block`.
    BreakBlock {
        /// `block_pos` (default `[0, 0, 0]`), or the key `block_pos_id` (default key
        /// `target_block_pos`).
        block_pos: Param<BlockPos>,
        /// `num_of_ticks` (default 0), or the key `num_of_ticks_id` (default key
        /// `target_num_of_ticks`).
        ticks: Param<i32>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:consume_item`: no parameters.
    ConsumeItem {
        /// The body members: the node reads none.
        extra: Extra,
    },
    /// `minecraft:find_block`.
    FindBlock {
        /// `block_name` (default empty), or the key `block_name_id`. The key is `block_name`
        /// from the start, so it is `block_name` also when the literal was read.
        block_name: Param<String>,
        /// `search_radius`, or the key `search_radius_id` (default key
        /// `target_search_radius`). The literal is 32 when the member is absent or of the
        /// wrong type.
        search_radius: Param<i32>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:find_entity`.
    FindEntity {
        /// `entity_name` (default empty), or the key `entity_type` (default key
        /// `entity_name`).
        entity_name: Param<String>,
        /// `search_radius`, or the key `search_radius_id` (default key
        /// `target_search_radius`). The literal is 32 when the member is absent or of the
        /// wrong type.
        search_radius: Param<i32>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:fly`.
    ///
    /// Vanilla stores both literals in one member: `should_throw_event_if_no_state_change_
    /// necessary_bool`, when it is a bool, replaces the value of `should_be_flying_bool`, and
    /// the node uses that one value for both flags. Both `literal`s here are that value: the
    /// throw literal if it was read, else the flying literal, else `false`.
    Fly {
        /// `should_be_flying_bool`, or the key `should_be_flying_id` (default key
        /// `should_be_flying`).
        should_be_flying: Param<bool>,
        /// `should_throw_event_if_no_state_change_necessary_bool`, or the key
        /// `should_throw_event_if_no_state_change_necessary_id` (default key
        /// `should_throw_event_if_no_state_change_necessary`).
        throw_if_no_change: Param<bool>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:get_interaction_position_for_block`.
    GetInteractionPositionForBlock {
        /// `target_block_pos` (default `[0, 0, 0]`), or the key `target_block_pos_id` (default
        /// key `target_block_pos`).
        target: Param<BlockPos>,
        /// `anchor_block_pos` (default `[0, 0, 0]`), or the key `anchor_block_pos_id` (default
        /// key `anchor_block_pos`).
        anchor: Param<BlockPos>,
        /// `facing_string` (default empty), or the key `facing_string_id` (default key
        /// `facing_string`).
        facing: Param<String>,
        /// `max_search_distance` (default 0), or the key `max_search_distance_id` (default
        /// key `max_search_distance`).
        max_search_distance: Param<i32>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:interact`.
    Interact {
        /// `num_of_ticks` (default 1), or the key `num_of_ticks_id` (default key
        /// `target_num_of_ticks`).
        ticks: Param<i32>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:look_at`.
    LookAt {
        /// `block_pos` (default `[0, 0, 0]`), or the key `block_pos_id` (default key
        /// `target_block_pos`).
        block_pos: Param<BlockPos>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:look_at_entity`.
    LookAtEntity {
        /// `entity_name` (default empty), or the key `entity_type` (default key
        /// `entity_name`).
        entity_name: Param<String>,
        /// `search_radius`, or the key `search_radius_id` (default key
        /// `target_search_radius`). The literal is 32 when the member is absent or of the
        /// wrong type. Loaded, but the node never uses it.
        search_radius: Param<i32>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:move`.
    Move {
        /// `desired_direction` (default empty; `forward` in vanilla's files), or the key
        /// `direction_id` (default key `direction`).
        direction: Param<String>,
        /// `desired_second_direction` (default empty), or the key `second_direction_id`
        /// (default key `second_direction`).
        second_direction: Param<String>,
        /// `max_number_of_ticks` (default 0), or the key `number_of_ticks_id` (default key
        /// `number_of_ticks`).
        ticks: Param<i32>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:move_to`.
    MoveTo {
        /// `block_pos`, a position of numbers (default `[0, 0, 0]`), or the key
        /// `block_pos_id` (default key `target_block_pos`). The blackboard entry may be a
        /// position of numbers or a block position.
        pos: Param<Vec3>,
        /// `distance_epsilon` (default 0), or the key `distance_epsilon_id` (default key
        /// `target_distance_epsilon`).
        distance_epsilon: Param<f32>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:place_block`: no parameters.
    PlaceBlock {
        /// The body members: the node reads none.
        extra: Extra,
    },
    /// `minecraft:shoot_bow`.
    ShootBow {
        /// `num_of_shots` (default 0), or the key `num_of_shots_id` (default key
        /// `target_num_of_shots`).
        shots: Param<i32>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:use_entity`: no parameters.
    UseEntity {
        /// The body members: the node reads none.
        extra: Extra,
    },
    /// `minecraft:wait`: waits whole seconds.
    Wait {
        /// `wait_duration` as an integer, in seconds (default 0), or the same member as a
        /// string, which is then the key (default key `duration`).
        seconds: Param<i32>,
        /// The body members the node does not read.
        extra: Extra,
    },
    /// `minecraft:wait_ticks`.
    WaitTicks {
        /// `num_of_ticks` (default 0), or the key `num_of_ticks_id` (default key
        /// `target_num_of_ticks`).
        ticks: Param<i32>,
        /// The body members the node does not read.
        extra: Extra,
    },
}

impl Node {
    /// The node type as the JSON member names it: `minecraft:sequence`, `minecraft:wait`, ….
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Sequence { .. } => "minecraft:sequence",
            Self::Selector { .. } => "minecraft:selector",
            Self::RepeatUntilSuccess { .. } => "minecraft:repeat_until_success",
            Self::RepeatUntilFailure { .. } => "minecraft:repeat_until_failure",
            Self::Inverter { .. } => "minecraft:inverter",
            Self::Subtree { .. } => "minecraft:subtree",
            Self::ActivateTool { .. } => "minecraft:activate_tool",
            Self::Attack { .. } => "minecraft:attack",
            Self::BreakBlock { .. } => "minecraft:break_block",
            Self::ConsumeItem { .. } => "minecraft:consume_item",
            Self::FindBlock { .. } => "minecraft:find_block",
            Self::FindEntity { .. } => "minecraft:find_entity",
            Self::Fly { .. } => "minecraft:fly",
            Self::GetInteractionPositionForBlock { .. } => {
                "minecraft:get_interaction_position_for_block"
            }
            Self::Interact { .. } => "minecraft:interact",
            Self::LookAt { .. } => "minecraft:look_at",
            Self::LookAtEntity { .. } => "minecraft:look_at_entity",
            Self::Move { .. } => "minecraft:move",
            Self::MoveTo { .. } => "minecraft:move_to",
            Self::PlaceBlock { .. } => "minecraft:place_block",
            Self::ShootBow { .. } => "minecraft:shoot_bow",
            Self::UseEntity { .. } => "minecraft:use_entity",
            Self::Wait { .. } => "minecraft:wait",
            Self::WaitTicks { .. } => "minecraft:wait_ticks",
        }
    }

    /// The nodes directly below this one, in order: a composite's children, a decorator's
    /// child; none for any other node.
    pub fn nodes(&self) -> &[Node] {
        match self {
            Self::Sequence { children, .. } | Self::Selector { children, .. } => {
                children.as_deref().unwrap_or_default()
            }
            Self::RepeatUntilSuccess { child, .. }
            | Self::RepeatUntilFailure { child, .. }
            | Self::Inverter { child, .. } => child.as_deref().map_or(&[], std::slice::from_ref),
            _ => &[],
        }
    }

    /// The body members the node does not read.
    #[must_use]
    pub fn extra(&self) -> &Extra {
        match self {
            Self::Sequence { extra, .. }
            | Self::Selector { extra, .. }
            | Self::RepeatUntilSuccess { extra, .. }
            | Self::RepeatUntilFailure { extra, .. }
            | Self::Inverter { extra, .. }
            | Self::Subtree { extra, .. }
            | Self::ActivateTool { extra, .. }
            | Self::Attack { extra, .. }
            | Self::BreakBlock { extra, .. }
            | Self::ConsumeItem { extra }
            | Self::FindBlock { extra, .. }
            | Self::FindEntity { extra, .. }
            | Self::Fly { extra, .. }
            | Self::GetInteractionPositionForBlock { extra, .. }
            | Self::Interact { extra, .. }
            | Self::LookAt { extra, .. }
            | Self::LookAtEntity { extra, .. }
            | Self::Move { extra, .. }
            | Self::MoveTo { extra, .. }
            | Self::PlaceBlock { extra }
            | Self::ShootBow { extra, .. }
            | Self::UseEntity { extra }
            | Self::Wait { extra, .. }
            | Self::WaitTicks { extra, .. } => extra,
        }
    }
}

impl Node {
    /// The nodes directly below this one, moved out of it: the node is left without them.
    fn take_nodes(&mut self) -> Vec<Node> {
        match self {
            Self::Sequence { children, .. } | Self::Selector { children, .. } => {
                children.take().unwrap_or_default()
            }
            Self::RepeatUntilSuccess { child, .. }
            | Self::RepeatUntilFailure { child, .. }
            | Self::Inverter { child, .. } => {
                child.take().map(|node| vec![*node]).unwrap_or_default()
            }
            _ => Vec::new(),
        }
    }
}

/// Drops the nodes below without a stack frame per level: a tree built in memory may nest
/// without bound, and the drop the compiler writes recurses through every `child`.
impl Drop for Node {
    fn drop(&mut self) {
        // Every node is taken apart on this one list; a node dropped at the end of a turn of
        // the loop has no node below it left, so its own drop goes no deeper.
        let mut pending = self.take_nodes();
        while let Some(mut node) = pending.pop() {
            pending.append(&mut node.take_nodes());
        }
    }
}

#[cfg(test)]
mod param_derive_tests {
    use super::Param;
    use std::collections::HashSet;

    #[test]
    fn a_param_of_an_integer_is_eq_and_hash() {
        let a = Param {
            literal: 1_i32,
            key: String::new(),
        };
        let mut set = HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&a));
        assert_eq!(a, a.clone());
    }
}
