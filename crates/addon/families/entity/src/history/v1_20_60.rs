//! Format 1.20.60: the struct and the step to 1.20.70.

use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};
use facet_value::VArray;

use super::Payload;
use super::ops::{section_b, sections};

/// An entity document as written in format 1.20.60 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 20, 60>;

/// 1.20.60 → 1.20.70 (gate 1.20.70).
impl LegacyStep<1, 20, 60> for Payload {
    type Next = super::v1_20_70::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
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
        Legacy(payload)
    }
}
