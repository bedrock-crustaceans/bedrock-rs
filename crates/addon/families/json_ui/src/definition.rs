//! The control factory's key set declared as one struct: its rows are the factory's tables.

// The derives build a table of every field: 286 keys and the members no component reads.
#![allow(
    clippy::large_stack_arrays,
    reason = "the derives' tables of a struct of 287 fields"
)]

use facet::Facet;

use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::schema::{self, Schema};

/// A definition as the control factory reads it: every key one of its components reads, each
/// with the components that read it (`schema::read_by`, by the names of `props::COMPONENTS`)
/// and the getter they read it with (`schema::getter`, by the names of `props::kind`). The rows
/// of its `Schema::FIELDS` are the factory's tables; nothing is read into it, because a
/// definition is held as the JSON it was written as: the merge, the base chain and the write
/// work on that, and a value is judged by its getter only when a control is built from it.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
pub(crate) struct Definition {
    #[facet(schema::read_by = "common", schema::getter = "any")]
    r#type: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "array")]
    controls: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "any")]
    ignored: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "any")]
    variables: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "any")]
    bindings: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "array")]
    anims: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "bool")]
    disable_anim_fast_forward: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "string")]
    animation_reset_name: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "any")]
    modifications: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "object")]
    factory: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "array")]
    grid_position: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "number")]
    collection_index: Option<Value>,
    #[facet(
        rename = "ignoreCollectionItem",
        schema::read_by = "common",
        schema::getter = "bool"
    )]
    ignore_collection_item: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "toggle_manager")]
    toggle_manager_behavior: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "array")]
    toggle_manage_groups: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "slider_manager")]
    slider_manager_behavior: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "array")]
    slider_manage_groups: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "bool")]
    use_anchored_offset: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "bool")]
    focus_container: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "bool")]
    use_last_focus: Option<Value>,
    #[facet(schema::read_by = "common focus", schema::getter = "bool")]
    focus_wrap_enabled: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "focus_navigation")]
    focus_navigation_mode_up: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "focus_navigation")]
    focus_navigation_mode_down: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "focus_navigation")]
    focus_navigation_mode_left: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "focus_navigation")]
    focus_navigation_mode_right: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "array")]
    focus_container_custom_up: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "array")]
    focus_container_custom_down: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "array")]
    focus_container_custom_left: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "array")]
    focus_container_custom_right: Option<Value>,
    #[facet(schema::read_by = "common", schema::getter = "debug_color")]
    debug: Option<Value>,
    // Text-to-speech reads it with the any getter; every type has common, whose string getter
    // is the one a lookup gives.
    #[facet(schema::read_by = "common text_to_speech", schema::getter = "string")]
    tts_section_header: Option<Value>,
    #[facet(schema::read_by = "control", schema::getter = "bool")]
    visible: Option<Value>,
    #[facet(schema::read_by = "control", schema::getter = "bool")]
    enabled: Option<Value>,
    #[facet(schema::read_by = "control", schema::getter = "number")]
    layer: Option<Value>,
    #[facet(schema::read_by = "control", schema::getter = "number")]
    alpha: Option<Value>,
    #[facet(schema::read_by = "control", schema::getter = "bool")]
    propagate_alpha: Option<Value>,
    #[facet(schema::read_by = "control", schema::getter = "bool")]
    clips_children: Option<Value>,
    #[facet(schema::read_by = "control", schema::getter = "array")]
    clip_offset: Option<Value>,
    #[facet(schema::read_by = "control", schema::getter = "bool")]
    allow_clipping: Option<Value>,
    #[facet(schema::read_by = "control", schema::getter = "bool")]
    enable_scissor_test: Option<Value>,
    #[facet(schema::read_by = "control", schema::getter = "object")]
    property_bag: Option<Value>,
    #[facet(schema::read_by = "control", schema::getter = "object")]
    property_bag_for_children: Option<Value>,
    #[facet(schema::read_by = "control", schema::getter = "string")]
    clip_state_change_event: Option<Value>,
    #[facet(schema::read_by = "control", schema::getter = "bool")]
    selected: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "axes")]
    size: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "axes")]
    offset: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "axes")]
    min_size: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "axes")]
    max_size: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "anchor")]
    anchor_from: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "anchor")]
    anchor_to: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "bool")]
    contained: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "draggable")]
    draggable: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "bool")]
    follows_cursor: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "bool")]
    follows_cursor_inside_parent: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "bool")]
    inherit_max_sibling_width: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "bool")]
    inherit_max_sibling_height: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "number")]
    priority: Option<Value>,
    #[facet(schema::read_by = "layout", schema::getter = "priority_rule")]
    priority_rule: Option<Value>,
    #[facet(
        rename = "ttsSectionContainer",
        schema::read_by = "layout",
        schema::getter = "bool"
    )]
    tts_section_container: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "string")]
    texture: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "string")]
    texture_file_system: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "array")]
    uv: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "array")]
    uv_size: Option<Value>,
    #[facet(schema::read_by = "sprite text", schema::getter = "array_or_string")]
    color: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "array_or_number")]
    nineslice_size: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "any")]
    tiled: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "array")]
    tiled_scale: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "clip_direction")]
    clip_direction: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "number")]
    clip_ratio: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "bool")]
    clip_pixelperfect: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "bool")]
    pixel_perfect: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "bool")]
    keep_ratio: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "bool")]
    fill: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "bool")]
    default_size_scales_to_ratio: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "string")]
    zip_folder: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "bool")]
    grayscale: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "bool")]
    bilinear: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "bool")]
    force_texture_reload: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "bool")]
    allow_debug_missing_texture: Option<Value>,
    #[facet(schema::read_by = "sprite", schema::getter = "bool")]
    color_corrected: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "string")]
    text: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "string")]
    text_tts: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "array_or_string")]
    locked_color: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "bool")]
    shadow: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "bool")]
    hide_hyphen: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "number")]
    locked_alpha: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "font_size")]
    font_size: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "number")]
    font_scale_factor: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "bool")]
    localize: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "number")]
    line_padding: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "bool")]
    enable_profanity_filter: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "array")]
    notify_on_ellipses: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "bool")]
    notify_ellipses_sibling: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "any")]
    font_type: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "any")]
    backup_font_type: Option<Value>,
    #[facet(schema::read_by = "text", schema::getter = "text_alignment")]
    text_alignment: Option<Value>,
    #[facet(
        schema::read_by = "button slider slider_box",
        schema::getter = "string"
    )]
    default_control: Option<Value>,
    #[facet(
        schema::read_by = "button slider slider_box",
        schema::getter = "string"
    )]
    hover_control: Option<Value>,
    #[facet(schema::read_by = "button", schema::getter = "string")]
    pressed_control: Option<Value>,
    #[facet(schema::read_by = "button slider_box", schema::getter = "string")]
    locked_control: Option<Value>,
    #[facet(schema::read_by = "button selection_wheel", schema::getter = "bool")]
    consume_event: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "bool")]
    radio_toggle_group: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    toggle_name: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    descriptive_text: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "bool")]
    toggle_default_state: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "number")]
    toggle_group_forced_index: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "number")]
    toggle_group_default_selected: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "bool")]
    toggle_on_hover: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "bool")]
    enable_directional_toggling: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    toggle_on_button: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    toggle_off_button: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    toggle_grid_collection_name: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    tts_toggle_on: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    tts_toggle_off: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    checked_control: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    unchecked_control: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    checked_hover_control: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    unchecked_hover_control: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    checked_locked_control: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    unchecked_locked_control: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    checked_locked_hover_control: Option<Value>,
    #[facet(schema::read_by = "toggle", schema::getter = "string")]
    unchecked_locked_hover_control: Option<Value>,
    #[facet(schema::read_by = "input", schema::getter = "array")]
    button_mappings: Option<Value>,
    #[facet(schema::read_by = "input", schema::getter = "bool")]
    modal: Option<Value>,
    #[facet(schema::read_by = "input", schema::getter = "bool")]
    inline_modal: Option<Value>,
    #[facet(schema::read_by = "input", schema::getter = "bool")]
    gamepad_deflection_mode: Option<Value>,
    #[facet(schema::read_by = "input", schema::getter = "bool")]
    always_listen_to_input: Option<Value>,
    #[facet(schema::read_by = "input", schema::getter = "bool")]
    always_handle_pointer: Option<Value>,
    #[facet(schema::read_by = "input", schema::getter = "bool")]
    always_handle_controller_direction: Option<Value>,
    #[facet(schema::read_by = "input", schema::getter = "bool")]
    hover_enabled: Option<Value>,
    #[facet(schema::read_by = "input", schema::getter = "bool")]
    consume_hover_events: Option<Value>,
    #[facet(schema::read_by = "input", schema::getter = "bool")]
    prevent_touch_input: Option<Value>,
    #[facet(schema::read_by = "focus", schema::getter = "number")]
    default_focus_precedence: Option<Value>,
    #[facet(schema::read_by = "focus", schema::getter = "bool")]
    focus_enabled: Option<Value>,
    #[facet(schema::read_by = "focus", schema::getter = "bool")]
    focus_magnet_enabled: Option<Value>,
    #[facet(schema::read_by = "focus", schema::getter = "string")]
    focus_identifier: Option<Value>,
    #[facet(schema::read_by = "focus", schema::getter = "bool")]
    reset_on_focus_lost: Option<Value>,
    #[facet(schema::read_by = "focus", schema::getter = "string")]
    focus_change_up: Option<Value>,
    #[facet(schema::read_by = "focus", schema::getter = "string")]
    focus_change_down: Option<Value>,
    #[facet(schema::read_by = "focus", schema::getter = "string")]
    focus_change_left: Option<Value>,
    #[facet(schema::read_by = "focus", schema::getter = "string")]
    focus_change_right: Option<Value>,
    #[facet(schema::read_by = "focus", schema::getter = "any")]
    focus_mapping: Option<Value>,
    #[facet(schema::read_by = "sound", schema::getter = "string")]
    sound_name: Option<Value>,
    #[facet(schema::read_by = "sound", schema::getter = "number")]
    sound_volume: Option<Value>,
    #[facet(schema::read_by = "sound", schema::getter = "number")]
    sound_pitch: Option<Value>,
    #[facet(schema::read_by = "sound", schema::getter = "array")]
    sounds: Option<Value>,
    #[facet(schema::read_by = "grid", schema::getter = "array")]
    grid_dimensions: Option<Value>,
    #[facet(schema::read_by = "grid", schema::getter = "number")]
    maximum_grid_items: Option<Value>,
    #[facet(schema::read_by = "grid", schema::getter = "string")]
    grid_dimension_binding: Option<Value>,
    #[facet(schema::read_by = "grid collection", schema::getter = "string")]
    collection_name: Option<Value>,
    #[facet(schema::read_by = "grid", schema::getter = "orientation")]
    grid_rescaling_type: Option<Value>,
    #[facet(schema::read_by = "grid", schema::getter = "orientation")]
    grid_fill_direction: Option<Value>,
    #[facet(schema::read_by = "grid", schema::getter = "string")]
    grid_item_template: Option<Value>,
    #[facet(schema::read_by = "stack_panel", schema::getter = "orientation")]
    orientation: Option<Value>,
    #[facet(schema::read_by = "stack_panel", schema::getter = "bool")]
    use_child_anchors: Option<Value>,
    #[facet(schema::read_by = "stack_panel", schema::getter = "bool")]
    use_priority: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    screen_not_flushable: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    always_accepts_input: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    render_game_behind: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    absorbs_input: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    is_showing_menu: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    is_modal: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    should_steal_mouse: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    low_frequency_rendering: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    screen_draws_last: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    force_render_below: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    send_telemetry: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    close_on_player_hurt: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    use_custom_pocket_toast: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    cache_screen: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    gamepad_cursor: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    gamepad_cursor_deflection_mode: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "number")]
    vertical_scroll_delta: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    load_screen_immediately: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    render_only_when_topmost: Option<Value>,
    #[facet(schema::read_by = "screen", schema::getter = "bool")]
    should_be_skipped_during_automation: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "string")]
    renderer: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "array_or_string")]
    text_color: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "array_or_string")]
    background_color: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "array_or_string")]
    primary_color: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "array_or_string")]
    secondary_color: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "array_or_string")]
    full_storage_color: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "array_or_string")]
    foreground_color: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "gradient_direction")]
    gradient_direction: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "array_or_string")]
    color1: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "array_or_string")]
    color2: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "bool")]
    use_selected_skin: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "bool")]
    use_uuid: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "bool")]
    use_player_paperdoll: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "bool")]
    use_skin_gui_scale: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "number")]
    camera_tilt_degrees: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "rotation")]
    rotation: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "number")]
    starting_rotation: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "number")]
    rotation_x: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "number")]
    rotation_y: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "number")]
    hover_text_max_width: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "bool")]
    replaced_while_inactive: Option<Value>,
    #[facet(schema::read_by = "custom", schema::getter = "string")]
    end_event: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "any")]
    scrollbar_track_button: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "any")]
    scrollbar_touch_button: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "number")]
    scroll_speed: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "bool")]
    gesture_control_enabled: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "bool")]
    always_handle_scrolling: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "bool")]
    touch_mode: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "bool")]
    allow_scroll_even_when_content_fits: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "bool")]
    scrollbar_always_visible: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "any")]
    scrollbar_box: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "any")]
    scrollbar_track: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "any")]
    scroll_view_port: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "any")]
    scroll_content: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "any")]
    scroll_box_and_track_panel: Option<Value>,
    #[facet(schema::read_by = "scroll_view", schema::getter = "any")]
    jump_to_bottom_on_update: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "any")]
    slider_track_button: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "any")]
    slider_small_decrease_button: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "any")]
    slider_small_increase_button: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "any")]
    slider_selected_button: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "any")]
    slider_deselected_button: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "number")]
    slider_steps: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "orientation")]
    slider_direction: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "bool")]
    slider_inverted: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "number")]
    slider_speed: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "number")]
    slider_timeout: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "any")]
    slider_collection_name: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "any")]
    slider_name: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "bool")]
    slider_select_on_hover: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "any")]
    slider_box_control: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "any")]
    tts_value_changed: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "any")]
    background_control: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "any")]
    background_hover_control: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "any")]
    progress_control: Option<Value>,
    #[facet(schema::read_by = "slider", schema::getter = "any")]
    progress_hover_control: Option<Value>,
    #[facet(schema::read_by = "slider_box", schema::getter = "any")]
    indent_control: Option<Value>,
    #[facet(schema::read_by = "text_edit", schema::getter = "any")]
    text_box_name: Option<Value>,
    #[facet(schema::read_by = "text_edit", schema::getter = "any")]
    text_edit_box_grid_collection_name: Option<Value>,
    #[facet(schema::read_by = "text_edit", schema::getter = "bool")]
    constrain_to_rect: Option<Value>,
    #[facet(schema::read_by = "text_edit", schema::getter = "bool")]
    enabled_newline: Option<Value>,
    #[facet(schema::read_by = "text_edit", schema::getter = "any")]
    text_type: Option<Value>,
    #[facet(schema::read_by = "text_edit", schema::getter = "number")]
    max_length: Option<Value>,
    #[facet(schema::read_by = "text_edit", schema::getter = "any")]
    virtual_keyboard_buffer_control: Option<Value>,
    #[facet(schema::read_by = "text_edit", schema::getter = "any")]
    text_control: Option<Value>,
    #[facet(schema::read_by = "text_edit", schema::getter = "any")]
    place_holder_control: Option<Value>,
    #[facet(schema::read_by = "text_edit", schema::getter = "bool")]
    can_be_deselected: Option<Value>,
    #[facet(schema::read_by = "text_edit", schema::getter = "bool")]
    always_listening: Option<Value>,
    #[facet(schema::read_by = "text_edit", schema::getter = "array_or_string")]
    place_holder_text_hover_color: Option<Value>,
    #[facet(schema::read_by = "dropdown", schema::getter = "any")]
    dropdown_name: Option<Value>,
    #[facet(schema::read_by = "dropdown", schema::getter = "any")]
    dropdown_content_control: Option<Value>,
    #[facet(schema::read_by = "dropdown", schema::getter = "any")]
    dropdown_area: Option<Value>,
    #[facet(schema::read_by = "gesture", schema::getter = "any")]
    gesture_tracking_button: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "any")]
    tts_name: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "any")]
    tts_control_header: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "any")]
    tts_override_control_value: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "any")]
    tts_inherit_siblings: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "bool")]
    tts_message_interruptible: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "any")]
    tts_message_interruptible_id: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "any")]
    tts_index_order: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "number")]
    tts_index_priority: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "any")]
    tts_ignore_subsections: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "any")]
    tts_ignore_count: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "any")]
    tts_skip_message: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "any")]
    tts_skip_children: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "any")]
    tts_play_on_unchanged_focus_control: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "number")]
    tts_control_type_order_priority: Option<Value>,
    #[facet(schema::read_by = "text_to_speech", schema::getter = "number")]
    tts_value_order_priority: Option<Value>,
    #[facet(schema::read_by = "selection_wheel", schema::getter = "number")]
    slice_count: Option<Value>,
    #[facet(schema::read_by = "selection_wheel", schema::getter = "number")]
    inner_radius: Option<Value>,
    #[facet(schema::read_by = "selection_wheel", schema::getter = "number")]
    outer_radius: Option<Value>,
    #[facet(schema::read_by = "selection_wheel", schema::getter = "any")]
    state_controls: Option<Value>,
    #[facet(schema::read_by = "selection_wheel", schema::getter = "any")]
    analog_button_name: Option<Value>,
    #[facet(schema::read_by = "selection_wheel", schema::getter = "any")]
    hover_button_name: Option<Value>,
    #[facet(schema::read_by = "selection_wheel", schema::getter = "any")]
    select_button_name: Option<Value>,
    #[facet(schema::read_by = "selection_wheel", schema::getter = "any")]
    iterate_left_button_name: Option<Value>,
    #[facet(schema::read_by = "selection_wheel", schema::getter = "any")]
    iterate_right_button_name: Option<Value>,
    #[facet(schema::read_by = "selection_wheel", schema::getter = "number")]
    initial_button_slice: Option<Value>,
    // With the PageIndicatorManager both cyclers also get.
    #[facet(schema::read_by = "image_cycling", schema::getter = "any")]
    images: Option<Value>,
    #[facet(schema::read_by = "image_cycling text_cycling", schema::getter = "any")]
    next_sub_page_button_name: Option<Value>,
    #[facet(schema::read_by = "image_cycling text_cycling", schema::getter = "any")]
    prev_sub_page_button_name: Option<Value>,
    #[facet(schema::read_by = "image_cycling text_cycling", schema::getter = "any")]
    target_cycler_to_compare: Option<Value>,
    #[facet(schema::read_by = "text_cycling", schema::getter = "array")]
    text_labels: Option<Value>,
    #[facet(schema::read_by = "page_indicator_grid", schema::getter = "any")]
    cycler_manager_size_control_target: Option<Value>,
    #[facet(schema::read_by = "page_indicator_grid", schema::getter = "any")]
    grid_item_when_current: Option<Value>,
    #[facet(schema::read_by = "page_indicator_grid", schema::getter = "any")]
    grid_item_when_not_current: Option<Value>,
    #[facet(schema::read_by = "tooltip_trigger", schema::getter = "any")]
    tooltip_name: Option<Value>,
    #[facet(schema::read_by = "tooltip_trigger", schema::getter = "any")]
    tooltip_tts_value: Option<Value>,
    #[facet(schema::read_by = "tooltip_trigger", schema::getter = "any")]
    tooltip_top_content_control: Option<Value>,
    #[facet(schema::read_by = "tooltip_trigger", schema::getter = "any")]
    tooltip_bottom_content_control: Option<Value>,
    #[facet(schema::read_by = "tooltip_trigger", schema::getter = "any")]
    tooltip_area: Option<Value>,
    /// The members no component reads: variables, their defaults, the merge's `texture:<n>`.
    extra: UnknownMembers,
}
