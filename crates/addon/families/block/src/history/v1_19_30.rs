//! Format 1.19.30: `display_name` is a literal or a localisation key.

use super::legacy::{Legacy, string, text, upgrade_finding};
use crate::parse::child;
use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

/// A block document of `[1.19.30, 1.19.40)`.
pub type Block = Legacy<1, 19, 30>;

/// The categories vanilla keeps as written.
const VALID: [&str; 4] = ["construction", "nature", "equipment", "items"];

/// `block_light_filter` → `light_dampening`; the menu category
/// rules (vanilla's 1.19.40 fixer).
impl Upgrade for Block {
    type Next = super::v1_19_40::Block;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block: Self::Next = self.retag();
        block.rename_component(
            "minecraft:block_light_filter",
            "minecraft:light_dampening",
            |v| v,
            cx.diagnostics,
        );
        let category_from = block.category_from;
        let Some(menu) = block.description.get_mut("menu_category") else {
            return block;
        };
        let Some(mut object) = super::legacy::object(menu) else {
            return block;
        };
        // The category of a file below 1.19.20 was a `minecraft:creative_category`
        // component of the base or of a permutation: the finding is at the member the file
        // has.
        let at = match category_from {
            Some(section) => child(
                &super::legacy::component_path(section, "minecraft:creative_category"),
                &["category"],
            ),
            None => child(
                &JsonPath::new(),
                &[
                    "minecraft:block",
                    "description",
                    "menu_category",
                    "category",
                ],
            ),
        };
        let (category, hidden) = match object.get("category").and_then(string) {
            None => (None, false),
            Some(written) => {
                let lower = written.to_ascii_lowercase();
                match lower.as_str() {
                    "" => {
                        // The game logs this at the content log's error level and the value
                        // becomes `none`: a kept error.
                        cx.diagnostics.push_kept_error(upgrade_finding(
                            DiagnosticCode::InvalidValue {
                                text: text(
                                    "Block Menu Category should not have an empty value",
                                    &[],
                                ),
                            },
                            Severity::Error,
                            &at,
                        ));
                        (Some("none".to_owned()), true)
                    }
                    "none" => (Some("none".to_owned()), true),
                    "all" | "commands" => {
                        // The game's table: `commands` → `none`, shown to commands; `all` →
                        // `none`, hidden; each with no log. The game's own conversion of a
                        // category the format does not have any more: a note where the file has it.
                        cx.diagnostics.push(upgrade_finding(
                            DiagnosticCode::DeprecatedMember { text: None },
                            Severity::Info,
                            &at,
                        ));
                        (Some("none".to_owned()), lower == "all")
                    }
                    valid if VALID.contains(&valid) => (Some(written.to_owned()), false),
                    _ => {
                        // The game logs this at the content log's error level and the value
                        // becomes `none`: a kept error.
                        cx.diagnostics.push_kept_error(upgrade_finding(
                            DiagnosticCode::InvalidValue {
                                text: text(
                                    "Block Menu Category has an invalid value of {}",
                                    &[written],
                                ),
                            },
                            Severity::Error,
                            &at,
                        ));
                        (Some("none".to_owned()), true)
                    }
                }
            }
        };
        if let Some(category) = category {
            object.insert("category".to_owned(), Value::from(category));
        }
        // "`is_hidden_in_commands` is added in every row": the file's own member,
        // another value, is written over as the game's fixer writes over it, and noted.
        let hidden = Value::from(hidden);
        if object
            .get("is_hidden_in_commands")
            .is_some_and(|had| !had.is_null() && *had != hidden)
        {
            super::legacy::replaced(
                &child(
                    &JsonPath::new(),
                    &[
                        "minecraft:block",
                        "description",
                        "menu_category",
                        "is_hidden_in_commands",
                    ],
                ),
                cx.diagnostics,
            );
        }
        object.insert("is_hidden_in_commands".to_owned(), hidden);
        *menu = super::to_object(&object);
        block
    }
}
