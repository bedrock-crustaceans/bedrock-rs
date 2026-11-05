//! Placing and digging: `block_placer`, `digger`, `entity_placer`.

use bedrock_addon_core::id::EntityId;
use bedrock_addon_core::shared::{BlockDescriptor, BlockSpecifier, EventTrigger};
use facet::Facet;

use super::{visit_block_descriptor, visit_block_descriptors};
use bedrock_addon_core::schema::{self, Schema};

/// `minecraft:block_placer`: using the item places `block`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct BlockPlacer {
    /// The block placed (JSON `block`, required).
    pub block: BlockSpecifier,
    /// The blocks it may be placed on; empty means any (JSON `use_on`; at most 256 distinct
    /// entries from 1.26.0).
    #[facet(default, skip_serializing_if = Vec::is_empty, schema::molang_with = visit_block_descriptors)]
    pub use_on: Vec<BlockDescriptor>,
    /// The item is the block's item: it replaces the block's own item (JSON
    /// `replace_block_item`, from 1.21.40; default false).
    pub replace_block_item: Option<bool>,
    /// The block is aligned to the player's facing (JSON `aligned_placement`, from 1.26.0,
    /// engine 1.26.10; default false).
    pub aligned_placement: Option<bool>,
}

impl BlockPlacer {
    /// Places `block` on anything.
    pub fn new(block: BlockSpecifier) -> Self {
        Self {
            block,
            use_on: Vec::new(),
            replace_block_item: None,
            aligned_placement: None,
        }
    }
}

/// `minecraft:digger`: how fast the item breaks blocks.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Digger {
    /// The speed per block (JSON `destroy_speeds`).
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub destroy_speeds: Vec<DestroySpeed>,
    /// The `efficiency` enchantment applies (JSON `use_efficiency`).
    pub use_efficiency: Option<bool>,
    /// The deprecated `on_dig` trigger, as the 1.20.50 upgrade keeps it under its new name
    /// (JSON `__deprecated_on_dig`).
    #[facet(rename = "__deprecated_on_dig")]
    pub deprecated_on_dig: Option<EventTrigger>,
}

/// One entry of a [`Digger`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DestroySpeed {
    /// The blocks (JSON `block`, required).
    #[facet(schema::molang_with = visit_block_descriptor)]
    pub block: BlockDescriptor,
    /// The speed (JSON `speed`, required).
    pub speed: i32,
    /// The deprecated per-block `on_dig` trigger (JSON `__deprecated_on_dig`).
    #[facet(rename = "__deprecated_on_dig")]
    pub deprecated_on_dig: Option<EventTrigger>,
}

impl Digger {
    /// Whether a trigger is set: the top-level one, or a per-block one with a non-empty event.
    /// Such a digger is copied to the side-car by the 1.26.30 step.
    pub(crate) fn has_triggers(&self) -> bool {
        self.deprecated_on_dig.is_some()
            || self.destroy_speeds.iter().any(|speed| {
                speed
                    .deprecated_on_dig
                    .as_ref()
                    .is_some_and(|trigger| trigger.event.as_deref().is_some_and(|e| !e.is_empty()))
            })
    }
}

/// `minecraft:entity_placer`: using the item spawns `entity`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct EntityPlacer {
    /// The entity spawned, with an optional `<event>` (JSON `entity`).
    pub entity: Option<EntityId>,
    /// The blocks it may be used on; empty means any (JSON `use_on`).
    #[facet(default, skip_serializing_if = Vec::is_empty, schema::molang_with = visit_block_descriptors)]
    pub use_on: Vec<BlockDescriptor>,
    /// The blocks a dispenser may place it on (JSON `dispense_on`).
    #[facet(default, skip_serializing_if = Vec::is_empty, schema::molang_with = visit_block_descriptors)]
    pub dispense_on: Vec<BlockDescriptor>,
}
