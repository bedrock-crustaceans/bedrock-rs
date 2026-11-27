//! The property tables of the control factory: which internal component reads which key, with
//! what getter (the rows of [`Definition`], one field per key), and which components each control
//! type is made of.

use super::codes;
use super::control::{AnimationType, ControlType};
use super::definition::Definition;
use bedrock_addon_core::diagnostic::CodeDef;
use bedrock_addon_core::schema::Schema;

/// An internal UI component: a fixed set of keys the factory reads, the fields of [`Definition`]
/// that name it in `schema::read_by`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Component {
    /// Read for every type: the keys of the definition itself and of `_postCreate`.
    Common,
    Control,
    Layout,
    Sprite,
    Text,
    Button,
    Toggle,
    Input,
    Focus,
    Sound,
    Grid,
    StackPanel,
    Collection,
    Screen,
    Custom,
    ScrollView,
    Slider,
    SliderBox,
    TextEdit,
    Dropdown,
    Gesture,
    TextToSpeech,
    SelectionWheel,
    ImageCycling,
    TextCycling,
    PageIndicatorGrid,
    TooltipTrigger,
}

/// The getter a key is read with: the JSON type it expects.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Kind {
    /// Read by its own code path, or of no fixed type.
    Any,
    Bool,
    Number,
    Str,
    Array,
    Object,
    /// A colour: `[r, g, b(, a)]` or a name.
    ArrayOrString,
    ArrayOrNumber,
    /// A pair of layout axes.
    Axes,
    /// A string out of a fixed set.
    Enum(&'static EnumRule),
}

/// The strings of an enum-valued key and what the game does with any other string.
#[derive(Debug)]
pub(crate) struct EnumRule {
    pub values: &'static [&'static str],
    pub miss: Miss,
}

