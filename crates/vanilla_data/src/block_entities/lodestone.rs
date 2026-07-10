/// A lodestone tile entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Lodestone {
    /// The tracking ID of the lodestone.
    #[serde(rename = "trackingHandle")]
    pub tracking_handle: Option<i32>,
}
