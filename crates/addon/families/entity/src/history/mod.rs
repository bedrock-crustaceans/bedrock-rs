//! The entity documents as written in older format versions, and the steps that carry each one
//! to the next.
//!
//! Every struct from 1.8.0 to 1.26.40 holds the [`Payload`] as written: vanilla keeps the
//! components of these slices as JSON until the struct that types them, and each step rewrites
//! that JSON the way vanilla's fixer and legacy upgrade callbacks do. The step from 1.26.40 reads
//! the result in the newest struct's shape, where each component is typed: read strictly when
//! the document's own version already typed it, leniently otherwise. The 1.26.50 struct is that
//! shape too ([`v1_26_50`]); its step to the newest, [`Entity`] (1.26.60), only converts the keys
//! 1.26.60 types first, which its read already did.
//!
//! Every op of a step belongs to one of the game's three sections, named A, B and C in the
//! step files: A holds the per-type and generic rewrites (skipped when the payload has
//! `do_not_upgrade` or the document is at the engine version), B the component upgrades that
//! always run, and C the conversions to typed structs.
// One file per struct; the shared struct is `Payload`.

mod ops;

pub mod v1_10_0;
pub mod v1_11_0;
pub mod v1_12_0;
pub mod v1_13_0;
pub mod v1_15_0;
pub mod v1_16_0;
pub mod v1_16_100;
pub mod v1_16_200;
pub mod v1_17_10;
pub mod v1_17_20;
pub mod v1_18_0;
pub mod v1_18_10;
pub mod v1_18_20;
pub mod v1_18_30;
pub mod v1_19_0;
pub mod v1_19_50;
pub mod v1_19_60;
pub mod v1_20_0;
pub mod v1_20_60;
pub mod v1_20_70;
pub mod v1_20_80;
pub mod v1_21_0;
pub mod v1_21_10;
pub mod v1_21_100;
pub mod v1_21_110;
pub mod v1_21_120;
pub mod v1_21_130;
pub mod v1_21_20;
pub mod v1_21_40;
pub mod v1_21_60;
pub mod v1_21_70;
pub mod v1_21_80;
pub mod v1_21_89;
pub mod v1_21_90;
pub mod v1_26_0;
pub mod v1_26_10;
pub mod v1_26_20;
pub mod v1_26_30;
pub mod v1_26_40;
pub mod v1_26_50;
pub mod v1_8_0;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::id::EntityId;
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::molang::{MolangContext, pass_one};
use bedrock_addon_core::schema;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{Family, Identity, ParseCx, UpgradeCx, parse_payload};
use facet::Facet;
use indexmap::IndexMap;

use super::{Beta, Entity};
use crate::read::{Reader, read_entity, span_of};

