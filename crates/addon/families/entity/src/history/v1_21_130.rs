//! Format 1.21.130: the struct and the step to 1.26.0.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::ops::empty_object;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx, Versioned};
use std::collections::HashMap;

use facet_value::VObject;

use super::Payload;
use super::fragments;
use super::ops::{Members, Removed, Told, half, overwrite, removed, section_b, sections};
use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath};

/// An entity document as written in format 1.21.130 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 21, 130>;

/// The step's version, for its findings.
const V1_26_0: FormatVersion = known::V1_26_0;

/// 1.21.130 → 1.26.0.
impl LegacyStep<1, 21, 130> for Payload {
    type Next = super::v1_26_0::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // The conversions of this step whose legacy reader is one of the game's schema-driven
        // readers.
        super::ops::schema_conversions(&mut payload, super::v1_26_0::Entity::VERSION, cx);
        // The crate's own note: what the step reads as another value, drops or replaces is told at
        // the member.
        let mut told = Told::default();
        if section_b(&payload, cx) {
            for section in sections(&mut payload) {
                let at = section.at;
                for goal in section_b_1260(section.members, &at, &mut told) {
                    let member = "target_distance";
                    removed(
                        cx,
                        Removed::Field,
                        &at.with(&[goal, member]),
                        member,
                        V1_26_0,
                    );
                }
            }
        }
        // C, typed → typed: `experience_reward` and `behavior.aquatic_charge_attack`.
        for section in sections(&mut payload) {
            if let Some(reward) = section.members.object("minecraft:experience_reward") {
                fold_reward(reward, "on_bred");
                fold_reward(reward, "on_death");
            }
            if let Some(goal) = section
                .members
                .object("minecraft:behavior.aquatic_charge_attack")
                && let Some(speed) = goal.remove("charge_speed_multiplier")
            {
                const GOAL: &str = "minecraft:behavior.aquatic_charge_attack";
                let lost = !speed.is_null();
                let from = section.at.with(&[GOAL, "charge_speed_multiplier"]);
                if goal.insert_absent("charge_speed", speed) {
                    told.moved(from, section.at.with(&[GOAL, "charge_speed"]));
                } else if lost {
                    told.dropped(from);
                }
            }
        }
        told.report(cx);
        Legacy(payload)
    }
}

/// Section B of the step in one section; the `look_at_*` goals whose `target_distance` was
/// removed.
fn section_b_1260(members: &mut dyn Members, at: &JsonPath, told: &mut Told) -> Vec<&'static str> {
    let mut dropped = Vec::new();
    // `interact`: every interaction gets `swing: false` when absent (the 1.26.0 default is
    // `true`).
    if let Some(interactions) = members
        .object("minecraft:interact")
        .and_then(|i| i.get_mut("interactions"))
    {
        let swing = |object: &mut VObject| {
            object.insert_absent("swing", false.into());
        };
        if let Some(list) = interactions.as_array_mut() {
            for item in list.as_mut_slice() {
                if let Some(object) = item.as_object_mut() {
                    swing(object);
                }
            }
        } else if let Some(object) = interactions.as_object_mut() {
            swing(object);
        }
    }
    for goal in [
        "minecraft:behavior.look_at_player",
        "minecraft:behavior.look_at_target",
        "minecraft:behavior.look_at_entity",
        "minecraft:behavior.look_at_trading_player",
    ] {
        let goal_at = at.child(goal);
        if let Some(object) = members.object(goal)
            && look_time(object, &goal_at, told)
        {
            dropped.push(goal);
        }
    }
    breedable_split(members, at, told);
    if let Some(jump) = members.object("minecraft:jump.dynamic") {
        // Written whatever the document has; a member that held another value is told.
        let at = at.child("minecraft:jump.dynamic");
        overwrite(
            jump,
            "regular_skip_data",
            fragments::regular_skip_data(),
            &at,
            told,
        );
        overwrite(
            jump,
            "fast_skip_data",
            fragments::fast_skip_data(),
            &at,
            told,
        );
    }
    dropped
}

