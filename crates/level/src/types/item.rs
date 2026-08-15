use facet::Facet;

use crate::subchunk::BlockDef;

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct ItemStack {
    pub block: Option<BlockDef>,
    pub can_destroy: Option<Vec<String>>,
    pub can_place_on: Option<Vec<String>>,
    pub count: i8,
    pub damage: i16,
    pub name: String,
    #[facet(rename = "tag")]
    pub tag: Option<nbtx::Value>,
    /// Whether the stack has been picked up.
    pub was_picked_up: bool,
}

/// An [`ItemStack`] together with the container slot it occupies.
///
/// # Wire shape
///
/// One flat compound: `Slot` sits alongside the stack's own keys rather than
/// nesting them. nbtx has no way to merge a field's keys into its parent's
/// compound, so the stack's fields are inlined here instead of composed. They
/// keep the same order and the same NBT names, and
/// [`Self::stack`]/[`Self::from_parts`] convert between the two views.
///
/// Inlining rather than decoding this type by hand is what lets it keep working
/// where it is used — as `Vec<ItemSlot>` inside `Dispenser`, `Furnace` and
/// `Hopper`, which a hand-written `ItemSlot` codec would never be reached from.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct ItemSlot {
    pub slot: i8,
    pub block: Option<BlockDef>,
    pub can_destroy: Option<Vec<String>>,
    pub can_place_on: Option<Vec<String>>,
    pub count: i8,
    pub damage: i16,
    pub name: String,
    #[facet(rename = "tag")]
    pub tag: Option<nbtx::Value>,
    /// See [`ItemStack::was_picked_up`].
    pub was_picked_up: bool,
}

impl ItemSlot {
    /// Splits this entry into its slot index and the stack it holds.
    pub fn into_parts(self) -> (i8, ItemStack) {
        (
            self.slot,
            ItemStack {
                block: self.block,
                can_destroy: self.can_destroy,
                can_place_on: self.can_place_on,
                count: self.count,
                damage: self.damage,
                name: self.name,
                tag: self.tag,
                was_picked_up: self.was_picked_up,
            },
        )
    }

    /// The stack this entry holds, cloned out of it.
    pub fn stack(&self) -> ItemStack {
        self.clone().into_parts().1
    }

    /// Builds an entry from a slot index and a stack.
    pub fn from_parts(slot: i8, stack: ItemStack) -> Self {
        Self {
            slot,
            block: stack.block,
            can_destroy: stack.can_destroy,
            can_place_on: stack.can_place_on,
            count: stack.count,
            damage: stack.damage,
            name: stack.name,
            tag: stack.tag,
            was_picked_up: stack.was_picked_up,
        }
    }
}
