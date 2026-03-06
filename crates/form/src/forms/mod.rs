use crate::forms::custom::CustomForm;
use crate::forms::modal::ModalForm;
use crate::forms::simple::SimpleForm;
use serde::{Deserialize, Serialize};

pub mod custom;
pub mod modal;
pub mod simple;

/// An enum of all possible Forms, including [`Custom`](CustomForm), [`Modal`](ModalForm) and [`Simple`](SimpleForm).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum Form {
    /// [`CustomForm`] represents a form that can be sent to a player,
    /// containing fields that the player may fill out.
    /// All possible fields are in the [`Element`] enum.
    #[serde(rename = "custom_form")]
    Custom(CustomForm),
    /// [`ModalForm`] represents a modal form.
    /// These forms consist of a body containing text and two [`Buttons`](Button) at the bottom,
    /// usually labeled "Yes" (`gui.yes` for automatic translation) and "No" (`gui.no` for automatic translation)
    /// While the button text can be customized,
    /// unlike a [`SimpleForm`](crate::forms::SimpleForm), they cannot include images next to them.
    #[serde(rename = "modal")]
    Modal(ModalForm),
    /// [`SimpleForm`] represents a form consisting of a title,
    /// body, and a set of buttons beneath the body.
    /// These [`Buttons`](Button) can optionally include images alongside them.
    #[serde(rename = "form")]
    Simple(SimpleForm),
}

impl TryFrom<String> for Form {
    type Error = serde_json::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for Form {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value)
    }
}

impl TryFrom<Form> for String {
    type Error = serde_json::Error;

    fn try_from(value: Form) -> Result<Self, Self::Error> {
        serde_json::to_string(&value)
    }
}

#[cfg(test)]
mod test {
    use crate::elems::Element;
    use crate::elems::button::{Button, ButtonImage};
    use crate::elems::dropdown::Dropdown;
    use crate::elems::toggle::Toggle;
    use crate::forms::Form;
    use crate::forms::custom::CustomForm;
    use crate::forms::modal::ModalForm;
    use crate::forms::simple::SimpleForm;

    #[test]
    fn custom_form() {
        let json = r#"
        {
            "type": "custom_form",
            "title": "Title of this custom form",
            "content": [
                {
                    "type": "toggle",
                    "text": "A toggle",
                    "default": true
                },
                {
                    "type": "dropdown",
                    "text": "Dropdown, choose ur character",
                    "default": 2,
                    "options": [":3", ":p", ":<", ":D", ":/"]
                }
            ]
        }
        "#;

        let form = Form::Custom(CustomForm {
            title: "Title of this custom form".to_string(),
            elements: vec![
                Element::Toggle(Toggle {
                    text: "A toggle".to_string(),
                    default: true,
                }),
                Element::Dropdown(Dropdown {
                    text: "Dropdown, choose ur character".to_string(),
                    options: vec![
                        ":3".to_string(),
                        ":p".to_string(),
                        ":<".to_string(),
                        ":D".to_string(),
                        ":/".to_string(),
                    ],
                    default: 2,
                }),
            ],
        });

        assert_eq!(serde_json::from_str::<Form>(json).unwrap(), form);
    }

    #[test]
    fn modal_form() {
        let json = r#"
        {
            "type": "modal",
            "title": "Title of this modal form",
            "content": "Body of this modal form",
            "button1": {
                "text": "Ferris is happy!",
                "image": {
                    "type": "url",
                    "data": "https://rustacean.net/assets/rustacean-flat-happy.png"
                }
            },
            "button2": {
                "text": "Ferris is gesturing... but where?"
            }
        }
        "#;

        let form = Form::Modal(ModalForm {
            title: "Title of this modal form".to_string(),
            body: "Body of this modal form".to_string(),
            button1: Button {
                text: "Ferris is happy!".to_string(),
                image: Some(ButtonImage::Url(
                    "https://rustacean.net/assets/rustacean-flat-happy.png".to_string(),
                )),
            },
            button2: Button {
                text: "Ferris is gesturing... but where?".to_string(),
                image: None,
            },
        });

        assert_eq!(serde_json::from_str::<Form>(json).unwrap(), form);
    }

    #[test]
    fn simple_form() {
        let json = r#"
        {
            "type": "form",
            "title": "Title of this simple form",
            "content": "Body of this simple form",
            "buttons": [
                {
                    "text": "Ferris is happy!",
                    "image": {
                        "type": "url",
                        "data": "https://rustacean.net/assets/rustacean-flat-happy.png"
                    }
                },
                {
                    "text": "Ferris is extra cute!!!",
                    "image": {
                        "type": "path",
                        "data": "some/special/path"
                    }
                },
                {
                    "text": "Ferris is gone..."
                }
            ]
        }
        "#;

        let form = Form::Simple(SimpleForm {
            title: "Title of this simple form".to_string(),
            body: "Body of this simple form".to_string(),
            buttons: vec![
                Button {
                    text: "Ferris is happy!".to_string(),
                    image: Some(ButtonImage::Url(
                        "https://rustacean.net/assets/rustacean-flat-happy.png".to_string(),
                    )),
                },
                Button {
                    text: "Ferris is extra cute!!!".to_string(),
                    image: Some(ButtonImage::Path("some/special/path".to_string())),
                },
                Button {
                    text: "Ferris is gone...".to_string(),
                    image: None,
                },
            ],
        });

        assert_eq!(serde_json::from_str::<Form>(json).unwrap(), form);
    }
}
