use facet::Facet;

/// [`Input`] represents a text input field element
/// where players can enter text of any length without restrictions.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(deny_unknown_fields)]
pub struct Input {
    /// Refers to the content displayed over the input element,
    /// which may include Minecraft formatting codes.
    pub text: String,
    /// Represents the pre-filled value in the input field.
    /// The player can remove this value and enter their own text,
    /// which may include Minecraft formatting codes.
    pub default: String,
    /// Is shown in the input box when no player-entered text is present.
    /// This text can include Minecraft formatting codes.
    pub placeholder: String,
}
