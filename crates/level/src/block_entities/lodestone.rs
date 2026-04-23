/// A lodestone tile entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Lodestone {
    /// The tracking ID of the lodestone.
    #[serde(rename = "trackingHandle")]
    pub tracking_handle: Option<i32>,
}
