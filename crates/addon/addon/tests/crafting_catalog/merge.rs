//! The merge rows (`merge/m01` … `merge/m14`), through
//! `bp.crafting_catalog()`: each directory's packs are a stack (index = load order, vanilla
//! first), `merge/items.json` is the item registry, `expected_merge.json` the four categories.

use bedrock_addon::diagnostic::{DiagnosticDetail, Severity, codes};
use bedrock_addon::families::crafting_catalog;
use bedrock_addon::families::crafting_catalog::{CatalogKey, CategoryName};
use bedrock_addon::id::ItemId;
use bedrock_addon::load::LoadOptions;

use super::common::*;
use super::stacks::*;

/// Merges the fixture `name` and requires its expected catalogue and no finding.
#[track_caller]
fn merges_cleanly(name: &str) {
    let stack = fixture_stack(name);
    let merged = stack.crafting_catalog();
    assert_eq!(merged.value(), &expected_merge(name), "{name}");
    assert_eq!(found(merged), [], "{name}");
}

#[test]
fn m01_a_group_of_the_same_name_merges_and_keeps_its_first_icon() {
    merges_cleanly("m01_same_name_merges");
    let stack = fixture_stack("m01_same_name_merges");
    let merged = stack.crafting_catalog();
    let group = &merged.categories[0].groups[0];
    let identifier = group.group_identifier.as_ref().unwrap();
    assert_eq!(
        identifier.icon,
        Some(item("custom:my_icon")),
        "pack0's icon"
    );
    assert_eq!(group.items, [item("custom:item1"), item("custom:item2")]);
    // The group is pack0's, each item its own pack's.
    let pack = |key: &CatalogKey| {
        merged
            .origin(key)
            .map(|origin| origin.pack.name().to_owned())
    };
    let group_key = CatalogKey::Group {
        category: CategoryName::Construction,
        name: "custom:g".to_owned(),
    };
    assert_eq!(pack(&group_key).as_deref(), Some("pack0"));
    let item_key = |name: &str| CatalogKey::Item {
        id: ItemId::parse(name),
        aux: 0,
    };
    assert_eq!(pack(&item_key("custom:item1")).as_deref(), Some("pack0"));
    assert_eq!(pack(&item_key("custom:item2")).as_deref(), Some("pack1"));
    assert_eq!(pack(&item_key("custom:item3")), None);
}

#[test]
fn m02_loose_groups_never_merge() {
    merges_cleanly("m02_anon_never_merge");
    let stack = fixture_stack("m02_anon_never_merge");
    let nature = &stack.crafting_catalog().categories[1];
    assert_eq!(nature.category_name, CategoryName::Nature);
    assert_eq!(
        nature.groups,
        [loose(&["custom:item1"]), loose(&["custom:item2"])]
    );
}

#[test]
fn m03_a_new_group_is_appended_and_a_known_one_keeps_its_place() {
    merges_cleanly("m03_new_group_appended");
    let stack = fixture_stack("m03_new_group_appended");
    let names: Vec<&str> = stack.crafting_catalog().categories[3]
        .groups
        .iter()
        .map(|group| group.group_identifier.as_ref().unwrap().name.as_str())
        .collect();
    assert_eq!(names, ["custom:a", "custom:b", "custom:c"]);
}

#[test]
fn m04_the_same_name_in_two_categories_is_two_groups() {
    merges_cleanly("m04_same_name_other_category");
    let stack = fixture_stack("m04_same_name_other_category");
    let merged = stack.crafting_catalog();
    assert_eq!(merged.categories[0].groups.len(), 1);
    assert_eq!(merged.categories[3].groups.len(), 1);
}

#[test]
fn m05_a_duplicate_is_dropped_with_vanilla_s_error_and_the_first_placement_wins() {
    let stack = fixture_stack("m05_duplicate_first_wins");
    let merged = stack.crafting_catalog();
    assert_eq!(merged.value(), &expected_merge("m05_duplicate_first_wins"));
    assert_eq!(
        found(merged),
        [(
            "crafting_catalog/duplicate_item".to_owned(),
            Severity::Error,
            "pack1".to_owned(),
            "You can only add an item to the crafting item catalog once. Found duplicate minecraft:stone"
                .to_owned()
        )]
    );
    let finding = merged.diagnostics().next().unwrap();
    assert!(finding.is(&crafting_catalog::codes::DUPLICATE_ITEM));
    assert_eq!(
        finding
            .location
            .path
            .as_ref()
            .map(bedrock_addon::storage::PackPath::as_str),
        Some(PATH)
    );
    assert_eq!(
        finding.location.json_path.to_string(),
        "minecraft:crafting_items_catalog/categories/0/groups/0/items/0"
    );
}

