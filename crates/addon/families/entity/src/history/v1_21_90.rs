//! Format 1.21.90: the struct and the step to 1.21.100.

use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx, Versioned};

use super::Payload;
use super::fragments;
use super::ops::{Told, overwrite, rename, section_b, sections};

/// An entity document as written in format 1.21.90 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 21, 90>;

/// 1.21.90 → 1.21.100.
impl LegacyStep<1, 21, 90> for Payload {
    type Next = super::v1_21_100::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // The conversions of this step whose legacy reader is one of the game's schema-driven
        // readers.
        super::ops::schema_conversions(&mut payload, super::v1_21_100::Entity::VERSION, cx);
        if section_b(&payload, cx) {
            // The crate's own note: the step writes these members whatever the document has; a
            // member of the document that held another value is told as dropped.
            let mut told = Told::default();
            for section in sections(&mut payload) {
                let members = section.members;
                rename(
                    members,
                    &section.at,
                    "minecraft:body_rotation_axis_aligned",
                    "minecraft:rotation_axis_aligned",
                    &mut told,
                );
                if let Some(goal) = members.object("minecraft:behavior.enderman_leave_block") {
                    let at = section.at.child("minecraft:behavior.enderman_leave_block");
                    overwrite(goal, "xz_range", fragments::xz_range(), &at, &mut told);
                    overwrite(goal, "y_range", fragments::y_range_2(), &at, &mut told);
                    overwrite(goal, "chance", 0.0005.into(), &at, &mut told);
                }
                rename(
                    members,
                    &section.at,
                    "minecraft:behavior.enderman_leave_block",
                    "minecraft:behavior.place_block",
                    &mut told,
                );
                if let Some(goal) = members.object("minecraft:behavior.enderman_take_block") {
                    let at = section.at.child("minecraft:behavior.enderman_take_block");
                    overwrite(goal, "xz_range", fragments::xz_range(), &at, &mut told);
                    overwrite(goal, "y_range", fragments::y_range_3(), &at, &mut told);
                    overwrite(goal, "chance", 0.05.into(), &at, &mut told);
                    overwrite(goal, "requires_line_of_sight", false.into(), &at, &mut told);
                    overwrite(goal, "blocks", fragments::take_blocks(), &at, &mut told);
                }
                rename(
                    members,
                    &section.at,
                    "minecraft:behavior.enderman_take_block",
                    "minecraft:behavior.take_block",
                    &mut told,
                );
            }
            told.report(cx);
        }
        Legacy(payload)
    }
}
