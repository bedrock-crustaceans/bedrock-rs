//! The references of a recipe (20 RC1 to RC4): its items, its item tags, the smithing
//! ingredients and the crafting stations.

#![allow(
    unused_imports,
    dead_code,
    reason = "which tests exist depends on the enabled features"
)]

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, stacks};
use super::{said_of, text};

const NO_LINTS: LinkOptions = LinkOptions {
    lints: false,
    client: true,
};
const FILE: &str = "recipes/demo.json";

/// The link findings of a world whose behaviour pack must load `recipes` recipes.
fn linked(
    recipes: usize,
    behavior: &[(&'static str, &str)],
    load: LoadOptions,
    options: LinkOptions,
) -> Diagnostics {
    let set = stacks(behavior, &[], load);
    let loaded = set.behavior.recipes();
    assert_eq!(
        loaded.len(),
        recipes,
        "{:#?}",
        loaded
            .diagnostics()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    set.link_with(&options)
}

fn link(behavior: &[(&'static str, &str)]) -> Diagnostics {
    let recipes = behavior
        .iter()
        .filter(|(path, _)| path.starts_with("recipes/"))
        .count();
    linked(
        recipes,
        behavior,
        LoadOptions::default(),
        LinkOptions::default(),
    )
}

/// A recipe file of `kind` (`shaped`, `furnace`) with the JSON `members` after its
/// description.
fn recipe(kind: &str, members: &str) -> String {
    format!(
        r#"{{"format_version": "1.12", "minecraft:recipe_{kind}": {{"description": {{"identifier": "demo:recipe"}}, {members}}}}}"#
    )
}

fn shaped(key: &str, result: &str) -> String {
    recipe(
        "shaped",
        &format!(
            r##""tags": ["crafting_table"], "pattern": ["#"], "key": {{"#": {key}}}, "result": {result}"##
        ),
    )
}

fn shapeless(ingredients: &str, result: &str) -> String {
    recipe(
        "shapeless",
        &format!(r#""tags": ["crafting_table"], "ingredients": {ingredients}, "result": {result}"#),
    )
}

/// What the findings in the recipe file say: the code, the severity, the JSON path, the
/// vanilla text and the reference (kind, name).
type Told = (
    &'static str,
    Severity,
    String,
    Option<String>,
    &'static str,
    String,
);

fn told_of(finding: &Diagnostic) -> Told {
    let said = said_of(finding);
    (said.0, said.1, said.3, text(finding), said.4, said.5)
}

fn told(found: &Diagnostics) -> Vec<Told> {
    found
        .iter()
        .filter(|finding| said_of(finding).2 == FILE)
        .map(told_of)
        .collect()
}

fn error(code: &'static str, path: &str, text: &str, kind: &'static str, id: &str) -> Told {
    (
        code,
        Severity::Error,
        path.to_owned(),
        Some(text.to_owned()),
        kind,
        id.to_owned(),
    )
}

const SHAPED: &str = "minecraft:recipe_shaped";
const SHAPELESS: &str = "minecraft:recipe_shapeless";

#[cfg(feature = "item")]
mod item {
    use super::*;

    const GEM: (&str, &str) = (
        "items/gem.json",
        r#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:gem"}, "components": {"minecraft:max_stack_size": 16, "minecraft:tags": {"tags": ["demo:gems"]}}}}"#,
    );

    // ---- RC1 -------------------------------------------------------------------------------

    /// The source member's own rule (the game's item parser): one token is a
    /// `minecraft:` name, the name is lower-cased, an `item.` prefix is dropped, an aux value
    /// after the name is no part of it.
    #[test]
    fn rc1_an_item_of_the_stack_or_of_the_game_is_no_finding_however_it_is_written() {
        for written in [
            r#""demo:gem""#,
            r#""Demo:Gem""#,
            r#""demo:gem:3""#,
            r#"{"item": "demo:gem", "data": 2}"#,
            r#""stick""#,
            r#""minecraft:Stick""#,
            r#""minecraft:item.stick""#,
            r#"{"item": "minecraft:planks", "data": 1}"#,
            // Names of older versions the game's own recipes use.
            r#""minecraft:boat""#,
            r#""minecraft:appleEnchanted""#,
            r#""minecraft:wood:3""#,
            // A recipe's names are lower-cased before they are looked up.
            r#""Minecraft:WOOD""#,
        ] {
            let file = shaped(written, r#"{"item": "minecraft:torch"}"#);
            let found = link(&[(FILE, &file), GEM]);
            assert_eq!(told(&found), [], "{written}");
        }
        let host = hosted(&[("item", "host:coin"), ("item", "minecraft:torch")]);
        let file = shaped(r#""host:coin""#, r#"{"item": "minecraft:torch"}"#);
        let found = linked(1, &[(FILE, &file)], host, LinkOptions::default());
        assert_eq!(told(&found), []);
    }

    #[cfg(feature = "block")]
    #[test]
    fn rc1_a_block_of_the_stack_is_an_item() {
        let lamp = ("blocks/lamp.json", &*super::super::block("demo:lamp", ""));
        let file = shaped(r#""demo:lamp""#, r#"{"item": "demo:lamp"}"#);
        let found = link(&[(FILE, &file), lamp]);
        assert_eq!(told(&found), []);
    }

    /// A name the game's entity files or loot tables write that is
    /// no name of the game (`nautilusarmor`, `iron_pick`) is no item to a recipe: RC1.
    #[test]
    fn rc1_a_name_only_another_familys_vanilla_files_write_is_vanillas_error() {
        for name in ["minecraft:nautilusarmor", "minecraft:iron_pick"] {
            let file = shaped(
                &format!(r#"{{"item": "{name}"}}"#),
                r#"{"item": "minecraft:torch"}"#,
            );
            let found = linked(1, &[(FILE, &file)], LoadOptions::default(), NO_LINTS);
            let items: Vec<_> = told(&found)
                .into_iter()
                .filter(|told| told.4 == "item")
                .map(|told| (told.0, told.1, told.5))
                .collect();
            assert_eq!(
                items,
                [(
                    "core/unresolved_reference_worded",
                    Severity::Error,
                    name.to_owned()
                )],
                "{name}"
            );
        }
    }

    // Without `block` a name outside `minecraft` may be a pack's block: not in view.
    #[cfg(feature = "block")]
    #[test]
    fn rc1_an_ingredient_nothing_defines_is_vanillas_error_and_the_sites() {
        let file = shaped(r#"{"item": "Demo:None"}"#, r#"{"item": "minecraft:torch"}"#);
        let found = linked(1, &[(FILE, &file)], LoadOptions::default(), NO_LINTS);
        assert_eq!(
            told(&found),
            [
                error(
                    "core/unresolved_reference_worded",
                    &format!("{SHAPED}/key/#/item"),
                    "The Item: demo:none is missing or invalid, can't make the recipe",
                    "item",
                    "Demo:None"
                ),
                error(
                    "core/document_rejected_worded",
                    &format!("{SHAPED}/key/#"),
                    "Recipe for: demo:recipe is missing (unknown) ingredient",
                    "",
                    ""
                ),
            ]
        );
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "item")
            .unwrap();
        assert_eq!(
            finding
                .location
                .pack
                .as_ref()
                .map(bedrock_addon::pack::PackRef::name),
            Some("bp 1")
        );
        // The recipe is as the load left it: the pass changes no registry.
        let set = stacks(&[(FILE, &file)], &[], LoadOptions::default());
        let _ = set.link();
        assert_eq!(set.behavior.recipes().len(), 1);
    }

    // Without `block` a name outside `minecraft` may be a pack's block: not in view.
    #[cfg(feature = "block")]
    #[test]
    fn rc1_a_result_that_is_no_item_is_an_invalid_result_once_per_entry() {
        let file = shaped(
            r#""stick""#,
            r#"[{"item": "minecraft:torch"}, {"item": "demo:none"}, "minecraft:air"]"#,
        );
        let found = link(&[(FILE, &file)]);
        assert_eq!(
            told(&found),
            [
                error(
                    "core/unresolved_reference_worded",
                    &format!("{SHAPED}/result/1/item"),
                    "Recipe has an invalid result",
                    "item",
                    "demo:none"
                ),
                error(
                    "core/unresolved_reference_worded",
                    &format!("{SHAPED}/result/2"),
                    "Recipe has an invalid result",
                    "item",
                    "minecraft:air"
                ),
            ]
        );
    }

    // Without `block` a name outside `minecraft` may be a pack's block: not in view.
    #[cfg(feature = "block")]
    #[test]
    fn rc1_a_shapeless_ingredient_nothing_defines_is_a_malformed_result() {
        let file = shapeless(
            r#"["stick", "demo:none"]"#,
            r#"{"item": "minecraft:torch"}"#,
        );
        let found = link(&[(FILE, &file)]);
        assert_eq!(
            told(&found),
            [
                error(
                    "core/unresolved_reference_worded",
                    &format!("{SHAPELESS}/ingredients/1"),
                    "The Item: demo:none is missing or invalid, can't make the recipe",
                    "item",
                    "demo:none"
                ),
                error(
                    "core/document_rejected_worded",
                    &format!("{SHAPELESS}/ingredients/1"),
                    "Recipe result malformed",
                    "",
                    ""
                ),
            ]
        );
    }

    /// The game logs the line at level 3,
    /// clears the requirement and returns; the
    /// recipe loads: a kept error.
    // Without `block` a name outside `minecraft` may be a pack's block: not in view.
    #[cfg(feature = "block")]
    #[test]
    fn rc1_an_unlock_item_nothing_defines_is_a_kept_error() {
        let file = recipe(
            "shapeless",
            r#""tags": ["crafting_table"], "ingredients": ["stick"], "result": {"item": "minecraft:torch"}, "unlock": [{"item": "minecraft:stick"}, {"item": "demo:none"}]"#,
        );
        let found = link(&[(FILE, &file)]);
        assert_eq!(
            told(&found),
            [(
                "core/unresolved_reference_worded",
                Severity::Error,
                format!("{SHAPELESS}/unlock/1/item"),
                Some("Recipe: demo:recipe has an invalid unlock ingredient".to_owned()),
                "item",
                "demo:none".to_owned()
            )]
        );
        assert!(found.iter().all(|finding| finding.kept), "{found:?}");
    }

    // Without `block` a name outside `minecraft` may be a pack's block: not in view.
    #[cfg(feature = "block")]
    #[test]
    fn rc1_the_items_of_every_other_recipe_type_are_resolved_with_the_sites_text() {
        let item = "The Item: demo:none is missing or invalid, can't make the recipe";
        // (kind, members, the member that names the missing item, the site's text)
        let cases: [(&str, &str, &str, Option<&str>); 9] = [
            (
                "furnace",
                r#""tags": ["furnace"], "input": "demo:none", "output": "minecraft:torch""#,
                "input",
                Some("Recipe result malformed"),
            ),
            (
                "furnace",
                r#""tags": ["furnace"], "input": "stick", "output": "demo:none""#,
                "output",
                Some("Recipe result malformed"),
            ),
            (
                "brewing_mix",
                r#""tags": ["brewing_stand"], "input": "minecraft:potion_type:awkward", "reagent": "demo:none", "output": "minecraft:potion_type:swiftness""#,
                "reagent",
                None,
            ),
            (
                "brewing_container",
                r#""tags": ["brewing_stand"], "input": "demo:none", "reagent": "minecraft:gunpowder", "output": "minecraft:splash_potion""#,
                "input",
                Some("Brewing Container Recipe: demo:recipe has an invalid input item"),
            ),
            (
                "brewing_container",
                r#""tags": ["brewing_stand"], "input": "minecraft:potion", "reagent": "demo:none", "output": "minecraft:splash_potion""#,
                "reagent",
                Some("Brewing Container Recipe: demo:recipe has an invalid reagent item"),
            ),
            (
                "brewing_container",
                r#""tags": ["brewing_stand"], "input": "minecraft:potion", "reagent": "minecraft:gunpowder", "output": "demo:none""#,
                "output",
                Some("Brewing Container Recipe: demo:recipe has an invalid output item"),
            ),
            (
                "material_reduction",
                r#""tags": "material_reducer", "input": "demo:none", "output": [{"item": "minecraft:stick"}]"#,
                "input",
                Some("Material Reduction Recipe: demo:recipe has an invalid input item"),
            ),
            (
                "material_reduction",
                r#""tags": "material_reducer", "input": "minecraft:stick", "output": [{"item": "demo:none"}]"#,
                "output/0/item",
                Some("Material Reduction Recipe: demo:recipe has an invalid output item"),
            ),
            (
                "smithing_transform",
                r#""tags": ["smithing_table"], "template": "minecraft:netherite_upgrade_smithing_template", "base": "minecraft:diamond_axe", "addition": "minecraft:netherite_ingot", "result": "demo:none""#,
                "result",
                Some("Smithing Transform Recipe: demo:recipe has an invalid result item."),
            ),
        ];
        for (kind, members, member, site) in cases {
            let found = link(&[(FILE, &recipe(kind, members))]);
            let payload = format!("minecraft:recipe_{kind}");
            let mut expected = vec![error(
                "core/unresolved_reference_worded",
                &format!("{payload}/{member}"),
                item,
                "item",
                "demo:none",
            )];
            if let Some(site) = site {
                let owner = member.split('/').next().unwrap();
                expected.push(error(
                    "core/document_rejected_worded",
                    &format!("{payload}/{owner}"),
                    site,
                    "",
                    "",
                ));
            }
            assert_eq!(told(&found), expected, "{kind} {member}");
        }
    }

    /// A potion is no item name: the game looks it up among its potions when it reads the
    /// file, and the pass has nothing to resolve.
    #[test]
    fn rc1_a_potion_is_not_looked_up_as_an_item() {
        let file = recipe(
            "brewing_mix",
            r#""tags": ["brewing_stand"], "input": "minecraft:potion_type:awkward", "reagent": "minecraft:sugar", "output": "minecraft:potion_type:swiftness""#,
        );
        let found = link(&[(FILE, &file)]);
        assert_eq!(told(&found), []);
    }

    // ---- RC2 -------------------------------------------------------------------------------

    fn tagged(tag: &str, count: i32) -> String {
        shapeless(
            &format!(r#"[{{"tag": "{tag}", "count": {count}}}]"#),
            r#"{"item": "minecraft:torch"}"#,
        )
    }

    #[test]
    fn rc2_a_tag_an_item_of_the_stack_or_of_the_game_carries_is_no_finding() {
        for tag in ["demo:gems", "minecraft:planks", "minecraft:trim_materials"] {
            let found = link(&[(FILE, &tagged(tag, 1)), GEM]);
            assert_eq!(told(&found), [], "{tag}");
        }
        // With a host the host answers for the game's tags, yes or no.
        let host = hosted(&[("item tag", "host:tag"), ("item", "minecraft:torch")]);
        let found = linked(
            1,
            &[(FILE, &tagged("host:tag", 1))],
            host,
            LinkOptions::default(),
        );
        assert_eq!(told(&found), []);
        // The tags of the game's own recipes stand with any host, as the legacy item names do.
        let host = hosted(&[("item", "minecraft:torch")]);
        let found = linked(
            1,
            &[(FILE, &tagged("minecraft:planks", 1))],
            host,
            LinkOptions::default(),
        );
        assert_eq!(told(&found), []);
    }

    /// The crate has no list of the tags the game's items carry: a `minecraft:` tag that
    /// neither the stack, the host nor the list of the game's recipe tags knows may be one of
    /// them. It is an information, never the row's error, with or without a host.
    #[test]
    fn rc2_a_minecraft_tag_nothing_here_knows_is_an_information_not_an_error() {
        for tag in ["minecraft:is_axe", "minecraft:is_sword", "minecraft:arrow"] {
            let expected = [(
                "core/unresolved_reference",
                Severity::Info,
                format!("{SHAPELESS}/ingredients/0/tag"),
                None,
                "item tag",
                tag.to_owned(),
            )];
            let found = link(&[(FILE, &tagged(tag, 1))]);
            assert_eq!(told(&found), expected, "{tag}");
            // A host that answers for no item tag changes nothing.
            let host = hosted(&[("item", "minecraft:torch")]);
            let found = linked(1, &[(FILE, &tagged(tag, 1))], host, LinkOptions::default());
            assert_eq!(told(&found), expected, "{tag} hosted");
            // A lint.
            let found = linked(
                1,
                &[(FILE, &tagged(tag, 1))],
                LoadOptions::default(),
                NO_LINTS,
            );
            assert_eq!(told(&found), [], "{tag} without the lints");
        }
        // What the host says the game's items carry is known.
        let host = hosted(&[
            ("item tag", "minecraft:is_axe"),
            ("item", "minecraft:torch"),
        ]);
        let found = linked(
            1,
            &[(FILE, &tagged("minecraft:is_axe", 1))],
            host,
            LinkOptions::default(),
        );
        assert_eq!(told(&found), []);
    }

    #[test]
    fn rc2_a_tag_no_item_carries_is_vanillas_error() {
        // Tags compare as written. Every pack item is in view, so a tag outside `minecraft`
        // that none carries is missing, with or without a host.
        let found = linked(
            1,
            &[(FILE, &tagged("demo:nothing", 1)), GEM],
            hosted(&[("item", "minecraft:torch")]),
            NO_LINTS,
        );
        assert_eq!(told(&found).len(), 2);
        for tag in ["demo:nothing", "demo:Gems"] {
            let found = linked(
                1,
                &[(FILE, &tagged(tag, 1)), GEM],
                LoadOptions::default(),
                NO_LINTS,
            );
            assert_eq!(
                told(&found),
                [
                    error(
                        "core/unresolved_reference_worded",
                        &format!("{SHAPELESS}/ingredients/0/tag"),
                        &format!(
                            "The Item Tag: {tag} is missing or invalid, can't make the recipe"
                        ),
                        "item tag",
                        tag
                    ),
                    error(
                        "core/document_rejected_worded",
                        &format!("{SHAPELESS}/ingredients/0"),
                        "Recipe result malformed",
                        "",
                        ""
                    ),
                ],
                "{tag}"
            );
        }
    }

    /// Both sides of the bound: a count up to the stack size of every item that carries the
    /// tag is fine, one above the smallest is the first offending item's error.
    #[test]
    fn rc2_a_count_above_the_stack_size_of_an_item_with_the_tag_is_an_error() {
        let found = link(&[(FILE, &tagged("demo:gems", 16)), GEM]);
        assert_eq!(told(&found), []);
        let found = link(&[(FILE, &tagged("demo:gems", 17)), GEM]);
        assert_eq!(
            told(&found),
            [
                error(
                    "core/invalid_reference_worded",
                    &format!("{SHAPELESS}/ingredients/0/count"),
                    "The Item from tag 'demo:gems': demo:gem has invalid stack size: 17",
                    "item",
                    "demo:gem"
                ),
                error(
                    "core/document_rejected_worded",
                    &format!("{SHAPELESS}/ingredients/0"),
                    "Recipe result malformed",
                    "",
                    ""
                ),
            ]
        );
        // An item without `minecraft:max_stack_size` stacks to 64.
        let plain = (
            "items/plain.json",
            r#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:plain"}, "components": {"minecraft:tags": {"tags": ["demo:plains"]}}}}"#,
        );
        let found = link(&[(FILE, &tagged("demo:plains", 64)), plain]);
        assert_eq!(told(&found), []);
        let found = link(&[(FILE, &tagged("demo:plains", 65)), plain]);
        assert_eq!(told(&found).len(), 2);
    }

    // ---- RC3 -------------------------------------------------------------------------------

    fn transform(addition: &str) -> String {
        format!(
            r#"{{"format_version": "1.20.10", "minecraft:recipe_smithing_transform": {{"description": {{"identifier": "demo:recipe"}}, "tags": ["smithing_table"], "template": "minecraft:netherite_upgrade_smithing_template", "base": "minecraft:diamond_axe", "addition": {addition}, "result": "minecraft:netherite_axe"}}}}"#
        )
    }

    const TRANSFORM: &str = "minecraft:recipe_smithing_transform";
    const MATERIAL: (&str, &str) = (
        "items/alloy.json",
        r#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:alloy"}, "components": {"minecraft:tags": {"tags": ["minecraft:transform_materials"]}}}}"#,
    );

    #[test]
    fn rc3_an_addition_with_the_transform_tag_is_no_finding() {
        for addition in [
            r#""minecraft:netherite_ingot""#,
            r#""demo:alloy""#,
            r#"{"item": "demo:alloy"}"#,
        ] {
            let found = link(&[(FILE, &transform(addition)), MATERIAL]);
            assert_eq!(told(&found), [], "{addition}");
        }
    }

    #[test]
    fn rc3_an_addition_without_the_transform_tag_is_vanillas_error() {
        for (addition, member, written) in [
            (r#""minecraft:diamond""#, "addition", "minecraft:diamond"),
            (r#"{"item": "demo:gem"}"#, "addition/item", "demo:gem"),
        ] {
            let found = linked(
                1,
                &[(FILE, &transform(addition)), GEM],
                LoadOptions::default(),
                NO_LINTS,
            );
            assert_eq!(
                told(&found),
                [error(
                    "core/invalid_reference_worded",
                    &format!("{TRANSFORM}/{member}"),
                    "Smithing Transform Recipe: demo:recipe addition item must have the 'minecraft:transform_materials' tag.",
                    "item",
                    written
                )],
                "{addition}"
            );
        }
    }

    fn trim(template: &str, addition: &str) -> String {
        format!(
            r#"{{"format_version": "1.20.10", "minecraft:recipe_smithing_trim": {{"description": {{"identifier": "demo:recipe"}}, "tags": ["smithing_table"], "template": {template}, "base": {{"tag": "minecraft:trimmable_armors"}}, "addition": {addition}}}}}"#
        )
    }

    const TRIM: &str = "minecraft:recipe_smithing_trim";

    #[test]
    fn rc3_the_templates_and_materials_of_the_game_are_no_finding() {
        for (template, addition) in [
            (
                r#"{"tag": "minecraft:trim_templates"}"#,
                r#"{"tag": "minecraft:trim_materials"}"#,
            ),
            (
                r#""minecraft:ward_armor_trim_smithing_template""#,
                r#""minecraft:quartz""#,
            ),
            (
                r#"{"item": "minecraft:bolt_armor_trim_smithing_template"}"#,
                r#"{"item": "minecraft:resin_brick"}"#,
            ),
        ] {
            let found = link(&[(FILE, &trim(template, addition))]);
            assert_eq!(told(&found), [], "{template} {addition}");
        }
    }

    /// Trim rule T6 and its material twin.
    #[test]
    fn rc3_a_template_that_is_no_pattern_and_an_addition_that_is_no_material_are_errors() {
        let file = trim(
            r#""minecraft:stick""#,
            r#"{"item": "minecraft:diamond_axe"}"#,
        );
        let found = linked(1, &[(FILE, &file)], LoadOptions::default(), NO_LINTS);
        assert_eq!(
            told(&found),
            [
                error(
                    "core/invalid_reference_worded",
                    &format!("{TRIM}/template"),
                    "Smithing Trim Recipe: demo:recipe has minecraft:stick as a template ingredient, which should be a valid trim pattern, but isn't. Did you register it in the Trim Pattern Registry?",
                    "item",
                    "minecraft:stick"
                ),
                error(
                    "core/invalid_reference_worded",
                    &format!("{TRIM}/addition/item"),
                    "Smithing Trim Recipe: demo:recipe has minecraft:diamond_axe as an addition ingredient, which should be a valid trim material, but isn't.",
                    "item",
                    "minecraft:diamond_axe"
                ),
            ]
        );
    }

    /// The game holds every item that carries the template's tag to the pattern table: a
    /// pack item tagged `minecraft:trim_templates` (trim rule T4) fails a recipe that takes
    /// its template by that tag.
    #[test]
    fn rc3_an_item_a_pack_tags_as_a_template_fails_a_recipe_that_takes_the_tag() {
        let spiral = (
            "items/spiral.json",
            r#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:spiral"}, "components": {"minecraft:tags": {"tags": ["minecraft:trim_templates", "minecraft:trim_materials"]}}}}"#,
        );
        let file = trim(
            r#"{"tag": "minecraft:trim_templates"}"#,
            r#"{"tag": "minecraft:trim_materials"}"#,
        );
        let found = link(&[(FILE, &file), spiral]);
        assert_eq!(
            told(&found),
            [
                error(
                    "core/invalid_reference_worded",
                    &format!("{TRIM}/template/tag"),
                    "Smithing Trim Recipe: demo:recipe has demo:spiral as a template ingredient, which should be a valid trim pattern, but isn't. Did you register it in the Trim Pattern Registry?",
                    "item",
                    "demo:spiral"
                ),
                error(
                    "core/invalid_reference_worded",
                    &format!("{TRIM}/addition/tag"),
                    "Smithing Trim Recipe: demo:recipe has demo:spiral as an addition ingredient, which should be a valid trim material, but isn't.",
                    "item",
                    "demo:spiral"
                ),
            ]
        );
    }
}

#[cfg(not(feature = "item"))]
#[test]
fn rc1_rc2_rc3_without_the_item_feature_nothing_is_told() {
    let file = shaped(r#"{"item": "demo:none"}"#, r#"{"item": "demo:none"}"#);
    let found = linked(1, &[(FILE, &file)], LoadOptions::default(), &NO_LINTS);
    assert_eq!(told(&found), []);
    let file = shapeless(r#"[{"tag": "demo:none"}]"#, r#"{"item": "demo:none"}"#);
    let found = linked(1, &[(FILE, &file)], LoadOptions::default(), &NO_LINTS);
    assert_eq!(told(&found), []);
}

// ---- RC4 -----------------------------------------------------------------------------------

#[cfg(feature = "block")]
mod station {
    use super::*;

    fn offered_at(tags: &str) -> String {
        recipe(
            "shapeless",
            &format!(
                r#""tags": {tags}, "ingredients": ["stick"], "result": {{"item": "minecraft:torch"}}"#
            ),
        )
    }

    fn stations(found: &Diagnostics) -> Vec<Told> {
        told(found)
            .into_iter()
            .filter(|told| told.4 == "crafting tag")
            .collect()
    }

    const BENCH: (&str, &str) = (
        "blocks/bench.json",
        r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:bench", "states": {"demo:big": [false, true]}}, "components": {"minecraft:crafting_table": {"crafting_tags": ["demo_bench"], "table_name": "Bench"}}, "permutations": [{"condition": "q.block_state('demo:big') == true", "components": {"minecraft:crafting_table": {"crafting_tags": ["demo_big_bench"], "table_name": "Bench"}}}]}}"#,
    );

    #[test]
    fn rc4_a_station_of_the_game_or_of_a_block_of_the_stack_is_no_finding() {
        for tags in [
            r#"["crafting_table", "stonecutter", "cartography_table"]"#,
            r#"["furnace", "blast_furnace", "smoker", "campfire", "soul_campfire"]"#,
            r#"["brewing_stand", "smithing_table", "deprecated"]"#,
            r#""material_reducer""#,
            r#"["demo_bench", "demo_big_bench"]"#,
        ] {
            let found = link(&[(FILE, &offered_at(tags)), BENCH]);
            assert_eq!(stations(&found), [], "{tags}");
        }
        let host = hosted(&[
            ("crafting tag", "host_bench"),
            ("item", "minecraft:torch"),
            ("item", "minecraft:stick"),
        ]);
        let found = linked(
            1,
            &[(FILE, &offered_at(r#"["host_bench"]"#))],
            host,
            LinkOptions::default(),
        );
        assert_eq!(stations(&found), []);
    }

    #[test]
    fn rc4_a_tag_no_block_offers_is_an_information_at_its_place() {
        let found = link(&[
            (
                FILE,
                &offered_at(r#"["crafting_table", "demo_none", "Demo_Bench"]"#),
            ),
            BENCH,
        ]);
        let expected: Vec<Told> = [(1, "demo_none"), (2, "Demo_Bench")]
            .into_iter()
            .map(|(index, tag)| {
                (
                    "core/unresolved_reference",
                    Severity::Info,
                    format!("{SHAPELESS}/tags/{index}"),
                    None,
                    "crafting tag",
                    tag.to_owned(),
                )
            })
            .collect();
        assert_eq!(stations(&found), expected, "{found:#?}");
        // The one-string form is told at `tags`.
        let found = link(&[(FILE, &offered_at(r#""demo_none""#))]);
        assert_eq!(
            stations(&found),
            [(
                "core/unresolved_reference",
                Severity::Info,
                format!("{SHAPELESS}/tags"),
                None,
                "crafting tag",
                "demo_none".to_owned()
            )]
        );
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "crafting tag")
            .unwrap();
        assert_eq!(
            finding
                .location
                .pack
                .as_ref()
                .map(bedrock_addon::pack::PackRef::name),
            Some("bp 1")
        );
        // A lint.
        let found = linked(
            1,
            &[(FILE, &offered_at(r#""demo_none""#))],
            LoadOptions::default(),
            NO_LINTS,
        );
        assert_eq!(stations(&found), []);
    }
}

#[cfg(not(feature = "block"))]
#[test]
fn rc4_without_the_block_feature_nothing_is_told() {
    let file = recipe(
        "shapeless",
        r#""tags": ["demo_none"], "ingredients": ["stick"], "result": {"item": "minecraft:torch"}"#,
    );
    let found = link(&[(FILE, &file)]);
    assert_eq!(
        told(&found)
            .into_iter()
            .filter(|told| told.4 == "crafting tag")
            .count(),
        0
    );
}

// ---- the index of an entry is the file's -----------------------------------------------------

/// RC4 behind entries of `tags` the reader leaves out (one that is no string): the index is
/// the file's.
#[cfg(feature = "block")]
#[test]
fn rc4_a_tag_behind_entries_the_reader_leaves_out_is_at_the_index_of_the_file() {
    let file = recipe(
        "shapeless",
        r#""tags": [7, "crafting_table", null, "demo_none", {}, "demo_nothing"], "ingredients": ["stick"], "result": {"item": "minecraft:torch"}"#,
    );
    let found = link(&[(FILE, &file)]);
    let stations: Vec<(String, String)> = told(&found)
        .into_iter()
        .filter(|told| told.4 == "crafting tag")
        .map(|told| (told.2, told.5))
        .collect();
    assert_eq!(
        stations,
        [
            (format!("{SHAPELESS}/tags/3"), "demo_none".to_owned()),
            (format!("{SHAPELESS}/tags/5"), "demo_nothing".to_owned()),
        ],
        "{found:#?}"
    );
}

/// RC1 behind `null` entries of `result`, which the reader leaves out: the index is the
/// file's, of a shaped and of a shapeless recipe.
#[cfg(feature = "item")]
#[test]
fn rc1_a_result_behind_null_entries_is_at_the_index_of_the_file() {
    let result = r#"[null, {"item": "demo:none"}, null, {"item": "demo:nothing"}]"#;
    for (payload, file) in [
        (SHAPED, shaped(r#""stick""#, result)),
        (SHAPELESS, shapeless(r#"["stick"]"#, result)),
    ] {
        let found = linked(1, &[(FILE, &file)], LoadOptions::default(), NO_LINTS);
        let items: Vec<(String, String)> = told(&found)
            .into_iter()
            .filter(|told| told.4 == "item")
            .map(|told| (told.2, told.5))
            .collect();
        assert_eq!(
            items,
            [
                (format!("{payload}/result/1/item"), "demo:none".to_owned()),
                (
                    format!("{payload}/result/3/item"),
                    "demo:nothing".to_owned()
                ),
            ],
            "{found:#?}"
        );
    }
}
