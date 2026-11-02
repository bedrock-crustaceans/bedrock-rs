//! Reading [`ParticleEffect`] as the game reads a 1.20.80 document, and the parts of the read
//! the legacy struct shares.

use indexmap::IndexMap;

use super::component::{ParticleComponent, read_named};
use super::curve::{Curve, named_map, object_member};
use super::event::EventNode;
use super::read::{Era, Members, Node, Opt, Reader, Writer, req, walk};
use super::{BasicRenderParameters, Description, ParticleEffect, texts};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::id::ParticleId;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor, Step};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned};

/// The first format version of the newest struct: from this version the envelope is checked
/// and the document is read strictly.
pub(crate) const CEREAL: FormatVersion = FormatVersion::new(1, 20, 80);

/// The component the 1.26.30 engine added to the legacy registry only: its parser reads
/// nothing, and the upgrade drops it.
pub(crate) const CAMERA_DISTANCE_FADE: &str = "minecraft:particle_appearance_camera_distance_fade";

/// The longest `texture` a 1.20.80 document takes.
pub(crate) const MAX_TEXTURE: usize = 256;

impl Node for ParticleId {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        String::read(value, r, o).map(Self::parse)
    }

    fn fallback(_: Opt) -> Self {
        Self::parse(String::new())
    }

    fn write(&self, _: &mut Writer) -> Result<Value, EncodeError> {
        Ok(Value::from(self.as_str()))
    }
}

/// The length constraint of `texture`. Only a 1.20.80 document has it, and only its read
/// comes here: the legacy description is read member by member.
pub(super) fn render_parameters(
    parameters: &mut BasicRenderParameters,
    _: Members<'_>,
    r: &mut Reader<'_>,
) {
    if parameters.texture.chars().count() > MAX_TEXTURE {
        r.key("texture", |r| {
            r.error(DiagnosticCode::InvalidValue { text: None });
        });
    }
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

/// Reads the entries of `components`, each handed to `sink` under its key, in file order.
pub(crate) fn read_components(
    components: Members<'_>,
    r: &mut Reader<'_>,
    mut sink: impl FnMut(&str, Entry),
) {
    for (key, value) in components.iter() {
        r.key(key, |r| {
            if key.is_empty() && r.legacy() {
                r.warn(texts::COMPONENT_WITHOUT_NAME);
                return;
            }
            match read_named(key, value, r) {
                Some(Some(component)) => sink(key, Entry::Typed(component)),
                // The 1.20.80 reader refused it; the finding rejects the document.
                Some(None) => {}
                None if r.legacy() && key == CAMERA_DISTANCE_FADE => {
                    sink(key, Entry::CameraDistanceFade(value.clone()));
                }
                None => {
                    // The game logs this as a warning in both eras; the legacy parser's
                    // line names the key.
                    let text = r.legacy().then(|| VanillaText::Formatted {
                        template: texts::UNRECOGNIZED_COMPONENT,
                        args: vec![key.to_owned()],
                    });
                    // Not kept: the game stores no unknown key. The legacy parser writes its
                    // line at the error level and the effect loads, so the finding is a kept
                    // error; a 1.20.80 document logs a warning, which its strict loading
                    // makes an error that refuses the document.
                    if r.legacy() {
                        r.push_kept(DiagnosticCode::UnrecognizedComponent { text });
                    } else {
                        r.push(
                            DiagnosticCode::UnrecognizedComponent { text },
                            Severity::Warning,
                        );
                    }
                }
            }
        });
    }
}

/// Reads `curves`.
pub(crate) fn read_curves(m: Members<'_>, r: &mut Reader<'_>) -> Option<IndexMap<String, Curve>> {
    named_map(m, "curves", texts::NO_CURVES, texts::CURVE_WITHOUT_NAME, r)
}

/// Reads `events`.
pub(crate) fn read_events(
    m: Members<'_>,
    r: &mut Reader<'_>,
) -> Option<IndexMap<String, EventNode>> {
    named_map(m, "events", texts::NO_EVENTS, texts::EVENT_WITHOUT_NAME, r)
}

/// The members of `particle_effect`.
pub(crate) fn known(key: &str) -> bool {
    matches!(key, "description" | "components" | "curves" | "events")
}

/// `text` as the JSON value the readers take. Every number token is a number in it: the
/// crate's JSON reader reads an integer literal too long for 64 bits as the double it spells
/// (nothing else needs to inspect the text first).
pub(crate) fn value_of(text: &str) -> Result<Value, facet_json::DeserializeError> {
    bedrock_addon_core::json::from_str_jsonc::<Value>(text)
}

/// The text of a version struct as JSON; a finding and `None` when it is none.
pub(crate) fn json(text: JsonSlice<'_>, at: &JsonPath, d: &mut Diagnostics) -> Option<Value> {
    match value_of(text.text) {
        Ok(value) => Some(value),
        Err(error) => {
            d.push(Diagnostic::new(
                DiagnosticCode::MalformedInput {
                    message: error.kind.to_string(),
                    text: None,
                },
                Severity::Error,
                Phase::Parse,
                Location::new(None, None, at.clone(), None),
            ));
            None
        }
    }
}

impl ParticleEffect {
    /// Reads the payload object as the game reads a 1.20.80 document. A member that cannot
    /// be used is reported as an error, which refuses the document; the value returned then
    /// only carries what could be read.
    fn read(payload: &Value, r: &mut Reader<'_>) -> Option<Self> {
        let Some(m) = Members::of(payload) else {
            r.error(DiagnosticCode::InvalidType { text: None });
            return None;
        };
        let description: Description = req(m, "description", Opt::NONE, r);
        let mut components = ComponentMap::new();
        if let Some(entries) = object_member(m, "components", r) {
            r.key("components", |r| {
                read_components(entries, r, |key, entry| match entry {
                    Entry::Typed(component) => {
                        components.insert_typed(component, false);
                    }
                    Entry::CameraDistanceFade(json) => {
                        components.insert_unknown(key, json);
                    }
                });
            });
        }
        Some(Self {
            description,
            components,
            curves: read_curves(m, r),
            events: read_events(m, r),
            extra: m.extra(known, r),
        })
    }
}

impl Versioned for ParticleEffect {
    const VERSION: FormatVersion = CEREAL;
}

impl VersionStruct for ParticleEffect {
    /// Reads the payload (from 1.20.80 the envelope is checked, so the slice is the value of
    /// `particle_effect`).
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let value = json(payload, &cx.at, d)?;
        let mut r = Reader::new(Era::Cereal, cx.load.molang_version, cx.at.clone(), d);
        Self::read(&value, &mut r)
    }

    /// The payload.
    fn write(&self) -> Result<Value, EncodeError> {
        super::write::payload(self, &mut Writer::default())
    }
}

