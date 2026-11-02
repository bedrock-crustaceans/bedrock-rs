//! Data-driven UI form styles (`ddui/styling/`, payload `minecraft:ui-style`): the third kind
//! of the `DataDrivenUi` group, new in engine 1.26.60, read at `"format_version": "beta"` only.

use facet_value::{VArray, VObject};

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{EncodeError, Unrepresentable, Value};

/// A form style's identifier (`description.identifier`): core's `id::kind::UiStyle`.
pub use bedrock_addon_core::id::UiStyleId;

/// A data-driven UI form style (`minecraft:ui-style`, "`DataDrivenUI` UI Style"): the styling of
/// form components, by component.
///
/// JSON: `{"format_version": "beta", "minecraft:ui-style": {"description": {"identifier":
/// "demo:style"}, "components": {"minecraft:form_label": {"color": [255, 255, 255, 255]}}}}`.
/// The kind is beta-only: `"beta"` is the one version with a parser, accepted from every pack
/// without the Beta APIs toggle. Every member not listed is an error and the file is skipped.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:ui-style")]
#[doc(alias = "Style")]
pub struct UiStyle {
    /// JSON `description` ("Identifies this form style definition.").
    pub description: StyleDescription,
    /// JSON `components` ("Defines styling for form components."): at least one.
    pub components: StyleComponents,
}

/// The `description` of a style.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleDescription {
    /// The style's name (JSON `identifier`): a non-empty namespace, `:` and a non-empty name,
    /// under the namespace rule of roots and compositions.
    pub identifier: UiStyleId,
}

/// The `components` of a style: one entry per form component type, at
/// least one written.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StyleComponents {
    /// `minecraft:form_panel`: "Defines styling for panel components."
    pub form_panel: Option<FormPanelStyle>,
    /// `minecraft:form_label`: "Defines styling for label components."
    pub form_label: Option<FormLabelStyle>,
    /// `minecraft:form_textfield`: "Defines styling for text field components."
    pub form_textfield: Option<FormTextFieldStyle>,
}

impl StyleComponents {
    /// The JSON keys of the three components, in bind order.
    pub const KEYS: [&'static str; 3] = [
        "minecraft:form_panel",
        "minecraft:form_label",
        "minecraft:form_textfield",
    ];

    /// How many components are written.
    #[must_use]
    pub fn len(&self) -> usize {
        usize::from(self.form_panel.is_some())
            + usize::from(self.form_label.is_some())
            + usize::from(self.form_textfield.is_some())
    }

    /// Whether no component is written (the client refuses such a style).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// `minecraft:form_panel`: "Defines the visual styling for panel components." Every member
/// may be left out.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[doc(alias = "FormPanel")]
pub struct FormPanelStyle {
    /// JSON `padding`: "Sets the panel padding as [top, right] or [top, right, bottom,
    /// left]."
    pub padding: Option<BoxSize>,
    /// JSON `image`: "Sets the resource pack image path used for the panel background."
    pub image: Option<String>,
    /// JSON `image_repeat_top_bottom`: "Sets how the background image repeats along its top
    /// and bottom edges."
    pub image_repeat_top_bottom: Option<BorderImageRepeat>,
    /// JSON `image_repeat_right_left`: "Sets how the background image repeats along its right
    /// and left edges."
    pub image_repeat_right_left: Option<BorderImageRepeat>,
}

/// `minecraft:form_label`: "Defines the visual styling for panel text components." Both
/// members may be left out; the client then holds their defaults
/// ([`color`](Self::color), [`shadow`](Self::shadow)), and the write-back leaves them out.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[doc(alias = "FormLabel")]
pub struct FormLabelStyle {
    /// JSON `color`: "Sets the text color."
    pub color: Option<StyleColor>,
    /// JSON `shadow`: "Enables or disables the text shadow."
    pub shadow: Option<bool>,
}

impl FormLabelStyle {
    /// The colour of a label that writes none: opaque black, RGBA (0, 0, 0, 255).
    pub const DEFAULT_COLOR: StyleColor = StyleColor::Rgba([0, 0, 0, 255]);
    /// The shadow of a label that writes none: on.
    pub const DEFAULT_SHADOW: bool = true;

    /// The text colour the client uses: the written one, else [`Self::DEFAULT_COLOR`].
    #[must_use]
    pub fn color(&self) -> StyleColor {
        self.color.clone().unwrap_or(Self::DEFAULT_COLOR)
    }

