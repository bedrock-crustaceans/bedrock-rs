//! Format 1.18.30: the struct and the step to 1.19.0.

use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::{Members, section_b, sections};

/// An entity document as written in format 1.18.30 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 18, 30>;

/// 1.18.30 → 1.19.0 (gate 1.19.0).
impl LegacyStep<1, 18, 30> for Payload {
    type Next = super::v1_19_0::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // B: `behavior.sleep.can_sleep_while_riding: true` when absent (the schema default is
        // `false`; this keeps the old behaviour).
        if section_b(&payload, cx) {
            for section in sections(&mut payload) {
                if let Some(sleep) = section.members.object("minecraft:behavior.sleep") {
                    sleep.insert_absent("can_sleep_while_riding", true.into());
                }
            }
        }
        Legacy(payload)
    }
}
