//! A resolved definition and the typed views of its properties: what the client's getters
//! read from the flattened JSON, with the defaults they pass.

use std::collections::HashSet;

use indexmap::IndexMap;

use super::chain::Inherited;
use super::expr;
use super::globals::ENGINE_VARIABLES;
use super::value::{member, number, sorted_members, string_or_empty};
use super::{DefKey, DefRef, UiDefinitions};
use bedrock_addon_core::json::Value;

/// Declares an enum of JSON strings with `from_name` and `name`.
macro_rules! named {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident = $text:literal),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        $vis enum $name {
            $($(#[$variant_meta])* $variant),*
        }

        impl $name {
            /// Every value with its JSON string, in the game's order.
            pub const ALL: &'static [(Self, &'static str)] = &[$((Self::$variant, $text)),*];

            /// The value a JSON string names.
            pub fn from_name(name: &str) -> Option<Self> {
                Self::ALL.iter().find(|(_, text)| *text == name).map(|(value, _)| *value)
            }

            /// The JSON string.
            pub fn name(self) -> &'static str {
                Self::ALL.iter().find(|(value, _)| *value == self).map_or("", |(_, text)| text)
            }
        }
    };
}

named! {
    /// The `type` of a control: 24 entries, numbered 0 to 23 in this order.
    pub enum ControlType {
        /// `button`
        Button = "button",
        /// `custom`
        Custom = "custom",
        /// `collection_panel`
        CollectionPanel = "collection_panel",
        /// `dropdown`
        Dropdown = "dropdown",
        /// `edit_box`
        EditBox = "edit_box",
        /// `factory`
        Factory = "factory",
        /// `grid`
        Grid = "grid",
        /// `image`
        Image = "image",
        /// `input_panel`
        InputPanel = "input_panel",
        /// `label`
        Label = "label",
        /// `panel`
        Panel = "panel",
        /// `screen`
        Screen = "screen",
        /// `scrollbar_box`
        ScrollbarBox = "scrollbar_box",
        /// `scroll_track`
        ScrollTrack = "scroll_track",
        /// `scroll_view`
        ScrollView = "scroll_view",
        /// `selection_wheel`
        SelectionWheel = "selection_wheel",
        /// `slider`
        Slider = "slider",
        /// `slider_box`
        SliderBox = "slider_box",
        /// `stack_panel`
        StackPanel = "stack_panel",
        /// `toggle`
        Toggle = "toggle",
        /// `image_cycler`
        ImageCycler = "image_cycler",
        /// `label_cycler`
        LabelCycler = "label_cycler",
        /// `grid_page_indicator`
        GridPageIndicator = "grid_page_indicator",
        /// `tooltip_trigger`
        TooltipTrigger = "tooltip_trigger",
    }
}

named! {
    /// The `anim_type` of an animation.
    // Numbered 0 to 8; anything else is 9, no animation.
    pub enum AnimationType {
        /// `alpha`
        Alpha = "alpha",
        /// `clip`
        Clip = "clip",
        /// `color`
        Color = "color",
        /// `flip_book`
        FlipBook = "flip_book",
        /// `aseprite_flip_book`
        AsepriteFlipBook = "aseprite_flip_book",
        /// `offset`
        Offset = "offset",
        /// `size`
        Size = "size",
        /// `uv`
        Uv = "uv",
        /// `wait`
        Wait = "wait",
    }
}

named! {
    /// An orientation.
    pub enum Orientation {
        /// `horizontal`
        Horizontal = "horizontal",
        /// `vertical`
        Vertical = "vertical",
        /// `none`
        None = "none",
    }
}

named! {
    /// The size of a label's font.
    pub enum FontSize {
        /// `small`
        Small = "small",
        /// `normal`
        Normal = "normal",
        /// `large`
        Large = "large",
        /// `extra_large`
        ExtraLarge = "extra_large",
    }
}

named! {
    /// The alignment of a label's text.
    pub enum TextAlignment {
        /// `left`
        Left = "left",
        /// `right`
        Right = "right",
        /// `center`
        Center = "center",
    }
}

/// The axes an image tiles along.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TileDirection {
    /// Not tiled.
    None,
    /// Along x.
    X,
    /// Along y.
    Y,
    /// Along both axes.
    Both,
}

named! {
    /// The kind of a data binding.
    // Numbered 0 to 4.
    pub enum BindingType {
        /// `none`: the binding is skipped.
        None = "none",
        /// `global`
        Global = "global",
        /// `collection`
        Collection = "collection",
        /// `collection_details`
        CollectionDetails = "collection_details",
        /// `view`
        View = "view",
    }
}

