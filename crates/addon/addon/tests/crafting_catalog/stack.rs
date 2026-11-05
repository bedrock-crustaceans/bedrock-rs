//! Which files of a stack `bp.crafting_catalog()` reads (S1 and
//! S2), what the item registry of the merge is (M1, M6), and the accessor itself.

use bedrock_addon::diagnostic::{DiagnosticCode, DiagnosticDetail, Severity};
use bedrock_addon::families::crafting_catalog::{CategoryName, CraftingCatalog};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, PackOrigin, PackStack};

use super::common::*;
use super::stacks::*;

/// A catalogue placing `items` in the construction group `group`.
fn construction(group: &str, items: &[&str]) -> String {
    in_category("construction", group, None, items)
}

/// A catalogue placing `items` in the group `group` (with `icon`) of `category`.
fn in_category(category: &str, group: &str, icon: Option<&str>, items: &[&str]) -> String {
    let icon = icon.map_or_else(String::new, |icon| format!(r#", "icon": "{icon}""#));
    let items: Vec<String> = items.iter().map(|item| format!("\"{item}\"")).collect();
    format!(
        r#"{{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {{"categories": [{{"category_name": "{category}", "groups": [{{"group_identifier": {{"name": "{group}"{icon}}}, "items": [{}]}}]}}]}}}}"#,
        items.join(", ")
    )
}

/// The names of the named groups of `category`.
fn groups(catalog: &CraftingCatalog, category: CategoryName) -> Vec<String> {
    catalog.categories[category as usize]
        .groups
        .iter()
        .filter_map(|group| group.group_identifier.as_ref().map(|i| i.name.clone()))
        .collect()
}

/// The packs the merged catalogue read, top first.
fn layers(stack: &PackStack<Behavior>) -> Vec<String> {
    stack
        .crafting_catalog()
        .layers()
        .iter()
        .map(|layer| layer.origin().pack.name().to_owned())
        .collect()
}

/// Whether the merge warned that the stack has no base catalogue.
fn no_base_warnings(stack: &PackStack<Behavior>) -> usize {
    stack
        .crafting_catalog()
        .diagnostics()
        .filter(|f| {
            matches!(f.code, DiagnosticCode::UnresolvedReference { .. })
                && matches!(
                    f.detail.as_deref(),
                    Some(DiagnosticDetail::Reference {
                        kind: "base crafting item catalog",
                        ..
                    })
                )
        })
        .inspect(|f| assert_eq!(f.severity, Severity::Info))
        .count()
}

fn options() -> LoadOptions {
    registry_options()
}

#[test]
fn a_stack_without_any_catalogue_file_is_four_empty_categories_and_no_finding() {
    let stack = stack_of(&[vanilla("vanilla"), addon("a")], options());
    let merged = stack.crafting_catalog();
    assert_eq!(merged.categories.len(), 4);
    let order: Vec<CategoryName> = merged.categories.iter().map(|c| c.category_name).collect();
    assert_eq!(order, CategoryName::ALL);
    assert!(merged.categories.iter().all(|c| c.groups.is_empty()));
    assert_eq!(merged.diagnostics().count(), 0);
    assert!(merged.layers().is_empty());
    // The same collection on every call.
    assert!(std::ptr::eq(merged, stack.crafting_catalog()));
}

#[test]
fn s1_a_vanilla_pack_without_the_file_is_no_base_deviation() {
    let stack = stack_of(
        &[
            vanilla("vanilla"),
            addon("a").catalog(&construction("custom:a", &["minecraft:stone"])),
        ],
        options(),
    );
    assert_eq!(layers(&stack), ["a"]);
    assert_eq!(no_base_warnings(&stack), 1);
    assert_eq!(
        groups(stack.crafting_catalog(), CategoryName::Construction),
        ["custom:a"]
    );
}

#[test]
fn s1_the_warning_is_raised_once_whatever_the_number_of_files_deviation() {
    let stack = stack_of(
        &[
            addon("a").catalog(&construction("custom:a", &[])),
            addon("b").catalog(&construction("custom:b", &[])),
        ],
        options(),
    );
    assert_eq!(layers(&stack), ["b", "a"]);
    assert_eq!(no_base_warnings(&stack), 1);
    assert_eq!(stack.crafting_catalog().diagnostics().count(), 1);
    assert_eq!(
        groups(stack.crafting_catalog(), CategoryName::Construction),
        ["custom:a", "custom:b"]
    );
}

/// The base is a pack that is both a base-game pack and has the vanilla UUID.
#[test]
fn s1_the_base_needs_the_vanilla_uuid_and_the_base_game_origin() {
    let catalog = construction("custom:a", &[]);
    // The vanilla UUID on a pack the user installed.
    let user = vanilla("lookalike");
    let user = bedrock_addon_test_spec(user, PackOrigin::User).catalog(&catalog);
    let stack = stack_of(&[user], options());
    assert_eq!(no_base_warnings(&stack), 1);
    // A base-game pack with another UUID (an experimental pack, chemistry).
    let other = bedrock_addon_test_spec(addon("chemistry"), PackOrigin::Package).catalog(&catalog);
    let stack = stack_of(&[other], options());
    assert_eq!(no_base_warnings(&stack), 1);
    // Both: the base.
    let stack = stack_of(&[vanilla("vanilla").catalog(&catalog)], options());
    assert_eq!(no_base_warnings(&stack), 0);
    assert_eq!(stack.crafting_catalog().diagnostics().count(), 0);
}

/// `spec` with the origin `origin`.
fn bedrock_addon_test_spec(mut spec: Spec, origin: PackOrigin) -> Spec {
    spec.origin = origin;
    spec
}

#[test]
fn packs_below_the_base_are_never_read() {
    let stack = stack_of(
        &[
            // Below the base: an add-on with a valid file and a vanilla slice with a broken one.
            addon("below").catalog(&construction("custom:below", &[])),
            vanilla("old").catalog("{ not json"),
            vanilla("base").catalog(&construction("custom:base", &[])),
            addon("above").catalog(&construction("custom:above", &[])),
        ],
        options(),
    );
    assert_eq!(layers(&stack), ["above", "base"]);
    let merged = stack.crafting_catalog();
    assert_eq!(
        merged.diagnostics().count(),
        0,
        "{:#?}",
        merged.diagnostics().collect::<Vec<_>>()
    );
    assert_eq!(
        groups(merged, CategoryName::Construction),
        ["custom:base", "custom:above"]
    );
}

#[test]
fn the_topmost_vanilla_slice_with_the_file_is_the_base() {
    let stack = stack_of(
        &[
            vanilla("v1").catalog(&construction("custom:v1", &[])),
            vanilla("v2").catalog(&construction("custom:v2", &[])),
            vanilla("v3"),
            vanilla("v4"),
        ],
        options(),
    );
    assert_eq!(layers(&stack), ["v2"]);
    assert_eq!(no_base_warnings(&stack), 0);
}

/// the walk down from the topmost vanilla pack stops at the first pack that
/// has the file, vanilla or not; the vanilla packs below it are not read. No finding.
#[test]
fn s2_a_non_vanilla_pack_between_vanilla_packs_can_be_the_base() {
    let stack = stack_of(
        &[
            vanilla("v1").catalog(&construction("custom:v1", &[])),
            addon("between").catalog(&construction("custom:between", &[])),
            vanilla("v2"),
            addon("top").catalog(&construction("custom:top", &[])),
        ],
        options(),
    );
    assert_eq!(layers(&stack), ["top", "between"]);
    assert_eq!(stack.crafting_catalog().diagnostics().count(), 0);
    assert_eq!(
        groups(stack.crafting_catalog(), CategoryName::Construction),
        ["custom:between", "custom:top"]
    );
}

/// The walk starts at the topmost vanilla pack, not at the top of the stack: an add-on above
/// it with the file is not the base.
#[test]
fn an_add_on_above_the_vanilla_packs_is_not_the_base() {
    let stack = stack_of(
        &[
            vanilla("v1").catalog(&construction("custom:v1", &[])),
            addon("top").catalog(&construction("custom:top", &[])),
        ],
        options(),
    );
    assert_eq!(layers(&stack), ["top", "v1"]);
    assert_eq!(
        groups(stack.crafting_catalog(), CategoryName::Construction),
        ["custom:v1", "custom:top"]
    );
}

#[test]
fn a_base_file_that_fails_to_load_is_still_the_base() {
    let stack = stack_of(
        &[
            vanilla("v0").catalog(&construction("custom:v0", &[])),
            vanilla("v1").catalog(&construction("custom:v1", &[]).replace("1.21.60", "1.21.50")),
            addon("top").catalog(&construction("custom:top", &[])),
        ],
        options(),
    );
    let merged = stack.crafting_catalog();
    assert_eq!(layers(&stack), ["top"]);
    assert_eq!(no_base_warnings(&stack), 0);
    let found = found(merged);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        (found[0].0.as_str(), found[0].2.as_str()),
        ("UnsupportedVersion", "v1")
    );
    assert_eq!(groups(merged, CategoryName::Construction), ["custom:top"]);
}

#[test]
fn a_file_that_fails_to_load_leaves_the_other_packs_alone() {
    let stack = stack_of(
        &[
            vanilla("base").catalog(&construction("custom:base", &[])),
            addon("broken").catalog(
                r#"{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {}}"#,
            ),
            addon("top").catalog(&construction("custom:top", &[])),
        ],
        options(),
    );
    let merged = stack.crafting_catalog();
    assert_eq!(layers(&stack), ["top", "base"]);
    let found = found(merged);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        (found[0].0.as_str(), found[0].1, found[0].2.as_str()),
        ("MissingRequiredField", Severity::Error, "broken")
    );
    assert_eq!(
        groups(merged, CategoryName::Construction),
        ["custom:base", "custom:top"]
    );
}

