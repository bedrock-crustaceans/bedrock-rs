//! The item side-car `ItemDeprecatedData` (JSON `__deprecated_components`).

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Disposition, Node, Profile, Read};
use bedrock_addon_core::schema::{self, FromJson, Schema, read_document, read_value};
use bedrock_addon_core::shared::EventTrigger;
use bedrock_addon_core::versioning::{BeyondWidth, MemberFailure, NullHandling, Reading, Scalars};
use facet::Facet;
use facet_value::VObject;

use super::component::Digger;

/// Data older item formats moved out of the components (JSON `__deprecated_components`).
///
/// The triggers of 1.20.20–1.20.40 components that no later format binds, the legacy
/// render offsets, and (from 1.26.30) a triggered digger and the list form of the custom
/// components. Each trigger sits under an inner key, as vanilla's legacy trigger components
/// bind it (`on_use` holds `{"on_use_trigger": …}`).
///
/// It is read as the crate's own reading of a document the game does not load ([`SIDE_CAR`]):
/// every finding is an information, a member of the wrong shape is kept in `extra` as written.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
#[non_exhaustive]
pub struct ItemDeprecatedData {
    /// `food.on_consume` (JSON `on_consume_trigger` › `on_consume_trigger`; from 1.20.30).
    #[facet(rename = "on_consume_trigger", schema::read_with = on_consume_slot, schema::or_kept = "told")]
    pub on_consume: Option<EventTrigger>,
    /// `minecraft:on_use` (JSON `on_use` › `on_use_trigger`; from 1.20.40).
    #[facet(schema::read_with = on_use_slot, schema::or_kept = "told")]
    pub on_use: Option<EventTrigger>,
    /// `minecraft:on_use_on` (JSON `on_use_on_trigger` › `on_use_on`; from 1.20.40).
    #[facet(rename = "on_use_on_trigger", schema::read_with = on_use_on_slot, schema::or_kept = "told")]
    pub on_use_on: Option<EventTrigger>,
    /// `minecraft:render_offsets`, as written (JSON `render_offsets`; from 1.20.40).
    pub render_offsets: Option<Value>,
    /// `weapon.on_hurt_entity` (JSON `on_hurt_actor_trigger` › `on_hurt_actor`; from 1.20.40).
    #[facet(rename = "on_hurt_actor_trigger", schema::read_with = on_hurt_actor_slot, schema::or_kept = "told")]
    pub on_hurt_actor: Option<EventTrigger>,
    /// `weapon.on_not_hurt_entity` (JSON `on_hit_actor_trigger` › `on_hit_actor`; from
    /// 1.20.40).
    #[facet(rename = "on_hit_actor_trigger", schema::read_with = on_hit_actor_slot, schema::or_kept = "told")]
    pub on_hit_actor: Option<EventTrigger>,
    /// `weapon.on_hit_block` (JSON `on_hit_block_trigger` › `on_hit_block`; from 1.20.40).
    #[facet(rename = "on_hit_block_trigger", schema::read_with = on_hit_block_slot, schema::or_kept = "told")]
    pub on_hit_block: Option<EventTrigger>,
    /// `chargeable.on_complete` (JSON `on_complete_trigger` › `on_complete`; from 1.20.50).
    #[facet(rename = "on_complete_trigger", schema::read_with = on_complete_slot, schema::or_kept = "told")]
    pub on_complete: Option<EventTrigger>,
    /// A digger with triggers, copied by the 1.26.30 upgrade (JSON `legacy_on_dig_trigger`, a
    /// key of the crate's own).
    #[facet(rename = "legacy_on_dig_trigger", schema::read_with = digger, schema::or_kept = "silent")]
    pub legacy_on_dig: Option<Digger>,
    /// The list form of `minecraft:custom_components`, moved by the 1.26.30 upgrade (JSON
    /// `custom_components_v1`, a key of the crate's own), as written.
    pub custom_components_v1: Option<Value>,
    /// Other members.
    pub extra: UnknownMembers,
}

/// How the side-car is read: nothing a member holds fails the document, a `null` is a member of
/// the wrong shape, a value no type holds is kept as written.
const SIDE_CAR: Reading = Reading::OWN
    .with_null(NullHandling::Rejected)
    .with_scalars(Scalars::Strict)
    .with_beyond_width(BeyondWidth::Error)
    .with_on_member_failure(MemberFailure::DefaultAndTell);

