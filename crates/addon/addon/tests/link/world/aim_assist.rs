//! 20 C3 (a camera preset's aim-assist preset), C4 (the categories an aim-assist preset names)
//! and C5 (its blocks, entities and items). C1, C2 and C6 are audited in `audit.rs`.

#![allow(unused_imports, reason = "which rows compile depends on the features")]

use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link, link_with};
use bedrock_addon::diagnostic::Severity;

use super::{Said, said, unresolved_error};

const PRESET_KIND: &str = "aim assist preset";
const CATEGORY_KIND: &str = "aim assist category";
const PRESET: &str = "aim_assist/presets/p.json";
const CATEGORIES: &str = "aim_assist/categories/c.json";

/// An aim-assist preset `demo:aim` at `version` with `members` after its identifier.
fn preset(version: &str, members: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:aim_assist_preset": {{"identifier": "demo:aim"{members}}}}}"#
    )
}

/// A categories file with the categories `categories` (the elements of the array).
fn categories(categories: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.50", "minecraft:aim_assist_categories": {{"categories": [{categories}]}}}}"#
    )
}

#[cfg(feature = "camera")]
/// A camera preset `demo:cam` that turns on the aim-assist preset `aim`.
fn camera(aim: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.60", "minecraft:camera_preset": {{"identifier": "demo:cam", "inherit_from": "minecraft:free", "aim_assist": {{"preset": "{aim}"}}}}}}"#
    )
}

// ---- C3 ------------------------------------------------------------------------------------

#[cfg(feature = "camera")]
#[test]
fn c3_an_aim_assist_preset_a_pack_defines_or_the_game_has_resolves() {
    let aim = preset("1.21.120", "");
    // The source member is compared in lower case, as the game registers every name.
    for name in ["demo:aim", "Demo:Aim", "minecraft:aim_assist_default"] {
        let camera = camera(name);
        let found = link(
            &[(PRESET, aim.as_str()), ("cameras/presets/c.json", &camera)],
            &[],
        );
        assert_eq!(said(&found, &[PRESET_KIND]), [], "{name}");
    }
    // A host answers for a preset of the game the crate does not know.
    let camera = camera("minecraft:other_default");
    let found = link_with(
        &[("cameras/presets/c.json", &camera)],
        &[],
        hosted(&[
            ("aim assist preset", "minecraft:other_default"),
            ("camera preset", "minecraft:free"),
        ]),
        LinkOptions::default(),
    );
    assert_eq!(said(&found, &[PRESET_KIND]), []);
}

#[cfg(feature = "camera")]
#[test]
fn c3_an_aim_assist_preset_nothing_defines_is_a_warning_at_the_member() {
    let (aim, camera) = (preset("1.21.120", ""), camera("demo:other"));
    let found = link(
        &[(PRESET, aim.as_str()), ("cameras/presets/c.json", &camera)],
        &[],
    );
    assert_eq!(
        said(&found, &[PRESET_KIND]),
        [unresolved_info(
            "cameras/presets/c.json",
            "minecraft:camera_preset/aim_assist/preset",
            PRESET_KIND,
            "demo:other"
        )]
    );
}

// ---- C4 ------------------------------------------------------------------------------------

