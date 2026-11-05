//! The member conversions of goals read from documents older than the newest struct: on the
//! JSON before a lenient read (a document below the goal's typed version), and on the typed
//! value after a strict read of a document below a later redeclaration.
//!
//! Each does nothing when the members it converts are absent, so applying it twice is applying
//! it once.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictFloat, Value};
use bedrock_addon_core::shared::{EventTrigger, FloatRange, RangeForm};
use facet_value::VObject;

use super::check::float_of;
use super::common::InRangeMovementMode;
use super::{EatAndReplaceBlockPair, EatBlock, RangedAttack};

/// A finding of a conversion at the top-level member `name`: an information. The game writes
/// no content-log line for what its own upgrade does, so the crate reports it as an information.
fn finding(code: DiagnosticCode, phase: Phase, name: &str) -> Diagnostic {
    let mut at = JsonPath::new();
    at.push(name);
    Diagnostic::info(code).at(at).phase(phase)
}

/// The path of the top-level member `name`.
fn member(name: &str) -> JsonPath {
    let mut at = JsonPath::new();
    at.push(name);
    at
}

/// A note of a conversion at the top-level member `name`: an information about what the game
/// itself does without a word.
fn note(code: DiagnosticCode, name: &str) -> Diagnostic {
    Diagnostic::info(code)
        .at(member(name))
        .phase(Phase::Upgrade)
}

/// The top-level member `name` as one half of a range a conversion builds, taken out: its
/// number. A value that is no number is no half (`InvalidType`), so the other
/// half is kept; a number beyond a float is the largest finite float of its sign
/// (`InvalidValue`); both warnings at the member. A `null` member is an absent one.
fn half(json: &mut VObject, name: &str, d: &mut Diagnostics) -> Option<Value> {
    let value = json.remove(name).filter(|value| !value.is_null())?;
    let Some(number) = value.as_number() else {
        d.push(finding(
            DiagnosticCode::InvalidType { text: None },
            Phase::Upgrade,
            name,
        ));
        return None;
    };
    #[allow(clippy::cast_possible_truncation, reason = "the range's f32")]
    let float = number.to_f64_lossy() as f32;
    if float.is_infinite() {
        d.push(finding(
            DiagnosticCode::InvalidValue { text: None },
            Phase::Upgrade,
            name,
        ));
        return Some(Value::from(f64::from(f32::MAX.copysign(float))));
    }
    Some(value)
}

/// The melee goals' `on_kill`: below 1.26.40 vanilla bound both `on_attack` and `on_kill` to the
/// attack trigger, the later one winning, so an `on_kill` replaces `on_attack` and is removed.
// The game's schema binds both keys to the same attack trigger, and the key bound later wins,
// which is why `on_kill` is the value kept.
pub(crate) fn fold_on_kill(json: &mut VObject, d: &mut Diagnostics) {
    if let Some(on_kill) = json.remove("on_kill") {
        json.insert("on_attack", on_kill);
        d.moved(member("on_kill"), member("on_attack"));
    }
}

/// [`fold_on_kill`] on a melee goal read strictly from a document at or above its typed version
/// and below 1.26.40.
pub(crate) fn fold_on_kill_fields(
    on_attack: &mut Option<EventTrigger>,
    on_kill: &mut Option<EventTrigger>,
) {
    if let Some(trigger) = on_kill.take() {
        *on_attack = Some(trigger);
    }
}

