//! The single-file rows of the fixtures (F.3.7), in the table's order. Stack
//! rows are in `stack.rs`, item-id and `data` rows in `ingredients.rs` and `molang.rs`, rows that
//! need the item registry in `link.rs`.

use bedrock_addon::diagnostic::{Severity, codes};
use bedrock_addon::families::recipe::{
    Ingredient, Output, PotionMix, Recipe, Unlock, UnlockContext, input_count_allowed,
    unlock_required,
};
use bedrock_addon::version::{FormatVersion, MinEngineVersion};

use super::common::*;

const UNLOCK_REQUIRED: &str = "1.20+ Recipes require unlock data";
const NO_LIST_OR_CONTEXT: &str = "Recipe: {} has an unlocking element but does not contain an ingredient list or and unlocking context";
const MALFORMED_CONTEXT: &str = "Recipe: {} has a malformed unlocking context";
const EMPTY_UNLOCK_ARRAY: &str = "Recipe: {} has an empty unlocking ingredient array";
const NO_INPUT_COUNT: &str =
    "Recipe ingredient is invalid: Cannot specify an item count for input items";
const RESULT_MALFORMED: &str = "Recipe result malformed";

#[test]
fn shaped_1_12_no_unlock_thirdparty_is_always_unlocked() {
    let document = loaded("shaped_1_12_no_unlock_thirdparty.json", &cx());
    clean(document.diagnostics());
    let requirement = document.unlock_requirement(false).unwrap();
    assert_eq!(requirement.context, UnlockContext::AlwaysUnlocked);
    assert!(requirement.ingredients.is_empty());
}

#[test]
fn shaped_1_12_no_unlock_basegame_has_context_none() {
    let cx = cx_base_game();
    let document = loaded("shaped_1_12_no_unlock_basegame.json", &cx);
    clean(document.diagnostics());
    let requirement = document.unlock_requirement(cx.is_base_game_pack).unwrap();
    assert_eq!(requirement.context, UnlockContext::None);
}

#[test]
fn shaped_1_20_10_no_unlock_is_rejected() {
    let rejected = rejected("shaped_1_20_10_no_unlock.json", &cx());
    let finding = one(&rejected.diagnostics, UNLOCK_REQUIRED, Severity::Error);
    // By legacy name: the variant and the definitions that were it.
    assert!(matches!(finding.code.id(), "recipe/unlock_required"));
}

#[test]
fn shaped_1_20_10_both_unlock_shapes_parse() {
    let context = loaded("shaped_1_20_10_unlock_context.json", &cx());
    clean(context.diagnostics());
    assert!(matches!(
        &shaped(&context).unlock,
        Some(Unlock::Context {
            context: UnlockContext::PlayerInWater,
            ..
        })
    ));
    let items = loaded("shaped_1_20_10_unlock_items.json", &cx());
    clean(items.diagnostics());
    let Some(Unlock::Ingredients(ingredients)) = &shaped(&items).unlock else {
        panic!("{:?}", shaped(&items).unlock);
    };
    assert_eq!(ingredients.len(), 1);
    assert_eq!(ingredients[0].item_name().unwrap().id(), "minecraft:stick");
}

#[test]
fn shaped_1_20_10_unlock_bad_context_loads_with_context_none() {
    let document = loaded("shaped_1_20_10_unlock_bad_context.json", &cx());
    let finding = one_kept(document.diagnostics(), MALFORMED_CONTEXT);
    assert_eq!(args(finding), ["demo:bad_context"]);
    let requirement = document.unlock_requirement(false).unwrap();
    assert_eq!(requirement.context, UnlockContext::None);
    assert!(requirement.ingredients.is_empty());
}

#[test]
fn shaped_1_20_10_unlock_context_none_is_silent() {
    let document = loaded("shaped_1_20_10_unlock_context_none.json", &cx());
    silent(document.diagnostics());
    let requirement = document.unlock_requirement(false).unwrap();
    assert_eq!(requirement.context, UnlockContext::None);
}

