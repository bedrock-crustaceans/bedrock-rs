use facet::Facet;

/// [`Slider`] represents a slider element,
/// allowing players to move it within its defined range to select a value.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(deny_unknown_fields)]
pub struct Slider {
    /// Represents the text displayed over the slider element.
    /// It may include Minecraft formatting codes.
    pub text: String,
    /// Specifies the lowest value that can be selected on the slider.
    pub min: f64,
    /// Specifies the highest value that can be selected on the slider
    pub max: f64,
    /// Determines the amount of space each slider step occupies.
    /// For instance, setting it to 1.0 allows the player to select only whole values.
    #[facet(rename = "step")]
    pub step_size: f64,
    /// Default value filled out for the slider.
    pub default: f64,
}
