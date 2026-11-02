//! 20 E15: the items an entity's components name.

#![allow(unused_imports, reason = "each feature set compiles part of the tests")]

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link, link_with};
use super::{FILE, block, item, miss, pet, said_of, unresolved};

/// The members of E15 over the kinds there are (a component the game reads at run time, a
/// typed component, a typed goal), each naming an item nothing defines.
const COMPONENTS: &str = r#"
    "minecraft:ageable": {"feed_items": ["demo:none1", {"item": "demo:none2", "result_item": "demo:none3"}], "drop_items": ["demo:none4"]},
    "minecraft:breedable": {"breed_items": ["demo:none6"]},
    "minecraft:tameable": {"tame_items": ["demo:none7"]},
    "minecraft:healable": {"items": [{"item": "demo:none8", "heal_amount": 1}]},
    "minecraft:giveable": {"triggers": {"items": ["demo:none9"]}},
    "minecraft:equippable": {"slots": [{"slot": 0, "item": "demo:none10", "accepted_items": ["demo:none11"]}]},
    "minecraft:shareables": {"items": [{"item": "demo:none12", "craft_into": "demo:none13"}]},
    "minecraft:bribeable": {"bribe_items": ["demo:none14"]},
    "minecraft:boostable": {"boost_items": [{"item": "demo:none15", "replace_item": "demo:none16"}]},
    "minecraft:item_controllable": {"control_items": "demo:none17"},
    "minecraft:trusting": {"trust_items": ["demo:none18"]},
    "minecraft:spawn_entity": {"entities": [{"spawn_item": "demo:none19"}]},
    "minecraft:equip_item": {"excluded_items": [{"item": "demo:none20"}]},
    "minecraft:tamemount": {"feed_items": [{"item": "demo:none21", "temper_mod": 1}], "auto_reject_items": [{"item": "demo:none22"}]},
    "minecraft:interact": {"interactions": [{"transform_to_item": "demo:none23"}]},
    "minecraft:behavior.beg": {"priority": 1, "items": ["demo:none24"]},
    "minecraft:behavior.snacking": {"priority": 1, "items": [{"name": "demo:none25"}]},
    "minecraft:behavior.pickup_items": {"priority": 1, "excluded_items": ["demo:none26"]},
    "minecraft:behavior.tempt": {"priority": 1, "items": ["demo:none27"]},
    "minecraft:behavior.charge_held_item": {"priority": 1, "items": ["demo:none28"]}
"#;

/// an older file's `ageable.transform_to_item` / `breedable.transform_to_item` is the
/// `result_item` of each entry of `feed_items` / `breed_items` from 1.21.130
/// (the game's upgrade of both components): the item is linked as that `result_item`, at the place the file has.
#[cfg(feature = "item")]
#[test]
fn e15_a_legacy_transform_to_item_is_linked_as_the_result_item_it_becomes() {
    if !cfg!(feature = "block") {
        return;
    }
    let components = r#""minecraft:ageable": {"feed_items": ["minecraft:apple"], "transform_to_item": "demo:none5"}, "minecraft:breedable": {"breed_items": ["minecraft:apple"], "transform_to_item": "demo:none6"}"#;
    let text = super::entity_with("1.21.120", "demo:pet", components, "");
    let found = link(&[(FILE, &text)], &[]);
    let told: Vec<_> = said_of(&found, "item")
        .into_iter()
        .map(|said| format!("{said:?}"))
        .collect();
    assert_eq!(told.len(), 2, "{told:#?}");
    for id in ["demo:none5", "demo:none6"] {
        assert!(told.iter().any(|said| said.contains(id)), "{id}: {told:#?}");
    }
}

