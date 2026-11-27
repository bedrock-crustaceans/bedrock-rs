use facet::Facet;

/// [`Toggle`] represents a switch-like element.
/// Players can turn it either on or off, resulting in a value of `true` when on and `false` when off.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(deny_unknown_fields)]
pub struct Toggle {
    /// The text displayed over the toggle element, which may include Minecraft formatting codes.
    pub text: String,
    /// Specifies whether the toggle should be on or off by default.
    pub default: bool,
}
