//! A toy family through `version_chain!`: three stable structs and a beta struct, a
//! `#[derive(Components)]` component enum, and the steps the step-file harness checks.
//!
//! - `v1_0_0::Toy` (the floor): `old_speed`, `legacy_flag`.
//! - step 1.0.0 → 1.1.0: rename `old_speed` → `speed`; drop `legacy_flag` with a
//!   `DroppedField` warning; default `color` = `"red"`.
//! - `v1_1_0::Toy`: `speed`, `color`, `note`, `components`.
//! - step 1.1.0 → 1.2.0: move `speed` → `movement/speed`; insert `legacy = true` for
//!   `original_version < 1.0.5`; side-car `note` → `__deprecated/note`; insert `limits`;
//!   section A: insert `marker = true` and the synthesized
//!   component `toy:marker`; callback `shout_color` upper-cases `color`.
//! - `Toy120` (1.2.0, Latest): `movement`, `color`, `legacy`, `marker`, `__deprecated`, `limits`,
//!   `condition` (Molang, the
//!   document's version), `components`.
//! - `ToyBeta`: `beta_only`.

#![allow(dead_code)]

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{ComponentMap, ComponentRef, CustomComponent, Extra};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::{Experiment, LoadRequirement};
use bedrock_addon_core::molang::{
    Molang, MolangFieldSpec, MolangFields, MolangVersionSource, MolangVisitor,
};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BETA_VERSION, ComponentSection, ComponentTable, ConvertMode, Family, Identity, Loaded, ParseCx,
    Upgrade, UpgradeCx, ValidateCx, VersionStruct, Versioned, parse_component, parse_component_map,
    parse_payload, write_payload,
};
use bedrock_addon_derive::Components;
use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;

/// The family marker.
pub struct Toy;

/// `description`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Description {
    pub identifier: String,
    #[facet(skip)]
    pub extra: Extra,
}

// ---- components ----

/// `toy:speed`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Speed {
    pub value: f32,
    #[facet(skip)]
    pub extra: Extra,
}

/// `toy:glow`, which supersedes `toy:old_glow`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Glow {
    pub level: u8,
}

/// `toy:old_glow`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct OldGlow {
    pub level: u8,
}

/// `toy:gated`: released 1.2.0, unlocked earlier by Upcoming Creator Features.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Gated {
    pub on: bool,
}

/// `toy:preview`: never released, needs the Upcoming Creator Features toggle.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Preview {
    pub on: bool,
}

/// `toy:retired`: deprecated since 1.1.0.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Retired {
    pub on: bool,
}

/// `toy:secret`: base-game packs only.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Secret {
    pub on: bool,
}

/// `toy:marker`: inserted by a step; needs a toggle no test turns on, which a synthesized
/// entry bypasses.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Marker {
    pub on: bool,
}

/// `toy:fresh`: released 1.2.0 with nothing that unlocks it earlier (an empty `requires`).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Fresh {
    pub on: bool,
}

const UCF: LoadRequirement = LoadRequirement::Experiment(Experiment::UpcomingCreatorFeatures);
const DRAFT: LoadRequirement = LoadRequirement::Experiment(Experiment::TestExperiment);

/// `toy:light`: a component with a Molang field (`#[component(molang)]`), whose version is
/// the document's.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Light {
    #[facet(default)]
    pub condition: Option<Molang>,
}

impl MolangFields for Light {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.field("condition", &CONDITION, &mut self.condition);
    }
}

/// `toy:needs`: a required member and no `Default`, so a missing `amount` is a missing required
/// member, not the default.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Needs {
    pub amount: u8,
}

/// `toy:kept`: a component whose reader logs a content-log error and keeps the component and
/// the document, as the game's particle reader does with "offset field required!".
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Kept {
    #[facet(default)]
    pub offset: Option<f32>,
}

/// Reads `toy:kept`: a missing `offset` is a kept error at the component.
fn parse_kept(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &bedrock_addon_core::load::LoadContext,
    d: &mut Diagnostics,
) -> Option<ToyComponent> {
    let component = parse_component::<Kept, ToyComponent>(slice, mode, cx, d)?;
    if matches!(&component, ToyComponent::Kept(kept) if kept.offset.is_none()) {
        d.push_kept_error(Diagnostic::new(
            DiagnosticCode::MissingRequiredField { text: None },
            Severity::Error,
            Phase::Parse,
            Location::new(None, None, JsonPath::new(), None),
        ));
    }
    Some(component)
}

