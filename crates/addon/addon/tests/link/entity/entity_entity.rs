//! 20 E14 and E16: the entities and the level sound events an entity's components name.

#![allow(unused_imports, reason = "each feature set compiles part of the tests")]

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link, link_with};
use super::{FILE, entity_with, miss, pet, said_of, unresolved};

/// Every member of E14, each naming an entity nothing defines.
const ENTITIES: &str = r#"
    "minecraft:transformation": {"into": "demo:none1<minecraft:as_adult>"},
    "minecraft:spawn_entity": {"entities": [{"spawn_entity": "demo:none2"}]},
    "minecraft:addrider": {"riders": [{"entity_type": "demo:none3"}]},
    "minecraft:spawn_on_death": {"entity_to_spawn": "demo:none4"},
    "minecraft:shooter": {"def": "demo:none5", "projectiles": [{"def": "demo:none6"}]},
    "minecraft:breedable": {"breeds_with": {"demo:none7": {}}},
    "minecraft:offspring": {"offspring_pairs": {"demo:none9": "demo:none10"}},
    "minecraft:reflect_projectiles": {"reflected_projectiles": ["demo:none11"]},
    "minecraft:behavior.fire_at_target": {"priority": 1, "projectile_def": "demo:none12"},
    "minecraft:behavior.follow_mob": {"priority": 1, "preferred_actor_type": "demo:none13"},
    "minecraft:behavior.mingle": {"priority": 1, "mingle_partner_type": "demo:none14"},
    "minecraft:behavior.summon_entity": {"priority": 1, "summon_choices": [{"sequence": [{"entity_type": "demo:none15"}]}]}
"#;

/// No loader of these members logs a miss, and the validator holds no entity component's
/// reference: an information of ours.
#[test]
fn e14_an_entity_nothing_defines_is_an_information() {
    let found = link(&[(FILE, &pet(ENTITIES))], &[]);
    let c = "components/minecraft:";
    let e = |path: &str, id: &str| unresolved(&format!("{c}{path}"), "entity", id);
    assert_eq!(
        said_of(&found, "entity"),
        [
            e("transformation/into", "demo:none1<minecraft:as_adult>"),
            e("spawn_entity/entities/0/spawn_entity", "demo:none2"),
            e("addrider/riders/0/entity_type", "demo:none3"),
            e("spawn_on_death/entity_to_spawn", "demo:none4"),
            e("shooter/def", "demo:none5"),
            e("shooter/projectiles/0/def", "demo:none6"),
            // the 1.21.130+ map's keys are the mates.
            e("breedable/breeds_with/demo:none7", "demo:none7"),
            e("offspring/offspring_pairs/demo:none9", "demo:none9"),
            e("offspring/offspring_pairs/demo:none9", "demo:none10"),
            e("reflect_projectiles/reflected_projectiles/0", "demo:none11"),
            e("behavior.fire_at_target/projectile_def", "demo:none12"),
            e("behavior.follow_mob/preferred_actor_type", "demo:none13"),
            e("behavior.mingle/mingle_partner_type", "demo:none14"),
            e(
                "behavior.summon_entity/summon_choices/0/sequence/0/entity_type",
                "demo:none15"
            ),
        ]
    );
}

/// an older file's `breeds_with` list is the 1.21.130+ map of mates and the
/// `offspring` pairs (the game's upgrade of the component); the mate and the baby it names are linked, each where
/// the file has it.
#[test]
fn e14_the_mate_and_baby_of_a_legacy_breeds_with_list_are_linked() {
    let components = r#""minecraft:breedable": {"breeds_with": [{"mate_type": "demo:none7", "baby_type": "demo:none8"}]}"#;
    let text = entity_with("1.21.120", "demo:pet", components, "");
    let found = link(&[(FILE, &text)], &[]);
    let told: Vec<_> = said_of(&found, "entity")
        .into_iter()
        .map(|said| format!("{said:?}"))
        .collect();
    for id in ["demo:none7", "demo:none8"] {
        assert!(
            told.iter()
                .any(|said| said.contains(id) && said.contains("breeds_with/0")),
            "{id}: {told:#?}"
        );
    }
}