#[test]
fn shaped_1_20_10_unlock_empty_array_loads_with_none_and_no_items() {
    let document = loaded("shaped_1_20_10_unlock_empty_array.json", &cx());
    one_kept(document.diagnostics(), EMPTY_UNLOCK_ARRAY);
    let requirement = document.unlock_requirement(false).unwrap();
    assert_eq!(requirement.context, UnlockContext::None);
    assert!(requirement.ingredients.is_empty());
}

#[test]
fn shaped_1_20_10_unlock_object_no_context_is_rejected_as_absent() {
    let rejected = rejected("shaped_1_20_10_unlock_object_no_context.json", &cx());
    one_kept(&rejected.diagnostics, NO_LIST_OR_CONTEXT);
    one(&rejected.diagnostics, UNLOCK_REQUIRED, Severity::Error);
    let order: Vec<_> = rejected
        .diagnostics
        .iter()
        .filter_map(template)
        .filter(|t| *t == NO_LIST_OR_CONTEXT || *t == UNLOCK_REQUIRED)
        .collect();
    assert_eq!(order, [NO_LIST_OR_CONTEXT, UNLOCK_REQUIRED]);
}

#[test]
fn shaped_1_16_unlock_object_no_context_gets_the_default() {
    let document = loaded("shaped_1_16_unlock_object_no_context.json", &cx());
    one_kept(document.diagnostics(), NO_LIST_OR_CONTEXT);
    assert!(with_text(document.diagnostics(), UNLOCK_REQUIRED).is_empty());
    let requirement = document.unlock_requirement(false).unwrap();
    assert_eq!(requirement.context, UnlockContext::AlwaysUnlocked);
}

/// Vanilla logs `Missing version tag` at level 3 (`_contentLog(true, 3, 25)`) and loads the recipe as 1.0.0: a kept error.
#[test]
fn shaped_missing_format_version_loads_as_1_0_0_with_a_kept_error() {
    let document = loaded("shaped_missing_format_version.json", &cx());
    let warnings: Vec<_> = document
        .diagnostics()
        .iter()
        // By legacy name: the variant and the definitions that were it.
        .filter(|d| matches!(d.code.id(), "recipe/missing_version_tag"))
        .collect();
    assert_eq!(warnings.len(), 1, "{:#?}", document.diagnostics());
    assert_eq!(warnings[0].severity, Severity::Error);
    assert!(warnings[0].kept);
    assert_eq!(template(warnings[0]), Some("Missing version tag"));
    assert_eq!(document.version().effective, FormatVersion::new(1, 0, 0));
    assert_eq!(document.version().written, None);
    assert_eq!(
        document.unlock_requirement(false).unwrap().context,
        UnlockContext::AlwaysUnlocked
    );
}

#[test]
fn shaped_format_version_99_loads_with_an_info() {
    let document = loaded("shaped_format_version_99.json", &cx());
    let above: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|d| {
            d.code
                .args_of(&bedrock_addon::diagnostic::codes::FORMAT_VERSION_ABOVE_ENGINE)
                .is_some_and(|args| args[0] == "99.0.0")
        })
        .collect();
    assert_eq!(above.len(), 1, "{:#?}", document.diagnostics());
    assert_eq!(above[0].severity, Severity::Info);
    clean(document.diagnostics());
}

#[test]
fn shaped_key_count_mev_1_19_20_accepts_count() {
    let document = loaded("shaped_key_count_mev_1_19_20.json", &cx_mev(1, 19, 20));
    clean(document.diagnostics());
    assert_eq!(shaped(&document).key["#"].count(), 2);
}

/// a pack without `min_engine_version` has vanilla's zeroed version, which is legacy.
#[test]
fn a_pack_without_min_engine_version_accepts_input_count() {
    let mut cx = cx();
    cx.min_engine_version = None;
    let document = loaded("shaped_key_count_mev_1_19_30.json", &cx);
    clean(document.diagnostics());
    assert_eq!(shaped(&document).key["#"].count(), 2);
}

