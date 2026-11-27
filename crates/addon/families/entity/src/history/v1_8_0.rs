//! Format 1.8.0, the floor: the struct and the step to 1.10.0.

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::ops::empty_object;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::{Members, components, section_a};

/// An entity document as written in format 1.8.0 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 8, 0>;

/// 1.8.0 → 1.10.0 (gate 1.10.0).
impl LegacyStep<1, 8, 0> for Payload {
    type Next = super::v1_10_0::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        if section_a(&payload, cx) {
            boss(&mut payload, cx);
        }
        Legacy(payload)
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
    components.insert_absent("minecraft:boss", empty_object());
    if let Some(boss) = components.object("minecraft:boss") {
        boss.insert_absent("should_darken_sky", darken.into());
        boss.insert_absent("hud_range", range.into());
    }
}
