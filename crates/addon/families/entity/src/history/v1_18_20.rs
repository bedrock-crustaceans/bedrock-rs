//! Format 1.18.20: the struct and the step to 1.18.30.

super::legacy_struct!(1, 18, 20);

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, components, json, section_a};

/// 1.18.20 → 1.18.30 (gate 1.18.30).
impl Upgrade for Entity {
    type Next = super::v1_18_30::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // A, vanilla `player`: `exhaustion_values` when absent. Vanilla writes `""`, which its
        // legacy parser reads as the defaults; this crate writes `{}`, which reads the same.
        if section_a(&payload, cx) && cx.actor_type == ActorType::Player {
            components(&mut payload).insert_absent("minecraft:exhaustion_values", json("{}"));
        }
        super::v1_18_30::Entity(payload)
    }
}
