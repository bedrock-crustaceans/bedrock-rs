use bedrock_level::types::ItemStack;
use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct ItemFrame {
    /// Degrees of rotation, always a `Float` in every record this crate has
    /// been checked against.
    ///
    /// Pre-1.13 records wrote this as a `Byte` step index (0-7, 45° per
    /// step), not degrees, so a bare `lenient_width(i8)` on this field would
    /// decode the raw byte as degrees directly — silently wrong, and, since
    /// widening is decode-only, permanently baked in on the next write.
    /// Reading that legacy encoding correctly needs the `* 45` conversion
    /// applied on the way in, which is out of scope here; until that lands,
    /// a pre-1.13 record's rotation is left unreadable rather than
    /// misread.
    pub item_rotation: f32,
    pub item_drop_chance: f32,
    pub item: ItemStack,
}
