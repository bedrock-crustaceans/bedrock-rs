use bedrock_level::types::ItemStack;
use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct ItemFrame {
    /// Degrees of rotation. `Float` in the only 2 real records checked so
    /// far (this crate's test fixture is a 1.26 world with no older item
    /// frames in it) — not enough to say what other versions write. No
    /// `lenient_width` here without more evidence: see `TODO.md` for a
    /// specific, externally-sourced reason to expect older records need one,
    /// and why a bare `lenient_width(i8)` would be the wrong fix for it.
    pub item_rotation: f32,
    pub item_drop_chance: f32,
    pub item: ItemStack,
}
