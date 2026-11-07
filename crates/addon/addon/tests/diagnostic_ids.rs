//! The registry of diagnostic definition ids: every table of definitions the toolkit raises
//! from, and the rule that no id appears twice across them. Each family that gains a `codes.rs`
//! adds its `ALL` table below, behind its feature; the migration is not done for a family until
//! its line is here.

use bedrock_addon::diagnostic::CodeMeta;
use std::collections::BTreeMap;

/// Every definition table, named for the crate that owns it.
fn tables() -> Vec<(&'static str, &'static [&'static CodeMeta])> {
    let tables: Vec<(&'static str, &'static [&'static CodeMeta])> = vec![
        ("core", bedrock_addon::diagnostic::codes::ALL),
        ("pack_settings", bedrock_addon::pack::settings::codes::ALL),
        // wave 2: one line per family, in the group's commit for that family, e.g.
        // #[cfg(feature = "fog")] ("fog", bedrock_addon::families::fog::codes::ALL),
    ];
    tables
}

#[test]
fn every_table_lists_at_least_one_definition() {
    for (owner, table) in tables() {
        assert!(!table.is_empty(), "{owner} lists no definitions");
    }
}

#[test]
fn ids_are_unique_across_every_table() {
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    let mut clashes = Vec::new();
    for (owner, table) in tables() {
        for meta in table {
            if let Some(first) = seen.insert(meta.id(), owner) {
                clashes.push(format!("{} ({first}, {owner})", meta.id()));
            }
        }
    }
    assert!(clashes.is_empty(), "ids defined twice: {clashes:?}");
}

#[test]
fn every_id_names_its_owner_first() {
    let mut wrong = Vec::new();
    for (owner, table) in tables() {
        for meta in table {
            let prefix = meta.id().split('/').next().unwrap_or("");
            if prefix != owner {
                wrong.push(format!("{} in the {owner} table", meta.id()));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "ids outside their owner's prefix: {wrong:?}"
    );
}
