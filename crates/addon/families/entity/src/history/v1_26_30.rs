//! Format 1.26.30: the struct and the step to 1.26.40.

use crate::codes;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx, Versioned};
use facet_value::VObject;

use super::Payload;
use super::ops::{Members, Removed, Told, finding, kept, note, removed, sections};
use crate::component::name_projectile_aliases;

/// An entity document as written in format 1.26.30 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 26, 30>;

/// The step's version, for its findings.
const V1_26_40: FormatVersion = known::V1_26_40;

/// 1.26.30 → 1.26.40.
impl LegacyStep<1, 26, 30> for Payload {
    type Next = super::v1_26_40::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // The conversions of this step whose legacy reader is one of the game's schema-driven
        // readers.
        super::ops::schema_conversions(&mut payload, super::v1_26_40::Entity::VERSION, cx);
        let mut found: Vec<(Found, JsonPath)> = Vec::new();
        let mut told = Told::default();
        // The renames of a projectile's aliases, recorded before the step's findings: a
        // finding of this step below a renamed member names the member of the file.
        let mut aliases = Told::default();
        for section in sections(&mut payload) {
            let members = section.members;
            let at = &section.at;
            for goal in [
                "minecraft:behavior.melee_attack",
                "minecraft:behavior.melee_box_attack",
                "minecraft:behavior.delayed_attack",
                "minecraft:behavior.stomp_attack",
                "minecraft:behavior.use_kinetic_weapon",
            ] {
                // C, the rebind of the melee attack base: the old binding read `on_kill` into
                // `on_attack`, so a present `on_kill` replaces it.
                // The crate's own note: the `on_attack` it replaces is told as dropped. A `null`
                // `on_kill` is an absent one.
                if let Some(object) = members.object(goal)
                    && let Some(on_kill) = object.remove("on_kill")
                    && !on_kill.is_null()
                {
                    if object
                        .get("on_attack")
                        .is_some_and(|had| !had.is_null() && *had != on_kill)
                    {
                        found.push((Found::Replaced, at.with(&[goal, "on_attack"])));
                    }
                    object.insert("on_attack", on_kill);
                    told.moved(at.with(&[goal, "on_kill"]), at.with(&[goal, "on_attack"]));
                }
            }
            if let Some(pushable) = members.object("minecraft:pushable_by_entity") {
                pushable_overlap(pushable);
            }
            if let Some(projectile) = members.object("minecraft:projectile") {
                let projectile_at = at.child("minecraft:projectile");
                // The legacy reader's aliases are the members: what this step converts
                // under a member it converts under its alias.
                name_projectile_aliases(projectile, &mut |from, to| {
                    aliases.moved(projectile_at.join(&from), projectile_at.join(&to));
                });
                old_flag_alias(projectile, &projectile_at, &mut aliases);
                projectile_1_26_40(projectile, &projectile_at, &mut found);
            }
            let ranged_at = at.child("minecraft:behavior.ranged_attack");
            if let Some(goal) = members.object("minecraft:behavior.ranged_attack")
                && interval_range(goal, &ranged_at, &mut told)
            {
                found.push((
                    Found::Dropped("attack_interval"),
                    at.with(&["minecraft:behavior.ranged_attack", "attack_interval"]),
                ));
            }
            if let Some(goal) = members.object("minecraft:behavior.beg") {
                array_to_range(goal, "look_time");
            }
            if let Some(goal) = members.object("minecraft:behavior.float_wander") {
                array_to_range(goal, "float_duration");
            }
        }
        aliases.report(cx);
        for (what, at) in found {
            match what {
                Found::Dropped(member) => removed(cx, Removed::Field, &at, member, V1_26_40),
                Found::Misread => finding(cx, DiagnosticCode::from(&core_codes::INVALID_TYPE), &at),
                // A note (an information): the game binds both keys to the one trigger, the
                // later one winning.
                Found::Replaced => note(cx, DiagnosticCode::from(&core_codes::DROPPED_FIELD), &at),
                // `should_bounce` / `difficulty_randomization`: level 3 and the default taken
                // (the game logs this at the error level and keeps the document: a kept error).
                Found::Invalid(member, value) => kept(
                    cx,
                    codes::NOT_A_VALID_VALUE
                        .with([value.as_str(), member])
                        .into(),
                    &at,
                ),
            }
        }
        told.report(cx);
        Legacy(payload)
    }
}

