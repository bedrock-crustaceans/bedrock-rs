//! Format 1.10.0 (`[1.10.0, 1.20.80)`): the document as the game's legacy parser reads it,
//! and its upgrade to the 1.20.80 struct.
//!
//! The members are those of the newest document. The legacy parser differs in what it does
//! with a member it cannot use: it logs a line and goes on with the member's default, where
//! the 1.20.80 reader refuses the document. Molang is a number or a string here, never the
//! `{"expression", "version"}` object.

#![allow(
    clippy::single_match_else,
    reason = "the two arms are the two outcomes of a read: the value, or the line about it"
)]

use facet_value::VObject;
use indexmap::IndexMap;

use crate::component::{
    DirectionMode, DirectionSettings, EmitterLifetimeEvents, EmitterLifetimeExpression,
    EmitterLifetimeLooping, EmitterLifetimeOnce, EmitterRateInstant, EmitterRateManual,
    EmitterRateSteady, EmitterShapePoint, ParticleAppearanceBillboard, ParticleComponent,
    write_component,
};
use crate::derived::{kept, kit_read, line, note, refuse};
use crate::read::{
    Entry, curves, events, json, known, read_components, walk_legacy_component, walk_named,
};
use crate::write::{PAYLOAD_KEY, document, payload_of};
use crate::{BasicRenderParameters, Curve, Description, EventNode, codes};
#[cfg(feature = "molang")]
use bedrock_addon_core::diagnostic::Location;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, CodeDef, Diagnostic, Diagnostics, JsonPath, Phase, Severity,
};
use bedrock_addon_core::document::{Component as _, ComponentMap, UnknownMembers};
use bedrock_addon_core::id::ParticleId;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{
    Molang, MolangFieldMut, MolangFields, MolangSource, MolangVisitor, RawVersion, SourceForm,
    Step, visit_fields,
};
use bedrock_addon_core::schema::UnknownRule;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

/// A particle effect of a format below 1.20.80, as the legacy parser leaves it.
///
/// JSON: the whole document, `{"format_version": "1.10.0", "particle_effect":
/// {"description", "components", "curves"?, "events"?}}`; the legacy parser reads the text
/// itself, so other root members are passed over.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleEffect {
    /// The document's `format_version` (the floor when the file has none).
    pub format_version: FormatVersion,
    /// What the effect is (JSON `description`).
    pub description: Description,
    /// The components by key, in file order (JSON `components`).
    pub components: IndexMap<String, Component>,
    /// Curves by the Molang variable each sets (JSON `curves`).
    pub curves: Option<IndexMap<String, Curve>>,
    /// Events by name (JSON `events`).
    pub events: Option<IndexMap<String, EventNode>>,
    /// Unknown members of `particle_effect`.
    pub extra: UnknownMembers,
}

/// One entry of a legacy document's `components`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
#[allow(
    clippy::large_enum_variant,
    reason = "a document holds few components, and most are typed"
)]
pub enum Component {
    /// One of the 29 components, with the legacy parser's defaults where a member was not
    /// usable.
    Typed(ParticleComponent),
    /// `minecraft:particle_appearance_camera_distance_fade`, as written: the game's legacy
    /// registry knows the name from engine 1.26.30, its parser reads nothing, and the
    /// upgrade drops it.
    CameraDistanceFade(Value),
    /// A key the legacy registry does not know, as written.
    Unknown(Value),
}

impl Versioned for ParticleEffect {
    const VERSION: FormatVersion = bedrock_addon_core::version::known::V1_10_0;
}

/// An error of the legacy envelope checks, and of a description that gives no identifier
/// (the error is the crate's own there: the game registers an effect named `""`), with
/// vanilla's text.
fn refused(read: &mut Read<'_>, at: &JsonPath, code: &'static CodeDef<0>) {
    refuse(read, at, Code::from(code));
}

/// A content-log line about a member that is missing, which the game writes at the error
/// level and which does not stop the load: a kept error.
fn lacking(read: &mut Read<'_>, at: &JsonPath, code: &'static CodeDef<0>) {
    kept(read, at, Code::from(code));
}