named! {
    /// When a data binding is evaluated.
    // Numbered 0 to 5.
    pub enum BindingCondition {
        /// `none`
        None = "none",
        /// `once`
        Once = "once",
        /// `always`
        Always = "always",
        /// `always_when_visible`
        AlwaysWhenVisible = "always_when_visible",
        /// `visible`
        Visible = "visible",
        /// `visibility_changed`
        VisibilityChanged = "visibility_changed",
    }
}

named! {
    /// The `mapping_type` of a button mapping.
    pub enum ButtonMappingType {
        /// `global`
        Global = "global",
        /// `double_pressed`
        DoublePressed = "double_pressed",
        /// `pressed`
        Pressed = "pressed",
        /// `focused`
        Focused = "focused",
    }
}

named! {
    /// The `scope` of a button mapping.
    pub enum MappingScope {
        /// `controller`, and every string the game does not compare.
        Controller = "controller",
        /// `view`
        View = "view",
        /// `global`
        Global = "global",
    }
}

named! {
    /// The `input_mode_condition` of a button mapping.
    pub enum InputModeCondition {
        /// No condition: every string the game does not compare.
        None = "none",
        /// `not_gamepad`
        NotGamepad = "not_gamepad",
        /// `gamepad`
        Gamepad = "gamepad",
    }
}

/// What a definition is: nothing but its members tells a control from an animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ControlKind {
    /// It has a `type` (its own or inherited) the game knows.
    Control(ControlType),
    /// It has an `anim_type` the game knows.
    Animation(AnimationType),
    /// Neither: a base other definitions inherit from, or a definition the game cannot build.
    Unknown,
}

/// A resolved definition of a UI file: a control or an animation, with every member of its
/// `@base` chain it does not define itself inherited.
///
/// The game copies the inherited members into each definition. Here a definition holds what
/// it writes ([`raw`](Self::raw)) and shares what its base hands on with every other
/// definition of that base, so a table is as large as its files: [`get`](Self::get) reads one
/// member through the chain and [`body`](Self::body) builds the flattened JSON when asked.
///
/// The typed methods read the flattened definition the way the client's getters do, with
/// the defaults they pass.
///
/// Two definitions are equal when they are written the same and inherit from the definition
/// of one namespace and name; `Debug` names that definition and does not print it.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
#[doc(alias = "UIResolvedDef")]
pub struct Control {
    /// The key as its file writes it (`name@namespace.base`).
    pub key: DefKey,
    /// The namespace of its file (`_root` when the file names none).
    pub namespace: String,
    /// The listed file it comes from.
    pub source_file: String,
    /// The definition as merged from the packs: the members it writes itself.
    pub raw: Value,
    /// What its base hands on.
    pub(crate) inherited: Inherited,
}

/// The variables a property is resolved against: `$name` to value.
///
/// The game supplies the platform variables ([`ENGINE_VARIABLES`]) when it creates a screen;
/// the global variables of the stack and the frames of the enclosing controls come on top.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Variables {
    /// The values, by name with the `$`.
    pub values: IndexMap<String, Value>,
}

impl Variables {
    /// The platform variables: every one of [`ENGINE_VARIABLES`] false, except the ones in
    /// `enabled`, which are true.
    pub fn platform(enabled: &[&str]) -> Self {
        let values = ENGINE_VARIABLES
            .iter()
            .map(|name| ((*name).to_owned(), Value::from(enabled.contains(name))))
            .collect();
        Self { values }
    }

    /// The value of `name` (with its `$`).
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    /// Sets `name` (with its `$`).
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<Value>) {
        self.values.insert(name.into(), value.into());
    }

    /// `value` with a `$variable` string replaced by the variable's value, and a string
    /// starting with `(` evaluated as an expression.
    pub fn resolve(&self, value: &Value) -> Value {
        match value.as_string().map(facet_value::VString::as_str) {
            Some(text) if text.starts_with('$') => self.get(text).cloned().unwrap_or(Value::NULL),
            Some(text) if text.starts_with('(') => {
                expr::evaluate(text, &|name| self.get(name).cloned()).into_value()
            }
            _ => value.clone(),
        }
    }

    /// Whether `value` holds as a `requires` condition: a number other than 0, a non-empty
    /// string, or any other non-empty value.
    pub fn is_truthy(&self, value: &Value) -> bool {
        expr::Operand::from_value(&self.resolve(value)).truthy()
    }
}