/// `look_at_*` below 1.26.0: `target_distance` is removed (`DroppedField`, the crate's own text,
/// an information), and `min_look_time` / `max_look_time` become `look_time: {min, max}` (2 and 4
/// for a missing one).
///
/// The goals are `behavior.look_at_player`, `look_at_target`, `look_at_entity` and
/// `look_at_trading_player`. The game's upgrade drops the removed member without a content-log
/// line, so the crate reports the removal, with its own text, as an information.
pub(crate) fn look_time(json: &mut VObject, d: &mut Diagnostics) {
    if json.remove("target_distance").is_some() {
        // The game's upgrade drops it without a word, so the crate's own text is an information.
        let mut dropped = finding(
            DiagnosticCode::DroppedField {
                text: Some(VanillaText::Fixed(
                    "`target_distance` was removed by the 1.26.0 upgrade",
                )),
            },
            Phase::Upgrade,
            "target_distance",
        );
        dropped.severity = Severity::Info;
        d.push(dropped);
    }
    let min = half(json, "min_look_time", d);
    let max = half(json, "max_look_time", d);
    if min.is_some() || max.is_some() {
        let moved = [
            ("min_look_time", "min", min.is_some()),
            ("max_look_time", "max", max.is_some()),
        ];
        let mut range = VObject::new();
        range.insert("min", min.unwrap_or_else(|| Value::from(2)));
        range.insert("max", max.unwrap_or_else(|| Value::from(4)));
        let range = Value::from(range);
        // A `look_time` the file has beside the old pair is written over; this is noted as an
        // information because the pair is what the game reads below 1.26.0.
        if json
            .get("look_time")
            .is_some_and(|had| !had.is_null() && *had != range)
        {
            d.push(note(
                DiagnosticCode::DroppedField { text: None },
                "look_time",
            ));
        }
        json.insert("look_time", range);
        // What is told of the range as a whole is told at the first member of the pair
        // the file has; of a half, at its own member.
        if let Some((old, _, _)) = moved.iter().find(|(_, _, was)| *was) {
            d.moved(member(old), member("look_time"));
        }
        for (old, half, was) in moved {
            if was {
                let mut to = member("look_time");
                to.push(half);
                d.moved(member(old), to);
            }
        }
    }
}

/// `ranged_attack` below 1.26.40: `attack_interval_min` / `attack_interval_max` become
/// `attack_interval: {min, max}` (a missing half 0); the pair wins over a scalar
/// `attack_interval`, which is dropped with `DroppedField` (the crate's own report).
pub(crate) fn ranged_interval(json: &mut VObject, d: &mut Diagnostics) {
    let min = half(json, "attack_interval_min", d);
    let max = half(json, "attack_interval_max", d);
    if min.is_some() || max.is_some() {
        if json.contains_key("attack_interval") {
            d.push(finding(
                DiagnosticCode::DroppedField { text: None },
                Phase::Upgrade,
                "attack_interval",
            ));
        }
        let moved = [
            ("attack_interval_min", "min", min.is_some()),
            ("attack_interval_max", "max", max.is_some()),
        ];
        let mut range = VObject::new();
        range.insert("min", min.unwrap_or_else(|| Value::from(0)));
        range.insert("max", max.unwrap_or_else(|| Value::from(0)));
        json.insert("attack_interval", Value::from(range));
        if let Some((old, _, _)) = moved.iter().find(|(_, _, was)| *was) {
            d.moved(member(old), member("attack_interval"));
        }
        for (old, half, was) in moved {
            if was {
                let mut to = member("attack_interval");
                to.push(half);
                d.moved(member(old), to);
            }
        }
    }
}

/// `ranged_attack` below 1.26.50: `attack_radius_min` / `attack_radius` become
/// `attack_range: {min, max}` (a missing half 0, the `v1_26_40` definition's default), in the
/// order written, and `in_range_movement_mode` is `hold_position`, the old behaviour, when not
/// written. The game's upgrade copies the pair into the range, in the order written and with no
/// check, so a pair out of order still makes a range, and writes mode 0. A half that is no number
/// is no half ([`half`]). The moves are recorded.
pub(crate) fn ranged_radius(json: &mut VObject, d: &mut Diagnostics) {
    {
        let min = half(json, "attack_radius_min", d);
        let max = half(json, "attack_radius", d);
        if min.is_some() || max.is_some() {
            let moved = [
                ("attack_radius_min", "min", min.is_some()),
                ("attack_radius", "max", max.is_some()),
            ];
            let mut range = VObject::new();
            range.insert("min", min.unwrap_or_else(|| Value::from(0)));
            range.insert("max", max.unwrap_or_else(|| Value::from(0)));
            json.insert("attack_range", Value::from(range));
            if let Some((old, _, _)) = moved.iter().find(|(_, _, was)| *was) {
                d.moved(member(old), member("attack_range"));
            }
            for (old, half, was) in moved {
                if was {
                    let mut to = member("attack_range");
                    to.push(half);
                    d.moved(member(old), to);
                }
            }
        }
    }
    if !json.contains_key("in_range_movement_mode") {
        json.insert("in_range_movement_mode", Value::from("hold_position"));
    }
}

