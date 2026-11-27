//! The loot table a block drops from.

use bedrock_addon_core::diagnostic::Severity;

use crate::pack::link::prelude::*;
use super::block::{placed, sections};
use crate::families::block::component::Loot;

/// `minecraft:loot` is the path of a loot table. The game's block loader checks its length
/// alone; the linked-asset check logs a file no pack holds as
/// `<breadcrumb>:  Invalid asset path <path>` and drops nothing: an error with that line.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for (_, document) in cx.behavior().blocks() {
        for (section, components) in sections(document) {
            let Some(Loot(table)) = components.get::<Loot>() else {
                continue;
            };
            if cx.loot_table(table).is_missing() {
                let path = placed(document, &at![&section; "minecraft:loot"]);
                // The linked-asset check's grading; the severity here is read by nothing else.
                let place = At::new(document.origin(), path, Severity::Error).validator(
                    Validator::InvalidPath,
                    1,
                    table,
                );
                out.push(unresolved(place, kinds::LOOT_TABLE, table, None));
            }
        }
    }
}