/// The members of the object `value` at `at` that `known` does not name: each reported as
/// `UnrecognizedField` (the legacy parser reads past them, so the crate tells them as an
/// information) and not kept: the game never stores an unknown key, so the value holds none
/// and the upgrade carries none. The `extra` of a value is for one built by hand.
fn passed_over(value: &Value, at: &JsonPath, known: fn(&str) -> bool, read: &mut Read<'_>) {
    for (key, member) in value
        .as_object()
        .into_iter()
        .flat_map(|object| object.iter())
    {
        if !known(key.as_str()) {
            let node = Node::new(member, at.child(key.as_str()));
            read.unknown_member(&node, UnknownRule::Keep);
        }
    }
}

/// A string member of `basic_render_parameters` (the object `parameters` at `at`): its value,
/// and whether the legacy parser stops at it. A missing member stops it with `missing`; one
/// that is no string gets `not_string` and stops only where `stops` says so.
fn render_string(
    parameters: &Value,
    at: &JsonPath,
    key: &str,
    (missing, not_string, stops): (&'static CodeDef<0>, &'static CodeDef<0>, bool),
    read: &mut Read<'_>,
) -> (String, bool) {
    let at = at.child(key);
    match parameters.as_object().and_then(|object| object.get(key)) {
        None => {
            lacking(read, &at, missing);
            (String::new(), true)
        }
        Some(value) => match value.as_string() {
            Some(text) => (text.as_str().to_owned(), false),
            None => {
                line(read, &at, not_string);
                (String::new(), stops)
            }
        },
    }
}

/// The member `key` of the object `value`.
fn member<'v>(value: &'v Value, key: &str) -> Option<&'v Value> {
    value.as_object()?.get(key)
}

impl ParticleEffect {
    /// Reads the payload object at `at` the way the legacy parser does.
    fn read(
        payload: &Value,
        at: &JsonPath,
        version: FormatVersion,
        read: &mut Read<'_>,
    ) -> Option<Self> {
        let at_description = at.child("description");
        let Some(description) = member(payload, "description").filter(|d| d.is_object()) else {
            refused(read, &at_description, &codes::NO_DESCRIPTION);
            return None;
        };
        let Some(identifier) = member(description, "identifier").and_then(Value::as_string) else {
            refused(
                read,
                &at_description.child("identifier"),
                &codes::NO_IDENTIFIER,
            );
            return None;
        };
        // From here on the parser has the identifier: where it stops, the game still upgrades
        // and registers what was parsed.
        let at_parameters = at_description.child("basic_render_parameters");
        let parameters = member(description, "basic_render_parameters").filter(|p| p.is_object());
        let (material, texture, stopped) = match parameters {
            None => {
                lacking(read, &at_parameters, &codes::NO_RENDER_PARAMETERS);
                (String::new(), String::new(), true)
            }
            Some(parameters) => {
                let (material, no_material) = render_string(
                    parameters,
                    &at_parameters,
                    "material",
                    (&codes::NO_MATERIAL, &codes::MATERIAL_NOT_STRING, true),
                    read,
                );
                let (texture, no_texture) = render_string(
                    parameters,
                    &at_parameters,
                    "texture",
                    (&codes::NO_TEXTURE, &codes::TEXTURE_NOT_STRING, false),
                    read,
                );
                let known = |key: &str| matches!(key, "material" | "texture");
                passed_over(parameters, &at_parameters, known, read);
                (material, texture, no_material || no_texture)
            }
        };
        let known = |key: &str| matches!(key, "identifier" | "basic_render_parameters");
        passed_over(description, &at_description, known, read);
        let description = Description {
            identifier: ParticleId::parse(identifier.as_str().to_owned()),
            basic_render_parameters: BasicRenderParameters {
                material,
                texture,
                extra: UnknownMembers::default(),
            },
            extra: UnknownMembers::default(),
        };
        Some(Self::read_rest(
            payload,
            at,
            description,
            stopped,
            version,
            read,
        ))
    }