#[test]
fn only_the_one_path_is_a_catalogue() {
    let catalog = construction("custom:a", &[]);
    let pack = vanilla("vanilla")
        .file("item_catalog/other.json", catalog.clone())
        .file("item_catalog/Crafting_Item_Catalog.json", catalog.clone())
        .file(
            "item_catalog/sub/crafting_item_catalog.json",
            catalog.clone(),
        )
        .file("crafting_item_catalog.json", catalog);
    let stack = stack_of(&[pack], options());
    assert!(layers(&stack).is_empty());
    assert_eq!(stack.crafting_catalog().diagnostics().count(), 0);
}

// ---- the item registry  ---------------------------------------------

/// An item definition file for `identifier`.
#[cfg(feature = "block")]
fn item_file(identifier: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.60", "minecraft:item": {{"description": {{"identifier": "{identifier}"}}, "components": {{"minecraft:max_stack_size": 64}}}}}}"#
    )
}

/// The unresolved names the merge reported, with the path of each.
fn unresolved(stack: &PackStack<Behavior>) -> Vec<(String, String)> {
    stack
        .crafting_catalog()
        .diagnostics()
        .filter_map(|f| match f.detail.as_deref() {
            Some(DiagnosticDetail::Reference {
                kind: "item", id, ..
            }) => {
                // The game's link pass holds every catalogue item and logs a miss at the
                // content log's error level.
                assert!(matches!(f.code, DiagnosticCode::UnresolvedReference { .. }));
                let text = f.code.to_string();
                assert!(
                    text.ends_with(&format!(":  Missing referenced asset {id}"))
                        && text.starts_with("-> categories["),
                    "{text}"
                );
                assert_eq!(f.severity, Severity::Error);
                Some((id.clone(), f.location.json_path.to_string()))
            }
            _ => None,
        })
        .collect()
}

