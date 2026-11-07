//! Format 1.26.0: the struct and the step to 1.26.10.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::ops::empty_object;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx, Versioned};
use facet_value::VObject;

use super::Payload;
use super::ops::{Told, finding, overwrite, same, section_b, sections};
use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath};

/// An entity document as written in format 1.26.0 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 26, 0>;

/// 1.26.0 → 1.26.10.
impl LegacyStep<1, 26, 0> for Payload {
    type Next = super::v1_26_10::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // The conversions of this step whose legacy reader is one of the game's schema-driven
        // readers.
        super::ops::schema_conversions(&mut payload, super::v1_26_10::Entity::VERSION, cx);
        if section_b(&payload, cx) {
            // The crate's own note (it names `true` and `false` alone): a member the
            // step reads as its default because it is no bool, and a `pushable` that is no
            // object, are read as before and told at the member.
            let mut misread: Vec<JsonPath> = Vec::new();
            // The crate's own note: a member of `pushable` the split does not read goes with the
            // component, and is told.
            let mut told = Told::default();
            for section in sections(&mut payload) {
                let members = section.members;
                // `buoyant.simulate_waves` → `movement_type`.
                if let Some(buoyant) = members.object("minecraft:buoyant")
                    && let Some(waves) = buoyant.remove("simulate_waves")
                {
                    if !waves.is_bool() {
                        misread.push(section.at.with(&["minecraft:buoyant", "simulate_waves"]));
                    }
                    let waves = waves.as_bool().unwrap_or(false);
                    // A `movement_type` of the file is written over, as the game's step
                    // writes it.
                    let at = section.at.child("minecraft:buoyant");
                    let value = Value::from(if waves { "waves" } else { "none" });
                    overwrite(buoyant, "movement_type", value, &at, &mut told);
                    told.moved(at.child("simulate_waves"), at.child("movement_type"));
                }
                // The `pushable` split: each member (default `true`) that holds gives its
                // component.
                if let Some(pushable) = members.take("minecraft:pushable") {
                    if !pushable.is_object() {
                        misread.push(section.at.child("minecraft:pushable"));
                    }
                    for key in pushable.as_object().into_iter().flat_map(VObject::keys) {
                        let key = key.as_str();
                        if !["is_pushable", "is_pushable_by_piston"].contains(&key) {
                            told.unknown(section.at.with(&["minecraft:pushable", key]));
                        }
                    }
                    let mut holds = |key: &str| {
                        let member = pushable.as_object().and_then(|p| p.get(key));
                        if member.is_some_and(|member| !member.is_bool()) {
                            misread.push(section.at.with(&["minecraft:pushable", key]));
                        }
                        member.and_then(Value::as_bool).unwrap_or(true)
                    };
                    // A component of the file the split writes over is noted where it holds
                    // something: the step writes the empty component.
                    for (member, component) in [
                        ("is_pushable", "minecraft:pushable_by_entity"),
                        ("is_pushable_by_piston", "minecraft:pushable_by_block"),
                    ] {
                        if !holds(member) {
                            continue;
                        }
                        let empty = empty_object();
                        if members
                            .get(component)
                            .is_some_and(|had| !had.is_null() && !same(had, &empty))
                        {
                            told.dropped(section.at.child(component));
                        }
                        members.set(component, empty);
                    }
                }
            }
            for at in misread {
                finding(cx, DiagnosticCode::from(&core_codes::INVALID_TYPE), &at);
            }
            told.report(cx);
        }
        Legacy(payload)
    }
}
