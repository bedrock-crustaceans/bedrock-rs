//! The references of an item (20 I2 to I5, I7) and the item rows that are not the link
//! pass's (I1, I8 to I10).

#![allow(
    unused_imports,
    dead_code,
    reason = "which tests exist depends on the enabled features"
)]

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, stacks, stacks_of, unloaded_pack};
use super::{item, one, said, said_of, text};

const NO_LINTS: LinkOptions = LinkOptions {
    lints: false,
    client: true,
};
const FILE: &str = "items/wand.json";
const COMPONENTS: &str = "minecraft:item/components";

/// The link findings of a world whose behaviour pack must load `items` items.
fn linked(
    behavior: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
    load: LoadOptions,
    options: &LinkOptions,
) -> bedrock_addon::diagnostic::Diagnostics {
    let set = stacks(behavior, resource, load);
    let items = behavior
        .iter()
        .filter(|(path, _)| path.starts_with("items/"))
        .count();
    let loaded = set.behavior.items();
    let beta = loaded.beta().map_or(0, |beta| beta.len());
    assert_eq!(
        loaded.len() + beta,
        items,
        "{:#?}",
        loaded
            .diagnostics()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    set.link_with(options)
}

fn link(
    behavior: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
) -> bedrock_addon::diagnostic::Diagnostics {
    linked(
        behavior,
        resource,
        LoadOptions::default(),
        &LinkOptions::default(),
    )
}

fn wand(components: &str) -> String {
    item("demo:wand", components)
}

// ---- I2: a resource-pack item and its behaviour-pack item ----------------------------------

/// A resource-pack item of the legacy format.
fn client_item(id: &str) -> String {
    format!(
        r#"{{"format_version": "1.10", "minecraft:item": {{"description": {{"identifier": "{id}"}}, "components": {{"minecraft:icon": "stick"}}}}}}"#
    )
}

const ITEM_KINDS: [&str; 1] = ["item"];

#[test]
fn i2_a_resource_pack_item_a_behaviour_pack_or_the_game_defines_is_no_finding() {
    let found = link(
        &[("items/twin.json", &item("demo:twin", ""))],
        &[
            ("items/twin.json", &client_item("demo:twin")),
            ("items/apple.json", &client_item("minecraft:apple")),
        ],
    );
    assert_eq!(said(&found, &ITEM_KINDS), [], "{found:#?}");
    let host = hosted(&[("item", "host:item")]);
    let found = linked(
        &[],
        &[("items/host.json", &client_item("host:item"))],
        host,
        &LinkOptions::default(),
    );
    assert_eq!(said(&found, &ITEM_KINDS), []);
}

/// The game logs the line at level 3 and skips the item: the resource pack's definition
/// is dropped and the load goes on, a kept error.
#[test]
fn i2_a_resource_pack_item_no_behaviour_pack_defines_is_vanillas_kept_error() {
    let found = linked(
        &[("items/twin.json", &item("demo:twin", ""))],
        &[
            ("items/lone.json", &client_item("demo:lone")),
            ("items/twin.json", &client_item("demo:twin")),
        ],
        LoadOptions::default(),
        &NO_LINTS,
    );
    assert_eq!(
        said(&found, &ITEM_KINDS),
        [one(
            "UnresolvedReference",
            Severity::Error,
            "items/lone.json",
            "minecraft:item/description/identifier",
            "item",
            "demo:lone"
        )]
    );
    let finding = found
        .iter()
        .find(|finding| said_of(finding).4 == "item")
        .unwrap();
    assert!(finding.kept, "{finding:?}");
    assert_eq!(
        finding.location.pack.as_ref().map(|pack| pack.name()),
        Some("rp 2")
    );
    assert_eq!(
        text(finding).as_deref(),
        Some(
            "Resource pack has item definitions not found in the behavior pack. Item 'demo:lone' is being ignored."
        )
    );
}

/// A block of the stack is no behaviour-pack item definition: the rule asks for the item
/// definition itself.
#[cfg(feature = "block")]
#[test]
fn i2_a_block_of_the_same_name_is_no_item_definition() {
    let found = link(
        &[("blocks/lamp.json", &super::block("demo:lamp", ""))],
        &[("items/lamp.json", &client_item("demo:lamp"))],
    );
    assert_eq!(said(&found, &ITEM_KINDS).len(), 1, "{found:#?}");
}

/// Vanilla refuses a resource-pack item of 1.16.100 or later when it reads the file; such a
/// file is not an ignored definition.
#[test]
fn i2_a_resource_pack_item_of_the_component_format_is_not_this_rules() {
    let found = link(&[], &[("items/lone.json", &item("demo:lone", ""))]);
    assert_eq!(said(&found, &ITEM_KINDS), [], "{found:#?}");
}

/// A file that is not resident cannot be read by a pure pass: it is told once, as an
/// information, and not judged.
#[test]
fn i2_a_resource_pack_item_that_is_not_loaded_is_not_judged() {
    let lone = client_item("demo:lone");
    let resources = unloaded_pack(2, "resources", &[("items/lone.json", lone.as_bytes())]);
    let set = stacks_of(Vec::new(), vec![resources], LoadOptions::default());
    let found = set.link();
    assert_eq!(said(&found, &ITEM_KINDS), []);
    let not_loaded: Vec<_> = found
        .iter()
        .filter(|finding| finding.code.name() == "FileNotLoaded")
        .map(|finding| said_of(finding).2)
        .collect();
    assert_eq!(not_loaded, ["items/lone.json"]);
    assert_eq!(
        set.link_with(&NO_LINTS).len(),
        0,
        "without the lints the file is skipped in silence"
    );
}

/// Of two resource packs with the same file, the upper one's is the one the game reads.
#[test]
fn i2_of_two_resource_packs_with_the_file_the_upper_one_is_read() {
    use super::super::common::resource_pack;
    let upper = resource_pack(2, &[("items/a.json", &client_item("demo:upper"))]);
    let lower = resource_pack(3, &[("items/a.json", &client_item("demo:lower"))]);
    let found = stacks_of(Vec::new(), vec![upper, lower], LoadOptions::default()).link();
    let named: Vec<_> = said(&found, &ITEM_KINDS)
        .into_iter()
        .map(|said| said.5)
        .collect();
    assert_eq!(named, ["demo:upper"]);
}

// ---- I3: entities --------------------------------------------------------------------------

#[cfg(feature = "entity")]
mod entity {
    use super::*;

    const KINDS: [&str; 1] = ["entity"];

    fn placing(entity: &str, projectile: &str) -> String {
        wand(&format!(
            r#""minecraft:entity_placer": {{"entity": "{entity}"}}, "minecraft:projectile": {{"projectile_entity": "{projectile}"}}"#
        ))
    }

    #[test]
    fn i3_an_entity_of_the_stack_or_of_the_game_is_no_finding_with_or_without_an_event() {
        let pet = ("entities/pet.json", &*super::super::entity("demo:pet"));
        for (placed, shot) in [
            ("demo:pet", "minecraft:arrow"),
            ("demo:pet<demo:born>", "minecraft:arrow<minecraft:any>"),
            ("minecraft:pig", "arrow"),
        ] {
            let found = link(&[(FILE, &placing(placed, shot)), pet], &[]);
            assert_eq!(said(&found, &KINDS), [], "{placed} {shot}");
        }
        let host = hosted(&[("entity", "host:mob")]);
        let found = linked(
            &[(FILE, &placing("host:mob", "host:mob<host:event>"))],
            &[],
            host,
            &LinkOptions::default(),
        );
        assert_eq!(said(&found, &KINDS), []);
    }

    /// The identifier of the game's `pillager_patrol` spawn rules is
    /// no entity an item can place.
    #[test]
    fn i3_the_identifier_of_the_games_patrol_spawn_rules_is_no_entity() {
        let found = linked(
            &[(
                FILE,
                &placing(
                    "minecraft:pillager_patrol",
                    "minecraft:pillager_patrol_easy",
                ),
            )],
            &[],
            LoadOptions::default(),
            &NO_LINTS,
        );
        let named: Vec<_> = said(&found, &KINDS)
            .into_iter()
            .map(|said| (said.1, said.5))
            .collect();
        assert_eq!(
            named,
            [
                (Severity::Error, "minecraft:pillager_patrol".to_owned()),
                (Severity::Error, "minecraft:pillager_patrol_easy".to_owned()),
            ]
        );
    }

    #[test]
    fn i3_an_entity_nothing_defines_is_an_error_at_the_member_that_names_it() {
        let found = linked(
            &[(FILE, &placing("demo:nobody<demo:born>", "demo:no_arrow"))],
            &[],
            LoadOptions::default(),
            &NO_LINTS,
        );
        assert_eq!(
            said(&found, &KINDS),
            [
                one(
                    "UnresolvedReference",
                    Severity::Error,
                    FILE,
                    &format!("{COMPONENTS}/minecraft:entity_placer/entity"),
                    "entity",
                    "demo:nobody<demo:born>"
                ),
                one(
                    "UnresolvedReference",
                    Severity::Error,
                    FILE,
                    &format!("{COMPONENTS}/minecraft:projectile/projectile_entity"),
                    "entity",
                    "demo:no_arrow"
                ),
            ]
        );
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "entity")
            .unwrap();
        assert_eq!(
            finding.location.pack.as_ref().map(|pack| pack.name()),
            Some("bp 1")
        );
        // without its event.
        assert_eq!(
            text(finding).as_deref(),
            Some(
                "-> components -> minecraft:entity_placer -> entity:  Missing referenced asset demo:nobody"
            )
        );
    }
}

