//! The node types, one row each: the variant and the JSON member that names it, which is the
//! variant's `rename`. The rows give [`Node::type_name`] and the reader's lookup of a type by
//! name (an unknown one is the game's own line, not the read's); the body of each is read by
//! [`Node`]'s derived read.
//!
//! The writer (`write::node_value`) and the link pass match on the variant instead: each arm
//! destructures its own fields, which a row of this table cannot hold.

use super::Node;

/// One node type: its name.
pub(super) struct Kind {
    pub(super) name: &'static str,
}

/// The table and [`Node::type_name`] from one list of rows: `Variant "name"`.
macro_rules! kinds {
    ($($variant:ident $name:literal,)*) => {
        /// Every node type, in the order of [`Node`]'s variants.
        pub(super) const KINDS: &[Kind] = &[$(Kind { name: $name },)*];

        impl Node {
            /// The node type as the JSON member names it: `minecraft:sequence`,
            /// `minecraft:wait`, ….
            pub fn type_name(&self) -> &'static str {
                match self {
                    $(Self::$variant { .. } => $name,)*
                }
            }
        }
    };
}

// `BehaviorFactory::loadNodeDefinition` and the 21 `load` overrides.
kinds! {
    Sequence "minecraft:sequence",
    Selector "minecraft:selector",
    RepeatUntilSuccess "minecraft:repeat_until_success",
    RepeatUntilFailure "minecraft:repeat_until_failure",
    Inverter "minecraft:inverter",
    Subtree "minecraft:subtree",
    ActivateTool "minecraft:activate_tool",
    Attack "minecraft:attack",
    BreakBlock "minecraft:break_block",
    ConsumeItem "minecraft:consume_item",
    FindBlock "minecraft:find_block",
    FindEntity "minecraft:find_entity",
    Fly "minecraft:fly",
    GetInteractionPositionForBlock "minecraft:get_interaction_position_for_block",
    Interact "minecraft:interact",
    LookAt "minecraft:look_at",
    LookAtEntity "minecraft:look_at_entity",
    Move "minecraft:move",
    MoveTo "minecraft:move_to",
    PlaceBlock "minecraft:place_block",
    ShootBow "minecraft:shoot_bow",
    UseEntity "minecraft:use_entity",
    Wait "minecraft:wait",
    WaitTicks "minecraft:wait_ticks",
}

/// The node type named `name`, when there is one.
pub(super) fn find(name: &str) -> Option<&'static Kind> {
    KINDS.iter().find(|kind| kind.name == name)
}
