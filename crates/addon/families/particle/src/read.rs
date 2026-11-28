//! The parts of a particle document's read both structs share, and the 1.20.80 payload.
//!
//! A particle document has the same members in both eras; what differs is how a value that is
//! missing or of the wrong type is treated. The legacy parser (`format_version` below 1.20.80)
//! logs a line and goes on with the field's default; the reader of 1.20.80 and later
//! documents refuses the document. Every object of the document derives its read and its walk
//! (`#[derive(Facet, Schema)]`, the era's lines and specs as `schema::era` keys); the values the
//! derive has no form for (a curve, the colours, the leaves of several forms) read themselves
//! as [`FromJson`] leaves, and the helpers they tell the game's lines with are in `derived.rs`.
//! The 1.20.80 payload is read here as one derived struct ([`Payload`]); the legacy root is the
//! parser's own walk, in `history/v1_10_0.rs`. `write.rs` writes every part with the generic
//! writer.

use indexmap::IndexMap;

use super::component::{ParticleComponent, read_named, walk_legacy};
use super::curve::Curve;
use super::derived::{
    STRICT, entries, kept, kit_read, legacy, line, object, refuse, string, walk_in,
};
use super::event::EventNode;
use super::{BasicRenderParameters, Description, ParticleEffect, codes};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Class, Code, CodeDef, Diagnostic, Diagnostics, JsonPath, builders,
};
use bedrock_addon_core::document::{ComponentMap, UnknownMembers};
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor, Step};
use bedrock_addon_core::schema::{self, FromJson, Schema};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned};

/// An accessor for each optional member of a derived struct that has a default: the member, or
/// that default when the document does not write it.
macro_rules! defaults {
    ($name:ident { $($field:ident: $ty:ty = $json:literal, $default:expr;)* }) => {
        impl $name {
            $(
                #[doc = concat!("JSON `", $json, "`, or its default when the document does not write it.")]
                pub fn $field(&self) -> $ty {
                    self.$field.clone().unwrap_or_else(|| $default)
                }
            )*
        }
    };
}
pub(crate) use defaults;

/// The first format version of the newest struct: from this version the envelope is checked
/// and the document is read strictly.
pub(crate) const CEREAL: FormatVersion = FormatVersion::new(1, 20, 80);

/// The component the 1.26.30 engine added to the legacy registry only: its parser reads
/// nothing, and the upgrade drops it.
pub(crate) const CAMERA_DISTANCE_FADE: &str = "minecraft:particle_appearance_camera_distance_fade";

/// The longest `texture` a 1.20.80 document takes.
pub(crate) const MAX_TEXTURE: usize = 256;

/// The length constraint of `texture`. Only a 1.20.80 document has it, and only its read
/// comes here: the legacy description is read member by member.
pub(super) fn render_parameters(
    parameters: &mut BasicRenderParameters,
    node: &Node<'_>,
    read: &mut Read<'_>,
) {
    if parameters.texture.chars().count() > MAX_TEXTURE {
        let at = node.path().child("texture");
        refuse(read, &at, Code::from(&core_codes::INVALID_VALUE));
    }
}

/// `description/identifier`: a string, with no line of the game's for one that is none.
pub(super) fn identifier(
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<bedrock_addon_core::id::ParticleId> {
    string(node, read, None).map(bedrock_addon_core::id::ParticleId::parse)
}

/// One entry of `components` as read.
#[allow(
    clippy::large_enum_variant,
    reason = "handed straight on to the map that holds it; nearly every entry is typed"
)]
pub(crate) enum Entry {
    /// One of the 29 components.
    Typed(ParticleComponent),
    /// `minecraft:particle_appearance_camera_distance_fade` in a legacy document, as written.
    CameraDistanceFade(Value),
}

/// Reads the entries of the `components` object at `node`, each handed to `sink` under its
/// key, in file order.
pub(crate) fn read_components(
    node: &Node<'_>,
    read: &mut Read<'_>,
    mut sink: impl FnMut(&str, Entry),
) {
    let Some(members) = read.object(node) else {
        return;
    };
    let legacy = legacy(read);
    for (key, member) in members.iter() {
        let at = member.path();
        if key.is_empty() && legacy {
            line(read, at, &codes::COMPONENT_WITHOUT_NAME);
            continue;
        }
        match read_named(key, &member, read) {
            Some(Some(component)) => sink(key, Entry::Typed(component)),
            // The 1.20.80 reader refused it; the finding rejects the document.
            Some(None) => {}
            None if legacy && key == CAMERA_DISTANCE_FADE => {
                sink(key, Entry::CameraDistanceFade(member.value().clone()));
            }
            // The game logs this as a warning in both eras; the legacy parser's line names
            // the key. Not kept: the game stores no unknown key. The legacy parser writes its
            // line at the error level and the effect loads, so the finding is a kept error; a
            // 1.20.80 document logs a warning, which its strict loading grades an error that
            // refuses the document.
            None if legacy => kept(read, at, codes::UNRECOGNIZED_COMPONENT.with([key])),
            None => {
                let unknown = read.unknown_grade();
                read.push(
                    Diagnostic::error(Code::from(&core_codes::UNRECOGNIZED_COMPONENT))
                        .severity(unknown)
                        .at(at.clone()),
                );
            }
        }
    }
}

