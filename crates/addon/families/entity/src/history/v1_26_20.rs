//! Format 1.26.20: the struct and the step to 1.26.30.

super::legacy_struct!(1, 26, 20);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};
use facet_value::VArray;

use super::ops::{Members, note, sections};
use bedrock_addon_core::diagnostic::DiagnosticCode;

/// The member of `behavior.pickup_items` the step adds.
const MEMBER: &str = "stop_if_holding_item";

/// 1.26.20 → 1.26.30.
impl Upgrade for Entity {
    type Next = super::v1_26_30::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // The conversions of this step whose legacy reader is one of the game's schema-driven
        // readers.
        super::ops::schema_conversions(&mut payload, super::v1_26_30::Entity::VERSION, cx);
        for section in sections(&mut payload) {
            let members = section.members;
            // C, typed → typed: the new members take their struct defaults.
            if let Some(goal) = members.object("minecraft:behavior.pickup_items")
                && !goal.insert_absent(MEMBER, false.into())
                && goal.get(MEMBER).and_then(Value::as_bool) == Some(false)
            {
                // The file's own `false`: the member gate forgets the member below 1.26.30,
                // and the reader holds its finding of a `false` back as the step's
                // (`read::step_inserted_member`), so it is told here; an information,
                // as the reader's unknown member of the hand-written legacy reader.
                let at = section
                    .at
                    .with(&["minecraft:behavior.pickup_items", MEMBER]);
                note(cx, DiagnosticCode::UnrecognizedField { text: None }, &at);
            }
            if let Some(pushable) = members.object("minecraft:pushable_by_entity") {
                pushable.insert_absent("presets", VArray::new().into());
            }
            if let Some(presets) = members
                .object("minecraft:apply_knockback_rules")
                .and_then(|r| r.get_mut("presets"))
                .and_then(|p| p.as_array_mut())
            {
                for preset in presets.as_mut_slice() {
                    if let Some(preset) = preset.as_object_mut() {
                        preset.insert_absent("extra_knockback_approach", "reapply_default".into());
                        preset.insert_absent("knockback_mode", "relative_horizontal".into());
                        preset.insert_absent("scale_with_damage", false.into());
                        preset.insert_absent("slowdown_scale", 0.5.into());
                    }
                }
            }
            // B-equivalent on the dynamic leash components: `unleash_on_removal: false`.
            for key in ["minecraft:leashable", "minecraft:leashable_to"] {
                if let Some(component) = members.object(key) {
                    component.insert_absent("unleash_on_removal", false.into());
                }
            }
        }
        super::v1_26_30::Entity(payload)
    }
}
