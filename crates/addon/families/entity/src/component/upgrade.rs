//! The component side of the entity steps: the Section B operations on this module's
//! components while they are still JSON (`Dynamic`) in the source struct, one function per
//! step. Each takes the component's value (in `components` or a group) and changes it in
//! place; findings are relative to the component. The version chain calls them; the typed
//! conversions (`From` impls in [`older`](super::older), the `ExperienceReward` fold) are on the
//! types.

use crate::codes;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::write_payload;
use facet_value::{VArray, VObject};

use super::movement::JumpDynamic;

/// An information of the crate's own at `at`, relative to the component: a member a step of this
/// library drops or reads as a default where the game says nothing (the game writes no content-log
/// line for this).
fn warning(code: Code, at: &JsonPath) -> Diagnostic {
    Diagnostic::info(code).at(at.clone())
}

/// The interactions of an `interact` value: each element of `interactions` (or the single
/// interaction object it holds).
fn interactions(value: &mut Value) -> Vec<&mut VObject> {
    let Some(list) = value
        .as_object_mut()
        .and_then(|o| o.get_mut("interactions"))
    else {
        return Vec::new();
    };
    if list.is_object() {
        return list.as_object_mut().into_iter().collect();
    }
    list.as_array_mut()
        .map(|array| array.iter_mut().filter_map(Value::as_object_mut).collect())
        .unwrap_or_default()
}

/// 1.13.0, `interact` (documents below 1.13.0): in the object, or each element of an array,
/// `event`, `target` and `filters` move into `on_interact` when that is absent (when it is
/// present they are dropped, `DroppedField`); then the whole value becomes
/// `{"interactions": <old>}`.
pub fn interact_1_13_0(value: &mut Value, d: &mut Diagnostics) {
    let mut move_trigger = |object: &mut VObject, at: &JsonPath| {
        let had_on_interact = object.contains_key("on_interact");
        let mut trigger = VObject::new();
        for key in ["event", "target", "filters"] {
            if let Some(member) = object.remove(key) {
                if had_on_interact {
                    let here = at.child(key);
                    d.push(warning(Code::from(&core_codes::DROPPED_FIELD), &here));
                } else {
                    trigger.insert(key, member);
                }
            }
        }
        if !had_on_interact && !trigger.is_empty() {
            object.insert("on_interact", Value::from(trigger));
        }
    };
    if let Some(object) = value.as_object_mut() {
        move_trigger(object, &JsonPath::new());
    } else if let Some(array) = value.as_array_mut() {
        for (i, element) in array.iter_mut().enumerate() {
            if let Some(object) = element.as_object_mut() {
                let at = JsonPath::new().index(i);
                move_trigger(object, &at);
            }
        }
    }
    let mut wrapped = VObject::new();
    wrapped.insert("interactions", std::mem::replace(value, Value::NULL));
    *value = Value::from(wrapped);
}

/// 1.20.80, `interact`: an integer `equip_item_slot` or `drop_item_slot` of an interaction
/// becomes its decimal string (`2` → `"2"`).
pub fn interact_slots_1_20_80(value: &mut Value) {
    for interaction in interactions(value) {
        for key in ["equip_item_slot", "drop_item_slot"] {
            let number = interaction
                .get(key)
                .and_then(Value::as_number)
                .and_then(facet_value::VNumber::to_i64);
            if let Some(number) = number {
                interaction.insert(key, Value::from(number.to_string().as_str()));
            }
        }
    }
}

/// 1.21.40, `home`: without `restriction_type` and with `restriction_radius > 0`,
/// `restriction_type` becomes `random_movement`.
pub fn home_1_21_40(value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    let radius = object
        .get("restriction_radius")
        .and_then(Value::as_number)
        .and_then(facet_value::VNumber::to_f64)
        .unwrap_or(0.0);
    if !object.contains_key("restriction_type") && radius > 0.0 {
        object.insert("restriction_type", Value::from("random_movement"));
    }
}

/// 1.21.120, `projectile`: an `on_hit.impact_damage` object gets
/// `apply_knockback_to_blocking_targets: true` (the field did not exist before).
pub fn projectile_1_21_120(value: &mut Value) {
    if let Some(damage) = impact_damage(value) {
        damage.insert("apply_knockback_to_blocking_targets", Value::from(true));
    }
}

/// 1.26.0, `interact`: every interaction gets `swing: false` when absent (the new default is
/// `true`).
pub fn interact_swing_1_26_0(value: &mut Value) {
    for interaction in interactions(value) {
        if !interaction.contains_key("swing") {
            interaction.insert("swing", Value::from(false));
        }
    }
}