/// `look_at_*`: `target_distance` removed; `min_look_time` / `max_look_time` → `look_time
/// {min, max}` (2 and 4 when only one is present). Whether `target_distance` was removed.
fn look_time(goal: &mut VObject, at: &JsonPath, told: &mut Told) -> bool {
    let dropped = goal.remove("target_distance").is_some();
    let min = half(goal, "min_look_time", at, told);
    let max = half(goal, "max_look_time", at, told);
    if min.is_none() && max.is_none() {
        return dropped;
    }
    let moved = [
        ("min_look_time", "min", min.is_some()),
        ("max_look_time", "max", max.is_some()),
    ];
    let mut range = VObject::new();
    range.insert("min", min.unwrap_or_else(|| 2.into()));
    range.insert("max", max.unwrap_or_else(|| 4.into()));
    // A `look_time` the file has beside the old pair is written over, and noted.
    overwrite(goal, "look_time", range.into(), at, told);
    // What is told of the range as a whole is told at the first member of the pair the
    // file has; of a half, at its own member.
    if let Some((old, _, _)) = moved.iter().find(|(_, _, was)| *was) {
        told.moved(at.child(old), at.child("look_time"));
    }
    for (old, half, was) in moved {
        if was {
            told.moved(at.child(old), at.with(&["look_time", half]));
        }
    }
    dropped
}

