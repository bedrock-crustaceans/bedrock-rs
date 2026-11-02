//! Format 1.21.110: the struct and the step to 1.21.120.

super::legacy_struct!(1, 21, 110);

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{
    Members, Told, below, components, json, overwrite, section_a, section_b, sections,
};

/// 1.21.110 → 1.21.120.
impl Upgrade for Entity {
    type Next = super::v1_21_120::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // The conversions of this step whose legacy reader is one of the game's schema-driven
        // readers.
        super::ops::schema_conversions(&mut payload, super::v1_21_120::Entity::VERSION, cx);
        // A, vanilla skeleton: `rotation_locked_to_vehicle {}` when absent.
        if section_a(&payload, cx) && cx.actor_type == ActorType::Skeleton {
            components(&mut payload)
                .insert_absent("minecraft:rotation_locked_to_vehicle", json("{}"));
        }
        // B, `projectile`: an object `on_hit.impact_damage` gets
        // `apply_knockback_to_blocking_targets: true` (written unconditionally).
        if section_b(&payload, cx) {
            // A flag of the file is written over, as the game writes it.
            let mut told = Told::default();
            for section in sections(&mut payload) {
                let at = below(
                    &section.at,
                    &["minecraft:projectile", "on_hit", "impact_damage"],
                );
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
        super::v1_21_120::Entity(payload)
    }
}
