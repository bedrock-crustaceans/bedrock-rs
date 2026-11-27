use crate::elems::Element;
use facet::Facet;

/// [`CustomForm`] represents a form that can be sent to a player,
/// containing fields that the player may fill out.
/// All possible fields are in the [`Element`] enum.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(deny_unknown_fields)]
pub struct CustomForm {
    /// Refers to the title.
    pub title: String,
    /// All elements in the [`CustomForm`](CustomForm). Sequence is maintained.
    #[facet(rename = "content")]
    pub elements: Vec<Element>,
}