/// A trigger slot `{"<inner>": trigger}`: another member beside the trigger has no place in the
/// slot and is told where it is; a slot that is no such object, or holds no trigger, is a
/// wrong type at the slot.
fn slot(inner: &str, node: &Node<'_>, read: &mut Read<'_>) -> Option<EventTrigger> {
    let members = read.object(node)?;
    let trigger = members
        .get(inner)
        .and_then(|trigger| EventTrigger::try_from(trigger.value().clone()).ok());
    let Some(trigger) = trigger else {
        read.wrong_type(node);
        return None;
    };
    for (key, member) in members.iter() {
        if key != inner {
            read.unknown_member(&member, Disposition::Report);
        }
    }
    Some(trigger)
}

/// The reads of the trigger slots, by their inner keys.
macro_rules! slots {
    ($($name:ident: $inner:literal),+ $(,)?) => {
        $(
            fn $name(node: &Node<'_>, read: &mut Read<'_>) -> Option<EventTrigger> {
                slot($inner, node, read)
            }
        )+
    };
}

slots! {
    on_consume_slot: "on_consume_trigger",
    on_use_slot: "on_use_trigger",
    on_use_on_slot: "on_use_on",
    on_hurt_actor_slot: "on_hurt_actor",
    on_hit_actor_slot: "on_hit_actor",
    on_hit_block_slot: "on_hit_block",
    on_complete_slot: "on_complete",
}

/// A digger that reads whole and strictly, as a component's own text reads (one that does not is
/// kept as written, in silence); its unknown members are kept and told where they are.
fn digger(node: &Node<'_>, read: &mut Read<'_>) -> Option<Digger> {
    read_value::<Digger>(node.value())?;
    Digger::from_json(node, read)
}

/// The trigger slots: JSON key, inner key, field.
type Slot = (
    &'static str,
    &'static str,
    fn(&mut ItemDeprecatedData) -> &mut Option<EventTrigger>,
);

/// The trigger slots in write order.
// `on_use` holds `on_use_trigger`.
const SLOTS: [Slot; 7] = [
    ("on_consume_trigger", "on_consume_trigger", |d| {
        &mut d.on_consume
    }),
    ("on_use", "on_use_trigger", |d| &mut d.on_use),
    ("on_use_on_trigger", "on_use_on", |d| &mut d.on_use_on),
    ("on_hurt_actor_trigger", "on_hurt_actor", |d| {
        &mut d.on_hurt_actor
    }),
    ("on_hit_actor_trigger", "on_hit_actor", |d| {
        &mut d.on_hit_actor
    }),
    ("on_hit_block_trigger", "on_hit_block", |d| {
        &mut d.on_hit_block
    }),
    ("on_complete_trigger", "on_complete", |d| &mut d.on_complete),
];

impl ItemDeprecatedData {
    /// Whether nothing is stored.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Reads the side-car from its JSON ([`SIDE_CAR`]); a value that is no object is an
    /// `InvalidType` information at `at`.
    pub(crate) fn from_value(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Self {
        let mut read = Read::new(SIDE_CAR, d)
            .profile(Profile::EVERY_FAILURE)
            .unknown_severity(Severity::Info);
        read_document(&Node::new(value, at.clone()), &mut read).unwrap_or_default()
    }

    /// The side-car as JSON.
    pub(crate) fn to_value(&self) -> Result<Value, bedrock_addon_core::json::EncodeError> {
        let mut object = VObject::new();
        let mut copy = self.clone();
        for (outer, inner, field) in SLOTS {
            if let Some(trigger) = field(&mut copy).as_ref() {
                let mut wrapper = VObject::new();
                wrapper.insert(inner, trigger_value(trigger));
                object.insert(outer, Value::from(wrapper));
            }
            if outer == "on_use_on_trigger"
                && let Some(offsets) = &self.render_offsets
            {
                object.insert("render_offsets", offsets.clone());
            }
        }
        if let Some(digger) = &self.legacy_on_dig {
            object.insert(
                "legacy_on_dig_trigger",
                bedrock_addon_core::versioning::write_payload(digger)?,
            );
        }
        if let Some(list) = &self.custom_components_v1 {
            object.insert("custom_components_v1", list.clone());
        }
        for (key, value) in self.extra.iter() {
            object.insert(key, value.clone());
        }
        Ok(Value::from(object))
    }
}

/// A trigger as JSON (its own writer; a trigger always has a JSON form).
pub(crate) fn trigger_value(trigger: &EventTrigger) -> Value {
    Value::try_from(trigger).unwrap_or(Value::NULL)
}