/// `toy:legacy`: released 1.2.0, earlier only for the legacy creator feature packs.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Legacy {
    pub on: bool,
}

#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = Toy, custom = Custom, unknown = Unknown)]
pub enum ToyComponent {
    #[component(name = "toy:speed", parse = parse_component::<Speed, ToyComponent>)]
    Speed(Speed),
    #[component(name = "toy:glow", supersedes = "toy:old_glow", parse = parse_component::<Glow, ToyComponent>)]
    Glow(Glow),
    #[component(name = "toy:old_glow", parse = parse_component::<OldGlow, ToyComponent>)]
    OldGlow(OldGlow),
    #[component(name = "toy:gated", released = "1.2.0", requires = [UCF], parse = parse_component::<Gated, ToyComponent>)]
    Gated(Gated),
    #[component(name = "toy:preview", requires = [UCF], parse = parse_component::<Preview, ToyComponent>)]
    Preview(Preview),
    #[component(name = "toy:retired", deprecated_since = "1.1.0", parse = parse_component::<Retired, ToyComponent>)]
    Retired(Retired),
    #[component(name = "toy:secret", area = Vanilla, parse = parse_component::<Secret, ToyComponent>)]
    Secret(Secret),
    #[component(name = "toy:marker", requires = [DRAFT], parse = parse_component::<Marker, ToyComponent>)]
    Marker(Marker),
    #[component(name = "toy:fresh", released = "1.2.0", parse = parse_component::<Fresh, ToyComponent>)]
    Fresh(Fresh),
    #[component(
        name = "toy:legacy",
        released = "1.2.0",
        requires = [LoadRequirement::LegacyCreatorFeatures],
        parse = parse_component::<Legacy, ToyComponent>
    )]
    Legacy(Legacy),
    #[component(name = "toy:light", molang, parse = parse_component::<Light, ToyComponent>)]
    Light(Light),
    #[component(name = "toy:needs", parse = parse_component::<Needs, ToyComponent>)]
    Needs(Needs),
    #[component(name = "toy:kept", parse = parse_kept)]
    Kept(Kept),
    Custom(CustomComponent),
    Unknown(Value),
}

/// `demo:*` keys are custom components.
fn is_custom(key: &str) -> bool {
    key.starts_with("demo:")
}

