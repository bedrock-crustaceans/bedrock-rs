use facet::Facet;

/// An enchantment table tile entity.
#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct EnchantmentTable {
    /// The custom name of this enchantment table.
    ///
    /// May not exist. Unverified against a real record — the one real
    /// `EnchantTable` record checked so far has no `CustomName` key at all —
    /// but every other block entity with a custom name in this crate writes
    /// it under the key `CustomName` (`CommandBlock::custom_name`, via its
    /// struct's `rename_all = "PascalCase"`), and this struct has no
    /// `rename_all`, so the field needs the rename spelled out explicitly to
    /// match: without it, the field would look for the literal key
    /// `custom_name`, which is not what the format uses anywhere else.
    #[facet(rename = "CustomName")]
    pub custom_name: Option<String>,
    /// The clockwise rotation of the book on the enchantment table in radians.
    ///
    /// Top of the book points west when set to 0. Stored under the key
    /// `rott`, confirmed against a real record.
    #[facet(rename = "rott")]
    pub rotation: f32,
}
