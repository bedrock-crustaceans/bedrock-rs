//! Format 1.20.60: the struct and the step to 1.20.70.

super::legacy_struct!(1, 20, 60);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};
use facet_value::VArray;

use super::ops::{section_b, sections};

/// 1.20.60 → 1.20.70 (gate 1.20.70).
impl Upgrade for Entity {
    type Next = super::v1_20_70::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // B: each `entity_sensor.subsensors[].range` that is a number `r` becomes `[r, r]`
        // (horizontal, vertical).
        if section_b(&payload, cx) {
            for section in sections(&mut payload) {
                let Some(list) = section
                    .members
                    .object("minecraft:entity_sensor")
                    .and_then(|s| s.get_mut("subsensors"))
                    .and_then(|s| s.as_array_mut())
                else {
                    continue;
                };
                for subsensor in list.as_mut_slice() {
                    if let Some(range) = subsensor
                        .as_object_mut()
                        .and_then(|s| s.get_mut("range"))
                        .filter(|r| r.is_number())
                    {
                        let mut pair = VArray::new();
                        pair.push(range.clone());
                        pair.push(range.clone());
                        *range = pair.into();
                    }
                }
            }
        }
        super::v1_20_70::Entity(payload)
    }
}
