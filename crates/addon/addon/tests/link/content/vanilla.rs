//! The rules of this group over the game's own files: no loot table, recipe, trade table or
//! item of the dedicated server and of bedrock-samples names an item or an item tag the pass
//! does not know. Skipped without `BEDROCK_ADDON_CORPUS`; nothing of the corpus is committed.

use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::Pack;

use super::super::common::{behavior_pack_of, stacks_of};
use super::said_of;

/// The folders of a behaviour pack the rules of this group read item names from.
const FOLDERS: [&str; 4] = ["loot_tables", "recipes", "trading", "items"];

/// Every file below `dir`, with its path from `root`.
fn files(root: &Path, dir: &Path, out: &mut Vec<(&'static str, &'static [u8])>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            files(root, &path, out);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let relative = path.strip_prefix(root).unwrap().to_string_lossy();
            let relative: &'static str = Box::leak(relative.replace('\\', "/").into_boxed_str());
            let bytes: &'static [u8] = Box::leak(std::fs::read(&path).unwrap().into_boxed_slice());
            out.push((relative, bytes));
        }
    }
}

/// The pack `n` with the files of `root` this group reads.
fn pack_of(n: u128, root: &Path) -> Option<Pack> {
    let mut all = Vec::new();
    for folder in FOLDERS {
        files(root, &root.join(folder), &mut all);
    }
    (!all.is_empty()).then(|| behavior_pack_of(n, &all))
}

/// 20 L1, T1, RC1, RC2, I5 over vanilla: with the frame's legacy item names and this group's
/// lists, no item and no item tag of the game's own files is missing, at any severity above
/// information.
#[test]
fn the_vanilla_tables_recipes_trades_and_items_name_only_what_the_game_has() {
    let Some(corpus) = std::env::var_os("BEDROCK_ADDON_CORPUS") else {
        eprintln!("skipped: BEDROCK_ADDON_CORPUS is not set");
        return;
    };
    let corpus = PathBuf::from(corpus);
    let mut roots = vec![corpus.join("bedrock-samples").join("behavior_pack")];
    let mut server: Vec<PathBuf> = std::fs::read_dir(corpus.join("bds-1.26.36.1/behavior_packs"))
        .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default();
    server.sort();
    roots.extend(server);
    // The items the game's packs define themselves: a pack of this test is a world's pack,
    // which may not define a `minecraft:` item, so those definitions do not load here.
    let mut defined = String::new();
    for root in &roots {
        let mut own = Vec::new();
        files(root, &root.join("items"), &mut own);
        for (_, bytes) in own {
            defined.push_str(&String::from_utf8_lossy(bytes));
        }
    }
    let mut tables = 0;
    let mut all_missing = 0;
    for (index, root) in roots.iter().enumerate() {
        // Each pack on its own: every copy of every file is read, none shadowed.
        let Some(pack) = pack_of(index as u128 + 1, root) else {
            continue;
        };
        let set = stacks_of(vec![pack], Vec::new(), LoadOptions::default());
        tables += set.behavior.loot_tables().iter().count();
        let found = set.link();
        let missing: Vec<_> = found
            .iter()
            .filter(|finding| finding.severity > Severity::Info)
            .map(said_of)
            .filter(|said| said.4 == "item" || said.4 == "item tag")
            .filter(|said| !defined.contains(&format!("\"{}\"", said.5)))
            .collect();
        for said in &missing {
            eprintln!("MISSING {} {}", said.4, said.5);
        }
        all_missing += missing.len();
    }
    assert_eq!(all_missing, 0, "see the MISSING lines");
    assert!(
        tables > 600,
        "the loot tables of the corpus were read: {tables}"
    );
}
