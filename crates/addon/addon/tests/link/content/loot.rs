//! The references of a loot table (20 L1, L2, L5), the trim rules of its `set_armor_trim`
//! function, and the loot rows that are not the
//! link pass's.

#![allow(
    unused_imports,
    dead_code,
    reason = "which tests exist depends on the enabled features"
)]

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, stacks};
use super::{one, said, said_of, text};

const NO_LINTS: LinkOptions = LinkOptions {
    lints: false,
    client: true,
};
const FILE: &str = "loot_tables/refs.json";

/// The link findings of a world whose behaviour pack must load every loot table it has.
fn linked(
    behavior: &[(&'static str, &str)],
    load: LoadOptions,
    options: &LinkOptions,
) -> bedrock_addon::diagnostic::Diagnostics {
    let set = stacks(behavior, &[], load);
    let tables = behavior
        .iter()
        .filter(|(path, _)| path.starts_with("loot_tables/"))
        .count();
    assert_eq!(set.behavior.loot_tables().iter().count(), tables);
    set.link_with(options)
}

fn link(behavior: &[(&'static str, &str)]) -> bedrock_addon::diagnostic::Diagnostics {
    linked(behavior, LoadOptions::default(), &LinkOptions::default())
}

/// A table of one pool with the JSON `entries`.
fn table(entries: &str) -> String {
    format!(r#"{{"pools": [{{"rolls": 1, "entries": [{entries}]}}]}}"#)
}

// ---- L2: tables ----------------------------------------------------------------------------

const TABLE_KINDS: [&str; 1] = ["loot table"];
const THERE: (&str, &str) = ("loot_tables/chests/there.json", r#"{"pools": []}"#);

fn referring(name: &str) -> String {
    table(&format!(
        r#"{{"type": "loot_table", "name": "{name}"}}, {{"type": "item", "name": "minecraft:chest", "functions": [{{"function": "fill_container", "loot_table": "{name}"}}]}}"#
    ))
}

/// The source member's own rule (X16): everything up to the last `:` is no part of the
/// path, and the path is looked up lower-cased, with or without `.json`.
#[test]
fn l2_a_table_of_the_stack_is_no_finding_however_it_is_written() {
    for written in [
        "loot_tables/chests/there.json",
        "minecraft:loot_tables/chests/there.json",
        "a:b:loot_tables/chests/there.json",
        "loot_tables/Chests/There.json",
        "loot_tables/chests/there",
    ] {
        let found = link(&[(FILE, &referring(written)), THERE]);
        assert_eq!(said(&found, &TABLE_KINDS), [], "{written}");
    }
}

#[test]
fn l2_a_table_the_host_says_the_game_has_is_no_finding() {
    let host = hosted(&[("loot table", "loot_tables/chests/vanilla.json")]);
    let found = linked(
        &[(
            FILE,
            &referring("minecraft:loot_tables/chests/Vanilla.json"),
        )],
        host,
        &LinkOptions::default(),
    );
    assert_eq!(said(&found, &TABLE_KINDS), []);
}

#[test]
fn l2_a_table_nothing_defines_is_an_information_at_each_member_that_names_it() {
    let file = referring("minecraft:loot_tables/chests/missing.json");
    let found = link(&[(FILE, &file), THERE]);
    assert_eq!(
        said(&found, &TABLE_KINDS),
        [
            one(
                "UnresolvedReference",
                Severity::Info,
                FILE,
                "pools/0/entries/0/name",
                "loot table",
                "minecraft:loot_tables/chests/missing.json"
            ),
            one(
                "UnresolvedReference",
                Severity::Info,
                FILE,
                "pools/0/entries/1/functions/0/loot_table",
                "loot table",
                "minecraft:loot_tables/chests/missing.json"
            ),
        ]
    );
    let finding = found
        .iter()
        .find(|finding| said_of(finding).4 == "loot table")
        .unwrap();
    assert_eq!(
        finding.location.pack.as_ref().map(|pack| pack.name()),
        Some("bp 1")
    );
    assert_eq!(text(finding), None);
    // A lint.
    let found = linked(&[(FILE, &file)], LoadOptions::default(), &NO_LINTS);
    assert_eq!(said(&found, &TABLE_KINDS), []);
}

/// A table nested in an entry (`pools` of an entry) is walked like the table itself.
#[test]
fn l2_a_reference_in_a_nested_table_is_told_at_its_own_path() {
    let file = table(
        r#"{"type": "empty", "pools": [{"rolls": 1, "entries": [{"type": "loot_table", "name": "loot_tables/none.json"}]}]}"#,
    );
    let found = link(&[(FILE, &file)]);
    assert_eq!(
        said(&found, &TABLE_KINDS),
        [one(
            "UnresolvedReference",
            Severity::Info,
            FILE,
            "pools/0/entries/0/pools/0/entries/0/name",
            "loot table",
            "loot_tables/none.json"
        )]
    );
}

// ---- L1, L5: items -------------------------------------------------------------------------

#[cfg(feature = "item")]
mod item {
    use super::*;

    const KINDS: [&str; 1] = ["item"];

    fn giving(name: &str) -> String {
        table(&format!(r#"{{"type": "item", "name": "{name}"}}"#))
    }

    #[test]
    fn l1_an_item_of_the_stack_of_the_game_or_of_an_older_version_is_no_finding() {
        let gem = ("items/gem.json", &*super::super::item("demo:gem", ""));
        for name in [
            "demo:gem",
            "minecraft:stick",
            "stick",
            "minecraft:wool:14",
            // A name of an older version the game's own tables still use.
            "minecraft:appleEnchanted",
        ] {
            let found = link(&[(FILE, &giving(name)), gem]);
            assert_eq!(said(&found, &KINDS), [], "{name}");
        }
        let host = hosted(&[("item", "host:item")]);
        let found = linked(
            &[(FILE, &giving("host:item"))],
            host,
            &LinkOptions::default(),
        );
        assert_eq!(said(&found, &KINDS), []);
    }

    /// Every block has an item of its name.
    #[cfg(feature = "block")]
    #[test]
    fn l1_a_block_of_the_stack_is_an_item() {
        let lamp = ("blocks/lamp.json", &*super::super::block("demo:lamp", ""));
        let found = link(&[(FILE, &giving("demo:lamp")), lamp]);
        assert_eq!(said(&found, &KINDS), []);
    }

    // Without `block` a name outside `minecraft` may be a pack's block: not in view.
    #[cfg(feature = "block")]
    /// The game logs the line at level 3
    /// and falls through to the entry's `functions`: the entry is
    /// kept, a kept error.
    #[test]
    fn l1_an_item_nothing_defines_is_vanillas_kept_error_and_the_entry_is_kept() {
        let set = stacks(
            &[(FILE, &giving("demo:no_item"))],
            &[],
            LoadOptions::default(),
        );
        let entries = set
            .behavior
            .loot_tables()
            .iter()
            .next()
            .map(|(_, table)| table.pools.as_deref().unwrap()[0].entries().len());
        assert_eq!(entries, Some(1));
        let found = set.link_with(&NO_LINTS);
        assert_eq!(
            said(&found, &KINDS),
            [one(
                "UnresolvedReference",
                Severity::Error,
                FILE,
                "pools/0/entries/0/name",
                "item",
                "demo:no_item"
            )]
        );
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "item")
            .unwrap();
        assert!(finding.kept, "{finding:?}");
        assert_eq!(
            finding.location.pack.as_ref().map(|pack| pack.name()),
            Some("bp 1")
        );
        assert_eq!(
            text(finding).as_deref(),
            Some("demo:no_item is not a valid item name.")
        );
    }

    /// Without the `block` feature a pack's blocks cannot be known: a name outside the
    /// `minecraft` namespace that is no item may be a block's and is not told; a
    /// `minecraft:` name that is no item is.
    #[cfg(not(feature = "block"))]
    #[test]
    fn l1_without_the_block_feature_only_a_minecraft_name_can_be_missing() {
        let found = link(&[(FILE, &giving("demo:no_item"))]);
        assert_eq!(said(&found, &KINDS), []);
        let found = link(&[(FILE, &giving("minecraft:not_an_item"))]);
        assert_eq!(
            said(&found, &KINDS),
            [one(
                "UnresolvedReference",
                Severity::Error,
                FILE,
                "pools/0/entries/0/name",
                "item",
                "minecraft:not_an_item"
            )]
        );
    }

    /// A name of an older version is compared without case (the game's files write
    /// `appleEnchanted` in a loot table and `appleenchanted` in an entity file, and the game's
    /// registry holds the name in lower case); the fold is those names' alone.
    // K113: one list for every group, compared as the game does.
    #[test]
    fn l1_a_legacy_name_resolves_in_any_case_and_no_other_name_does() {
        for known in [
            "minecraft:wood",
            "minecraft:WOOD",
            "minecraft:planks",
            "minecraft:Planks",
            "minecraft:appleEnchanted",
            "minecraft:appleenchanted",
            // A name the game's entity files write, a recipe's and a trade table's: the
            // one list answers for every family.
            "minecraft:carrotOnAStick",
            "minecraft:boat",
            "minecraft:emptyMap",
        ] {
            let found = link(&[(FILE, &giving(known))]);
            assert_eq!(said(&found, &KINDS), [], "{known}");
        }
        for unknown in [
            // Written by the game's entity files alone and no name of the game: the game
            // logs "is not a valid item name" for it in a loot table.
            "minecraft:nautilusarmor",
            "minecraft:Stick",
            "minecraft:APPLE",
            "minecraft:appleenchanted_",
            "demo:wood",
        ] {
            let found = link(&[(FILE, &giving(unknown))]);
            assert_eq!(said(&found, &KINDS).len(), 1, "{unknown}");
        }
    }

    /// `minecraft:iron_pick` is no name of the game; the game's own
    /// `loot_tables/entities/raider_drops.json` writes it as an entry's `name`, and only
    /// there is it let through. As the tool of a `match_tool` condition it is told.
    #[test]
    fn l1_a_name_only_the_games_loot_tables_write_is_let_through_in_an_entry_alone() {
        let found = link(&[(FILE, &giving("minecraft:iron_pick"))]);
        assert_eq!(said(&found, &KINDS), [], "{found:#?}");
        let found = link(&[(FILE, &matching("minecraft:iron_pick"))]);
        let named: Vec<_> = said(&found, &KINDS)
            .into_iter()
            .map(|said| (said.1, said.5))
            .collect();
        assert_eq!(
            named,
            vec![(Severity::Info, "minecraft:iron_pick".to_owned()); 3]
        );
    }

    fn matching(item: &str) -> String {
        format!(
            r#"{{"pools": [{{"rolls": 1, "conditions": [{{"condition": "match_tool", "item": "{item}"}}], "entries": [{{"type": "empty", "conditions": [{{"condition": "match_tool", "item": "{item}"}}]}}, {{"type": "item", "name": "minecraft:stick", "functions": [{{"function": "set_count", "count": 1, "conditions": [{{"condition": "match_tool", "item": "{item}"}}]}}]}}]}}]}}"#
        )
    }

    #[test]
    fn l5_a_tool_of_the_stack_or_of_the_game_is_no_finding() {
        let gem = ("items/gem.json", &*super::super::item("demo:gem", ""));
        for name in ["demo:gem", "minecraft:shears", "shears"] {
            let found = link(&[(FILE, &matching(name)), gem]);
            assert_eq!(said(&found, &KINDS), [], "{name}");
        }
    }

    /// The conditions of a pool, of an entry and of a function are all read.
    // Without `block` a name outside `minecraft` may be a pack's block: not in view.
    #[cfg(feature = "block")]
    #[test]
    fn l5_a_tool_nothing_defines_is_an_information_wherever_the_condition_is() {
        let found = link(&[(FILE, &matching("demo:no_tool"))]);
        let expected: Vec<_> = [
            "pools/0/conditions/0/item",
            "pools/0/entries/0/conditions/0/item",
            "pools/0/entries/1/functions/0/conditions/0/item",
        ]
        .into_iter()
        .map(|path| {
            one(
                "UnresolvedReference",
                Severity::Info,
                FILE,
                path,
                "item",
                "demo:no_tool",
            )
        })
        .collect();
        assert_eq!(said(&found, &KINDS), expected, "{found:#?}");
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "item")
            .unwrap();
        assert_eq!(text(finding), None);
        // A lint.
        let found = linked(
            &[(FILE, &matching("demo:no_tool"))],
            LoadOptions::default(),
            &NO_LINTS,
        );
        assert_eq!(said(&found, &KINDS), []);
    }

    // ---- trim: T1 to T3, T7 ----------------------------------------------------------------

    const TRIM_KINDS: [&str; 2] = ["trim pattern", "trim material"];

    fn trimmed(members: &str) -> String {
        table(&format!(
            r#"{{"type": "item", "name": "minecraft:diamond_chestplate", "functions": [{{"function": "set_armor_trim"{members}}}]}}"#
        ))
    }

    const FUNCTION: &str = "pools/0/entries/0/functions/0";

    #[test]
    fn t1_t2_every_pattern_and_material_of_the_game_resolves() {
        for pattern in ["ward", "host", "flow", "bolt"] {
            for material in ["quartz", "copper", "resin"] {
                let file = trimmed(&format!(
                    r#", "pattern": "{pattern}", "material": "{material}""#
                ));
                let found = link(&[(FILE, &file)]);
                assert_eq!(
                    found
                        .iter()
                        .map(said_of)
                        .filter(|said| said.2 == FILE)
                        .collect::<Vec<_>>(),
                    [],
                    "{pattern} {material}"
                );
            }
        }
    }

    /// The game reads both names without a line, and
    /// at apply time there is only an internal assertion: informations of ours.
    #[test]
    fn t1_t2_an_unknown_name_is_an_information_with_the_name_of_another_case_as_its_hint() {
        use bedrock_addon::diagnostic::ReferenceHint;
        let file = trimmed(r#", "pattern": "Ward", "material": "obsidian""#);
        let found = linked(
            &[(FILE, &file)],
            LoadOptions::default(),
            &LinkOptions::default(),
        );
        assert_eq!(
            said(&found, &TRIM_KINDS),
            [
                one(
                    "UnresolvedReference",
                    Severity::Info,
                    FILE,
                    &format!("{FUNCTION}/pattern"),
                    "trim pattern",
                    "Ward"
                ),
                one(
                    "UnresolvedReference",
                    Severity::Info,
                    FILE,
                    &format!("{FUNCTION}/material"),
                    "trim material",
                    "obsidian"
                ),
            ]
        );
        let hints: Vec<_> = found
            .iter()
            .filter(|finding| TRIM_KINDS.contains(&said_of(finding).4))
            .map(super::super::hint)
            .collect();
        assert_eq!(
            hints,
            [Some(ReferenceHint::CaseMismatch("ward".to_owned())), None]
        );
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "trim pattern")
            .unwrap();
        assert_eq!(
            finding.location.pack.as_ref().map(|pack| pack.name()),
            Some("bp 1")
        );
        assert_eq!(text(finding), None);
    }

    /// The template filter of world start (G2): a pattern whose template item a pack
    /// redefines without the tag `minecraft:trim_templates` is no pattern of that world.
    #[test]
    fn t1_a_pattern_whose_template_item_lost_its_tag_is_unknown() {
        let untagged = super::super::item("minecraft:ward_armor_trim_smithing_template", "");
        let tagged = super::super::item(
            "minecraft:ward_armor_trim_smithing_template",
            r#""minecraft:tags": {"tags": ["minecraft:trim_templates"]}"#,
        );
        let file = trimmed(r#", "pattern": "ward", "material": "copper""#);
        for (template, unknown) in [(&untagged, 1), (&tagged, 0)] {
            let set = stacks(
                &[(FILE, &file), ("items/ward.json", template)],
                &[],
                LoadOptions::default(),
            );
            // A `minecraft:` item loads only where the game allows it; the filter is told
            // by what loaded.
            let overridden = set.behavior.items().len();
            let found = set.link();
            let told = said(&found, &["trim pattern"]).len();
            assert_eq!(
                told,
                if overridden == 1 { unknown } else { 0 },
                "{found:#?}"
            );
        }
    }

    #[test]
    fn t3_an_empty_or_absent_name_is_missing_once_per_member() {
        for members in ["", r#", "pattern": "", "material": """#] {
            let found = linked(
                &[(FILE, &trimmed(members))],
                LoadOptions::default(),
                &LinkOptions::default(),
            );
            let told: Vec<_> = found
                .iter()
                .map(said_of)
                .filter(|said| said.2 == FILE)
                .map(|said| (said.0, said.1, said.3))
                .collect();
            assert_eq!(
                told,
                [
                    (
                        "MissingRequiredField",
                        Severity::Info,
                        format!("{FUNCTION}/pattern")
                    ),
                    (
                        "MissingRequiredField",
                        Severity::Info,
                        format!("{FUNCTION}/material")
                    ),
                ],
                "{members}"
            );
        }
        // One member present and known: only the other is missing.
        let found = link(&[(FILE, &trimmed(r#", "pattern": "ward""#))]);
        let told: Vec<_> = found
            .iter()
            .map(said_of)
            .filter(|said| said.2 == FILE)
            .map(|said| (said.0, said.3))
            .collect();
        assert_eq!(
            told,
            [("MissingRequiredField", format!("{FUNCTION}/material"))]
        );
    }

    fn world(major: u16, minor: u16, patch: u16) -> LoadOptions {
        use bedrock_addon::version::{BaseGameVersion, SemVersion};
        LoadOptions {
            base_game_version: BaseGameVersion::new(SemVersion::new(major, minor, patch)),
            ..LoadOptions::default()
        }
    }

    /// Both sides of the world gate (G1): trims exist from base-game version 1.20.0.
    #[test]
    fn t7_below_1_20_a_trim_function_has_no_effect_and_its_names_are_not_looked_up() {
        let file = trimmed(r#", "pattern": "spiral""#);
        let found = linked(&[(FILE, &file)], world(1, 19, 80), &LinkOptions::default());
        let told: Vec<_> = found
            .iter()
            .map(said_of)
            .filter(|said| said.2 == FILE)
            .map(|said| (said.0, said.1, said.3))
            .collect();
        assert_eq!(told, [("NoEffect", Severity::Info, FUNCTION.to_owned())]);
        // A lint.
        let found = linked(&[(FILE, &file)], world(1, 19, 80), &NO_LINTS);
        assert_eq!(found.iter().filter(|f| said_of(f).2 == FILE).count(), 0);
        // From 1.20.0 the names are looked up.
        let found = linked(&[(FILE, &file)], world(1, 20, 0), &LinkOptions::default());
        let told: Vec<_> = found
            .iter()
            .map(said_of)
            .filter(|said| said.2 == FILE)
            .map(|said| said.0)
            .collect();
        assert_eq!(told, ["UnresolvedReference", "MissingRequiredField"]);
    }
}

#[cfg(not(feature = "item"))]
#[test]
fn l1_l5_and_the_trim_rules_without_the_item_feature_tell_nothing() {
    let file = r#"{"pools": [{"rolls": 1, "conditions": [{"condition": "match_tool", "item": "demo:none"}], "entries": [{"type": "item", "name": "demo:none", "functions": [{"function": "set_armor_trim", "pattern": "spiral"}]}]}]}"#;
    let found = link(&[(FILE, file)]);
    assert_eq!(
        found
            .iter()
            .map(said_of)
            .filter(|said| said.2 == FILE)
            .collect::<Vec<_>>(),
        []
    );
}

// ---- L5: entities --------------------------------------------------------------------------

#[cfg(feature = "entity")]
mod entity {
    use super::*;

    const KINDS: [&str; 1] = ["entity"];

    fn naming(entity: &str) -> String {
        format!(
            r#"{{"pools": [{{"rolls": 1, "conditions": [
            {{"condition": "killed_by_entity", "entity_type": "{entity}"}},
            {{"condition": "entity_killed", "entity_type": "{entity}"}},
            {{"condition": "damaged_by_entity", "entity_type": "{entity}"}},
            {{"condition": "passenger_of_entity", "entity_type": "{entity}"}}],
            "entries": [{{"type": "item", "name": "minecraft:stick", "functions": [{{"function": "set_actor_id", "id": "{entity}"}}]}}]}}]}}"#
        )
    }

    #[test]
    fn l5_an_entity_of_the_stack_or_of_the_game_is_no_finding() {
        let pet = ("entities/pet.json", &*super::super::entity("demo:pet"));
        for name in ["demo:pet", "minecraft:pig", "pig"] {
            let found = link(&[(FILE, &naming(name)), pet]);
            assert_eq!(said(&found, &KINDS), [], "{name}");
        }
        let host = hosted(&[("entity", "host:mob")]);
        let found = linked(
            &[(FILE, &naming("host:mob"))],
            host,
            &LinkOptions::default(),
        );
        assert_eq!(said(&found, &KINDS), []);
    }

    /// The identifier of the game's `pillager_patrol` spawn rules
    /// and of their delays is no entity: outside spawn rules it is told like any name.
    #[test]
    fn l5_the_identifier_of_the_games_patrol_spawn_rules_is_no_entity() {
        for name in [
            "minecraft:pillager_patrol",
            "minecraft:pillager_patrol_hard",
        ] {
            let found = link(&[(FILE, &naming(name))]);
            assert_eq!(said(&found, &KINDS).len(), 5, "{name}");
        }
    }

    #[test]
    fn l5_an_entity_nothing_defines_is_an_information_at_each_member_that_names_it() {
        let found = link(&[(FILE, &naming("demo:nobody"))]);
        let expected: Vec<_> = [
            "pools/0/conditions/0/entity_type",
            "pools/0/conditions/1/entity_type",
            "pools/0/conditions/2/entity_type",
            "pools/0/conditions/3/entity_type",
            "pools/0/entries/0/functions/0/id",
        ]
        .into_iter()
        .map(|path| {
            one(
                "UnresolvedReference",
                Severity::Info,
                FILE,
                path,
                "entity",
                "demo:nobody",
            )
        })
        .collect();
        assert_eq!(said(&found, &KINDS), expected, "{found:#?}");
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "entity")
            .unwrap();
        assert_eq!(
            finding.location.pack.as_ref().map(|pack| pack.name()),
            Some("bp 1")
        );
        assert_eq!(text(finding), None);
        // A lint.
        let found = linked(
            &[(FILE, &naming("demo:nobody"))],
            LoadOptions::default(),
            &NO_LINTS,
        );
        assert_eq!(said(&found, &KINDS), []);
    }
}

#[cfg(not(feature = "entity"))]
#[test]
fn l5_without_the_entity_feature_no_entity_is_told() {
    let file = table(
        r#"{"type": "item", "name": "minecraft:stick", "functions": [{"function": "set_actor_id", "id": "demo:nobody"}]}"#,
    );
    let found = link(&[(FILE, &file)]);
    assert_eq!(said(&found, &["entity"]), []);
}

/// 20 L5, the members that are not checked: a potion, a map destination, a block state, a
/// biome tag and a property name are accepted as read, by the load and by the pass.
#[test]
fn l5_a_potion_a_destination_a_block_state_a_biome_tag_and_a_property_are_not_checked() {
    let file = r#"{"pools": [{"rolls": 1, "conditions": [{"condition": "biome_has_tag", "tag": "demo_no_tag"}, {"condition": "has_property", "domain": "demo:no_property"}, {"condition": "enum_property", "domain": "demo:no_property", "value": "x"}],
        "entries": [{"type": "item", "name": "minecraft:stick", "functions": [{"function": "set_potion", "id": "demo_no_potion"}, {"function": "exploration_map", "destination": "demo_no_structure"}, {"function": "random_block_state", "block_state": "demo:no_state", "values": {"min": 0, "max": 1}}]}]}]}"#;
    let set = stacks(&[(FILE, file)], &[], LoadOptions::default());
    let tables = set.behavior.loot_tables();
    assert_eq!(tables.iter().count(), 1);
    assert_eq!(
        tables
            .diagnostics()
            .filter(|finding| finding.severity >= Severity::Warning)
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        [] as [String; 0]
    );
    let found = set.link();
    assert_eq!(found.iter().map(said_of).collect::<Vec<_>>(), []);
}

// ---- the index of an element is the file's ---------------------------------------------------

/// The paths of the findings about a reference of one of `kinds`, sorted.
fn paths(found: &bedrock_addon::diagnostic::Diagnostics, kinds: &[&str]) -> Vec<String> {
    let mut paths: Vec<String> = said(found, kinds).into_iter().map(|said| said.3).collect();
    paths.sort();
    paths
}

/// A table with an element the reader leaves out before each element that names something
/// missing, at every level: a pool, a condition and an entry that are no object, an entry
/// of an unknown type, a function and a condition that are no object or of an unknown name,
/// and the same in the table an entry nests.
const LEFT_OUT: &str = r#"{"pools": [
    7,
    {"rolls": 1,
     "conditions": [7, {"condition": "killed_by_entity", "entity_type": "demo:none_p"}],
     "entries": [
        7,
        {"type": "bogus"},
        {"type": "loot_table", "name": "loot_tables/none_a.json",
         "conditions": [{"condition": "no_such"}, {"condition": "entity_killed", "entity_type": "demo:none_e"}]},
        {"type": "item", "name": "minecraft:chest", "functions": [
            7,
            {"function": "no_such"},
            {"function": "fill_container", "loot_table": "loot_tables/none_b.json",
             "conditions": [7, {"condition": "killed_by_entity", "entity_type": "demo:none_f"}]}]},
        {"type": "empty", "pools": [
            7,
            {"rolls": 1, "entries": [7, {"type": "loot_table", "name": "loot_tables/none_c.json"}]}]}
     ]}
]}"#;

