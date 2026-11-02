//! Format 1.18.30: the struct and the step to 1.19.0.

super::legacy_struct!(1, 18, 30);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, section_b, sections};

/// 1.18.30 → 1.19.0 (gate 1.19.0).
impl Upgrade for Entity {
    type Next = super::v1_19_0::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // B: `behavior.sleep.can_sleep_while_riding: true` when absent (the schema default is
        // `false`; this keeps the old behaviour).
        if section_b(&payload, cx) {
            for section in sections(&mut payload) {
                if let Some(sleep) = section.members.object("minecraft:behavior.sleep") {
                    sleep.insert_absent("can_sleep_while_riding", true.into());
                }
            }
        }
        super::v1_19_0::Entity(payload)
    }
}
