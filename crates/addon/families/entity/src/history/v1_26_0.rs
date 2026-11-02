//! Format 1.26.0: the struct and the step to 1.26.10.

super::legacy_struct!(1, 26, 0);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};
use facet_value::VObject;

use super::ops::{Told, below, finding, json, overwrite, same, section_b, sections};
use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath};

/// 1.26.0 → 1.26.10.
impl Upgrade for Entity {
    type Next = super::v1_26_10::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
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
                        misread.push(below(&section.at, &["minecraft:buoyant", "simulate_waves"]));
                    }
                    let waves = waves.as_bool().unwrap_or(false);
                    // A `movement_type` of the file is written over, as the game's step
                    // writes it.
                    let at = below(&section.at, &["minecraft:buoyant"]);
                    let value = Value::from(if waves { "waves" } else { "none" });
                    overwrite(buoyant, "movement_type", value, &at, &mut told);
                    told.moved(
                        below(&at, &["simulate_waves"]),
                        below(&at, &["movement_type"]),
                    );
                }
                // The `pushable` split: each member (default `true`) that holds gives its
                // component.
                if let Some(pushable) = members.take("minecraft:pushable") {
                    if !pushable.is_object() {
                        misread.push(below(&section.at, &["minecraft:pushable"]));
                    }
                    for key in pushable.as_object().into_iter().flat_map(VObject::keys) {
                        let key = key.as_str();
                        if !["is_pushable", "is_pushable_by_piston"].contains(&key) {
                            told.unknown(below(&section.at, &["minecraft:pushable", key]));
                        }
                    }
                    let mut holds = |key: &str| {
                        let member = pushable.as_object().and_then(|p| p.get(key));
                        if member.is_some_and(|member| !member.is_bool()) {
                            misread.push(below(&section.at, &["minecraft:pushable", key]));
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
                        let empty = json("{}");
                        if members
                            .get(component)
                            .is_some_and(|had| !had.is_null() && !same(had, &empty))
                        {
                            told.dropped(below(&section.at, &[component]));
                        }
                        members.set(component, empty);
                    }
                }
            }
            for at in misread {
                finding(cx, DiagnosticCode::InvalidType { text: None }, &at);
            }
            told.report(cx);
        }
        super::v1_26_10::Entity(payload)
    }
}
