//! Format 1.19.60: the struct and the step to 1.20.0.

use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::{Members, section_b, sections};

/// An entity document as written in format 1.19.60 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 19, 60>;

/// 1.19.60 → 1.20.0 (gate 1.20.0).
impl LegacyStep<1, 19, 60> for Payload {
    type Next = super::v1_20_0::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // B: `boostable.speed_multiplier: 1.35` when absent.
        if section_b(&payload, cx) {
            for section in sections(&mut payload) {
                if let Some(boostable) = section.members.object("minecraft:boostable") {
                    boostable.insert_absent("speed_multiplier", 1.35.into());
                }
            }
        }
        Legacy(payload)
    }
}
