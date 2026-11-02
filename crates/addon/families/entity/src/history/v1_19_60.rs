//! Format 1.19.60: the struct and the step to 1.20.0.

super::legacy_struct!(1, 19, 60);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, section_b, sections};

/// 1.19.60 → 1.20.0 (gate 1.20.0).
impl Upgrade for Entity {
    type Next = super::v1_20_0::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // B: `boostable.speed_multiplier: 1.35` when absent.
        if section_b(&payload, cx) {
            for section in sections(&mut payload) {
                if let Some(boostable) = section.members.object("minecraft:boostable") {
                    boostable.insert_absent("speed_multiplier", 1.35.into());
                }
            }
        }
        super::v1_20_0::Entity(payload)
    }
}