/// The `breedable` upgrade: the offspring members move into a new `offspring`;
/// `breeds_with` entries are split into the mate's event and the offspring pair.
///
/// The crate's own note, told at the member (`at` is the section's path): a `breeds_with` or an
/// entry that is no object, a `mate_type` that is no string (`InvalidType`), an entry without
/// a `mate_type` that holds something (`MissingRequiredField`) — each gives no mate and no
/// offspring — and a member of an entry the split does not read (`UnrecognizedField`). A
/// `null` `breeds_with` is an absent one.
#[allow(
    clippy::too_many_lines,
    reason = "the split of one component, in vanilla's order"
)]
fn breedable_split(members: &mut dyn Members, at: &JsonPath, told: &mut Told) {
    const READ: [&str; 3] = ["mate_type", "baby_type", "breed_event"];
    const MOVED: [&str; 10] = [
        "blend_attributes",
        "inherit_tamed",
        "mutation_factor",
        "mutation_strategy",
        "random_variant_mutation_interval",
        "random_extra_variant_mutation_interval",
        "deny_parents_variant",
        "combine_parent_colors",
        "property_inheritance",
        "parent_centric_attribute_blending",
    ];
    let Some(breedable) = members.object("minecraft:breedable") else {
        return;
    };
    let breedable_at = at.child("minecraft:breedable");
    let offspring_at = at.child("minecraft:offspring");
    let mut moves = Vec::new();
    let mut offspring = VObject::new();
    for key in MOVED {
        if let Some(value) = breedable.remove(key) {
            offspring.insert(key, value);
            moves.push((breedable_at.child(key), offspring_at.child(key)));
        }
    }
    if breedable.get("breeds_with").is_some_and(Value::is_null) {
        breedable.remove("breeds_with");
    }
    if let Some(with) = breedable.get_mut("breeds_with") {
        let with_at = at.with(&["minecraft:breedable", "breeds_with"]);
        let listed = with.is_array();
        let entries: Vec<Value> = match with.as_array() {
            Some(list) => list.iter().cloned().collect(),
            None => vec![with.clone()],
        };
        let mut events = VObject::new();
        let mut pairs = VObject::new();
        // The entry that gave a mate its event and its baby: a later entry of the same mate
        // takes its place, and what the earlier one held is noted there.
        let mut gave_event: HashMap<String, JsonPath> = HashMap::new();
        let mut gave_baby: HashMap<String, JsonPath> = HashMap::new();
        for (index, entry) in entries.iter().enumerate() {
            let entry_at = if listed {
                with_at.index(index)
            } else {
                with_at.clone()
            };
            let Some(entry) = entry.as_object() else {
                told.wrong_type(entry_at);
                continue;
            };
            let Some(mate) = entry.get("mate_type").and_then(Value::as_string) else {
                match entry.get("mate_type") {
                    Some(mate) if !mate.is_null() => {
                        told.wrong_type(entry_at.child("mate_type"));
                        // The entry goes with what else it holds: told at the entry
                        // as the file has it.
                        if entry.len() > 1 {
                            told.dropped(entry_at);
                        }
                    }
                    _ if entry.is_empty() => {}
                    _ => told.note(
                        DiagnosticCode::MissingRequiredField { text: None },
                        entry_at,
                    ),
                }
                continue;
            };
            for key in entry.keys().map(facet_value::VString::as_str) {
                if !READ.contains(&key) {
                    told.unknown(entry_at.child(key));
                }
            }
            let event = entry
                .get("breed_event")
                .cloned()
                .unwrap_or_else(empty_object);
            let event_at = entry_at.child("breed_event");
            if events.get(mate.as_str()).is_some_and(|had| *had != event)
                && let Some(earlier) = gave_event.get(mate.as_str())
            {
                told.dropped(earlier.clone());
            }
            if entry.contains_key("breed_event") {
                gave_event.insert(mate.as_str().to_owned(), event_at.clone());
                moves.push((event_at, with_at.child(mate.as_str())));
            } else {
                gave_event.remove(mate.as_str());
                // The mate's key is the entry's `mate_type`.
                moves.push((entry_at.child("mate_type"), with_at.child(mate.as_str())));
            }
            events.insert(mate.as_str(), event);
            if let Some(baby) = entry.get("baby_type") {
                let baby_at = entry_at.child("baby_type");
                if pairs.get(mate.as_str()).is_some_and(|had| had != baby)
                    && let Some(earlier) = gave_baby.get(mate.as_str())
                {
                    told.dropped(earlier.clone());
                }
                gave_baby.insert(mate.as_str().to_owned(), baby_at.clone());
                moves.push((
                    baby_at,
                    offspring_at.with(&["offspring_pairs", mate.as_str()]),
                ));
                pairs.insert(mate.as_str(), baby.clone());
            }
        }
        *with = events.into();
        if !pairs.is_empty() {
            offspring.insert("offspring_pairs", pairs);
        }
    }
    if !offspring.is_empty() {
        let offspring = Value::from(offspring);
        // The offspring members move into a new `offspring`: one the file has
        // beside the `breedable` is replaced whole, and noted where it is.
        if members
            .get("minecraft:offspring")
            .is_some_and(|had| !had.is_null() && *had != offspring)
        {
            told.dropped(offspring_at);
        }
        members.set("minecraft:offspring", offspring);
    }
    // A move into `offspring` exists only where it was made; the mates' and events' moves
    // stand without one.
    for (from, to) in moves {
        told.moved(from, to);
    }
}

/// `experience_reward` (typed → typed): an empty scalar whose `*_array` has exactly one
/// element takes that element; anything else is carried unchanged.
fn fold_reward(reward: &mut VObject, scalar: &str) {
    let array_key = format!("{scalar}_array");
    // A `null` member is an absent one.
    if reward.get(scalar).is_some_and(Value::is_null) {
        reward.remove(scalar);
    }
    let empty = reward
        .get(scalar)
        .is_none_or(|v| v.as_string().is_some_and(|s| s.as_str().is_empty()));
    let single = reward
        .get(&array_key)
        .and_then(Value::as_array)
        .filter(|a| a.len() == 1)
        .and_then(|a| a.get(0).cloned());
    if empty && let Some(element) = single {
        reward.insert(scalar, element);
        reward.remove(&array_key);
    }
}
