//! The loot tables an entity's components name. A lint: the game keeps the path as
//! written and looks the file up when it rolls the table, saying nothing of a missing one.
use bedrock_addon_core::diagnostic::Severity;

use super::super::{At, Findings, LinkCx, kinds, unresolved};
use super::table::LOOT_TABLES;
use super::walk::Entities;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &Entities<'_>) {
    if !cx.lints() {
        return;
    }
    // The loot tables the load could not read, in lower case, gathered on the first miss.
    let mut files = None;
    entities.each(LOOT_TABLES, &mut |component, at, value| {
        let Some(path) = super::name(value) else {
            return;
        };
        // The path's normalisation is the loot lookup's.
        if cx.loot_table(path).is_missing() {
            // A file the load could not read is not judged.
            let files =
                files.get_or_insert_with(|| super::unread(cx, "loot_tables", super::Fold::Lower));
            let file = path
                .rsplit_once(':')
                .map_or(path, |(_, file)| file)
                .to_lowercase();
            if files.contains(&file) || files.contains(&format!("{file}.json")) {
                return;
            }
            let at = At::new(component.origin, at, Severity::Info);
            out.push(unresolved(at, kinds::LOOT_TABLE, path, None));
        }
    });
}
