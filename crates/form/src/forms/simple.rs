use crate::elems::button::Button;
use facet::Facet;

/// [`SimpleForm`] represents a form consisting of a title,
/// body, and a set of buttons beneath the body.
/// These [`Buttons`](Button) can optionally include images alongside them.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(deny_unknown_fields)]
pub struct SimpleForm {
    /// Refers to the title.
    pub title: String,
    /// Refers to the body.
    #[facet(rename = "content")]
    pub body: String,
    /// Refers to all available buttons. Sequence is maintained.
    pub buttons: Vec<Button>,
}