// With `item` alone a non-vanilla name that is no item is kept (it may be a
// pack's block), so the drop this test needs takes the block registry in view.
#[cfg(feature = "block")]
#[test]
fn m1_an_item_any_pack_of_the_stack_defines_is_registered() {
    let stack = stack_of(
        &[
            // Below the base: its catalogue is not read, its items are registered.
            addon("below").file("items/low.json", item_file("custom:low")),
            vanilla("base").catalog(&construction(
                "custom:g",
                &[
                    "custom:low",
                    "custom:high",
                    "custom:high:2",
                    "custom:nowhere",
                ],
            )),
            addon("above").file("items/deep/high.json", item_file("custom:high")),
        ],
        options(),
    );
    assert_eq!(
        unresolved(&stack),
        [(
            "custom:nowhere".to_owned(),
            "minecraft:crafting_items_catalog/categories/0/groups/0/items/3".to_owned()
        )]
    );
    let merged = stack.crafting_catalog();
    assert_eq!(
        merged.categories[0].groups[0].items,
        [
            item("custom:low"),
            item("custom:high"),
            item("custom:high:2")
        ]
    );
}

// With `item` alone a non-vanilla name that is no item is kept (it may be a
// pack's block), so the drop this test needs takes the block registry in view.
#[cfg(feature = "block")]
#[test]
fn m1_an_item_whose_definition_fails_to_load_is_not_registered() {
    let stack = stack_of(
        &[vanilla("base")
            .catalog(&construction("custom:g", &["custom:broken"]))
            .file("items/broken.json", r#"{"format_version": "1.21.60", "minecraft:item": {"description": {"identifier": "custom:broken"}, "components": 5}}"#)],
        options(),
    );
    assert_eq!(unresolved(&stack).len(), 1);
    assert!(
        stack.crafting_catalog().categories[0].groups[0]
            .items
            .is_empty()
    );
}

/// A block has an item of its own name: the blocks the stack's packs define count as
/// registered, as `bp.blocks()` has them.
#[cfg(feature = "block")]
#[test]
fn m1_a_block_of_the_stack_is_registered() {
    let block = r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "custom:ore"}, "components": {}}}"#;
    let stack = stack_of(
        &[
            vanilla("base").catalog(&construction("custom:g", &["custom:ore", "custom:ore2"])),
            addon("blocks").file("blocks/ore.json", block).file(
                "blocks/notes.txt",
                block.replace("custom:ore", "custom:ore2"),
            ),
        ],
        options(),
    );
    assert_eq!(unresolved(&stack).len(), 1, "{:#?}", unresolved(&stack));
    assert_eq!(unresolved(&stack)[0].0, "custom:ore2");
    assert_eq!(
        stack.crafting_catalog().categories[0].groups[0].items,
        [item("custom:ore")]
    );
}

