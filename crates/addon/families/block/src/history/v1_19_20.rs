//! Format 1.19.20: `selection_box`, the `destructible_by_*` components, the new `friction`
//! and `description.menu_category`.

use super::legacy::{Legacy, string};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

/// A block document of `[1.19.20, 1.19.30)`.
pub type Block = Legacy<1, 19, 20>;

/// a `display_name` string `s` becomes the key `tile.s.name`.
impl Upgrade for Block {
    type Next = super::v1_19_30::Block;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block: Self::Next = self.retag();
        for (_, map) in block.sections_mut() {
            if let Some(value) = map.get_mut("minecraft:display_name")
                && let Some(name) = string(value)
            {
                *value = Value::from(format!("tile.{name}.name"));
            }
        }
        block
    }
}