    /// What follows the description: curves, events and components, as far as the parser
    /// gets. Where it stops (`stopped`: the render parameters; or no `components` object) it
    /// returns what it has, and that is the effect.
    fn read_rest(
        payload: &Value,
        at: &JsonPath,
        description: Description,
        stopped: bool,
        version: FormatVersion,
        read: &mut Read<'_>,
    ) -> Self {
        let mut effect = Self {
            format_version: version,
            description,
            components: IndexMap::new(),
            curves: None,
            events: None,
            extra: UnknownMembers::default(),
        };
        if stopped {
            // The effect is registered with what was read so far: no curve, event or
            // component. Each of the three the file has is noted as not read, at the
            // information level (the game stops there too): the line at the render
            // parameters does not say that the rest of the effect goes with them.
            for key in ["curves", "events", "components"] {
                if member(payload, key).is_some_and(|value| !value.is_null()) {
                    note(read, &at.child(key), &core_codes::DROPPED_FIELD);
                }
            }
            // An unknown member is no part of what the parser stopped before.
            passed_over(payload, at, known, read);
            return effect;
        }
        let at_curves = at.child("curves");
        effect.curves =
            member(payload, "curves").and_then(|value| curves(&Node::new(value, at_curves), read));
        let at_events = at.child("events");
        effect.events =
            member(payload, "events").and_then(|value| events(&Node::new(value, at_events), read));
        let at_components = at.child("components");
        let Some(components) = member(payload, "components").filter(|c| c.is_object()) else {
            // The parser stops here too: the effect has its curves and events, no component.
            lacking(read, &at_components, &codes::NO_COMPONENTS);
            passed_over(payload, at, known, read);
            return effect;
        };
        read_components(
            &Node::new(components, at_components.clone()),
            read,
            |key, entry| {
                let entry = match entry {
                    Entry::Typed(component) => Component::Typed(component),
                    Entry::CameraDistanceFade(json) => Component::CameraDistanceFade(json),
                };
                effect.components.insert(key.to_owned(), entry);
            },
        );
        passed_over(payload, at, known, read);
        let parameters = &effect.description.basic_render_parameters;
        let at_parameters = at.child("description").child("basic_render_parameters");
        if parameters.material.is_empty() {
            lacking(
                read,
                &at_parameters.child("material"),
                &codes::MATERIAL_NOT_FOUND,
            );
        }
        if parameters.texture.is_empty() {
            lacking(
                read,
                &at_parameters.child("texture"),
                &codes::TEXTURE_NOT_FOUND,
            );
        }
        let has = |names: &[&str]| names.iter().any(|name| effect.has_typed(name));
        if !has(&[
            EmitterLifetimeOnce::NAME,
            EmitterLifetimeLooping::NAME,
            EmitterLifetimeExpression::NAME,
            EmitterLifetimeEvents::NAME,
        ]) {
            lacking(read, &at_components, &codes::NEEDS_LIFETIME);
        }
        if !has(&[
            EmitterRateInstant::NAME,
            EmitterRateSteady::NAME,
            EmitterRateManual::NAME,
        ]) {
            lacking(read, &at_components, &codes::NEEDS_RATE);
        }
        effect
    }

    fn has_typed(&self, name: &str) -> bool {
        matches!(self.components.get(name), Some(Component::Typed(_)))
    }

    /// The payload object.
    fn payload(&self) -> Result<Value, EncodeError> {
        let at = JsonPath::new().child(PAYLOAD_KEY).child("components");
        let mut components = VObject::new();
        for (key, entry) in &self.components {
            let json = match entry {
                Component::Typed(component) => write_component(component, true, &at.child(key))?,
                Component::CameraDistanceFade(json) | Component::Unknown(json) => json.clone(),
            };
            components.insert(key.as_str(), json);
        }
        payload_of(
            &self.description,
            components,
            self.curves.as_ref(),
            self.events.as_ref(),
            &self.extra,
            true,
        )
    }
}

