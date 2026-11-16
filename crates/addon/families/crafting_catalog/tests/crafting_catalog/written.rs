//! The written text of the catalogue, pinned in `crafting_catalog/written.pins`: what
//! `to_json` writes for every fixture that loads, for probes of each member and for a value
//! built by hand (`bedrock_addon_test_support::written_pins`).

use bedrock_addon_core::document::Content;
use bedrock_addon_family_crafting_catalog::CraftingCatalog;
use bedrock_addon_test_support::written_pins::{
    assert_as_pinned, fixtures_below, parse, render, written,
};

use super::common::{PATH, cat01};

/// Documents that reach each member: both item forms, an aux value, an icon, a loose group,
/// unknown members at each level, an empty list.
const PROBES: &[&str] = &[
    r#"{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {"categories": [{"category_name": "items", "groups": [{"items": ["a:b", {"name": "a:c:2"}]}]}]}}"#,
    r#"{"format_version": "1.26.50", "minecraft:crafting_items_catalog": {"categories": [{"category_name": "nature", "groups": [{"group_identifier": {"name": "a:g", "icon": {"name": "a:i"}, "odd": 1}, "items": [], "colour": "red"}], "x": [null]}], "y": {}}, "z": true}"#,
    r#"{"format_version": "1.26.50", "minecraft:crafting_items_catalog": {"categories": []}}"#,
];

#[test]
fn the_catalogue_is_written_as_pinned() {
    let mut cases: Vec<(String, String)> = Vec::new();
    let addon = concat!(env!("CARGO_MANIFEST_DIR"), "/../../addon");
    for (name, text) in fixtures_below(addon, "item/catalog", ".json") {
        if parse::<CraftingCatalog>(PATH, &text).is_ok() {
            cases.push((name, written::<CraftingCatalog>(PATH, &text)));
        }
    }
    for probe in PROBES {
        cases.push(((*probe).to_owned(), written::<CraftingCatalog>(PATH, probe)));
    }
    cases.push((
        "cat01(), built by hand".to_owned(),
        cat01().to_json().unwrap(),
    ));
    assert_as_pinned(
        env!("CARGO_MANIFEST_DIR"),
        "crafting_catalog/written.pins",
        &render(&cases),
    );
}
