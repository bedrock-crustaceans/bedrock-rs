//! `bp.loot_tables()`: `loot_tables/**/*.json` of every behaviour pack, keyed by path, the top
//! pack's copy winning.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::loot::RandomValueBounds;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, PackStack};
use bedrock_addon::storage::PackPath;

use super::pack_common::bp;

const HIGH: &[u8] = br#"{"pools": [{"rolls": 3, "entries": [{"type": "empty"}]}]}"#;
const LOW: &[u8] = br#"{"pools": [{"rolls": 1, "entries": [{"type": "empty"}]}]}"#;
const BROKEN: &[u8] = br#"[{"pools": []}]"#;

fn key(text: &str) -> PackPath {
    PackPath::new(text).unwrap()
}

#[test]
fn the_top_packs_copy_of_a_path_wins() {
    let top = bp(1).file("loot_tables/chests/x.json", HIGH).open();
    let bottom = bp(2)
        .file("loot_tables/chests/x.json", LOW)
        .file("loot_tables/chests/y.json", LOW)
        .open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    let tables = stack.loot_tables();
    let x = tables.get(&key("loot_tables/chests/x.json")).unwrap();
    assert_eq!(
        x.pools.as_deref().unwrap()[0].rolls,
        Some(RandomValueBounds::Exact(3.0))
    );
    assert_eq!(x.origin().pack.name(), "bp 1");
    assert!(tables.get(&key("loot_tables/chests/y.json")).is_some());
    // a path no pack has is no table and no finding.
    assert!(tables.get(&key("loot_tables/chests/none.json")).is_none());
    assert_eq!(tables.iter().count(), 2);
    assert!(tables.diagnostics().next().is_none());
}

#[test]
fn a_failing_top_copy_leaves_the_path_without_a_table() {
    // Vanilla loads the one file the stack resolves the path to and caches nothing when it is
    // invalid: the lower copy does not stand in.
    let top = bp(1).file("loot_tables/chests/x.json", BROKEN).open();
    let bottom = bp(2).file("loot_tables/chests/x.json", LOW).open();
    let stack = PackStack::<Behavior>::of([top, bottom], LoadOptions::default()).unwrap();
    let tables = stack.loot_tables();
    assert!(tables.get(&key("loot_tables/chests/x.json")).is_none());
    assert_eq!(tables.rejected().len(), 1);
    let errors: Vec<_> = tables
        .diagnostics()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code.name(), "MalformedInput");
    assert_eq!(
        errors[0].location.path.as_ref().unwrap().as_str(),
        "loot_tables/chests/x.json"
    );
    assert_eq!(
        errors[0].code.to_string(),
        "Loot table for loot_tables/chests/x.json is invalid."
    );
}

#[test]
fn only_json_files_below_loot_tables_are_read() {
    let pack = bp(1)
        .file("loot_tables/a.json", LOW)
        .file("loot_tables/deep/er/b.json", LOW)
        .file("loot_tables/notes.txt", b"not a table")
        .file("loot_tables/upper.JSON", LOW)
        .file("loot_tables/json", LOW)
        .file("loot_tables/old.json.bak", LOW)
        .file("loot_tables_extra/d.json", LOW)
        .file("trading/c.json", LOW)
        .open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    let paths: Vec<String> = stack
        .loot_tables()
        .iter()
        .map(|(path, _)| path.as_str().to_owned())
        .collect();
    assert_eq!(paths, ["loot_tables/a.json", "loot_tables/deep/er/b.json"]);
}

#[test]
fn a_findings_location_names_the_pack_and_the_file() {
    let pack = bp(1)
        .file(
            "loot_tables/w.json",
            br#"{"pools": [{"rolls": 1, "entries": [{"type": "nothing"}]}]}"#,
        )
        .open();
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    let finding = stack.loot_tables().diagnostics().next().unwrap();
    assert_eq!(finding.location.pack.as_ref().unwrap().name(), "bp 1");
    assert_eq!(
        finding.location.path.as_ref().unwrap().as_str(),
        "loot_tables/w.json"
    );
    assert_eq!(finding.location.json_path.to_string(), "pools/0/entries/0");
}
