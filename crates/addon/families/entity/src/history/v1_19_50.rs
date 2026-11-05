//! Format 1.19.50: the struct and the step to 1.19.60.

super::legacy_struct!(1, 19, 50);

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, Told, components, json, section_a, section_b, sections};
use super::v1_19_0::append_all_of;

/// 1.19.50 → 1.19.60 (gate 1.19.60).
impl Upgrade for Entity {
    type Next = super::v1_19_60::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        if section_a(&payload, cx) && cx.actor_type == ActorType::Witch {
            // The 1.19.60 witch fragment is one constant. The crate's own
            // information, since the fixer does it: a component of the document that the
            // fragment replaces by another value is told.
            let mut told = Told::default();
            let components = components(&mut payload);
            for (key, fragment) in [
                ("minecraft:shooter", SHOOTER),
                ("minecraft:behavior.drink_potion", DRINK_POTION),
            ] {
                let fragment = json(fragment);
                if components
                    .get(key)
                    .is_some_and(|had| !had.is_null() && *had != fragment)
                {
                    told.dropped(JsonPath::new().with(&["components", key]));
                }
                components.set(key, fragment);
            }
            told.report(cx);
        }
        if section_b(&payload, cx) {
            let mut told = Told::default();
            for section in sections(&mut payload) {
                if let Some(goal) = section
                    .members
                    .object("minecraft:behavior.trade_with_player")
                {
                    append_all_of(
                        goal,
                        &[
                            r#"{"test": "in_water", "value": false}"#,
                            r#"{"any_of": [{"test": "on_ground", "value": true}, {"test": "is_sleeping", "value": true}]}"#,
                        ],
                        &section.at.child("minecraft:behavior.trade_with_player"),
                        &mut told,
                    );
                }
            }
            told.report(cx);
        }
        super::v1_19_60::Entity(payload)
    }
}

/// The witch's shooter. The filter JSON follows vanilla's own witch file.
const SHOOTER: &str = r#"{"def": "minecraft:splash_potion", "power": 0.75, "aux_val": 23, "sound": "throw", "magic": true, "projectiles": [
  {"aux_val": 17, "filters": {"all_of": [{"test": "target_distance", "subject": "self", "operator": ">=", "value": 8}, {"none_of": [{"test": "has_mob_effect", "subject": "other", "value": "slowness"}]}]}},
  {"aux_val": 25, "filters": {"all_of": [{"test": "actor_health", "subject": "other", "operator": ">=", "value": 8}, {"none_of": [{"test": "has_mob_effect", "subject": "other", "value": "poison"}]}]}},
  {"aux_val": 34, "chance": 0.25, "filters": {"all_of": [{"test": "target_distance", "subject": "self", "operator": "<=", "value": 3}, {"none_of": [{"test": "has_mob_effect", "subject": "other", "value": "weakness"}]}]}}
]}"#;

/// The witch's potion goal.
const DRINK_POTION: &str = r#"{"priority": 1, "speed_modifier": -0.25, "potions": [
  {"id": 19, "chance": 0.15, "filters": {"test": "is_underwater", "subject": "self", "value": true}},
  {"id": 12, "chance": 0.15, "filters": {"any_of": [{"test": "on_fire", "subject": "self", "value": true}, {"test": "on_hot_block", "subject": "self", "value": true}]}},
  {"id": 21, "chance": 0.05, "filters": {"test": "is_missing_health", "subject": "self", "value": true}},
  {"id": 14, "chance": 0.25, "filters": {"all_of": [{"test": "has_target", "subject": "self", "value": true}, {"test": "target_distance", "subject": "self", "operator": ">=", "value": 11}]}}
]}"#;
