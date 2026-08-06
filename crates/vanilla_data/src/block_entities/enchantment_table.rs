use facet::Facet;

/// An enchantment table tile entity.
#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct EnchantmentTable {
    /// The custom name of this enchantment table.
    ///
    /// May not exist.
    pub custom_name: Option<String>,
    /// The clockwise rotation of the book on the enchantment table in radians.
    ///
    /// Top of the book points west when set to 0.
    pub rotation: f32,
}
