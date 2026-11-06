//! Format 1.19.40: `light_dampening`, `menu_category.is_hidden_in_commands`.

use super::legacy::{BlockPayload, component_path, fixed};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Phase};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

/// A block document of `[1.19.40, 1.19.80)`.
pub type Block = Legacy<BlockPayload, 1, 19, 40>;

/// `part_visibility` is removed with vanilla's two messages;
/// `rotation [x, y, z]` becomes `transformation {rotation}`.
impl LegacyStep<1, 19, 40> for BlockPayload {
    type Next = super::v1_19_80::Block;

    fn step(this: Block, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block = this.0;
        let mut removed = Vec::new();
        for (section, map) in block.sections_mut() {
            if map.shift_remove("minecraft:part_visibility").is_some() {
                removed.push(section);
            }
        }
        for section in removed {
            let at = component_path(section, "minecraft:part_visibility");
            for message in [
                "The part_visibility component has been deprecated, use the bone_visibility field in the geometry component instead. See Block Documentation for more information.",
                "The part_visibility component is no longer supported. Please use the bone_visibility field in the geometry component instead.",
            ] {
                cx.diagnostics.push(
                    // The game writes neither line when it loads a block, so the finding is an information.
                    Diagnostic::info(DiagnosticCode::DroppedComponent {
                        text: fixed(message),
                    })
                    .at(at.clone())
                    .phase(Phase::Upgrade),
                );
            }
        }
        // A `rotation` that is `null` gives a transformation without one; the crate tells it
        // at the component.
        for (section, map) in block.sections_mut() {
            if map.get("minecraft:rotation").is_some_and(Value::is_null) {
                super::legacy::misread(
                    &component_path(section, "minecraft:rotation"),
                    cx.diagnostics,
                );
            }
        }
        block.wrap_component(
            "minecraft:rotation",
            "minecraft:transformation",
            "rotation",
            cx.diagnostics,
        );
        Legacy(block)
    }
}
