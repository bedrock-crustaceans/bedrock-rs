use facet::Facet;

/// A conduit tile entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Conduit {
    /// Whether the conduit is active.
    pub active: bool,
    /// The unique ID of the hostile mob the conduit is currently attacking. If there is no target,
    /// this is set to -1.
    pub target: i64,
}
