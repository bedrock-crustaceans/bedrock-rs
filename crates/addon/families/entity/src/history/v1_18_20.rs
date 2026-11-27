//! Format 1.18.20: the struct and the step to 1.18.30.

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::ops::empty_object;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::{Members, components, section_a};

/// An entity document as written in format 1.18.20 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 18, 20>;

/// 1.18.20 → 1.18.30 (gate 1.18.30).
impl LegacyStep<1, 18, 20> for Payload {
    type Next = super::v1_18_30::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // A, vanilla `player`: `exhaustion_values` when absent. Vanilla writes `""`, which its
        // legacy parser reads as the defaults; this crate writes `{}`, which reads the same.
        if section_a(&payload, cx) && cx.actor_type == ActorType::Player {
            components(&mut payload).insert_absent("minecraft:exhaustion_values", empty_object());
        }
        Legacy(payload)
    }
}
