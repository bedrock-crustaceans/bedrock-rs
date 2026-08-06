use facet::Facet;


#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Bell {
    /// The direction the bell is facing.
    pub direction: i32,
    /// Whether the bell is currently ringing.
    pub ringing: bool,
    /// Unknown.
    pub ticks: i32,
}