/// What a string outside an enum gets.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Miss {
    /// Nothing, in vanilla and here: the default is used (font size, clip direction).
    Silent,
    /// Nothing in vanilla, which falls back to a fixed value; here an `InvalidEnumValue`
    /// without a game text, the crate's own finding.
    CrateOwn,
    /// A fixed vanilla text.
    Fixed(&'static CodeDef<0>),
    /// A vanilla text with the value in it.
    Format(&'static CodeDef<1>),
}

const ANCHOR: EnumRule = EnumRule {
    values: &[
        "top_left",
        "top_middle",
        "top_right",
        "left_middle",
        "center",
        "right_middle",
        "bottom_left",
        "bottom_middle",
        "bottom_right",
    ],
    miss: Miss::Format(&codes::ANCHOR_UNRECOGNIZED),
};
pub(crate) const ORIENTATION: EnumRule = EnumRule {
    values: &["horizontal", "vertical", "none"],
    miss: Miss::Format(&codes::ORIENTATION_UNKNOWN),
};
const TEXT_ALIGNMENT: EnumRule = EnumRule {
    values: &["left", "right", "center"],
    miss: Miss::Fixed(&codes::TEXT_ALIGNMENT_INVALID),
};
const DRAGGABLE: EnumRule = EnumRule {
    values: &["not_draggable", "horizontal", "vertical", "both"],
    miss: Miss::Format(&codes::DRAGGABLE_UNKNOWN),
};
const PRIORITY_RULE: EnumRule = EnumRule {
    values: &["default", "show_when_controls_are_hidden"],
    miss: Miss::Format(&codes::PRIORITY_RULE_UNKNOWN),
};
const TOGGLE_MANAGER: EnumRule = EnumRule {
    values: &["clear", "select", "gather", "default"],
    miss: Miss::Format(&codes::TOGGLE_MANAGER_UNKNOWN),
};
// The game's slider code logs the text its toggle code uses, so a slider's miss carries the
// toggle's wording.
const SLIDER_MANAGER: EnumRule = EnumRule {
    values: &["gather"],
    miss: Miss::Format(&codes::TOGGLE_MANAGER_UNKNOWN),
};
const CLIP_DIRECTION: EnumRule = EnumRule {
    values: &["left", "right", "up", "down", "center"],
    miss: Miss::Silent,
};
const FONT_SIZE: EnumRule = EnumRule {
    values: &["small", "normal", "large", "extra_large"],
    miss: Miss::Silent,
};
const FOCUS_NAVIGATION: EnumRule = EnumRule {
    values: &["", "none", "stop", "contained", "custom"],
    miss: Miss::Fixed(&codes::FOCUS_NAVIGATION_INVALID),
};
const DEBUG_COLOR: EnumRule = EnumRule {
    values: &[
        "red", "black", "white", "gray", "blue", "green", "yellow", "purple",
    ],
    miss: Miss::Fixed(&codes::DEBUG_COLOR_UNRECOGNIZED),
};
const GRADIENT_DIRECTION: EnumRule = EnumRule {
    values: &["vertical", "horizontal"],
    miss: Miss::CrateOwn,
};
// The paper doll also knows `custom_y`; the panorama renderer does not.
const ROTATION: EnumRule = EnumRule {
    values: &["none", "auto", "gesture_x", "custom_y"],
    miss: Miss::CrateOwn,
};
pub(crate) const INSERT_LOCATION: EnumRule = EnumRule {
    values: &["front", "back"],
    miss: Miss::CrateOwn,
};
pub(crate) const MAPPING_SCOPE: EnumRule = EnumRule {
    values: &["controller", "view", "global"],
    miss: Miss::CrateOwn,
};
pub(crate) const INPUT_MODE: EnumRule = EnumRule {
    values: &["none", "not_gamepad", "gamepad"],
    miss: Miss::CrateOwn,
};

/// The components by the names the declaration gives them in `schema::read_by`.
const COMPONENTS: &[(Component, &str)] = &[
    (Component::Common, "common"),
    (Component::Control, "control"),
    (Component::Layout, "layout"),
    (Component::Sprite, "sprite"),
    (Component::Text, "text"),
    (Component::Button, "button"),
    (Component::Toggle, "toggle"),
    (Component::Input, "input"),
    (Component::Focus, "focus"),
    (Component::Sound, "sound"),
    (Component::Grid, "grid"),
    (Component::StackPanel, "stack_panel"),
    (Component::Collection, "collection"),
    (Component::Screen, "screen"),
    (Component::Custom, "custom"),
    (Component::ScrollView, "scroll_view"),
    (Component::Slider, "slider"),
    (Component::SliderBox, "slider_box"),
    (Component::TextEdit, "text_edit"),
    (Component::Dropdown, "dropdown"),
    (Component::Gesture, "gesture"),
    (Component::TextToSpeech, "text_to_speech"),
    (Component::SelectionWheel, "selection_wheel"),
    (Component::ImageCycling, "image_cycling"),
    (Component::TextCycling, "text_cycling"),
    (Component::PageIndicatorGrid, "page_indicator_grid"),
    (Component::TooltipTrigger, "tooltip_trigger"),
];

/// The component a `schema::read_by` name names.
fn component(name: &str) -> Option<Component> {
    COMPONENTS
        .iter()
        .find(|(_, known)| *known == name)
        .map(|(component, _)| *component)
}

/// The getter a `schema::getter` name names; `None` for a row without one (the members no
/// component reads) and for a name that is none.
fn kind(getter: Option<&str>) -> Option<Kind> {
    Some(match getter? {
        "any" => Kind::Any,
        "bool" => Kind::Bool,
        "number" => Kind::Number,
        "string" => Kind::Str,
        "array" => Kind::Array,
        "object" => Kind::Object,
        "array_or_string" => Kind::ArrayOrString,
        "array_or_number" => Kind::ArrayOrNumber,
        "axes" => Kind::Axes,
        "anchor" => Kind::Enum(&ANCHOR),
        "orientation" => Kind::Enum(&ORIENTATION),
        "text_alignment" => Kind::Enum(&TEXT_ALIGNMENT),
        "draggable" => Kind::Enum(&DRAGGABLE),
        "priority_rule" => Kind::Enum(&PRIORITY_RULE),
        "toggle_manager" => Kind::Enum(&TOGGLE_MANAGER),
        "slider_manager" => Kind::Enum(&SLIDER_MANAGER),
        "clip_direction" => Kind::Enum(&CLIP_DIRECTION),
        "font_size" => Kind::Enum(&FONT_SIZE),
        "focus_navigation" => Kind::Enum(&FOCUS_NAVIGATION),
        "debug_color" => Kind::Enum(&DEBUG_COLOR),
        "gradient_direction" => Kind::Enum(&GRADIENT_DIRECTION),
        "rotation" => Kind::Enum(&ROTATION),
        _ => return None,
    })
}

/// The components the factory populates for a control type, besides the ones every type gets.
// The game builds a control from a jump table over its type; this is that table.
pub(crate) fn components(control: ControlType) -> &'static [Component] {
    use Component as C;
    match control {
        ControlType::Button => &[C::Button, C::Focus, C::Input, C::Sound, C::TextToSpeech],
        ControlType::Custom => &[C::Custom, C::TextToSpeech],
        ControlType::CollectionPanel => &[C::Collection],
        ControlType::Dropdown => &[
            C::Focus,
            C::Input,
            C::Sound,
            C::Toggle,
            C::Dropdown,
            C::TextToSpeech,
        ],
        ControlType::EditBox => &[C::Button, C::Focus, C::Input, C::TextEdit, C::TextToSpeech],
        ControlType::Factory | ControlType::Panel => &[],
        ControlType::Grid => &[C::Focus, C::Grid],
        ControlType::Image => &[C::Sprite],
        ControlType::InputPanel => &[C::Focus, C::Input, C::Gesture, C::Sound, C::TextToSpeech],
        ControlType::Label => &[C::Text, C::TextToSpeech],
        ControlType::Screen => &[C::Input, C::Screen, C::Sound],
        ControlType::ScrollbarBox => &[C::Input],
        ControlType::ScrollTrack => &[C::Focus, C::Input, C::TextToSpeech],
        ControlType::ScrollView => &[C::Input, C::ScrollView],
        ControlType::SelectionWheel => &[
            C::SelectionWheel,
            C::Focus,
            C::Input,
            C::Sound,
            C::TextToSpeech,
        ],
        ControlType::Slider => &[C::Focus, C::Input, C::Slider, C::Sound, C::TextToSpeech],
        ControlType::SliderBox => &[C::Input, C::SliderBox],
        ControlType::StackPanel => &[C::Collection, C::StackPanel],
        ControlType::Toggle => &[C::Focus, C::Input, C::Sound, C::Toggle, C::TextToSpeech],
        ControlType::ImageCycler => &[C::Input, C::Sprite, C::ImageCycling],
        ControlType::LabelCycler => &[C::Text, C::TextToSpeech, C::Input, C::TextCycling],
        ControlType::GridPageIndicator => &[C::Focus, C::Grid, C::PageIndicatorGrid],
        ControlType::TooltipTrigger => &[
            C::Focus,
            C::Input,
            C::Gesture,
            C::Sound,
            C::TooltipTrigger,
            C::TextToSpeech,
        ],
    }
}

