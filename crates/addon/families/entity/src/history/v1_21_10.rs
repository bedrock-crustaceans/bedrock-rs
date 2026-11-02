//! Format 1.21.10: the struct and the step to 1.21.20.

super::legacy_struct!(1, 21, 10);

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, components, json, section_a, section_b, sections};

/// 1.21.10 → 1.21.20 (gate 1.21.20).
impl Upgrade for Entity {
    type Next = super::v1_21_20::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // A, vanilla dolphin: `behavior.swim_up_for_breath {priority: 1}` when absent.
        if section_a(&payload, cx) && cx.actor_type == ActorType::Dolphin {
            components(&mut payload).insert_absent(
                "minecraft:behavior.swim_up_for_breath",
                json(r#"{"priority": 1}"#),
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
        super::v1_21_20::Entity(payload)
    }
}