bedrock_addon_core::version_chain! {
    family = Entity;
    /// The struct the file's `format_version` selected.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// [1.8.0, 1.10.0), the floor.
        V1_8_0(v1_8_0::Entity),
        /// [1.10.0, 1.11.0).
        V1_10_0(v1_10_0::Entity),
        /// [1.11.0, 1.12.0).
        V1_11_0(v1_11_0::Entity),
        /// [1.12.0, 1.13.0).
        V1_12_0(v1_12_0::Entity),
        /// [1.13.0, 1.15.0).
        V1_13_0(v1_13_0::Entity),
        /// [1.15.0, 1.16.0).
        V1_15_0(v1_15_0::Entity),
        /// [1.16.0, 1.16.100).
        V1_16_0(v1_16_0::Entity),
        /// [1.16.100, 1.16.200).
        V1_16_100(v1_16_100::Entity),
        /// [1.16.200, 1.17.10).
        V1_16_200(v1_16_200::Entity),
        /// [1.17.10, 1.17.20).
        V1_17_10(v1_17_10::Entity),
        /// [1.17.20, 1.18.0).
        V1_17_20(v1_17_20::Entity),
        /// [1.18.0, 1.18.10).
        V1_18_0(v1_18_0::Entity),
        /// [1.18.10, 1.18.20).
        V1_18_10(v1_18_10::Entity),
        /// [1.18.20, 1.18.30).
        V1_18_20(v1_18_20::Entity),
        /// [1.18.30, 1.19.0).
        V1_18_30(v1_18_30::Entity),
        /// [1.19.0, 1.19.50).
        V1_19_0(v1_19_0::Entity),
        /// [1.19.50, 1.19.60).
        V1_19_50(v1_19_50::Entity),
        /// [1.19.60, 1.20.0).
        V1_19_60(v1_19_60::Entity),
        /// [1.20.0, 1.20.60).
        V1_20_0(v1_20_0::Entity),
        /// [1.20.60, 1.20.70).
        V1_20_60(v1_20_60::Entity),
        /// [1.20.70, 1.20.80).
        V1_20_70(v1_20_70::Entity),
        /// [1.20.80, 1.21.0).
        V1_20_80(v1_20_80::Entity),
        /// [1.21.0, 1.21.10).
        V1_21_0(v1_21_0::Entity),
        /// [1.21.10, 1.21.20).
        V1_21_10(v1_21_10::Entity),
        /// [1.21.20, 1.21.40).
        V1_21_20(v1_21_20::Entity),
        /// [1.21.40, 1.21.60).
        V1_21_40(v1_21_40::Entity),
        /// [1.21.60, 1.21.70).
        V1_21_60(v1_21_60::Entity),
        /// [1.21.70, 1.21.80).
        V1_21_70(v1_21_70::Entity),
        /// [1.21.80, 1.21.89).
        V1_21_80(v1_21_80::Entity),
        /// [1.21.89, 1.21.90).
        V1_21_89(v1_21_89::Entity),
        /// [1.21.90, 1.21.100).
        V1_21_90(v1_21_90::Entity),
        /// [1.21.100, 1.21.110).
        V1_21_100(v1_21_100::Entity),
        /// [1.21.110, 1.21.120).
        V1_21_110(v1_21_110::Entity),
        /// [1.21.120, 1.21.130).
        V1_21_120(v1_21_120::Entity),
        /// [1.21.130, 1.26.0).
        V1_21_130(v1_21_130::Entity),
        /// [1.26.0, 1.26.10).
        V1_26_0(v1_26_0::Entity),
        /// [1.26.10, 1.26.20).
        V1_26_10(v1_26_10::Entity),
        /// [1.26.20, 1.26.30).
        V1_26_20(v1_26_20::Entity),
        /// [1.26.30, 1.26.40).
        V1_26_30(v1_26_30::Entity),
        /// [1.26.40, 1.26.50).
        V1_26_40(v1_26_40::Entity),
        /// [1.26.50, 1.26.60).
        V1_26_50(v1_26_50::Entity),
        /// [1.26.60, the engine version]: the newest struct.
        V1_26_60(Entity),
    }
    beta = /// `"format_version": "beta"`.
        Beta(Beta);
}

/// The payload of an older entity document, as written: what vanilla's slices up to 1.26.40
/// hold before the struct that types each component.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Payload {
    /// `description`, as written (an object).
    pub description: Value,
    /// `components`; `None` when the payload has none.
    pub components: Option<IndexMap<String, Value>>,
    /// `component_groups`: each group's components (an object), or `null` below 1.21.89.
    pub component_groups: Option<IndexMap<String, Value>>,
    /// `events`, each event's body as written.
    pub events: Option<IndexMap<String, Value>>,
    /// `brain` (format 1.21.130 onward).
    pub brain: Option<Value>,
    /// `upgrades` (format 1.26.20 onward).
    pub upgrades: Option<Value>,
    /// `do_not_upgrade`: present (any value) when the author asked vanilla to skip its
    /// section-A rewrites.
    pub do_not_upgrade: Option<Value>,
}

/// What a [`Payload`] is parsed from.
#[derive(Facet)]
#[facet(schema::unknown = "keep")]
struct Shell {
    #[facet(default)]
    description: Option<Value>,
    #[facet(default)]
    components: Option<Value>,
    #[facet(default)]
    component_groups: Option<Value>,
    #[facet(default)]
    events: Option<Value>,
    #[facet(default)]
    brain: Option<Value>,
    #[facet(default)]
    upgrades: Option<Value>,
    #[facet(default)]
    do_not_upgrade: Option<Value>,
    // The pre-1.8 in-payload version (`Family::payload_version`): consumed, not kept.
    #[facet(default)]
    format_version: Option<Value>,
}

