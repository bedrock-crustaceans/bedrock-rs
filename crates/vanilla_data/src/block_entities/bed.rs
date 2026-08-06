use bedrock_level::color::Color;
use facet::Facet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Bed {
    /// Color of the bed.
    pub color: Color,
}
