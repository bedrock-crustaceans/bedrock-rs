//! Format 1.16.200: the struct and the step to 1.17.10.

use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::{Told, rename, section_b, sections};

/// An entity document as written in format 1.16.200 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 16, 200>;

/// 1.16.200 → 1.17.10 (gate 1.17.10).
impl LegacyStep<1, 16, 200> for Payload {
    type Next = super::v1_17_10::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // B, component rename: `scaffolding_climber` → `block_climber`, value unchanged.
        if section_b(&payload, cx) {
            let mut told = Told::default();
            for section in sections(&mut payload) {
                rename(
                    section.members,
                    &section.at,
                    "minecraft:scaffolding_climber",
                    "minecraft:block_climber",
                    &mut told,
                );
            }
            told.report(cx);
        }
        Legacy(payload)
    }
}