impl VersionStruct for ParticleEffect {
    /// Reads the whole document (the raw-text path below 1.20.80) or the payload alone (as
    /// the step-file harness hands it).
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let root = json(text, &cx.at, d)?;
        let mut read = kit_read(true, cx.load.molang_version, Severity::Info, d);
        let version = cx.original_version;
        let whole =
            member(&root, PAYLOAD_KEY).is_some() || member(&root, "format_version").is_some();
        let effect = if whole {
            // The legacy parser reads `format_version` and `particle_effect` and nothing else
            // of the root: another member is passed over, which the crate reports (it is gone
            // after the upgrade, whose envelope has two members).
            let known = |key: &str| key == PAYLOAD_KEY || key == "format_version";
            passed_over(&root, &cx.at, known, &mut read);
            let at = cx.at.child(PAYLOAD_KEY);
            match member(&root, PAYLOAD_KEY) {
                Some(payload) => Self::read(payload, &at, version, &mut read),
                None => {
                    refused(&mut read, &at, &codes::NO_PAYLOAD);
                    None
                }
            }
        } else {
            Self::read(&root, &cx.at, version, &mut read)
        };
        read.finish();
        effect
    }

    /// The whole document, at the format version it was read at.
    fn write(&self) -> Result<Value, EncodeError> {
        Ok(document(&self.format_version.to_string(), self.payload()?))
    }
}

impl MolangFields for ParticleEffect {
    /// The walk of the whole document: every path starts at `particle_effect`.
    fn visit_molang(&mut self, v: &mut MolangVisitor<'_>) {
        v.enter(Step::Key(PAYLOAD_KEY));
        v.enter(Step::Key("components"));
        for (key, entry) in &mut self.components {
            if let Component::Typed(component) = entry {
                walk_legacy_component(key, component, v);
            }
        }
        v.leave();
        walk_named(&mut self.curves, &mut self.events, true, v);
        v.leave();
    }
}

/// The Molang version the legacy parser compiles its built-in `direction` at.
const POINT_DIRECTION_VERSION: RawVersion = RawVersion(13);
const POINT_DIRECTION: &str = "math.random(-1, 1)";
/// The legacy `min_speed_threshold` of a billboard's direction.
const LEGACY_MIN_SPEED_THRESHOLD: f32 = 0.01;
/// The legacy `facing_camera_mode`.
const LEGACY_FACING: &str = "lookat_xyz";

impl ParticleEffect {
    /// A member whose expression the compiler rejects becomes the member's default constant,
    /// after the compiler's lines. Runs only with the `molang` feature and a compile check in
    /// the load context; without them the source is kept.
    #[cfg(feature = "molang")]
    fn default_rejected_molang(&mut self, cx: &mut UpgradeCx<'_>) {
        use bedrock_addon_core::molang::{MolangContext, MolangFailure, MolangOutcome, Side};
        use bedrock_addon_core::version::FormatVersionInput;
        let Some(check) = cx.load.molang.clone() else {
            return;
        };
        let written = FormatVersionInput {
            version: Some(cx.original_version),
            beta_literal: false,
            prerelease: false,
        };
        let molang = MolangContext::new(
            cx.load.min_engine_version.as_ref(),
            cx.load.molang_experiments,
            &written,
            Some(cx.load.molang_version),
        );
        let diagnostics = &mut *cx.diagnostics;
        visit_fields(self, &JsonPath::new(), |field| {
            let MolangFieldMut::Value(value) = field.value else {
                return;
            };
            let MolangFailure::ReplaceWith(default) = field.spec.on_failure else {
                return;
            };
            // A member that compiles is reported once, by the check of the upgraded
            // document; only a rejected one is reported here, where it is replaced.
            let mut found = Diagnostics::default();
            let location = Location::new(None, None, field.path.clone(), None);
            let outcome = check.check(
                value,
                field.spec,
                Side::Client,
                &molang,
                &location,
                &mut found,
            );
            if matches!(outcome, MolangOutcome::Failed(_)) {
                for finding in found {
                    diagnostics.push(finding);
                }
                *value = Molang::Const(default);
            }
        });
    }

