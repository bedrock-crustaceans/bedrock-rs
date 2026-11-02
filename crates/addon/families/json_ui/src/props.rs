//! The property tables of the control factory: which internal component reads which key, with
//! what getter, and which components each control type is made of.

use super::control::{AnimationType, ControlType};

/// An internal UI component: a fixed set of keys the factory reads.
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
    Fixed(&'static str),
    /// A vanilla text with the value in it.
    Format(&'static str),
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
    miss: Miss::Format("Unrecognized anchor or alignment value: '{}'"),
};
pub(crate) const ORIENTATION: EnumRule = EnumRule {
    values: &["horizontal", "vertical", "none"],
    miss: Miss::Format("Unknown Orientation value: {}"),
};
const TEXT_ALIGNMENT: EnumRule = EnumRule {
    values: &["left", "right", "center"],
    miss: Miss::Fixed("JSON UI parse failure - Text Alignment value invalid."),
};
const DRAGGABLE: EnumRule = EnumRule {
    values: &["not_draggable", "horizontal", "vertical", "both"],
    miss: Miss::Format("Unknown draggable value: {}"),
};
const PRIORITY_RULE: EnumRule = EnumRule {
    values: &["default", "show_when_controls_are_hidden"],
    miss: Miss::Format("Unknown priority rule value: {}"),
};
const TOGGLE_MANAGER: EnumRule = EnumRule {
    values: &["clear", "select", "gather", "default"],
    miss: Miss::Format("Unknown toggle manager behavior type value: {}"),
};
// The game's slider code logs the text its toggle code uses, so a slider's miss carries the
// toggle's wording.
const SLIDER_MANAGER: EnumRule = EnumRule {
    values: &["gather"],
    miss: Miss::Format("Unknown toggle manager behavior type value: {}"),
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
    miss: Miss::Fixed("Invalid Input for Focus Navigation"),
};
const DEBUG_COLOR: EnumRule = EnumRule {
    values: &[
        "red", "black", "white", "gray", "blue", "green", "yellow", "purple",
    ],
    miss: Miss::Fixed("unrecognized debug color specified: "),
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

use Kind::{Any, Array, ArrayOrNumber, ArrayOrString, Axes, Bool, Number, Object, Str};

type Keys = &'static [(&'static str, Kind)];

