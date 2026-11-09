//! The cost of the rules of this group that do more than one probe per reference: a
//! localisation key no file defines asks every language, the catalogue's checkers find the
//! place of each entry they tell of, and a tag ingredient is held to every item of its tag.

#![allow(
    unused_imports,
    dead_code,
    reason = "which tests exist depends on the enabled features"
)]
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::needless_pass_by_value,
    clippy::format_push_string,
    clippy::items_after_statements,
    reason = "tests: one of them reads the game's files from disk"
)]

use crate::pack::PackOrigin;
use crate::pack::StackSet;
use crate::pack::tests::pack_from;

use crate::pack::link::tests::support::{leak, pack, set_of};
use crate::pack::link::{LinkOptions, link_with_rules};
use bedrock_addon_core::load::LoadOptions;

/// A world whose behaviour pack has `files` and whose resource pack has one language of `keys`
/// keys, none of them one the tests ask for.
fn world(files: Vec<(&'static str, String)>, keys: usize) -> StackSet {
    let files: Vec<(&'static str, Option<&str>)> = files
        .iter()
        .map(|(path, text)| (*path, Some(text.as_str())))
        .collect();
    let mut lang = String::new();
    for key in 0..keys {
        lang.push_str(&format!("other.key.{key}=Text\n"));
    }
    let resources = [
        ("texts/languages.json", Some(r#"["en_US"]"#)),
        ("texts/en_US.lang", Some(lang.as_str())),
    ];
    set_of(
        vec![pack(1, "data", &files)],
        vec![pack(2, "resources", &resources)],
        &LoadOptions::default(),
    )
}

fn path(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// The display-name rule over `n` blocks of seventeen display names each (the block's and one per
/// permutation) that no language defines, beside `n` keys.
#[cfg(all(feature = "block", feature = "lang", feature = "languages"))]
#[test]
fn n_names_no_language_defines_cost_n() {
    const NAMES: usize = 17;
    fn input(n: usize) -> StackSet {
        let files = (0..n)
            .map(|i| {
                let permutations: Vec<String> = (0..NAMES - 1)
                    .map(|p| {
                        format!(
                            r#"{{"condition": "q.block_state('demo:s') == {p}", "components": {{"minecraft:display_name": "tile.demo.b{i}.p{p}.name"}}}}"#
                        )
                    })
                    .collect();
                let block = format!(
                    r#"{{"format_version": "1.26.20", "minecraft:block": {{"description": {{"identifier": "demo:b{i}", "states": {{"demo:s": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]}}}}, "components": {{"minecraft:display_name": "tile.demo.b{i}.name"}}, "permutations": [{}]}}}}"#,
                    permutations.join(", ")
                );
                (path(format!("blocks/b{i}.json")), block)
            })
            .collect();
        let set = world(files, n);
        // The collections load on their first use: before the clock.
        assert_eq!(set.behavior.blocks().len(), n);
        let _ = link_with_rules(&set, LinkOptions::default(), &[super::block_text::run]);
        set
    }
    crate::test_cost::assert_linear_at(
        "link: n blocks of 17 display names no language defines",
        1_000,
        input,
        |set: &StackSet| {
            let found = link_with_rules(set, LinkOptions::default(), &[super::block_text::run]);
            assert_eq!(found.len(), NAMES * set.behavior.blocks().len());
            found
        },
    );
}

/// The manifest of the game's own behaviour pack: the catalogue merges from its file.
#[cfg(feature = "item")]
const VANILLA_MANIFEST: &str = r#"{ "format_version": 2,
  "header": { "name": "vanilla", "description": "", "uuid": "fe9f8597-5454-481a-8730-8d070a8e2e58",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 60] },
  "modules": [{ "type": "data", "uuid": "00000000-0000-4000-9000-0000000000aa", "version": [1, 0, 0] }] }"#;

/// A world whose base catalogue is `catalog` and whose own behaviour pack has `files`.
#[cfg(feature = "item")]
fn cataloged(catalog: &str, files: Vec<(&'static str, String)>) -> StackSet {
    let base = pack_from(
        VANILLA_MANIFEST,
        &[(
            "item_catalog/crafting_item_catalog.json",
            Some(leak(catalog)),
        )],
        PackOrigin::Package,
    );
    let files: Vec<(&'static str, Option<&str>)> = files
        .iter()
        .map(|(path, text)| (*path, Some(text.as_str())))
        .collect();
    set_of(
        vec![pack(1, "data", &files), base],
        Vec::new(),
        &LoadOptions::default(),
    )
}

/// The catalogue-group rule over a catalogue of `n` named groups whose names no language defines.
#[cfg(all(feature = "item", feature = "lang", feature = "languages"))]
#[test]
fn n_group_names_no_language_defines_cost_n() {
    fn input(n: usize) -> StackSet {
        let groups: Vec<String> = (0..n)
            .map(|i| {
                format!(r#"{{"group_identifier": {{"name": "demo:group_{i}"}}, "items": []}}"#)
            })
            .collect();
        let catalog = format!(
            r#"{{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {{"categories": [{{"category_name": "construction", "groups": [{}]}}]}}}}"#,
            groups.join(", ")
        );
        let set = cataloged(&catalog, Vec::new());
        let found = link_with_rules(&set, LinkOptions::default(), &[super::item_text::run]);
        assert_eq!(found.len(), n, "{found:#?}");
        set
    }
    crate::test_cost::assert_linear_at(
        "link: n group names no language defines",
        2_000,
        input,
        |set: &StackSet| link_with_rules(set, LinkOptions::default(), &[super::item_text::run]),
    );
}

/// The placement rule over `n` items the catalogue places outside their own category.
#[cfg(feature = "item")]
#[test]
fn n_items_placed_outside_their_category_cost_n() {
    fn input(n: usize) -> StackSet {
        let names: Vec<String> = (0..n).map(|i| format!(r#""demo:item_{i}""#)).collect();
        let catalog = format!(
            r#"{{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {{"categories": [{{"category_name": "construction", "groups": [{{"group_identifier": {{"name": "demo:group"}}, "items": [{}]}}]}}]}}}}"#,
            names.join(", ")
        );
        let files = (0..n)
            .map(|i| {
                let item = format!(
                    r#"{{"format_version": "1.21.60", "minecraft:item": {{"description": {{"identifier": "demo:item_{i}", "menu_category": {{"category": "nature"}}}}, "components": {{"minecraft:max_stack_size": 64}}}}}}"#
                );
                (path(format!("items/i{i}.json")), item)
            })
            .collect();
        let set = cataloged(&catalog, files);
        assert_eq!(set.behavior.items().len(), n);
        let found = link_with_rules(&set, LinkOptions::default(), &[placements]);
        assert_eq!(found.len(), n, "{found:#?}");
        set
    }
    fn placements(cx: &crate::pack::link::LinkCx<'_>, out: &mut crate::pack::link::Findings) {
        let stack = cx.behavior();
        out.extend(crate::families::crafting_catalog::link_placements(
            stack.crafting_catalog(),
            &super::item::ItemDefinitions(stack.items()),
        ));
    }
    crate::test_cost::assert_linear_at(
        "link: n items placed outside their category",
        1_000,
        input,
        |set: &StackSet| link_with_rules(set, LinkOptions::default(), &[placements]),
    );
}

/// The tag-ingredient rule over `n` recipes that each take an ingredient by a tag `n` items
/// carry, every one above the stack size of the last of them.
#[cfg(all(feature = "recipe", feature = "item"))]
#[test]
fn n_tag_ingredients_among_n_items_of_the_tag_cost_n() {
    fn input(n: usize) -> StackSet {
        let mut files: Vec<(&'static str, String)> = (0..n)
            .map(|i| {
                // Only the last item stacks low enough to offend.
                let stack = if i + 1 == n { 8 } else { 64 };
                let item = format!(
                    r#"{{"format_version": "1.26.30", "minecraft:item": {{"description": {{"identifier": "demo:item_{i}"}}, "components": {{"minecraft:max_stack_size": {stack}, "minecraft:tags": {{"tags": ["demo:gems"]}}}}}}}}"#
                );
                (path(format!("items/i{i}.json")), item)
            })
            .collect();
        files.extend((0..n).map(|i| {
            let recipe = format!(
                r#"{{"format_version": "1.12", "minecraft:recipe_shapeless": {{"description": {{"identifier": "demo:recipe_{i}"}}, "tags": ["crafting_table"], "ingredients": [{{"tag": "demo:gems", "count": 9}}], "result": {{"item": "demo:item_0"}}}}}}"#
            );
            (path(format!("recipes/r{i}.json")), recipe)
        }));
        let set = world(files, 1);
        assert_eq!(set.behavior.items().len(), n);
        assert_eq!(set.behavior.recipes().len(), n);
        set
    }
    crate::test_cost::assert_linear_at(
        "link: n tag ingredients among n items of the tag",
        1_500,
        input,
        |set: &StackSet| {
            let found = link_with_rules(set, LinkOptions::default(), &[super::recipe_item::run]);
            // Each recipe: the item's line and the place's.
            assert_eq!(found.len(), 2 * set.behavior.recipes().len());
            found
        },
    );
}

/// The item tags the pass knows without a host are exactly the tags the ingredients of the
/// game's own recipes are taken by.
#[cfg(all(feature = "recipe", feature = "item"))]
#[test]
fn every_tag_of_the_vanilla_recipes_is_a_tag_of_the_game() {
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

    use bedrock_addon_core::json::Value;

    let Some(files_root) = std::env::var_os("BEDROCK_ADDON_CORPUS") else {
        eprintln!("skipped: BEDROCK_ADDON_CORPUS is not set");
        return;
    };
    fn tags(value: &Value, out: &mut BTreeSet<String>) {
        if let Some(object) = value.as_object() {
            if let Some(tag) = object.get("tag").and_then(Value::as_string) {
                out.insert(tag.as_str().to_owned());
            }
            for (_, member) in object {
                tags(member, out);
            }
        } else if let Some(array) = value.as_array() {
            for element in array {
                tags(element, out);
            }
        }
    }
    fn read(dir: &Path, out: &mut BTreeSet<String>, files: &mut usize) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            if path.is_dir() {
                read(&path, out, files);
            } else if let Ok(text) = std::fs::read_to_string(&path)
                && let Ok(value) = crate::json::from_str_jsonc::<Value>(&text)
            {
                *files += 1;
                tags(&value, out);
            }
        }
    }
    let files_root = PathBuf::from(files_root);
    let mut roots = vec![files_root.join("bedrock-samples").join("behavior_pack")];
    let server = files_root.join("bds-1.26.36.1").join("behavior_packs");
    roots.extend(
        std::fs::read_dir(server)
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path()),
    );
    let (mut used, mut files) = (BTreeSet::new(), 0);
    for root in roots {
        read(&root.join("recipes"), &mut used, &mut files);
    }
    assert!(
        files > 1000,
        "the recipes of the game's files were read: {files}"
    );
    let known: BTreeSet<String> = crate::vanilla::names::VANILLA_ITEM_TAGS
        .iter()
        .map(|tag| (*tag).to_owned())
        .collect();
    assert_eq!(used, known);
    assert!(
        crate::vanilla::names::VANILLA_ITEM_TAGS
            .windows(2)
            .all(|pair| pair[0] < pair[1]),
        "sorted"
    );
}
