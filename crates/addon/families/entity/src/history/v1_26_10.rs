//! Format 1.26.10: the struct and the step to 1.26.20.

super::legacy_struct!(1, 26, 10);

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, Told, components, json, rename, section_a, section_b, sections};

/// The format version of the game's iron-golem knockback upgrade flag: the iron golem's
/// knockback constants changed.
const IRON_GOLEM_KNOCKBACK: FormatVersion = FormatVersion::new(1, 18, 30);

/// The generic memory goal without its suffix.
const MOVE_TO_POSITION: &str = "minecraft:memory_behavior.move_to_position";

/// The key a legacy bare generic memory goal gets.
const LEGACY_MOVE_TO_POSITION: &str = "minecraft:memory_behavior.move_to_position.legacy";

/// 1.26.10 → 1.26.20.
impl Upgrade for Entity {
    type Next = super::v1_26_20::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // The conversions of this step whose legacy reader is one of the game's schema-driven
        // readers.
        super::ops::schema_conversions(&mut payload, super::v1_26_20::Entity::VERSION, cx);
        if section_a(&payload, cx) {
            // All identifiers: `uses_legacy_friction {}` when absent ("automatically added to
            // all entities with a format_version lower than 1.26.20").
            components(&mut payload).insert_absent("minecraft:uses_legacy_friction", json("{}"));
            if let Some(rules) = knockback_rules(cx) {
                // The crate's own information, since the fixer does it: a component of the document
                // that the fixer replaces by another value is told.
                const RULES: &str = "minecraft:apply_knockback_rules";
                let rules = json(&rules);
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
                    goal.insert_absent(
                        "leader_filters",
                        json(r#"{"test": "has_component", "subject": "other", "value": "minecraft:is_illager_captain"}"#),
                    );
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
        super::v1_26_20::Entity(payload)
    }
}

/// A, vanilla: the component written unconditionally. The
/// 1.26.20 horizontal powers (2.0 / 1.3 / 1.5 / 1.0) are written × 0.4, because this crate's
/// runtime applies `horizontal_power` unscaled, as the game's 1.26.40 does.
fn knockback_rules(cx: &UpgradeCx<'_>) -> Option<String> {
    let (h, v, cap, water) = match cx.actor_type {
        ActorType::IronGolem if cx.original_version < IRON_GOLEM_KNOCKBACK => {
            (0.8, 1.6, 0.8, false)
        }
        ActorType::IronGolem => (0.52, 0.39, 0.8, false),
        ActorType::Drowned => (0.6, -0.6, -0.4, true),
        ActorType::Egg => (0.4, 0.12, 0.12, false),
        _ => return None,
    };
    Some(format!(
        r#"{{"presets": [{{"horizontal_power": {h}, "vertical_power": {v}, "vertical_velocity_cap": {cap}, "check_if_target_is_immersed_in_water": {water}}}]}}"#
    ))
}
