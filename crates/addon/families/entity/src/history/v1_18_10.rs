//! Format 1.18.10: the struct and the step to 1.18.20.

super::legacy_struct!(1, 18, 10);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Told, below, section_b, sections, single_to_array_at};

/// 1.18.10 → 1.18.20 (gate 1.18.20).
impl Upgrade for Entity {
    type Next = super::v1_18_20::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // B, `behavior.go_home`: a single `on_home` and `on_failed` member becomes a
        // one-element array.
        if section_b(&payload, cx) {
            let mut told = Told::default();
            for section in sections(&mut payload) {
                let at = below(&section.at, &["minecraft:behavior.go_home"]);
                if let Some(goal) = section.members.object("minecraft:behavior.go_home") {
                    single_to_array_at(goal, "on_home", &at, &mut told);
                    single_to_array_at(goal, "on_failed", &at, &mut told);
                }
            }
            told.report(cx);
        }
        super::v1_18_20::Entity(payload)
    }
}