/// Whether a control of `control` (`None`: of no known type) has `component`.
fn has(control: Option<ControlType>, component: Component) -> bool {
    matches!(
        component,
        Component::Common | Component::Control | Component::Layout
    ) || control.is_some_and(|control| components(control).contains(&component))
}

/// How the factory reads `key` on a control of type `control`.
pub(crate) enum Lookup {
    /// A component of the type reads it, with this getter.
    Read(Kind),
    /// Only components the type does not have read it.
    Unused,
    /// No table names the key.
    Unknown,
}

/// Looks `key` up for a control of type `control` in [`Definition`]'s rows; without a type, a
/// key is read as any component that has it reads it.
pub(crate) fn lookup(control: Option<ControlType>, key: &str) -> Lookup {
    let row = Definition::FIELDS.iter().find(|row| row.json == key);
    let Some((row, getter)) = row.and_then(|row| Some((row, kind(row.getter)?))) else {
        return Lookup::Unknown;
    };
    let mut readers = row.read_by.iter().filter_map(|name| component(name));
    if control.is_none() || readers.any(|reader| has(control, reader)) {
        return Lookup::Read(getter);
    }
    // A renderer's own keys are exempt: only `renderer` itself is the factory's.
    if key == "renderer" || row.read_by.iter().any(|name| *name != "custom") {
        Lookup::Unused
    } else {
        Lookup::Unknown
    }
}