/// Parses the raw `components` object of a payload.
fn components(
    raw: &IndexMap<String, RawJson<'_>>,
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> ComponentMap<ToyComponent> {
    let mut at = cx.at.clone();
    at.push("components");
    parse_component_map::<Toy>(
        raw,
        payload,
        &at,
        ConvertMode::Strict,
        is_custom,
        cx.load,
        d,
    )
}

/// The components as JSON.
fn write_components(map: &ComponentMap<ToyComponent>) -> Result<Value, EncodeError> {
    let mut object = facet_value::VObject::new();
    for entry in map.iter() {
        let (name, json) = match entry {
            ComponentRef::Typed { name, value, .. } => (
                name.to_owned(),
                match value {
                    ToyComponent::Speed(c) => write_payload(c)?,
                    ToyComponent::Glow(c) => write_payload(c)?,
                    ToyComponent::OldGlow(c) => write_payload(c)?,
                    ToyComponent::Gated(c) => write_payload(c)?,
                    ToyComponent::Preview(c) => write_payload(c)?,
                    ToyComponent::Retired(c) => write_payload(c)?,
                    ToyComponent::Secret(c) => write_payload(c)?,
                    ToyComponent::Marker(c) => write_payload(c)?,
                    ToyComponent::Fresh(c) => write_payload(c)?,
                    ToyComponent::Legacy(c) => write_payload(c)?,
                    ToyComponent::Light(c) => write_payload(c)?,
                    ToyComponent::Needs(c) => write_payload(c)?,
                    ToyComponent::Kept(c) => write_payload(c)?,
                    ToyComponent::Custom(c) => c.parameters.clone(),
                    ToyComponent::Unknown(v) => v.clone(),
                },
            ),
            ComponentRef::Custom(c) => (c.name.clone(), c.parameters.clone()),
            ComponentRef::Dynamic { name, json } | ComponentRef::Unknown { name, json } => {
                (name.to_owned(), json.clone())
            }
            _ => continue,
        };
        object.insert(name, json);
    }
    Ok(object.into())
}

/// Inserts `json` at `key` of a written struct, unless it is an empty object.
fn with_member(mut value: Value, key: &str, json: Value) -> Value {
    let empty = json.as_object().is_some_and(|o| o.is_empty());
    if let Some(object) = value.as_object_mut()
        && !empty
    {
        object.insert(key, json);
    }
    value
}

// ---- the structs ----

pub mod v1_0_0 {
    use super::*;

    /// The floor struct. Its components are this era's enum, `Component` (`from_only`: the
    /// types are Latest's, which own their `Component` impls).
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct Toy {
        pub description: Description,
        pub old_speed: Option<f32>,
        pub legacy_flag: Option<bool>,
        pub components: IndexMap<String, Component>,
        pub extra: Extra,
    }

    /// The 1.0.0 component set.
    #[derive(Clone, Debug, PartialEq, Components)]
    #[component(from_only, unknown = Unknown)]
    pub enum Component {
        Speed(Speed),
        OldGlow(OldGlow),
        Unknown(Value),
    }

    /// What `Toy` is parsed from: `components` as raw slices.
    #[derive(Facet)]
    struct Shell<'a> {
        description: Description,
        #[facet(default)]
        old_speed: Option<f32>,
        #[facet(default)]
        legacy_flag: Option<bool>,
        #[facet(default)]
        components: IndexMap<String, RawJson<'a>>,
        #[facet(skip)]
        extra: Extra,
    }

    /// What `Toy` is written as.
    #[derive(Facet)]
    struct Written<'a> {
        description: &'a Description,
        #[facet(default)]
        old_speed: Option<f32>,
        #[facet(default)]
        legacy_flag: Option<bool>,
        #[facet(skip)]
        extra: Extra,
    }

    impl Versioned for Toy {
        const VERSION: FormatVersion = FormatVersion::new(1, 0, 0);
    }

    impl VersionStruct for Toy {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            let shell: Shell<'_> = parse_payload(payload, cx, d)?;
            let mut components = IndexMap::new();
            for (key, raw) in &shell.components {
                let slice = payload.nested(raw)?;
                let mut found = Diagnostics::default();
                let value = match key.as_str() {
                    "toy:speed" => parse_component::<Speed, Component>(
                        slice,
                        ConvertMode::Strict,
                        cx.load,
                        &mut found,
                    ),
                    "toy:old_glow" => parse_component::<OldGlow, Component>(
                        slice,
                        ConvertMode::Strict,
                        cx.load,
                        &mut found,
                    ),
                    _ => facet_json::from_str_jsonc::<Value>(slice.text)
                        .ok()
                        .map(Component::Unknown),
                };
                for mut finding in found {
                    let mut path = cx.at.clone();
                    path.push("components");
                    path.push(key);
                    path.append(&finding.location.json_path);
                    finding.location.json_path = path;
                    d.push(finding);
                }
                if let Some(value) = value {
                    components.insert(key.clone(), value);
                }
            }
            Some(Toy {
                description: shell.description,
                old_speed: shell.old_speed,
                legacy_flag: shell.legacy_flag,
                components,
                extra: shell.extra,
            })
        }
        fn write(&self) -> Result<Value, EncodeError> {
            let written = write_payload(&Written {
                description: &self.description,
                old_speed: self.old_speed,
                legacy_flag: self.legacy_flag,
                extra: Extra::default(),
            })?;
            let mut components = facet_value::VObject::new();
            for (key, component) in &self.components {
                let json = match component {
                    Component::Speed(c) => write_payload(c)?,
                    Component::OldGlow(c) => write_payload(c)?,
                    Component::Unknown(v) => v.clone(),
                };
                components.insert(key.as_str(), json);
            }
            Ok(with_member(written, "components", components.into()))
        }
    }

    impl MolangFields for Toy {
        fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
    }

    impl Upgrade for Toy {
        type Next = super::v1_1_0::Toy;
        fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
            if self.legacy_flag.is_some() {
                let mut at = JsonPath::new();
                at.push("minecraft:toy");
                at.push("legacy_flag");
                cx.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::DroppedField { text: None },
                    Severity::Warning,
                    Phase::Upgrade,
                    Location::new(None, None, at, None),
                ));
            }
            // The 1.0.0 enum's components move into Latest's map.
            let mut components = ComponentMap::new();
            for (key, component) in self.components {
                match component {
                    Component::Speed(c) => {
                        components.insert_typed(ToyComponent::from(c), false);
                    }
                    Component::OldGlow(c) => {
                        components.insert_typed(ToyComponent::from(c), false);
                    }
                    Component::Unknown(json) => components.insert_unknown(&key, json),
                }
            }
            super::v1_1_0::Toy {
                description: self.description,
                speed: self.old_speed,
                color: "red".to_owned(),
                note: None,
                components,
                extra: self.extra,
            }
        }
    }
}

