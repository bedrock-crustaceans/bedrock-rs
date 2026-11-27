use facet::Facet;

/// [`Dropdown`] represents a dropdown which, when clicked,
/// opens a window with the options set in the Options field.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(deny_unknown_fields)]
pub struct Dropdown {
    /// Refers to the content shown above the dropdown element.
    /// It may include Minecraft formatting codes.
    pub text: String,
    /// Contains a list of choices that a player can select from.
    pub options: Vec<String>,
    /// Refers to the index in the Options slice designated as the default.
    /// The value at this index within the Options slice will be chosen.
    pub default: i32,
}
