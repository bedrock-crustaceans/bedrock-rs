//! Format 1.21.90: keyed custom components; `ordered_v1_custom_components`.

use super::legacy::BlockPayload;
use super::ops::edit_object;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

/// A block document of `[1.21.90, 1.26.0)`.
pub type Block = Legacy<BlockPayload, 1, 21, 90>;

/// `liquid_detection.detection_rules[0].use_liquid_clipping`
/// is set to `true` when absent (only the first rule).
impl LegacyStep<1, 21, 90> for BlockPayload {
    type Next = super::v1_26_0::Block;

    fn step(this: Block, _: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block = this.0;
        for (_, map) in block.sections_mut() {
            let Some(detection) = map.get_mut("minecraft:liquid_detection") else {
                continue;
            };
            edit_object(detection, |d| {
                let Some(rules) = d.get_mut("detection_rules").and_then(Value::as_array_mut) else {
                    return;
                };
                if let Some(first) = rules.get_mut(0) {
                    edit_object(first, |rule| {
                        // A `null` member is an absent member.
                        if rule.get("use_liquid_clipping").is_none_or(Value::is_null) {
                            rule.insert("use_liquid_clipping".to_owned(), Value::TRUE);
                        }
                    });
                }
            });
        }
        Legacy(block)
    }
}