/// A finding of the step.
enum Found {
    /// A member dropped with `DroppedField`.
    Dropped(&'static str),
    /// A member of another JSON type than the step reads, read as its default.
    Misread,
    /// A member replaced by what the step makes of another.
    Replaced,
    /// A value not in a member's set, replaced by its fallback.
    Invalid(&'static str, String),
}

/// The `pushable_by_entity` upgrade: every preset gets
/// `require_collision_overlap: false`; without presets the component gets
/// `use_default_with_overlap_check: false`.
fn pushable_overlap(pushable: &mut VObject) {
    let presets = pushable.get_mut("presets").and_then(|p| p.as_array_mut());
    match presets {
        Some(list) if !list.is_empty() => {
            for preset in list.as_mut_slice() {
                if let Some(preset) = preset.as_object_mut() {
                    preset.insert_absent("require_collision_overlap", false.into());
                }
            }
        }
        _ => {
            pushable.insert_absent("use_default_with_overlap_check", false.into());
        }
    }
}

/// The legacy reader's alias of the flag this step converts, `semirandomdiffdamage`, renamed
/// to `semi_random_diff_damage` where that is absent, the move recorded. The alias is the
/// step's alone: the newest struct has no such member, so the lenient read leaves it be.
// `projectile.on_hit.impact_damage`: "`catchFire`, `semirandomdiffdamage`,
// `destroyonhit`, …".
fn old_flag_alias(projectile: &mut VObject, at: &JsonPath, aliases: &mut Told) {
    const ALIAS: &str = "semirandomdiffdamage";
    const MEMBER: &str = "semi_random_diff_damage";
    let Some(damage) = projectile
        .get_mut("on_hit")
        .and_then(|h| h.as_object_mut())
        .and_then(|h| h.get_mut("impact_damage"))
        .and_then(|d| d.as_object_mut())
    else {
        return;
    };
    if !damage.contains_key(MEMBER)
        && let Some(flag) = damage.remove(ALIAS)
    {
        damage.insert(MEMBER, flag);
        let damage_at = at.with(&["on_hit", "impact_damage"]);
        aliases.moved(damage_at.child(ALIAS), damage_at.child(MEMBER));
    }
}

/// The two `projectile` migrations of 1.26.40 (B-equivalent on the dynamic component).
fn projectile_1_26_40(projectile: &mut VObject, at: &JsonPath, found: &mut Vec<(Found, JsonPath)>) {
    if let Some(bounce) = projectile.get_mut("should_bounce") {
        let value = if let Some(flag) = bounce.as_bool() {
            if flag { "if_invulnerable" } else { "no" }
        } else if let Some(text) = bounce.as_string() {
            match text.as_str() {
                "no" | "if_invulnerable" | "if_no_damage_dealt" => "",
                other => {
                    found.push((
                        Found::Invalid("should_bounce", other.to_owned()),
                        at.child("should_bounce"),
                    ));
                    "no"
                }
            }
        } else {
            ""
        };
        if !value.is_empty() {
            *bounce = value.into();
        }
    }
    let Some(damage) = projectile
        .get_mut("on_hit")
        .and_then(|h| h.as_object_mut())
        .and_then(|h| h.get_mut("impact_damage"))
        .and_then(|d| d.as_object_mut())
    else {
        return;
    };
    damage.insert_absent("power_multiplier", 2.0.into());
    if let Some(semi) = damage.remove("semi_random_diff_damage") {
        if damage.contains_key("difficulty_randomization") {
            found.push((
                Found::Dropped("semi_random_diff_damage"),
                at.with(&["on_hit", "impact_damage", "semi_random_diff_damage"]),
            ));
        } else {
            // The crate's own note: a value that is no bool is read as `false`, and told.
            if !semi.is_bool() {
                found.push((
                    Found::Misread,
                    at.with(&["on_hit", "impact_damage", "semi_random_diff_damage"]),
                ));
            }
            let additive = semi.as_bool().unwrap_or(false);
            damage.insert(
                "difficulty_randomization",
                if additive { "additive" } else { "none" },
            );
        }
    }
}

/// `ranged_attack`: `attack_interval_min` / `attack_interval_max` → `attack_interval {min,
/// max}` (a missing half 0). Returns whether a scalar `attack_interval` was dropped for the
/// pair.
///
/// The crate's own note, told at the half as the file has it (`at` is the goal's path): a
/// half that is no number is no half (`InvalidType`), so the other half is kept; one beyond
/// a float becomes the largest float (`InvalidValue`). A `null` half is an absent one.
fn interval_range(goal: &mut VObject, at: &JsonPath, told: &mut Told) -> bool {
    let min = super::ops::half(goal, "attack_interval_min", at, told);
    let max = super::ops::half(goal, "attack_interval_max", at, told);
    if min.is_none() && max.is_none() {
        return false;
    }
    let dropped = goal.get("attack_interval").is_some();
    let mut range = VObject::new();
    for (old, new, half) in [
        ("attack_interval_min", "min", &min),
        ("attack_interval_max", "max", &max),
    ] {
        if half.is_some() {
            told.moved(at.child(old), at.with(&["attack_interval", new]));
        }
    }
    range.insert("min", min.unwrap_or_else(|| 0.into()));
    range.insert("max", max.unwrap_or_else(|| 0.into()));
    goal.insert("attack_interval", range);
    dropped
}

/// An array `[a, b]` member becomes `{min: a, max: b}`.
fn array_to_range(goal: &mut VObject, key: &str) {
    let Some(member) = goal.get_mut(key) else {
        return;
    };
    let Some([a, b]) = member
        .as_array()
        .map(|a| a.iter().cloned().collect::<Vec<_>>())
        .and_then(|a| <[Value; 2]>::try_from(a).ok())
    else {
        return;
    };
    let mut range = VObject::new();
    range.insert("min", a);
    range.insert("max", b);
    *member = range.into();
}
