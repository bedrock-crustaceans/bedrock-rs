use facet::Facet;

/// A noteblock tile entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Noteblock {
    /// The pitch of the note block.
    pub note: i8,
}
