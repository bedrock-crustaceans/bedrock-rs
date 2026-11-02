//! Format 1.17.20: the struct and the step to 1.18.0.

super::legacy_struct!(1, 17, 20);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Removed, below, removed, section_a, sections};

/// 1.17.20 → 1.18.0 (gate 1.18.0).
impl Upgrade for Entity {
    type Next = super::v1_18_0::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // A, every section: `rideable.controlling_seat` is removed; from 1.18.0 the controlling
        // seat is always seat 0.
        if section_a(&payload, cx) {
            let mut gone = Vec::new();
            for section in sections(&mut payload) {
                if let Some(rideable) = section.members.object("minecraft:rideable")
                    && rideable.remove("controlling_seat").is_some()
                {
                    gone.push(below(
                        &section.at,
                        &["minecraft:rideable", "controlling_seat"],
                    ));
                }
            }
            for at in gone {
                removed(
                    cx,
                    Removed::Field,
                    &at,
                    "controlling_seat",
                    FormatVersion::new(1, 18, 0),
                );
            }
        }
        super::v1_18_0::Entity(payload)
    }
}
