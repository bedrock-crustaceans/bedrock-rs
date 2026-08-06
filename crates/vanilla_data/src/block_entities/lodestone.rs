use facet::Facet;

/// A lodestone tile entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Lodestone {
    /// The tracking ID of the lodestone.
    #[facet(rename = "trackingHandle")]
    pub tracking_handle: Option<i32>,
}