/// 1.26.0, `jump.dynamic`: the skip data of [`JumpDynamic::upgraded_1_26_0`] is written over
/// whatever the author wrote; other members stay.
pub fn jump_dynamic_1_26_0(value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    let upgraded = JumpDynamic::upgraded_1_26_0();
    for (key, data) in [
        ("regular_skip_data", &upgraded.regular_skip_data),
        ("fast_skip_data", &upgraded.fast_skip_data),
    ] {
        // The constants are finite, so they always have a JSON form.
        if let Ok(skip) = write_payload(data) {
            object.insert(key, skip);
        }
    }
}

/// 1.26.40, `projectile`: a boolean `should_bounce` becomes `"no"` (`false`) or
/// `"if_invulnerable"` (`true`); a string outside the three values is `InvalidValue` with
/// vanilla's text and becomes `"no"`. In `on_hit.impact_damage`, `power_multiplier: 2.0` (the
/// old default) is written when absent, and `semi_random_diff_damage` becomes
/// `difficulty_randomization` (`true` → `additive`, `false` → `none`; when both are present the
/// old member is dropped with `DroppedField`). `impact_damage.should_bounce` is left alone.
pub fn projectile_1_26_40(value: &mut Value, d: &mut Diagnostics) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    let bounce = object.get("should_bounce").cloned();
    if let Some(flag) = bounce.as_ref().and_then(Value::as_bool) {
        let name = if flag { "if_invulnerable" } else { "no" };
        object.insert("should_bounce", Value::from(name));
    } else if let Some(text) = bounce.as_ref().and_then(Value::as_string)
        && !["no", "if_invulnerable", "if_no_damage_dealt"].contains(&text.as_str())
    {
        // The game logs this at the error level and keeps the document with the default taken: a
        // kept error.
        d.push_kept_error(
            Diagnostic::error(codes::NOT_A_VALID_VALUE.with([text.as_str(), "should_bounce"]))
                .at(JsonPath::new().child("should_bounce")),
        );
        object.insert("should_bounce", Value::from("no"));
    }
    let Some(damage) = impact_damage(value) else {
        return;
    };
    if let Some(old) = damage.get("semi_random_diff_damage").cloned() {
        if damage.contains_key("difficulty_randomization") {
            damage.remove("semi_random_diff_damage");
            d.push(warning(
                Code::from(&core_codes::DROPPED_FIELD),
                &JsonPath::new().with(&["on_hit", "impact_damage", "semi_random_diff_damage"]),
            ));
        } else {
            if old.as_bool().is_none() {
                // The crate's own: not a bool, so `false` (`none`) is used, with a finding.
                d.push(warning(
                    Code::from(&core_codes::INVALID_VALUE),
                    &JsonPath::new().with(&["on_hit", "impact_damage", "semi_random_diff_damage"]),
                ));
            }
            let name = if old.as_bool() == Some(true) {
                "additive"
            } else {
                "none"
            };
            replace_key(
                damage,
                "semi_random_diff_damage",
                "difficulty_randomization",
                &Value::from(name),
            );
        }
    }
    if !damage.contains_key("power_multiplier") {
        damage.insert("power_multiplier", Value::from(2.0_f64));
    }
}

/// Puts `new_key: value` where `old_key` was, keeping the order of the other members.
fn replace_key(object: &mut VObject, old_key: &str, new_key: &str, value: &Value) {
    let members: Vec<(String, Value)> = object
        .iter()
        .map(|(k, v)| (k.as_str().to_owned(), v.clone()))
        .collect();
    let mut rebuilt = VObject::new();
    for (key, member) in members {
        if key == old_key {
            rebuilt.insert(new_key, value.clone());
        } else {
            rebuilt.insert(key.as_str(), member);
        }
    }
    *object = rebuilt;
}

/// The `on_hit.impact_damage` object of a `projectile` value, if there is one.
fn impact_damage(value: &mut Value) -> Option<&mut VObject> {
    value
        .as_object_mut()?
        .get_mut("on_hit")?
        .as_object_mut()?
        .get_mut("impact_damage")?
        .as_object_mut()
}

