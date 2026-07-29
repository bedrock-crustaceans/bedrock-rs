/// A redstone comparator.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Comparator {
    /// The strength of the signal output of this redstone comparator.
    #[serde(rename = "OutputSignal")]
    pub output_signal: i32,
}