/// The first struct whose `component_groups` may not hold `null`.
const GROUPS_CORRECTED: FormatVersion = FormatVersion::new(1, 21, 89);

/// A member that is not an object where one must be (the finding is reported).
struct Invalid;

/// A member that must be an object when present: `Ok(None)` when absent or `null`,
/// `Ok(Some(members))` for an object, `Err` (an `InvalidType` error) for anything else.
fn object_member(
    value: Option<Value>,
    key: &str,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Result<Option<IndexMap<String, Value>>, Invalid> {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return Ok(None);
    };
    if let Some(object) = value.as_object() {
        return Ok(Some(members(object)));
    }
    d.push(Diagnostic::error(DiagnosticCode::InvalidType { text: None }).at(cx.at.child(key)));
    Err(Invalid)
}

/// An object's members, in order.
pub(crate) fn members(object: &facet_value::VObject) -> IndexMap<String, Value> {
    object
        .iter()
        .map(|(k, v)| (k.as_str().to_owned(), v.clone()))
        .collect()
}

impl Payload {
    /// Parses a payload as the struct of `version` holds it.
    pub(crate) fn parse(
        payload: JsonSlice<'_>,
        version: FormatVersion,
        cx: &ParseCx<'_>,
        d: &mut Diagnostics,
    ) -> Option<Self> {
        let shell: Shell = parse_payload(payload, cx, d)?;
        let description = shell.description.unwrap_or(Value::NULL);
        if !description.is_object() {
            // A payload without a `description` names no identifier; from 1.21.89
            // vanilla's cereal parse requires it. Vanilla skips such a file silently, with no
            // content-log line, and ships two in its own packs, so in a base-game pack the skip is
            // an information.
            let severity = if cx.load.is_base_game_pack {
                Severity::Info
            } else {
                Severity::Error
            };
            d.push(
                Diagnostic::error(DiagnosticCode::MissingIdentifier { text: None })
                    .at(cx.at.child("description"))
                    .span(span_of(payload))
                    .severity(severity),
            );
            return None;
        }
        let components = object_member(shell.components, "components", cx, d).ok()?;
        let mut component_groups =
            object_member(shell.component_groups, "component_groups", cx, d).ok()?;
        if version >= GROUPS_CORRECTED
            && let Some(groups) = &mut component_groups
        {
            let groups_at = cx.at.child("component_groups");
            groups.retain(|name, group| {
                if group.is_object() {
                    return true;
                }
                // the typed slice from 1.21.89 refuses the entity (cereal's
                // "expected an object" is an error, and the entity's level-3 "failed to load" line
                // follows): an error.
                d.push(
                    Diagnostic::error(DiagnosticCode::InvalidValue { text: None })
                        .at(groups_at.child(name)),
                );
                false
            });
        }
        let events = object_member(shell.events, "events", cx, d).ok()?;
        if let Some(found) = crate::read::payload_version_unread(shell.format_version.as_ref(), cx)
        {
            d.push(found);
        }
        if shell
            .do_not_upgrade
            .as_ref()
            .is_some_and(|flag| flag.as_bool() != Some(true))
        {
            // the key acts by its presence, "any value"; a value that is not
            // `true` skips the rewrites all the same, which is told (an information, the game
            // does not read the value).
            d.push(
                Diagnostic::info(DiagnosticCode::InvalidType { text: None })
                    .at(cx.at.child("do_not_upgrade")),
            );
        }
        Some(Self {
            description,
            components,
            component_groups,
            events,
            brain: shell.brain,
            upgrades: shell.upgrades,
            do_not_upgrade: shell.do_not_upgrade,
        })
    }

