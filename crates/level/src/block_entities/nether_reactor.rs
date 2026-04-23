/// A nether reactor tile entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetherReactor {
    /// Whether the nether reactor has completed its activation phase and has gone dark.
    pub has_finished: bool,
    /// If the reactor has been activated and has turned red.
    pub is_initialized: bool,
    /// Number of ticks the reactor has been active for. It finishes after 900 game ticks.
    pub progress: i16,
}
