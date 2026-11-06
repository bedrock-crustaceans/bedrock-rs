//! Format 1.20.80: the struct and the step to 1.21.0.

use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::fragments;
use super::ops::{Members, section_b, sections};

/// An entity document as written in format 1.20.80 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 20, 80>;

/// 1.20.80 → 1.21.0 (gate 1.21.0).
impl LegacyStep<1, 20, 80> for Payload {
    type Next = super::v1_21_0::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // B: `behavior.panic.damage_sources: ["all"]` when absent, so an old document keeps
        // panicking on every cause.
        if section_b(&payload, cx) {
            for section in sections(&mut payload) {
                if let Some(panic) = section.members.object("minecraft:behavior.panic") {
                    panic.insert_absent("damage_sources", fragments::all_damage_sources());
                }
            }
        }
        Legacy(payload)
    }
}