/// Reads `curves` or `events` at `node`: a map whose entries are read one by one by `entry`;
/// an entry without a name is left out with `unnamed` in a legacy document.
fn named<T>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    (not_object, unnamed): (&'static CodeDef<0>, &'static CodeDef<0>),
    entry: fn(&Node<'_>, &mut Read<'_>) -> Option<T>,
) -> Option<IndexMap<String, T>> {
    entries(node, read, Some(not_object), |name, member, read| {
        if name.is_empty() && legacy(read) {
            line(read, member.path(), unnamed);
            return None;
        }
        entry(member, read)
    })
}

/// `curves`.
pub(crate) fn curves(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, Curve>> {
    named(
        node,
        read,
        (&codes::NO_CURVES, &codes::CURVE_WITHOUT_NAME),
        Curve::from_json,
    )
}

/// `events`: an event that is no object gets the legacy parser's line of an event node.
pub(crate) fn events(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, EventNode>> {
    named(
        node,
        read,
        (&codes::NO_EVENTS, &codes::EVENT_WITHOUT_NAME),
        |event, read| object(event, read, Some(&codes::EVENT_NO_DATA)),
    )
}

/// The members of `particle_effect`.
pub(crate) fn known(key: &str) -> bool {
    matches!(key, "description" | "components" | "curves" | "events")
}

/// `text` as the JSON value the readers take. Every number token is a number in it: the
/// crate's JSON reader reads an integer literal too long for 64 bits as the double it spells
/// (nothing else needs to inspect the text first).
pub(crate) fn value_of(text: &str) -> Result<Value, bedrock_addon_core::json::parse::Fault> {
    bedrock_addon_core::json::from_str_jsonc(text)
}

/// The text of a version struct as JSON; a finding and `None` when it is none.
pub(crate) fn json(text: JsonSlice<'_>, at: &JsonPath, d: &mut Diagnostics) -> Option<Value> {
    match value_of(text.text) {
        Ok(value) => Some(value),
        Err(error) => {
            d.push(Diagnostic::of(builders::syntax(error.kind.to_string())).at(at.clone()));
            None
        }
    }
}

/// The payload object of a 1.20.80 document, as its reader reads it: a member that cannot be
/// used is reported as an error, which refuses the document, and the value then only carries
/// what could be read.
#[derive(facet::Facet, Schema)]
#[facet(schema::unknown = "report")]
struct Payload {
    #[facet(schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD))]
    description: Description,
    #[facet(
        opaque,
        default,
        schema::read_with = components,
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    components: Components,
    #[facet(opaque, schema::read_with = curves)]
    curves: Option<IndexMap<String, Curve>>,
    #[facet(schema::read_with = events)]
    events: Option<IndexMap<String, EventNode>>,
    #[facet(skip_deserializing)]
    extra: UnknownMembers,
}

/// The components of a 1.20.80 payload.
#[derive(Default)]
struct Components(ComponentMap<ParticleComponent>);

// The components are read by the family's own loop, and the effect that holds them walks them.
bedrock_addon_core::impl_no_molang!(Components);
bedrock_addon_core::impl_schema_leaf!(Components);

/// `components` of a 1.20.80 payload: the camera-distance fade is no component there.
fn components(node: &Node<'_>, read: &mut Read<'_>) -> Option<Components> {
    if !node.value().is_object() {
        read.wrong_type(node);
        return None;
    }
    let mut components = ComponentMap::new();
    read_components(node, read, |key, entry| match entry {
        Entry::Typed(component) => {
            components.insert_typed(component, false);
        }
        Entry::CameraDistanceFade(json) => components.insert_unknown(key, json),
    });
    Some(Components(components))
}

impl Versioned for ParticleEffect {
    const VERSION: FormatVersion = CEREAL;
}

impl VersionStruct for ParticleEffect {
    /// Reads the payload (from 1.20.80 the envelope is checked, so the slice is the value of
    /// `particle_effect`).
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let value = json(payload, &cx.at, d)?;
        let unknown = cx.load.grade(Class::Unknown);
        let mut read = kit_read(false, cx.load.molang_version, unknown, d);
        let payload: Payload = schema::read_document(&Node::new(&value, cx.at.clone()), &mut read)?;
        Some(Self {
            description: payload.description,
            components: payload.components.0,
            curves: payload.curves,
            events: payload.events,
            extra: UnknownMembers::default(),
        })
    }

    /// The payload (`write::document_payload`).
    fn write(&self) -> Result<Value, EncodeError> {
        super::write::document_payload(self)
    }
}

impl MolangFields for ParticleEffect {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        v.nested("components", &mut self.components);
        walk_named(&mut self.curves, &mut self.events, false, v);
    }
}