/// An entity a pack defines resolves, with or without an `<event>` after its name; so does
/// the entity itself, and a member that is empty names none.
#[test]
fn e14_an_entity_a_pack_defines_resolves_with_or_without_an_event() {
    let components = r#"
        "minecraft:transformation": {"into": "demo:friend<demo:grow>"},
        "minecraft:spawn_on_death": {"entity_to_spawn": "demo:friend"},
        "minecraft:shooter": {"def": "demo:pet"},
        "minecraft:offspring": {"offspring_pairs": {"demo:pet": ""}},
        "minecraft:spawn_entity": {"entities": {"spawn_entity": "demo:Friend"}}
    "#;
    let found = link(
        &[
            (FILE, &pet(components)),
            (
                "entities/friend.json",
                &entity_with("1.26.50", "demo:friend", "", ""),
            ),
        ],
        &[],
    );
    assert_eq!(
        said_of(&found, "entity"),
        [unresolved(
            "components/minecraft:spawn_entity/entities/spawn_entity",
            "entity",
            "demo:Friend"
        )]
    );
}

/// The game's entities resolve without a host by the crate's list, bare (the namespace
/// defaults to `minecraft`) or with it; with a host only the host's do.
#[test]
fn e14_the_games_entities_resolve_with_a_host_and_without_one() {
    let components = r#"
        "minecraft:transformation": {"into": "minecraft:zombie<minecraft:as_baby>"},
        "minecraft:behavior.follow_mob": {"priority": 1, "preferred_actor_type": "player"},
        "minecraft:shooter": {"def": "minecraft:arrow"}
    "#;
    let files = [(FILE, &*pet(components))];
    let found = link(&files, &[]);
    assert!(said_of(&found, "entity").is_empty(), "{found:?}");
    let found = link_with(
        &files,
        &[],
        hosted(&[
            ("entity", "minecraft:zombie"),
            ("entity", "minecraft:player"),
        ]),
        LinkOptions::default(),
    );
    assert_eq!(
        said_of(&found, "entity"),
        [unresolved(
            "components/minecraft:shooter/def",
            "entity",
            "minecraft:arrow"
        )]
    );
}

/// `type_family.family` and `rideable.family_types` are families, not entity identifiers,
/// and `angry.broadcast_targets` are families too: none is checked.
#[test]
fn e14_families_are_not_entity_identifiers() {
    let components = r#"
        "minecraft:type_family": {"family": ["demo:none", "nothing"]},
        "minecraft:rideable": {"seat_count": 1, "family_types": ["demo:none", "nobody"], "seats": {"position": [0, 1, 0]}},
        "minecraft:angry": {"broadcast_targets": ["demo:none"]}
    "#;
    let found = link(&[(FILE, &pet(components))], &[]);
    assert!(said_of(&found, "entity").is_empty(), "{found:?}");
}

/// Every member of E16 the game reads as a plain string, each naming a sound nothing knows.
const SOUNDS: &str = r#"
    "minecraft:ambient_sound_interval": {"event_name": "demo.none1", "event_names": [{"event_name": "demo.none2"}]},
    "minecraft:explode": {"sound_effect": "demo.none3"},
    "minecraft:shooter": {"def": "minecraft:arrow", "sound": "demo.none4"},
    "minecraft:spawn_entity": {"entities": [{"spawn_sound": "demo.none5"}]},
    "minecraft:transformation": {"into": "minecraft:zombie", "begin_transform_sound": "demo.none6", "transformation_sound": "demo.none7"},
    "minecraft:angry": {"angry_sound": "demo.none8"},
    "minecraft:celebrate_hunt": {"celebrate_sound": "demo.none9"},
    "minecraft:heartbeat": {"sound_event": "demo.none10"},
    "minecraft:reflect_projectiles": {"reflection_sound": "demo.none11"},
    "minecraft:damage_sensor": {"triggers": [{"on_damage_sound_event": "demo.none12"}]},
    "minecraft:behavior.panic": {"priority": 1, "panic_sound": "demo.none13"},
    "minecraft:behavior.tempt": {"priority": 1, "tempt_sound": "demo.none14"},
    "minecraft:interact": {"interactions": [{"play_sounds": ["demo.none15"]}]},
    "minecraft:projectile": {"hit_sound": "demo.none16", "shoot_sound": "demo.none17", "hit_ground_sound": "demo.none18"},
    "minecraft:anger_level": {"on_increase_sounds": [{"sound": "demo.none19", "condition": "query.anger_level > 1"}]}
