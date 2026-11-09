//! The localisation keys a block names (`display_name`, `crafting_table.table_name`: the game
//! reads both as localisation ids).

use bedrock_addon_core::diagnostic::Severity;

use crate::pack::link::prelude::*;
use super::block::{placed, sections};
use crate::families::block::component::{CraftingTable, DisplayName};

/// `minecraft:display_name` and `minecraft:crafting_table.table_name` are
/// localisation keys. Vanilla shows a text no `.lang` file defines as written: a lint.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    if !cx.lints() {
        return;
    }
    for (_, document) in cx.behavior().blocks() {
        for (section, components) in sections(document) {
            if let Some(DisplayName(name)) = components.get::<DisplayName>()
                && cx.lang_key(name).is_missing()
            {
                let path = placed(document, &at![&section; "minecraft:display_name"]);
                let place = At::new(document.origin(), path, Severity::Info);
                out.push(unresolved_lang_key(place, name));
            }
            if let Some(table) = components.get::<CraftingTable>()
                && let Some(name) = &table.table_name
                && !name.is_empty()
                && cx.lang_key(name).is_missing()
            {
                let path = placed(document, &at![&section; "minecraft:crafting_table", "table_name"]);
                let place = At::new(document.origin(), path, Severity::Info);
                out.push(unresolved_lang_key(place, name));
            }
        }
    }
}
