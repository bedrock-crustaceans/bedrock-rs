//! Format 1.19.80: `transformation` replaces `rotation`; `geometry` takes the object form with
//! `bone_visibility`.

use super::legacy::{BlockPayload, component_path, text};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Phase};
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

/// A block document of `[1.19.80, 1.20.10)`.
pub type Block = Legacy<BlockPayload, 1, 19, 80>;

/// `geometry.bone_visibility` entries that are a string, an
/// object or an array are removed ("Only boolean values are supported for bone visibilities in
/// format versions before 1.20.10."); booleans, numbers and `null` are kept.
impl LegacyStep<1, 19, 80> for BlockPayload {
    type Next = super::v1_20_10::Block;

    fn step(this: Block, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block = this.0;
        let mut removed = Vec::new();
        for (section, map) in block.sections_mut() {
            let Some(geometry) = map.get_mut("minecraft:geometry") else {
                continue;
            };
            super::ops::edit_object(geometry, |g| {
                let Some(bones) = g.get_mut("bone_visibility") else {
                    return;
                };
                super::ops::edit_object(bones, |bones| {
                    bones.retain(|bone, value| {
                        let keep = !(value.is_string() || value.is_object() || value.is_array());
                        if !keep {
                            removed.push((section, bone.clone()));
                        }
                        keep
                    });
                });
            });
        }
        for (section, bone) in removed {
            let mut at = component_path(section, "minecraft:geometry");
            at.push("bone_visibility");
            at.push(&bone);
            // The game logs this once per bone at the content log's error level and goes on:
            // a kept error.
            cx.diagnostics.push_kept_error(Diagnostic::error(DiagnosticCode::DroppedField {
                    text: text(
                        "bone_visibility | {}: Only boolean values are supported for bone visibilities in format versions before {}.",
                        &[&bone, "1.20.10"],
                    ),
                }).at(at.clone()).phase(Phase::Upgrade));
        }
        Legacy(block)
    }
}
