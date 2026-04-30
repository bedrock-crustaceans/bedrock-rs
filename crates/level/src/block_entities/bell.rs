#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Bell {
    /// The direction the bell is facing.
    pub direction: i32,
    /// Whether the bell is currently ringing.
    pub ringing: bool,
    /// Unknown.
    pub ticks: i32,
}