#[test]
fn m06_duplicates_in_one_file_compare_the_identifier_and_the_aux_value() {
    let stack = fixture_stack("m06_duplicate_same_file");
    let merged = stack.crafting_catalog();
    assert_eq!(merged.value(), &expected_merge("m06_duplicate_same_file"));
    let texts: Vec<String> = found(merged)
        .into_iter()
        .map(|(code, severity, pack, text)| {
            assert_eq!(
                (code.as_str(), severity, pack.as_str()),
                ("crafting_catalog/duplicate_item", Severity::Error, "pack0")
            );
            text
        })
        .collect();
    assert_eq!(
        texts,
        [
            "You can only add an item to the crafting item catalog once. Found duplicate custom:item1",
            "You can only add an item to the crafting item catalog once. Found duplicate custom:item2:0",
        ]
    );
}

// With `item` alone a non-vanilla name that is no item is kept (it may be a
// pack's block), so the drop this test needs takes the block registry in view.
#[cfg(feature = "block")]
#[test]
fn m07_an_unknown_item_is_dropped_with_the_validators_error() {
    // The lookup drops it without a message; the game's link pass logs it at the content
    // log's error level.
    let stack = fixture_stack("m07_unknown_item");
    let merged = stack.crafting_catalog();
    assert_eq!(merged.value(), &expected_merge("m07_unknown_item"));
    let findings: Vec<_> = merged.diagnostics().collect();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].is(&crafting_catalog::codes::MISSING_ITEM));
    assert_eq!(
        findings[0].code.to_string(),
        "-> categories[0] -> groups[0] -> items[1]:  Missing referenced asset custom:not_a_valid_item"
    );
    assert_eq!(findings[0].severity, Severity::Error);
    assert_eq!(
        findings[0].location.json_path.to_string(),
        "minecraft:crafting_items_catalog/categories/0/groups/0/items/1"
    );
    assert!(matches!(
        findings[0].detail.as_deref(),
        Some(DiagnosticDetail::Reference { kind: "item", id, hint: None }) if id == "custom:not_a_valid_item"
    ));
}

#[test]
fn m08_a_bad_aux_value_is_dropped_with_vanilla_s_error() {
    let stack = fixture_stack("m08_bad_aux");
    let merged = stack.crafting_catalog();
    assert_eq!(merged.value(), &expected_merge("m08_bad_aux"));
    assert_eq!(
        found(merged),
        [(
            "crafting_catalog/invalid_aux".to_owned(),
            Severity::Error,
            "pack0".to_owned(),
            "Invalid aux value in item ID \"custom:item3:not_an_aux_value\"".to_owned()
        )]
    );
}

#[test]
fn m09_an_icon_keeps_its_aux_value() {
    merges_cleanly("m09_icon_aux");
    let stack = fixture_stack("m09_icon_aux");
    let merged = stack.crafting_catalog();
    let icon = merged.categories[2].groups[0]
        .group_identifier
        .as_ref()
        .and_then(|identifier| identifier.icon.as_ref())
        .unwrap();
    assert_eq!(icon.name.as_str(), "custom:my_icon:1");
    assert_eq!(icon.name.aux(), Some(1));
}

/// The item is placed where the catalogue says; the two warnings about its own
/// `menu_category` are the link pass's.
#[test]
fn m10_an_item_defined_for_another_category_and_group_is_placed() {
    merges_cleanly("m10_category_group_warnings");
}

#[test]
fn m11_a_minecraft_item_in_another_group_raises_nothing() {
    merges_cleanly("m11_minecraft_no_warning");
}