const COMMON: Keys = &[
    ("type", Any),
    ("controls", Array),
    ("ignored", Any),
    ("variables", Any),
    ("bindings", Any),
    ("anims", Array),
    ("disable_anim_fast_forward", Bool),
    ("animation_reset_name", Str),
    ("modifications", Any),
    ("factory", Object),
    ("grid_position", Array),
    ("collection_index", Number),
    ("ignoreCollectionItem", Bool),
    ("toggle_manager_behavior", Kind::Enum(&TOGGLE_MANAGER)),
    ("toggle_manage_groups", Array),
    ("slider_manager_behavior", Kind::Enum(&SLIDER_MANAGER)),
    ("slider_manage_groups", Array),
    ("use_anchored_offset", Bool),
    ("focus_container", Bool),
    ("use_last_focus", Bool),
    ("focus_wrap_enabled", Bool),
    ("focus_navigation_mode_up", Kind::Enum(&FOCUS_NAVIGATION)),
    ("focus_navigation_mode_down", Kind::Enum(&FOCUS_NAVIGATION)),
    ("focus_navigation_mode_left", Kind::Enum(&FOCUS_NAVIGATION)),
    ("focus_navigation_mode_right", Kind::Enum(&FOCUS_NAVIGATION)),
    ("focus_container_custom_up", Array),
    ("focus_container_custom_down", Array),
    ("focus_container_custom_left", Array),
    ("focus_container_custom_right", Array),
    ("debug", Kind::Enum(&DEBUG_COLOR)),
    ("tts_section_header", Str),
];
const CONTROL: Keys = &[
    ("visible", Bool),
    ("enabled", Bool),
    ("layer", Number),
    ("alpha", Number),
    ("propagate_alpha", Bool),
    ("clips_children", Bool),
    ("clip_offset", Array),
    ("allow_clipping", Bool),
    ("enable_scissor_test", Bool),
    ("property_bag", Object),
    ("property_bag_for_children", Object),
    ("clip_state_change_event", Str),
    ("selected", Bool),
];
const LAYOUT: Keys = &[
    ("size", Axes),
    ("offset", Axes),
    ("min_size", Axes),
    ("max_size", Axes),
    ("anchor_from", Kind::Enum(&ANCHOR)),
    ("anchor_to", Kind::Enum(&ANCHOR)),
    ("contained", Bool),
    ("draggable", Kind::Enum(&DRAGGABLE)),
    ("follows_cursor", Bool),
    ("follows_cursor_inside_parent", Bool),
    ("inherit_max_sibling_width", Bool),
    ("inherit_max_sibling_height", Bool),
    ("priority", Number),
    ("priority_rule", Kind::Enum(&PRIORITY_RULE)),
    ("ttsSectionContainer", Bool),
];
const SPRITE: Keys = &[
    ("texture", Str),
    ("texture_file_system", Str),
    ("uv", Array),
    ("uv_size", Array),
    ("color", ArrayOrString),
    ("nineslice_size", ArrayOrNumber),
    ("tiled", Any),
    ("tiled_scale", Array),
    ("clip_direction", Kind::Enum(&CLIP_DIRECTION)),
    ("clip_ratio", Number),
    ("clip_pixelperfect", Bool),
    ("pixel_perfect", Bool),
    ("keep_ratio", Bool),
    ("fill", Bool),
    ("default_size_scales_to_ratio", Bool),
    ("zip_folder", Str),
    ("grayscale", Bool),
    ("bilinear", Bool),
    ("force_texture_reload", Bool),
    ("allow_debug_missing_texture", Bool),
    ("color_corrected", Bool),
];
const TEXT: Keys = &[
    ("text", Str),
    ("text_tts", Str),
    ("color", ArrayOrString),
    ("locked_color", ArrayOrString),
    ("shadow", Bool),
    ("hide_hyphen", Bool),
    ("locked_alpha", Number),
    ("font_size", Kind::Enum(&FONT_SIZE)),
    ("font_scale_factor", Number),
    ("localize", Bool),
    ("line_padding", Number),
    ("enable_profanity_filter", Bool),
    ("notify_on_ellipses", Array),
    ("notify_ellipses_sibling", Bool),
    ("font_type", Any),
    ("backup_font_type", Any),
    ("text_alignment", Kind::Enum(&TEXT_ALIGNMENT)),
];
const BUTTON: Keys = &[
    ("default_control", Str),
    ("hover_control", Str),
    ("pressed_control", Str),
    ("locked_control", Str),
    ("consume_event", Bool),
];
const TOGGLE: Keys = &[
    ("radio_toggle_group", Bool),
    ("toggle_name", Str),
    ("descriptive_text", Str),
    ("toggle_default_state", Bool),
    ("toggle_group_forced_index", Number),
    ("toggle_group_default_selected", Number),
    ("toggle_on_hover", Bool),
    ("enable_directional_toggling", Bool),
    ("toggle_on_button", Str),
    ("toggle_off_button", Str),
    ("toggle_grid_collection_name", Str),
    ("tts_toggle_on", Str),
    ("tts_toggle_off", Str),
    ("checked_control", Str),
    ("unchecked_control", Str),
    ("checked_hover_control", Str),
    ("unchecked_hover_control", Str),
    ("checked_locked_control", Str),
    ("unchecked_locked_control", Str),
    ("checked_locked_hover_control", Str),
    ("unchecked_locked_hover_control", Str),
];
const INPUT: Keys = &[
    ("button_mappings", Array),
    ("modal", Bool),
    ("inline_modal", Bool),
    ("gamepad_deflection_mode", Bool),
    ("always_listen_to_input", Bool),
    ("always_handle_pointer", Bool),
    ("always_handle_controller_direction", Bool),
    ("hover_enabled", Bool),
    ("consume_hover_events", Bool),
    ("prevent_touch_input", Bool),
];
const FOCUS: Keys = &[
    ("default_focus_precedence", Number),
    ("focus_enabled", Bool),
    ("focus_wrap_enabled", Bool),
    ("focus_magnet_enabled", Bool),
    ("focus_identifier", Str),
    ("reset_on_focus_lost", Bool),
    ("focus_change_up", Str),
    ("focus_change_down", Str),
    ("focus_change_left", Str),
    ("focus_change_right", Str),
    ("focus_mapping", Any),
];
const SOUND: Keys = &[
    ("sound_name", Str),
    ("sound_volume", Number),
    ("sound_pitch", Number),
    ("sounds", Array),
];
const GRID: Keys = &[
    ("grid_dimensions", Array),
    ("maximum_grid_items", Number),
    ("grid_dimension_binding", Str),
    ("collection_name", Str),
    ("grid_rescaling_type", Kind::Enum(&ORIENTATION)),
    ("grid_fill_direction", Kind::Enum(&ORIENTATION)),
    ("grid_item_template", Str),
];
const STACK_PANEL: Keys = &[
    ("orientation", Kind::Enum(&ORIENTATION)),
    ("use_child_anchors", Bool),
    ("use_priority", Bool),
];
const COLLECTION: Keys = &[("collection_name", Str)];
const SCREEN: Keys = &[
    ("screen_not_flushable", Bool),
    ("always_accepts_input", Bool),
    ("render_game_behind", Bool),
    ("absorbs_input", Bool),
    ("is_showing_menu", Bool),
    ("is_modal", Bool),
    ("should_steal_mouse", Bool),
    ("low_frequency_rendering", Bool),
    ("screen_draws_last", Bool),
    ("force_render_below", Bool),
    ("send_telemetry", Bool),
    ("close_on_player_hurt", Bool),
    ("use_custom_pocket_toast", Bool),
    ("cache_screen", Bool),
    ("gamepad_cursor", Bool),
    ("gamepad_cursor_deflection_mode", Bool),
    ("vertical_scroll_delta", Number),
    ("load_screen_immediately", Bool),
    ("render_only_when_topmost", Bool),
    ("should_be_skipped_during_automation", Bool),
];
const CUSTOM: Keys = &[
    ("renderer", Str),
    ("text_color", ArrayOrString),
    ("background_color", ArrayOrString),
    ("primary_color", ArrayOrString),
    ("secondary_color", ArrayOrString),
    ("full_storage_color", ArrayOrString),
    ("foreground_color", ArrayOrString),
    ("gradient_direction", Kind::Enum(&GRADIENT_DIRECTION)),
    ("color1", ArrayOrString),
    ("color2", ArrayOrString),
    ("use_selected_skin", Bool),
    ("use_uuid", Bool),
    ("use_player_paperdoll", Bool),
    ("use_skin_gui_scale", Bool),
    ("camera_tilt_degrees", Number),
    ("rotation", Kind::Enum(&ROTATION)),
    ("starting_rotation", Number),
    ("rotation_x", Number),
    ("rotation_y", Number),
    ("hover_text_max_width", Number),
    ("replaced_while_inactive", Bool),
    ("end_event", Str),
];
const SCROLL_VIEW: Keys = &[
    ("scrollbar_track_button", Any),
    ("scrollbar_touch_button", Any),
    ("scroll_speed", Number),
    ("gesture_control_enabled", Bool),
    ("always_handle_scrolling", Bool),
    ("touch_mode", Bool),
    ("allow_scroll_even_when_content_fits", Bool),
    ("scrollbar_always_visible", Bool),
    ("scrollbar_box", Any),
    ("scrollbar_track", Any),
    ("scroll_view_port", Any),
    ("scroll_content", Any),
    ("scroll_box_and_track_panel", Any),
    ("jump_to_bottom_on_update", Any),
];
const SLIDER: Keys = &[
    ("slider_track_button", Any),
    ("slider_small_decrease_button", Any),
    ("slider_small_increase_button", Any),
    ("slider_selected_button", Any),
    ("slider_deselected_button", Any),
    ("slider_steps", Number),
    ("slider_direction", Kind::Enum(&ORIENTATION)),
    ("slider_inverted", Bool),
    ("slider_speed", Number),
    ("slider_timeout", Number),
    ("slider_collection_name", Any),
    ("slider_name", Any),
    ("slider_select_on_hover", Bool),
    ("slider_box_control", Any),
    ("tts_value_changed", Any),
    ("default_control", Str),
    ("hover_control", Str),
    ("background_control", Any),
    ("background_hover_control", Any),
    ("progress_control", Any),
    ("progress_hover_control", Any),
];
const SLIDER_BOX: Keys = &[
    ("default_control", Str),
    ("hover_control", Str),
    ("locked_control", Str),
    ("indent_control", Any),
];
const TEXT_EDIT: Keys = &[
    ("text_box_name", Any),
    ("text_edit_box_grid_collection_name", Any),
    ("constrain_to_rect", Bool),
    ("enabled_newline", Bool),
    ("text_type", Any),
    ("max_length", Number),
    ("virtual_keyboard_buffer_control", Any),
    ("text_control", Any),
    ("place_holder_control", Any),
    ("can_be_deselected", Bool),
    ("always_listening", Bool),
    ("place_holder_text_hover_color", ArrayOrString),
];
const DROPDOWN: Keys = &[
    ("dropdown_name", Any),
    ("dropdown_content_control", Any),
    ("dropdown_area", Any),
];
const GESTURE: Keys = &[("gesture_tracking_button", Any)];
const TEXT_TO_SPEECH: Keys = &[
    ("tts_name", Any),
    ("tts_control_header", Any),
    ("tts_section_header", Any),
    ("tts_override_control_value", Any),
    ("tts_inherit_siblings", Any),
    ("tts_message_interruptible", Bool),
    ("tts_message_interruptible_id", Any),
    ("tts_index_order", Any),
    ("tts_index_priority", Number),
    ("tts_ignore_subsections", Any),
    ("tts_ignore_count", Any),
    ("tts_skip_message", Any),
    ("tts_skip_children", Any),
    ("tts_play_on_unchanged_focus_control", Any),
    ("tts_control_type_order_priority", Number),
    ("tts_value_order_priority", Number),
];
const SELECTION_WHEEL: Keys = &[
    ("consume_event", Bool),
    ("slice_count", Number),
    ("inner_radius", Number),
    ("outer_radius", Number),
    ("state_controls", Any),
    ("analog_button_name", Any),
    ("hover_button_name", Any),
    ("select_button_name", Any),
    ("iterate_left_button_name", Any),
    ("iterate_right_button_name", Any),
    ("initial_button_slice", Number),
];
// With the PageIndicatorManager both cyclers also get.
const IMAGE_CYCLING: Keys = &[
    ("images", Any),
    ("next_sub_page_button_name", Any),
    ("prev_sub_page_button_name", Any),
    ("target_cycler_to_compare", Any),
];
const TEXT_CYCLING: Keys = &[
    ("text_labels", Array),
    ("next_sub_page_button_name", Any),
    ("prev_sub_page_button_name", Any),
    ("target_cycler_to_compare", Any),
];
const PAGE_INDICATOR_GRID: Keys = &[
    ("cycler_manager_size_control_target", Any),
    ("grid_item_when_current", Any),
    ("grid_item_when_not_current", Any),
];
const TOOLTIP_TRIGGER: Keys = &[
    ("tooltip_name", Any),
    ("tooltip_tts_value", Any),
    ("tooltip_top_content_control", Any),
    ("tooltip_bottom_content_control", Any),
    ("tooltip_area", Any),
];

