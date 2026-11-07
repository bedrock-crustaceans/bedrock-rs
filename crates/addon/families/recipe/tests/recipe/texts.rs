//! Every line the recipe module raises is a definition of its `codes.rs`, whose arity
//! `CodeDef` checks when it compiles: no source file builds a legacy `VanillaText`, and the
//! table lists every definition the module declares.

use std::path::PathBuf;

#[test]
fn no_line_is_built_outside_the_definitions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut scanned = 0;
    for entry in std::fs::read_dir(&root).unwrap() {
        let path = entry.unwrap().path();
        let source = std::fs::read_to_string(&path).unwrap();
        scanned += 1;
        assert!(
            !source.contains("VanillaText"),
            "{}: raise a definition of codes.rs",
            path.display()
        );
    }
    assert!(scanned >= 5, "the scan finds the sources ({scanned})");
}

#[test]
fn the_table_lists_every_definition() {
    let source = include_str!("../../src/codes.rs");
    let declared = source.matches("pub static ").count() - 1; // ALL itself
    assert_eq!(bedrock_addon_family_recipe::codes::ALL.len(), declared);
}