/// Walks `curves` and `events` as a document of the era holds them.
pub(crate) fn walk_named(
    curves: &mut Option<IndexMap<String, Curve>>,
    events: &mut Option<IndexMap<String, EventNode>>,
    legacy: bool,
    v: &mut MolangVisitor<'_>,
) {
    fn each<T: MolangFields>(
        key: &str,
        map: &mut IndexMap<String, T>,
        legacy: bool,
        v: &mut MolangVisitor<'_>,
    ) {
        v.enter(Step::Key(key));
        for (name, entry) in map.iter_mut() {
            v.enter(Step::Key(name));
            walk_in(entry, legacy, v);
            v.leave();
        }
        v.leave();
    }
    if let Some(curves) = curves {
        each("curves", curves, legacy, v);
    }
    if let Some(events) = events {
        each("events", events, legacy, v);
    }
}

/// Walks a typed entry of a legacy `components` map under its key.
pub(crate) fn walk_legacy_component(
    key: &str,
    component: &mut ParticleComponent,
    v: &mut MolangVisitor<'_>,
) {
    v.enter(Step::Key(key));
    walk_legacy(component, v);
    v.leave();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{DistanceEvent, VectorEvents};
    use crate::linear::{ERAS, array_of, assert_reads_linear, object_of, reading, root};

    /// The map reader alone, as `emitter_lifetime_events/timeline` and
    /// `particle_lifetime_events/timeline` use it, on a value parsed beforehand.
    #[test]
    fn the_map_reader_reads_a_timeline_of_100_000_entries_in_linear_time() {
        for (era, legacy) in ERAS {
            assert_reads_linear(
                &format!("timeline ({era})"),
                |count| object_of(count, "0.", r#"["a", "b"]"#),
                |timeline, count| {
                    let (read, found) = reading(legacy, |r| {
                        crate::component::types::timeline(&root(timeline), r)
                    });
                    assert_eq!(
                        read.as_ref().map(IndexMap::<String, VectorEvents>::len),
                        Some(count)
                    );
                    assert!(found.is_empty());
                    (read, found)
                },
            );
        }
    }

    /// The list reader alone, as `looping_travel_distance_events` uses it.
    #[test]
    fn the_list_reader_reads_100_000_distance_events_in_linear_time() {
        for (era, legacy) in ERAS {
            assert_reads_linear(
                &format!("distance events ({era})"),
                |count| array_of(count, r#"{"distance": 1, "effects": "a"}"#),
                |events, count| {
                    let (read, found) = reading(legacy, |r| {
                        crate::derived::elements(&root(events), r, None, |entry, r| {
                            DistanceEvent::from_json(entry, r)
                        })
                    });
                    assert_eq!(read.as_ref().map(Vec::len), Some(count));
                    assert!(found.is_empty());
                    (read, found)
                },
            );
        }
    }

    /// `read_components` alone on a `components` object parsed beforehand: every key is its
    /// own (unknown) component, which is one finding each and no entry.
    #[test]
    fn read_components_reads_100_000_component_keys_in_linear_time() {
        for (era, legacy) in ERAS {
            assert_reads_linear(
                &format!("read_components ({era})"),
                |count| object_of(count, "demo:unknown", r#"{"a": 1}"#),
                |components, count| {
                    let mut entries = Vec::with_capacity(count);
                    let ((), found) = reading(legacy, |r| {
                        read_components(&root(components), r, |key, entry| {
                            entries.push((key.to_owned(), entry));
                        });
                    });
                    assert_eq!(entries.len(), 0);
                    assert_eq!(found.len(), count);
                    (entries, found)
                },
            );
        }
    }
}
