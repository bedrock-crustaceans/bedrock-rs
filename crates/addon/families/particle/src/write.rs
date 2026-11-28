//! Writing a particle effect: the payload object, and the document with its envelope. Only
//! members the value holds are written, in the form they were read. Every part is written by
//! the generic writer ([`schema::write_value_with`]) under the era's [`Writing`]; this module
//! assembles the payload around the component map and holds the rules the Shape cannot say.

use facet_value::VObject;
use indexmap::IndexMap;

use super::component::{EmitterLifetimeEvents, ParticleMotionCollision, write_component};
use super::curve::Curve;
use super::event::EventNode;
use super::read::{MAX_TEXTURE, known};
use super::{Description, ParticleEffect};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::Component as _;
use bedrock_addon_core::document::{ComponentRef, UnknownMembers};
use bedrock_addon_core::json::{EncodeError, MAX_JSON_DEPTH, Unrepresentable, Value};
use bedrock_addon_core::schema::{self, Unknowns, Writing};

/// The key of the payload.
pub(crate) const PAYLOAD_KEY: &str = "particle_effect";

/// The rules a document of the era is written by: a legacy document takes a Molang bool and an
/// unknown member not named like a member of its struct, a 1.20.80 document neither, since
/// its strict reader would not read them back.
fn writing(legacy: bool) -> Writing {
    if legacy {
        Writing::REAL.unknown(Unknowns::NoneNamedKnown)
    } else {
        Writing::REAL
            .unknown(Unknowns::Refused)
            .refusing_molang_bools()
    }
}

/// The JSON of a part of the document at `at`, by the generic writer under the era's rules; a
/// refusal is at its path in the document.
pub(crate) fn json<'f, T: facet::Facet<'f>>(
    value: &T,
    legacy: bool,
    at: &JsonPath,
) -> Result<Value, EncodeError> {
    schema::write_value_with(value, writing(legacy)).map_err(|error| match error {
        EncodeError::NotRepresentable { at: below, what } => EncodeError::NotRepresentable {
            at: at.join(&below),
            what,
        },
        other => other,
    })
}

/// The refusal of the value at `at`.
fn refuse(at: JsonPath) -> EncodeError {
    EncodeError::NotRepresentable {
        at,
        what: Unrepresentable::Unsupported,
    }
}

/// The payload object from its parts: `description`, `components`, `curves`, `events`, then
/// the unknown members, which only a legacy document has a place for, and there only one not
/// named like a member of the payload.
pub(crate) fn payload_of(
    description: &Description,
    components: VObject,
    curves: Option<&IndexMap<String, Curve>>,
    events: Option<&IndexMap<String, EventNode>>,
    extra: &UnknownMembers,
    legacy: bool,
) -> Result<Value, EncodeError> {
    let at = JsonPath::new().child(PAYLOAD_KEY);
    let mut out = VObject::new();
    let description = json(description, legacy, &at.child("description"))?;
    out.insert("description", description);
    out.insert("components", components);
    if let Some(curves) = curves {
        out.insert("curves", write_curves(curves, legacy, &at.child("curves"))?);
    }
    if let Some(events) = events {
        let at = at.child("events");
        if let Some(deep) = too_deep(events, &at) {
            return Err(refuse(deep));
        }
        out.insert("events", json(events, legacy, &at)?);
    }
    for (key, value) in extra.iter() {
        if !legacy || known(key) {
            return Err(refuse(at.child(key)));
        }
        out.insert(key, value.clone());
    }
    Ok(out.into())
}