#[test]
fn shaped_key_count_mev_1_19_30_is_rejected() {
    let rejected = rejected("shaped_key_count_mev_1_19_30.json", &cx_mev(1, 19, 30));
    one(&rejected.diagnostics, NO_INPUT_COUNT, Severity::Error);
    let site = one(
        &rejected.diagnostics,
        "Recipe for: {} is missing (unknown) ingredient",
        Severity::Error,
    );
    assert_eq!(args(site), ["demo:key_count"]);
}

#[test]
fn shaped_assume_symmetry_default_false_and_string() {
    let default = loaded("shaped_assume_symmetry_default.json", &cx());
    assert!(shaped(&default).assumes_symmetry());
    let off = loaded("shaped_assume_symmetry_false.json", &cx());
    assert!(!shaped(&off).assumes_symmetry());
    clean(off.diagnostics());
    let string = loaded("shaped_assume_symmetry_string.json", &cx());
    assert!(shaped(&string).assumes_symmetry());
    // vanilla ignores the value silently: an info, and the member kept for write-back
    clean(string.diagnostics());
    let infos: Vec<_> = string
        .diagnostics()
        .iter()
        .filter(|d| d.code.is(&codes::INVALID_TYPE))
        .collect();
    assert_eq!(infos.len(), 1, "{:#?}", string.diagnostics());
    assert_eq!(infos[0].severity, Severity::Info);
    assert_eq!(
        infos[0].location.json_path.to_string(),
        "minecraft:recipe_shaped/assume_symmetry"
    );
    assert!(kept(&string, &["minecraft:recipe_shaped"], "assume_symmetry").is_some());
}

#[test]
fn shaped_result_array_accepts_several_outputs() {
    let document = loaded("shaped_result_array.json", &cx());
    clean(document.diagnostics());
    let Output::Many(items) = &shaped(&document).result else {
        panic!("{:?}", shaped(&document).result);
    };
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].count(), 4);
}

#[test]
fn shapeless_result_array_two_loads_with_the_first_result() {
    let document = loaded("shapeless_result_array_two.json", &cx());
    one_kept(
        document.diagnostics(),
        "unsupported multiple output types for result list in shapeless",
    );
    let Recipe::Shapeless(recipe) = document.value() else {
        panic!()
    };
    let first = recipe.result_item().unwrap();
    assert_eq!(first.item_name().unwrap().id(), "minecraft:torch");
}

#[test]
fn shapeless_count_zero_contributes_no_slot() {
    let document = loaded("shapeless_count_zero.json", &cx());
    clean(document.diagnostics());
    let Recipe::Shapeless(recipe) = document.value() else {
        panic!()
    };
    let slots: Vec<_> = recipe
        .slots()
        .map(|i| i.item_name().unwrap().id())
        .collect();
    assert_eq!(slots, ["minecraft:coal"]);
}

#[test]
fn shaped_pattern_4_wide_is_not_added() {
    let rejected = rejected("shaped_pattern_4_wide.json", &cx());
    let finding = one(
        &rejected.diagnostics,
        "Adding a recipe larger than 3x3: {} width, {} height",
        Severity::Error,
    );
    assert_eq!(args(finding), ["4", "1"]);
}

#[test]
fn shaped_pattern_4_tall_is_not_added() {
    let text = br##"{"format_version": "1.12", "minecraft:recipe_shaped": {"description": {"identifier": "demo:tall"}, "tags": ["crafting_table"], "pattern": ["#", "#", "#", "#"], "key": {"#": "minecraft:stick"}, "result": "minecraft:torch"}}"##;
    let rejected = parse_bytes("tall.json", text, &cx()).unwrap_err();
    let finding = one(
        &rejected.diagnostics,
        "Adding a recipe larger than 3x3: {} width, {} height",
        Severity::Error,
    );
    assert_eq!(args(finding), ["1", "4"]);
}

#[test]
fn shaped_result_array_one_bad_is_rejected() {
    let rejected = rejected("shaped_result_array_one_bad.json", &cx());
    one(&rejected.diagnostics, RESULT_MALFORMED, Severity::Error);
}

#[test]
fn shapeless_count_expands_to_three_slots() {
    let document = loaded("shapeless_count_expands.json", &cx());
    silent(document.diagnostics());
    let Recipe::Shapeless(recipe) = document.value() else {
        panic!()
    };
    assert_eq!(recipe.slots().count(), 3);
}

