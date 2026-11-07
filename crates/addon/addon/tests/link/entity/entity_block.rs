//! 20 E10 and E11: the blocks an entity's components name.

#![allow(unused_imports, reason = "each feature set compiles part of the tests")]

use bedrock_addon::diagnostic::{DiagnosticCode, Severity, VanillaText};

use super::super::common::{hosted, link, link_with};
use super::{FILE, block, entity_with, kept_unresolved, pet, said, said_of, unresolved};
use bedrock_addon::pack::LinkOptions;

const EAT: &str = r#""minecraft:behavior.eat_block": {"eat_and_replace_block_pairs": [
    {"eat_block": "grass_block", "replace_block": "dirt"},
    {"eat_block": "demo:lamp", "replace_block": "demo:nothing"},
    {"eat_block": "demo:none", "replace_block": "air"}]}"#;

#[cfg(feature = "block")]
#[test]
fn e10_an_eat_block_pair_naming_a_block_nothing_defines_is_an_error_with_vanillas_text() {
    let found = link(
        &[
            (FILE, &entity_with("1.26.40", "demo:pet", EAT, "")),
            ("blocks/lamp.json", &block("demo:lamp")),
        ],
        &[],
    );
    let at = "minecraft:entity/components/minecraft:behavior.eat_block/eat_and_replace_block_pairs";
    assert_eq!(
        said_of(&found, "block"),
        [
            super::miss(
                "UnresolvedReference",
                Severity::Error,
                &format!("{at}/1/replace_block"),
                "block",
                "demo:nothing"
            ),
            super::miss(
                "UnresolvedReference",
                Severity::Error,
                &format!("{at}/2/eat_block"),
                "block",
                "demo:none"
            ),
        ]
    );
    let texts: Vec<_> = found.iter().map(|finding| finding.code.clone()).collect();
    let legacy = DiagnosticCode::UnresolvedReference {
        text: Some(VanillaText::EatBlockInvalidBlock {
            block: "demo:nothing".to_owned(),
        }),
    };
    assert!(
        bedrock_addon::diagnostic::testing::same_line(&texts[0], &legacy),
        "{:?}",
        texts[0]
    );
}

/// Below the goal's typed version (1.26.40) the goal was read by the lenient conversion, and
/// the same miss is a warning.
#[cfg(feature = "block")]
#[test]
fn e10_in_a_document_older_than_the_typed_goal_the_miss_is_a_warning() {
    let pairs = r#""minecraft:behavior.eat_block": {"eat_and_replace_block_pairs": [{"eat_block": "demo:none", "replace_block": "air"}]}"#;
    for (version, severity) in [
        ("1.26.30", Severity::Error),
        ("1.26.40", Severity::Error),
        ("1.26.50", Severity::Error),
    ] {
        let found = link(&[(FILE, &entity_with(version, "demo:pet", pairs, ""))], &[]);
        let told = said_of(&found, "block");
        assert_eq!(told.len(), 1, "{version}: {found:?}");
        assert_eq!(told[0].1, severity, "{version}");
        assert_eq!(told[0].5, "demo:none");
    }
}

/// A group's goal is checked like the entity's own, and one pair is told once however many of
/// its two names miss.
#[cfg(feature = "block")]
#[test]
fn e10_a_pair_of_a_component_group_is_told_once_at_its_first_unknown_name() {
    let text = entity_with(
        "1.26.50",
        "demo:pet",
        "",
        r#", "component_groups": {"hungry": {"minecraft:behavior.eat_block": {"eat_and_replace_block_pairs": [{"eat_block": "demo:a", "replace_block": "demo:b"}]}}}"#,
    );
    let found = link(&[(FILE, &text)], &[]);
    assert_eq!(
        said_of(&found, "block"),
        [super::miss(
            "UnresolvedReference",
            Severity::Error,
            "minecraft:entity/component_groups/hungry/minecraft:behavior.eat_block/eat_and_replace_block_pairs/0/eat_block",
            "block",
            "demo:a"
        )]
    );
}