    /// A string becomes `{"expression", "version"}` at the pack's Molang version, so the
    /// upgraded document says what the legacy parser compiled it at; a bool (which the
    /// 1.20.80 reader does not take) becomes its number.
    fn molang_to_expression_nodes(&mut self, version: RawVersion, d: &mut Diagnostics) {
        visit_fields(self, &JsonPath::new(), |field| {
            let MolangFieldMut::Value(value) = field.value else {
                return;
            };
            let converted = match &*value {
                Molang::Bool(b) => {
                    // The effect is held and written with the number: noted at the member,
                    // here, where the value is rewritten (a finding of the crate's own; a
                    // document written as it was read keeps its boolean and gets no note).
                    d.push(
                        Diagnostic::of(&core_codes::VALUE_REWRITTEN)
                            .at(field.path.clone())
                            .phase(Phase::Upgrade),
                    );
                    Molang::Const(f32::from(u8::from(*b)))
                }
                Molang::Expr(source) if matches!(source.form(), SourceForm::String { .. }) => {
                    Molang::Expr(MolangSource::object(source.as_str(), version))
                }
                _ => return,
            };
            *value = converted;
        });
    }
}

/// Every node gets the two defaults the game builds it with: `expression` 0 and
/// `particle_effect.pre_effect_expression` 0.
fn event_defaults(events: &mut IndexMap<String, EventNode>) {
    let mut pending: Vec<&mut EventNode> = events.values_mut().collect();
    while let Some(node) = pending.pop() {
        node.expression.get_or_insert(Molang::Const(0.0));
        if let Some(effect) = &mut node.particle_effect {
            effect
                .pre_effect_expression
                .get_or_insert(Molang::Const(0.0));
        }
        pending.extend(node.sequence.iter_mut().flatten());
        pending.extend(
            node.randomize
                .iter_mut()
                .flatten()
                .map(|option| &mut option.node),
        );
    }
}

/// The members the legacy parser has given a component that the 1.20.80 defaults would not:
/// the upgrade writes them out.
fn legacy_defaults(component: &mut ParticleComponent) {
    match component {
        ParticleComponent::EmitterShapePoint(EmitterShapePoint { direction, .. }) => {
            direction.get_or_insert_with(|| {
                std::array::from_fn(|_| {
                    Molang::Expr(MolangSource::object(
                        POINT_DIRECTION,
                        POINT_DIRECTION_VERSION,
                    ))
                })
            });
        }
        ParticleComponent::ParticleAppearanceBillboard(ParticleAppearanceBillboard {
            facing_camera_mode,
            direction,
            ..
        }) => {
            facing_camera_mode.get_or_insert_with(|| LEGACY_FACING.to_owned());
            direction
                .get_or_insert_with(|| DirectionSettings {
                    mode: DirectionMode::DeriveFromVelocity,
                    custom_direction: None,
                    min_speed_threshold: None,
                    extra: UnknownMembers::default(),
                })
                .min_speed_threshold
                .get_or_insert(LEGACY_MIN_SPEED_THRESHOLD);
        }
        _ => {}
    }
}

impl Upgrade for ParticleEffect {
    type Next = crate::ParticleEffect;

    /// The game's own conversion of a legacy document: a typed conversion in which every
    /// member keeps its key and place.
    fn upgrade(mut self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        #[cfg(feature = "molang")]
        self.default_rejected_molang(cx);
        self.molang_to_expression_nodes(cx.load.molang_version, cx.diagnostics);
        let mut components = ComponentMap::new();
        for (key, entry) in std::mem::take(&mut self.components) {
            match entry {
                Component::Typed(mut component) => {
                    legacy_defaults(&mut component);
                    components.insert_typed(component, false);
                }
                // The game moves the 29 components by name and never looks this one up, so
                // the crate says so once, at the information level.
                Component::CameraDistanceFade(_) => {
                    let mut at = JsonPath::new();
                    for step in [PAYLOAD_KEY, "components", key.as_str()] {
                        at.push(step);
                    }
                    cx.diagnostics.push(
                        Diagnostic::of(&core_codes::NO_EFFECT)
                            .at(at)
                            .phase(Phase::Upgrade),
                    );
                }
                Component::Unknown(json) => components.insert_unknown(&key, json),
            }
        }
        let mut events = self.events.take();
        if let Some(events) = &mut events {
            event_defaults(events);
        }
        Self::Next {
            description: self.description.clone(),
            components,
            curves: self.curves.take(),
            events,
            extra: std::mem::take(&mut self.extra),
        }
    }
}