/// The game logs "Unknown item during Deferred ItemDescriptor
/// resolution: %s" at level 1 alone: an
/// information.
#[cfg(feature = "item")]
#[test]
fn e15_an_item_nothing_defines_is_an_information() {
    // Blocks are in view or not by the feature; a name outside `minecraft` that is no item
    // is missing only when they are.
    let found = link(&[(FILE, &pet(COMPONENTS))], &[]);
    let told = said_of(&found, "item");
    if !cfg!(feature = "block") {
        assert!(told.is_empty(), "{found:?}");
        return;
    }
    let c = "components/minecraft:";
    let i = |path: &str, id: &str| unresolved(&format!("{c}{path}"), "item", id);
    assert_eq!(
        told,
        [
            i("ageable/drop_items/0", "demo:none4"),
            i("ageable/feed_items/0", "demo:none1"),
            i("ageable/feed_items/1/item", "demo:none2"),
            i("ageable/feed_items/1/result_item", "demo:none3"),
            i("breedable/breed_items/0", "demo:none6"),
            i("tameable/tame_items/0", "demo:none7"),
            i("healable/items/0/item", "demo:none8"),
            i("giveable/triggers/items/0", "demo:none9"),
            i("equippable/slots/0/accepted_items/0", "demo:none11"),
            i("equippable/slots/0/item", "demo:none10"),
            // An item or an item tag: a lint (see the shareables test).
            miss(
                "UnresolvedReference",
                Severity::Info,
                "minecraft:entity/components/minecraft:shareables/items/0/item",
                "item",
                "demo:none12",
            ),
            i("shareables/items/0/craft_into", "demo:none13"),
            i("bribeable/bribe_items/0", "demo:none14"),
            i("boostable/boost_items/0/item", "demo:none15"),
            i("boostable/boost_items/0/replace_item", "demo:none16"),
            i("item_controllable/control_items", "demo:none17"),
            i("trusting/trust_items/0", "demo:none18"),
            i("spawn_entity/entities/0/spawn_item", "demo:none19"),
            i("equip_item/excluded_items/0/item", "demo:none20"),
            i("tamemount/auto_reject_items/0/item", "demo:none22"),
            i("tamemount/feed_items/0/item", "demo:none21"),
            i("interact/interactions/0/transform_to_item", "demo:none23"),
            i("behavior.beg/items/0", "demo:none24"),
            i("behavior.snacking/items/0/name", "demo:none25"),
            i("behavior.pickup_items/excluded_items/0", "demo:none26"),
            i("behavior.tempt/items/0", "demo:none27"),
            i("behavior.charge_held_item/items/0", "demo:none28"),
        ]
    );
}

/// An item a pack defines resolves, and so does the item of a block a pack defines; the
/// game's items resolve bare or with the namespace, and with the legacy `name:aux` form the
/// game's entity files write (`water_bucket:0`).
#[cfg(all(feature = "item", feature = "block"))]
#[test]
fn e15_a_packs_item_a_blocks_item_and_the_games_items_resolve() {
    let components = r#"
        "minecraft:ageable": {"feed_items": ["demo:gem", "demo:lamp", "wheat", "minecraft:apple", {"item": "fish", "result_item": "water_bucket:0"}], "transform_to_item": "bucket:0"},
        "minecraft:tameable": {"tame_items": ["demo:gem:3", "nothing:0", "minecraft:wool:3"]}
    "#;
    let found = link(
        &[
            (FILE, &pet(components)),
            ("items/gem.json", &item("demo:gem")),
            ("blocks/lamp.json", &block("demo:lamp")),
        ],
        &[],
    );
    assert_eq!(
        said_of(&found, "item"),
        [unresolved(
            "components/minecraft:tameable/tame_items/1",
            "item",
            "nothing:0"
        )]
    );
}

/// With a host only the host's items are the game's.
#[cfg(feature = "item")]
#[test]
fn e15_a_host_answers_for_the_games_items() {
    let components = r#""minecraft:bribeable": {"bribe_items": ["minecraft:apple", "wheat"]}"#;
    let files = [(FILE, &*pet(components))];
    let found = link_with(
        &files,
        &[],
        hosted(&[("item", "minecraft:apple")]),
        &LinkOptions::default(),
    );
    assert_eq!(
        said_of(&found, "item"),
        [unresolved(
            "components/minecraft:bribeable/bribe_items/1",
            "item",
            "wheat"
        )]
    );
}

