//! `ui/_global_variables.json`: the `$name` values every screen sees, merged across the stack
//! variables the game supplies itself.

use indexmap::IndexMap;

use super::merge::CopyContent;
use super::value::{number, sorted_members};
use bedrock_addon_core::json::Value;

/// The colour variables the game reads from the global variables, in the order of its colour
/// table: the sixteen `§` format colours, the coin colour and the eleven material colours.
// The game hands these 28 colours to its text colour table when the global variables load.
pub const COLOR_VARIABLES: [&str; 28] = [
    "$0_color_format",
    "$1_color_format",
    "$2_color_format",
    "$3_color_format",
    "$4_color_format",
    "$5_color_format",
    "$6_color_format",
    "$7_color_format",
    "$8_color_format",
    "$9_color_format",
    "$a_color_format",
    "$b_color_format",
    "$c_color_format",
    "$d_color_format",
    "$e_color_format",
    "$f_color_format",
    "$coin_color",
    "$material_quartz_color",
    "$material_iron_color",
    "$material_netherite_color",
    "$material_redstone_color",
    "$material_copper_color",
    "$material_gold_color",
    "$material_emerald_color",
    "$material_diamond_color",
    "$material_lapis_color",
    "$material_amethyst_color",
    "$material_resin_color",
];

/// The platform variables the game sets itself when it creates a screen; a `requires`
/// condition may use them without any file defining them.
// The game sets these 88 names when it creates a screen.
pub const ENGINE_VARIABLES: [&str; 88] = [
    "$apple_os",
    "$beta_build",
    "$bottom_vertical_safezone_size",
    "$build_platform_UWP",
    "$built_with_ore_ui_docs_and_tests",
    "$can_quit",
    "$can_splitscreen",
    "$console_edition",
    "$creator_build",
    "$desktop_screen",
    "$device_must_be_removed_for_xbl_signin",
    "$disable_gamertag_controls",
    "$edu_save_to_cloud_general_toggle_on",
    "$edu_save_to_cloud_on",
    "$education_edition",
    "$file_picking_supported",
    "$fire_tv",
    "$game_pad",
    "$gear_vr",
    "$google_os",
    "$hide_xbox_live_icon",
    "$ignore_3rd_party_servers",
    "$ignore_add_servers",
    "$image_picking_not_supported",
    "$is_android",
    "$is_chromebook",
    "$is_console",
    "$is_creative_enabled",
    "$is_desktop",
    "$is_dynamic_textures_platform_supported",
    "$is_editor_mode_enabled",
    "$is_editor_playtest_roundtrip",
    "$is_holographic",
    "$is_improve_input_response_platform_supported",
    "$is_ios",
    "$is_living_room_mode",
    "$is_low_memory_device",
    "$is_mobile_vr",
    "$is_multiplayer_enabled",
    "$is_on_3p_server",
    "$is_packs_enabled",
    "$is_pregame",
    "$is_preview_app",
    "$is_ps4",
    "$is_ps5",
    "$is_publish",
    "$is_reality_mode",
    "$is_realms_enabled",
    "$is_secondary_client",
    "$is_seeds_enabled",
    "$is_server_enabled",
    "$is_settopbox",
    "$is_store_enabled",
    "$is_win10_arm",
    "$is_windows_10_mobile",
    "$is_xboxlive_enabled",
    "$left_horizontal_safezone_size",
    "$microsoft_os",
    "$mouse",
    "$ms_platform",
    "$multiplayer_requires_live_gold",
    "$new_video_settings",
    "$nx_os",
    "$oculus_rift",
    "$osx_edition",
    "$pocket_edition",
    "$pocket_screen",
    "$pre_release",
    "$psvr",
    "$requires_xbl_signin_to_play",
    "$right_horizontal_safezone_size",
    "$screen_transitions_enabled",
    "$store_disabled",
    "$supports_add_friend",
    "$supports_clipboard_set",
    "$supports_hand_controllers",
    "$supports_share",
    "$supports_xbl_achievements",
    "$test_infrastructure_disabled",
    "$thirdpartyconsole",
    "$top_vertical_safezone_size",
    "$touch",
    "$trial",
    "$use_normalized_font_size",
    "$vibration_supported",
    "$win10_edition",
    "$world_archive_support",
    "$xbox_one",
];

/// The global variables (`ui/_global_variables.json`): `$name` to value.
///
/// JSON: `{"$name": value, …}`. Across the stack, objects merge member by member and anything
/// else, arrays included, is replaced by the higher pack's value.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "_global_variables.json")]
pub struct GlobalVariables {
    /// The variables in file order, by name with the `$`.
    pub variables: IndexMap<String, Value>,
}