/// [`ranged_radius`] on a `ranged_attack` read strictly from a 1.26.40 document: the 1.26.40
/// struct's `attack_radius_min` / `attack_radius` (in `extra`) become `attack_range`, in the
/// order written, as the game's upgrade makes it.
pub(crate) fn ranged_radius_typed(goal: &mut RangedAttack) {
    let take = |extra: &mut Extra, name: &str| {
        let value = extra.remove(name)?;
        let x = float_of(&value);
        if x.is_none() {
            // Not a number: left for the unknown-member report.
            extra.insert(name.to_owned(), value);
        }
        x
    };
    let min = take(&mut goal.extra, "attack_radius_min");
    let max = take(&mut goal.extra, "attack_radius");
    if min.is_some() || max.is_some() {
        goal.attack_range = Some(FloatRange::new(
            min.unwrap_or(0.0),
            max.unwrap_or(0.0),
            RangeForm::MinMax,
        ));
    }
    goal.in_range_movement_mode
        .get_or_insert(InRangeMovementMode::HoldPosition);
}

/// `aquatic_charge_attack` below 1.21.120: `charge_speed_multiplier` is `charge_speed`.
pub(crate) fn charge_speed(json: &mut VObject) {
    if let Some(value) = json.remove("charge_speed_multiplier")
        && !json.contains_key("charge_speed")
    {
        json.insert("charge_speed", value);
    }
}

/// [`charge_speed`] on an `aquatic_charge_attack` read strictly from a document in
/// [1.21.120, 1.26.0), where `charge_speed_multiplier` is the live name (in `extra`).
pub(crate) fn charge_speed_typed(charge_speed: &mut Option<StrictFloat<f32>>, extra: &mut Extra) {
    let Some(value) = extra.remove("charge_speed_multiplier") else {
        return;
    };
    match float_of(&value) {
        Some(x) => {
            charge_speed.get_or_insert(StrictFloat(x));
        }
        None => {
            extra.insert("charge_speed_multiplier".to_owned(), value);
        }
    }
}

/// `eat_block.success_chance` below 1.26.40, read by the schema-validated reader: the object
/// form `{expression, version}` is an `InvalidType` warning and the member takes its default.
pub(crate) fn success_chance(json: &mut VObject, d: &mut Diagnostics) {
    if json.get("success_chance").is_some_and(Value::is_object) {
        json.remove("success_chance");
        d.push(finding(
            DiagnosticCode::InvalidType { text: None },
            Phase::Parse,
            "success_chance",
        ));
    }
}

