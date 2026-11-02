#![allow(clippy::disallowed_methods, clippy::disallowed_types, reason = "outside the add-on toolkit: facet-json is called directly")]
use crate::elems::dropdown::Dropdown;
use crate::elems::input::Input;
use crate::elems::label::Label;
use crate::elems::slider::Slider;
use crate::elems::step_slider::StepSlider;
use crate::elems::toggle::Toggle;
use facet::Facet;

pub mod button;
pub mod dropdown;
pub mod input;
pub mod label;
pub mod slider;
pub mod step_slider;
pub mod toggle;

/// An enum of all possible [`Elements`](Element) for a [`CustomForm`](crate::forms::custom::CustomForm).
#[derive(Debug, Clone, PartialEq, Facet)]
#[repr(u8)]
#[facet(tag = "type")]
#[facet(rename_all = "snake_case")]
#[facet(deny_unknown_fields)]
pub enum Element {
    Dropdown(Dropdown),
    Input(Input),
    Label(Label),
    Slider(Slider),
    StepSlider(StepSlider),
    Toggle(Toggle),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elems::button::ButtonImage;
    use crate::elems::toggle::Toggle as ToggleElem;

    #[test]
    fn deserializes_internally_tagged_toggle_element() {
        let json = r#"{"type":"toggle","text":"A toggle","default":true}"#;
        let element: Element = facet_json::from_str(json).unwrap();
        assert_eq!(
            element,
            Element::Toggle(ToggleElem {
                text: "A toggle".to_string(),
                default: true,
            })
        );
    }

    #[test]
    fn deserializes_adjacently_tagged_button_image() {
        let json = r#"{"type":"url","data":"x"}"#;
        let image: ButtonImage = facet_json::from_str(json).unwrap();
        assert_eq!(image, ButtonImage::Url("x".to_string()));
    }

    #[test]
    fn button_deserializes_with_missing_image_field() {
        use crate::elems::button::Button;

        let json = r#"{"text":"Click me"}"#;
        let button: Button = facet_json::from_str(json).unwrap();
        assert_eq!(
            button,
            Button {
                text: "Click me".to_string(),
                image: None,
            }
        );
    }
}
