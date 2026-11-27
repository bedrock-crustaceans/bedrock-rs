//! Format 1.16.100: the struct and the step to 1.16.200.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::{Members, Told, section_a, sections};

/// An entity document as written in format 1.16.100 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 16, 100>;

/// 1.16.100 → 1.16.200 (gate 1.16.200, carried by vanilla's 1.17.20 fixer).
impl LegacyStep<1, 16, 100> for Payload {
    type Next = super::v1_16_200::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        if section_a(&payload, cx) {
            // The working version is below 1.16.200 here, which selects a vertical strength
            // of 1.
            let mut told = Told::default();
            knockback_roar(&mut payload, true, &mut told);
            told.report(cx);
        }
        Legacy(payload)
    }
}

/// The knockback-roar upgrade, every section: an integer `knockback_strength: n` is
/// removed, and `knockback_horizontal_strength: n` and `knockback_vertical_strength` (1 when
/// `vertical_one`, else `n`) are written when absent.
pub(super) fn knockback_roar(payload: &mut Payload, vertical_one: bool, told: &mut Told) {
    const GOAL: &str = "minecraft:behavior.knockback_roar";
    for section in sections(payload) {
        let at = section.at.child(GOAL);
        let Some(goal) = section.members.object(GOAL) else {
            continue;
        };
        let integer = goal
            .get("knockback_strength")
            .and_then(Value::as_number)
            .is_some_and(facet_value::VNumber::is_integer);
        if !integer {
            continue;
        }
        let Some(strength) = goal.remove("knockback_strength") else {
            continue;
        };
        let vertical = if vertical_one {
            Value::from(1)
        } else {
            strength.clone()
        };
        let old = at.child("knockback_strength");
        if goal.insert_absent("knockback_horizontal_strength", strength) {
            told.moved(old.clone(), at.child("knockback_horizontal_strength"));
        } else {
            // The file has the new member beside the old one: the old one goes, as in the
            // game, and is noted.
            told.dropped(old.clone());
        }
        if goal.insert_absent("knockback_vertical_strength", vertical) && !vertical_one {
            told.moved(old, at.child("knockback_vertical_strength"));
        }
    }
}
