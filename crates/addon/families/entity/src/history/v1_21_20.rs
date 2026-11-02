//! Format 1.21.20: the struct and the step to 1.21.40.

super::legacy_struct!(1, 21, 20);

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};
use facet_value::VObject;

use super::ops::{
    Members, Removed, Told, below, components, is, json, removed, rename, rename_member, section_a,
    section_b, sections,
};
use bedrock_addon_core::diagnostic::JsonPath;

/// The step's version, for its findings.
const V1_21_40: FormatVersion = FormatVersion::new(1, 21, 40);

/// 1.21.20 → 1.21.40 (gate 1.21.40).
impl Upgrade for Entity {
    type Next = super::v1_21_40::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        if section_a(&payload, cx) {
            use ActorType as T;
            let bound = [
                T::EnderDragon,
                T::FishingHook,
                T::ThrownTrident,
                T::Arrow,
                T::DragonFireball,
                T::Fireball,
                T::WitherSkull,
                T::WitherSkullDangerous,
                T::SmallFireball,
            ];
            if is(cx, &bound) {
                components(&mut payload).insert_absent("minecraft:dimension_bound", json("{}"));
            }
            if cx.actor_type == T::FishingHook {
                components(&mut payload).insert_absent("minecraft:transient", json("{}"));
            }
        }
        if section_b(&payload, cx) {
            let mut lost = Vec::new();
            let mut told = Told::default();
            for section in sections(&mut payload) {
                let members = section.members;
                if let Some(home) = members.object("minecraft:home") {
                    let radius = home
                        .get("restriction_radius")
                        .and_then(Value::as_number)
                        .map_or(0.0, facet_value::VNumber::to_f64_lossy);
                    if !home.contains_key("restriction_type") && radius > 0.0 {
                        home.insert("restriction_type", "random_movement");
                    }
                }
                let lookat_at = below(&section.at, &["minecraft:lookat"]);
                if let Some(lookat) = members.object("minecraft:lookat")
                    && looked_at(lookat, &lookat_at, &mut told)
                {
                    // At the member as the file has it: under `lookat`.
                    lost.push(below(
                        &section.at,
                        &["minecraft:lookat", "allow_invulnerable"],
                    ));
                }
                rename(
                    members,
                    &section.at,
                    "minecraft:lookat",
                    "minecraft:looked_at",
                    &mut told,
                );
                if let Some(sensor) = members.object("minecraft:damage_sensor")
                    && let Some(triggers) = sensor.get_mut("triggers")
                {
                    deals_damage_as_string(triggers);
                }
            }
            for at in lost {
                removed(cx, Removed::Field, &at, "allow_invulnerable", V1_21_40);
            }
            told.report(cx);
        }
        super::v1_21_40::Entity(payload)
    }
}

/// `lookat` → `looked_at` members; returns whether `allow_invulnerable` was removed.
fn looked_at(lookat: &mut VObject, at: &JsonPath, told: &mut Told) -> bool {
    if let Some(target) = lookat.get_mut("set_target")
        && let Some(set) = target.as_bool()
    {
        *target = if set {
            "once_and_stop_scanning"
        } else {
            "never"
        }
        .into();
    }
    for (old, new) in [
        ("look_event", "looked_at_event"),
        ("look_cooldown", "looked_at_cooldown"),
    ] {
        // A new-named member the file has beside the old one is written over.
        rename_member(lookat, old, new, at, told);
    }
    lookat.remove("allow_invulnerable").is_some()
}

/// A boolean `deals_damage` in `triggers` (an object or an array) becomes `"yes"` / `"no"`.
fn deals_damage_as_string(triggers: &mut Value) {
    let fix = |trigger: &mut VObject| {
        if let Some(deals) = trigger.get_mut("deals_damage")
            && let Some(flag) = deals.as_bool()
        {
            *deals = if flag { "yes" } else { "no" }.into();
        }
    };
    if let Some(list) = triggers.as_array_mut() {
        for trigger in list.as_mut_slice() {
            if let Some(object) = trigger.as_object_mut() {
                fix(object);
            }
        }
    } else if let Some(object) = triggers.as_object_mut() {
        fix(object);
    }
}
