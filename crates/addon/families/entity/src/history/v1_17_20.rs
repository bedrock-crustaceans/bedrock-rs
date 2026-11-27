//! Format 1.17.20: the struct and the step to 1.18.0.

use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::{Removed, removed, section_a, sections};

/// An entity document as written in format 1.17.20 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 17, 20>;

/// 1.17.20 → 1.18.0 (gate 1.18.0).
impl LegacyStep<1, 17, 20> for Payload {
    type Next = super::v1_18_0::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // A, every section: `rideable.controlling_seat` is removed; from 1.18.0 the controlling
        // seat is always seat 0.
        if section_a(&payload, cx) {
            let mut gone = Vec::new();
            for section in sections(&mut payload) {
                if let Some(rideable) = section.members.object("minecraft:rideable")
                    && rideable.remove("controlling_seat").is_some()
                {
                    gone.push(section.at.with(&["minecraft:rideable", "controlling_seat"]));
                }
            }
            for at in gone {
                removed(
                    cx,
                    Removed::Field,
                    &at,
                    "controlling_seat",
                    FormatVersion::new(1, 18, 0),
                );
            }
        }
        Legacy(payload)
    }
}
