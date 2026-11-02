//! Format 1.21.0: the struct and the step to 1.21.10.

super::legacy_struct!(1, 21, 0);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, components, json, section_a, sections};

/// 1.21.0 → 1.21.10 (gate 1.21.10).
impl Upgrade for Entity {
    type Next = super::v1_21_10::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // A, all identifiers: `equippable` in `components` or any group puts the armor slot
        // mapping into `components` (only there) when absent.
        if section_a(&payload, cx) {
            let equippable = sections(&mut payload)
                .into_iter()
                .any(|section| section.members.has("minecraft:equippable"));
            if equippable {
                components(&mut payload).insert_absent(
                    "minecraft:entity_armor_equipment_slot_mapping",
                    json(r#"{"armor_slot": "slot.armor.chest"}"#),
                );
            }
        }
        super::v1_21_10::Entity(payload)
    }
}
