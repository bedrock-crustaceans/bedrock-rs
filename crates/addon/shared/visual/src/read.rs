//! What the description's members read beyond their types: the schema's patterns for the names of
//! slots, variables, conditions and controllers, told as the older client's tree tells them
//! (`child '%s' not valid here.`, `Required child %s not found`), and the wording of every
//! wrong type, "unknown child schema option type.  Allowed types: …", by the member's place.
//!
//! The pattern findings are told after the read, as the schema validates what it has read: a
//! member that fails to read tells nothing of them. Each is a `schema::read_with` of the member
//! that holds the names.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, JsonPath, JsonPathSegment, Severity};
use bedrock_addon_core::id::{Id, Kind};
use bedrock_addon_core::json::Value;
#[cfg(test)]
use bedrock_addon_core::json::kit::Profile;
use bedrock_addon_core::json::kit::{Node, Pass, Read, Texts, WrongAt};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::{FromJson, read_each};
use bedrock_addon_shared_shapes::named::Name;
use indexmap::IndexMap;

use crate::{NamedCondition, codes};

/// The pattern of a slot name (`materials`, `textures`, `geometry`, …), as vanilla prints it.
const MAP_KEY: &str = "[a-zA-Z0-9_.-]+";
/// The pattern of a `scripts.variables` key, as vanilla prints it.
const VARIABLE_KEY: &str = "variable.[a-zA-Z_][a-zA-Z0-9_]*";

/// `"child '<key>' not valid here."` at `at` (the member itself).
pub fn not_valid_here(key: &str, at: JsonPath) -> Diagnostic {
    Diagnostic::of(core_codes::CHILD_NOT_VALID.with([key]))
        .severity(Severity::Error)
        .at(at)
}

/// `"Required child <child> not found"` at `at` (the object that lacks it).
pub fn required(child: &str, at: JsonPath) -> Diagnostic {
    Diagnostic::of(core_codes::REQUIRED_CHILD.with([child])).at(at)
}

/// Whether `key` fully matches `[a-zA-Z0-9_.-]+`, the pattern of every slot name.
pub fn is_map_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

/// Whether `key` fully matches `[a-zA-Z0-9_.:-]+`, the pattern of a conditional object's key.
pub fn is_condition_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b':' | b'-'))
}

/// Whether `key` fully matches `variable.[a-zA-Z_][a-zA-Z0-9_]*`; the `.` is the regex's
/// any-character.
fn is_variable_key(key: &str) -> bool {
    let Some(rest) = key.strip_prefix("variable") else {
        return false;
    };
    let mut chars = rest.chars();
    // Any one character, then an identifier.
    if chars.next().is_none() {
        return false;
    }
    let name = chars.as_str().as_bytes();
    matches!(name.first(), Some(b) if b.is_ascii_alphabetic() || *b == b'_')
        && name.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'_')
}

/// Tells `finding` after the read, where the schema's checks come.
fn after(read: &mut Read<'_>, finding: Diagnostic) {
    read.defer(Pass::Unknown, finding);
}