/// Every component with its keys.
const TABLE: &[(Component, Keys)] = &[
    (Component::Common, COMMON),
    (Component::Control, CONTROL),
    (Component::Layout, LAYOUT),
    (Component::Sprite, SPRITE),
    (Component::Text, TEXT),
    (Component::Button, BUTTON),
    (Component::Toggle, TOGGLE),
    (Component::Input, INPUT),
    (Component::Focus, FOCUS),
    (Component::Sound, SOUND),
    (Component::Grid, GRID),
    (Component::StackPanel, STACK_PANEL),
    (Component::Collection, COLLECTION),
    (Component::Screen, SCREEN),
    (Component::Custom, CUSTOM),
    (Component::ScrollView, SCROLL_VIEW),
    (Component::Slider, SLIDER),
    (Component::SliderBox, SLIDER_BOX),
    (Component::TextEdit, TEXT_EDIT),
    (Component::Dropdown, DROPDOWN),
    (Component::Gesture, GESTURE),
    (Component::TextToSpeech, TEXT_TO_SPEECH),
    (Component::SelectionWheel, SELECTION_WHEEL),
    (Component::ImageCycling, IMAGE_CYCLING),
    (Component::TextCycling, TEXT_CYCLING),
    (Component::PageIndicatorGrid, PAGE_INDICATOR_GRID),
    (Component::TooltipTrigger, TOOLTIP_TRIGGER),
];

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

/// Looks `key` up for a control of type `control`; without a type, a key is read as the first
/// component that has it reads it.
pub(crate) fn lookup(control: Option<ControlType>, key: &str) -> Lookup {
    let mut known = None;
    for (component, keys) in TABLE {
        if let Some((_, kind)) = keys.iter().find(|(name, _)| *name == key) {
            if control.is_none() || has(control, *component) {
                return Lookup::Read(*kind);
            }
            // A renderer's own keys are exempt: only `renderer` itself is the factory's.
            if *component != Component::Custom || key == "renderer" {
                known = Some(());
            }
        }
    }
    match known {
        Some(()) => Lookup::Unused,
        None => Lookup::Unknown,
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
