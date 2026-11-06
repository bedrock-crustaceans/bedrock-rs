//! Format 1.16.0 (the floor): the description still has `is_experimental` and
//! `register_to_creative_menu`; components are 1.16.100's.

use super::legacy::BlockPayload;
use super::ops::dropped_field;
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

/// A block document of `[1.16.0, 1.16.100)`.
pub type Block = Legacy<BlockPayload, 1, 16, 0>;

/// the two description flags 1.16.100 removed are dropped
/// (vanilla reads neither); components pass through.
impl LegacyStep<1, 16, 0> for BlockPayload {
    type Next = super::v1_16_100::Block;

    fn step(this: Block, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block = this.0;
        for flag in ["is_experimental", "register_to_creative_menu"] {
            if block.description.shift_remove(flag).is_some() {
                let at = JsonPath::new().with(&["minecraft:block", "description", flag]);
                dropped_field(&at, None, cx.diagnostics);
            }
        }
        Legacy(block)
    }
}