    /// Whether the text has a shadow: the written value, else [`Self::DEFAULT_SHADOW`].
    #[must_use]
    pub fn shadow(&self) -> bool {
        self.shadow.unwrap_or(Self::DEFAULT_SHADOW)
    }
}

/// `minecraft:form_textfield`: "Defines the visual styling for text field components." The
/// four colours are required, the five images may be left out.
#[derive(Clone, Debug, PartialEq, Eq)]
#[doc(alias = "FormTextField")]
pub struct FormTextFieldStyle {
    /// JSON `text_color`: "Sets the entered text color."
    pub text_color: StyleColor,
    /// JSON `caret_color`: "Sets the text input caret color."
    pub caret_color: StyleColor,
    /// JSON `selection_background_color`: "Sets the background color of selected text."
    pub selection_background_color: StyleColor,
    /// JSON `text_color_disabled`: "Sets the entered text color when the text field is
    /// disabled."
    pub text_color_disabled: StyleColor,
    /// JSON `default_image`: "Sets the default image for the text field."
    pub default_image: Option<String>,
    /// JSON `hovered_image`: "Sets the image for the text field when hovered."
    pub hovered_image: Option<String>,
    /// JSON `disabled_image`: "Sets the image for the text field when disabled."
    pub disabled_image: Option<String>,
    /// JSON `focused_image`: "Sets the image for the text field when focused."
    pub focused_image: Option<String>,
    /// JSON `panel_image`: "Sets the image for the panel behind the text field."
    pub panel_image: Option<String>,
}

impl FormTextFieldStyle {
    /// A text field style with the four colours and no image.
    #[must_use]
    pub fn new(
        text_color: StyleColor,
        caret_color: StyleColor,
        selection_background_color: StyleColor,
        text_color_disabled: StyleColor,
    ) -> Self {
        Self {
            text_color,
            caret_color,
            selection_background_color,
            text_color_disabled,
            default_image: None,
            hovered_image: None,
            disabled_image: None,
            focused_image: None,
            panel_image: None,
        }
    }
}

/// A box size in texels: an array of two or four integers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[doc(alias = "BoxSize2D")]
#[doc(alias = "BoxSize4D")]
pub enum BoxSize {
    /// `[top and bottom, right and left]`: "A two-value box size in texels using [top and
    /// bottom, right and left]."
    Two([i32; 2]),
    /// `[top, right, bottom, left]`: "A four-value box size in texels using [top, right,
    /// bottom, left]."
    Four([i32; 4]),
}

impl BoxSize {
    /// The four sides, `[top, right, bottom, left]`: a two-value size repeats its pair.
    #[must_use]
    pub fn sides(self) -> [i32; 4] {
        match self {
            Self::Two([vertical, horizontal]) => [vertical, horizontal, vertical, horizontal],
            Self::Four(sides) => sides,
        }
    }
}

/// How a border image repeats along an edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[doc(alias = "BorderImageRepeatType")]
pub enum BorderImageRepeat {
    /// `"stretch"` (0).
    Stretch,
    /// `"repeat"` (1).
    Repeat,
}

impl BorderImageRepeat {
    /// The JSON name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Stretch => "stretch",
            Self::Repeat => "repeat",
        }
    }

    /// The value a JSON name selects, compared with case.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "stretch" => Some(Self::Stretch),
            "repeat" => Some(Self::Repeat),
            _ => None,
        }
    }
}

/// A colour of a style: RGBA (four integers 0–255, or `#` and eight hex digits,
/// ARGB) or RGB (three integers, or `#` and six hex digits). Held in the form
/// written; a hex text keeps its case.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[doc(alias = "Color255RGBA")]
#[doc(alias = "Color255RGB")]
pub enum StyleColor {
    /// `[r, g, b, a]`.
    Rgba([u8; 4]),
    /// `[r, g, b]`.
    Rgb([u8; 3]),
    /// `"#AARRGGBB"`, as written.
    HexArgb(String),
    /// `"#RRGGBB"`, as written.
    HexRgb(String),
}

impl StyleColor {
    /// The colour as `[r, g, b, a]` (an RGB colour is opaque); `None` for a hex text that is
    /// not `#` and eight (or six) hex digits.
    #[must_use]
    pub fn rgba(&self) -> Option<[u8; 4]> {
        match self {
            Self::Rgba(rgba) => Some(*rgba),
            Self::Rgb([r, g, b]) => Some([*r, *g, *b, u8::MAX]),
            Self::HexArgb(text) => {
                let [a, r, g, b] = hex_bytes::<4>(text)?;
                Some([r, g, b, a])
            }
            Self::HexRgb(text) => {
                let [r, g, b] = hex_bytes::<3>(text)?;
                Some([r, g, b, u8::MAX])
            }
        }
    }
}

/// The `N` bytes of `#` followed by `2 × N` hex digits (`^#[a-fA-F0-9]{2N}$`).
pub(crate) fn hex_bytes<const N: usize>(text: &str) -> Option<[u8; N]> {
    let digits = text.strip_prefix('#')?;
    if digits.len() != 2 * N || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut bytes = [0; N];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(digits.get(2 * index..2 * index + 2)?, 16).ok()?;
    }
    Some(bytes)
}

