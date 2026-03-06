use serde::{Deserialize, Serialize};

/// [`Button`] signifies a button incorporated into a [`SimpleForm`](crate::forms::simple::SimpleForm)
/// or [`ModalForm`](crate::forms::modal::ModalForm) form.
/// This button features text and may optionally include an image,
/// which can be sourced from a website or the game's local assets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Button {
    /// Contains the text shown on the button.
    /// It can include Minecraft formatting codes and may contain newlines.
    pub text: String,
    /// Holds an optional image for the button.
    pub image: Option<ButtonImage>,
}

/// An image for a [`Button`].
/// Can either be an url or a path to a resource location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum ButtonImage {
    /// Refers to an image stored at the given resource location in the local game/texture packs.
    Path(String),
    /// Refers to an image stored at the given url.
    Url(String),
}
