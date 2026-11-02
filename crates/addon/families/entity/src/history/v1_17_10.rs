//! Format 1.17.10: the struct and the step to 1.17.20.

super::legacy_struct!(1, 17, 10);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::section_a;

/// 1.17.10 → 1.17.20 (gate 1.17.20).
impl Upgrade for Entity {
    type Next = super::v1_17_20::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // The 1.16.200 rewrite with the flag false: the vertical strength is the horizontal
        // one (only documents originally in [1.16.200, 1.17.20) still hold the old member).
        if section_a(&payload, cx) {
            let mut told = super::ops::Told::default();
            super::v1_16_100::knockback_roar(&mut payload, false, &mut told);
            told.report(cx);
        }
        super::v1_17_20::Entity(payload)
    }
}