impl UiStyle {
    /// A style named `identifier` with `components`.
    #[must_use]
    pub fn new(identifier: UiStyleId, components: StyleComponents) -> Self {
        Self {
            description: StyleDescription { identifier },
            components,
        }
    }
}

// ---- writing --------------------------------------------------------------------------------

/// The refusal of the member at `members` below the payload.
fn refusal(members: &[&str]) -> EncodeError {
    let mut at = JsonPath::new();
    for member in members {
        at.push(member);
    }
    EncodeError::NotRepresentable {
        at,
        what: Unrepresentable::Unsupported,
    }
}

/// The JSON of a colour; `None` for a hex text the reader would refuse.
fn color_json(color: &StyleColor) -> Option<Value> {
    let numbers = |values: &[u8]| {
        let mut array = VArray::new();
        for value in values {
            array.push(Value::from(i64::from(*value)));
        }
        Value::from(array)
    };
    match color {
        StyleColor::Rgba(rgba) => Some(numbers(rgba)),
        StyleColor::Rgb(rgb) => Some(numbers(rgb)),
        StyleColor::HexArgb(text) => hex_bytes::<4>(text).map(|_| Value::from(text.as_str())),
        StyleColor::HexRgb(text) => hex_bytes::<3>(text).map(|_| Value::from(text.as_str())),
    }
}

/// Inserts the colour `color` as `key` of `out`, at `members` + `key` for a refusal.
fn put_color(
    out: &mut VObject,
    key: &str,
    color: &StyleColor,
    members: &[&str],
) -> Result<(), EncodeError> {
    let json = color_json(color).ok_or_else(|| refusal(&[members, &[key]].concat()))?;
    out.insert(key, json);
    Ok(())
}

/// Inserts the string `value` as `key` of `out` when it is written.
fn put_text(out: &mut VObject, key: &str, value: Option<&String>) {
    if let Some(value) = value {
        out.insert(key, Value::from(value.as_str()));
    }
}

/// The payload of `style`: what the reader takes back equal, nothing it refuses.
pub(crate) fn payload(style: &UiStyle) -> Result<Value, EncodeError> {
    let identifier = style.description.identifier.as_str();
    if !super::read::namespaced(identifier) {
        return Err(refusal(&["description", "identifier"]));
    }
    let mut description = VObject::new();
    description.insert("identifier", Value::from(identifier));
    if style.components.is_empty() {
        return Err(refusal(&["components"]));
    }
    let mut components = VObject::new();
    let [panel_key, label_key, textfield_key] = StyleComponents::KEYS;
    if let Some(panel) = &style.components.form_panel {
        let mut out = VObject::new();
        if let Some(padding) = panel.padding {
            let mut array = VArray::new();
            let values: &[i32] = match &padding {
                BoxSize::Two(values) => values,
                BoxSize::Four(values) => values,
            };
            for value in values {
                array.push(Value::from(i64::from(*value)));
            }
            out.insert("padding", Value::from(array));
        }
        put_text(&mut out, "image", panel.image.as_ref());
        for (key, repeat) in [
            ("image_repeat_top_bottom", panel.image_repeat_top_bottom),
            ("image_repeat_right_left", panel.image_repeat_right_left),
        ] {
            if let Some(repeat) = repeat {
                out.insert(key, Value::from(repeat.name()));
            }
        }
        components.insert(panel_key, Value::from(out));
    }
    if let Some(label) = &style.components.form_label {
        let mut out = VObject::new();
        if let Some(color) = &label.color {
            put_color(&mut out, "color", color, &["components", label_key])?;
        }
        if let Some(shadow) = label.shadow {
            out.insert("shadow", Value::from(shadow));
        }
        components.insert(label_key, Value::from(out));
    }
    if let Some(field) = &style.components.form_textfield {
        let mut out = VObject::new();
        let at = ["components", textfield_key];
        for (key, color) in [
            ("text_color", &field.text_color),
            ("caret_color", &field.caret_color),
            (
                "selection_background_color",
                &field.selection_background_color,
            ),
            ("text_color_disabled", &field.text_color_disabled),
        ] {
            put_color(&mut out, key, color, &at)?;
        }
        for (key, image) in [
            ("default_image", &field.default_image),
            ("hovered_image", &field.hovered_image),
            ("disabled_image", &field.disabled_image),
            ("focused_image", &field.focused_image),
            ("panel_image", &field.panel_image),
        ] {
            put_text(&mut out, key, image.as_ref());
        }
        components.insert(textfield_key, Value::from(out));
    }
    let mut payload = VObject::new();
    payload.insert("description", Value::from(description));
    payload.insert("components", Value::from(components));
    Ok(Value::from(payload))
}