/// One entry of a control's `bindings`, as the game registers it.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Binding {
    /// JSON `binding_type`; absent, not a string or unknown is `global`.
    pub binding_type: BindingType,
    /// JSON `binding_condition`; absent, not a string or unknown is `none`.
    pub condition: BindingCondition,
    /// JSON `binding_name`.
    pub name: Option<String>,
    /// JSON `binding_name_override`.
    pub name_override: Option<String>,
    /// JSON `binding_collection_name`.
    pub collection_name: Option<String>,
    /// JSON `source_control_name`.
    pub source_control_name: Option<String>,
    /// JSON `source_property_name`.
    pub source_property_name: Option<String>,
    /// JSON `target_property_name`.
    pub target_property_name: Option<String>,
}

/// One entry of a control's `button_mappings`, as the game registers it.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ButtonMapping {
    /// JSON `from_button_id`.
    pub from_button_id: String,
    /// JSON `to_button_id`.
    pub to_button_id: String,
    /// JSON `mapping_type`; anything the game does not know, an absent one included, is
    /// `pressed`.
    pub mapping_type: ButtonMappingType,
    /// JSON `scope`; anything but `global` and `view` is `controller`.
    pub scope: MappingScope,
    /// JSON `input_mode_condition`; anything but `gamepad` and `not_gamepad` is none.
    pub input_mode_condition: InputModeCondition,
}

