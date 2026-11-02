//! Format 1.16.200: the struct and the step to 1.17.10.

super::legacy_struct!(1, 16, 200);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Told, rename, section_b, sections};

/// 1.16.200 → 1.17.10 (gate 1.17.10).
impl Upgrade for Entity {
    type Next = super::v1_17_10::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // B, component rename: `scaffolding_climber` → `block_climber`, value unchanged.
        if section_b(&payload, cx) {
            let mut told = Told::default();
            for section in sections(&mut payload) {
                rename(
                    section.members,
                    &section.at,
                    "minecraft:scaffolding_climber",
                    "minecraft:block_climber",
                    &mut told,
                );
            }
            told.report(cx);
        }
        super::v1_17_10::Entity(payload)
    }
}