/// With a host, the host's blocks are the game's: a vanilla name it does not list is missing,
/// one it lists is not.
#[cfg(feature = "block")]
#[test]
fn e10_a_host_answers_for_the_games_blocks() {
    let pairs = r#""minecraft:behavior.eat_block": {"eat_and_replace_block_pairs": [{"eat_block": "grass_block", "replace_block": "minecraft:dirt"}]}"#;
    let files = [(FILE, &*pet(pairs))];
    assert!(link(&files, &[]).is_empty());
    let found = link_with(
        &files,
        &[],
        hosted(&[("block", "minecraft:grass_block")]),
        LinkOptions::default(),
    );
    let told = said_of(&found, "block");
    assert_eq!(told.len(), 1, "{found:?}");
    assert_eq!(told[0].5, "minecraft:dirt");
    let both = hosted(&[
        ("block", "minecraft:grass_block"),
        ("block", "minecraft:dirt"),
    ]);
    assert!(link_with(&files, &[], both, LinkOptions::default()).is_empty());
}

/// Without the `block` feature no block can be known, and nothing is told.
#[cfg(not(feature = "block"))]
#[test]
fn e10_e11_without_the_block_feature_no_block_reference_is_told() {
    let components = format!(
        r#"{EAT}, "minecraft:trail": {{"block_type": "demo:none"}}, "minecraft:home": {{"home_block_list": ["demo:none"]}}"#
    );
    let found = link(&[(FILE, &pet(&components))], &[]);
    assert!(said_of(&found, "block").is_empty(), "{found:?}");
    let _ = (unresolved("", "", ""), block(""));
    let _ = (hosted(&[]), link_with as fn(_, _, _, _) -> _);
}

/// E11 over the kinds of member there are: a component the game reads at run time (`trail`,
/// `buoyant`, `navigation.walk`), a typed component (`home`, `preferred_path`), a typed goal
/// (`take_block`, `raid_garden`, `random_search_and_dig`) and a descriptor in an object.
///
/// A block descriptor's name is resolved on first use by
/// `BlockDescriptor::ResolveHelper::resolve`, which logs "Unknown block during Deferred
/// `BlockDescriptor` resolution: %s" and goes on: a
/// kept error with that text. The two lists of plain names (`home.home_block_list`,
/// `behavior.raid_garden.blocks`) are bound as names, not
/// descriptors, and no line was found for them: informations.
#[cfg(feature = "block")]
#[test]
fn e11_a_block_list_naming_a_block_nothing_defines_is_a_kept_error_a_name_an_information() {
    let components = r#"
        "minecraft:trail": {"block_type": "demo:none1"},
        "minecraft:buoyant": {"liquid_blocks": ["minecraft:water", "demo:none2", "demo:lamp"]},
        "minecraft:navigation.walk": {"blocks_to_avoid": [{"name": "demo:none3"}, {"tags": "query.any_tag('trapdoors')"}, "demo:lamp"]},
        "minecraft:home": {"home_block_list": ["minecraft:bee_nest", "demo:none4"]},
        "minecraft:preferred_path": {"preferred_path_blocks": [{"cost": 1, "blocks": ["grass_path", "demo:none5"]}]},
        "minecraft:behavior.take_block": {"priority": 1, "blocks": ["demo:none6", "dirt"]},
        "minecraft:behavior.raid_garden": {"priority": 1, "blocks": ["demo:none7", "carrots"]},
        "minecraft:behavior.random_search_and_dig": {"priority": 1, "target_blocks": ["demo:none8"]},
        "minecraft:inside_block_notifier": {"block_list": [{"block": {"name": "demo:none9"}}, {"block": {"name": "minecraft:bubble_column"}}]}
    "#;
    let found = link(
        &[
            (FILE, &pet(components)),
            ("blocks/lamp.json", &block("demo:lamp")),
        ],
        &[],
    );
    let c = "components/minecraft:";
    assert_eq!(
        said_of(&found, "block"),
        [
            kept_unresolved(&format!("{c}trail/block_type"), "block", "demo:none1"),
            kept_unresolved(
                &format!("{c}buoyant/liquid_blocks/1"),
                "block",
                "demo:none2"
            ),
            kept_unresolved(
                &format!("{c}navigation.walk/blocks_to_avoid/0/name"),
                "block",
                "demo:none3"
            ),
            unresolved(&format!("{c}home/home_block_list/1"), "block", "demo:none4"),
            kept_unresolved(
                &format!("{c}preferred_path/preferred_path_blocks/0/blocks/1"),
                "block",
                "demo:none5"
            ),
            kept_unresolved(
                &format!("{c}behavior.take_block/blocks/0"),
                "block",
                "demo:none6"
            ),
            unresolved(
                &format!("{c}behavior.raid_garden/blocks/0"),
                "block",
                "demo:none7"
            ),
            kept_unresolved(
                &format!("{c}behavior.random_search_and_dig/target_blocks/0"),
                "block",
                "demo:none8"
            ),
            kept_unresolved(
                &format!("{c}inside_block_notifier/block_list/0/block/name"),
                "block",
                "demo:none9"
            ),
        ]
    );
    // Each error is kept and carries the game's line; an information has no text.
    for finding in found.iter().filter(|finding| said(finding).4 == "block") {
        assert_eq!(finding.code.name(), "UnresolvedReference", "{finding:?}");
        let text = finding.vanilla();
        if finding.severity == Severity::Error {
            assert!(finding.kept, "{finding:?}");
            assert!(
                text.as_ref().is_some_and(|text| {
                    text.starts_with(
                        "Unknown block during Deferred BlockDescriptor resolution: demo:none",
                    )
                }),
                "{finding:?}"
            );
        } else {
            assert_eq!(text, None, "{finding:?}");
        }
    }
}

