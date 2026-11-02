//! Format 1.21.40: the struct and the step to 1.21.60.

super::legacy_struct!(1, 21, 40);

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, components, is, json, section_a};

/// 1.21.40 → 1.21.60 (gate 1.21.60).
impl Upgrade for Entity {
    type Next = super::v1_21_60::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
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
                    .insert_absent("minecraft:renders_when_invisible", json("{}"));
            }
            if cx.actor_type == T::Strider {
                components(&mut payload).insert_absent(
                    "minecraft:movement_sound_distance_offset",
                    json(r#"{"value": 0.6}"#),
                );
            }
        }
        super::v1_21_60::Entity(payload)
    }
}