/// The three block files the block family rejects: a `minecraft:` block of an add-on, a
/// `format_version` below the block floor, and components that are no object.
#[cfg(feature = "block")]
fn rejected_block_files() -> [(&'static str, String); 3] {
    let block = |version: &str, identifier: &str, components: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:block": {{"description": {{"identifier": "{identifier}"}}, "components": {components}}}}}"#
        )
    };
    [
        (
            "minecraft:fake_ore",
            block("1.21.60", "minecraft:fake_ore", "{}"),
        ),
        ("custom:old", block("1.10.0", "custom:old", "{}")),
        ("custom:broken", block("1.21.60", "custom:broken", "5")),
    ]
}

/// A stack whose add-on defines the one block `file` and whose base catalogue places `name`.
#[cfg(feature = "block")]
fn stack_with_block(name: &str, file: String) -> PackStack<Behavior> {
    // Without built-ins of its own the host knows no vanilla name either.
    struct NoBuiltIns;
    impl bedrock_addon::load::BuiltIns for NoBuiltIns {
        fn contains(&self, _: &'static str, _: &str) -> bool {
            false
        }
    }
    stack_of(
        &[
            vanilla("base").catalog(&construction("custom:g", &[name])),
            addon("blocks").file("blocks/block.json", file),
        ],
        LoadOptions {
            built_ins: Some(std::sync::Arc::new(NoBuiltIns)),
            ..LoadOptions::default()
        },
    )
}

