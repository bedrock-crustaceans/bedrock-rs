use bedrock_level::types::ItemStack;
use facet::Facet;

/// An ender chest tile entity.
///
/// The real records in the fixture never carry a non-empty `Items` list — a
/// player's ender chest inventory is stored per-player, not on the block —
/// so the element type this crate's `Chest` already settled on ([`ItemStack`],
/// no per-entry slot index) is carried over rather than independently
/// re-derived; nothing in the corpus distinguishes the two shapes here.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct EnderChest {
    /// Every real record writes this as a `Byte`, uniformly — unlike
    /// `Chest::findable`, no `Int`-tagged record has been seen, but the same
    /// lenient acceptance is kept for consistency with every other container
    /// that carries this key.
    #[facet(nbtx::lenient_width(i8))]
    pub findable: i32,
    pub items: Vec<ItemStack>,
}
