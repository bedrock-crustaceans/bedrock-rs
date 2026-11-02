//! Constructing trees in memory: `BehaviorTree::new` and the two forms of a parameter.

use super::{BehaviorTree, Node, Param};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::BehaviorTreeId;

impl BehaviorTree {
    /// A tree named `name` with the root node `root`, no raw text and no other members.
    pub fn new(name: BehaviorTreeId, root: Node) -> Self {
        Self {
            name,
            root,
            raw: String::new(),
            extra: Extra::default(),
            root_extra: Extra::default(),
        }
    }
}

impl<T> Param<T> {
    /// A parameter given as a literal: the key is empty.
    pub fn literal(value: T) -> Self {
        Self {
            literal: value,
            key: String::new(),
        }
    }
}

impl<T: Default> Param<T> {
    /// A parameter read from the blackboard entry `key`; the literal is the type's default.
    ///
    /// It is written as the key member, without the literal. Two keys are not: the pair's
    /// default key is what an absent member reads as and writes nothing, and the key a read
    /// literal leaves (the empty string; [`Node::BLOCK_NAME_KEY`] for `find_block`'s block
    /// name, which is its default key too) stands for the literal, so `Param::key("")` is
    /// written as the literal member with the default literal: the only text that reads
    /// back to it.
    pub fn key(key: impl Into<String>) -> Self {
        Self {
            literal: T::default(),
            key: key.into(),
        }
    }
}

impl<T> Param<T> {
    /// A parameter read from the blackboard entry `key`, with the literal vanilla keeps beside
    /// it. For the two parameters whose literal is not the type's default when the literal
    /// member is absent: [`Node::FAILED_SEARCH_RADIUS`] and [`Node::INTERACT_DEFAULT_TICKS`].
    /// The literal is not written; a file that gives the key reads back to it.
    pub fn key_with(key: impl Into<String>, literal: T) -> Self {
        Self {
            literal,
            key: key.into(),
        }
    }
}

impl Node {
    /// The `search_radius` literal of `minecraft:find_block`, `minecraft:find_entity` and
    /// `minecraft:look_at_entity` when the literal member is absent or of the wrong type.
    pub const FAILED_SEARCH_RADIUS: i32 = 32;
    /// The `num_of_ticks` literal of `minecraft:interact` when the literal member is absent
    /// or of the wrong type.
    pub const INTERACT_DEFAULT_TICKS: i32 = 1;
    /// The key `minecraft:find_block`'s `block_name` has when its literal was read: build a
    /// block name given as a literal with this key, and it reads back equal.
    pub const BLOCK_NAME_KEY: &'static str = "block_name";
}