/// A block is registered exactly when the block family loads it (`bp.blocks()`): a block file
/// it rejects registers nothing, and the catalogue drops its name with the M1 warning. The
/// catalogue's `combine` asks the stack's block registry (`StackCx::registered`).
#[cfg(feature = "block")]
#[track_caller]
fn rejected_block_is_not_registered(index: usize) {
    let (name, file) = rejected_block_files().into_iter().nth(index).unwrap();
    let stack = stack_with_block(name, file);
    assert!(
        stack
            .blocks()
            .get(&bedrock_addon::id::BlockId::parse(name))
            .is_none(),
        "{name} is not a block of the stack"
    );
    let names: Vec<String> = unresolved(&stack).into_iter().map(|(n, _)| n).collect();
    assert_eq!(names, [name], "{name} is unresolved");
    assert!(
        stack.crafting_catalog().categories[0].groups[0]
            .items
            .is_empty(),
        "{name} is dropped"
    );
}

#[cfg(feature = "block")]
#[test]
fn m1_a_minecraft_block_of_an_add_on_is_not_registered() {
    rejected_block_is_not_registered(0);
}

#[cfg(feature = "block")]
#[test]
fn m1_a_block_below_the_block_floor_is_not_registered() {
    rejected_block_is_not_registered(1);
}

#[cfg(feature = "block")]
#[test]
fn m1_a_block_whose_components_are_no_object_is_not_registered() {
    rejected_block_is_not_registered(2);
}

/// Without the `block` feature no collection of the stack registers blocks, so the catalogue
/// cannot know a pack's blocks: a name that is no item and could be a pack's block is kept,
/// with no warning. A `minecraft:` name
/// can be no pack's block and is still judged by the built-in items.
#[cfg(not(feature = "block"))]
#[test]
fn m1_without_the_block_feature_a_name_that_may_be_a_pack_s_block_is_kept() {
    let block = r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "custom:ore"}, "components": {}}}"#;
    let stack = stack_of(
        &[
            vanilla("base").catalog(&construction(
                "custom:g",
                &[
                    "custom:ore",
                    "custom:nothing",
                    "minecraft:stone",
                    "minecraft:no_such_item",
                ],
            )),
            addon("blocks").file("blocks/ore.json", block),
        ],
        LoadOptions::default(),
    );
    let names: Vec<String> = unresolved(&stack).into_iter().map(|(n, _)| n).collect();
    assert_eq!(names, ["minecraft:no_such_item"]);
    assert_eq!(
        stack.crafting_catalog().categories[0].groups[0].items,
        [
            item("custom:ore"),
            item("custom:nothing"),
            item("minecraft:stone")
        ]
    );
}

/// Names resolve with case, as vanilla's lookup of a registered name does: a block file
/// naming `custom:Mixed` registers that spelling only.
#[cfg(feature = "block")]
#[test]
fn m1_a_block_identifier_is_compared_with_case() {
    let block = r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "custom:Mixed"}, "components": {}}}"#;
    let stack = stack_of(
        &[
            vanilla("base").catalog(&construction(
                "custom:g",
                &["custom:mixed", "custom:Mixed", "custom:MIXED"],
            )),
            addon("blocks").file("blocks/mixed.json", block),
        ],
        options(),
    );
    let names: Vec<String> = unresolved(&stack).into_iter().map(|(n, _)| n).collect();
    assert_eq!(names, ["custom:mixed", "custom:MIXED"]);
    assert_eq!(
        stack.crafting_catalog().categories[0].groups[0].items,
        [item("custom:Mixed")]
    );
}

