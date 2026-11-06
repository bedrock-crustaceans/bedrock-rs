//! Format 1.21.40: the struct and the step to 1.21.60.

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::ops::empty_object;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::fragments;
use super::ops::{Members, components, is, section_a};

/// An entity document as written in format 1.21.40 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 21, 40>;

/// 1.21.40 → 1.21.60 (gate 1.21.60).
impl LegacyStep<1, 21, 40> for Payload {
    type Next = super::v1_21_60::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        if section_a(&payload, cx) {
            use ActorType as T;
            let invisible = [
                T::Enderman,
                T::Shulker,
                T::Phantom,
                T::Spider,
                T::CaveSpider,
            ];
            if is(cx, &invisible) {
                components(&mut payload)
                    .insert_absent("minecraft:renders_when_invisible", empty_object());
            }
            if cx.actor_type == T::Strider {
                components(&mut payload).insert_absent(
                    "minecraft:movement_sound_distance_offset",
                    fragments::strider_sound_offset(),
                );
            }
        }
        Legacy(payload)
    }
}
