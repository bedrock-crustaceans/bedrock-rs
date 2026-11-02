//! The crafting catalogue's link rows (20 M2, M3). The rows the load checks (M1, M4) are
//! pinned in `tests/catalog`.

#![allow(
    unused_imports,
    dead_code,
    reason = "which tests exist depends on the enabled features"
)]

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{LinkOptions, PackOrigin, PackRepository, PackSelection, WorldPacks};
use uuid::Uuid;

use super::super::common::{behavior_pack, leak, resource_pack};
use super::super::pack_common::bp;
use super::{said_of, text};

const NO_LINTS: LinkOptions = LinkOptions {
    lints: false,
    client: true,
};
const FILE: &str = "item_catalog/crafting_item_catalog.json";
const GROUP: &str = "minecraft:crafting_items_catalog/categories/0/groups/0";

/// The header UUID of the game's own behaviour pack, whose catalogue is the base.
const VANILLA: Uuid = Uuid::from_u128(0xfe9f8597_5454_481a_8730_8d070a8e2e58);

const EMPTY: &str =
    r#"{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {"categories": []}}"#;

/// A catalogue of one group `group` (its `group_identifier` object) of `construction` with
/// the JSON `items`.
fn catalog(group: &str, items: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {{"categories": [{{"category_name": "construction", "groups": [{{"group_identifier": {group}, "items": [{items}]}}]}}]}}}}"#
    )
}

/// An item `id` with the JSON `menu_category` (or none).
fn item(id: &str, menu: Option<&str>) -> String {
    let menu = menu.map_or_else(String::new, |menu| format!(r#", "menu_category": {menu}"#));
    format!(
        r#"{{"format_version": "1.21.60", "minecraft:item": {{"description": {{"identifier": "{id}"{menu}}}, "components": {{"minecraft:max_stack_size": 64}}}}}}"#
    )
}

/// The link findings of a world whose pack `bp 1` has `files`, above the game's pack with an
/// empty base catalogue, with the resource files `resource`.
fn link(
    files: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
    options: &LinkOptions,
) -> Diagnostics {
    let mut base = bp(9).file(FILE, leak(EMPTY));
    base.uuid = VANILLA;
    let world = behavior_pack(1, files);
    let resources = resource_pack(2, resource);
    let select = |pack: &bedrock_addon::pack::Pack| PackSelection {
        pack_id: pack.id().uuid,
        version: pack.id().version.clone(),
        subpack: None,
    };
    let selected = WorldPacks {
        behavior: vec![select(&world)],
        resource: vec![select(&resources)],
    };
    let mut repository = PackRepository::new();
    repository.add_pack(base.open(), PackOrigin::Package);
    repository.add_pack(world, PackOrigin::User);
    repository.add_pack(resources, PackOrigin::User);
    let set = repository.stacks(&selected, LoadOptions::default());
    // No finding of the load: the base catalogue is there and every placed item is one.
    let loud: Vec<String> = set
        .behavior
        .crafting_catalog()
        .diagnostics()
        .filter(|finding| finding.severity >= Severity::Warning)
        .map(ToString::to_string)
        .collect();
    assert_eq!(loud, [] as [String; 0]);
    set.link_with(options)
}

/// (code, severity, JSON path, vanilla text or the reference's name) of the findings in the
/// catalogue file.
fn told(found: &Diagnostics) -> Vec<(&'static str, Severity, String, String)> {
    found
        .iter()
        .filter(|finding| said_of(finding).2 == FILE)
        .map(|finding| {
            let said = said_of(finding);
            (said.0, said.1, said.3, text(finding).unwrap_or(said.5))
        })
        .collect()
}

// ---- M2 ------------------------------------------------------------------------------------

#[cfg(all(feature = "lang", feature = "languages"))]
mod group_name {
    use super::*;

    const LANGUAGES: (&str, &str) = ("texts/languages.json", r#"["en_US"]"#);

    #[test]
    fn m2_a_group_name_a_lang_file_defines_and_a_minecraft_group_are_no_finding() {
        let files = [(FILE, &*catalog(r#"{"name": "Demo:Gems"}"#, ""))];
        // Keys compare without case.
        let lang = ("texts/en_US.lang", "demo:gems=Gems\n");
        let found = link(&files, &[LANGUAGES, lang], &LinkOptions::default());
        assert_eq!(told(&found), []);
        // A group of the `minecraft` namespace is the game's: not looked up.
        let files = [(
            FILE,
            &*catalog(r#"{"name": "minecraft:itemGroup.name.no_such"}"#, ""),
        )];
        let found = link(&files, &[], &LinkOptions::default());
        assert_eq!(told(&found), []);
    }

    #[test]
    fn m2_a_group_name_no_lang_file_defines_is_an_information_at_the_name() {
        let files = [(FILE, &*catalog(r#"{"name": "demo:gems"}"#, ""))];
        let found = link(&files, &[], &LinkOptions::default());
        assert_eq!(
            told(&found),
            [(
                "UnresolvedLangKey",
                Severity::Info,
                format!("{GROUP}/group_identifier/name"),
                "demo:gems".to_owned()
            )]
        );
        let finding = found.iter().next().unwrap();
        assert_eq!(said_of(finding).4, "lang key");
        assert_eq!(
            finding.location.pack.as_ref().map(|pack| pack.name()),
            Some("bp 1")
        );
        // A lint.
        assert_eq!(told(&link(&files, &[], &NO_LINTS)), []);
    }
}

#[cfg(not(all(feature = "lang", feature = "languages")))]
#[test]
fn m2_without_the_text_feature_nothing_is_told() {
    let files = [(FILE, &*catalog(r#"{"name": "demo:gems"}"#, ""))];
    assert_eq!(told(&link(&files, &[], &LinkOptions::default())), []);
}

// ---- M3 ------------------------------------------------------------------------------------

#[test]
fn m3_an_item_placed_where_its_definition_says_or_without_a_category_is_no_finding() {
    for menu in [
        None,
        Some(r#"{"category": "construction"}"#),
        Some(r#"{"category": "construction", "group": "demo:gems"}"#),
    ] {
        let files = [
            (FILE, &*catalog(r#"{"name": "demo:gems"}"#, r#""demo:gem""#)),
            ("items/gem.json", &*item("demo:gem", menu)),
        ];
        let found = link(&files, &[], &NO_LINTS);
        assert_eq!(told(&found), [], "{menu:?}");
    }
}

#[test]
fn m3_an_item_placed_outside_its_own_category_and_group_is_vanillas_two_warnings() {
    let files = [
        (FILE, &*catalog(r#"{"name": "demo:gems"}"#, r#""demo:gem""#)),
        (
            "items/gem.json",
            &*item(
                "demo:gem",
                Some(r#"{"category": "nature", "group": "demo:stones"}"#),
            ),
        ),
    ];
    // Warnings: told without the lints too.
    let found = link(&files, &[], &NO_LINTS);
    assert_eq!(
        told(&found),
        [
            (
                "InvalidValue",
                Severity::Warning,
                format!("{GROUP}/items/0"),
                "The item demo:gem was created with the category set to 'nature', and is now being set to 'construction'".to_owned()
            ),
            (
                "InvalidValue",
                Severity::Warning,
                format!("{GROUP}/items/0"),
                "The item demo:gem was created with the group set to 'demo:stones', but is now being set to 'demo:gems'".to_owned()
            ),
        ]
    );
    let finding = found.iter().next().unwrap();
    assert_eq!(
        finding.location.pack.as_ref().map(|pack| pack.name()),
        Some("bp 1")
    );
}
