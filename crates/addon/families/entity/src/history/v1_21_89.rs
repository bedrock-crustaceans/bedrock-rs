//! Format 1.21.89: the struct and the step to 1.21.90.

super::legacy_struct!(1, 21, 89);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, section_b, sections};

/// 1.21.89 → 1.21.90.
impl Upgrade for Entity {
    type Next = super::v1_21_90::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // B, `leashable`: `can_be_cut` and `can_be_stolen` `false` when absent; every preset
        // gets `hard_distance: 6.0`, `max_distance: 10.0` and `spring_type: "bouncy"` when
        // absent. (C: `behavior.breed` becomes typed, which the last step's lenient read does.)
        if section_b(&payload, cx) {
            for section in sections(&mut payload) {
                let Some(leashable) = section.members.object("minecraft:leashable") else {
                    continue;
                };
                leashable.insert_absent("can_be_cut", false.into());
                leashable.insert_absent("can_be_stolen", false.into());
                if let Some(presets) = leashable.get_mut("presets").and_then(|p| p.as_array_mut()) {
                    for preset in presets.as_mut_slice() {
                        if let Some(preset) = preset.as_object_mut() {
                            preset.insert_absent("hard_distance", 6.0.into());
                            preset.insert_absent("max_distance", 10.0.into());
                            preset.insert_absent("spring_type", "bouncy".into());
                        }
                    }
                }
            }
        }
        super::v1_21_90::Entity(payload)
    }
}
