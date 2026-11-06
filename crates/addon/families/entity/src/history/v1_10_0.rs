//! Format 1.10.0: the struct and the step to 1.11.0.

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::ops::empty_object;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::{Members, components, section_a};

/// An entity document as written in format 1.10.0 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 10, 0>;

/// 1.10.0 → 1.11.0 (gate 1.11.0).
impl LegacyStep<1, 10, 0> for Payload {
    type Next = super::v1_11_0::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // A, vanilla `player`, `components`: `minecraft:is_hidden_when_invisible {}` when absent.
        if section_a(&payload, cx) && cx.actor_type == ActorType::Player {
            components(&mut payload)
                .insert_absent("minecraft:is_hidden_when_invisible", empty_object());
        }
        Legacy(payload)
    }
}