#[test]
fn m12_without_a_base_catalogue_the_add_on_files_merge_with_one_warning_deviation() {
    // vanilla = "debug assert; no catalog loaded"
    let stack = fixture_stack("m12_no_base_catalog");
    let merged = stack.crafting_catalog();
    assert_eq!(merged.value(), &expected_merge("m12_no_base_catalog"));
    let findings: Vec<_> = merged.diagnostics().collect();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].is(&codes::UNRESOLVED_REFERENCE));
    assert_eq!(findings[0].severity, Severity::Info);
    assert!(matches!(
        findings[0].detail.as_deref(),
        Some(DiagnosticDetail::Reference {
            kind: "base crafting item catalog",
            ..
        })
    ));
}

#[test]
fn m13_versioned_vanilla_slices_replace_each_other() {
    merges_cleanly("m13_versioned_slices_replace");
    let stack = fixture_stack("m13_versioned_slices_replace");
    let merged = stack.crafting_catalog();
    // Only `vanilla_1.21.110` (pack1) and the add-on (pack3) were read.
    let layers: Vec<&str> = merged
        .layers()
        .iter()
        .map(|layer| layer.origin().pack.name())
        .collect();
    assert_eq!(layers, ["pack3", "pack1"]);
    let construction: Vec<&str> = merged.categories[0]
        .groups
        .iter()
        .map(|group| group.group_identifier.as_ref().unwrap().name.as_str())
        .collect();
    assert_eq!(construction, ["custom:h"]);
}

/// `vanilla_1.26.40` and `experimental_y_2026_drop_3` of the retail 1.26.45 build, verbatim;
/// the expected catalogue is a reference merge of the two files.
#[test]
fn m14_an_experiment_pack_joins_vanilla_s_groups_by_name() {
    let packs = [
        vanilla("pack0").catalog(&fixture(
            "merge/m14_experiment_joins_vanilla_groups/pack0_vanilla.json",
        )),
        addon("pack1").catalog(&fixture(
            "merge/m14_experiment_joins_vanilla_groups/pack1.json",
        )),
    ];
    let stack = stack_of(&packs, LoadOptions::default());
    let merged = stack.crafting_catalog();
    assert_eq!(found(merged), []);
    assert_eq!(
        merged.value(),
        &expected_merge("m14_experiment_joins_vanilla_groups")
    );
    let groups: Vec<usize> = merged.categories.iter().map(|c| c.groups.len()).collect();
    let items: Vec<usize> = merged
        .categories
        .iter()
        .map(|c| c.groups.iter().map(|g| g.items.len()).sum())
        .collect();
    // 119 vanilla groups and one new; 1700 vanilla items and the pack's 73.
    assert_eq!(groups, [25, 27, 29, 39]);
    assert_eq!(items.iter().sum::<usize>(), 1773);
    let last = merged.categories[3].groups.last().unwrap();
    assert_eq!(
        last.group_identifier.as_ref().unwrap().name,
        "minecraft:itemGroup.name.cushion"
    );
    // The vanilla file alone has the other 119, in the same places.
    let alone = stack_of(&packs[..1], LoadOptions::default());
    let vanilla_names =
        |catalog: &bedrock_addon::families::crafting_catalog::CraftingCatalog| -> Vec<String> {
            catalog
                .categories
                .iter()
                .flat_map(|c| &c.groups)
                .filter_map(|g| g.group_identifier.as_ref().map(|i| i.name.clone()))
                .filter(|name| name != "minecraft:itemGroup.name.cushion")
                .collect()
        };
    assert_eq!(
        vanilla_names(merged.value()),
        vanilla_names(alone.crafting_catalog().value())
    );
}

#[test]
fn cat01_as_the_base_gives_four_categories_in_vanilla_s_order() {
    let stack = stack_of(
        &[
            vanilla("vanilla").catalog(&fixture("cat01_all_categories.json")),
            registry_pack(),
        ],
        registry_options(),
    );
    let merged = stack.crafting_catalog();
    assert_eq!(found(merged), []);
    let order: Vec<CategoryName> = merged.categories.iter().map(|c| c.category_name).collect();
    assert_eq!(order, CategoryName::ALL);
    assert_eq!(
        merged.categories[1].groups,
        [named("custom:plants", None, &[])]
    );
    assert_eq!(merged.categories[2].groups[0].items, [item("custom:item3")]);
}
