//! Format 1.21.0: the struct and the step to 1.21.10.

use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::fragments;
use super::ops::{Members, components, section_a, sections};

/// An entity document as written in format 1.21.0 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 21, 0>;

/// 1.21.0 → 1.21.10 (gate 1.21.10).
impl LegacyStep<1, 21, 0> for Payload {
    type Next = super::v1_21_10::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // A, all identifiers: `equippable` in `components` or any group puts the armor slot
        // mapping into `components` (only there) when absent.
        if section_a(&payload, cx) {
            let equippable = sections(&mut payload)
                .into_iter()
                .any(|section| section.members.has("minecraft:equippable"));
            if equippable {
                components(&mut payload).insert_absent(
                    "minecraft:entity_armor_equipment_slot_mapping",
                    fragments::chest_armor_slot(),
                );
            }
        }
        Legacy(payload)
    }
}
