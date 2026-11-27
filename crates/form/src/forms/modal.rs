use crate::elems::button::Button;
use facet::Facet;

/// [`ModalForm`] represents a modal form.
/// These forms consist of a body containing text and two [`Buttons`](Button) at the bottom,
/// usually labeled "Yes" (`gui.yes` for automatic translation) and "No" (`gui.no` for automatic translation)
/// While the button text can be customized,
/// unlike a [`SimpleForm`](crate::forms::SimpleForm), they cannot include images next to them.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(deny_unknown_fields)]
pub struct ModalForm {
    /// Refers to the title.
    pub title: String,
    /// Refers to the body.
    #[facet(rename = "content")]
    pub body: String,
    /// Refers to the first displayed button.
    pub button1: Button,
    /// Refers to the second displayed button.
    pub button2: Button,
}