/// The keys read through `_resolveAnimatedProperty`: the value may name an animation or be an
/// inline one.
pub(crate) const ANIMATED: &[&str] = &[
    "alpha",
    "size",
    "offset",
    "min_size",
    "max_size",
    "uv",
    "color",
    "clip_ratio",
];

/// Keys vanilla files and the documentation carry and the game never reads.
pub(crate) const DEAD_KEYS: &[&str] = &[
    "always_rotate",
    "rotate_speed",
    "hover_color",
    "hover_alpha",
    "pressed_color",
    "pressed_alpha",
    "nine_slice_buttom",
    "nine_slice_left",
    "nine_slice_right",
    "nine_slice_top",
    "place_holder_text",
    "slider_range",
];

/// Animation keys vanilla files carry and the game never reads.
pub(crate) const DEAD_ANIMATION_KEYS: &[&str] = &["activated", "start_event"];

/// The keys only a `flip_book` animation reads.
pub(crate) const FLIP_BOOK_KEYS: &[&str] = &[
    "fps",
    "frame_count",
    "frame_step",
    "reversible",
    "orientation",
    "looping",
];

/// Whether an animation of type `animation` reads `key`, for the keys that depend on the type.
pub(crate) fn animation_reads(animation: AnimationType, key: &str) -> bool {
    if FLIP_BOOK_KEYS.contains(&key) {
        animation == AnimationType::FlipBook
    } else if key == "initial_uv" {
        matches!(
            animation,
            AnimationType::FlipBook | AnimationType::AsepriteFlipBook
        )
    } else if key == "scale_from_starting_alpha" {
        animation == AnimationType::Alpha
    } else {
        true
    }
}