/// `curves`: a `bezier_chain` curve's `type` is no field of its struct but the variant it is,
/// written before its members.
fn write_curves(
    curves: &IndexMap<String, Curve>,
    legacy: bool,
    at: &JsonPath,
) -> Result<Value, EncodeError> {
    let mut out = VObject::new();
    for (name, curve) in curves {
        let at = at.child(name);
        let written = match curve {
            Curve::Simple(curve) => json(curve, legacy, &at)?,
            Curve::BezierChain(curve) => {
                if curve.extra.get("type").is_some() {
                    return Err(refuse(at.child("type")));
                }
                let mut chain = VObject::new();
                chain.insert("type", Value::from("bezier_chain"));
                let members = json(curve, legacy, &at)?;
                for (key, value) in members.as_object().into_iter().flat_map(VObject::iter) {
                    chain.insert(key.as_str(), value.clone());
                }
                chain.into()
            }
        };
        out.insert(name.as_str(), written);
    }
    Ok(out.into())
}

/// The path of the first node of `events`, in the order they are written, that is as deep as
/// no file the loader reads may be: a chain built in memory may be of any length, and the walk
/// down to it does not end on one built without end.
fn too_deep(events: &IndexMap<String, EventNode>, at: &JsonPath) -> Option<JsonPath> {
    let mut pending: Vec<(&EventNode, JsonPath)> = events
        .iter()
        .rev()
        .map(|(name, node)| (node, at.child(name)))
        .collect();
    while let Some((node, path)) = pending.pop() {
        if path.len() >= MAX_JSON_DEPTH {
            return Some(path);
        }
        let sequence = node.sequence.iter().flatten().enumerate();
        let sequence = sequence.map(|(i, node)| (node, path.child("sequence").index(i)));
        let options = node.randomize.iter().flatten().enumerate();
        let options = options.map(|(i, option)| (&option.node, path.child("randomize").index(i)));
        let below: Vec<_> = sequence.chain(options).collect();
        pending.extend(below.into_iter().rev());
    }
    None
}

/// The constraints the 1.20.80 reader refuses a document for and a caller can break: the
/// length of `texture`, the range of `collision_radius`, a negative `distance`. The first one
/// broken is the error, at its path.
fn constraints(effect: &ParticleEffect, at: &JsonPath) -> Result<(), EncodeError> {
    let texture = &effect.description.basic_render_parameters.texture;
    if texture.chars().count() > MAX_TEXTURE {
        let at = at.child("description").child("basic_render_parameters");
        return Err(refuse(at.child("texture")));
    }
    let components = at.child("components");
    if let Some(collision) = effect.components.get::<ParticleMotionCollision>()
        && !(0.0..=0.5).contains(&collision.collision_radius)
        && !collision.collision_radius.is_nan()
    {
        let at = components.child(ParticleMotionCollision::NAME);
        return Err(refuse(at.child("collision_radius")));
    }
    if let Some(events) = effect.components.get::<EmitterLifetimeEvents>() {
        let entries = events.looping_travel_distance_events.iter().flatten();
        for (i, entry) in entries.enumerate() {
            if entry.distance < 0.0 {
                let at = components.child(EmitterLifetimeEvents::NAME);
                let at = at.child("looping_travel_distance_events").index(i);
                return Err(refuse(at.child("distance")));
            }
        }
    }
    Ok(())
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
    let at = JsonPath::new().child(PAYLOAD_KEY);
    constraints(effect, &at)?;
    let at = at.child("components");
    let mut components = VObject::new();
    for entry in effect.components.iter() {
        match entry {
            ComponentRef::Typed { name, value, .. } => {
                let json = write_component(value, false, &at.child(name))?;
                components.insert(name, json);
            }
            // An unknown component refuses a 1.20.80 document, so it has no place in the
            // text. Only a value built by hand holds one.
            ComponentRef::Custom(custom) => return Err(refuse(at.child(custom.name.as_str()))),
            ComponentRef::Dynamic { name, .. } | ComponentRef::Unknown { name, .. } => {
                return Err(refuse(at.child(name)));
            }
            _ => {}
        }
    }
    payload_of(
        &effect.description,
        components,
        effect.curves.as_ref(),
        effect.events.as_ref(),
        &effect.extra,
        false,
    )
}
