use facet::Facet;

/// [`StepSlider`] is an element that functions as a slider with multiple selectable options.
/// It effectively merges the characteristics of a dropdown and a slider,
/// appearing like a slider while incorporating features typical of a dropdown.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(deny_unknown_fields)]
pub struct StepSlider {
    /// Represents the text displayed over the step slider element.
    /// It may include Minecraft formatting codes.
    pub text: String,
    /// Contains a list of choices that a player can select from.
    #[facet(rename = "steps")]
    pub options: Vec<String>,
    /// Refers to the index in the Options slice designated as the default.
    /// The value at this index within the Options slice will be chosen.
    pub default: i32,
}