/// The renderers a `custom` control may name.
// `editor_gizmo_renderer` is a Windows Editor renderer that other platforms lack; vanilla's
// own `hud_screen.json` uses it, so it is accepted.
pub(crate) const RENDERERS: &[&str] = &[
    "hud_player_renderer",
    "live_player_renderer",
    "live_horse_renderer",
    "name_tag_renderer",
    "paper_doll_renderer",
    "inventory_item_renderer",
    "background_renderer",
    "splash_text_renderer",
    "debug_screen_renderer",
    "progress_bar_renderer",
    "hover_text_renderer",
    "enchanting_book_renderer",
    "hotbar_renderer",
    "hotbar_cooldown_renderer",
    "flying_item_renderer",
    "heart_renderer",
    "horse_heart_renderer",
    "armor_renderer",
    "horse_jump_renderer",
    "dash_renderer",
    "locator_bar",
    "hunger_renderer",
    "bubbles_renderer",
    "vignette_renderer",
    "cursor_renderer",
    "progress_indicator_renderer",
    "mob_effects_renderer",
    "camera_renderer",
    "equipment_preview_renderer",
    "gradient_renderer",
    "panorama_renderer",
    "3d_structure_renderer",
    "bohr_model_renderer",
    "web_view_renderer",
    "actor_portrait_renderer",
    "animated_gif_renderer",
    "banner_pattern_renderer",
    "credits_renderer",
    "qr_code_renderer",
    "profile_image_renderer",
    "editor_volume_highlight_renderer",
    "editor_compass_renderer",
    "bundle_tooltip_renderer",
    "trial_time_renderer",
    "editor_gizmo_renderer",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_type_has_the_components_of_the_factory_table() {
        // The 24 rows of metadata "Type → component table", without the three every type has
        // (Control, Layout) or that read no key of their own (DataBinding, Scrollbar, …).
        assert_eq!(ControlType::ALL.len(), 24);
        assert!(components(ControlType::Panel).is_empty());
        assert!(components(ControlType::Factory).is_empty());
        assert!(components(ControlType::Button).contains(&Component::Input));
        assert!(!components(ControlType::Label).contains(&Component::Input));
        assert!(components(ControlType::LabelCycler).contains(&Component::Input));
        assert!(!components(ControlType::Grid).contains(&Component::Input));
        assert!(components(ControlType::StackPanel).contains(&Component::Collection));
        assert!(!components(ControlType::CollectionPanel).contains(&Component::StackPanel));
    }

    #[test]
    fn every_row_names_known_components_and_a_known_getter() {
        for row in Definition::FIELDS.iter().filter(|row| row.json != "extra") {
            assert!(
                kind(row.getter).is_some(),
                "{}: getter {:?}",
                row.json,
                row.getter
            );
            assert!(!row.read_by.is_empty(), "{}: no component", row.json);
            for name in row.read_by {
                assert!(component(name).is_some(), "{}: component {name}", row.json);
            }
        }
        assert_eq!(
            Definition::FIELDS.len(),
            287,
            "286 keys and the unread members"
        );
    }

    /// Characterization of the rows' edges: the renamed keys, the first component's getter of
    /// a key two components read, a renderer's own keys, and the member that holds the rest.
    #[test]
    fn the_rows_keep_the_tables_edges() {
        let read = |control, key| matches!(lookup(control, key), Lookup::Read(_));
        assert!(read(Some(ControlType::Panel), "ignoreCollectionItem"));
        assert!(read(Some(ControlType::Panel), "ttsSectionContainer"));
        assert!(matches!(
            lookup(Some(ControlType::Panel), "ignore_collection_item"),
            Lookup::Unknown
        ));
        assert!(matches!(
            lookup(Some(ControlType::Label), "tts_section_header"),
            Lookup::Read(Kind::Str)
        ));
        assert!(matches!(
            lookup(Some(ControlType::Panel), "use_uuid"),
            Lookup::Unknown
        ));
        assert!(matches!(
            lookup(Some(ControlType::Panel), "renderer"),
            Lookup::Unused
        ));
        assert!(read(Some(ControlType::Custom), "use_uuid"));
        assert!(matches!(lookup(None, "extra"), Lookup::Unknown));
        assert!(matches!(
            lookup(Some(ControlType::Panel), "type"),
            Lookup::Read(Kind::Any)
        ));
    }

    #[test]
    fn a_key_is_read_unused_or_unknown_by_type() {
        assert!(matches!(
            lookup(Some(ControlType::Panel), "button_mappings"),
            Lookup::Unused
        ));
        assert!(matches!(
            lookup(Some(ControlType::Button), "button_mappings"),
            Lookup::Read(Kind::Array)
        ));
        assert!(matches!(
            lookup(Some(ControlType::Panel), "visible"),
            Lookup::Read(Kind::Bool)
        ));
        assert!(matches!(
            lookup(Some(ControlType::Panel), "no_such_key"),
            Lookup::Unknown
        ));
        assert!(matches!(lookup(None, "texture"), Lookup::Read(Kind::Str)));
        // `color` is a sprite's and a label's
        assert!(matches!(
            lookup(Some(ControlType::Label), "color"),
            Lookup::Read(Kind::ArrayOrString)
        ));
        assert!(matches!(
            lookup(Some(ControlType::Panel), "color"),
            Lookup::Unused
        ));
    }
}
