//! Format 1.21.110: the struct and the step to 1.21.120.

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::ops::empty_object;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx, Versioned};

use super::Payload;
use super::ops::{Members, Told, components, overwrite, section_a, section_b, sections};

/// An entity document as written in format 1.21.110 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 21, 110>;

/// 1.21.110 → 1.21.120.
impl LegacyStep<1, 21, 110> for Payload {
    type Next = super::v1_21_120::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // The conversions of this step whose legacy reader is one of the game's schema-driven
        // readers.
        super::ops::schema_conversions(&mut payload, super::v1_21_120::Entity::VERSION, cx);
        // A, vanilla skeleton: `rotation_locked_to_vehicle {}` when absent.
        if section_a(&payload, cx) && cx.actor_type == ActorType::Skeleton {
            components(&mut payload)
                .insert_absent("minecraft:rotation_locked_to_vehicle", empty_object());
        }
        // B, `projectile`: an object `on_hit.impact_damage` gets
        // `apply_knockback_to_blocking_targets: true` (written unconditionally).
        if section_b(&payload, cx) {
            // A flag of the file is written over, as the game writes it.
            let mut told = Told::default();
            for section in sections(&mut payload) {
                let at = section
                    .at
                    .with(&["minecraft:projectile", "on_hit", "impact_damage"]);
                if let Some(damage) = section
                    .members
                    .object("minecraft:projectile")
                    .and_then(|p| p.get_mut("on_hit"))
                    .and_then(|h| h.as_object_mut())
                    .and_then(|h| h.get_mut("impact_damage"))
                    .and_then(|d| d.as_object_mut())
                {
                    overwrite(
                        damage,
                        "apply_knockback_to_blocking_targets",
                        true.into(),
                        &at,
                        &mut told,
                    );
                }
            }
            told.report(cx);
        }
        Legacy(payload)
    }
}
