//! Format 1.21.90: the struct and the step to 1.21.100.

super::legacy_struct!(1, 21, 90);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Told, below, json, overwrite, rename, section_b, sections};

/// 1.21.90 → 1.21.100.
impl Upgrade for Entity {
    type Next = super::v1_21_100::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
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
                    let at = below(&section.at, &["minecraft:behavior.enderman_leave_block"]);
                    overwrite(goal, "xz_range", json("[-1, 1]"), &at, &mut told);
                    overwrite(goal, "y_range", json("[0, 2]"), &at, &mut told);
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
                    let at = below(&section.at, &["minecraft:behavior.enderman_take_block"]);
                    overwrite(goal, "xz_range", json("[-1, 1]"), &at, &mut told);
                    overwrite(goal, "y_range", json("[0, 3]"), &at, &mut told);
                    overwrite(goal, "chance", 0.05.into(), &at, &mut told);
                    overwrite(goal, "requires_line_of_sight", false.into(), &at, &mut told);
                    overwrite(goal, "blocks", json(TAKE_BLOCKS), &at, &mut told);
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
        super::v1_21_100::Entity(payload)
    }
}

/// The legacy enderman-holdable blocks.
const TAKE_BLOCKS: &str = r#"["minecraft:grass_block", "minecraft:dirt", "minecraft:coarse_dirt", "minecraft:podzol",
  "minecraft:sand", "minecraft:red_sand", "minecraft:gravel", "minecraft:crimson_roots", "minecraft:warped_roots",
  "minecraft:dandelion", "minecraft:poppy", "minecraft:brown_mushroom", "minecraft:red_mushroom",
  "minecraft:crimson_fungus", "minecraft:warped_fungus", "minecraft:tnt", "minecraft:cactus", "minecraft:clay",
  "minecraft:pumpkin", "minecraft:carved_pumpkin", "minecraft:melon_block", "minecraft:mycelium",
  "minecraft:warped_nylium", "minecraft:crimson_nylium", "minecraft:dirt_with_roots", "minecraft:cactus_flower"]"#;