/// The same for the items of the stack and for the built-in items.
// With `item` alone a non-vanilla name that is no item is kept (it may be a
// pack's block), so the drop this test needs takes the block registry in view.
#[cfg(feature = "block")]
#[test]
fn m1_an_item_identifier_is_compared_with_case() {
    let stack = stack_of(
        &[vanilla("base")
            .catalog(&construction(
                "custom:g",
                &[
                    "custom:Thing",
                    "custom:thing",
                    "minecraft:Stone",
                    "Minecraft:stone",
                ],
            ))
            .file("items/thing.json", item_file("custom:thing"))],
        options(),
    );
    let names: Vec<String> = unresolved(&stack).into_iter().map(|(n, _)| n).collect();
    assert_eq!(
        names,
        ["custom:Thing", "minecraft:Stone", "Minecraft:stone"]
    );
    assert_eq!(
        stack.crafting_catalog().categories[0].groups[0].items,
        [item("custom:thing")]
    );
    // Without a host, the crate's own table is read with case too.
    let stack = stack_of(
        &[vanilla("base").catalog(&construction(
            "custom:g",
            &["minecraft:Stone", "minecraft:stone"],
        ))],
        LoadOptions::default(),
    );
    let names: Vec<String> = unresolved(&stack).into_iter().map(|(n, _)| n).collect();
    assert_eq!(names, ["minecraft:Stone"]);
}

// With `item` alone a non-vanilla name that is no item is kept (it may be a
// pack's block), so the drop this test needs takes the block registry in view.
#[cfg(feature = "block")]
#[test]
fn m1_the_host_s_built_in_items_answer_for_vanilla() {
    struct Host;
    impl bedrock_addon::load::BuiltIns for Host {
        fn contains(&self, kind: &'static str, id: &str) -> bool {
            (kind, id) == ("item", "host:item") || (kind, id) == ("block", "host:block")
        }
    }
    let stack = stack_of(
        &[vanilla("base").catalog(&construction(
            "custom:g",
            &["host:item", "host:block", "host:other", "minecraft:stone"],
        ))],
        LoadOptions {
            built_ins: Some(std::sync::Arc::new(Host)),
            ..LoadOptions::default()
        },
    );
    let names: Vec<String> = unresolved(&stack)
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    // The host is asked for items (a block that has an item is one of its items), and it is
    // asked instead of the crate's own table: it does not know `minecraft:stone`.
    assert_eq!(names, ["host:block", "host:other", "minecraft:stone"]);
}

#[test]
fn m1_without_built_ins_the_crate_s_vanilla_identifiers_answer() {
    let stack = stack_of(
        &[vanilla("base").catalog(&construction(
            "custom:g",
            &[
                "minecraft:stone",
                "minecraft:apple",
                "minecraft:arrow:6",
                "minecraft:not_an_item",
                // A technical block: the game gives every registered block but air an item of
                // its name.
                "minecraft:fire",
                // An education item and a Drop 4 item the game registers in 1.26.60.
                "minecraft:element_1",
                "minecraft:frostbite_spawn_egg",
            ],
        ))],
        LoadOptions::default(),
    );
    let names: Vec<String> = unresolved(&stack)
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(names, ["minecraft:not_an_item"]);
}

/// A beta item document (`"format_version": "beta"`) registers its item too.
// With `item` alone a non-vanilla name that is no item is kept (it may be a
// pack's block), so the drop this test needs takes the block registry in view.
#[cfg(feature = "block")]
#[test]
fn m1_a_beta_item_is_registered() {
    let beta = item_file("custom:beta_item").replace("\"1.21.60\"", "\"beta\"");
    let stack = stack_of(
        &[vanilla("base")
            .catalog(&construction(
                "custom:g",
                &["custom:beta_item", "custom:other"],
            ))
            .file("items/beta.json", beta)],
        // a JSON beta item needs the Beta APIs, in a base-game pack too
        LoadOptions {
            beta_apis: true,
            ..options()
        },
    );
    let names: Vec<String> = unresolved(&stack)
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(names, ["custom:other"]);
}

