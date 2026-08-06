use bedrock_level::types::Effect;
use facet::Facet;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Beacon {
    pub primary: Effect,
    pub secondary: Effect,
}
