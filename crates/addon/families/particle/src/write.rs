//! Writing a particle effect: the payload object, and the document with its envelope. Only
//! members the value holds are written, in the form they were read.

use facet_value::VObject;
use indexmap::IndexMap;

use super::component::{EmitterLifetimeEvents, ParticleMotionCollision, write_component};
use super::curve::Curve;
use super::event::EventNode;
use super::read::{MAX_TEXTURE, known};
use super::read::{Writer, put, put_opt};
use super::{Description, ParticleEffect};
use bedrock_addon_core::document::Component as _;
use bedrock_addon_core::document::{ComponentRef, Extra};
use bedrock_addon_core::json::{EncodeError, Unrepresentable, Value};

/// The key of the payload.
pub(crate) const PAYLOAD_KEY: &str = "particle_effect";

/// The payload object from its parts: `description`, `components`, `curves`, `events`, then
/// the unknown members.
pub(crate) fn payload_of(
    description: &Description,
    components: VObject,
    curves: Option<&IndexMap<String, Curve>>,
    events: Option<&IndexMap<String, EventNode>>,
    extra: &Extra,
    w: &mut Writer,
) -> Result<Value, EncodeError> {
    let mut out = VObject::new();
    put(&mut out, "description", description, w)?;
    out.insert("components", components);
    put_opt(&mut out, "curves", curves, w)?;
    put_opt(&mut out, "events", events, w)?;
    w.extra(&mut out, extra, known)?;
    Ok(out.into())
}

/// The refusal of the member `steps` lead to, and from there of `index`'s member when given.
fn refuse_at(w: &mut Writer, steps: &[&str], index: Option<(usize, &str)>) -> EncodeError {
    match (steps.split_first(), index) {
        (Some((step, rest)), _) => w.key(step, |w| refuse_at(w, rest, index)),
        (None, Some((i, key))) => w.index(i, |w| w.key(key, |w| refuse_at(w, &[], None))),
        (None, None) => w.refuse(Unrepresentable::Unsupported),
    }
}

/// The constraints the 1.20.80 reader refuses a document for and a caller can break: the
/// length of `texture`, the range of `collision_radius`, a negative `distance`. The first one
/// broken is the error, at its path.
fn constraints(effect: &ParticleEffect, w: &mut Writer) -> Result<(), EncodeError> {
    let refuse = refuse_at;
    let texture = &effect.description.basic_render_parameters.texture;
    if texture.chars().count() > MAX_TEXTURE {
        let at = ["description", "basic_render_parameters", "texture"];
        return Err(refuse(w, &at, None));
    }
    if let Some(collision) = effect.components.get::<ParticleMotionCollision>()
        && !(0.0..=0.5).contains(&collision.collision_radius)
        && !collision.collision_radius.is_nan()
    {
        let at = [
            "components",
            ParticleMotionCollision::NAME,
            "collision_radius",
        ];
        return Err(refuse(w, &at, None));
    }
    if let Some(events) = effect.components.get::<EmitterLifetimeEvents>() {
        let entries = events.looping_travel_distance_events.iter().flatten();
        for (i, entry) in entries.enumerate() {
            if entry.distance < 0.0 {
                let at = [
                    "components",
                    EmitterLifetimeEvents::NAME,
                    "looping_travel_distance_events",
                ];
                return Err(refuse(w, &at, Some((i, "distance"))));
            }
        }
    }
    Ok(())
}

/// The payload of the newest struct.
pub(crate) fn payload(effect: &ParticleEffect, w: &mut Writer) -> Result<Value, EncodeError> {
    constraints(effect, w)?;
    let components = w.key("components", |w| {
        let mut out = VObject::new();
        for entry in effect.components.iter() {
            match entry {
                ComponentRef::Typed { name, value, .. } => {
                    let json = w.key(name, |w| write_component(value, w))?;
                    out.insert(name, json);
                }
                // An unknown component refuses a 1.20.80 document, so it has no place in the
                // text. Only a value built by hand holds one.
                ComponentRef::Custom(custom) => {
                    let name = custom.name.as_str();
                    return Err(w.key(name, |w| w.refuse(Unrepresentable::Unsupported)));
                }
                ComponentRef::Dynamic { name, .. } | ComponentRef::Unknown { name, .. } => {
                    return Err(w.key(name, |w| w.refuse(Unrepresentable::Unsupported)));
                }
                _ => {}
            }
        }
        Ok(out)
    })?;
    payload_of(
        &effect.description,
        components,
        effect.curves.as_ref(),
        effect.events.as_ref(),
        &effect.extra,
        w,
    )
}

/// The document around a payload: `format_version`, then the payload. The 1.10.0 struct's
/// text is the whole document (the raw-text path, below the envelope `Family::ENVELOPE`
/// checks), so it writes its envelope itself; the newest struct's goes through core's
/// `write_envelope`.
pub(crate) fn document(version: &str, payload: Value) -> Value {
    let mut root = VObject::new();
    root.insert("format_version", Value::from(version));
    root.insert(PAYLOAD_KEY, payload);
    root.into()
}

/// The payload of the newest struct, as a 1.20.80 document holds it under its key: every
/// refusal is at its path in the document, below the key.
///
/// `description` and `components` are always written (the 1.20.80 reader requires them),
/// every other member only where the value has it. An effect that was read partially from a
/// file below 1.20.80, because the legacy parser stopped in it, is written whole at its
/// defaults: `basic_render_parameters` with an empty `material` or `texture` and an empty
/// `components` are written although the file had none, and what the parser never reached
/// (components, curves, events after the stop) is not. The text loads again to an equal
/// value, without the warning the first load gave, since the written file has the members
/// that warning was about.
///
/// A value the loader would not read back has no text: a number that is not finite, a
/// Molang bool, an unknown member or component (only a value built by hand holds one: the
/// reader keeps none; the 1.20.80 reader refuses them), events nested deeper than a file may
/// be.
pub(crate) fn document_payload(effect: &ParticleEffect) -> Result<Value, EncodeError> {
    let mut w = Writer::default();
    // No depth check on the payload: nothing in it nests without bound but the events, and
    // `EventNode::write_node` refuses a node at the reader's depth limit. Every other member
    // has a fixed depth, and an unknown member or component (which could nest as it likes) is
    // refused at its key, since the strict 1.20.80 reader refuses it.
    w.key(PAYLOAD_KEY, |w| payload(effect, w))
}
