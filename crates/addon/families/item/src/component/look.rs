//! What the item shows: `display_name`, `glint`, `hover_text_color`, `icon`,
//! `interact_button`, `rarity`, `record`, `tags`.

use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::ConvertMode;
use facet::Facet;
use indexmap::IndexMap;

use super::ItemComponent;
use super::value::{ValueComponent, value_component};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics};
use bedrock_addon_core::schema::{self, Schema};

/// `minecraft:display_name`: the item's name, or a localization key (JSON `{"value": text}`).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DisplayName {
    /// The name or localization key (JSON `value`).
    #[facet(default)]
    pub value: String,
}

value_component! {
    /// `minecraft:glint`: the item shimmers like an enchanted one.
    Glint(bool);
}

value_component! {
    /// `minecraft:hover_text_color`: the colour code of the item's name.
    HoverTextColor(String);
}

value_component! {
    /// `minecraft:rarity`: `common`, `uncommon`, `rare` or `epic`.
    Rarity(String);
}

/// `minecraft:icon`: the item atlas keys of the item's icon.
///
/// JSON `{"textures": {"default": key, …}}`, or a bare key (the `default` texture). Keys
/// besides `default` (`dyed`, `icon_trim`, `bundle_open_front`, `bundle_open_back`, trim
/// palettes) are read by the engine where they apply.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Icon {
    /// The texture keys by role (JSON `textures`).
    #[facet(default)]
    pub textures: IndexMap<String, String>,
}

impl Icon {
    /// An icon with `key` as its `default` texture.
    pub fn new(key: impl Into<String>) -> Self {
        let mut textures = IndexMap::new();
        textures.insert("default".to_owned(), key.into());
        Self { textures }
    }

    /// The `default` texture key.
    pub fn default_texture(&self) -> Option<&str> {
        self.textures.get("default").map(String::as_str)
    }
}

impl ValueComponent for Icon {
    type Scalar = String;
    fn from_scalar(value: String) -> Self {
        Self::new(value)
    }
}

/// `minecraft:interact_button`: the touch-control interact button.
///
/// JSON `true`/`false`, or the button's text (which also turns it on); no object form.
/// Its row's parser reads it; the JSON value is facet's proxy, so the component is
/// written as the flag or the text.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(opaque, proxy = Value)]
pub struct InteractButton {
    /// The button is shown.
    pub requires_interact: bool,
    /// The button's text, when one is written.
    pub interact_text: Option<String>,
}

impl InteractButton {
    /// The component as JSON: its text, or the flag.
    pub(crate) fn to_value(&self) -> Value {
        match &self.interact_text {
            Some(text) => Value::from(text.as_str()),
            None => Value::from(self.requires_interact),
        }
    }

    /// Reads a bool or a string; anything else is `None`.
    pub(crate) fn from_value(value: &Value) -> Option<Self> {
        if let Some(flag) = value.as_bool() {
            return Some(Self {
                requires_interact: flag,
                interact_text: None,
            });
        }
        value.as_string().map(|text| Self {
            requires_interact: true,
            interact_text: Some(text.as_str().to_owned()),
        })
    }
}

impl TryFrom<Value> for InteractButton {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Self::from_value(&value).ok_or("expected a bool or a string")
    }
}

impl From<&InteractButton> for Value {
    fn from(button: &InteractButton) -> Self {
        button.to_value()
    }
}

/// The parser of `minecraft:interact_button`: a bool or a string; another type is a
/// type error, which refuses the item. The one context without `strict_components` is the
/// write-back probe of `write.rs`, which reads only the `Unrecognized*` codes, so the error
/// stands for it too.
pub(crate) fn parse_interact_button(
    slice: JsonSlice<'_>,
    _: ConvertMode,
    _: &LoadContext,
    d: &mut Diagnostics,
) -> Option<ItemComponent> {
    let value = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text).ok();
    let button = value.as_ref().and_then(InteractButton::from_value);
    if button.is_none() {
        d.push(Diagnostic::error(DiagnosticCode::InvalidType {
            text: None,
        }));
    }
    button.map(ItemComponent::InteractButton)
}

/// `minecraft:record`: the item is a music disc.
///
/// A member the document leaves out stays unset (and is not written); the game gives it the
/// value [`effective_duration`](Self::effective_duration) and its siblings state.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct Record {
    /// The sound played (JSON `sound_event`): a level sound event's name in a stable
    /// document, any sound name in a beta one. Unset: the stable struct plays the level sound
    /// event [`STABLE_DEFAULT_SOUND_EVENT`](Self::STABLE_DEFAULT_SOUND_EVENT), the beta struct
    /// the empty name.
    pub sound_event: Option<SoundEventName>,
    /// The length in seconds (JSON `duration`; unset: 0.0).
    pub duration: Option<f32>,
    /// The comparator signal of a jukebox playing it, 1–13 (JSON `comparator_signal`; unset:
    /// 1).
    pub comparator_signal: Option<i32>,
}

impl Record {
    /// The `LevelSoundEvent` value the stable struct plays
    /// when `sound_event` is unset; the name of that value is not known, so it stays a number.
    pub const STABLE_DEFAULT_SOUND_EVENT: u32 = 615;

    /// The comparator signal the game uses: the written one, else 1 (in the stable and in
    /// the beta struct alike).
    pub fn effective_comparator_signal(&self) -> i32 {
        self.comparator_signal.unwrap_or(1)
    }

    /// The duration the game uses: the written one, else 0.0.
    pub fn effective_duration(&self) -> f32 {
        self.duration.unwrap_or(0.0)
    }

    /// The sound name the beta struct plays: the written one, else the empty name. A stable
    /// document's unset sound is
    /// [`STABLE_DEFAULT_SOUND_EVENT`](Self::STABLE_DEFAULT_SOUND_EVENT).
    pub fn effective_beta_sound_event(&self) -> &str {
        self.sound_event.as_ref().map_or("", |name| name.as_str())
    }
}

/// `minecraft:tags`: the item tags.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Tags {
    /// The tags (JSON `tags`).
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{InteractButton, ItemComponent};
    use bedrock_addon_core::json::Value;

    /// The write dispatch writes the button through its proxy: the flag, or the text.
    #[test]
    fn interact_button_writes_its_flag_or_its_text() {
        for button in [
            InteractButton {
                requires_interact: false,
                interact_text: None,
            },
            InteractButton {
                requires_interact: true,
                interact_text: Some("Use".to_owned()),
            },
        ] {
            let written = ItemComponent::InteractButton(button.clone()).write_component();
            assert_eq!(written.ok(), Some(button.to_value()));
        }
        assert_eq!(
            ItemComponent::InteractButton(InteractButton::default())
                .write_component()
                .ok(),
            Some(Value::from(false))
        );
    }
}
