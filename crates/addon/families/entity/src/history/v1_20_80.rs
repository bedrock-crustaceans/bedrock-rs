//! Format 1.20.80: the struct and the step to 1.21.0.

super::legacy_struct!(1, 20, 80);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, json, section_b, sections};

/// 1.20.80 → 1.21.0 (gate 1.21.0).
impl Upgrade for Entity {
    type Next = super::v1_21_0::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // B: `behavior.panic.damage_sources: ["all"]` when absent, so an old document keeps
        // panicking on every cause.
        if section_b(&payload, cx) {
            for section in sections(&mut payload) {
                if let Some(panic) = section.members.object("minecraft:behavior.panic") {
                    panic.insert_absent("damage_sources", json(r#"["all"]"#));
                }
            }
        }
        super::v1_21_0::Entity(payload)
    }
}