// With `item` alone a non-vanilla name that is no item is kept (it may be a
// pack's block), so the drop this test needs takes the block registry in view.
#[cfg(feature = "block")]
#[test]
fn m1_an_unknown_item_is_not_placed_so_a_second_one_is_no_duplicate() {
    let stack = stack_of(
        &[vanilla("base").catalog(&construction(
            "custom:g",
            &[
                "custom:nowhere",
                "custom:nowhere",
                "custom:item9:x",
                "custom:item9:x",
            ],
        ))],
        options(),
    );
    let found = found(stack.crafting_catalog());
    let codes: Vec<&str> = found.iter().map(|f| f.0.as_str()).collect();
    // The game's link-pass error for each miss.
    assert_eq!(
        codes,
        [
            "UnresolvedReference",
            "UnresolvedReference",
            "InvalidValue",
            "InvalidValue"
        ]
    );
}

/// A bad aux value is reported before the name is looked up: one finding, vanilla's.
#[test]
fn m2_a_bad_aux_value_on_an_unknown_name_is_one_error() {
    for name in [
        "custom:nowhere:x",
        "custom:nowhere:",
        "custom:nowhere:1.5",
        "a:b:c:d",
        "::x",
    ] {
        let stack = stack_of(
            &[vanilla("base").catalog(&construction("custom:g", &[name]))],
            options(),
        );
        let found = found(stack.crafting_catalog());
        assert_eq!(found.len(), 1, "{name}: {found:#?}");
        assert_eq!(found[0].0, "InvalidValue", "{name}");
        assert_eq!(found[0].1, Severity::Error);
        assert_eq!(
            found[0].3,
            format!("Invalid aux value in item ID \"{name}\"")
        );
    }
}

#[test]
fn m2_an_aux_value_is_a_16_bit_integer() {
    let stack = stack_of(
        &[
            vanilla("base").catalog(&construction(
                "custom:g",
                &[
                    "custom:item1:0",
                    "custom:item1:32767",
                    "custom:item1:-32768",
                    "custom:item1:32768",
                    "custom:item1:-32769",
                ],
            )),
            registry_pack(),
        ],
        options(),
    );
    let merged = stack.crafting_catalog();
    assert_eq!(
        merged.categories[0].groups[0].items,
        [
            item("custom:item1:0"),
            item("custom:item1:32767"),
            item("custom:item1:-32768")
        ]
    );
    let found = found(merged);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found.iter().all(|f| f.0 == "InvalidValue"));
}

#[test]
fn m3_a_duplicate_in_another_category_or_file_is_dropped_too() {
    let stack = stack_of(
        &[
            vanilla("base").catalog(&in_category(
                "nature",
                "custom:a",
                None,
                &["custom:item1", "custom:item2:1"],
            )),
            addon("top").catalog(&in_category(
                "items",
                "custom:b",
                None,
                &[
                    "custom:item1:0",
                    "custom:item2",
                    "custom:item2:1",
                    "custom:item3",
                ],
            )),
            registry_pack(),
        ],
        options(),
    );
    let merged = stack.crafting_catalog();
    assert_eq!(
        merged.categories[CategoryName::Items as usize].groups[0].items,
        [item("custom:item2"), item("custom:item3")]
    );
    let found = found(merged);
    let texts: Vec<&str> = found.iter().map(|f| f.3.as_str()).collect();
    assert_eq!(
        texts,
        [
            "You can only add an item to the crafting item catalog once. Found duplicate custom:item1:0",
            "You can only add an item to the crafting item catalog once. Found duplicate custom:item2:1",
        ]
    );
    assert!(found.iter().all(|f| f.2 == "top" && f.1 == Severity::Error));
}

