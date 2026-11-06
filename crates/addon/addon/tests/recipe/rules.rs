//! The validation rules (F.3.6) that no fixture row covers, each with its
//! code and severity.

use bedrock_addon::diagnostic::{Severity, codes};
use bedrock_addon::families::recipe::{Recipe, UnlockContext};
use bedrock_addon::load::LoadContext;

use super::common::*;

/// A shaped 1.12 recipe `demo:r` with `members` replacing or adding payload members (JSON text
/// after the identifier and tags).
fn shaped_with(members: &str) -> Vec<u8> {
    format!(
        r#"{{"format_version": "1.12", "minecraft:recipe_shaped": {{"description": {{"identifier": "demo:r"}}, "tags": ["crafting_table"], {members}}}}}"#
    )
    .into_bytes()
}

const PATTERN_KEY_RESULT: &str =
    r##""pattern": ["#"], "key": {"#": "minecraft:stick"}, "result": "minecraft:torch""##;

/// A payload of `kind` (`brewing_container`, …) at 1.20.10 with `members`.
fn payload(kind: &str, members: &str) -> Vec<u8> {
    format!(
        r#"{{"format_version": "1.20.10", "minecraft:recipe_{kind}": {{"description": {{"identifier": "demo:r"}}, {members}}}}}"#
    )
    .into_bytes()
}