"#;

/// The game answers a name it lacks without a line; the
/// level-3 "Event name '%s' is not a valid `LevelSoundEvent`" of the game's sound-event lookup
/// is reached from the sound definitions and the particle events alone, not from
/// an entity: an information of ours.
#[test]
fn e16_a_sound_of_a_component_the_game_reads_as_a_string_that_nothing_knows_is_an_information() {
    let found = link(&[(FILE, &pet(SOUNDS))], &[]);
    let s = |path: &str, id: &str| {
        miss(
            "core/unknown_sound",
            Severity::Info,
            &format!("minecraft:entity/components/minecraft:{path}"),
            "level sound event",
            id,
        )
    };
    assert_eq!(
        said_of(&found, "level sound event"),
        [
            s("ambient_sound_interval/event_name", "demo.none1"),
            s(
                "ambient_sound_interval/event_names/0/event_name",
                "demo.none2"
            ),
            s("explode/sound_effect", "demo.none3"),
            s("shooter/sound", "demo.none4"),
            s("spawn_entity/entities/0/spawn_sound", "demo.none5"),
            s("transformation/begin_transform_sound", "demo.none6"),
            s("transformation/transformation_sound", "demo.none7"),
            s("angry/angry_sound", "demo.none8"),
            s("celebrate_hunt/celebrate_sound", "demo.none9"),
            s("heartbeat/sound_event", "demo.none10"),
            s("reflect_projectiles/reflection_sound", "demo.none11"),
            s(
                "damage_sensor/triggers/0/on_damage_sound_event",
                "demo.none12"
            ),
            s("behavior.panic/panic_sound", "demo.none13"),
            s("behavior.tempt/tempt_sound", "demo.none14"),
            s("interact/interactions/0/play_sounds/0", "demo.none15"),
            s("projectile/hit_ground_sound", "demo.none18"),
            s("projectile/hit_sound", "demo.none16"),
            s("projectile/shoot_sound", "demo.none17"),
            s("anger_level/on_increase_sounds/0/sound", "demo.none19"),
        ]
    );
}

/// E16 behind a sound of `anger_level` the lenient conversion drops (its condition is one
/// the reader does not take): the index is the file's.
#[test]
fn e16_a_sound_behind_one_the_lenient_conversion_drops_is_at_the_index_of_the_file() {
    let components = r#""minecraft:anger_level": {"on_increase_sounds": [7, {"sound": "demo.none1", "condition": {"expression": "1"}}, {"sound": "demo.none2", "condition": "1"}]}"#;
    // Below 1.15.0 (from it the whole component would keep its defaults).
    let text = entity_with("1.14.0", "demo:pet", components, "");
    let found = link(&[(FILE, &text)], &[]);
    assert_eq!(
        said_of(&found, "level sound event"),
        [miss(
            "core/unknown_sound",
            Severity::Info,
            "minecraft:entity/components/minecraft:anger_level/on_increase_sounds/2/sound",
            "level sound event",
            "demo.none2",
        )],
        "{found:#?}"
    );
}