#[test]
fn c4_a_category_a_pack_defines_or_the_game_has_resolves() {
    let aim = preset(
        "1.21.120",
        r#", "item_settings": {"minecraft:bow": "demo:ranged", "minecraft:stick": "Demo:Ranged", "minecraft:bucket": "minecraft:bucket"}, "default_item_settings": "minecraft:default", "hand_settings": "minecraft:empty_hand""#,
    );
    let ranged = categories(r#"{"name": "demo:ranged", "priorities": {}}"#);
    let found = link(&[(PRESET, aim.as_str()), (CATEGORIES, &ranged)], &[]);
    assert_eq!(said(&found, &[CATEGORY_KIND]), []);
    // A host answers for a category of the game the crate does not know.
    let aim = preset("1.21.120", r#", "hand_settings": "minecraft:fist""#);
    let found = link_with(
        &[(PRESET, aim.as_str())],
        &[],
        hosted(&[("aim assist category", "minecraft:fist")]),
        LinkOptions::default(),
    );
    assert_eq!(said(&found, &[CATEGORY_KIND]), []);
}

#[test]
fn c4_a_category_nothing_defines_is_an_error_at_each_member() {
    let aim = preset(
        "1.21.120",
        r#", "item_settings": {"minecraft:bow": "demo:a"}, "default_item_settings": "demo:b", "hand_settings": "demo:c""#,
    );
    let found = link(&[(PRESET, aim.as_str())], &[]);
    let at = |member: &str| format!("minecraft:aim_assist_preset/{member}");
    assert_eq!(
        said(&found, &[CATEGORY_KIND]),
        [
            unresolved_error(
                PRESET,
                &at("item_settings/minecraft:bow"),
                CATEGORY_KIND,
                "demo:a"
            ),
            unresolved_error(
                PRESET,
                &at("default_item_settings"),
                CATEGORY_KIND,
                "demo:b"
            ),
            unresolved_error(PRESET, &at("hand_settings"), CATEGORY_KIND, "demo:c"),
        ]
    );
    // The game's own categories are the crate's list only while no host answers.
    let aim = preset("1.21.120", r#", "hand_settings": "minecraft:default""#);
    let found = link_with(
        &[(PRESET, aim.as_str())],
        &[],
        hosted(&[]),
        LinkOptions::default(),
    );
    assert_eq!(said(&found, &[CATEGORY_KIND]).len(), 1);
}

// ---- C5 ------------------------------------------------------------------------------------

/// An `UnresolvedReference` information: a name the game keeps without a word.
fn unresolved_info(file: &str, path: &str, kind: &'static str, id: &str) -> Said {
    super::finding(
        "core/unresolved_reference",
        Severity::Info,
        file,
        path,
        kind,
        id,
    )
}

/// A preset and a category that name the block, the entity and the item `name` in every
/// member of the row.
fn naming(block: &str, entity: &str, item: &str) -> (String, String) {
    let aim = preset(
        "1.21.120",
        &format!(
            r#", "exclusion_settings": {{"blocks": ["{block}"], "entities": ["{entity}"]}}, "liquid_targeting_list": ["{item}"], "item_settings": {{"{item}": "minecraft:default"}}"#
        ),
    );
    let targets = categories(&format!(
        r#"{{"name": "demo:targets", "priorities": {{"entities": {{"{entity}": 1}}, "blocks": {{"{block}": 2}}}}}}"#
    ));
    (aim, targets)
}

#[cfg(all(feature = "block", feature = "entity", feature = "item"))]
#[test]
fn c5_names_a_pack_defines_or_the_game_has_resolve_in_their_normalised_form() {
    use super::{block, entity, item};
    let (lamp, pet, wand) = (block("demo:lamp"), entity("demo:pet"), item("demo:wand"));
    let defined = [
        ("blocks/lamp.json", lamp.as_str()),
        ("entities/pet.json", pet.as_str()),
        ("items/wand.json", wand.as_str()),
    ];
    for (block, entity, item) in [
        ("demo:lamp", "demo:pet", "demo:wand"),
        // Lower-cased; an entity without its event; an item without its auxiliary value.
        (
            "Demo:Lamp",
            "Demo:Pet<minecraft:entity_born>",
            "Demo:Wand:3",
        ),
        ("minecraft:stone", "minecraft:pig", "minecraft:bucket"),
        (
            "minecraft:STONE",
            "minecraft:PIG<minecraft:entity_born>",
            "minecraft:Bucket:1",
        ),
    ] {
        let (aim, targets) = naming(block, entity, item);
        let mut files = defined.to_vec();
        files.push((PRESET, aim.as_str()));
        files.push((CATEGORIES, targets.as_str()));
        let found = link(&files, &[]);
        assert_eq!(
            said(&found, &["block", "entity", "item"]),
            [],
            "{block} {entity} {item}"
        );
    }
}

#[cfg(all(feature = "block", feature = "entity", feature = "item"))]
#[test]
fn c5_names_nothing_defines_are_told_at_each_member() {
    let (aim, targets) = naming(
        "demo:Lamp",
        "demo:Pet<minecraft:entity_born>",
        "demo:Wand:3",
    );
    let files = [(PRESET, aim.as_str()), (CATEGORIES, targets.as_str())];
    let found = link(&files, &[]);
    let category = "minecraft:aim_assist_categories/categories/0/priorities";
    let preset = "minecraft:aim_assist_preset";
    // The keys of a category's priorities and of a preset's `item_settings` are no references
    // the game's validator holds (`CategoryPriorities`: an associative container with no key
    // constraint): the game keeps them silently, so the finding is ours, an
    // information.
    let expected = [
        unresolved_info(
            CATEGORIES,
            &format!("{category}/entities/demo:Pet<minecraft:entity_born>"),
            "entity",
            "demo:Pet<minecraft:entity_born>",
        ),
        unresolved_info(
            CATEGORIES,
            &format!("{category}/blocks/demo:Lamp"),
            "block",
            "demo:Lamp",
        ),
        unresolved_error(
            PRESET,
            &format!("{preset}/exclusion_settings/blocks/0"),
            "block",
            "demo:Lamp",
        ),
        unresolved_error(
            PRESET,
            &format!("{preset}/exclusion_settings/entities/0"),
            "entity",
            "demo:Pet<minecraft:entity_born>",
        ),
        unresolved_error(
            PRESET,
            &format!("{preset}/liquid_targeting_list/0"),
            "item",
            "demo:Wand:3",
        ),
        unresolved_info(
            PRESET,
            &format!("{preset}/item_settings/demo:Wand:3"),
            "item",
            "demo:Wand:3",
        ),
    ];
    assert_eq!(said(&found, &["block", "entity", "item"]), expected);
    // A host is asked for the normalised names, and one it does not have stays missing.
    let found = link_with(
        &files,
        &[],
        hosted(&[
            ("block", "demo:lamp"),
            ("entity", "demo:pet"),
            ("item", "demo:wand"),
        ]),
        LinkOptions::default(),
    );
    assert_eq!(said(&found, &["block", "entity", "item"]), []);
}

/// An entry of a 1.21.50 `exclusion_list` is a block or an entity: it resolves as either, and
/// one that is neither is told once, where the file has it.
#[cfg(all(feature = "block", feature = "entity"))]
#[test]
fn c5_an_entry_of_an_old_exclusion_list_is_a_block_or_an_entity() {
    let aim = preset(
        "1.21.50",
        r#", "exclusion_list": ["minecraft:stone", "minecraft:pig", "demo:neither"]"#,
    );
    let found = link(&[(PRESET, aim.as_str())], &[]);
    assert_eq!(
        said(&found, &["block", "entity", "block or entity"]),
        [unresolved_error(
            PRESET,
            "minecraft:aim_assist_preset/exclusion_list/2",
            "block or entity",
            "demo:neither"
        )]
    );
}

/// The identifier of the game's `pillager_patrol` spawn rules is no
/// entity an aim-assist preset can exclude.
#[cfg(all(feature = "block", feature = "entity"))]
#[test]
fn c5_the_identifier_of_the_games_patrol_spawn_rules_is_no_entity() {
    let aim = preset(
        "1.21.50",
        r#", "exclusion_list": ["minecraft:pig", "minecraft:pillager_patrol"]"#,
    );
    let found = link(&[(PRESET, aim.as_str())], &[]);
    assert_eq!(
        said(&found, &["block", "entity", "block or entity"]),
        [unresolved_error(
            PRESET,
            "minecraft:aim_assist_preset/exclusion_list/1",
            "block or entity",
            "minecraft:pillager_patrol"
        )]
    );
}

/// Without the module of a target kind its names are not looked up.
#[cfg(not(any(feature = "block", feature = "entity", feature = "item")))]
#[test]
fn c5_without_the_modules_of_the_targets_no_name_is_reported() {
    let (aim, targets) = naming("demo:lamp", "demo:pet", "demo:wand");
    let found = link(
        &[(PRESET, aim.as_str()), (CATEGORIES, targets.as_str())],
        &[],
    );
    assert_eq!(said(&found, &["block", "entity", "item"]), []);
}