/// E11 in `inside_block_notifier`: a state the block does not have is the typed node's own
/// line, "Block of type '%s' does not have the state '%s'. This block will not be added to
/// `InsideBlockNotifierComponent`'s watch list due to an invalid state."
/// at the content log's error level, after which the entry is left out and the
/// entity loads: a kept error with that text. In every other row a
/// state is the crate's own check, an information.
#[cfg(feature = "block")]
#[test]
fn e11_a_state_the_block_does_not_have_is_the_notifiers_kept_error_and_another_rows_information() {
    let components = r#"
        "minecraft:inside_block_notifier": {"block_list": [{"block": {"name": "demo:lamp", "states": {"demo:dim": true}}}, {"block": {"name": "demo:lamp", "states": {"demo:lit": 3}}}]},
        "minecraft:trail": {"block_type": {"name": "demo:lamp", "states": {"demo:dim": true}}}
    "#;
    let lamp = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp", "states": {"demo:lit": [false, true]}}, "components": {}}}"#;
    let found = link(&[(FILE, &pet(components)), ("blocks/lamp.json", lamp)], &[]);
    let c = "components/minecraft:";
    assert_eq!(
        said_of(&found, "block state"),
        [
            kept_unresolved(
                &format!("{c}inside_block_notifier/block_list/0/block/states/demo:dim"),
                "block state",
                "demo:dim"
            ),
            super::miss(
                "InvalidReference",
                Severity::Info,
                &format!(
                    "minecraft:entity/{c}inside_block_notifier/block_list/1/block/states/demo:lit"
                ),
                "block state",
                "demo:lit"
            ),
            unresolved(
                &format!("{c}trail/block_type/states/demo:dim"),
                "block state",
                "demo:dim"
            ),
        ]
    );
    let kept = found
        .iter()
        .find(|finding| finding.severity == Severity::Error)
        .unwrap();
    assert!(kept.kept, "{kept:?}");
    let legacy = DiagnosticCode::UnresolvedReference {
        text: Some(VanillaText::Formatted {
            template: "Block of type '{}' does not have the state '{}'. This block will not be added to InsideBlockNotifierComponent's watch list due to an invalid state.",
            args: vec!["demo:lamp".to_owned(), "demo:dim".to_owned()],
        }),
    };
    assert!(
        bedrock_addon::diagnostic::testing::same_line(&kept.code, &legacy),
        "{:?}",
        kept.code
    );
}

