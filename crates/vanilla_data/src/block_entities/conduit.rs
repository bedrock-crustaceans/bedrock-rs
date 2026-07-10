/// A conduit tile entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Conduit {
    /// Whether the conduit is active.
    pub active: bool,
    /// The unique ID of the hostile mob the conduit is currently attacking. If there is no target,
    /// this is set to -1.
    pub target: i64,
}