/// The game's sound events resolve without a host by the crate's list, in any case; with a
/// host only the host's do. An empty member names none.
#[test]
fn e16_the_games_sound_events_resolve_with_a_host_and_without_one() {
    let components = r#"
        "minecraft:ambient_sound_interval": {"event_name": "ambient", "event_names": [{"event_name": "Ambient.Tame"}]},
        "minecraft:explode": {"sound_effect": ""}
    "#;
    let files = [(FILE, &*pet(components))];
    let found = link(&files, &[]);
    assert!(said_of(&found, "level sound event").is_empty(), "{found:?}");
    let found = link_with(
        &files,
        &[],
        hosted(&[("level sound event", "ambient.tame")]),
        LinkOptions::default(),
    );
    let told = said_of(&found, "level sound event");
    assert_eq!(told.len(), 1, "{found:?}");
    assert_eq!(
        told[0].3,
        "minecraft:entity/components/minecraft:ambient_sound_interval/event_name"
    );
    assert_eq!(told[0].5, "ambient");
}

/// E16, the typed half: a goal's sound is a `LevelSoundEvent` of the typed parse. An unknown
/// name is told by the load (rule 15), and the link pass tells nothing more of it.
#[test]
fn e16_a_typed_goals_unknown_sound_is_the_loads_finding_not_the_passs() {
    let components = r#""minecraft:behavior.sneeze": {"priority": 1, "sound": "demo.none", "prepare_sound": "demo.none"}"#;
    let set = super::super::common::stacks(
        &[(FILE, &pet(components))],
        &[],
        bedrock_addon::load::LoadOptions::default(),
    );
    let found = set.link();
    assert!(said_of(&found, "level sound event").is_empty(), "{found:?}");
    let load: Vec<_> = set
        .behavior
        .entities()
        .diagnostics()
        .chain(
            set.behavior
                .entities()
                .iter()
                .flat_map(|(_, document)| document.diagnostics().iter()),
        )
        .filter(|finding| {
            finding
                .location
                .json_path
                .to_string()
                .contains("behavior.sneeze")
        })
        .map(|finding| (finding.code.id(), finding.severity))
        .collect();
    assert!(
        load.iter().any(|(_, severity)| *severity == Severity::Info),
        "the load tells the unknown sound: {load:?}"
    );
}

/// The game's own files name entities that no file defines and the list of the game's
/// entities lacks (`minecraft:evocation_fang`, the old `minecraft:pig_zombie`,
/// `minecraft:skeleton.stray`): each resolves, with any host.
#[test]
fn e14_an_entity_name_of_the_games_own_files_resolves() {
    let components = r#"
        "minecraft:behavior.summon_entity": {"priority": 1, "summon_choices": [{"sequence": [{"entity_type": "minecraft:evocation_fang"}]}]},
        "minecraft:transformation": {"into": "minecraft:pig_zombie"},
        "minecraft:addrider": {"riders": [{"entity_type": "minecraft:skeleton.stray"}, {"entity_type": "minecraft:skeleton.wither"}, {"entity_type": "minecraft:skeleton.none"}, {"entity_type": "Minecraft:Zombie"}, {"entity_type": "minecraft:Evocation_Fang"}]}
    "#;
    let files = [(FILE, &*pet(components))];
    // A name is compared as written: another case of a known name is told.
    let at = "components/minecraft:addrider/riders";
    let expected = [
        unresolved(
            &format!("{at}/2/entity_type"),
            "entity",
            "minecraft:skeleton.none",
        ),
        unresolved(&format!("{at}/3/entity_type"), "entity", "Minecraft:Zombie"),
        unresolved(
            &format!("{at}/4/entity_type"),
            "entity",
            "minecraft:Evocation_Fang",
        ),
    ];
    assert_eq!(said_of(&link(&files, &[]), "entity"), expected);
    let found = link_with(&files, &[], hosted(&[]), LinkOptions::default());
    assert_eq!(said_of(&found, "entity"), expected);
}