/// A map of slot names (`materials`, `textures`, `geometry`): every key fully matches
/// `[a-zA-Z0-9_.-]+`, and the map holds at least one member.
pub fn slot_map<V: FromJson>(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, V>> {
    names::<V>(node, read, true)
}

/// A map of names (`particle_effects`, `animations`, …): every key fully matches
/// `[a-zA-Z0-9_.-]+`; it may be empty.
pub fn name_map<V: FromJson>(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, V>> {
    names::<V>(node, read, false)
}

fn names<V: FromJson>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    required_child: bool,
) -> Option<IndexMap<String, V>> {
    let map = IndexMap::<String, V>::from_json(node, read)?;
    for key in map.keys().filter(|key| !is_map_key(key)) {
        after(read, not_valid_here(key, node.path().child(key)));
    }
    if required_child && map.is_empty() {
        after(read, required(MAP_KEY, node.path().clone()));
    }
    Some(map)
}

/// `scripts.variables`: every key fully matches `variable.[a-zA-Z_][a-zA-Z0-9_]*`, the object
/// holds at least one, and every value is exactly `public`.
pub fn variables(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, String>> {
    let map = IndexMap::<String, String>::from_json(node, read)?;
    for (key, value) in &map {
        let at = node.path().child(key);
        if !is_variable_key(key) {
            after(read, not_valid_here(key, at));
        } else if value != "public" {
            let code = core_codes::NOT_ALLOWED_REGEX.with([value.as_str(), "public"]);
            after(read, Diagnostic::of(code).at(at));
        }
    }
    if map.is_empty() {
        after(read, required(VARIABLE_KEY, node.path().clone()));
    }
    Some(map)
}

/// A list of `render_controllers` or `scripts.animate` entries: the keys of a conditional
/// object fully match `[a-zA-Z0-9_.:-]+`, and the members after the one vanilla reads are
/// dropped with an information. An empty object passes the schema; the loader's rules warn of
/// it (`validate::empty_conditions`).
pub fn conditions<N: Name>(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<NamedCondition<N>>> {
    let list = Vec::<NamedCondition<N>>::from_json(node, read)?;
    for (index, entry) in list.iter().enumerate() {
        let NamedCondition::Conditional(members) = entry else {
            continue;
        };
        let at = node.path().index(index);
        for key in members.keys().filter(|key| !is_condition_key(key)) {
            after(read, not_valid_here(key, at.child(key)));
        }
        // Members after the one the game reads get a `DroppedField` information of the crate's
        // own; the game drops them without a message.
        for key in entry.ignored() {
            after(
                read,
                Diagnostic::of(&core_codes::DROPPED_FIELD).at(at.child(key)),
            );
        }
    }
    Some(list)
}

/// `animation_controllers` of format versions below 1.10.0: a list of `{ name: controller }`
/// objects whose names fully match `[a-zA-Z0-9_.-]+`.
pub fn animation_controllers(
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<Vec<IndexMap<String, String>>> {
    let list = Vec::<IndexMap<String, String>>::from_json(node, read)?;
    for (index, element) in list.iter().enumerate() {
        let at = node.path().index(index);
        for key in element.keys().filter(|key| !is_map_key(key)) {
            after(read, not_valid_here(key, at.child(key)));
        }
    }
    Some(list)
}

/// `initialize` and `pre_animation` below 1.26.0: a list of Molang values. An element object that
/// lacks a member of the object form (`expression`, `version`) is the older tree's "Required
/// child <list> not found", told at the list: the tree's path cut names the list, not the member.
pub fn molang_elements(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Molang>> {
    read_each(node, read, |element, read| {
        let lacks = element.value().as_object().is_some_and(|object| {
            object.get("expression").is_none() || object.get("version").is_none()
        });
        if lacks {
            let list = node.path().last_key().unwrap_or_default();
            read.refuse(node, core_codes::REQUIRED_CHILD.with([list]));
            return None;
        }
        Molang::from_json(element, read)
    })
}

/// An identifier as the description reads it: a text.
pub trait Named {
    /// The text as written.
    fn text(&self) -> &str;
    /// The identifier of `text`.
    fn named(text: String) -> Self;
}

impl<K: Kind> Named for Id<K> {
    fn text(&self) -> &str {
        self.as_str()
    }

    fn named(text: String) -> Self {
        Self::parse(text)
    }
}

/// `identifier`: a text, which names nothing when it is empty.
pub fn identifier<I: Named + FromJson>(node: &Node<'_>, read: &mut Read<'_>) -> Option<I> {
    let id = I::from_json(node, read)?;
    if id.text().is_empty() {
        after(
            read,
            Diagnostic::of(&codes::MISSING_IDENTIFIER).at(node.path().clone()),
        );
    }
    Some(id)
}

/// `min_engine_version` is a string or an array; any other JSON type fails the schema. A string
/// that is no version passes (the loader reads 0.0.0).
pub fn min_engine_version(node: &Node<'_>, read: &mut Read<'_>) -> Option<Value> {
    if !node.value().is_string() && !node.value().is_array() {
        read.wrong_type(node);
        return None;
    }
    Value::from_json(node, read)
}

/// The JSON types vanilla's schema option of a `description` member takes, by the member's
/// name, as "Allowed types: …" lists them. The game's text is a template over the schema's
/// own type list; the words of each list here are the crate's.
fn allowed_types(member: Option<&str>, in_array: bool) -> &'static str {
    match member {
        Some("min_engine_version") => "string, array",
        Some("texture_index") => "int",
        Some("enable_attachables" | "held_item_ignores_lighting" | "hide_armor") => "bool",
        Some("held_item_scale") => "float",
        Some("render_controllers" | "animate") if in_array => "string, object",
        Some("animation_controllers") if in_array => "object",
        Some("initialize" | "pre_animation") if in_array => "string",
        Some(
            "render_controllers"
            | "animate"
            | "animation_controllers"
            | "initialize"
            | "pre_animation",
        ) => "array",
        Some(
            "scale"
            | "scaleX"
            | "scalex"
            | "scaleY"
            | "scaley"
            | "scaleZ"
            | "scalez"
            | "parent_setup"
            | "should_update_bones_and_effects_offscreen"
            | "should_update_effects_offscreen"
            | "hide_held_items",
        ) => "molang",
        Some(
            "description" | "materials" | "textures" | "geometry" | "spawn_egg" | "scripts"
            | "particle_effects" | "particle_emitters" | "animations" | "sound_effects"
            | "variables",
        ) => "object",
        // `identifier`, `queryable_geometry`, the members of `spawn_egg` that are text, and a
        // value inside one of the string maps.
        _ => "string",
    }
}

/// The wording of a wrong type at `path` (from the document's root): the types the schema
/// option of that member takes. `BY_PLACE` words a condition below a list entry as the place it
/// is (Molang); without it the condition is worded by its key, as any member is. `NAMES_ONLY` is
/// the 1.8 shape, whose `render_controllers` holds names only.
pub fn wording<const BY_PLACE: bool, const NAMES_ONLY: bool>(path: &JsonPath) -> Code {
    // The root, the payload and its `description` are objects.
    if path.len() <= 2 {
        return bedrock_addon_core::diagnostic::cereal::unknown_child_option("object");
    }
    // The path below the payload's `description`.
    let segments: Vec<JsonPathSegment<'_>> = path.segments().skip(2).collect();
    let mut last = (None, false);
    for segment in segments.iter().rev() {
        match segment {
            JsonPathSegment::Key(key) => {
                last = (Some(*key), last.1);
                break;
            }
            JsonPathSegment::Index(_) => last.1 = true,
        }
    }
    let below_element = segments
        .windows(2)
        .any(|pair| matches!(pair, [JsonPathSegment::Index(_), JsonPathSegment::Key(_)]));
    let allowed = if BY_PLACE && below_element {
        // 1.8.x `animation_controllers[]` maps a name to a controller's name.
        if segments.first() == Some(&JsonPathSegment::Key("animation_controllers")) {
            "string"
        } else {
            "molang"
        }
    } else {
        match last {
            // 1.8.x: `render_controllers` is an array of strings.
            (Some("render_controllers"), true) if NAMES_ONLY => "string",
            (member, in_array) => allowed_types(member, in_array),
        }
    };
    bedrock_addon_core::diagnostic::cereal::unknown_child_option(allowed)
}

/// The words of the older client's schema tree for the description: a member missing, a member
/// the schema does not declare (an error, whatever the mask says), a wrong type by its place.
pub const fn texts(wording: fn(&JsonPath) -> Code) -> Texts {
    Texts {
        missing_named: Some(&core_codes::REQUIRED_CHILD),
        unknown_named: Some(&core_codes::CHILD_NOT_VALID),
        missing_in_object: true,
        wrong_at: Some(WrongAt(wording)),
        ..Texts::CODES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_names_match_their_pattern_in_full() {
        assert!(is_map_key("default"));
        assert!(is_map_key("a.b-c_D9"));
        assert!(!is_map_key("my:tex"));
        assert!(!is_map_key(""));
        assert!(!is_map_key("a b"));
    }

    #[test]
    fn condition_keys_may_hold_a_colon() {
        assert!(is_condition_key("controller.render.a"));
        assert!(is_condition_key("demo:walk"));
        assert!(!is_condition_key("a b"));
        assert!(!is_condition_key(""));
    }

    #[test]
    fn variable_keys_take_any_character_after_variable() {
        assert!(is_variable_key("variable.foo"));
        assert!(is_variable_key("variable_foo"));
        assert!(is_variable_key("variable._a1"));
        assert!(!is_variable_key("variable."));
        assert!(!is_variable_key("variable"));
        assert!(!is_variable_key("variable.1a"));
        assert!(!is_variable_key("variable.a-b"));
        assert!(!is_variable_key("v.foo"));
    }
}

#[cfg(test)]
mod cost {
    use bedrock_addon_core::diagnostic::Diagnostics;
    use bedrock_addon_core::json::from_str_jsonc;
    use bedrock_addon_core::schema::read_document;
    use bedrock_addon_core::test_cost::{assert_linear_timed_at, timed};
    use bedrock_addon_core::versioning::Reading;

    use super::*;
    use crate::VisualSlots;

    /// Reads `text` as the slots of a description, as the families read it.
    fn slots(text: &str, found: &mut Diagnostics) -> Option<VisualSlots> {
        let json: Value = from_str_jsonc(text).expect("test JSON");
        let mut read = Read::new(Reading::DOCUMENT, found).profile(Profile::FIRST_THEN_PASSES);
        read_document(&Node::new(&json, JsonPath::new()), &mut read)
    }

    #[test]
    fn the_pattern_of_every_name_is_checked_in_linear_time() {
        // `n` slot names outside their pattern, each in a map of its own member kind.
        assert_linear_timed_at("visual names", 20_000, |n| {
            let names: Vec<String> = (0..n).map(|i| format!(r#""k{i} x": "m""#)).collect();
            let text = format!(r#"{{"materials": {{{}}}}}"#, names.join(", "));
            let mut found = Diagnostics::default();
            let (read, took) = timed(|| slots(&text, &mut found));
            assert_eq!(read.map(|slots| slots.materials.len()), Some(n));
            assert_eq!(found.len(), n);
            took
        });
    }
}