#[test]
fn shapeless_tag_ingredient_is_accepted() {
    let document = loaded("shapeless_tag_ingredient.json", &cx());
    clean(document.diagnostics());
    let Recipe::Shapeless(recipe) = document.value() else {
        panic!()
    };
    assert_eq!(recipe.ingredients[0].tag(), Some("minecraft:planks"));
}

#[test]
fn furnace_no_unlock_1_20_10_loads_always_unlocked() {
    let document = loaded("furnace_no_unlock_1_20_10.json", &cx_base_game());
    clean(document.diagnostics());
    assert_eq!(
        document.unlock_requirement(true).unwrap().context,
        UnlockContext::AlwaysUnlocked
    );
}

#[test]
fn furnace_input_count_is_rejected_at_every_min_engine_version() {
    for cx in [cx(), cx_mev(1, 19, 20), cx_mev(1, 16, 0)] {
        let rejected = rejected("furnace_input_count.json", &cx);
        one(&rejected.diagnostics, NO_INPUT_COUNT, Severity::Error);
        one(&rejected.diagnostics, RESULT_MALFORMED, Severity::Error);
    }
}

#[test]
fn furnace_input_no_count_has_stack_size_one() {
    let document = loaded("furnace_input_no_count.json", &cx());
    clean(document.diagnostics());
    let Recipe::Furnace(recipe) = document.value() else {
        panic!()
    };
    assert_eq!(recipe.input.count(), 1);
}

#[test]
fn brewing_mix_potion_type_expands_to_three_mixes() {
    let document = loaded("brewing_mix_potion_type.json", &cx_base_game());
    clean(document.diagnostics());
    let Recipe::BrewingMix(mix) = document.value() else {
        panic!()
    };
    let potions = mix.potion_mix().unwrap();
    assert_eq!(potions.input, "minecraft:awkward");
    assert_eq!(potions.output, "minecraft:weakness");
    assert_eq!(PotionMix::CONTAINERS.len(), 3);
}

#[test]
fn brewing_container_reagent_tag_is_rejected() {
    let rejected = rejected("brewing_container_reagent_tag.json", &cx());
    one(
        &rejected.diagnostics,
        "Recipe ingredient is invalid: Must contain a valid item name",
        Severity::Error,
    );
    let site = one(
        &rejected.diagnostics,
        "Brewing Container Recipe: {} has an invalid reagent item",
        Severity::Error,
    );
    assert_eq!(args(site), ["demo:reagent_tag"]);
}

#[test]
fn brewing_mix_reagent_count_is_rejected_even_for_a_legacy_pack() {
    let rejected = rejected("brewing_mix_reagent_count.json", &cx_mev(1, 19, 20));
    one(&rejected.diagnostics, NO_INPUT_COUNT, Severity::Error);
}

#[test]
fn brewing_mix_input_count_mev_1_19_20_is_accepted() {
    let document = loaded(
        "brewing_mix_input_count_mev_1_19_20.json",
        &cx_mev(1, 19, 20),
    );
    clean(document.diagnostics());
    let Recipe::BrewingMix(mix) = document.value() else {
        panic!()
    };
    assert_eq!(mix.input.count(), 2);
    assert!(mix.potion_mix().is_none());
}

#[test]
fn brewing_mix_input_count_in_a_modern_pack_is_rejected_without_a_site_text() {
    for cx in [cx_mev(1, 19, 30), cx()] {
        let rejected = rejected("brewing_mix_input_count_mev_1_19_20.json", &cx);
        one(&rejected.diagnostics, NO_INPUT_COUNT, Severity::Error);
        // the brewing mix logs only the ingredient's message
        assert_eq!(
            rejected.diagnostics.at_least(Severity::Error).count(),
            1,
            "{:#?}",
            rejected.diagnostics
        );
    }
}

#[test]
fn material_reduction_10_outputs_is_too_many() {
    let rejected = rejected("material_reduction_10_outputs.json", &cx());
    let finding = one(
        &rejected.diagnostics,
        "Material Reduction Recipe: {} has too many outputs, the max is {}.",
        Severity::Error,
    );
    assert_eq!(args(finding), ["demo:ten_outputs", "9"]);
}

