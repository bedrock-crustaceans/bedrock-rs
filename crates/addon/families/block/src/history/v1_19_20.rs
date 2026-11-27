//! Format 1.19.20: `selection_box`, the `destructible_by_*` components, the new `friction`
//! and `description.menu_category`.

use super::legacy::{BlockPayload, string};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

/// A block document of `[1.19.20, 1.19.30)`.
pub type Block = Legacy<BlockPayload, 1, 19, 20>;

/// a `display_name` string `s` becomes the key `tile.s.name`.
impl LegacyStep<1, 19, 20> for BlockPayload {
    type Next = super::v1_19_30::Block;

    fn step(this: Block, _: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block = this.0;
        for (_, map) in block.sections_mut() {
            if let Some(value) = map.get_mut("minecraft:display_name")
                && let Some(name) = string(value)
            {
                *value = Value::from(format!("tile.{name}.name"));
            }
        }
        Legacy(block)
    }
}
