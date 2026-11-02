//! Format 1.8.0, the floor: the struct and the step to 1.10.0.

super::legacy_struct!(1, 8, 0);

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, components, json, section_a};

/// 1.8.0 → 1.10.0 (gate 1.10.0).
impl Upgrade for Entity {
    type Next = super::v1_10_0::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        if section_a(&payload, cx) {
            boss(&mut payload, cx);
        }
        super::v1_10_0::Entity(payload)
    }
}

/// A, vanilla `wither` and `ender_dragon`, `components`: `minecraft:boss {}` when absent, then
/// `should_darken_sky` and `hud_range` when absent.
fn boss(payload: &mut Payload, cx: &UpgradeCx<'_>) {
    let (darken, range) = match cx.actor_type {
        ActorType::Wither => (true, 55),
        ActorType::EnderDragon => (false, 125),
        _ => return,
    };
    let components = components(payload);
    components.insert_absent("minecraft:boss", json("{}"));
    if let Some(boss) = components.object("minecraft:boss") {
        boss.insert_absent("should_darken_sky", darken.into());
        boss.insert_absent("hud_range", range.into());
    }
}
