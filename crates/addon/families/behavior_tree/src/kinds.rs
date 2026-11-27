//! The node types, one row each: the variant, the JSON member that names it, and how its body
//! is read. The rows give [`Node::type_name`] and the reader's lookup of a type by name, so a
//! type's name is written once.
//!
//! The writer (`write::node_value`) and the link pass match on the variant instead: each arm
//! destructures its own fields, which a row of this table cannot hold.

use super::Node;
use super::read::{
    ANCHOR_BLOCK_POS, BLOCK_NAME, BLOCK_POS, Body, DIRECTION, DISTANCE_EPSILON, ENTITY_NAME,
    FACING_STRING, FAILED_SEARCH_RADIUS, MAX_ATTEMPTS, MAX_NUMBER_OF_TICKS, MAX_SEARCH_DISTANCE,
    NUM_OF_SHOTS, NUM_OF_TICKS, Reader, SEARCH_RADIUS, SECOND_DIRECTION, TARGET_BLOCK_POS,
};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::id::BehaviorTreeId;

/// One node type: its name and the read of its body (the body's members and the path of the
/// body).
pub(super) struct Kind {
    pub(super) name: &'static str,
    pub(super) read: fn(&mut Reader<'_>, &mut Body<'_>, &JsonPath) -> Node,
}

/// The table and [`Node::type_name`] from one list of rows: `Variant "name" => read`, the read
/// an expression over the reader, the body and the body's path named once at the top.
macro_rules! kinds {
    (|$r:ident, $m:ident, $at:ident| $($variant:ident $name:literal => $read:expr,)*) => {
        /// Every node type, in the order of [`Node`]'s variants.
        #[allow(
            unused_variables,
            reason = "a type without members reads neither its body nor its path"
        )]
        pub(super) const KINDS: &[Kind] = &[$(Kind {
            name: $name,
            read: |$r: &mut Reader<'_>, $m: &mut Body<'_>, $at: &JsonPath| -> Node { $read },
        },)*];

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

// `BehaviorFactory::loadNodeDefinition` and the 21 `load` overrides; the keys, default keys and
// defaults are those of the overrides. A row's members are read in the order its fields are
// written.
kinds! { |r, m, at|
    Sequence "minecraft:sequence" => Node::Sequence {
        children: r.children(m, at),
    },
    Selector "minecraft:selector" => Node::Selector {
        children: r.children(m, at),
    },
    RepeatUntilSuccess "minecraft:repeat_until_success" => Node::RepeatUntilSuccess {
        max_attempts: r.param(m, &MAX_ATTEMPTS, 0),
        child: r.child(m, at),
    },
    RepeatUntilFailure "minecraft:repeat_until_failure" => Node::RepeatUntilFailure {
        child: r.child(m, at),
    },
    Inverter "minecraft:inverter" => Node::Inverter {
        child: r.child(m, at),
    },
    Subtree "minecraft:subtree" => Node::Subtree {
        subtree_id: BehaviorTreeId::parse(r.text(m.take("subtree_id").as_ref(), "")),
        data_items: r.data_items(m),
    },
    ActivateTool "minecraft:activate_tool" => Node::ActivateTool {
        block_pos: r.param(m, &BLOCK_POS, [0; 3]),
    },
    Attack "minecraft:attack" => Node::Attack {
        ticks: r.param(m, &NUM_OF_TICKS, 0),
    },
    BreakBlock "minecraft:break_block" => Node::BreakBlock {
        block_pos: r.param(m, &BLOCK_POS, [0; 3]),
        ticks: r.param(m, &NUM_OF_TICKS, 0),
    },
    ConsumeItem "minecraft:consume_item" => Node::ConsumeItem {},
    FindBlock "minecraft:find_block" => Node::FindBlock {
        block_name: r.param(m, &BLOCK_NAME, String::new()),
        search_radius: r.param(m, &SEARCH_RADIUS, FAILED_SEARCH_RADIUS),
    },
    FindEntity "minecraft:find_entity" => Node::FindEntity {
        entity_name: r.param(m, &ENTITY_NAME, String::new()),
        search_radius: r.param(m, &SEARCH_RADIUS, FAILED_SEARCH_RADIUS),
    },
    Fly "minecraft:fly" => {
        let (should_be_flying, throw_if_no_change) = r.fly(m, at);
        Node::Fly {
            should_be_flying,
            throw_if_no_change,
        }
    },
    GetInteractionPositionForBlock "minecraft:get_interaction_position_for_block" => {
        Node::GetInteractionPositionForBlock {
            target: r.param(m, &TARGET_BLOCK_POS, [0; 3]),
            anchor: r.param(m, &ANCHOR_BLOCK_POS, [0; 3]),
            facing: r.param(m, &FACING_STRING, String::new()),
            max_search_distance: r.param(m, &MAX_SEARCH_DISTANCE, 0),
        }
    },
    Interact "minecraft:interact" => Node::Interact {
        // `make_unique<InteractActionDefinition>` stores 1.
        ticks: r.param(m, &NUM_OF_TICKS, Node::INTERACT_DEFAULT_TICKS),
    },
    LookAt "minecraft:look_at" => Node::LookAt {
        block_pos: r.param(m, &BLOCK_POS, [0; 3]),
    },
    LookAtEntity "minecraft:look_at_entity" => Node::LookAtEntity {
        entity_name: r.param(m, &ENTITY_NAME, String::new()),
        search_radius: r.param(m, &SEARCH_RADIUS, FAILED_SEARCH_RADIUS),
    },
    Move "minecraft:move" => Node::Move {
        direction: r.param(m, &DIRECTION, String::new()),
        second_direction: r.param(m, &SECOND_DIRECTION, String::new()),
        ticks: r.param(m, &MAX_NUMBER_OF_TICKS, 0),
    },
    MoveTo "minecraft:move_to" => Node::MoveTo {
        pos: r.param(m, &BLOCK_POS, [0.0; 3]),
        distance_epsilon: r.param(m, &DISTANCE_EPSILON, 0.0),
    },
    PlaceBlock "minecraft:place_block" => Node::PlaceBlock {},
    ShootBow "minecraft:shoot_bow" => Node::ShootBow {
        shots: r.param(m, &NUM_OF_SHOTS, 0),
    },
    UseEntity "minecraft:use_entity" => Node::UseEntity {},
    Wait "minecraft:wait" => Node::Wait {
        seconds: r.wait_duration(m),
    },
    WaitTicks "minecraft:wait_ticks" => Node::WaitTicks {
        ticks: r.param(m, &NUM_OF_TICKS, 0),
    },
}

/// The node type named `name`, when there is one.
pub(super) fn find(name: &str) -> Option<&'static Kind> {
    KINDS.iter().find(|kind| kind.name == name)
}
