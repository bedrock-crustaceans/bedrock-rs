use facet::Facet;

/// A redstone comparator.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Comparator {
    /// The strength of the signal output of this redstone comparator.
    #[facet(rename = "OutputSignal")]
    pub output_signal: i32,
}
