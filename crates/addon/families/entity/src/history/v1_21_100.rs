//! Format 1.21.100: the struct and the step to 1.21.110.

super::legacy_struct!(1, 21, 100);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, Told, json, rename, section_b, sections};

/// 1.21.100 → 1.21.110.
impl Upgrade for Entity {
    type Next = super::v1_21_110::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // The conversions of this step whose legacy reader is one of the game's schema-driven
        // readers.
        super::ops::schema_conversions(&mut payload, super::v1_21_110::Entity::VERSION, cx);
        if section_b(&payload, cx) {
            let mut told = Told::default();
            for section in sections(&mut payload) {
                let members = section.members;
                // `input_air_controlled`: `vertical_movement_action {}` in the same section
                // when absent, then the rename to `free_camera_controlled`.
                if members.has("minecraft:input_air_controlled") {
                    members.insert_absent("minecraft:vertical_movement_action", json("{}"));
                    rename(
                        members,
                        &section.at,
                        "minecraft:input_air_controlled",
                        "minecraft:free_camera_controlled",
                        &mut told,
                    );
                }
                // `dash`: `direction: "entity"` when absent, then the rename to `dash_action`.
                if let Some(dash) = members.object("minecraft:dash") {
                    dash.insert_absent("direction", "entity".into());
                }
                rename(
                    members,
                    &section.at,
                    "minecraft:dash",
                    "minecraft:dash_action",
                    &mut told,
                );
            }
            told.report(cx);
        }
        super::v1_21_110::Entity(payload)
    }
}
