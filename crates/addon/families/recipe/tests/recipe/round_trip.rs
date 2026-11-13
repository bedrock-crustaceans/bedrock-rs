//! Write-back at the Value level: every fixture that loads is parsed, written,
//! re-read strictly and rewritten identically; members are written only where the author wrote
//! them.

use bedrock_addon_core::document::Content;
use bedrock_addon_core::id::RecipeId;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_recipe::{
    Description, Ingredient, IngredientObject, Output, Recipe, Shaped, Tags, Unlock,
};

use super::common::*;

/// Parses `bytes`, writes, parses the text again and writes again: the values and the texts
/// must be equal.
fn round_trip(name: &str, bytes: &[u8]) -> String {
    let cx = cx_base_game();
    let Ok(first) = parse_bytes(name, bytes, &cx) else {
        return String::new();
    };
    let written = first.to_json().unwrap();
    let second = parse_bytes(name, written.as_bytes(), &cx)
        .unwrap_or_else(|r| panic!("{name}: the written text is rejected\n{written}\n{r:#?}"));
    assert_eq!(first.value(), second.value(), "{name}\n{written}");
    assert_eq!(second.to_json().unwrap(), written, "{name}");
    written
}

#[test]
fn every_loading_fixture_round_trips_value_for_value() {
    let mut count = 0;
    for entry in std::fs::read_dir(fixtures()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "json") {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if !round_trip(&name, &std::fs::read(&path).unwrap()).is_empty() {
                count += 1;
            }
        }
    }
    assert!(count > 40, "only {count} fixtures loaded");
}

#[test]
fn a_vanilla_file_is_written_back_with_its_members() {
    // a copy of retail 1.26.45 `vanilla_1.26.30/recipes/sulfur.json`
    let written = round_trip("sulfur.json", &fixture("shaped_1_26_30_sulfur.json"));
    let value: Value = bedrock_addon_core::json::from_str(&written).unwrap();
    let original: Value = bedrock_addon_core::json::from_str(
        &String::from_utf8(fixture("shaped_1_26_30_sulfur.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(value, original);
}

#[test]
fn members_the_author_did_not_write_are_not_written() {
    let written = round_trip(
        "missing.json",
        &fixture("shaped_missing_format_version.json"),
    );
    for absent in [
        "format_version",
        "priority",
        "assume_symmetry",
        "unlock",
        "\"data\"",
        "\"count\"",
    ] {
        assert!(!written.contains(absent), "{absent} in\n{written}");
    }
}

#[test]
fn unknown_and_ignored_members_are_written_back() {
    let text = br##"{"format_version": [1, 12, 0], "minecraft:recipe_shaped": {"description": {"identifier": "demo:r", "note": 1}, "group": "wool", "tags": "crafting_table", "assume_symmetry": "no", "pattern": ["#"], "key": {"#": {"item": "minecraft:stick", "data": {"expression": "1"}, "colour": "red"}}, "result": ["minecraft:torch", {"item": "minecraft:coal", "data": "1 + 1", "count": "two"}]}, "comment": "kept", "minecraft:recipe_furnace": null}"##;
    let written = round_trip("extra.json", text);
    for kept in [
        "\"note\"",
        "\"group\"",
        "\"no\"",
        "\"expression\"",
        "\"two\"",
        "\"colour\"",
        "\"comment\"",
        "minecraft:recipe_furnace",
        "1 + 1",
    ] {
        assert!(written.contains(kept), "{kept} missing from\n{written}");
    }
}

/// A malformed `unlock` vanilla reads as `{None, []}` keeps the author's text.
#[test]
fn a_malformed_unlock_is_written_back_as_the_author_wrote_it() {
    for (name, kept) in [
        ("shaped_1_20_10_unlock_bad_context.json", "Sometimes"),
        ("bad_element.json", "\"count\": \"x\""),
    ] {
        let bytes = if name == "bad_element.json" {
            br##"{"format_version": "1.20.10", "minecraft:recipe_shaped": {"description": {"identifier": "demo:r"}, "tags": ["crafting_table"], "pattern": ["#"], "key": {"#": "minecraft:stick"}, "result": "minecraft:torch", "unlock": [{"count": "x"}]}}"##.to_vec()
        } else {
            fixture(name)
        };
        let written = round_trip(name, &bytes);
        assert!(
            written.contains(kept),
            "{name}: {kept} missing from\n{written}"
        );
        let document = parse_bytes(name, written.as_bytes(), &cx()).unwrap();
        let requirement = document.unlock_requirement(false).unwrap();
        assert_eq!(
            requirement.context,
            bedrock_addon_family_recipe::UnlockContext::None
        );
        assert!(requirement.ingredients.is_empty());
    }
}

#[test]
fn a_recipe_built_by_hand_is_written_and_read_back() {
    let recipe = Recipe::Shaped(Shaped {
        format_version: Some(FormatVersion::new(1, 20, 10)),
        description: Description {
            identifier: RecipeId::new("demo:ruby_block").unwrap(),
        },
        tags: Some(Tags::Many(vec!["crafting_table".to_owned()])),
        pattern: vec!["##".to_owned(), "##".to_owned()],
        key: [(
            "#".to_owned(),
            Ingredient::Object(IngredientObject {
                item: Some("demo:ruby".to_owned()),
                ..IngredientObject::default()
            }),
        )]
        .into_iter()
        .collect(),
        result: Output::One(Ingredient::Name("demo:ruby_block".to_owned())),
        unlock: Some(Unlock::Ingredients(vec![Ingredient::Name(
            "demo:ruby".to_owned(),
        )])),
        ..Shaped::default()
    });
    let text = recipe.to_json().unwrap();
    let document = parse_bytes("ruby_block.json", text.as_bytes(), &cx()).unwrap();
    silent(document.diagnostics());
    assert_eq!(document.value(), &recipe);
    assert_eq!(
        bedrock_addon_core::document::Placed::default_path(&recipe).as_str(),
        "recipes/demo_ruby_block.json"
    );
}
