//! Format 1.21.60: the struct and the step to 1.21.70.

super::legacy_struct!(1, 21, 60);

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, components, is, json, section_a, section_b, sections};

/// 1.21.60 → 1.21.70 (gate 1.21.70).
impl Upgrade for Entity {
    type Next = super::v1_21_70::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // A, vanilla boat, chest boat and shulker: `is_collidable {}` when absent.
        if section_a(&payload, cx)
            && is(
                cx,
                &[ActorType::Boat, ActorType::ChestBoat, ActorType::Shulker],
            )
        {
            components(&mut payload).insert_absent("minecraft:is_collidable", json("{}"));
        }
        // B: `behavior.float_wander` gets two `false` members when absent.
        if section_b(&payload, cx) {
            for section in sections(&mut payload) {
                if let Some(goal) = section.members.object("minecraft:behavior.float_wander") {
                    goal.insert_absent("use_home_position_restriction", false.into());
                    goal.insert_absent("float_wander_has_move_control", false.into());
                }
            }
        }
        super::v1_21_70::Entity(payload)
    }
}