fn parse(
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<bedrock_addon::document::Document<Recipe>, bedrock_addon::document::Rejected> {
    parse_bytes("r.json", bytes, cx)
}

fn rejects(bytes: &[u8], cx: &LoadContext) -> bedrock_addon::diagnostic::Diagnostics {
    match parse(bytes, cx) {
        Ok(document) => panic!("loaded: {:#?}", document.diagnostics()),
        Err(rejected) => rejected.diagnostics,
    }
}

#[test]
fn a_root_that_is_not_an_object_is_skipped() {
    let diagnostics = rejects(b"[1, 2]", &cx());
    let finding = one(
        &diagnostics,
        "JSON: {} root value is not a object",
        Severity::Error,
    );
    // By legacy name: the variant and the definitions that were it.
    assert!(matches!(finding.code.name(), "MalformedInput"));
}

#[test]
fn a_syntax_error_is_skipped() {
    let diagnostics = rejects(b"{\"format_version\": ", &cx());
    let finding = one(
        &diagnostics,
        "JSON: {} has an error: \n {}",
        Severity::Error,
    );
    // By legacy name: the variant and the definitions that were it.
    assert!(matches!(finding.code.name(), "MalformedInput"));
}

#[test]
fn a_missing_or_empty_identifier_is_skipped() {
    for description in [
        r#""description": {}"#,
        r#""description": {"identifier": ""}"#,
        r#""description": {"identifier": 7}"#,
    ] {
        let text = format!(
            r#"{{"format_version": "1.12", "minecraft:recipe_shaped": {{{description}, "tags": ["crafting_table"], {PATTERN_KEY_RESULT}}}}}"#
        );
        let diagnostics = rejects(text.as_bytes(), &cx());
        let finding = one(&diagnostics, "JSON: {} has no identifier", Severity::Error);
        // By legacy name: the variant and the definitions that were it.
        assert!(matches!(finding.code.name(), "MissingIdentifier"));
    }
}

#[test]
fn an_invalid_unlock_ingredient_is_logged_and_the_recipe_loads() {
    let text = shaped_with(&format!(
        r#"{PATTERN_KEY_RESULT}, "unlock": [{{"count": 1}}]"#
    ));
    let document = parse(&text, &cx()).unwrap();
    let finding = one_kept(
        document.diagnostics(),
        "Recipe: {} has an invalid unlock ingredient",
    );
    assert_eq!(args(finding), ["demo:r"]);
    // The ingredient's own message is the same level-3 line of `_loadIngredientFromJson`,
    // and the recipe loads: a kept error too.
    one_kept(
        document.diagnostics(),
        "Recipe ingredient is invalid: Must contain either a valid item name or item tag",
    );
    assert!(!document.diagnostics().has_errors());
    let requirement = document.unlock_requirement(false).unwrap();
    assert_eq!(requirement.context, UnlockContext::None);
    assert!(requirement.ingredients.is_empty());
}

#[test]
fn an_unlock_that_is_a_string_is_absent() {
    let text = shaped_with(&format!(
        r#"{PATTERN_KEY_RESULT}, "unlock": "AlwaysUnlocked""#
    ));
    let document = parse(&text, &cx()).unwrap();
    let Recipe::Shaped(recipe) = document.value() else {
        panic!()
    };
    assert!(recipe.unlock.is_none());
    assert!(kept(&document, &["minecraft:recipe_shaped"], "unlock").is_some());
}

#[test]
fn a_missing_empty_or_mistyped_pattern_or_key_drops_the_recipe() {
    for members in [
        r##""key": {"#": "minecraft:stick"}, "result": "minecraft:torch""##,
        r##""pattern": [], "key": {"#": "minecraft:stick"}, "result": "minecraft:torch""##,
        r##""pattern": "#", "key": {"#": "minecraft:stick"}, "result": "minecraft:torch""##,
        r##""pattern": [1], "key": {"#": "minecraft:stick"}, "result": "minecraft:torch""##,
        r##""pattern": ["#"], "result": "minecraft:torch""##,
        r##""pattern": ["#"], "key": {}, "result": "minecraft:torch""##,
        r##""pattern": ["#"], "key": ["minecraft:stick"], "result": "minecraft:torch""##,
        r##""pattern": ["#"], "key": {"": "minecraft:stick"}, "result": "minecraft:torch""##,
    ] {
        let diagnostics = rejects(&shaped_with(members), &cx());
        // vanilla drops these silently: our finding carries no text
        let errors: Vec<_> = diagnostics.at_least(Severity::Error).collect();
        assert!(!errors.is_empty(), "{members}");
        assert!(
            errors.iter().all(|d| template(d).is_none()),
            "{members}: {errors:#?}"
        );
    }
}

#[test]
fn a_pattern_with_no_keyed_symbol_is_not_added() {
    let diagnostics = rejects(
        &shaped_with(
            r##""pattern": ["ab"], "key": {"#": "minecraft:stick"}, "result": "minecraft:torch""##,
        ),
        &cx(),
    );
    let finding = one(
        &diagnostics,
        "Recipes::addShapedRecipe: '{}' doesn't have a single valid key in its pattern!",
        Severity::Error,
    );
    assert_eq!(args(finding), ["demo:r"]);
    let diagnostics = rejects(
        &shaped_with(
            r##""pattern": ["  "], "key": {"#": "minecraft:stick"}, "result": "minecraft:torch""##,
        ),
        &cx(),
    );
    one(
        &diagnostics,
        "Recipes::addShapedRecipe: adding an empty recipe!",
        Severity::Error,
    );
}

#[test]
fn a_symbol_is_the_first_byte_of_its_key() {
    let document = parse(
        &shaped_with(r##""pattern": ["#"], "key": {"#ignored": "minecraft:stick"}, "result": "minecraft:torch""##),
        &cx(),
    )
    .unwrap();
    clean(document.diagnostics());
}

#[test]
fn shapeless_ingredients_that_are_not_a_list_drop_the_recipe() {
    let text = br#"{"format_version": "1.12", "minecraft:recipe_shapeless": {"description": {"identifier": "demo:r"}, "tags": ["crafting_table"], "ingredients": "minecraft:stick", "result": "minecraft:torch"}}"#;
    let diagnostics = rejects(text, &cx());
    assert!(diagnostics.has_errors());
    let text = br#"{"format_version": "1.12", "minecraft:recipe_shapeless": {"description": {"identifier": "demo:r"}, "tags": ["crafting_table"], "ingredients": [5], "result": "minecraft:torch"}}"#;
    let diagnostics = rejects(text, &cx());
    one(&diagnostics, "Recipe result malformed", Severity::Error);
}

#[test]
fn an_empty_result_list_has_no_result_items() {
    let diagnostics = rejects(
        &shaped_with(r##""pattern": ["#"], "key": {"#": "minecraft:stick"}, "result": []"##),
        &cx(),
    );
    one(
        &diagnostics,
        "Recipe has no result item(s)",
        Severity::Error,
    );
    let diagnostics = rejects(
        &shaped_with(r##""pattern": ["#"], "key": {"#": "minecraft:stick"}"##),
        &cx(),
    );
    one(&diagnostics, "Recipe result malformed", Severity::Error);
}

#[test]
fn null_entries_of_a_result_list_are_skipped() {
    let document = parse(
        &shaped_with(r##""pattern": ["#"], "key": {"#": "minecraft:stick"}, "result": [null, "minecraft:torch"]"##),
        &cx(),
    )
    .unwrap();
    let Recipe::Shaped(recipe) = document.value() else {
        panic!()
    };
    assert_eq!(recipe.result.as_slice().len(), 1);
}

#[test]
fn an_ingredient_without_item_or_tag_is_invalid() {
    // tags not allowed (a result): no item name
    let diagnostics = rejects(
        &shaped_with(
            r##""pattern": ["#"], "key": {"#": "minecraft:stick"}, "result": {"count": 2}"##,
        ),
        &cx(),
    );
    one(
        &diagnostics,
        "Recipe ingredient is invalid: Must contain a valid item name",
        Severity::Error,
    );
    // tags allowed (a key): neither name nor tag
    let diagnostics = rejects(
        &shaped_with(
            r##""pattern": ["#"], "key": {"#": {"tag": 5}}, "result": "minecraft:torch""##,
        ),
        &cx(),
    );
    one(
        &diagnostics,
        "Recipe ingredient is invalid: Must contain either a valid item name or item tag",
        Severity::Error,
    );
    // a number is an item with an empty name
    let diagnostics = rejects(
        &shaped_with(r##""pattern": ["#"], "key": {"#": 5}, "result": "minecraft:torch""##),
        &cx(),
    );
    one(
        &diagnostics,
        "The Item: {} is missing or invalid, can't make the recipe",
        Severity::Error,
    );
}

#[test]
fn brewing_container_input_and_output_messages() {
    let diagnostics = rejects(
        &payload(
            "brewing_container",
            r#""tags": ["brewing_stand"], "input": {"tag": 1}, "reagent": "minecraft:gunpowder", "output": "minecraft:splash_potion""#,
        ),
        &cx(),
    );
    one(
        &diagnostics,
        "Brewing Container Recipe: {} has an invalid input item",
        Severity::Error,
    );
    let diagnostics = rejects(
        &payload(
            "brewing_container",
            r#""tags": ["brewing_stand"], "input": "minecraft:potion", "reagent": "minecraft:gunpowder", "output": {"tag": "x"}"#,
        ),
        &cx(),
    );
    one(
        &diagnostics,
        "Brewing Container Recipe: {} has an invalid output item",
        Severity::Error,
    );
    // a count on the input of a modern pack
    let diagnostics = rejects(
        &payload(
            "brewing_container",
            r#""tags": ["brewing_stand"], "input": {"item": "minecraft:potion", "count": 2}, "reagent": "minecraft:gunpowder", "output": "minecraft:splash_potion""#,
        ),
        &cx(),
    );
    one(
        &diagnostics,
        "Brewing Container Recipe: {} has an invalid input item",
        Severity::Error,
    );
}

#[test]
fn material_reduction_input_and_output_messages() {
    let diagnostics = rejects(
        &payload(
            "material_reduction",
            r#""tags": "material_reducer", "input": {"item": "minecraft:stone", "count": 2}, "output": []"#,
        ),
        &cx(),
    );
    one(
        &diagnostics,
        "Material Reduction Recipe: {} has an invalid input item",
        Severity::Error,
    );
    let diagnostics = rejects(
        &payload(
            "material_reduction",
            r#""tags": "material_reducer", "input": "minecraft:stone", "output": [{"tag": "x"}]"#,
        ),
        &cx(),
    );
    one(
        &diagnostics,
        "Material Reduction Recipe: {} has an invalid output item",
        Severity::Error,
    );
}

#[test]
fn material_reduction_output_of_another_type_gives_no_outputs() {
    let document = parse(
        &payload(
            "material_reduction",
            r#""tags": "material_reducer", "input": "minecraft:stone", "output": "minecraft:element_1""#,
        ),
        &cx(),
    )
    .unwrap();
    let Recipe::MaterialReduction(recipe) = document.value() else {
        panic!()
    };
    assert!(recipe.output.is_none());
    assert!(
        kept(
            &document,
            &["minecraft:recipe_material_reduction"],
            "output"
        )
        .is_some()
    );
    clean(document.diagnostics());
}

#[test]
fn a_brewing_mix_part_logs_only_the_ingredient_message() {
    let diagnostics = rejects(
        &payload(
            "brewing_mix",
            r#""tags": ["brewing_stand"], "input": "minecraft:potion_type:awkward", "reagent": {"tag": "x"}, "output": "minecraft:potion_type:swiftness""#,
        ),
        &cx(),
    );
    one(
        &diagnostics,
        "Recipe ingredient is invalid: Must contain a valid item name",
        Severity::Error,
    );
    assert_eq!(
        diagnostics.at_least(Severity::Error).count(),
        1,
        "{diagnostics:#?}"
    );
}

const TRANSFORM: &str = r#""tags": ["smithing_table"], "template": "minecraft:netherite_upgrade_smithing_template", "base": "minecraft:diamond_axe", "addition": "minecraft:netherite_ingot", "result": "minecraft:netherite_axe""#;

#[test]
fn smithing_transform_part_messages() {
    for (part, text) in [
        (
            "template",
            "Smithing Transform Recipe: {} has an invalid template item.",
        ),
        (
            "base",
            "Smithing Transform Recipe: {} has an invalid base item.",
        ),
        (
            "addition",
            "Smithing Transform Recipe: {} has an invalid addition item.",
        ),
        (
            "result",
            "Smithing Transform Recipe: {} has an invalid result item.",
        ),
    ] {
        let members = TRANSFORM.replace(
            &format!(r#""{part}": "#),
            &format!(r#""{part}": {{"tag": "x"}}, "{part}_was": "#),
        );
        let diagnostics = rejects(&payload("smithing_transform", &members), &cx());
        one(&diagnostics, text, Severity::Error);
    }
}

#[test]
fn smithing_transform_template_is_optional_and_unread_before_1_20() {
    let members = TRANSFORM.replace(
        r#""template": "minecraft:netherite_upgrade_smithing_template", "#,
        "",
    );
    let document = parse(&payload("smithing_transform", &members), &cx()).unwrap();
    let Recipe::SmithingTransform(recipe) = document.value() else {
        panic!()
    };
    assert!(recipe.template.is_none());
    // in a world before 1.20 the template is not read, so even a bad one passes
    let members = TRANSFORM.replace(
        r#""template": "minecraft:netherite_upgrade_smithing_template""#,
        r#""template": {"tag": "x"}"#,
    );
    let document = parse(&payload("smithing_transform", &members), &cx_world(1, 19)).unwrap();
    let Recipe::SmithingTransform(recipe) = document.value() else {
        panic!()
    };
    assert!(recipe.template.is_none());
    assert!(
        kept(
            &document,
            &["minecraft:recipe_smithing_transform"],
            "template"
        )
        .is_some()
    );
    assert!(!document.diagnostics().has_errors());
}

#[test]
fn smithing_transform_result_takes_no_count() {
    let members = TRANSFORM.replace(
        r#""result": "minecraft:netherite_axe""#,
        r#""result": {"item": "minecraft:netherite_axe", "count": 3}"#,
    );
    let diagnostics = rejects(&payload("smithing_transform", &members), &cx());
    one(
        &diagnostics,
        "Smithing Transform Recipe: {} has an invalid result item.",
        Severity::Error,
    );
}

#[test]
fn smithing_trim_tag_message() {
    let text = payload(
        "smithing_trim",
        r#""tags": ["crafting_table"], "template": {"tag": "minecraft:trim_templates"}, "base": {"tag": "minecraft:trimmable_armors"}, "addition": {"tag": "minecraft:trim_materials"}"#,
    );
    let diagnostics = rejects(&text, &cx());
    let finding = one(
        &diagnostics,
        "Smithing Trim Recipe: {} has an invalid tag. They only supported one is {}.",
        Severity::Error,
    );
    assert_eq!(args(finding), ["demo:r", "smithing_table"]);
}

#[test]
fn unknown_members_are_kept_with_an_info() {
    let document = parse(
        &shaped_with(&format!(
            r#"{PATTERN_KEY_RESULT}, "group": "wool", "format_version": "1.12""#
        )),
        &cx(),
    )
    .unwrap();
    let Recipe::Shaped(_) = document.value() else {
        panic!()
    };
    assert!(kept(&document, &["minecraft:recipe_shaped"], "group").is_some());
    assert!(kept(&document, &["minecraft:recipe_shaped"], "format_version").is_some());
    let unknown: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|d| d.code.is(&codes::UNRECOGNIZED_FIELD))
        .collect();
    assert_eq!(unknown.len(), 2, "{:#?}", document.diagnostics());
    assert!(unknown.iter().all(|d| d.severity == Severity::Info));
}

#[test]
fn the_unlock_of_a_furnace_recipe_is_parsed_and_its_malformed_forms_logged() {
    let text = payload(
        "furnace",
        r#""tags": ["furnace"], "input": "minecraft:beef", "output": "minecraft:cooked_beef", "unlock": {"context": "PlayerHasManyItems"}"#,
    );
    let document = parse(&text, &cx()).unwrap();
    assert_eq!(
        document.unlock_requirement(false).unwrap().context,
        UnlockContext::PlayerHasManyItems
    );
    let text = payload(
        "furnace",
        r#""tags": ["furnace"], "input": "minecraft:beef", "output": "minecraft:cooked_beef", "unlock": {}"#,
    );
    let document = parse(&text, &cx()).unwrap();
    assert_eq!(
        document.unlock_requirement(true).unwrap().context,
        UnlockContext::AlwaysUnlocked
    );
}

#[test]
fn unlock_on_a_type_that_does_not_read_it_is_an_unknown_member() {
    let text = payload(
        "brewing_container",
        r#""tags": ["brewing_stand"], "input": "minecraft:potion", "reagent": "minecraft:gunpowder", "output": "minecraft:splash_potion", "unlock": {"context": "None"}"#,
    );
    let document = parse(&text, &cx()).unwrap();
    assert!(document.unlock_requirement(false).is_none());
    let Recipe::BrewingContainer(_) = document.value() else {
        panic!()
    };
    assert!(kept(&document, &["minecraft:recipe_brewing_container"], "unlock").is_some());
    assert!(matches!(
        kept(&document, &["minecraft:recipe_brewing_container"], "unlock")
            .and_then(|v| v.as_object())
            .map(facet_value::VObject::len),
        Some(1)
    ));
}

/// Vanilla's `_isRecipeValidToAdd` compares every recipe with the ones already registered,
/// hard-coded recipes included, by their item instances; neither is known at load:
/// two recipes with the same ingredients and different results both load.
#[test]
fn add_time_duplicate_checks_are_not_run_deviation() {
    let first = shaped_with(PATTERN_KEY_RESULT);
    let second = String::from_utf8(shaped_with(
        r##""pattern": ["#"], "key": {"#": "minecraft:stick"}, "result": "minecraft:coal""##,
    ))
    .unwrap()
    .replace("demo:r", "demo:s");
    for text in [first, second.into_bytes()] {
        let document = parse(&text, &cx()).unwrap();
        clean(document.diagnostics());
    }
}
