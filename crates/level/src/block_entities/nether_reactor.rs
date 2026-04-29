use crate::deserialize_bool;

/// A nether reactor tile entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetherReactor {
    /// Whether the nether reactor has completed its activation phase and has gone dark.
    #[serde(deserialize_with = "deserialize_bool")]
    pub has_finished: bool,
    /// If the reactor has been activated and has turned red.
    #[serde(deserialize_with = "deserialize_bool")]
    pub is_initialized: bool,
    /// Number of ticks the reactor has been active for. It finishes after 900 game ticks.
    pub progress: i16,
}