pub mod v1_1_0 {
    use super::*;

    /// The middle struct.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Toy {
        pub description: Description,
        pub speed: Option<f32>,
        pub color: String,
        pub note: Option<String>,
        pub components: ComponentMap<ToyComponent>,
        pub extra: Extra,
    }

    /// What `Toy` is parsed from: `components` as raw slices.
    #[derive(Facet)]
    struct Shell<'a> {
        description: Description,
        #[facet(default)]
        speed: Option<f32>,
        #[facet(default)]
        color: Option<String>,
        #[facet(default)]
        note: Option<String>,
        #[facet(default)]
        components: IndexMap<String, RawJson<'a>>,
        #[facet(skip)]
        extra: Extra,
    }

    /// What `Toy` is written as.
    #[derive(Facet)]
    struct Written<'a> {
        description: &'a Description,
        #[facet(default)]
        speed: Option<f32>,
        color: &'a str,
        #[facet(default)]
        note: Option<&'a str>,
        #[facet(skip)]
        extra: Extra,
    }

    impl Versioned for Toy {
        const VERSION: FormatVersion = FormatVersion::new(1, 1, 0);
    }

    impl VersionStruct for Toy {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            let shell: Shell<'_> = parse_payload(payload, cx, d)?;
            Some(Toy {
                components: components(&shell.components, payload, cx, d),
                description: shell.description,
                speed: shell.speed,
                color: shell.color.unwrap_or_else(|| "red".to_owned()),
                note: shell.note,
                extra: shell.extra,
            })
        }
        fn write(&self) -> Result<Value, EncodeError> {
            let written = write_payload(&Written {
                description: &self.description,
                speed: self.speed,
                color: &self.color,
                note: self.note.as_deref(),
                extra: Extra::default(),
            })?;
            Ok(with_member(
                written,
                "components",
                write_components(&self.components)?,
            ))
        }
    }

    impl MolangFields for Toy {
        fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
    }

    impl Upgrade for Toy {
        type Next = super::Toy120;
        fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
            let mut components = self.components;
            let mut marker = None;
            if !cx.skip_section_a {
                marker = Some(true);
                components.insert_typed(ToyComponent::Marker(Marker { on: true }), true);
            }
            let legacy = (cx.original_version < FormatVersion::new(1, 0, 5)).then_some(true);
            super::Toy120 {
                description: self.description,
                movement: self.speed.map(|speed| Movement { speed }),
                color: shout_color(self.color),
                legacy,
                upgraded_as: Some((cx.identifier.to_owned(), cx.actor_type)),
                marker,
                deprecated: self.note.map(|note| Sidecar { note: Some(note) }),
                limits: Some(Limits { min: 0, max: 10 }),
                condition: None,
                components,
                extra: self.extra,
            }
        }
    }
}

/// The callback of step 1.1.0 → 1.2.0.
pub fn shout_color(color: String) -> String {
    color.to_uppercase()
}

/// `movement`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Movement {
    pub speed: f32,
}

/// `__deprecated`: the side-car of `note`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Sidecar {
    #[facet(default)]
    pub note: Option<String>,
}

/// `limits`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Limits {
    pub min: i32,
    pub max: i32,
}

/// The newest struct.
#[derive(Clone, Debug, PartialEq)]
pub struct Toy120 {
    pub description: Description,
    pub movement: Option<Movement>,
    pub color: String,
    pub legacy: Option<bool>,
    /// What the 1.1.0 step saw: `UpgradeCx::identifier` and `actor_type` (not JSON).
    pub upgraded_as: Option<(String, ActorType)>,
    pub marker: Option<bool>,
    pub deprecated: Option<Sidecar>,
    pub limits: Option<Limits>,
    pub condition: Option<Molang>,
    pub components: ComponentMap<ToyComponent>,
    pub extra: Extra,
}

/// What `Toy120` is parsed from.
#[derive(Facet)]
struct Shell120<'a> {
    description: Description,
    #[facet(default)]
    movement: Option<Movement>,
    #[facet(default)]
    color: Option<String>,
    #[facet(default)]
    legacy: Option<bool>,
    #[facet(default)]
    marker: Option<bool>,
    #[facet(default, rename = "__deprecated")]
    deprecated: Option<Sidecar>,
    #[facet(default)]
    limits: Option<Limits>,
    #[facet(default)]
    condition: Option<Molang>,
    #[facet(default)]
    components: IndexMap<String, RawJson<'a>>,
    #[facet(skip)]
    extra: Extra,
}

