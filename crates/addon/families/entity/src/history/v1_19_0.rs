//! Format 1.19.0: the struct and the step to 1.19.50.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::ops::empty_object;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};
use facet_value::VArray;

use super::Payload;
use super::fragments;
use super::ops::{Members, Told, components, group, section_a, section_b, sections};
use bedrock_addon_core::diagnostic::JsonPath;

/// An entity document as written in format 1.19.0 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 19, 0>;

/// 1.19.0 → 1.19.50 (gate 1.19.50).
impl LegacyStep<1, 19, 0> for Payload {
    type Next = super::v1_19_50::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        if section_a(&payload, cx) {
            slime_area_attack(&mut payload, cx.actor_type);
            if cx.actor_type == ActorType::Enderman {
                components(&mut payload)
                    .insert_absent("minecraft:variable_max_auto_step", fragments::auto_step());
            }
            let horses = [
                ActorType::Horse,
                ActorType::Donkey,
                ActorType::Mule,
                ActorType::SkeletonHorse,
                ActorType::Camel,
            ];
            if !horses.contains(&cx.actor_type) {
                input_ground_controlled(&mut payload, cx);
            }
        }
        if section_b(&payload, cx) {
            let mut told = Told::default();
            for section in sections(&mut payload) {
                for goal in [
                    "minecraft:behavior.take_flower",
                    "minecraft:behavior.offer_flower",
                ] {
                    if let Some(object) = section.members.object(goal) {
                        append_all_of(
                            object,
                            vec![fragments::is_daytime()],
                            &section.at.child(goal),
                            &mut told,
                        );
                    }
                }
            }
            told.report(cx);
        }
        Legacy(payload)
    }
}

/// Appends `entries` to `filters.all_of`, creating `filters` and `all_of` as needed. A `filters`
/// that is no object gets nothing, and an `all_of` that is no array is replaced by the list of the
/// entries; both are told at the member (`at` is the goal's path). A `null` member is an absent
/// one.
pub(super) fn append_all_of(
    object: &mut facet_value::VObject,
    entries: Vec<Value>,
    at: &JsonPath,
    told: &mut Told,
) {
    object.insert_absent("filters", empty_object());
    let Some(filters) = object.get_mut("filters").and_then(|f| f.as_object_mut()) else {
        told.wrong_type(at.child("filters"));
        return;
    };
    if !filters.get("all_of").is_some_and(Value::is_array) {
        if filters.remove("all_of").is_some_and(|old| !old.is_null()) {
            told.wrong_type(at.with(&["filters", "all_of"]));
        }
        filters.insert("all_of", VArray::new());
    }
    if let Some(all_of) = filters.get_mut("all_of").and_then(|a| a.as_array_mut()) {
        for entry in entries {
            all_of.push(entry);
        }
    }
}

/// A, vanilla slime and magma cube: each existing size
/// group without `area_attack` gets one.
fn slime_area_attack(payload: &mut Payload, actor: ActorType) {
    let (sizes, families): (&[(&str, i32)], &[&str]) = match actor {
        ActorType::MagmaCube => (
            &[
                ("minecraft:slime_large", 6),
                ("minecraft:slime_medium", 4),
                ("minecraft:slime_small", 3),
            ],
            &["player", "irongolem"],
        ),
        ActorType::Slime => (
            &[("minecraft:slime_large", 4), ("minecraft:slime_medium", 4)],
            &["player", "irongolem", "snowgolem"],
        ),
        _ => return,
    };
    for (name, damage) in sizes {
        let Some(group) = group(payload, name) else {
            continue;
        };
        group.insert_absent(
            "minecraft:area_attack",
            fragments::slime_area_attack(*damage, families),
        );
    }
}

/// `input_ground_controlled` in a section: `variable_max_auto_step` when absent, then its
/// `controlled_value` takes the old `base_value` when absent and `base_value` becomes 0.5625.
fn input_ground_controlled(payload: &mut Payload, cx: &mut UpgradeCx<'_>) {
    // The crate's own information, since the fixer does it: a `base_value` that is replaced while
    // the document has its own `controlled_value` is told.
    let mut told = Told::default();
    for section in sections(payload) {
        let members = section.members;
        if !members.has("minecraft:input_ground_controlled") {
            continue;
        }
        members.insert_absent("minecraft:variable_max_auto_step", fragments::auto_step());
        if let Some(step) = members.object("minecraft:variable_max_auto_step") {
            if let Some(base) = step.get("base_value").cloned() {
                let kept = step.insert_absent("controlled_value", base.clone());
                let same = base.as_number().map(facet_value::VNumber::to_f64_lossy) == Some(0.5625);
                if !kept && !base.is_null() && !same {
                    told.dropped(
                        section
                            .at
                            .with(&["minecraft:variable_max_auto_step", "base_value"]),
                    );
                }
            }
            step.insert("base_value", 0.5625);
        }
    }
    told.report(cx);
}