/// E11 in a document below the typed version of a component or a goal: the lenient
/// conversion leaves a list element out that the typed member does not take. A name behind
/// it is at the index the file has it at, at every level.
#[cfg(feature = "block")]
#[test]
fn e11_a_name_behind_an_element_the_lenient_conversion_leaves_out_is_at_the_index_of_the_file() {
    let components = r#"
        "minecraft:home": {"home_block_list": [7, "demo:none1", {}, "demo:none2"]},
        "minecraft:preferred_path": {"preferred_path_blocks": [7, {"cost": 1, "blocks": [7, "demo:none3"]}]},
        "minecraft:behavior.raid_garden": {"priority": 1, "blocks": [7, "demo:none4", 8, "demo:none5"]}
    "#;
    // Below 1.15.0, where the game's legacy readers take a value of another type out alone
    // (from 1.15.0 it defaults the whole component).
    let text = entity_with("1.14.0", "demo:pet", components, "");
    let found = link(&[(FILE, &text)], &[]);
    let c = "components/minecraft:";
    assert_eq!(
        said_of(&found, "block"),
        [
            unresolved(&format!("{c}home/home_block_list/1"), "block", "demo:none1"),
            unresolved(&format!("{c}home/home_block_list/3"), "block", "demo:none2"),
            kept_unresolved(
                &format!("{c}preferred_path/preferred_path_blocks/1/blocks/1"),
                "block",
                "demo:none3"
            ),
            unresolved(
                &format!("{c}behavior.raid_garden/blocks/1"),
                "block",
                "demo:none4"
            ),
            unresolved(
                &format!("{c}behavior.raid_garden/blocks/3"),
                "block",
                "demo:none5"
            ),
        ],
        "{found:#?}"
    );
}

