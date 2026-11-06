//! Format 1.11.0: the struct and the step to 1.12.0.

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::ops::empty_object;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::{Members, components, f32_value, section_a};

/// An entity document as written in format 1.11.0 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 11, 0>;

/// 1.11.0 → 1.12.0 (gate 1.12.0).
impl LegacyStep<1, 11, 0> for Payload {
    type Next = super::v1_12_0::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        if section_a(&payload, cx) {
            water_movement(&mut payload, cx);
        }
        Legacy(payload)
    }
}

/// A, vanilla, `components`: `minecraft:water_movement {}` when absent, then `drag_factor`
/// when absent: the `f32` 0.98 for panda and polar bear, the `f32` 0.9 for turtle, widened.
fn water_movement(payload: &mut Payload, cx: &UpgradeCx<'_>) {
    let drag = match cx.actor_type {
        ActorType::Panda | ActorType::PolarBear => 0.98_f32,
        ActorType::Turtle => 0.9_f32,
        _ => return,
    };
    let components = components(payload);
    components.insert_absent("minecraft:water_movement", empty_object());
    if let Some(movement) = components.object("minecraft:water_movement") {
        movement.insert_absent("drag_factor", f32_value(drag));
    }
}