/// L2 behind elements the reader leaves out: every index of the path is the file's.
#[test]
fn l2_a_reference_behind_elements_the_reader_leaves_out_is_at_the_indices_of_the_file() {
    let found = link(&[(FILE, LEFT_OUT)]);
    assert_eq!(
        paths(&found, &TABLE_KINDS),
        [
            "pools/1/entries/2/name",
            "pools/1/entries/3/functions/2/loot_table",
            "pools/1/entries/4/pools/1/entries/1/name",
        ],
        "{found:#?}"
    );
}

/// L5 the same: the conditions of a pool, of an entry and of a function.
#[cfg(feature = "entity")]
#[test]
fn l5_a_condition_behind_elements_the_reader_leaves_out_is_at_the_indices_of_the_file() {
    let found = link(&[(FILE, LEFT_OUT)]);
    assert_eq!(
        paths(&found, &["entity"]),
        [
            "pools/1/conditions/1/entity_type",
            "pools/1/entries/2/conditions/1/entity_type",
            "pools/1/entries/3/functions/2/conditions/1/entity_type",
        ],
        "{found:#?}"
    );
}

/// The same in a file with a number no double holds, whose findings are settled after the
/// reading: the indices are still the file's.
#[test]
fn l2_the_indices_are_the_files_in_a_table_with_a_number_no_double_holds() {
    let file = r#"{"pools": [7, {"rolls": 1e999, "entries": [7,
        {"type": "loot_table", "name": "loot_tables/none_a.json"}]}]}"#;
    let found = link(&[(FILE, file)]);
    assert_eq!(paths(&found, &TABLE_KINDS), ["pools/1/entries/1/name"]);
}