impl MolangFields for ParticleEffect {
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        v.nested("components", &mut self.components);
        walk_named(&mut self.curves, &mut self.events, Era::Cereal, v);
    }
}

/// Walks `curves` and `events`.
pub(crate) fn walk_named(
    curves: &mut Option<IndexMap<String, Curve>>,
    events: &mut Option<IndexMap<String, EventNode>>,
    era: Era,
    v: &mut MolangVisitor<'_>,
) {
    if let Some(curves) = curves {
        walk("curves", curves, era, Opt::NONE, v);
    }
    if let Some(events) = events {
        walk("events", events, era, Opt::NONE, v);
    }
}

/// Walks a typed entry of a legacy `components` map under its key.
pub(crate) fn walk_legacy_component(
    key: &str,
    component: &mut ParticleComponent,
    v: &mut MolangVisitor<'_>,
) {
    v.enter(Step::Key(key));
    super::component::walk_component(component, Era::Legacy, v);
    v.leave();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linear::{assert_reads_linear, object_of, reading};

    /// `read_components` alone on a `components` object parsed beforehand: every key is its
    /// own (unknown) component, which is one finding each and no entry.
    #[test]
    fn read_components_reads_100_000_component_keys_in_linear_time() {
        for era in [Era::Legacy, Era::Cereal] {
            assert_reads_linear(
                &format!("read_components ({era:?})"),
                |count| object_of(count, "demo:unknown", r#"{"a": 1}"#),
                |components, count| {
                    let members = Members::of(components).expect("an object");
                    let mut entries = Vec::with_capacity(count);
                    let ((), found) = reading(era, |r| {
                        read_components(members, r, |key, entry| {
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