impl Control {
    /// The member `key` of the flattened definition, unless it is absent or `null`: the
    /// definition's own, else the nearest base's that has a value for it. A `null` the
    /// definition writes itself hides the inherited member.
    ///
    /// The lookup walks the base chain up to the definition that writes `key`.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self.raw.as_object().and_then(|own| own.get(key)) {
            Some(own) => Some(own).filter(|own| !own.is_null()),
            None => self.inherited.get(key),
        }
    }

    /// The definition with its base chain flattened in, as the game holds it: its own
    /// members and every member of a base that is not `null` and that no nearer definition
    /// writes. An array is never merged. The members are listed by key, bytewise, the order
    /// the game's own object iterates in; where in the chain a member comes from does not
    /// show.
    ///
    /// Built on every call from everything the chain writes; [`get`](Self::get) reads one
    /// member without building it.
    // The game inherits shallowly, per key, into a jsoncpp object, which iterates its keys in
    // sorted order.
    pub fn body(&self) -> Value {
        if !self.raw.is_object() {
            return self.raw.clone();
        }
        let mut body = facet_value::VObject::new();
        for (key, value) in self.sorted_members() {
            body.insert(key, value.clone());
        }
        body.into_value()
    }

    /// The members of the flattened definition: its own in the order written, then the
    /// inherited ones.
    fn members(&self) -> Vec<(&str, &Value)> {
        let mut members: Vec<(&str, &Value)> = self
            .raw
            .as_object()
            .map(|own| {
                own.iter()
                    .map(|(key, value)| (key.as_str(), value))
                    .collect()
            })
            .unwrap_or_default();
        let mut seen: HashSet<&str> = members.iter().map(|(key, _)| *key).collect();
        members.extend(self.inherited.members(&mut seen));
        members
    }

    /// The members of the flattened definition in the order jsoncpp iterates them: by key.
    fn sorted_members(&self) -> Vec<(&str, &Value)> {
        let mut members = self.members();
        members.sort_by(|a, b| a.0.cmp(b.0));
        members
    }

    /// The control type, when the definition has (or inherits) a `type` the game knows.
    pub fn control_type(&self) -> Option<ControlType> {
        ControlType::from_name(string_or_empty(self.get("type")))
    }

    /// The animation type, when the definition has an `anim_type` the game knows.
    pub fn animation_type(&self) -> Option<AnimationType> {
        AnimationType::from_name(string_or_empty(self.get("anim_type")))
    }

    /// Whether the definition is a control, an animation or neither.
    pub fn kind(&self) -> ControlKind {
        match (self.control_type(), self.animation_type()) {
            (Some(control), _) => ControlKind::Control(control),
            (None, Some(animation)) => ControlKind::Animation(animation),
            (None, None) => ControlKind::Unknown,
        }
    }

    /// The children written in the definition's `controls`: each element's key and body.
    pub fn children(&self) -> Vec<(DefKey, &Value)> {
        self.get("controls").map(children_in).unwrap_or_default()
    }

    /// The texture of a sprite. With `uv` or `uv_size`, a `texture:<n>` member (the texture
    /// an overriding pack set together with its own UV rectangle) wins over `texture`, the
    /// highest pack's first.
    pub fn texture(&self) -> Option<&str> {
        let plain = || self.get("texture").and_then(Value::as_string);
        let texture = if self.get("uv").is_some() || self.get("uv_size").is_some() {
            self.sorted_members()
                .into_iter()
                .filter_map(|(key, value)| {
                    let index: usize = key.strip_prefix("texture:")?.parse().ok()?;
                    Some((index, value.as_string()?))
                })
                .max_by_key(|(index, _)| *index)
                .map(|(_, texture)| texture)
                .or_else(plain)
        } else {
            plain()
        };
        texture.map(facet_value::VString::as_str)
    }

    /// `font_size` of a label: `normal` when absent or not one of the four sizes (the game
    /// ignores an unknown size, vanilla's own `medium` included, without a message).
    pub fn font_size(&self) -> FontSize {
        FontSize::from_name(string_or_empty(self.get("font_size"))).unwrap_or(FontSize::Normal)
    }

    /// `line_padding` of a label; 0 when absent.
    pub fn line_padding(&self) -> f64 {
        self.get("line_padding").and_then(number).unwrap_or(0.0)
    }

    /// `text_alignment` of a label; `left` when absent or unknown.
    pub fn text_alignment(&self) -> TextAlignment {
        TextAlignment::from_name(string_or_empty(self.get("text_alignment")))
            .unwrap_or(TextAlignment::Left)
    }

    /// `orientation`: of a stack panel (vertical when absent) or of a flip-book animation
    /// (horizontal when absent).
    // The defaults differ by the caller: a stack panel passes 1 (vertical), a flip-book 0
    // (horizontal).
    pub fn orientation(&self) -> Orientation {
        let default = if self.get("anim_type").is_some() {
            Orientation::Horizontal
        } else {
            Orientation::Vertical
        };
        self.orientation_of("orientation", default)
    }

    /// `grid_rescaling_type` of a grid; none when absent.
    pub fn grid_rescaling_type(&self) -> Orientation {
        self.orientation_of("grid_rescaling_type", Orientation::None)
    }

    /// `grid_fill_direction` of a grid; none when absent.
    pub fn grid_fill_direction(&self) -> Orientation {
        self.orientation_of("grid_fill_direction", Orientation::None)
    }

    /// `slider_direction` of a slider; horizontal when absent.
    pub fn slider_direction(&self) -> Orientation {
        self.orientation_of("slider_direction", Orientation::Horizontal)
    }

    fn orientation_of(&self, key: &str, default: Orientation) -> Orientation {
        Orientation::from_name(string_or_empty(self.get(key))).unwrap_or(default)
    }

    /// `tiled` of a sprite: `true`, `"xy"` and `"yx"` tile both axes, `"x"` and `"y"` one;
    /// anything else, of any type, is none.
    pub fn tiled(&self) -> TileDirection {
        let Some(tiled) = self.get("tiled") else {
            return TileDirection::None;
        };
        match (tiled.as_bool(), string_or_empty(Some(tiled))) {
            (Some(true), _) | (_, "xy" | "yx") => TileDirection::Both,
            (_, "x") => TileDirection::X,
            (_, "y") => TileDirection::Y,
            _ => TileDirection::None,
        }
    }

    /// The alpha a control starts with: its `alpha` number; for an animation (a reference to
    /// one of `definitions`, or an inline object with `anim_type`) the animation's `from`;
    /// 1 when absent.
    // The default is 1.0; an animation's initial value is its initial-value key, `from`.
    pub fn initial_alpha(&self, definitions: &UiDefinitions) -> f64 {
        let from = |animation: &Value| member(animation, "from").and_then(number).unwrap_or(1.0);
        match self.get("alpha") {
            None => 1.0,
            Some(alpha) if alpha.is_object() && member(alpha, "anim_type").is_some() => from(alpha),
            Some(alpha) => match alpha.as_string() {
                Some(reference) => {
                    let reference = reference.as_str();
                    let reference = DefRef::parse(reference.strip_prefix('@').unwrap_or(reference));
                    definitions
                        .resolve(&self.namespace, &reference)
                        .map_or(1.0, |animation| {
                            animation.get("from").and_then(number).unwrap_or(1.0)
                        })
                }
                None => number(alpha).unwrap_or(1.0),
            },
        }
    }

    /// The data bindings the game registers: every object of `bindings` that is not `ignored`
    /// and whose type is not `none`.
    pub fn bindings(&self) -> Vec<Binding> {
        let text = |entry: &Value, key: &str| {
            member(entry, key)
                .and_then(Value::as_string)
                .map(|text| text.as_str().to_owned())
        };
        self.array("bindings")
            .iter()
            .filter(|entry| entry.is_object())
            .filter(|entry| member(entry, "ignored").and_then(Value::as_bool) != Some(true))
            .filter_map(|entry| {
                let binding_type = match member(entry, "binding_type") {
                    None => BindingType::Global,
                    Some(written) => BindingType::from_name(string_or_empty(Some(written)))
                        .unwrap_or(BindingType::Global),
                };
                (binding_type != BindingType::None).then(|| Binding {
                    binding_type,
                    condition: BindingCondition::from_name(string_or_empty(member(
                        entry,
                        "binding_condition",
                    )))
                    .unwrap_or(BindingCondition::None),
                    name: text(entry, "binding_name"),
                    name_override: text(entry, "binding_name_override"),
                    collection_name: text(entry, "binding_collection_name"),
                    source_control_name: text(entry, "source_control_name"),
                    source_property_name: text(entry, "source_property_name"),
                    target_property_name: text(entry, "target_property_name"),
                })
            })
            .collect()
    }

    /// The button mappings the game registers, one per object of `button_mappings`.
    pub fn button_mappings(&self) -> Vec<ButtonMapping> {
        self.array("button_mappings")
            .iter()
            .filter(|entry| entry.is_object())
            .map(|entry| {
                let text = |key: &str| string_or_empty(member(entry, key));
                ButtonMapping {
                    from_button_id: text("from_button_id").to_owned(),
                    to_button_id: text("to_button_id").to_owned(),
                    mapping_type: ButtonMappingType::from_name(text("mapping_type"))
                        .unwrap_or(ButtonMappingType::Pressed),
                    scope: match text("scope") {
                        "global" => MappingScope::Global,
                        "view" => MappingScope::View,
                        _ => MappingScope::Controller,
                    },
                    input_mode_condition: match text("input_mode_condition") {
                        "gamepad" => InputModeCondition::Gamepad,
                        "not_gamepad" => InputModeCondition::NotGamepad,
                        _ => InputModeCondition::None,
                    },
                }
            })
            .collect()
    }

    fn array(&self, key: &str) -> &[Value] {
        self.get(key)
            .and_then(Value::as_array)
            .map_or(&[], facet_value::VArray::as_slice)
    }

    /// The variables frame the control pushes over `enclosing`: its own `$name` members, the
    /// `$name|default` ones `enclosing` does not define, and every member of each `variables`
    /// entry whose `requires` holds. A string value starting with `$` is resolved against the
    /// enclosing variables first.
    pub fn variables(&self, enclosing: &Variables) -> Variables {
        let mut frame = Variables::default();
        for (key, value) in self.sorted_members() {
            if !key.starts_with('$') {
                continue;
            }
            match key.split_once('|') {
                None => frame.set(key, enclosing.resolve(value)),
                Some((name, "default")) if enclosing.get(name).is_none() => {
                    frame
                        .values
                        .entry(name.to_owned())
                        .or_insert_with(|| enclosing.resolve(value));
                }
                Some(_) => {}
            }
        }
        let entries: Vec<&Value> = match self.get("variables") {
            Some(variables) if variables.is_object() => vec![variables],
            Some(variables) => variables
                .as_array()
                .map(|entries| entries.as_slice().iter().collect())
                .unwrap_or_default(),
            None => Vec::new(),
        };
        for entry in entries {
            let applies = member(entry, "requires").is_none_or(|requires| {
                let mut seen = enclosing.clone();
                seen.values.extend(frame.values.clone());
                seen.is_truthy(requires)
            });
            if !applies {
                continue;
            }
            for (key, value) in sorted_members(entry) {
                if key != "requires" {
                    frame.set(
                        key.split_once('|').map_or(key, |(name, _)| name),
                        enclosing.resolve(value),
                    );
                }
            }
        }
        frame
    }

    /// The value the game reads for property `key`: the definition's own member, else the
    /// member of that name in the control's variables frame (which is why vanilla writes
    /// plain properties such as `offset` inside `variables` entries).
    // The definition is asked first, then the frame.
    pub fn value(&self, key: &str, enclosing: &Variables) -> Option<Value> {
        let frame = self.variables(enclosing);
        let mut seen = enclosing.clone();
        seen.values.extend(frame.values.clone());
        match self.get(key) {
            Some(own) => Some(seen.resolve(own)),
            None => frame.get(key).cloned(),
        }
        .filter(|value| !value.is_null())
    }
}

/// The children a `controls` array writes: each element's first key and its body.
fn children_in(controls: &Value) -> Vec<(DefKey, &Value)> {
    controls
        .as_array()
        .map(|children| {
            children
                .as_slice()
                .iter()
                .filter_map(|child| {
                    let key = super::value::first_key(child)?;
                    Some((DefKey::parse(key), child.as_object()?.get(key)?))
                })
                .collect()
        })
        .unwrap_or_default()
}
