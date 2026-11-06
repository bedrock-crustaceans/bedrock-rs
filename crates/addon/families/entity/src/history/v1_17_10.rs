//! Format 1.17.10: the struct and the step to 1.17.20.

use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::section_a;

/// An entity document as written in format 1.17.10 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 17, 10>;

/// 1.17.10 → 1.17.20 (gate 1.17.20).
impl LegacyStep<1, 17, 10> for Payload {
    type Next = super::v1_17_20::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // The 1.16.200 rewrite with the flag false: the vertical strength is the horizontal
        // one (only documents originally in [1.16.200, 1.17.20) still hold the old member).
        if section_a(&payload, cx) {
            let mut told = super::ops::Told::default();
            super::v1_16_100::knockback_roar(&mut payload, false, &mut told);
            told.report(cx);
        }
        Legacy(payload)
    }
}