    /// The payload as JSON.
    pub(crate) fn write(&self) -> Value {
        let mut object = facet_value::VObject::new();
        object.insert("description", self.description.clone());
        let map = |members: &IndexMap<String, Value>| {
            let mut object = facet_value::VObject::new();
            for (key, value) in members {
                object.insert(key.as_str(), value.clone());
            }
            Value::from(object)
        };
        if let Some(components) = &self.components {
            object.insert("components", map(components));
        }
        if let Some(groups) = &self.component_groups {
            object.insert("component_groups", map(groups));
        }
        if let Some(events) = &self.events {
            object.insert("events", map(events));
        }
        if let Some(brain) = &self.brain {
            object.insert("brain", brain.clone());
        }
        if let Some(upgrades) = &self.upgrades {
            object.insert("upgrades", upgrades.clone());
        }
        if let Some(flag) = &self.do_not_upgrade {
            object.insert("do_not_upgrade", flag.clone());
        }
        // The unknown members are the document's, written from its table.
        object.into()
    }
}

/// The last step's conversion: the payload, rewritten by every step before, read as the newest
/// struct. Each component the document's own version already typed is read strictly, any other
/// leniently (vanilla's legacy upgrade callbacks); the Molang fields take their versions here,
/// as a parse would give them. The findings are placed below the payload key, without spans (the
/// text is the steps' output, not the file).
pub(crate) fn into_latest(payload: &Payload, cx: &mut UpgradeCx<'_>) -> Entity {
    let at = JsonPath::new().child(<Entity as Family>::PAYLOAD_KEY);
    // The payload's unknown members are read again with it: the newest struct keeps those it
    // does not know, where it keeps them.
    let mut written = payload.write();
    if let Some(object) = written.as_object_mut() {
        for (key, value) in cx.diagnostics.unknown_mut().take_all(&at) {
            object.insert(&*key, value);
        }
    }
    let Ok(text) = bedrock_addon_core::json::to_json(&written) else {
        // A non-finite number cannot come from JSON text, so this is not reached.
        cx.diagnostics.push(
            Diagnostic::error(DiagnosticCode::InternalError)
                .at(at)
                .phase(Phase::Upgrade),
        );
        return placeholder(payload);
    };
    let mut reader = Reader {
        load: cx.load,
        original_version: cx.original_version,
        d: cx.diagnostics,
        spans: false,
    };
    let Some(mut entity) = read_entity(&mut reader, JsonSlice::new(&text), &at) else {
        return placeholder(payload);
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
    let base = Location::new(None, None, at, None);
    pass_one(&mut entity, &molang, &base, cx.diagnostics);
    entity
}

/// The value of a payload the newest struct could not read (its error rejects the document).
fn placeholder(payload: &Payload) -> Entity {
    let identifier = payload
        .description
        .as_object()
        .and_then(|d| d.get("identifier"))
        .and_then(Value::as_string)
        .map_or_else(String::new, |s| s.as_str().to_owned());
    Entity::new(EntityId::parse(identifier))
}

/// Declares the struct of one older version: the [`Payload`] as that version's slice holds it.
macro_rules! legacy_struct {
    ($major:literal, $minor:literal, $patch:literal) => {
        use bedrock_addon_core::diagnostic::Diagnostics;
        use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
        use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
        use bedrock_addon_core::version::FormatVersion;
        use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned};

        use super::Payload;

        #[doc = concat!(
                    "An entity document as written in format ", $major, ".", $minor, ".", $patch,
                    " (up to the next struct's version): its payload as written."
                )]
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Entity(pub Payload);

        impl Versioned for Entity {
            const VERSION: FormatVersion = FormatVersion::new($major, $minor, $patch);
        }

        impl VersionStruct for Entity {
            fn parse(
                payload: JsonSlice<'_>,
                cx: &ParseCx<'_>,
                d: &mut Diagnostics,
            ) -> Option<Self> {
                Payload::parse(payload, Self::VERSION, cx, d).map(Self)
            }

            fn write(&self) -> Result<Value, EncodeError> {
                Ok(self.0.write())
            }
        }

        /// The components of this version are kept as JSON: no Molang field is typed yet.
        impl MolangFields for Entity {
            fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
        }
    };
}
pub(crate) use legacy_struct;