impl GlobalVariables {
    /// The value of `name` (with its `$`).
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// The colour of variable `name` as `[r, g, b, a]`, when it holds one: an array of three
    /// or four numbers (alpha 1 when left out) or a colour name.
    pub fn color(&self, name: &str) -> Option<[f32; 4]> {
        self.get(name).and_then(color_of)
    }

    /// The colour table the game fills from the variables: each of [`COLOR_VARIABLES`] the
    /// stack sets to a colour.
    pub fn color_format(&self) -> impl Iterator<Item = (&'static str, [f32; 4])> + '_ {
        COLOR_VARIABLES
            .iter()
            .filter_map(|name| Some((*name, self.color(name)?)))
    }

    /// The variables of a root value: its members when it is an object.
    pub(crate) fn from_value(root: &Value) -> Self {
        let variables = root
            .as_object()
            .map(|object| {
                object
                    .iter()
                    .map(|(name, value)| (name.as_str().to_owned(), value.clone()))
                    .collect()
            })
            .unwrap_or_default();
        Self { variables }
    }
}

/// The colour names `getAsColor` knows.
const COLOR_NAMES: [(&str, [f32; 4]); 12] = [
    ("white", [1.0, 1.0, 1.0, 1.0]),
    ("black", [0.0, 0.0, 0.0, 1.0]),
    ("grey", [0.5, 0.5, 0.5, 1.0]),
    ("gray", [0.5, 0.5, 0.5, 1.0]),
    ("red", [1.0, 0.0, 0.0, 1.0]),
    ("green", [0.0, 1.0, 0.0, 1.0]),
    ("blue", [0.0, 0.0, 1.0, 1.0]),
    ("yellow", [1.0, 1.0, 0.0, 1.0]),
    ("orange", [1.0, 0.5, 0.0, 1.0]),
    ("purple", [1.0, 0.0, 1.0, 1.0]),
    ("cyan", [0.0, 1.0, 1.0, 1.0]),
    ("nil", [0.0, 0.0, 0.0, 0.0]),
];

/// A colour value: `[r, g, b]`, `[r, g, b, a]` or a colour name.
pub(crate) fn color_of(value: &Value) -> Option<[f32; 4]> {
    if let Some(name) = value.as_string() {
        return COLOR_NAMES
            .iter()
            .find(|(known, _)| *known == name.as_str())
            .map(|(_, color)| *color);
    }
    let parts = value.as_array()?.as_slice();
    if !(3..=4).contains(&parts.len()) {
        return None;
    }
    let mut color = [0.0, 0.0, 0.0, 1.0];
    for (slot, part) in color.iter_mut().zip(parts) {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "a colour channel is an f32"
        )]
        {
            *slot = number(part)? as f32;
        }
    }
    Some(color)
}

/// What the merge of the global variables has to say about one copy.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GlobalEvent {
    /// The lowest copy does not parse; no pack's variables load.
    BaseMalformed(String),
    /// A higher copy does not parse and is skipped.
    OverlayMalformed(String),
}

/// Merges the copies of `ui/_global_variables.json`, lowest priority first: the merged root,
/// the slots that are part of it and what was raised against which slot.
// The game merges the copies with its generic JSON merge: objects member by member, anything
// else replaced by the higher copy.
pub(crate) fn merge_globals(
    copies: Vec<(usize, CopyContent)>,
) -> (Value, Vec<usize>, Vec<(usize, GlobalEvent)>) {
    let mut root = Value::NULL;
    let mut merged = Vec::new();
    let mut raised = Vec::new();
    for (index, (slot, content)) in copies.into_iter().enumerate() {
        let json = match content {
            CopyContent::Json(json) => json,
            CopyContent::Empty | CopyContent::Malformed(_) if index == 0 => {
                let message = match content {
                    CopyContent::Malformed(message) => message,
                    _ => "empty document".to_owned(),
                };
                raised.push((slot, GlobalEvent::BaseMalformed(message)));
                // Nothing merges: the globals stay empty.
                return (Value::NULL, Vec::new(), raised);
            }
            CopyContent::Empty => continue,
            CopyContent::Malformed(message) => {
                raised.push((slot, GlobalEvent::OverlayMalformed(message)));
                continue;
            }
        };
        if index == 0 {
            root = json;
        } else {
            recursive_merge(&mut root, &json);
        }
        merged.push(slot);
    }
    (root, merged, raised)
}

/// Both objects: member by member; anything else: `src` replaces `dest`.
fn recursive_merge(dest: &mut Value, src: &Value) {
    match dest.as_object_mut() {
        Some(object) if src.is_object() => {
            for (name, value) in sorted_members(src) {
                match object.get_mut(name) {
                    Some(existing) => recursive_merge(existing, value),
                    None => {
                        object.insert(name, value.clone());
                    }
                }
            }
        }
        _ => *dest = src.clone(),
    }
}
