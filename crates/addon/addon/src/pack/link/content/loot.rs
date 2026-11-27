//! The loot tables a loot table names, and the walk over a table's entries,
//! functions and conditions that every loot rule of this group uses.

use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::document::Document;

use crate::pack::link::prelude::*;
use crate::families::loot::{
    Entry, EntryKind, FunctionKind, LootCondition, LootFunction, LootTable, Pool,
};

/// One place of a loot table a rule can read a reference from.
#[cfg_attr(
    not(any(feature = "item", feature = "entity")),
    allow(dead_code, reason = "conditions are read by the rules of `item` and `entity`")
)]
#[derive(Debug)]
pub(super) enum Site<'t> {
    /// An entry of a pool.
    Entry(&'t Entry),
    /// A function of an item entry.
    Function(&'t LootFunction),
    /// A condition of a pool, of an entry or of a function.
    Condition(&'t LootCondition),
}

/// Calls `visit` with every entry, function and condition of `table` and its path in the
/// file, in the order of the file; the tables nested in entries (`pools` of an entry) too.
///
/// The paths are those of a table whose `pools`, `entries`, `functions` and `conditions` are
/// arrays, as every table the game ships writes them.
pub(super) fn walk<'t>(
    document: &'t Document<LootTable>,
    visit: &mut dyn FnMut(&JsonPath, Site<'t>),
) {
    let table = document.value();
    // An element behind one the reader left out is at the index the file has it at: each
    // path is built as the table holds it, and asked of the load once, whole.
    let found = document.diagnostics();
    let mut visit = |held: &JsonPath, site: Site<'t>| visit(&found.place_of(held), site);
    // (the pools, the held path of their array): a nested table is walked after the one
    // that holds it, so that no table nests the walk.
    let mut waiting: Vec<(&'t [Pool], JsonPath)> = Vec::new();
    if let Some(pools) = &table.pools {
        waiting.push((pools, at!["pools"]));
    }
    while let Some((pools, base)) = waiting.pop() {
        for (pool_index, pool) in pools.iter().enumerate() {
            let pool_path = at![&base; pool_index];
            conditions(pool.conditions.as_deref(), &pool_path, &mut visit);
            for (entry_index, entry) in pool.entries().iter().enumerate() {
                let entry_path = at![&pool_path; "entries", entry_index];
                visit(&entry_path, Site::Entry(entry));
                conditions(entry.conditions.as_deref(), &entry_path, &mut visit);
                if let EntryKind::Item {
                    functions: Some(functions),
                    ..
                } = &entry.kind
                {
                    for (index, function) in functions.iter().enumerate() {
                        let path = at![&entry_path; "functions", index];
                        visit(&path, Site::Function(function));
                        conditions(function.conditions.as_deref(), &path, &mut visit);
                    }
                }
                if let Some(nested) = &entry.pools {
                    waiting.push((nested, at![&entry_path; "pools"]));
                }
            }
        }
    }
}

/// The conditions of the object at `owner` (a held path).
fn conditions<'t>(
    conditions: Option<&'t [LootCondition]>,
    owner: &JsonPath,
    visit: &mut dyn FnMut(&JsonPath, Site<'t>),
) {
    for (index, condition) in conditions.unwrap_or_default().iter().enumerate() {
        visit(&at![owner; "conditions", index], Site::Condition(condition));
    }
}

/// A `loot_table` entry's `name` and a `fill_container` function's `loot_table` are
/// the path of a loot table: everything up to the last `:` is no part of it, and it is
/// looked up lower-cased. The game resolves it when the table is rolled and a missing file
/// gives nothing, in silence: a lint.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    if !cx.lints() {
        return;
    }
    for (_, document) in cx.behavior().loot_tables() {
        walk(document, &mut |path, site| {
            let (key, table) = match site {
                Site::Entry(Entry {
                    kind: EntryKind::LootTable { name: Some(name) },
                    ..
                }) => ("name", name),
                Site::Function(LootFunction {
                    kind:
                        FunctionKind::FillContainer {
                            loot_table: Some(table),
                        },
                    ..
                }) => ("loot_table", table),
                _ => return,
            };
            if !table.as_str().is_empty() && cx.loot_table(table.as_str()).is_missing() {
                let place = At::new(document.origin(), at![path; key], Severity::Info);
                out.push(unresolved(place, kinds::LOOT_TABLE, table.as_str(), None));
            }
        });
    }
}