// With `item` alone a non-vanilla name that is no item is kept (it may be a
// pack's block), so the drop this test needs takes the block registry in view.
#[cfg(feature = "block")]
#[test]
fn m6_an_unknown_icon_is_a_warning_and_the_group_has_no_icon_deviation() {
    // vanilla = "silently dropped": an empty icon, no message.
    let stack = stack_of(
        &[
            vanilla("base").catalog(&in_category(
                "nature",
                "custom:g",
                Some("custom:nowhere"),
                &["custom:item1"],
            )),
            registry_pack(),
        ],
        options(),
    );
    assert_eq!(
        unresolved(&stack),
        [(
            "custom:nowhere".to_owned(),
            "minecraft:crafting_items_catalog/categories/0/groups/0/group_identifier/icon"
                .to_owned()
        )]
    );
    let merged = stack.crafting_catalog();
    assert_eq!(
        merged.categories[1].groups,
        [named("custom:g", None, &["custom:item1"])]
    );
}

#[test]
fn m6_an_icon_with_a_bad_aux_value_is_vanilla_s_error_and_the_group_has_no_icon() {
    let stack = stack_of(
        &[
            vanilla("base").catalog(&in_category(
                "nature",
                "custom:g",
                Some("custom:my_icon:x"),
                &[],
            )),
            registry_pack(),
        ],
        options(),
    );
    let merged = stack.crafting_catalog();
    let found = found(merged);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        found[0].3,
        "Invalid aux value in item ID \"custom:my_icon:x\""
    );
    assert_eq!(merged.categories[1].groups, [named("custom:g", None, &[])]);
}

/// The icon of a later definition of a group is ignored, so it is not looked up either; an
/// icon is not a placement, so the item itself can still be placed.
#[test]
fn m6_only_the_first_definition_s_icon_is_looked_at() {
    let stack = stack_of(
        &[
            vanilla("base").catalog(&in_category(
                "nature",
                "custom:g",
                Some("custom:item1"),
                &["custom:item1"],
            )),
            addon("top").catalog(&in_category(
                "nature",
                "custom:g",
                Some("custom:nowhere"),
                &[],
            )),
            registry_pack(),
        ],
        options(),
    );
    let merged = stack.crafting_catalog();
    assert_eq!(merged.diagnostics().count(), 0);
    assert_eq!(
        merged.categories[1].groups,
        [named("custom:g", Some("custom:item1"), &["custom:item1"])]
    );
}

#[test]
fn group_names_compare_byte_for_byte() {
    let stack = stack_of(
        &[
            vanilla("base").catalog(&construction("custom:Group", &[])),
            addon("top").catalog(&construction("custom:group", &[])),
        ],
        options(),
    );
    assert_eq!(
        groups(stack.crafting_catalog(), CategoryName::Construction),
        ["custom:Group", "custom:group"]
    );
}

#[test]
fn the_merged_catalogue_holds_no_unknown_members() {
    let text = r#"{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {"note": 1, "categories": [
        {"category_name": "items", "hidden": true, "groups": [{"colour": 1,
            "group_identifier": {"name": "custom:g", "tint": 2, "icon": {"name": "custom:item1", "x": 1}},
            "items": [{"name": "custom:item1", "count": 2}]}]}]}}"#;
    let stack = stack_of(&[vanilla("base").catalog(text), registry_pack()], options());
    let merged = stack.crafting_catalog();
    assert_eq!(
        merged.value(),
        &catalog(vec![
            category(CategoryName::Construction, Vec::new()),
            category(CategoryName::Nature, Vec::new()),
            category(CategoryName::Equipment, Vec::new()),
            category(
                CategoryName::Items,
                vec![named("custom:g", Some("custom:item1"), &["custom:item1"])]
            ),
        ])
    );
    // The layer does not keep them either; their warnings are the merged catalogue's.
    assert!(merged.layers()[0].unknown().is_empty());
    assert_eq!(merged.diagnostics().count(), 6);
}
