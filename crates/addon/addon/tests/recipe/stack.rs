//! `bp.recipes()`: the stack rows of the fixtures and the walk.

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::families::recipe::{Recipe, RecipeNamespace};
use bedrock_addon::id::RecipeId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, Pack, PackOrigin, PackStack};

use super::common::*;

const MEV: [u16; 3] = [1, 21, 0];

fn fixture_pack(n: u128, dir: &str) -> Pack {
    pack(n, MEV, PackOrigin::Unknown, files_below(dir))
}

/// A stack of `packs`, top first.
fn stack(packs: Vec<Pack>) -> PackStack<Behavior> {
    PackStack::<Behavior>::of(packs, LoadOptions::default()).unwrap()
}

fn id(text: &str) -> RecipeId {
    RecipeId::new(text).unwrap()
}

#[test]
fn same_pack_duplicate_keeps_the_first_and_rejects_the_second() {
    let stack = stack(vec![fixture_pack(1, "same_pack_duplicate")]);
    let recipes = stack.recipes();
    // `a.json` (shaped) loads; `b.json` (furnace, another namespace) is still a duplicate
    let first = recipes
        .get(RecipeNamespace::Crafting, &id("demo:twice"))
        .unwrap();
    assert_eq!(first.origin().path.as_str(), "recipes/a.json");
    assert!(
        recipes
            .get(RecipeNamespace::Furnace, &id("demo:twice"))
            .is_none()
    );
    let rejected: Vec<_> = recipes.rejected().collect();
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].origin.path.as_str(), "recipes/b.json");
    let finding = one(
        &rejected[0].diagnostics,
        "JSON: {} has duplicate identifier in same pack: {}",
        Severity::Error,
    );
    assert!(matches!(
        finding.code,
        DiagnosticCode::DuplicateIdentifier { .. }
    ));
    assert_eq!(args(finding), ["recipes/b.json", "demo:twice"]);
}

#[test]
fn stack_override_the_top_pack_wins_silently() {
    let stack = stack(vec![
        fixture_pack(1, "stack_override/top"),
        fixture_pack(2, "stack_override/bottom"),
    ]);
    let recipes = stack.recipes();
    let effective = recipes.get(RecipeNamespace::Crafting, &id("x:y")).unwrap();
    assert_eq!(effective.origin().pack.index(), Some(0));
    let Recipe::Shaped(shaped) = effective.value() else {
        panic!()
    };
    assert_eq!(shaped.result.as_slice()[0].count(), 8);
    assert_eq!(recipes.rejected().count(), 0);
    assert!(
        recipes.diagnostics().next().is_none(),
        "{:#?}",
        recipes.diagnostics().collect::<Vec<_>>()
    );
}

#[test]
fn stack_override_cross_type_shapeless_shadows_shaped() {
    let stack = stack(vec![
        fixture_pack(1, "stack_override_cross_type/top"),
        fixture_pack(2, "stack_override_cross_type/bottom"),
    ]);
    let recipes = stack.recipes();
    let effective = recipes.get(RecipeNamespace::Crafting, &id("x:y")).unwrap();
    assert!(matches!(effective.value(), Recipe::Shapeless(_)));
    assert_eq!(recipes.len(), 1);
}

#[test]
fn stack_override_other_type_both_load() {
    let stack = stack(vec![
        fixture_pack(1, "stack_override_other_type/top"),
        fixture_pack(2, "stack_override_other_type/bottom"),
    ]);
    let recipes = stack.recipes();
    assert!(recipes.get(RecipeNamespace::Furnace, &id("x:y")).is_some());
    assert!(recipes.get(RecipeNamespace::Crafting, &id("x:y")).is_some());
    assert_eq!(recipes.len(), 2);
}