/// Wraps a value that is not an array into a one-element array (the legacy single form).
pub fn one_or_many(value: &mut Value) -> bool {
    if value.is_array() {
        return false;
    }
    let mut array = VArray::new();
    array.push(std::mem::replace(value, Value::NULL));
    *value = Value::from(array);
    true
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods, clippy::disallowed_types)]

    use super::*;
    use bedrock_addon_core::diagnostic::Severity;

    /// The cases of fixture `<name>.before.json` / `<name>.after.json`: `{case: component}`.
    fn cases(name: &str) -> Vec<(String, Value, Value)> {
        let dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../addon/tests/fixtures/entity/entity/component/"
        );
        let read = |suffix: &str| {
            let text = std::fs::read_to_string(format!("{dir}{name}.{suffix}.json"))
                .unwrap_or_else(|e| panic!("{name}.{suffix}: {e}"));
            bedrock_addon_core::json::from_str::<Value>(&text).expect("valid JSON")
        };
        let (before, after) = (read("before"), read("after"));
        let before = before.as_object().expect("an object");
        let after = after.as_object().expect("an object");
        before
            .iter()
            .map(|(case, value)| {
                let expected = after
                    .get(case.as_str())
                    .unwrap_or_else(|| panic!("{name}: no expected `{}`", case.as_str()));
                (case.as_str().to_owned(), value.clone(), expected.clone())
            })
            .collect()
    }

    /// Runs `op` on every case of fixture `name` and compares with the expected value; returns
    /// the findings per case.
    fn run(name: &str, op: impl Fn(&mut Value, &mut Diagnostics)) -> Vec<(String, Diagnostics)> {
        let mut out = Vec::new();
        for (case, mut value, expected) in cases(name) {
            let mut d = Diagnostics::default();
            op(&mut value, &mut d);
            assert_eq!(
                bedrock_addon_core::json::to_json(&value).ok(),
                bedrock_addon_core::json::to_json(&expected).ok(),
                "{name} / {case}"
            );
            out.push((case, d));
        }
        out
    }

    /// The findings of `case`.
    fn of<'a>(found: &'a [(String, Diagnostics)], case: &str) -> &'a Diagnostics {
        &found.iter().find(|(c, _)| c == case).expect(case).1
    }

    #[test]
    fn e00d_interact_moves_the_trigger_and_wraps_the_value() {
        // e00d: "`interact` with and without `on_interact`".
        let found = run("e00d_interact", interact_1_13_0);
        assert!(of(&found, "without_on_interact").is_empty());
        assert!(of(&found, "array").is_empty());
        let dropped = of(&found, "with_on_interact");
        assert_eq!(dropped.len(), 2, "event and target dropped: {dropped:?}");
        assert!(dropped.iter().all(|f| f.is(&core_codes::DROPPED_FIELD)));
    }

    #[test]
    fn e15_interact_integer_slots_become_strings() {
        // e15: `2 → "2"`.
        run("e15_interact_slots", |v, _| interact_slots_1_20_80(v));
    }

    #[test]
    fn e19_home_radius_without_a_type_gets_random_movement() {
        // e19: `restriction_type`.
        run("e19_home_restriction_type", |v, _| home_1_21_40(v));
    }

    #[test]
    fn e27_projectile_impact_damage_gets_blocking_knockback() {
        // e27: `apply_knockback_to_blocking_targets`.
        run("e27_projectile_blocking_knockback", |v, _| {
            projectile_1_21_120(v);
        });
    }

    #[test]
    fn e29b_interact_swing_and_jump_skip_data() {
        // e29b: `swing: false`, skip data overwrites author-written values.
        run("e29b_interact_swing", |v, _| interact_swing_1_26_0(v));
        run("e29b_jump_dynamic", |v, _| jump_dynamic_1_26_0(v));
    }

    #[test]
    fn e33_projectile_should_bounce_and_impact_damage() {
        // e33: `should_bounce` `true` → `"if_invulnerable"`, `false` → `"no"`,
        // `"bogus"` → warning + `"no"`; `power_multiplier` 2.0 inserted (explicit value kept),
        // `semi_random_diff_damage` `true` / `false` → `additive` / `none`;
        // `impact_damage.should_bounce` left untouched.
        let found = run("e33_projectile", projectile_1_26_40);
        let bogus = of(&found, "bogus");
        assert_eq!(bogus.len(), 1);
        assert_eq!(
            bogus.iter().next().map(|f| f.code.to_string()).as_deref(),
            Some("'bogus' is not a valid 'should_bounce' value.")
        );
        assert!(of(&found, "changelog_example").is_empty());
        let non_bool = of(&found, "non_bool_semi_random");
        assert_eq!(non_bool.len(), 1, "{non_bool:?}");
        assert!(
            non_bool
                .iter()
                .all(|f| f.is(&core_codes::INVALID_VALUE) && f.severity == Severity::Info)
        );
        let both = of(&found, "both_members");
        assert!(both.iter().any(|f| f.is(&core_codes::DROPPED_FIELD)));
    }

    #[test]
    fn one_or_many_wraps_a_single_value() {
        let mut value = Value::from("eat");
        assert!(one_or_many(&mut value));
        assert!(value.is_array());
        assert!(!one_or_many(&mut value));
    }
}