const TRANSFORM_TAG: &str =
    "Smithing Transform Recipe: {} has an invalid tag. They only supported one is {}.";

#[test]
fn smithing_transform_bad_tag_is_rejected() {
    let rejected = rejected("smithing_transform_bad_tag.json", &cx());
    let finding = one(&rejected.diagnostics, TRANSFORM_TAG, Severity::Error);
    assert_eq!(args(finding), ["demo:bad_tag", "smithing_table"]);
}

#[test]
fn smithing_transform_two_tags_is_rejected() {
    let rejected = rejected("smithing_transform_two_tags.json", &cx());
    one(&rejected.diagnostics, TRANSFORM_TAG, Severity::Error);
}

#[test]
fn smithing_transform_tagged_addition_loads() {
    let document = loaded("smithing_transform_tagged_addition.json", &cx());
    clean(document.diagnostics());
    let Recipe::SmithingTransform(recipe) = document.value() else {
        panic!()
    };
    assert_eq!(
        recipe.addition.item_name().unwrap().id(),
        "demo:mythril_ingot"
    );
    assert!(recipe.template.is_some());
}

#[test]
fn shaped_1_26_30_sulfur_loads_like_1_20_10() {
    // a copy of retail 1.26.45 `vanilla_1.26.30/recipes/sulfur.json`
    let document = loaded("shaped_1_26_30_sulfur.json", &cx_base_game());
    silent(document.diagnostics());
    let requirement = document.unlock_requirement(true).unwrap();
    assert_eq!(requirement.context, UnlockContext::None);
    assert_eq!(requirement.ingredients.len(), 1);
    assert_eq!(
        requirement.ingredients[0].item_name().unwrap().id(),
        "minecraft:sulfur_spike"
    );
}

#[test]
fn shaped_1_26_30_no_unlock_is_rejected() {
    let rejected = rejected("shaped_1_26_30_no_unlock.json", &cx());
    one(&rejected.diagnostics, UNLOCK_REQUIRED, Severity::Error);
}

#[test]
fn material_reduction_1_26_0_accepts_a_string_tags() {
    // a copy of retail 1.26.45 `chemistry_1.26.40/recipes/reducer_sulfur.json`
    let document = loaded("material_reduction_1_26_0.json", &cx_base_game());
    silent(document.diagnostics());
    assert_eq!(document.tags(), ["material_reducer"]);
    let Recipe::MaterialReduction(recipe) = document.value() else {
        panic!()
    };
    assert_eq!(recipe.output.as_ref().unwrap().as_slice().len(), 2);
}

#[test]
fn material_reduction_line_comments_load() {
    // a copy of retail 1.26.45 `chemistry/recipes/reducer_quartz_gem.json`, `//` comment included
    let document = loaded("material_reduction_line_comments.json", &cx_base_game());
    silent(document.diagnostics());
    assert_eq!(document.identifier().as_str(), "minecraft:quartz");
}

#[test]
fn smithing_trim_prec_1_20_world_is_rejected() {
    let rejected = rejected("smithing_trim_prec_1_20_world.json", &cx_world(1, 19));
    let finding = one(
        &rejected.diagnostics,
        "Smithing Trim Recipe: {} is invalid. Smithing Trim Recipes are only supported from version 1.20 onwards.",
        Severity::Error,
    );
    assert_eq!(args(finding), ["minecraft:smithing_armor_trim"]);
    // from 1.20 the same file loads
    let document = loaded("smithing_trim_prec_1_20_world.json", &cx_world(1, 20));
    silent(document.diagnostics());
}

#[test]
fn tags_empty_is_logged_and_the_recipe_still_parsed() {
    let document = loaded("tags_empty.json", &cx());
    let finding = one_kept(document.diagnostics(), "No tags specified");
    // By legacy name: the variant and the definitions that were it.
    assert!(matches!(finding.code.id(), "recipe/no_tags"));
    assert!(document.tags().is_empty());
}