/// An item tag no item carries is the shared resolver's lint; a tag expression is the
/// game's to evaluate.
#[cfg(feature = "item")]
#[test]
fn e15_an_item_tag_is_a_lint_and_a_tag_expression_is_not_checked() {
    let components = r#""minecraft:behavior.pickup_items": {"priority": 1, "excluded_items": [{"item_tag": "demo:none"}, {"tags": "q.all_tags('demo:none')"}]}"#;
    let files = [(FILE, &*pet(components))];
    let found = link(&files, &[]);
    assert!(said_of(&found, "item").is_empty(), "{found:?}");
    assert_eq!(
        said_of(&found, "item tag"),
        [miss(
            "UnresolvedReference",
            Severity::Info,
            "minecraft:entity/components/minecraft:behavior.pickup_items/excluded_items/0/item_tag",
            "item tag",
            "demo:none"
        )]
    );
    let quiet = link_with(
        &files,
        &[],
        bedrock_addon::load::LoadOptions::default(),
        &LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert!(quiet.is_empty(), "{quiet:?}");
}

#[cfg(not(feature = "item"))]
#[test]
fn e15_without_the_item_feature_no_item_is_told() {
    let found = link(&[(FILE, &pet(COMPONENTS))], &[]);
    assert!(said_of(&found, "item").is_empty(), "{found:?}");
    let _ = (
        block(""),
        item(""),
        hosted(&[]),
        miss("", Severity::Info, "", "", ""),
    );
    let _ = (
        LinkOptions::default(),
        link_with as fn(_, _, _, _) -> _,
        unresolved("", "", ""),
    );
}

/// The game's own entity files still write item names of older versions (`carrotOnAStick`,
/// `horsearmorleather`, `muttonRaw`) that the list of the game's items lacks: each resolves
/// in any case (the files write `muttonCooked` and `muttoncooked`), with any host; the fold
/// is those names' alone, and a current name in another case is told.
// K113: one list for every group, compared as the game does.
#[cfg(feature = "item")]
#[test]
fn e15_a_legacy_item_name_of_the_games_own_files_resolves() {
    let components = r#"
        "minecraft:behavior.tempt": {"priority": 1, "items": ["carrotOnAStick", "appleenchanted", "appleEnchanted", "minecraft:muttonCooked", "muttonraw", "minecraft:melon", "cooked_fish", "muttoncooked", "muttonCooked", "notAnItem", "CarrotOnAStick", "Apple"]},
        "minecraft:equippable": {"slots": [{"slot": 0, "item": "nautilusarmor", "accepted_items": ["horsearmorleather", "carpet"]}]},
        "minecraft:ageable": {"drop_items": ["turtle_shell_piece", "minecraft:emptyMap", "minecraft:record_13"]}
    "#;
    // (`minecraft:boat` left the legacy names when the generated list took the 1.26.60
    // items: a host is asked for it like any current name.)
    let files = [(FILE, &*pet(components))];
    let at = "components/minecraft:behavior.tempt/items";
    let expected = [
        unresolved(&format!("{at}/9"), "item", "notAnItem"),
        unresolved(&format!("{at}/11"), "item", "Apple"),
    ];
    assert_eq!(said_of(&link(&files, &[]), "item"), expected);
    let found = link_with(&files, &[], hosted(&[]), &LinkOptions::default());
    assert_eq!(said_of(&found, "item"), expected);
}

/// `shareables.items[].item` is an item or an item tag (the game's fox shares
/// `minecraft:is_food`): a name that is an item, or a tag an item of the stack carries or the
/// host knows, resolves; any other is told as a lint, since without a host the game's tags
/// are not known.
#[cfg(all(feature = "item", feature = "block"))]
#[test]
fn e15_a_shareables_item_may_be_an_item_tag() {
    let tagged = r#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:gem"}, "components": {"minecraft:tags": {"tags": ["demo:shiny"]}}}}"#;
    let components = r#""minecraft:shareables": {"items": [{"item": "demo:gem"}, {"item": "demo:shiny"}, {"item": "minecraft:is_food"}, {"item": "minecraft:apple", "craft_into": "demo:none"}]}"#;
    let files = [(FILE, &*pet(components)), ("items/gem.json", tagged)];
    let found = link(&files, &[]);
    let at = "minecraft:entity/components/minecraft:shareables/items";
    assert_eq!(
        said_of(&found, "item"),
        [
            miss(
                "UnresolvedReference",
                Severity::Info,
                &format!("{at}/2/item"),
                "item",
                "minecraft:is_food"
            ),
            miss(
                "UnresolvedReference",
                Severity::Info,
                &format!("{at}/3/craft_into"),
                "item",
                "demo:none"
            ),
        ]
    );
    let host = hosted(&[
        ("item tag", "minecraft:is_food"),
        ("item", "minecraft:apple"),
    ]);
    let found = link_with(&files, &[], host, &LinkOptions::default());
    assert_eq!(said_of(&found, "item").len(), 1, "{found:?}");
    let quiet = link_with(
        &files,
        &[],
        bedrock_addon::load::LoadOptions::default(),
        &LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    // Both misses are informations now: `lints: false` drops them.
    assert_eq!(said_of(&quiet, "item").len(), 0, "{quiet:?}");
}