/// What `Toy120` is written as.
#[derive(Facet)]
struct Written120<'a> {
    description: &'a Description,
    #[facet(default)]
    movement: Option<&'a Movement>,
    color: &'a str,
    #[facet(default)]
    legacy: Option<bool>,
    #[facet(default)]
    marker: Option<bool>,
    #[facet(default, rename = "__deprecated")]
    deprecated: Option<&'a Sidecar>,
    #[facet(default)]
    limits: Option<&'a Limits>,
    #[facet(skip)]
    extra: Extra,
}

impl Versioned for Toy120 {
    const VERSION: FormatVersion = FormatVersion::new(1, 2, 0);
}

impl VersionStruct for Toy120 {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let shell: Shell120<'_> = parse_payload(payload, cx, d)?;
        Some(Toy120 {
            components: components(&shell.components, payload, cx, d),
            description: shell.description,
            movement: shell.movement,
            color: shell.color.unwrap_or_else(|| "RED".to_owned()),
            legacy: shell.legacy,
            upgraded_as: None,
            marker: shell.marker,
            deprecated: shell.deprecated,
            limits: shell.limits,
            condition: shell.condition,
            extra: shell.extra,
        })
    }
    fn write(&self) -> Result<Value, EncodeError> {
        let written = write_payload(&Written120 {
            description: &self.description,
            movement: self.movement.as_ref(),
            color: &self.color,
            legacy: self.legacy,
            marker: self.marker,
            deprecated: self.deprecated.as_ref(),
            limits: self.limits.as_ref(),
            extra: Extra::default(),
        })?;
        Ok(with_member(
            written,
            "components",
            write_components(&self.components)?,
        ))
    }
}

/// `condition`: a Molang field whose version is the document's.
pub const CONDITION: MolangFieldSpec = MolangFieldSpec {
    version: MolangVersionSource::FormatVersion,
    ..MolangFieldSpec::DEFAULT
};

impl MolangFields for Toy120 {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.field("condition", &CONDITION, &mut self.condition);
        // The derive's walk of the component enum, through the map.
        visitor.nested("components", &mut self.components);
    }
}

/// The beta struct.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct ToyBeta {
    pub description: Description,
    #[facet(default)]
    pub beta_only: Option<bool>,
    #[facet(skip)]
    pub extra: Extra,
}

impl Versioned for ToyBeta {
    const VERSION: FormatVersion = BETA_VERSION;
}

impl VersionStruct for ToyBeta {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_payload(payload, cx, d)
    }
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl MolangFields for ToyBeta {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

pub mod history {
    use super::*;

    bedrock_addon_core::version_chain! {
        family = Toy;
        /// The struct the file's `format_version` selected.
        #[derive(Clone, Debug, PartialEq)]
        pub enum Any {
            V1_0_0(v1_0_0::Toy),
            V1_1_0(v1_1_0::Toy),
            V1_2_0(Toy120),
        }
        beta = Beta(ToyBeta);
    }
}

impl Family for Toy {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const DIRECTORIES: &'static [&'static str] = &["toys"];
    const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
    const SKIP_UPGRADES_KEY: Option<&'static str> = Some("do_not_upgrade");
    type Latest = Toy120;
    type Beta = ToyBeta;
    type Component = ToyComponent;
    type Any = history::Any;

    fn components() -> &'static ComponentTable<Self> {
        ToyComponent::TABLE
    }

    fn actor_type(identity: &Identity) -> ActorType {
        ActorType::for_entity(
            identity.identifier.as_deref().unwrap_or_default(),
            identity.runtime_identifier.as_deref(),
        )
    }

    fn component_sections(v: &mut Toy120) -> Vec<ComponentSection<'_, ToyComponent>> {
        let mut at = JsonPath::new();
        at.push("components");
        vec![ComponentSection::new(at, false, &mut v.components)]
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        // A family rule: the colour `BLACK` is not allowed.
        if let Loaded::Stable(toy) = v
            && toy.color == "BLACK"
        {
            let mut at = cx.at.clone();
            at.push("color");
            cx.diagnostics.push(Diagnostic::new(
                DiagnosticCode::InvalidValue { text: None },
                Severity::Error,
                Phase::Validate,
                Location::new(None, None, at, None),
            ));
        }
    }
}