#[test]
fn a_borrowed_recipes_iterates_every_effective_recipe_in_namespace_order() {
    let stack = stack(vec![
        fixture_pack(1, "stack_override_other_type/top"),
        fixture_pack(2, "stack_override_other_type/bottom"),
    ]);
    let recipes = stack.recipes();
    let mut walked = 0;
    for (recipe_id, _document) in recipes {
        assert_eq!(recipe_id, &id("x:y"));
        walked += 1;
    }
    assert_eq!(walked, recipes.len());
    assert_eq!(recipes.into_iter().count(), recipes.iter().count());
}

#[test]
fn stack_override_failed_top_lets_the_lower_recipe_load() {
    let stack = stack(vec![
        fixture_pack(1, "stack_override_failed_top/top"),
        fixture_pack(2, "stack_override_failed_top/bottom"),
    ]);
    let recipes = stack.recipes();
    let effective = recipes.get(RecipeNamespace::Crafting, &id("x:y")).unwrap();
    assert_eq!(effective.origin().pack.index(), Some(1));
    let rejected: Vec<_> = recipes.rejected().collect();
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].origin.pack.index(), Some(0));
}

#[test]
fn tags_missing_override_reserves_the_identifier() {
    let stack = stack(vec![
        fixture_pack(1, "tags_missing_override/top"),
        fixture_pack(2, "tags_missing_override/bottom"),
    ]);
    let recipes = stack.recipes();
    let effective = recipes.get(RecipeNamespace::Crafting, &id("x:y")).unwrap();
    // A's recipe registers nothing (no tag), and B is still skipped
    assert_eq!(effective.origin().pack.index(), Some(0));
    assert!(effective.tags().is_empty());
    one_kept(effective.diagnostics(), "No tags specified");
}

#[test]
fn only_files_with_the_exact_json_extension_are_read() {
    let stack = stack(vec![fixture_pack(1, "extension")]);
    let recipes = stack.recipes();
    assert_eq!(recipes.len(), 1);
    assert!(
        recipes
            .get(RecipeNamespace::Crafting, &id("demo:lower_case_extension"))
            .is_some()
    );
    assert_eq!(recipes.rejected().count(), 0);
    assert!(recipes.diagnostics().next().is_none());
}

#[test]
fn a_base_game_pack_gets_the_base_game_unlock_default() {
    let files = vec![(
        "recipes/basegame.json".to_owned(),
        fixture("shaped_1_12_no_unlock_basegame.json"),
    )];
    let stack = stack(vec![pack(1, MEV, PackOrigin::Package, files)]);
    let recipes = stack.recipes();
    let recipe = recipes
        .get(RecipeNamespace::Crafting, &id("demo:basegame"))
        .unwrap();
    // a `Package` pack is a base-game pack
    assert_eq!(
        recipe.unlock_requirement(true).unwrap().context,
        bedrock_addon::families::recipe::UnlockContext::None
    );
}

#[test]
fn the_packs_min_engine_version_reaches_the_input_count_gate() {
    let files = vec![(
        "recipes/legacy.json".to_owned(),
        fixture("shaped_key_count_mev_1_19_20.json"),
    )];
    let legacy = stack(vec![pack(
        1,
        [1, 19, 20],
        PackOrigin::Unknown,
        files.clone(),
    )]);
    assert_eq!(legacy.recipes().len(), 1);
    let modern = stack(vec![pack(1, [1, 19, 30], PackOrigin::Unknown, files)]);
    assert_eq!(modern.recipes().len(), 0);
    assert_eq!(modern.recipes().rejected().count(), 1);
}

#[test]
fn subfolders_of_recipes_are_read() {
    let files = vec![(
        "recipes/sub/deep.json".to_owned(),
        fixture("shaped_1_12_no_unlock_thirdparty.json"),
    )];
    let stack = stack(vec![pack(1, MEV, PackOrigin::Unknown, files)]);
    assert_eq!(stack.recipes().len(), 1);
}

#[test]
fn the_accessor_caches_the_collection() {
    let stack = stack(vec![fixture_pack(1, "stack_override/top")]);
    assert!(std::ptr::eq(stack.recipes(), stack.recipes()));
    assert_eq!(stack.recipes().iter().count(), 1);
    assert!(!stack.recipes().is_empty());
}
