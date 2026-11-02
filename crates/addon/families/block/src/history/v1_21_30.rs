//! Format 1.21.30: `destructible_by_mining.item_specific_speeds`.

use super::legacy::{Legacy, string};
use super::ops::dropped_field;
use crate::parse::child;
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

/// A block document of `[1.21.30, 1.21.60)`.
pub type Block = Legacy<1, 21, 30>;

/// `menu_category.group` is namespaced: a string gets
/// `minecraft:` in front unconditionally; any other value is removed.
impl Upgrade for Block {
    type Next = super::v1_21_60::Block;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block: Self::Next = self.retag();
        let Some(menu) = block.description.get_mut("menu_category") else {
            return block;
        };
        let mut dropped = false;
        let mut namespaced_group = false;
        super::ops::edit_object(menu, |m| {
            let Some(group) = m.get("group") else { return };
            if let Some(name) = string(group) {
                let namespaced = Value::from(format!("minecraft:{name}"));
                m.insert("group".to_owned(), namespaced);
                namespaced_group = true;
            } else {
                m.shift_remove("group");
                dropped = true;
            }
        });
        let at = child(
            &JsonPath::new(),
            &["minecraft:block", "description", "menu_category", "group"],
        );
        if dropped {
            dropped_field(&at, None, cx.diagnostics);
        }
        // The group is written under vanilla's namespace: told as the item's same step
        // tells it (`item::history`, the 1.21.60 step): an information, since the game prefixes the
        // namespace without writing a log line.
        if namespaced_group {
            cx.diagnostics.push(super::legacy::upgrade_finding(
                bedrock_addon_core::diagnostic::DiagnosticCode::DeprecatedMember { text: None },
                bedrock_addon_core::diagnostic::Severity::Info,
                &at,
            ));
        }
        block
    }
}
