//! Format 1.10.0: the struct and the step to 1.11.0.

super::legacy_struct!(1, 10, 0);

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, components, json, section_a};

/// 1.10.0 → 1.11.0 (gate 1.11.0).
impl Upgrade for Entity {
    type Next = super::v1_11_0::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // A, vanilla `player`, `components`: `minecraft:is_hidden_when_invisible {}` when absent.
        if section_a(&payload, cx) && cx.actor_type == ActorType::Player {
            components(&mut payload)
                .insert_absent("minecraft:is_hidden_when_invisible", json("{}"));
        }
        super::v1_11_0::Entity(payload)
    }
}
