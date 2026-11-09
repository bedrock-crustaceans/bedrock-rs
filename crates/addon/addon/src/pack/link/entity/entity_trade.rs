//! The trade tables an entity's components name, by their path in the pack as written. A
//! path nothing defines gives the game a null table without a line, and the trade components
//! only assert: the game writes no content-log line for this, so the crate reports it as an
//! information.
use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::id::Id;

use super::table::TRADE_TABLES;
use super::walk::Entities;
use crate::pack::link::prelude::*;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &Entities<'_>) {
    // The trade tables the load could not read, gathered on the first miss.
    let mut files = None;
    entities.each(TRADE_TABLES, &mut |component, at, value| {
        let Some(path) = super::name(value) else {
            return;
        };
        if let Target::Missing(hint) = cx.trade_table(&Id::parse(path)) {
            // A file the load could not read is not judged.
            if files
                .get_or_insert_with(|| super::unread(cx, "trading", super::Fold::Keep))
                .contains(path)
            {
                return;
            }
            let at = At::new(component.origin, at, Severity::Info);
            out.push(unresolved(at, kinds::TRADE_TABLE, path, hint));
        }
    });
}
