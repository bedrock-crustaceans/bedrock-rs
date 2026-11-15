//! Reading an atomic client entity as the game's cereal parse does under its strict error mask:
//! member by member by core's reader (`versioning::read_payload`), with every unknown member,
//! wrong type and failed constraint an error.
//!
//! What is the family's own: the floats with a minimum of 0 (`schema::read_with = finite`: a
//! number beyond `f32` is the largest finite float, told), and, once the entity is read, the
//! rules of the game's validators: a namespace in the identifier, no condition on a render
//! controller, no float below 0.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::json::kit::{Node, Order, Read};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::beyond_f32_warning;
use bedrock_addon_core::schema::FromJson;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BETA_VERSION, ParseCx, VersionStruct, Versioned, read_payload_with, write_payload,
};
use bedrock_addon_shared_visual::ControllerUse;

use super::{AtomicClientEntity, codes};

const AABB: &str = "minecraft:aabb_dimensions";
const CLIPPING: &str = "minecraft:companion_block_clipping";

/// A float member that has a minimum of 0 (`schema::read_with`): a number beyond `f32` is the
/// largest finite float of its sign, told as an `InvalidValue` information at it; any other
/// value as an `f32` reads. The offsets of `companion_block_avoidance` are elements, not such
/// members: a number beyond `f32` there is a type error.
pub(crate) fn finite(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    #[allow(clippy::cast_possible_truncation)] // the nearest f32 is the question
    let beyond = node
        .value()
        .as_number()
        .map(|number| number.to_f64_lossy() as f32)
        .filter(|nearest| nearest.is_infinite());
    match beyond {
        Some(nearest) if !read.stopped() => {
            read.push(beyond_f32_warning(node.path().clone(), node.span()));
            Some(f32::MAX.copysign(nearest))
        }
        _ => f32::from_json(node, read),
    }
}

impl Versioned for AtomicClientEntity {
    const VERSION: FormatVersion = BETA_VERSION;
}

impl VersionStruct for AtomicClientEntity {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        // The strict error mask applies: every cereal entry is fatal.
        let entity: Self = read_payload_with(payload, cx, d, |read| read.order(Order::AsRead))?;
        let description_at = cx.at.child("description");
        if !has_namespace(entity.description.identifier.as_str()) {
            // A namespace is required: the identifier matches `^(?:.)+:(?:.)+$`.
            d.push(Diagnostic::of(&codes::NAMESPACE_TEXT).at(description_at.child("identifier")));
        }
        let controllers = entity.description.render_controllers.iter().flatten();
        for (index, controller) in controllers.enumerate() {
            if matches!(controller, ControllerUse::Conditional(_)) {
                // The list holds plain strings; the `{ name: condition }` form is no string.
                let at = description_at.child("render_controllers").index(index);
                d.push(Diagnostic::of(&core_codes::INVALID_TYPE).at(at));
            }
        }
        for (members, value) in not_negative(&entity) {
            // The minimum is 0.0, inclusive.
            if value < 0.0 {
                d.push(Diagnostic::of(&codes::NOT_NEGATIVE_TEXT).at(cx.at.with(&members)));
            }
        }
        Some(entity)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

/// Every float the entity writes that has a minimum of 0, with its members below the payload.
fn not_negative(entity: &AtomicClientEntity) -> Vec<(Vec<&'static str>, f32)> {
    let mut found = Vec::new();
    let Some(components) = &entity.components else {
        return found;
    };
    if let Some(aabb) = &components.aabb_dimensions {
        found.push((vec!["components", AABB, "width"], aabb.width));
        found.push((vec!["components", AABB, "height"], aabb.height));
    }
    if let Some(clipping) = &components.companion_block_clipping {
        for (name, value) in [
            ("min_hidden_seconds", clipping.min_hidden_seconds),
            ("min_visible_seconds", clipping.min_visible_seconds),
            ("hide_delay_seconds", clipping.hide_delay_seconds),
            ("hide_transition_seconds", clipping.hide_transition_seconds),
        ] {
            if let Some(value) = value {
                found.push((vec!["components", CLIPPING, name], value));
            }
        }
        for (boundary, written) in [
            ("reveal_boundary", clipping.reveal_boundary),
            ("hide_boundary", clipping.hide_boundary),
        ] {
            let Some(written) = written else { continue };
            for (name, value) in [
                ("margin", written.margin),
                ("lookahead_per_speed", written.lookahead_per_speed),
                ("lookahead_max", written.lookahead_max),
            ] {
                if let Some(value) = value {
                    found.push((vec!["components", CLIPPING, boundary, name], value));
                }
            }
        }
    }
    found
}

/// Whether `identifier` matches `^(?:.)+:(?:.)+$`: a `:` with at least one character on each
/// side.
pub(super) fn has_namespace(identifier: &str) -> bool {
    let count = identifier.chars().count();
    identifier
        .chars()
        .enumerate()
        .any(|(index, c)| c == ':' && index >= 1 && index + 1 < count)
}

#[cfg(test)]
mod tests {
    use super::has_namespace;

    #[test]
    fn a_namespace_needs_a_character_on_each_side_of_a_colon() {
        assert!(has_namespace("demo:wisp"));
        assert!(has_namespace("minecraft:wisp"));
        assert!(has_namespace("a:b:c"));
        assert!(has_namespace(":a:b"));
        assert!(!has_namespace("wisp"));
        assert!(!has_namespace(":wisp"));
        assert!(!has_namespace("demo:"));
        assert!(!has_namespace(":"));
        assert!(!has_namespace(""));
    }
}