impl Any {
    /// The payload of an older struct; `None` for the newest and the beta struct.
    pub(crate) fn payload(&self) -> Option<&Payload> {
        match self {
            Any::V1_8_0(v1_8_0::Entity(payload))
            | Any::V1_10_0(v1_10_0::Entity(payload))
            | Any::V1_11_0(v1_11_0::Entity(payload))
            | Any::V1_12_0(v1_12_0::Entity(payload))
            | Any::V1_13_0(v1_13_0::Entity(payload))
            | Any::V1_15_0(v1_15_0::Entity(payload))
            | Any::V1_16_0(v1_16_0::Entity(payload))
            | Any::V1_16_100(v1_16_100::Entity(payload))
            | Any::V1_16_200(v1_16_200::Entity(payload))
            | Any::V1_17_10(v1_17_10::Entity(payload))
            | Any::V1_17_20(v1_17_20::Entity(payload))
            | Any::V1_18_0(v1_18_0::Entity(payload))
            | Any::V1_18_10(v1_18_10::Entity(payload))
            | Any::V1_18_20(v1_18_20::Entity(payload))
            | Any::V1_18_30(v1_18_30::Entity(payload))
            | Any::V1_19_0(v1_19_0::Entity(payload))
            | Any::V1_19_50(v1_19_50::Entity(payload))
            | Any::V1_19_60(v1_19_60::Entity(payload))
            | Any::V1_20_0(v1_20_0::Entity(payload))
            | Any::V1_20_60(v1_20_60::Entity(payload))
            | Any::V1_20_70(v1_20_70::Entity(payload))
            | Any::V1_20_80(v1_20_80::Entity(payload))
            | Any::V1_21_0(v1_21_0::Entity(payload))
            | Any::V1_21_10(v1_21_10::Entity(payload))
            | Any::V1_21_20(v1_21_20::Entity(payload))
            | Any::V1_21_40(v1_21_40::Entity(payload))
            | Any::V1_21_60(v1_21_60::Entity(payload))
            | Any::V1_21_70(v1_21_70::Entity(payload))
            | Any::V1_21_80(v1_21_80::Entity(payload))
            | Any::V1_21_89(v1_21_89::Entity(payload))
            | Any::V1_21_90(v1_21_90::Entity(payload))
            | Any::V1_21_100(v1_21_100::Entity(payload))
            | Any::V1_21_110(v1_21_110::Entity(payload))
            | Any::V1_21_120(v1_21_120::Entity(payload))
            | Any::V1_21_130(v1_21_130::Entity(payload))
            | Any::V1_26_0(v1_26_0::Entity(payload))
            | Any::V1_26_10(v1_26_10::Entity(payload))
            | Any::V1_26_20(v1_26_20::Entity(payload))
            | Any::V1_26_30(v1_26_30::Entity(payload))
            | Any::V1_26_40(v1_26_40::Entity(payload)) => Some(payload),
            Any::V1_26_50(_) | Any::V1_26_60(_) | Any::Beta(_) => None,
        }
    }
}

/// What the steps of a parsed document read besides its struct: its identity and whether it
/// carries the skip key.
pub(crate) fn identity(any: &Any) -> (Identity, bool) {
    let text = |value: Option<&Value>| {
        value
            .and_then(Value::as_string)
            .map(|s| s.as_str().to_owned())
    };
    if let Some(payload) = any.payload() {
        let description = payload.description.as_object();
        let identity = Identity {
            identifier: text(description.and_then(|d| d.get("identifier"))),
            runtime_identifier: text(description.and_then(|d| d.get("runtime_identifier"))),
        };
        return (identity, payload.do_not_upgrade.is_some());
    }
    let entity = match any {
        Any::V1_26_50(v1_26_50::Entity(entity)) | Any::V1_26_60(entity) => entity,
        Any::Beta(beta) => &beta.0,
        _ => return (Identity::default(), false),
    };
    let description = &entity.description;
    let identity = Identity {
        identifier: Some(description.identifier.as_str().to_owned()),
        runtime_identifier: description
            .runtime_identifier
            .as_ref()
            .map(|id| id.as_str().to_owned()),
    };
    (identity, false)
}