#[cfg(not(feature = "entity"))]
#[test]
fn i3_without_the_entity_feature_nothing_is_told() {
    let file = wand(r#""minecraft:entity_placer": {"entity": "demo:nobody"}"#);
    let found = link(&[(FILE, &file)], &[]);
    assert_eq!(said(&found, &["entity"]), []);
}

// ---- I4, I5: blocks ------------------------------------------------------------------------

#[cfg(feature = "block")]
mod block {
    use super::*;

    const KINDS: [&str; 2] = ["block", "block state"];
    const BASE: (&str, &str) = (
        "blocks/base.json",
        r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:base", "states": {"demo:lit": [false, true]}}, "components": {}}}"#,
    );

    fn placing(block: &str) -> String {
        wand(&format!(
            r#""minecraft:block_placer": {{"block": {block}}}"#
        ))
    }

    #[test]
    fn i4_a_block_of_the_stack_or_of_the_game_is_no_finding() {
        for written in [
            r#""demo:base""#,
            r#"{"name": "demo:base", "states": {"demo:lit": true}}"#,
            r#""minecraft:stone""#,
        ] {
            let found = link(&[(FILE, &placing(written)), BASE], &[]);
            assert_eq!(said(&found, &KINDS), [], "{written}");
        }
        let host = hosted(&[("block", "host:block")]);
        let found = linked(
            &[(FILE, &placing(r#""host:block""#))],
            &[],
            host,
            &LinkOptions::default(),
        );
        assert_eq!(said(&found, &KINDS), []);
    }

    #[test]
    fn i4_a_placed_block_nothing_defines_is_an_error_at_the_member_that_names_it() {
        let found = linked(
            &[(FILE, &placing(r#""demo:none""#))],
            &[],
            LoadOptions::default(),
            &NO_LINTS,
        );
        assert_eq!(
            said(&found, &KINDS),
            [one(
                "UnresolvedReference",
                Severity::Error,
                FILE,
                &format!("{COMPONENTS}/minecraft:block_placer/block"),
                "block",
                "demo:none"
            )]
        );
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "block")
            .unwrap();
        assert_eq!(
            finding.location.pack.as_ref().map(|pack| pack.name()),
            Some("bp 1")
        );
        assert_eq!(
            text(finding).as_deref(),
            Some(
                "-> components -> minecraft:block_placer -> block:  Missing referenced asset demo:none"
            )
        );
        // The object form  names the block in `name`, and
        // its states are held to the block's.
        let object = placing(r#"{"name": "demo:base", "states": {"demo:dim": true}}"#);
        let found = link(&[(FILE, &object), BASE], &[]);
        // A state is the crate's own check of a block of the stack: no validator line, no
        // line of the game at all: an information.
        assert_eq!(
            said(&found, &KINDS),
            [one(
                "UnresolvedReference",
                Severity::Info,
                FILE,
                &format!("{COMPONENTS}/minecraft:block_placer/block/states/demo:dim"),
                "block state",
                "demo:dim"
            )]
        );
    }

    /// The block a legacy item stands for (`minecraft:block`). A miss reaches only the
    /// developer log (the block registry's lookup): an
    /// information.
    #[test]
    fn i4_the_block_of_a_legacy_item_is_resolved() {
        let legacy = |block: &str| {
            format!(
                r#"{{"format_version": "1.10", "minecraft:item": {{"description": {{"identifier": "demo:wand"}}, "components": {{"minecraft:block": "{block}"}}}}}}"#
            )
        };
        for known in ["demo:base", "minecraft:stone", "stone"] {
            let found = link(&[(FILE, &legacy(known)), BASE], &[]);
            assert_eq!(said(&found, &KINDS), [], "{known}");
        }
        let found = link(&[(FILE, &legacy("demo:none"))], &[]);
        assert_eq!(
            said(&found, &KINDS),
            [one(
                "UnresolvedReference",
                Severity::Info,
                FILE,
                &format!("{COMPONENTS}/minecraft:block"),
                "block",
                "demo:none"
            )]
        );
    }

    #[test]
    fn i5_the_block_descriptors_of_an_item_are_resolved() {
        let file = wand(
            r#""minecraft:digger": {"destroy_speeds": [{"block": "minecraft:stone", "speed": 2}, {"block": "demo:none_a", "speed": 2}, {"block": {"tags": "q.any_tag('demo:no_tag')"}, "speed": 2}]},
               "minecraft:block_placer": {"block": "minecraft:stone", "use_on": ["minecraft:dirt", {"name": "demo:none_b"}]},
               "minecraft:entity_placer": {"entity": "minecraft:pig", "use_on": ["demo:none_c"], "dispense_on": ["minecraft:dirt", "demo:none_d"]}"#,
        );
        let found = linked(&[(FILE, &file)], &[], LoadOptions::default(), &NO_LINTS);
        assert_eq!(
            said(&found, &KINDS),
            [
                one(
                    "UnresolvedReference",
                    Severity::Error,
                    FILE,
                    &format!("{COMPONENTS}/minecraft:block_placer/use_on/1/name"),
                    "block",
                    "demo:none_b"
                ),
                one(
                    "UnresolvedReference",
                    Severity::Error,
                    FILE,
                    &format!("{COMPONENTS}/minecraft:digger/destroy_speeds/1/block"),
                    "block",
                    "demo:none_a"
                ),
                one(
                    "UnresolvedReference",
                    Severity::Error,
                    FILE,
                    &format!("{COMPONENTS}/minecraft:entity_placer/use_on/0"),
                    "block",
                    "demo:none_c"
                ),
                one(
                    "UnresolvedReference",
                    Severity::Error,
                    FILE,
                    &format!("{COMPONENTS}/minecraft:entity_placer/dispense_on/1"),
                    "block",
                    "demo:none_d"
                ),
            ],
            "{found:#?}"
        );
    }
}

#[cfg(not(feature = "block"))]
#[test]
fn i4_i5_without_the_block_feature_no_block_is_told() {
    let file = wand(
        r#""minecraft:block_placer": {"block": "demo:none", "use_on": ["demo:none"]}, "minecraft:digger": {"destroy_speeds": [{"block": "demo:none", "speed": 2}]}"#,
    );
    let found = link(&[(FILE, &file)], &[]);
    assert_eq!(said(&found, &["block", "block state"]), []);
}

// ---- I5: items -----------------------------------------------------------------------------

const DESCRIBED: &str = r#""minecraft:durability": {"max_durability": 10}, "minecraft:max_stack_size": 1,
    "minecraft:use_modifiers": {"use_duration": 1.0},
    "minecraft:repairable": {"repair_items": [{"items": ["minecraft:stick", "demo:none_a", {"tags": "q.any_tag('demo:no_tag')"}], "repair_amount": 1}]},
    "minecraft:shooter": {"ammunition": [{"item": "minecraft:arrow"}, {"item": {"name": "demo:none_b"}}]},
    "minecraft:food": {"using_converts_to": "demo:none_c"},
    "minecraft:storage_item": {"max_slots": 4, "allowed_items": ["minecraft:stick", "demo:none_d"], "banned_items": ["demo:none_e", "demo:gem"]}"#;

// Without `block` a name outside `minecraft` may be a pack's block: not in view.
#[cfg(feature = "block")]
#[test]
fn i5_the_item_descriptors_of_an_item_are_resolved() {
    let gem = ("items/gem.json", &*item("demo:gem", ""));
    let found = linked(
        &[(FILE, &wand(DESCRIBED)), gem],
        &[],
        LoadOptions::default(),
        &NO_LINTS,
    );
    let expected: Vec<_> = [
        ("minecraft:repairable/repair_items/0/items/1", "demo:none_a"),
        ("minecraft:shooter/ammunition/1/item/name", "demo:none_b"),
        ("minecraft:food/using_converts_to", "demo:none_c"),
        ("minecraft:storage_item/banned_items/0", "demo:none_e"),
        ("minecraft:storage_item/allowed_items/1", "demo:none_d"),
    ]
    .into_iter()
    .map(|(path, id)| {
        one(
            "UnresolvedReference",
            Severity::Error,
            FILE,
            &format!("{COMPONENTS}/{path}"),
            "item",
            id,
        )
    })
    .collect();
    assert_eq!(said(&found, &ITEM_KINDS), expected, "{found:#?}");
    let finding = found
        .iter()
        .find(|finding| said_of(finding).4 == "item")
        .unwrap();
    assert_eq!(
        finding.location.pack.as_ref().map(|pack| pack.name()),
        Some("bp 1")
    );
    assert_eq!(
        text(finding).as_deref(),
        Some(
            "-> components -> minecraft:repairable -> repair_items[0] -> items[1]:  Missing referenced asset demo:none_a"
        )
    );
}

#[test]
fn i5_an_item_name_of_an_older_version_is_no_finding() {
    // The game's own bundles ban `minecraft:shulker_box`.
    let file = wand(
        r#""minecraft:max_stack_size": 1, "minecraft:storage_item": {"banned_items": ["minecraft:shulker_box"]}"#,
    );
    let found = link(&[(FILE, &file)], &[]);
    assert_eq!(said(&found, &ITEM_KINDS), [], "{found:#?}");
}

#[test]
fn i5_an_item_the_host_says_the_game_has_is_no_finding() {
    let file = wand(r#""minecraft:food": {"using_converts_to": "host:bowl"}"#);
    let host = hosted(&[("item", "host:bowl")]);
    let found = linked(&[(FILE, &file)], &[], host, &LinkOptions::default());
    assert_eq!(said(&found, &ITEM_KINDS), []);
}

// ---- I7: sounds ----------------------------------------------------------------------------

#[cfg(all(feature = "sound_definitions", feature = "sounds_json"))]
mod sound {
    use super::*;

    const KINDS: [&str; 1] = ["sound event"];
    const DEFINITIONS: (&str, &str) = (
        "sounds/sound_definitions.json",
        r#"{"format_version": "1.20.20", "sound_definitions": {"demo.ping": {"sounds": ["sounds/demo/ping"]}}}"#,
    );

    /// A document of `version` whose sound members name `sound`: the six of the beta struct,
    /// or at a stable version the five that take a sound name from 1.26.50 (the stable
    /// `record.sound_event` is a level sound event, refused at the parse otherwise).
    fn sounding(version: &str, sound: &str) -> String {
        let record = if version == "beta" {
            format!(r#""minecraft:record": {{"sound_event": "{sound}", "duration": 5.0}},"#)
        } else {
            String::new()
        };
        format!(
            r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:wand"}}, "components": {{
            "minecraft:use_modifiers": {{"use_duration": 1.0, "start_sound": "{sound}"}},
            "minecraft:swing_sounds": {{"attack_miss": "{sound}", "attack_hit": "{sound}", "attack_critical_hit": "{sound}"}},
            {record}
            "minecraft:durability": {{"max_durability": 10}},
            "minecraft:durability_sensor": {{"durability_thresholds": [{{"durability": 5, "sound_event": "{sound}"}}]}},
            "minecraft:max_stack_size": 1
        }}}}}}"#
        )
    }

    fn beta() -> LoadOptions {
        LoadOptions {
            beta_apis: true,
            ..LoadOptions::default()
        }
    }

    #[test]
    fn i7_a_level_sound_event_and_a_sound_of_the_definitions_are_no_finding() {
        for version in ["1.26.50", "beta"] {
            // An event of the game, in any case; a name of `sound_definitions.json`.
            for sound in ["record.cat", "Record.Cat", "demo.ping"] {
                let found = linked(
                    &[(FILE, &sounding(version, sound))],
                    &[DEFINITIONS],
                    beta(),
                    &LinkOptions::default(),
                );
                assert_eq!(said(&found, &KINDS), [], "{version} {sound}: {found:#?}");
            }
        }
    }

    #[test]
    fn i7_a_sound_the_host_says_the_game_has_is_no_finding() {
        let host = LoadOptions {
            beta_apis: true,
            ..hosted(&[("sound event", "host.sound")])
        };
        let found = linked(
            &[(FILE, &sounding("beta", "host.sound"))],
            &[],
            host,
            &LinkOptions::default(),
        );
        assert_eq!(said(&found, &KINDS), [], "{found:#?}");
    }

    /// The game answers a name it lacks without a line, and no
    /// loader of these members logs one: an information of ours.
    #[test]
    fn i7_a_sound_nothing_provides_is_an_information_at_each_member_that_names_it() {
        for version in ["1.26.50", "beta"] {
            let found = linked(
                &[(FILE, &sounding(version, "demo.pong"))],
                &[DEFINITIONS],
                beta(),
                &LinkOptions::default(),
            );
            let expected: Vec<_> = [
                "minecraft:record/sound_event",
                "minecraft:durability_sensor/durability_thresholds/0/sound_event",
                "minecraft:swing_sounds/attack_miss",
                "minecraft:swing_sounds/attack_hit",
                "minecraft:swing_sounds/attack_critical_hit",
                "minecraft:use_modifiers/start_sound",
            ]
            .into_iter()
            .filter(|path| version == "beta" || !path.starts_with("minecraft:record"))
            .map(|path| {
                one(
                    "UnknownSound",
                    Severity::Info,
                    FILE,
                    &format!("{COMPONENTS}/{path}"),
                    "sound event",
                    "demo.pong",
                )
            })
            .collect();
            assert_eq!(said(&found, &KINDS), expected, "{version}: {found:#?}");
            let finding = found
                .iter()
                .find(|finding| said_of(finding).4 == "sound event")
                .unwrap();
            assert_eq!(
                finding.location.pack.as_ref().map(|pack| pack.name()),
                Some("bp 1")
            );
            assert_eq!(text(finding), None);
        }
    }

    /// A sound of the definitions is compared as written: another case is another name.
    #[test]
    fn i7_a_sound_of_the_definitions_in_another_case_is_unknown() {
        let found = linked(
            &[(FILE, &sounding("1.26.50", "Demo.Ping"))],
            &[DEFINITIONS],
            LoadOptions::default(),
            &LinkOptions::default(),
        );
        assert_eq!(said(&found, &KINDS).len(), 5);
    }
}

#[cfg(not(all(feature = "sound_definitions", feature = "sounds_json")))]
#[test]
fn i7_without_the_sound_feature_the_load_refuses_the_record() {
    let file = wand(r#""minecraft:record": {"sound_event": "demo.none", "duration": 5.0}"#);
    let set = stacks(&[(FILE, &file)], &[], LoadOptions::default());
    let items = set.behavior.items();
    // The sound-event enum is empty without `sound`, so the load refuses the record and the
    // item is not in the stack the link pass reads: the load's finding is what is pinned.
    assert_eq!((items.len(), items.rejected().len()), (0, 1));
    let told: Vec<_> = items
        .diagnostics()
        .map(|finding| {
            (
                finding.code.name(),
                finding.severity,
                finding.location.json_path.to_string(),
            )
        })
        .collect();
    assert_eq!(
        told,
        [(
            "InvalidEnumValue",
            Severity::Error,
            format!("{COMPONENTS}/minecraft:record/sound_event")
        )]
    );
}

// ---- trim T4: a pack's own template item ---------------------------------------------------

/// the pattern list is the game's; an item a pack tags
/// `minecraft:trim_templates` is never a template.
#[test]
fn t4_a_pack_item_tagged_as_a_trim_template_has_no_effect() {
    let tagged = item(
        "demo:spiral_template",
        r#""minecraft:tags": {"tags": ["demo:shiny", "minecraft:trim_templates"]}"#,
    );
    // The game's pattern list is compiled in; its lookup has only an internal assertion: no line, an information of ours.
    let found = linked(
        &[("items/spiral.json", &tagged)],
        &[],
        LoadOptions::default(),
        &LinkOptions::default(),
    );
    let told: Vec<_> = found
        .iter()
        .map(said_of)
        .map(|said| (said.0, said.1, said.2, said.3))
        .collect();
    assert_eq!(
        told,
        [(
            "NoEffect",
            Severity::Info,
            "items/spiral.json".to_owned(),
            format!("{COMPONENTS}/minecraft:tags")
        )]
    );
    let finding = found.iter().next().unwrap();
    assert_eq!(
        finding.location.pack.as_ref().map(|pack| pack.name()),
        Some("bp 1")
    );
    assert_eq!(text(finding), None);
    // Another tag is no trim content.
    let other = item(
        "demo:spiral_template",
        r#""minecraft:tags": {"tags": ["demo:shiny", "minecraft:trim_template"]}"#,
    );
    let found = link(&[("items/spiral.json", &other)], &[]);
    assert_eq!(
        found
            .iter()
            .filter(|finding| finding.code.name() == "NoEffect")
            .count(),
        0
    );
}

// ---- the rows the load checks or nothing checks: I9, I10 ------------------------------------

/// The `UnresolvedReference` findings of the load of the one item `file`.
fn load_references(file: &str) -> Vec<(Severity, String, String)> {
    let set = stacks(&[(FILE, file)], &[], LoadOptions::default());
    let items = set.behavior.items();
    assert_eq!(
        items.len(),
        1,
        "{:#?}",
        items
            .diagnostics()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    items
        .diagnostics()
        .filter(|finding| finding.code.name() == "UnresolvedReference")
        .map(|finding| {
            let id = match finding.detail.as_deref() {
                Some(bedrock_addon::diagnostic::DiagnosticDetail::Reference { id, .. }) => {
                    id.clone()
                }
                _ => String::new(),
            };
            (finding.severity, finding.location.json_path.to_string(), id)
        })
        .collect()
}

/// 20 I9 (a load row): a trigger of an item names an event of the item's own `events`.
#[test]
fn i9_a_trigger_that_names_no_event_of_the_item_is_a_load_information() {
    let file = r#"{"format_version": "1.20.40", "minecraft:item": {"description": {"identifier": "demo:wand"},
        "components": {"minecraft:digger": {"destroy_speeds": [{"block": "minecraft:stone", "speed": 2, "on_dig": {"event": "demo:missing"}}], "on_dig": {"event": "demo:there"}}},
        "events": {"demo:there": {}}}}"#;
    let told = load_references(file);
    assert_eq!(told.len(), 1, "{told:#?}");
    assert_eq!(told[0].0, Severity::Info);
    assert_eq!(told[0].2, "demo:missing");
    assert!(
        told[0]
            .1
            .ends_with("destroy_speeds/0/__deprecated_on_dig/event"),
        "{}",
        told[0].1
    );
    let found = link(&[(FILE, file)], &[]);
    assert_eq!(said(&found, &["event"]), []);
}

/// 20 I10: a custom component is the scripts' to register; a cooldown category, a display
/// name and a tag name nothing a pack defines. Nothing is told of them.
#[test]
fn i10_a_custom_component_a_cooldown_category_a_name_and_a_tag_are_not_checked() {
    let file = wand(
        r#""demo:no_such_component": {"item": "demo:none"}, "minecraft:cooldown": {"category": "demo_no_category", "duration": 1.0}, "minecraft:display_name": {"value": "item.demo.no_key"}, "minecraft:tags": {"tags": ["demo:no_tag"]}"#,
    );
    assert_eq!(load_references(&file), []);
    let found = link(&[(FILE, &file)], &[]);
    assert_eq!(found.iter().map(said_of).collect::<Vec<_>>(), []);
}

// ---- the index of a repair entry is the file's -----------------------------------------------

/// I5 behind entries of a legacy `repair_items` list that the upgrade cannot read: the game
/// refuses the item for each (its " Error Parsing Item"), so the entries are not left out of a loaded item, and the
/// index each refusal is told at is still the file's.
#[cfg(feature = "block")]
#[test]
fn i5_a_repair_item_the_upgrade_cannot_read_refuses_the_item_at_the_index_of_the_file() {
    let file = r#"{"format_version": "1.16.100", "minecraft:item": {"description": {"identifier": "demo:wand"},
        "components": {"minecraft:durability": {"max_durability": 10}, "minecraft:max_stack_size": 1,
        "minecraft:repairable": {"repair_items": [
            7,
            {"items": ["minecraft:stick", "demo:none_a"], "repair_amount": 1},
            {"repair_amount": 1},
            {"items": ["demo:none_b"], "repair_amount": 1}]}}}}"#;
    // A pack the game reads an item below 1.20.0 of.
    const ALLOW_LISTED: u128 = 0xda09_e399_6f5e_4287_a013_4cf1_3de3_2166;
    const REPAIR: &str = "minecraft:item/components/minecraft:repairable/repair_items";
    let pack = super::super::common::behavior_pack(ALLOW_LISTED, &[(FILE, file)]);
    let set = stacks_of(vec![pack], Vec::new(), LoadOptions::default());
    let items = set.behavior.items();
    assert_eq!(items.iter().count(), 0, "the item is refused");
    let rejected: Vec<String> = items.diagnostics().map(ToString::to_string).collect();
    for index in [0, 2] {
        let at = format!("{REPAIR}/{index}");
        assert!(
            rejected
                .iter()
                .any(|line| line.starts_with("[error]") && line.contains(&at)),
            "no refusal at {at}: {rejected:#?}"
        );
    }
    assert_eq!(rejected.len(), 2, "{rejected:#?}");
}

/// I5 of a legacy `repair_items` list whose entries all read: the link pass tells the
/// names no item has at the file's own index of the entry and of the name.
#[cfg(feature = "block")]
#[test]
fn i5_a_repair_item_of_a_legacy_list_is_at_the_index_of_the_file() {
    let file = r#"{"format_version": "1.16.100", "minecraft:item": {"description": {"identifier": "demo:wand"},
        "components": {"minecraft:durability": {"max_durability": 10}, "minecraft:max_stack_size": 1,
        "minecraft:repairable": {"repair_items": [
            {"items": ["minecraft:stick"], "repair_amount": 1},
            {"items": ["minecraft:stick", "demo:none_a"], "repair_amount": 1},
            {"items": ["demo:none_b"], "repair_amount": 1}]}}}}"#;
    const ALLOW_LISTED: u128 = 0xda09_e399_6f5e_4287_a013_4cf1_3de3_2166;
    let pack = super::super::common::behavior_pack(ALLOW_LISTED, &[(FILE, file)]);
    let set = stacks_of(vec![pack], Vec::new(), LoadOptions::default());
    let items = set.behavior.items();
    let rejected: Vec<String> = items.diagnostics().map(ToString::to_string).collect();
    let item = items
        .iter()
        .next()
        .unwrap_or_else(|| panic!("{rejected:#?}"))
        .1;
    let held = item
        .components
        .get::<bedrock_addon::families::item::component::Repairable>()
        .map(|repairable| repairable.repair_items.len());
    assert_eq!(held, Some(3), "{:#?}", item.diagnostics());
    let found = set.link_with(&NO_LINTS);
    let told: Vec<(String, String)> = said(&found, &ITEM_KINDS)
        .into_iter()
        .map(|said| (said.3, said.5))
        .collect();
    const REPAIR: &str = "minecraft:item/components/minecraft:repairable/repair_items";
    assert_eq!(
        told,
        [
            (format!("{REPAIR}/1/items/1"), "demo:none_a".to_owned()),
            (format!("{REPAIR}/2/items/0"), "demo:none_b".to_owned()),
        ],
        "{found:#?}"
    );
}

// a pack defines (`is_spawnable`), so an item reference to it resolves.
#[cfg(all(feature = "entity", feature = "block"))]
#[test]
fn i5_the_spawn_egg_of_a_spawnable_entity_of_the_stack_is_an_item() {
    let entity = |spawnable: bool| {
        format!(
            r#"{{"format_version": "1.21.0", "minecraft:entity": {{"description": {{"identifier": "demo:wisp", "is_spawnable": {spawnable}}}, "components": {{}}}}}}"#
        )
    };
    let wand = wand(r#""minecraft:food": {"using_converts_to": "demo:wisp_spawn_egg"}"#);
    for (spawnable, told) in [(true, 0), (false, 1)] {
        let found = link(
            &[(FILE, &wand), ("entities/wisp.json", &entity(spawnable))],
            &[],
        );
        let eggs = said(&found, &ITEM_KINDS)
            .into_iter()
            .filter(|said| said.5 == "demo:wisp_spawn_egg")
            .count();
        assert_eq!(eggs, told, "is_spawnable {spawnable}: {found:#?}");
    }
}
