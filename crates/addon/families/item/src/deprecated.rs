//! The item side-car `ItemDeprecatedData` (JSON `__deprecated_components`).

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::shared::EventTrigger;
use facet_value::VObject;

use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;

use super::component::Digger;

/// Data older item formats moved out of the components (JSON `__deprecated_components`).
///
/// The triggers of 1.20.20–1.20.40 components that no later format binds, the legacy
/// render offsets, and (from 1.26.30) a triggered digger and the list form of the custom
/// components. Each trigger sits under an inner key, as vanilla's legacy trigger components
/// bind it (`on_use` holds `{"on_use_trigger": …}`).
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct ItemDeprecatedData {
    /// `food.on_consume` (JSON `on_consume_trigger` › `on_consume_trigger`; from 1.20.30).
    pub on_consume: Option<EventTrigger>,
    /// `minecraft:on_use` (JSON `on_use` › `on_use_trigger`; from 1.20.40).
    pub on_use: Option<EventTrigger>,
    /// `minecraft:on_use_on` (JSON `on_use_on_trigger` › `on_use_on`; from 1.20.40).
    pub on_use_on: Option<EventTrigger>,
    /// `minecraft:render_offsets`, as written (JSON `render_offsets`; from 1.20.40).
    pub render_offsets: Option<Value>,
    /// `weapon.on_hurt_entity` (JSON `on_hurt_actor_trigger` › `on_hurt_actor`; from 1.20.40).
    pub on_hurt_actor: Option<EventTrigger>,
    /// `weapon.on_not_hurt_entity` (JSON `on_hit_actor_trigger` › `on_hit_actor`; from
    /// 1.20.40).
    pub on_hit_actor: Option<EventTrigger>,
    /// `weapon.on_hit_block` (JSON `on_hit_block_trigger` › `on_hit_block`; from 1.20.40).
    pub on_hit_block: Option<EventTrigger>,
    /// `chargeable.on_complete` (JSON `on_complete_trigger` › `on_complete`; from 1.20.50).
    pub on_complete: Option<EventTrigger>,
    /// A digger with triggers, copied by the 1.26.30 upgrade (JSON `legacy_on_dig_trigger`, a
    /// key of the crate's own).
    pub legacy_on_dig: Option<Digger>,
    /// The list form of `minecraft:custom_components`, moved by the 1.26.30 upgrade (JSON
    /// `custom_components_v1`, a key of the crate's own), as written.
    pub custom_components_v1: Option<Value>,
    /// Other members.
    pub extra: UnknownMembers,
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

    /// Reads the side-car from its JSON; a member of the wrong shape is kept in `extra` with
    /// an `InvalidType` warning at its path below `at`.
    pub(crate) fn from_value(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Self {
        let mut out = Self::default();
        let Some(object) = value.as_object() else {
            d.push(note(Code::from(&core_codes::INVALID_TYPE), at.clone()));
            return out;
        };
        'members: for (key, member) in object {
            let key = key.as_str();
            for (outer, inner, field) in SLOTS {
                if key == outer {
                    let trigger = member
                        .as_object()
                        .and_then(|o| o.get(inner))
                        .and_then(|t| EventTrigger::try_from(t.clone()).ok());
                    if let Some(trigger) = trigger {
                        *field(&mut out) = Some(trigger);
                        // A member beside the slot's trigger is read by nothing and has
                        // no place in the slot: told where it is.
                        for (other, _) in member
                            .as_object()
                            .into_iter()
                            .flat_map(|object| object.iter())
                            .filter(|(other, _)| other.as_str() != inner)
                        {
                            let path = at.child(key).child(other.as_str());
                            d.push(note(Code::from(&core_codes::MEMBER_NOT_KEPT), path));
                        }
                    } else {
                        let path = at.child(key);
                        d.push(note(Code::from(&core_codes::INVALID_TYPE), path));
                        out.extra.insert(key.to_owned(), member.clone());
                    }
                    continue 'members;
                }
            }
            match key {
                "render_offsets" => out.render_offsets = Some(member.clone()),
                "custom_components_v1" => out.custom_components_v1 = Some(member.clone()),
                "legacy_on_dig_trigger" => {
                    // (With the sign of a negative zero where the digger reads it so.)
                    let text = bedrock_addon_core::json::signed_zero::text_for::<Digger>(member)
                        .unwrap_or_default();
                    match bedrock_addon_core::json::from_str_jsonc::<Digger>(&text) {
                        Ok(digger) => {
                            // An unknown member is kept where it is (the document's table)
                            // and told there, as in a component.
                            let mut kept = bedrock_addon_core::document::Unknown::new();
                            let found = bedrock_addon_core::json::keep_unknown_members(
                                &digger, member, &mut kept,
                            );
                            d.unknown_mut().absorb(&at.child(key), kept);
                            for (path, _) in found {
                                let mut full = at.clone();
                                full.push(key);
                                full.append(&path);
                                d.push(note(Code::from(&core_codes::UNRECOGNIZED_FIELD), full));
                            }
                            out.legacy_on_dig = Some(digger);
                        }
                        Err(_) => {
                            out.extra.insert(key.to_owned(), member.clone());
                        }
                    }
                }
                _ => {
                    let path = at.child(key);
                    d.push(note(Code::from(&core_codes::UNRECOGNIZED_FIELD), path));
                    out.extra.insert(key.to_owned(), member.clone());
                }
            }
        }
        out
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

/// A side-car finding at `path`. The side-car is the hand-written `__deprecated_components`
/// of a 1.20.30–1.26.0 document. The game logs its presence as a deprecated member and
/// refuses the item, so what is told of the members inside is the crate's own reading of a
/// document the game does not load: an information, since the game says nothing about them.
fn note(code: Code, path: JsonPath) -> Diagnostic {
    Diagnostic::info(code).at(path)
}
