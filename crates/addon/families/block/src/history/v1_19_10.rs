//! Format 1.19.10: the unwrapped components (`collision_box`, `light_emission`,
//! `queued_ticking`, …).

use super::legacy::{BlockPayload, input_check_failed, number};
use super::ops::{clamp, real};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, JsonPath, Phase};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx, Versioned};

/// A block document of `[1.19.10, 1.19.20)`.
pub type Block = Legacy<BlockPayload, 1, 19, 10>;

/// Vanilla's 1.19.20 fixers.
impl LegacyStep<1, 19, 10> for BlockPayload {
    type Next = super::v1_19_20::Block;

    fn step(this: Block, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block = this.0;
        let to = <Self::Next as Versioned>::VERSION;
        let d = &mut *cx.diagnostics;
        // Input check on the base: `friction` in 0.1..=1.0.
        if let Some(friction) = block.components.get("minecraft:friction")
            && !number(friction).is_some_and(|n| (0.1..=1.0).contains(&n))
        {
            input_check_failed("minecraft:friction", to, cx.original_version, d);
        }
        // `creative_category` → `description.menu_category`, base first, then each permutation
        // in order (the last one wins); removed from every section.
        let mut categories = Vec::new();
        for (section, map) in block.sections_mut() {
            if let Some(category) = map.shift_remove("minecraft:creative_category") {
                categories.push((section, category));
            }
        }
        // The category the file has before the component is copied over it: its own
        // `description.menu_category`, then each earlier section's component. The game
        // copies "without checking whether one exists already", so what is replaced
        // by another value is noted where the file has it.
        let mut menu_at = JsonPath::new();
        for key in ["minecraft:block", "description", "menu_category"] {
            menu_at.push(key);
        }
        let mut held: Option<JsonPath> = block
            .description
            .get("menu_category")
            .is_some_and(|had| !had.is_null())
            .then(|| menu_at.clone());
        for (section, category) in categories {
            // Where the file has the category, for the step that judges it.
            block.category_from = Some(section);
            let mut menu = facet_value::VObject::new();
            if let Some(object) = category.as_object() {
                for key in ["category", "group"] {
                    if let Some(value) = object.get(key) {
                        menu.insert(key, value.clone());
                    }
                }
                // The game copies `{category, group}` alone: any other member of
                // the component is not read, and noted where the file has it.
                for (key, _) in object {
                    if !["category", "group"].contains(&key.as_str()) {
                        let mut at =
                            super::legacy::component_path(section, "minecraft:creative_category");
                        at.push(key.as_str());
                        d.push(
                            Diagnostic::info(bedrock_addon_core::diagnostic::Code::from(
                                &core_codes::UNRECOGNIZED_FIELD,
                            ))
                            .at(at.clone())
                            .phase(Phase::Upgrade),
                        );
                    }
                }
            }
            let menu = Value::from(menu);
            let from = super::legacy::component_path(section, "minecraft:creative_category");
            if let Some(at) = held.replace(from.clone())
                && block
                    .description
                    .get("menu_category")
                    .is_some_and(|had| *had != menu)
            {
                super::legacy::replaced(&at, d);
            }
            block.description.insert("menu_category".to_owned(), menu);
            d.moved(from, menu_at.clone());
        }
        block.wrap_component(
            "minecraft:explosion_resistance",
            "minecraft:destructible_by_explosion",
            "explosion_resistance",
            d,
        );
        block.wrap_component(
            "minecraft:destroy_time",
            "minecraft:destructible_by_mining",
            "seconds_to_destroy",
            d,
        );
        // `friction`: `1 - n`, clamped to 0.0..=0.9 (the clamp is the crate's own; the game leaves the value unclamped).
        for (section, map) in block.sections_mut() {
            if let Some(value) = map.get_mut("minecraft:friction")
                && let Some(n) = number(value)
            {
                *value = real(clamp(1.0 - n, 0.0, 0.9, section, "minecraft:friction", d));
            }
        }
        block.rename_component(
            "minecraft:aim_collision",
            "minecraft:selection_box",
            |v| v,
            d,
        );
        Legacy(block)
    }
}