/// The two lists of block NAMES (`home.home_block_list`, `behavior.raid_garden.blocks`): each
/// name is linked as a block name. A block of the stack and a block of the game resolve; a
/// name nothing defines is `UnresolvedReference`, an information at its element (the game
/// binds them as names, not descriptors, and the game logs no line).
#[cfg(feature = "block")]
#[test]
fn e11_a_name_of_a_list_of_block_names_is_linked_as_a_block_name() {
    for (component, member) in [
        ("minecraft:home", "home_block_list"),
        ("minecraft:behavior.raid_garden", "blocks"),
    ] {
        let with = |names: &str| {
            let priority = if component.contains("behavior.") {
                r#""priority": 1, "#
            } else {
                ""
            };
            let components = format!(r#""{component}": {{{priority}"{member}": [{names}]}}"#);
            link(
                &[
                    (FILE, &pet(&components)),
                    ("blocks/lamp.json", &block("demo:lamp")),
                ],
                &[],
            )
        };
        let found = with(r#""demo:lamp", "minecraft:carrots", "carrots""#);
        assert!(
            said_of(&found, "block").is_empty(),
            "{component}: {found:?}"
        );
        let found = with(r#""demo:lamp", "demo:none", "minecraft:carrots", "none_either""#);
        let at = format!("components/{component}/{member}");
        assert_eq!(
            said_of(&found, "block"),
            [
                unresolved(&format!("{at}/1"), "block", "demo:none"),
                unresolved(&format!("{at}/3"), "block", "none_either"),
            ],
            "{component}"
        );
        assert!(
            found
                .iter()
                .all(|finding| finding.severity == Severity::Info
                    && finding.code.name() == "UnresolvedReference"),
            "{component}: {found:?}"
        );
    }
}

/// The members of a component group are checked at the group's path, and a member the game
/// reads as one block or a list is checked in either form.
#[cfg(feature = "block")]
#[test]
fn e11_a_group_and_a_single_block_where_a_list_may_stand() {
    let text = entity_with(
        "1.26.50",
        "demo:pet",
        r#""minecraft:breathable": {"breathe_blocks": "demo:one", "non_breathe_blocks": ["demo:two"]}"#,
        r#", "component_groups": {"g": {"minecraft:trail": {"block_type": "demo:three"}}}"#,
    );
    let found = link(&[(FILE, &text)], &[]);
    assert_eq!(
        said_of(&found, "block"),
        [
            kept_unresolved(
                "components/minecraft:breathable/breathe_blocks",
                "block",
                "demo:one"
            ),
            kept_unresolved(
                "components/minecraft:breathable/non_breathe_blocks/0",
                "block",
                "demo:two"
            ),
            kept_unresolved(
                "component_groups/g/minecraft:trail/block_type",
                "block",
                "demo:three"
            ),
        ]
    );
}

/// The game's own blocks resolve without a host by the crate's list, bare or with the
/// namespace; with a host only the host's do.
#[cfg(feature = "block")]
#[test]
fn e11_the_games_blocks_resolve_with_a_host_and_without_one() {
    let files = [(
        FILE,
        &*pet(r#""minecraft:buoyant": {"liquid_blocks": ["water", "minecraft:lava"]}"#),
    )];
    assert!(link(&files, &[]).is_empty());
    let found = link_with(
        &files,
        &[],
        hosted(&[("block", "minecraft:water")]),
        LinkOptions::default(),
    );
    assert_eq!(
        said_of(&found, "block"),
        [kept_unresolved(
            "components/minecraft:buoyant/liquid_blocks/1",
            "block",
            "minecraft:lava"
        )]
    );
}

/// A goal's blocks that the game reads as item descriptors (`avoid_block.target_blocks`,
/// `move_to_block.target_blocks`, `jump_to_block`'s lists) name blocks all the same: a name
/// that is a block or an item resolves, another is a missing block.
#[cfg(all(feature = "block", feature = "item"))]
#[test]
fn e11_a_goals_target_blocks_are_blocks_or_the_items_of_blocks() {
    let components = r#"
        "minecraft:behavior.avoid_block": {"priority": 1, "target_blocks": ["demo:lamp", "demo:gem", {"name": "demo:none1"}]},
        "minecraft:behavior.move_to_block": {"priority": 1, "target_blocks": ["demo:none2", "minecraft:wheat"]},
        "minecraft:behavior.jump_to_block": {"priority": 1, "preferred_blocks": ["demo:none3"], "forbidden_blocks": [{"name": "minecraft:water"}, "demo:none4"]}
    "#;
    let found = link(
        &[
            (FILE, &pet(components)),
            ("blocks/lamp.json", &block("demo:lamp")),
            ("items/gem.json", &super::item("demo:gem")),
        ],
        &[],
    );
    let c = "components/minecraft:behavior.";
    assert_eq!(
        said_of(&found, "block"),
        [
            unresolved(
                &format!("{c}avoid_block/target_blocks/2/name"),
                "block",
                "demo:none1"
            ),
            unresolved(
                &format!("{c}move_to_block/target_blocks/0"),
                "block",
                "demo:none2"
            ),
            // Within one component the members are told in the order of the rule's table.
            unresolved(
                &format!("{c}jump_to_block/forbidden_blocks/1"),
                "block",
                "demo:none4"
            ),
            unresolved(
                &format!("{c}jump_to_block/preferred_blocks/0"),
                "block",
                "demo:none3"
            ),
        ]
    );
}

/// A tag expression is the game's to evaluate (`{tags}`: run), and a material name is no
/// block (`lay_egg.target_materials_above_block`, `move_to_liquid.material_type`).
#[cfg(feature = "block")]
#[test]
fn e11_tags_and_materials_are_not_block_names() {
    let components = r#"
        "minecraft:navigation.walk": {"blocks_to_avoid": [{"tags": "query.any_tag('demo:none')"}]},
        "minecraft:behavior.lay_egg": {"priority": 1, "target_materials_above_block": ["Air"]},
        "minecraft:behavior.move_to_liquid": {"priority": 1, "material_type": "Lava"}
    "#;
    let found = link(&[(FILE, &pet(components))], &[]);
    assert!(said_of(&found, "block").is_empty(), "{found:?}");
}

/// The game's own entity files still write block names of older versions (`grass`,
/// `tallgrass`, `stonecutter`, `minecraft:item.soul_campfire`) that the list of the game's
/// blocks lacks: each resolves as the files write it, with any host, in a block list and in
/// an `eat_block` pair.
#[cfg(feature = "block")]
#[test]
fn e10_e11_a_legacy_block_name_of_the_games_own_files_resolves() {
    let components = r#"
        "minecraft:behavior.eat_block": {"eat_and_replace_block_pairs": [{"eat_block": "grass", "replace_block": "dirt"}, {"eat_block": "tallgrass", "replace_block": "air"}]},
        "minecraft:break_blocks": {"breakable_blocks": ["double_plant", "leaves", "leaves2", "red_flower", "sapling", "yellow_flower", "minecraft:grass"]},
        "minecraft:preferred_path": {"preferred_path_blocks": [{"cost": 1, "blocks": ["stonecutter", "stonebrick", "planks", "stone_slab", "stone_slab2", "stone_slab3", "stone_slab4", "double_stone_slab", "double_stone_slab2", "double_stone_slab3", "double_stone_slab4", "wooden_slab", "double_wooden_slab", "stained_glass"]}]},
        "minecraft:block_sensor": {"sensor_radius": 16, "on_break": [{"block_list": ["minecraft:shulker_box", "demo:none"], "on_block_broken": "x"}]},
        "minecraft:behavior.avoid_block": {"priority": 1, "target_blocks": ["minecraft:item.soul_campfire"]}
    "#;
    let files = [(FILE, &*pet(components))];
    let expected = [kept_unresolved(
        "components/minecraft:block_sensor/on_break/0/block_list/1",
        "block",
        "demo:none",
    )];
    assert_eq!(said_of(&link(&files, &[]), "block"), expected);
    // `stonecutter` is a block of the generated list, no legacy name: a
    // host is asked for it as for `dirt` and `air`.
    let host = hosted(&[
        ("block", "minecraft:dirt"),
        ("block", "minecraft:air"),
        ("block", "minecraft:stonecutter"),
    ]);
    let found = link_with(&files, &[], host, LinkOptions::default());
    assert_eq!(said_of(&found, "block"), expected);
}

/// One rule of case for every name: a name is compared as written. A block of the game, a
/// legacy name and a pack's block in another case are all told (with the pack's name as the
/// hint where a pack defines it).
#[cfg(feature = "block")]
#[test]
fn e11_a_name_in_another_case_is_told_whatever_would_have_resolved_it() {
    use bedrock_addon::diagnostic::{DiagnosticDetail, ReferenceHint};
    let components = r#""minecraft:break_blocks": {"breakable_blocks": ["Minecraft:Stone", "Stone", "Grass", "minecraft:Tallgrass", "Demo:Lamp", "minecraft:stone", "grass", "demo:lamp"]}"#;
    let found = link(
        &[
            (FILE, &pet(components)),
            ("blocks/lamp.json", &block("demo:lamp")),
        ],
        &[],
    );
    let told: Vec<String> = said_of(&found, "block")
        .into_iter()
        .map(|said| said.5)
        .collect();
    assert_eq!(
        told,
        [
            "Minecraft:Stone",
            "Stone",
            "Grass",
            "minecraft:Tallgrass",
            "Demo:Lamp"
        ]
    );
    let hints: Vec<_> = found
        .iter()
        .filter_map(|finding| match finding.detail.as_deref() {
            Some(DiagnosticDetail::Reference { hint, .. }) => Some(hint.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        hints.last(),
        Some(&Some(ReferenceHint::CaseMismatch("demo:lamp".to_owned())))
    );
}
