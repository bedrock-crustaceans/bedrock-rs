//! Format 1.26.10: the struct and the step to 1.26.20.

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::ops::empty_object;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx, Versioned};

use super::Payload;
use super::fragments;
use super::ops::{Members, Told, components, rename, section_a, section_b, sections};

/// An entity document as written in format 1.26.10 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 26, 10>;

/// The format version of the game's iron-golem knockback upgrade flag: the iron golem's
/// knockback constants changed.
const IRON_GOLEM_KNOCKBACK: FormatVersion = FormatVersion::new(1, 18, 30);

/// The generic memory goal without its suffix.
const MOVE_TO_POSITION: &str = "minecraft:memory_behavior.move_to_position";

/// The key a legacy bare generic memory goal gets.
const LEGACY_MOVE_TO_POSITION: &str = "minecraft:memory_behavior.move_to_position.legacy";

/// 1.26.10 → 1.26.20.
impl LegacyStep<1, 26, 10> for Payload {
    type Next = super::v1_26_20::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // The conversions of this step whose legacy reader is one of the game's schema-driven
        // readers.
        super::ops::schema_conversions(&mut payload, super::v1_26_20::Entity::VERSION, cx);
        if section_a(&payload, cx) {
            // All identifiers: `uses_legacy_friction {}` when absent ("automatically added to
            // all entities with a format_version lower than 1.26.20").
            components(&mut payload)
                .insert_absent("minecraft:uses_legacy_friction", empty_object());
            if let Some(rules) = knockback_rules(cx) {
                // The crate's own information, since the fixer does it: a component of the document
                // that the fixer replaces by another value is told.
                const RULES: &str = "minecraft:apply_knockback_rules";
                let components = components(&mut payload);
                if components
                    .get(RULES)
                    .is_some_and(|had| !had.is_null() && *had != rules)
                {
                    let mut told = Told::default();
                    told.dropped(JsonPath::new().with(&["components", RULES]));
                    told.report(cx);
                }
                components.set(RULES, rules);
            }
        }
        // B: `follow_target_captain` → `follow_target_leader`, then two defaults.
        if section_b(&payload, cx) {
            let mut told = Told::default();
            for section in sections(&mut payload) {
                let members = section.members;
                rename(
                    members,
                    &section.at,
                    "minecraft:behavior.follow_target_captain",
                    "minecraft:behavior.follow_target_leader",
                    &mut told,
                );
                if let Some(goal) = members.object("minecraft:behavior.follow_target_leader") {
                    goal.insert_absent("leader_filters", fragments::captain_leader_filter());
                    goal.insert_absent("search_cooldown", 1.into());
                }
            }
            told.report(cx);
        }
        // C, the crate's own: the bare generic memory goal is a legacy key
        // below 1.26.20 and gets the suffix `.legacy`.
        let mut told = Told::default();
        for section in sections(&mut payload) {
            rename(
                section.members,
                &section.at,
                MOVE_TO_POSITION,
                LEGACY_MOVE_TO_POSITION,
                &mut told,
            );
        }
        told.report(cx);
        Legacy(payload)
    }
}

/// A, vanilla: the component written unconditionally. The
/// 1.26.20 horizontal powers (2.0 / 1.3 / 1.5 / 1.0) are written × 0.4, because this crate's
/// runtime applies `horizontal_power` unscaled, as the game's 1.26.40 does.
fn knockback_rules(cx: &UpgradeCx<'_>) -> Option<Value> {
    let (h, v, cap, water) = match cx.actor_type {
        ActorType::IronGolem if cx.original_version < IRON_GOLEM_KNOCKBACK => {
            (0.8, 1.6, 0.8, false)
        }
        ActorType::IronGolem => (0.52, 0.39, 0.8, false),
        ActorType::Drowned => (0.6, -0.6, -0.4, true),
        ActorType::Egg => (0.4, 0.12, 0.12, false),
        _ => return None,
    };
    Some(fragments::knockback_rules(h, v, cap, water))
}