#[test]
fn smithing_table_tag_on_shaped_is_logged_and_the_recipe_still_parsed() {
    let document = loaded("smithing_table_tag_on_shaped.json", &cx());
    one_kept(
        document.diagnostics(),
        "smithing_table tag is only supported by recipes of type recipe_smithing_transform and of type recipe_smithing_trim",
    );
    assert_eq!(document.tags(), ["smithing_table"]);
}

#[test]
fn payload_key_order_picks_shaped() {
    let document = loaded("payload_key_order.json", &cx());
    assert!(matches!(document.value(), Recipe::Shaped(_)));
    // the shapeless payload is not read, and kept for write-back
    assert!(kept(&document, &[], "minecraft:recipe_shapeless").is_some());
    assert!(
        document
            .diagnostics()
            .iter()
            // By legacy name: the variant and the definitions that were it.
            .any(|d| matches!(d.code.id(), "core/ignored_payload") && d.severity == Severity::Info)
    );
    clean(document.diagnostics());
}

#[test]
fn no_payload_key_is_skipped_as_having_no_identifier() {
    let rejected = rejected("no_payload_key.json", &cx());
    let finding = one(
        &rejected.diagnostics,
        "JSON: {} has no identifier",
        Severity::Error,
    );
    // By legacy name: the variant and the definitions that were it.
    assert!(matches!(finding.code.id(), "recipe/no_identifier"));
    assert_eq!(args(finding), ["recipes/no_payload_key.json"]);
}

#[test]
fn format_version_array_loads_as_1_20_10() {
    let document = loaded("format_version_array.json", &cx());
    clean(document.diagnostics());
    assert_eq!(document.version().effective, FormatVersion::new(1, 20, 10));
    assert_eq!(
        document.format_version(),
        Some(FormatVersion::new(1, 20, 10))
    );
    for name in [
        "format_version_array_two.json",
        "format_version_number.json",
    ] {
        let document = loaded(name, &cx());
        assert!(
            document
                .diagnostics()
                .iter()
                // By legacy name: the variant and the definitions that were it.
                .any(|d| matches!(d.code.id(), "recipe/missing_version_tag")),
            "{name}: {:#?}",
            document.diagnostics()
        );
        assert_eq!(document.version().effective, FormatVersion::new(1, 0, 0));
    }
}

#[test]
fn tags_missing_shaped_is_logged_and_registers_nothing() {
    let document = loaded("tags_missing_shaped.json", &cx());
    one_kept(document.diagnostics(), "No tags specified");
    // one recipe object per tag: none
    assert!(document.tags().is_empty());
}

#[test]
fn furnace_two_tags_is_one_recipe_object_per_tag() {
    let document = loaded("furnace_two_tags.json", &cx());
    clean(document.diagnostics());
    assert_eq!(document.tags(), ["furnace", "smoker"]);
}

#[test]
fn payload_key_null_is_skipped_for_the_next_key() {
    let document = loaded("payload_key_null.json", &cx());
    assert!(matches!(document.value(), Recipe::Shapeless(_)));
    assert_eq!(document.identifier().as_str(), "demo:after_null");
    clean(document.diagnostics());
}

#[test]
fn the_two_gates_are_the_two_predicates() {
    assert!(!unlock_required(FormatVersion::new(1, 20, 0)));
    assert!(unlock_required(FormatVersion::new(1, 20, 10)));
    assert!(unlock_required(FormatVersion::new(1, 26, 30)));
    // the unit tests of the binary: 1.19.21 and 1.19.22 legacy, 1.19.23 not
    assert!(input_count_allowed(&MinEngineVersion::new(1, 19, 21)));
    assert!(input_count_allowed(&MinEngineVersion::new(1, 19, 22)));
    assert!(!input_count_allowed(&MinEngineVersion::new(1, 19, 23)));
}

#[test]
fn a_result_wildcard_aux_reads_as_zero() {
    let ingredient = Ingredient::Name("minecraft:torch".to_owned());
    assert_eq!(ingredient.aux(), 32767);
    assert_eq!(ingredient.result_aux(), 0);
}
