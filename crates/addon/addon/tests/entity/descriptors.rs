//! The descriptor table of the entity family against the link pass's reference tables.

use bedrock_addon::families::entity::DESCRIPTORS;

/// The quoted strings of `text`, in order.
fn quoted(text: &str) -> Vec<&str> {
    text.split('"').skip(1).step_by(2).collect()
}

/// The rows of the table `name` of the link pass, as its source writes them.
fn link_rows(name: &str) -> Vec<(&'static str, &'static str)> {
    const TABLE: &str = include_str!("../../src/pack/link/entity/table.rs");
    let head = format!("const {name}: &[Row] = &[");
    let start = TABLE
        .find(&head)
        .unwrap_or_else(|| panic!("no table {name}"))
        + head.len();
    let rows = &TABLE[start..start + TABLE[start..].find("];").expect("the table ends")];
    let strings = quoted(rows);
    assert_eq!(strings.len() % 2, 0, "{name}: a row is a key and a pattern");
    strings.chunks(2).map(|row| (row[0], row[1])).collect()
}

/// The keys of the components the family keeps as JSON, from its component list.
fn dynamic_components() -> Vec<&'static str> {
    const COMPONENTS: &str = include_str!("../../../families/entity/src/component/mod.rs");
    COMPONENTS
        .lines()
        .filter(|line| line.contains("#[component(name = ") && line.contains("parse = keep_json"))
        .map(|line| quoted(line)[0])
        .collect()
}

/// The table of this file is the link pass's block and item members of the components
/// kept as JSON, no row more and none less: a member one list gains, the other must.
#[test]
fn the_descriptor_members_are_the_link_pass_s_block_and_item_members_of_dynamic_components() {
    let dynamic = dynamic_components();
    assert!(dynamic.len() > 100, "{}", dynamic.len());
    let mut of_the_link_pass: Vec<(&str, &str)> = ["BLOCKS", "BLOCKS_AS_ITEMS", "ITEMS"]
        .into_iter()
        .flat_map(link_rows)
        .filter(|(component, _)| dynamic.contains(component))
        .collect();
    of_the_link_pass.sort_unstable();
    of_the_link_pass.dedup();
    assert_eq!(DESCRIPTORS, of_the_link_pass);
    // Sorted, and no component in it that is typed.
    assert!(DESCRIPTORS.is_sorted());
    assert_eq!(DESCRIPTORS.len(), 42);
}
