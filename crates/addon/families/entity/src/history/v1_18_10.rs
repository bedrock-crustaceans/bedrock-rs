//! Format 1.18.10: the struct and the step to 1.18.20.

use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::{Told, section_b, sections, single_to_array_at};

/// An entity document as written in format 1.18.10 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 18, 10>;

/// 1.18.10 → 1.18.20 (gate 1.18.20).
impl LegacyStep<1, 18, 10> for Payload {
    type Next = super::v1_18_20::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // B, `behavior.go_home`: a single `on_home` and `on_failed` member becomes a
        // one-element array.
        if section_b(&payload, cx) {
            let mut told = Told::default();
            for section in sections(&mut payload) {
                let at = section.at.child("minecraft:behavior.go_home");
                if let Some(goal) = section.members.object("minecraft:behavior.go_home") {
                    single_to_array_at(goal, "on_home", &at, &mut told);
                    single_to_array_at(goal, "on_failed", &at, &mut told);
                }
            }
            told.report(cx);
        }
        Legacy(payload)
    }
}