/// The check of `eat_block`'s pairs against the blocks `is_block` knows: a pair naming
/// an unknown block is reported with vanilla's text (a kept error, whatever the goal's typed
/// version) and dropped (the crate's own step). Needs the block registry, so the link pass
/// (`pack/link/entity/entity_block.rs`) runs it and reports `UnresolvedReference`.
#[doc(hidden)]
pub fn eat_block_pairs(goal: &mut EatBlock, is_block: impl Fn(&str) -> bool) -> Diagnostics {
    let mut out = Diagnostics::default();
    let Some(pairs) = &mut goal.eat_and_replace_block_pairs else {
        return out;
    };
    let mut index = 0;
    pairs.retain(|pair: &EatAndReplaceBlockPair| {
        let names = [&pair.eat_block, &pair.replace_block];
        let unknown = names
            .into_iter()
            .flatten()
            .find(|name| !is_block(&name.0))
            .map(|name| name.0.clone());
        if let Some(block) = &unknown {
            let mut at = JsonPath::new();
            at.push("eat_and_replace_block_pairs");
            at.push_index(index);
            // The game looks the block up when it sets the goal up, in either reader, and logs
            // an error but keeps the entity: a kept error.
            out.push_kept_error(
                Diagnostic::error(DiagnosticCode::UnresolvedReference {
                    text: Some(VanillaText::EatBlockInvalidBlock {
                        block: block.clone(),
                    }),
                })
                .at(at)
                .phase(Phase::Link),
            );
        }
        index += 1;
        unknown.is_none()
    });
    out
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::json::StrictString;

    use super::*;

    #[test]
    fn each_conversion_is_idempotent() {
        type Conversion = fn(&mut VObject, &mut Diagnostics);
        let json = |text: &str| {
            bedrock_addon_core::json::from_str::<Value>(text)
                .expect("json")
                .as_object()
                .expect("object")
                .clone()
        };
        let mut d = Diagnostics::default();
        let conversions: [(&str, Conversion); 5] = [
            (r#"{"on_attack": 1, "on_kill": 2}"#, fold_on_kill),
            (r#"{"target_distance": 3, "min_look_time": 1}"#, look_time),
            (r#"{"attack_interval_max": 3}"#, ranged_interval),
            (r#"{"attack_radius": 9}"#, ranged_radius),
            (r#"{"charge_speed_multiplier": 0.5}"#, |j, _| {
                charge_speed(j);
            }),
        ];
        for (text, convert) in conversions {
            let mut once = json(text);
            convert(&mut once, &mut d);
            let mut twice = once.clone();
            convert(&mut twice, &mut d);
            assert_eq!(once, twice, "{text}");
        }
    }

    /// `eat_block` with an unknown block in a pair:
    /// the game's text, and the pair is dropped; the other pairs stay.
    #[test]
    fn v12_an_eat_block_pair_naming_an_unknown_block_is_dropped_with_vanillas_text() {
        let pair = |eat: &str, replace: &str| EatAndReplaceBlockPair {
            eat_block: Some(StrictString(eat.to_owned())),
            replace_block: Some(StrictString(replace.to_owned())),
            ..EatAndReplaceBlockPair::default()
        };
        let mut goal = EatBlock {
            eat_and_replace_block_pairs: Some(vec![
                pair("grass", "dirt"),
                pair("demo:nothing", "air"),
                pair("fern", "air"),
            ]),
            ..EatBlock::default()
        };
        let known = |name: &str| !name.starts_with("demo:");
        let d = eat_block_pairs(&mut goal, known);
        assert_eq!(d.len(), 1);
        let finding = d.iter().next().expect("one finding");
        assert_eq!(finding.severity, Severity::Error);
        assert!(finding.kept, "the entity is kept");
        assert_eq!(
            finding.location.json_path.to_string(),
            "eat_and_replace_block_pairs/1"
        );
        let DiagnosticCode::UnresolvedReference { text: Some(text) } = &finding.code else {
            panic!("{finding:?}");
        };
        assert_eq!(
            text.to_string(),
            "Block demo:nothing on \"eat_and_replace_block_pairs\" on \"minecraft:behavior.eat_block\" is not a valid block."
        );
        let pairs = goal.eat_and_replace_block_pairs.expect("pairs");
        assert_eq!(pairs.len(), 2, "the pair is dropped");
        let lenient = eat_block_pairs(
            &mut EatBlock {
                eat_and_replace_block_pairs: Some(vec![pair("demo:nothing", "air")]),
                ..EatBlock::default()
            },
            known,
        );
        assert_eq!(
            lenient.iter().next().map(|f| (f.severity, f.kept)),
            Some((Severity::Error, true))
        );
    }
}
