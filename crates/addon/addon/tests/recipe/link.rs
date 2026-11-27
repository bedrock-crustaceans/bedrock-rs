//! Rows whose finding needs the item registry (unknown items and tags, stack sizes,
//! data values against block states, smithing materials and trim registries). Recipes cannot
//! read another module's registry at load, so these are the link pass's: the
//! recipe loads and `StackSet::link` tells each row with vanilla's text, as an error.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, WorldPacks};
use bedrock_addon::version::{BaseGameVersion, SemVersion};

use super::common::*;
use super::stacks::*;

// The four rows are told by the link pass; the next item and each `linked(…)` below.
/// The link findings of a world of base-game version `world` whose one behaviour pack has the
/// recipe fixture `name` and the item files `items`; the recipe loads.
fn linked(name: &str, items: &[(&str, &str)], world: Option<(u16, u16)>) -> Diagnostics {
    let mut files = vec![(format!("recipes/{name}"), fixture(name))];
    for (path, text) in items {
        files.push(((*path).to_owned(), text.as_bytes().to_vec()));
    }
    let pack = pack(1, [1, 21, 0], PackOrigin::World, files);
    let selection = PackSelection {
        pack_id: pack.id().uuid,
        version: pack.id().version.clone(),
        subpack: None,
    };
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::World);
    let world_packs = WorldPacks {
        behavior: vec![selection],
        resource: Vec::new(),
    };
    let options = LoadOptions {
        base_game_version: world.map_or_else(BaseGameVersion::default, |(major, minor)| {
            BaseGameVersion::new(SemVersion::new(major, minor, 0))
        }),
        ..LoadOptions::default()
    };
    let set = repository.stacks(&world_packs, options);
    assert_eq!(set.behavior.recipes().len(), 1, "the recipe loads");
    set.link()
}

#[test]
fn shaped_key_unknown_item_is_rejected() {
    let found = linked("shaped_key_unknown_item.json", &[], None);
    one(
        &found,
        "The Item: {} is missing or invalid, can't make the recipe",
        Severity::Error,
    );
    one(
        &found,
        "Recipe for: {} is missing (unknown) ingredient",
        Severity::Error,
    );
}

#[test]
fn shaped_result_no_item_is_rejected() {
    let found = linked("shaped_result_no_item.json", &[], None);
    one(&found, "Recipe has an invalid result", Severity::Error);
}

/// 1.26.20 required `minecraft:netherite_ingot` (`vanilla = "1.26.20: addition must be
/// minecraft:netherite_ingot"`).
#[test]
fn smithing_transform_diamond_addition_is_rejected_by_tag_deviation() {
    let found = linked(
        "smithing_transform_diamond_addition.json",
        &[],
        Some((1, 21)),
    );
    let finding = one(
        &found,
        "Smithing Transform Recipe: {} addition item must have the '{}' tag.",
        Severity::Error,
    );
    assert_eq!(
        args_in(
            finding,
            "Smithing Transform Recipe: {} addition item must have the '{}' tag."
        ),
        ["demo:diamond_addition", "minecraft:transform_materials"]
    );
}

// The item that carries the tag is the world's.
/// An item of the world that carries `minecraft:planks` and stacks to 16.
const PLANK: &str = r#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:plank"}, "components": {"minecraft:max_stack_size": 16, "minecraft:tags": {"tags": ["minecraft:planks"]}}}}"#;

#[test]
fn shapeless_tag_count_over_stack_is_rejected() {
    let found = linked(
        "shapeless_tag_count_over_stack.json",
        &[("items/plank.json", PLANK)],
        None,
    );
    let finding = one(
        &found,
        "The Item from tag '{}': {} has invalid stack size: {}",
        Severity::Error,
    );
    assert_eq!(
        args_in(
            finding,
            "The Item from tag '{}': {} has invalid stack size: {}"
        ),
        ["minecraft:planks", "demo:plank", "99"]
    );
    one(&found, "Recipe result malformed", Severity::Error);
}

/// The same files load: the parse stage has nothing to say about them.
#[test]
fn rows_that_need_the_item_registry_load_at_parse() {
    for (name, cx) in [
        ("shaped_key_unknown_item.json", cx()),
        ("shaped_result_no_item.json", cx()),
        ("smithing_transform_diamond_addition.json", cx_world(1, 21)),
        ("shapeless_tag_count_over_stack.json", cx()),
    ] {
        let document = loaded(name, &cx);
        clean(document.diagnostics());
    }
}
