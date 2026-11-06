//! Format 1.21.60: the struct and the step to 1.21.70.

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::ops::empty_object;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::{Members, components, is, section_a, section_b, sections};

/// An entity document as written in format 1.21.60 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 21, 60>;

/// 1.21.60 → 1.21.70 (gate 1.21.70).
impl LegacyStep<1, 21, 60> for Payload {
    type Next = super::v1_21_70::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // A, vanilla boat, chest boat and shulker: `is_collidable {}` when absent.
        if section_a(&payload, cx)
            && is(
                cx,
                &[ActorType::Boat, ActorType::ChestBoat, ActorType::Shulker],
            )
        {
            components(&mut payload).insert_absent("minecraft:is_collidable", empty_object());
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
        Legacy(payload)
    }
}
