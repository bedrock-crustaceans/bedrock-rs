//! The item-id rows of the fixtures (the game's item parser) and the `data` / inline-aux rows.

use bedrock_addon::diagnostic::Severity;

use super::common::*;

#[test]
fn ingredient_item_id_case_resolves_to_minecraft_stick() {
    let document = loaded("ingredient_item_id_case.json", &cx());
    silent(document.diagnostics());
    let key = &shaped(&document).key;
    for symbol in ["#", "$"] {
        assert_eq!(
            key[symbol].item_name().unwrap().id(),
            "minecraft:stick",
            "{symbol}"
        );
    }
}

#[test]
fn ingredient_item_id_aux_300_is_logged_and_parsing_continues() {
    let document = loaded("ingredient_item_id_aux_300.json", &cx());
    let finding = one(
        document.diagnostics(),
        "Invalid aux value in item ID \"{}\"",
        Severity::Error,
    );
    // The game's item parser logs it at level 3 and goes on: a kept error.
    assert!(finding.kept);
    assert!(finding.is(&bedrock_addon::families::recipe::codes::INVALID_AUX));
    assert_eq!(args(finding), ["minecraft:wool:300"]);
    let name = shaped(&document).key["#"].item_name().unwrap();
    assert_eq!((name.id().as_str(), name.aux), ("minecraft:wool", None));
}

#[test]
fn ingredient_item_id_four_tokens_uses_the_first_three() {
    let document = loaded("ingredient_item_id_four_tokens.json", &cx());
    let finding = one(
        document.diagnostics(),
        "Excessive number of tokens in item ID \"{}\"",
        Severity::Error,
    );
    // The game's item parser logs it at level 3 and goes on: a kept error.
    assert!(finding.kept);
    assert_eq!(args(finding), ["minecraft:wool:1:2"]);
    let ingredient = &shaped(&document).key["#"];
    assert_eq!(ingredient.item_name().unwrap().id(), "minecraft:wool");
    assert_eq!(ingredient.aux(), 1);
}

#[test]
fn ingredient_item_id_empty_namespace_is_logged() {
    let document = loaded("ingredient_item_id_empty_namespace.json", &cx());
    let finding = one(
        document.diagnostics(),
        "Empty namespace in item ID \"{}\"",
        Severity::Error,
    );
    // The game's item parser logs it at level 3 and goes on: a kept error.
    assert!(finding.kept);
    assert_eq!(args(finding), [":stick"]);
}

#[test]
fn ingredient_inline_aux_overrides_data() {
    let document = loaded("ingredient_inline_aux_overrides_data.json", &cx());
    clean(document.diagnostics());
    assert_eq!(shaped(&document).key["#"].aux(), 3);
}

#[cfg(feature = "molang")]
#[test]
fn ingredient_data_float_is_truncated_after_evaluation() {
    let document = loaded("ingredient_data_float.json", &super::molang::cx_molang());
    clean(document.diagnostics());
    let key = &shaped(&document).key;
    assert_eq!(key["#"].aux(), 2);
    assert_eq!(key["$"].aux(), 3);
}

#[test]
fn ingredient_data_float_number_is_truncated_without_the_compile_check() {
    let document = loaded("ingredient_data_float.json", &cx());
    assert_eq!(shaped(&document).key["#"].aux(), 2);
}

/// A host whose provider knows no potion: `minecraft:potion_type:awkward` is then an ordinary
/// item text to the reader, and the game's item parser logs its third token as an aux value.
struct NoPotions;

impl bedrock_addon::load::BuiltIns for NoPotions {
    fn contains(&self, _kind: &'static str, _id: &str) -> bool {
        false
    }
    fn potion(&self, _name: &str) -> Option<&str> {
        None
    }
}

#[test]
fn a_potion_the_provider_does_not_know_is_an_item_text_with_its_finding() {
    let tables = loaded("brewing_mix_potion_type.json", &cx_base_game());
    clean(tables.diagnostics());
    let mut cx = cx_base_game();
    cx.set_built_ins(Some(std::sync::Arc::new(NoPotions)));
    let hosted = loaded("brewing_mix_potion_type.json", &cx);
    let findings = with_text(hosted.diagnostics(), "Invalid aux value in item ID \"{}\"");
    let texts: Vec<Vec<String>> = findings.iter().map(|finding| args(finding)).collect();
    assert_eq!(
        texts,
        [
            ["minecraft:potion_type:awkward"],
            ["minecraft:potion_type:weakness"]
        ]
    );
    assert!(findings.iter().all(|finding| finding.kept));
}
