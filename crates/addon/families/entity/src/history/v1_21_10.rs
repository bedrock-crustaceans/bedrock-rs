//! Format 1.21.10: the struct and the step to 1.21.20.

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::fragments;
use super::ops::{Members, components, section_a, section_b, sections};

/// An entity document as written in format 1.21.10 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 21, 10>;

/// 1.21.10 → 1.21.20 (gate 1.21.20).
impl LegacyStep<1, 21, 10> for Payload {
    type Next = super::v1_21_20::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // A, vanilla dolphin: `behavior.swim_up_for_breath {priority: 1}` when absent.
        if section_a(&payload, cx) && cx.actor_type == ActorType::Dolphin {
            components(&mut payload).insert_absent(
                "minecraft:behavior.swim_up_for_breath",
                fragments::priority_one(),
            );
        }
        // B: `behavior.follow_owner.post_teleport_distance: 1` when absent.
        if section_b(&payload, cx) {
            for section in sections(&mut payload) {
                if let Some(goal) = section.members.object("minecraft:behavior.follow_owner") {
                    goal.insert_absent("post_teleport_distance", 1.into());
                }
            }
        }
        Legacy(payload)
    }
}
