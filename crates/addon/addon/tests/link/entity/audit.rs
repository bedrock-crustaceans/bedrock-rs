//! The entity and sound reference rows that the link pass does not resolve: the rows the load
//! tells (E5's and E8's own half, E6, E9, SR4) and the rows only the running game can
//! resolve (E7, E18, E19, SR3, SR5 and the other halves of E5 and E9), each pinned as what
//! it is.

#![allow(unused_imports, reason = "each feature set compiles part of the tests")]

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::load::{Experiment, Experiments, LoadOptions};
use bedrock_addon::pack::StackSet;

use super::super::common::stacks;
use super::{FILE, entity_with, pet};

/// The codes of a reference finding (one code per kind of miss).
const REFERENCE_CODES: [&str; 6] = [
    "UnresolvedReference",
    "InvalidReference",
    "ReferenceCycle",
    "UnknownSound",
    "TextureNotFound",
    "UnresolvedLangKey",
];

/// What a finding says: code, severity, JSON path.
type Told = (&'static str, Severity, String);

fn told(finding: &Diagnostic) -> Told {
    (
        finding.code.name(),
        finding.severity,
        finding.location.json_path.to_string(),
    )
}

/// The reference findings among `findings`.
fn references<'a>(findings: impl IntoIterator<Item = &'a Diagnostic>) -> Vec<Told> {
    findings
        .into_iter()
        .filter(|finding| REFERENCE_CODES.contains(&finding.code.name()))
        .map(told)
        .collect()
}

/// A world of the one entity file `text`, with the experiments the Brain fixtures need.
fn world(text: &str) -> StackSet {
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::BrainPrototype);
    let options = LoadOptions {
        experiments,
        ..LoadOptions::default()
    };
    stacks(&[(FILE, text)], &[], options)
}

/// The reference findings of the load of the entity of `set`: the registry's and the
/// document's own, each once.
fn load_references(set: &StackSet) -> Vec<Told> {
    let registry = set.behavior.entities();
    assert_eq!(
        registry.len(),
        1,
        "the entity loads: {:?}",
        registry.rejected()
    );
    let mut all = references(registry.diagnostics());
    for (_, document) in registry {
        for finding in references(document.diagnostics().iter()) {
            if !all.contains(&finding) {
                all.push(finding);
            }
        }
    }
    all
}

/// Neither the load nor the link pass tells a reference finding for the entity `text`.
fn assert_not_checked(text: &str) {
    let set = world(text);
    assert_eq!(load_references(&set), [], "the load");
    assert_eq!(references(set.link().iter()), [], "the link pass");
}

fn warning(path: &str) -> Told {
    // A kept error: "Filter test not found" is level 3 and the test evaluates
    // false.
    (
        "UnresolvedReference",
        Severity::Error,
        format!("minecraft:entity/{path}"),
    )
}

/// A reference of the entity to itself that names nothing: the game's own packs fire events
/// their definitions lack and the game runs them, so the finding is an information.
fn info(path: &str) -> Told {
    (
        "UnresolvedReference",
        Severity::Info,
        format!("minecraft:entity/{path}"),
    )
}

// ---- E5: triggers ---------------------------------------------------------------------------

/// E5, the load half: a trigger whose target is the entity itself (`self`, or no target)
/// names one of the entity's own events. In an event node the trigger is a name or an object.
#[test]
fn e5_an_event_nodes_trigger_on_self_naming_no_event_of_the_entity_is_told_at_load() {
    let text = entity_with(
        "1.26.50",
        "demo:pet",
        "",
        r#", "events": {
            "demo:a": {"trigger": "demo:b"},
            "demo:b": {"sequence": [{"trigger": "demo:none1"}, {"trigger": {"event": "demo:none2", "target": "self"}}, {"trigger": {"event": "demo:none3"}}, {"trigger": {"event": "demo:a"}}]},
            "demo:c": {"randomize": [{"weight": 1, "trigger": {"event": "demo:none4", "target": "other"}}]}
        }"#,
    );
    let set = world(&text);
    assert_eq!(
        load_references(&set),
        [
            info("events/demo:b/sequence/0/trigger"),
            info("events/demo:b/sequence/1/trigger/event"),
            info("events/demo:b/sequence/2/trigger/event"),
        ]
    );
    // The link pass adds nothing: the row is the load's.
    assert_eq!(references(set.link().iter()), []);
}

/// E5 in components: a typed goal's trigger, the trigger of a component the game reads at
/// run time, one in a component group, and a list of triggers.
#[test]
fn e5_a_components_trigger_on_self_naming_no_event_of_the_entity_is_an_information_at_load() {
    let text = entity_with(
        "1.26.50",
        "demo:pet",
        r#"
        "minecraft:behavior.eat_block": {"priority": 1, "on_eat": {"event": "demo:none1", "target": "self"}},
        "minecraft:timer": {"time": 1, "time_down_event": {"event": "demo:none2"}},
        "minecraft:on_death": {"event": "demo:known", "target": "self"},
        "minecraft:on_hurt": {"event": "demo:none3", "target": "other"},
        "minecraft:behavior.go_home": {"priority": 1, "on_home": [{"event": "demo:known"}, {"event": "demo:none4", "target": "self"}]},
        "minecraft:damage_sensor": {"triggers": [{"on_damage": {"event": "demo:none5", "filters": {"test": "is_family", "value": "x"}}, "deals_damage": "no"}]},
        "minecraft:looked_at": {"looked_at_event": {"event": "demo:none7", "filter": "self"}, "search_radius": 3}"#,
        r#", "component_groups": {"g": {"minecraft:on_target_acquired": {"event": "demo:none6"}}},
            "events": {"demo:known": {}}"#,
    );
    let set = world(&text);
    let c = "components/minecraft:";
    assert_eq!(
        load_references(&set),
        [
            info(&format!("{c}behavior.eat_block/on_eat/event")),
            info(&format!("{c}timer/time_down_event/event")),
            info(&format!("{c}behavior.go_home/on_home/1/event")),
            info(&format!("{c}damage_sensor/triggers/0/on_damage/event")),
            info(&format!("{c}looked_at/looked_at_event/event")),
            info("component_groups/g/minecraft:on_target_acquired/event"),
        ]
    );
    assert_eq!(references(set.link().iter()), []);
}

/// E5: the triggers a component writes as a bare event name.
#[test]
fn e5_a_components_trigger_written_as_a_name_is_checked_where_the_member_is_known() {
    let text = entity_with(
        "1.26.50",
        "demo:pet",
        r#"
        "minecraft:rideable": {"seat_count": 1, "seats": {"position": [0, 1, 0]}, "on_rider_enter_event": "demo:none1", "on_rider_exit_event": "demo:known"},
        "minecraft:entity_sensor": {"subsensors": [{"event": "demo:none2", "range": [1, 1]}, {"event": "demo:known"}]},
        "minecraft:block_sensor": {"sensor_radius": 16, "on_break": [{"block_list": ["minecraft:stone"], "on_block_broken": "demo:none3"}]}"#,
        r#", "events": {"demo:known": {}}"#,
    );
    let set = world(&text);
    let c = "components/minecraft:";
    assert_eq!(
        load_references(&set),
        [
            info(&format!("{c}rideable/on_rider_enter_event")),
            info(&format!("{c}entity_sensor/subsensors/0/event")),
            info(&format!("{c}block_sensor/on_break/0/on_block_broken")),
        ]
    );
}

/// K123: the event a mate's breeding fires. `minecraft:breedable.breeds_with` holds one
/// trigger per mate in the newest format (`{"<mate>": {event, target}}`), and an older file's
/// list of `{mate_type, baby_type, breed_event}` is converted to it; the game's own files
/// write `target: baby` (the newborn's event), `target: self` (frog, sniffer, turtle) or no
/// target. A trigger on the entity itself naming no event of it is told; one on the baby is
/// the baby's.
#[test]
fn e5_the_breed_event_of_a_mate_on_self_naming_no_event_is_an_information_at_load() {
    // The newest format's map.
    let text = entity_with(
        "1.26.50",
        "demo:pet",
        r#""minecraft:breedable": {"breeds_with": {
            "demo:a": {"event": "demo:none1", "target": "self"},
            "demo:b": {"event": "demo:none2"},
            "demo:c": {"event": "demo:none3", "target": "baby"},
            "demo:d": {"event": "demo:known", "target": "self"}}}"#,
        r#", "events": {"demo:known": {}}"#,
    );
    let set = world(&text);
    let at = "components/minecraft:breedable/breeds_with";
    assert_eq!(
        load_references(&set),
        [
            info(&format!("{at}/demo:a/event")),
            info(&format!("{at}/demo:b/event")),
        ]
    );
    // An older file's list, and its single object: converted to the map and checked there,
    // told at the member the file has.
    for (breeds_with, member) in [
        (
            r#"[{"mate_type": "demo:a", "baby_type": "demo:pet", "breed_event": {"event": "demo:none1", "target": "self"}},
            {"mate_type": "demo:c", "baby_type": "demo:pet", "breed_event": {"event": "demo:none3", "target": "baby"}}]"#,
            "0/breed_event",
        ),
        (
            r#"{"mate_type": "demo:a", "baby_type": "demo:pet", "breed_event": {"event": "demo:none1", "target": "self"}}"#,
            "breed_event",
        ),
    ] {
        let text = entity_with(
            "1.21.0",
            "demo:pet",
            &format!(r#""minecraft:breedable": {{"breeds_with": {breeds_with}}}"#),
            r#", "events": {"demo:known": {}}"#,
        );
        let set = world(&text);
        assert_eq!(
            load_references(&set),
            [info(&format!("{at}/{member}/event"))],
            "{breeds_with}"
        );
    }
}

/// K123: the triggers of `sittable`, `balloonable` and `ageable` that the schema lists and
/// no file of the game writes are triggers on the entity itself like the others.
#[test]
fn e5_the_sit_balloon_and_growth_triggers_on_self_are_checked() {
    let text = entity_with(
        "1.26.50",
        "demo:pet",
        r#"
        "minecraft:sittable": {"sit_event": {"event": "demo:none1"}, "stand_event": {"event": "demo:none2", "target": "self"}},
        "minecraft:balloonable": {"on_balloon": {"event": "demo:none3"}, "on_unballoon": {"event": "demo:known"}},
        "minecraft:ageable": {"duration": 1200, "grow_up": {"event": "demo:known"}, "on_pause_growth": {"event": "demo:none4"}, "on_reset_growth": {"event": "demo:none5", "target": "other"}}"#,
        r#", "events": {"demo:known": {}}"#,
    );
    let set = world(&text);
    let c = "components/minecraft:";
    let mut found = load_references(&set);
    found.sort();
    let mut expected = [
        info(&format!("{c}sittable/sit_event/event")),
        info(&format!("{c}sittable/stand_event/event")),
        info(&format!("{c}balloonable/on_balloon/event")),
        info(&format!("{c}ageable/on_pause_growth/event")),
    ];
    expected.sort();
    assert_eq!(found, expected);
}

/// K123: `spawn_egg_interaction.on_spawn` is the event of the entity the egg spawns, as the
/// spawn events of `addrider`, `spawn_entity` and a projectile are: it stays run.
#[test]
fn e5_the_event_a_spawn_egg_interaction_fires_on_the_spawned_entity_stays_run() {
    assert_not_checked(&entity_with(
        "1.26.50",
        "demo:pet",
        r#""minecraft:spawn_egg_interaction": {"on_spawn": {"event": "demo:none", "target": "self"}},
           "minecraft:offspring": {"offspring_pairs": {"demo:pet": "demo:pet"}},
           "minecraft:addrider": {"riders": [{"entity_type": "minecraft:pig", "spawn_event": "demo:none"}]}"#,
        "",
    ));
}

/// E5, the run half: a trigger whose target is another entity names an event of whatever
/// entity that is when it fires.
#[test]
fn e5_a_trigger_on_another_entity_stays_run() {
    assert_not_checked(&entity_with(
        "1.26.50",
        "demo:pet",
        r#""minecraft:on_hurt": {"event": "demo:none", "target": "other"},
           "minecraft:behavior.eat_block": {"priority": 1, "on_eat": {"event": "demo:none", "target": "player"}},
           "minecraft:spawn_entity": {"entities": [{"spawn_entity": "minecraft:pig", "spawn_event": "demo:none"}]},
           "minecraft:addrider": {"riders": [{"entity_type": "minecraft:pig", "spawn_event": "demo:none"}]}"#,
        r#", "events": {"demo:a": {"trigger": {"event": "demo:none", "target": "target"}}}"#,
    ));
}

/// E5, the run half: an event a component fires on another entity than its own (the entity a
/// projectile spawns or hits, the entities a spell reaches) is that entity's, whatever the
/// trigger's target says or leaves out.
#[test]
fn e5_an_event_fired_on_a_spawned_hit_or_reached_entity_stays_run() {
    assert_not_checked(&entity_with(
        "1.26.50",
        "demo:pet",
        r#""minecraft:projectile": {"on_hit": {
                "spawn_chance": {"first_spawn_chance": 0.125, "first_spawn_count": 1, "spawn_definition": "minecraft:chicken", "spawn_baby": true,
                    "on_spawn": [{"event": "minecraft:hatch_warm", "filters": {"domain": "minecraft:climate_variant", "subject": "other", "test": "enum_property", "value": "warm"}}, {"event": "minecraft:hatch_cold"}]},
                "definition_event": {"affect_target": true, "event_trigger": {"event": "demo:none", "target": "self"}}}},
           "minecraft:behavior.send_event": {"priority": 1, "event_choices": [{"min_activation_range": 0, "max_activation_range": 16, "cooldown_time": 5, "cast_duration": 3, "weight": 3,
                "sequence": [{"event": "wololo"}, {"base_delay": 2, "event": "wololo", "sound_event": "prepare.wololo"}]}]}"#,
        "",
    ));
}

/// E5 is a list of the members that are triggers on the entity itself: an object of the same
/// shape anywhere else in a component is none.
#[test]
fn e5_an_object_shaped_like_a_trigger_in_another_member_is_not_checked() {
    assert_not_checked(&pet(
        r#""minecraft:timer": {"time": 1, "time_down_event": {"event": "demo:known"}, "looping": {"event": "demo:none"}},
           "minecraft:shooter": {"def": "minecraft:arrow", "projectiles": [{"def": "minecraft:arrow", "filters": {"event": "demo:none", "target": "self"}}]}"#,
    ).replace(r#""components""#, r#""events": {"demo:known": {}}, "components""#));
}

// ---- E6, E7: properties ---------------------------------------------------------------------

/// E6: a `set_property` key names one of the entity's properties.
#[test]
fn e6_a_set_property_key_naming_no_property_of_the_entity_is_an_information_at_load() {
    let text = r#"{"format_version": "1.26.50", "minecraft:entity": {"description": {"identifier": "demo:pet",
        "properties": {"demo:mood": {"type": "enum", "values": ["calm", "wild"], "default": "calm"}, "demo:age": {"type": "int", "range": [0, 5], "default": 0}}},
        "components": {},
        "events": {
            "demo:a": {"set_property": {"demo:mood": "wild", "demo:none1": 1}},
            "demo:b": {"sequence": [{"set_property": {"demo:age": 2}}, {"set_property": {"demo:none2": true}}]}
        }}}"#;
    let set = world(text);
    assert_eq!(
        load_references(&set),
        [
            info("events/demo:a/set_property/demo:none1"),
            info("events/demo:b/sequence/1/set_property/demo:none2"),
        ]
    );
    assert_eq!(references(set.link().iter()), []);
}

/// E7: a group or a property named as the argument of a Molang query is the game's to
/// resolve when the expression runs.
#[test]
fn e7_a_group_or_property_named_in_a_molang_query_stays_run() {
    assert_not_checked(
        r#"{"format_version": "1.26.50", "minecraft:entity": {"description": {"identifier": "demo:pet",
        "properties": {"demo:big": {"type": "bool", "default": "query.had_component_group('demo:none')"}},
        "animations": {}, "scripts": {"animate": []}},
        "components": {"minecraft:behavior.eat_block": {"priority": 1, "success_chance": "query.property('demo:none') ? 0.1 : 0.2"}},
        "events": {"demo:a": {"set_property": {"demo:big": "query.property('demo:none')"}}}}}"#,
    );
}

// ---- E8, the load half: scripts.animate -----------------------------------------------------

/// E8, the load half: an entry of `scripts.animate` names a key of the entity's own
/// `animations`. A plain name is looked up in lower case, as the game does; the key of a
/// conditional entry as written.
#[test]
fn e8_an_animate_entry_naming_no_animation_of_the_entity_is_an_information_at_load() {
    let text = r#"{"format_version": "1.26.50", "minecraft:entity": {"description": {"identifier": "demo:pet",
        "animations": {"walk": "animation.demo.walk", "look": "controller.animation.demo.look"},
        "scripts": {"animate": ["walk", "Walk", "none1", {"look": "query.is_moving"}, {"none2": "1"}, {"Look": "1"}]}},
        "components": {}}}"#;
    let set = world(text);
    assert_eq!(
        load_references(&set),
        [
            info("description/scripts/animate/2"),
            info("description/scripts/animate/4/none2"),
            info("description/scripts/animate/5/Look"),
        ]
    );
}

/// E8 behind entries the reader does not hold (one that is neither a
/// string nor an object, one with a condition that is no string): the index is the file's.
#[test]
fn e8_an_animate_entry_behind_one_the_reader_drops_is_told_at_the_index_of_the_file() {
    let text = r#"{"format_version": "1.26.50", "minecraft:entity": {"description": {"identifier": "demo:pet",
        "animations": {"walk": "animation.demo.walk"},
        "scripts": {"animate": [7, "none1", {"walk": 3}, {"none2": "1"}]}},
        "components": {}}}"#;
    let set = world(text);
    assert_eq!(
        load_references(&set),
        [
            info("description/scripts/animate/1"),
            info("description/scripts/animate/3/none2"),
        ]
    );
}

// ---- E9: filters ----------------------------------------------------------------------------

/// E9 where the family did not look: an unknown filter test in the trigger of a component
/// the game reads at run time (`environment_sensor.triggers[].filters`,
/// `damage_sensor.triggers[].on_damage.filters`) warns as it does in a typed component.
#[test]
fn e9_an_unknown_filter_test_in_a_sensors_trigger_is_a_warning_at_load() {
    let text = entity_with(
        "1.26.50",
        "demo:pet",
        r#"
        "minecraft:environment_sensor": {"triggers": [
            {"filters": {"test": "no_such_test1", "value": 1}, "event": "demo:known"},
            {"filters": {"all_of": [{"test": "is_daytime", "value": true}, {"test": "no_such_test2"}]}, "event": "demo:known"}]},
        "minecraft:damage_sensor": {"triggers": [
            {"on_damage": {"filters": {"test": "no_such_test3"}, "event": "demo:known"}, "deals_damage": "no"},
            {"on_damage": {"filters": {"test": "no_such_test_c", "value": 1}}, "deals_damage": "no"}]},
        "minecraft:on_hurt": {"event": "demo:known", "filters": {"test": "no_such_test4"}}"#,
        r#", "events": {"demo:known": {}}"#,
    );
    let set = world(&text);
    let c = "components/minecraft:";
    assert_eq!(
        load_references(&set),
        [
            warning(&format!("{c}on_hurt/filters")),
            warning(&format!("{c}environment_sensor/triggers/0/filters")),
            warning(&format!(
                "{c}environment_sensor/triggers/1/filters/all_of/1"
            )),
            warning(&format!("{c}damage_sensor/triggers/0/on_damage/filters")),
            warning(&format!("{c}damage_sensor/triggers/1/on_damage/filters")),
        ]
    );
    let registry = set.behavior.entities();
    let texts: Vec<String> = registry
        .diagnostics()
        .filter(|finding| finding.code.name() == "UnresolvedReference")
        .map(|finding| finding.code.to_string())
        .collect();
    assert!(
        texts.contains(&"Filter test not found -> no_such_test3".to_owned()),
        "{texts:?}"
    );
    // Told in the phase the family tells a typed component's filter in.
    let phases: Vec<_> = registry
        .diagnostics()
        .filter(|finding| finding.code.name() == "UnresolvedReference")
        .map(|finding| finding.phase)
        .collect();
    assert_eq!(phases.len(), 5);
    assert!(phases.iter().all(|phase| *phase == phases[0]), "{phases:?}");
}

/// E9 in a sensor's trigger tells the unknown test names and nothing else: a member of the
/// older filter form the game still maps to a test (`other_with_families`, as the game's own
/// skeleton writes it) is none of this check's findings.
#[test]
fn e9_a_legacy_filter_member_in_a_sensors_trigger_is_not_told() {
    let text = pet(
        r#""minecraft:damage_sensor": {"triggers": [{"on_damage": {"filters": {"other_with_families": "lightning"}}, "deals_damage": "no"}]}"#,
    );
    let set = world(&text);
    let registry = set.behavior.entities();
    assert_eq!(registry.len(), 1);
    let told: Vec<Told> = registry
        .diagnostics()
        .filter(|finding| finding.severity >= Severity::Warning)
        .map(told)
        .collect();
    assert_eq!(told, []);
}

/// K123: a member of the older filter form that is none of the game's (a misspelt one) is
/// the game's "Invalid `EntityFilterGroup` member found", a warning at the member, in a list
/// and in a group too; the members the game maps to tests and the keys of a group are not
/// told, and the members of a test object are another rule's.
#[test]
fn e9_an_unknown_member_of_the_older_filter_form_in_a_sensors_trigger_is_a_warning_at_load() {
    let text = entity_with(
        "1.26.50",
        "demo:pet",
        r#"
        "minecraft:damage_sensor": {"triggers": [
            {"on_damage": {"filters": {"other_with_familys": "lightning", "with_damage_fatal": true}}, "deals_damage": "no"},
            {"on_damage": {"filters": {"all_of": [{"is_daytime": true}, {"is_daytim": true}], "any_of": {"without_item": "x", "whithout_item": "x"}}}, "deals_damage": "no"}]},
        "minecraft:environment_sensor": {"triggers": [
            {"filters": [{"is_underground": true}, {"is_undergrond": true}], "event": "demo:known"},
            {"filters": {"test": "is_daytime", "valeu": true}, "event": "demo:known"}]}"#,
        r#", "events": {"demo:known": {}}"#,
    );
    let set = world(&text);
    let registry = set.behavior.entities();
    assert_eq!(registry.len(), 1, "{:?}", registry.rejected());
    let unknown: Vec<(Told, String, bedrock_addon::diagnostic::Phase)> = registry
        .diagnostics()
        .filter(|finding| finding.code.name() == "UnrecognizedField")
        .filter(|finding| finding.location.json_path.to_string().contains("/filters/"))
        .map(|finding| (told(finding), finding.code.to_string(), finding.phase))
        .collect();
    let c = "minecraft:entity/components/minecraft:";
    let warning = |path: String, member: &str| {
        (
            ("UnrecognizedField", Severity::Error, path),
            format!("Invalid EntityFilterGroup member found: {member}"),
            bedrock_addon::diagnostic::Phase::Parse,
        )
    };
    assert_eq!(
        unknown,
        [
            warning(
                format!("{c}damage_sensor/triggers/0/on_damage/filters/other_with_familys"),
                "other_with_familys"
            ),
            warning(
                format!("{c}damage_sensor/triggers/1/on_damage/filters/all_of/1/is_daytim"),
                "is_daytim"
            ),
            warning(
                format!("{c}damage_sensor/triggers/1/on_damage/filters/any_of/whithout_item"),
                "whithout_item"
            ),
            warning(
                format!("{c}environment_sensor/triggers/0/filters/1/is_undergrond"),
                "is_undergrond"
            ),
        ]
    );
}

/// E9, what is never checked: the values of a filter test (a family, a component, a biome
/// tag) are an open set the game resolves when the test runs.
#[test]
fn e9_the_values_of_a_filter_test_are_not_checked() {
    assert_not_checked(&entity_with(
        "1.26.50",
        "demo:pet",
        r#""minecraft:on_hurt": {"event": "demo:known", "filters": {"any_of": [
            {"test": "is_family", "value": "demo_none"},
            {"test": "has_component", "value": "minecraft:none"},
            {"test": "is_biome", "value": "demo_none"},
            {"test": "has_biome_tag", "value": "demo_none"},
            {"test": "has_tag", "value": "demo_none"}]}}"#,
        r#", "events": {"demo:known": {}}"#,
    ));
}

// ---- E18, E19: what only the running game resolves ------------------------------------------

/// E18: the sound of `play_sound`, the command of `queue_command` and the block event of
/// `execute_event_on_home_block` are read when the response runs.
#[test]
fn e18_the_sound_command_and_block_event_of_an_event_response_stay_run() {
    assert_not_checked(&entity_with(
        "1.26.50",
        "demo:pet",
        "",
        r#", "events": {"demo:a": {"play_sound": {"sound": "demo.none"}, "queue_command": {"command": "no_such_command 1"}, "execute_event_on_home_block": {"event": "demo:none"}}}"#,
    ));
}

/// E19: a mob effect is a name of the game's own table, which no rule holds.
#[test]
fn e19_a_mob_effect_is_not_checked() {
    assert_not_checked(&pet(r#"
        "minecraft:mob_effect": {"mob_effect": "demo_none", "effect_range": 1, "effect_time": 1},
        "minecraft:mob_effect_immunity": {"mob_effects": ["demo_none"]},
        "minecraft:spell_effects": {"add_effects": [{"effect": "demo_none", "duration": 1}], "remove_effects": "demo_none"},
        "minecraft:attack": {"damage": 1, "effect_name": "demo_none", "effect_duration": 1}"#));
}

// ---- spawn rules ----------------------------------------------------------------------------

/// SR3: the events of a herd and the mob event of a filter are read when the entity spawns.
#[cfg(all(feature = "spawn_rules", feature = "spawn_groups"))]
#[test]
fn sr3_the_events_of_spawn_rules_stay_run() {
    let rules = r#"{"format_version": "1.8.0", "minecraft:spawn_rules": {"description": {"identifier": "minecraft:pig", "population_control": "animal"}, "conditions": [{"minecraft:weight": {"default": 1},
        "minecraft:herd": {"min_size": 1, "max_size": 2, "event": "demo:none", "event_skip_count": 1, "initial_event": "demo:none", "initial_event_count": 1},
        "minecraft:mob_event_filter": {"event": "demo:none"}}]}}"#;
    let set = stacks(
        &[("spawn_rules/pig.json", rules)],
        &[],
        LoadOptions::default(),
    );
    let registry = set.behavior.spawn_rules();
    assert_eq!(registry.len(), 1);
    assert_eq!(references(registry.diagnostics()), []);
    assert_eq!(references(set.link().iter()), []);
}

/// SR4 is the load's: a placement name is one of the game's registries, which only a script
/// extends, at start-up, with a name it computes; nothing on a stack registers one, so the
/// link pass has nothing to resolve it against and adds no finding.
#[cfg(all(feature = "spawn_rules", feature = "spawn_groups"))]
#[test]
fn sr4_an_unregistered_placement_name_is_the_loads_warning_and_the_pass_adds_none() {
    let rules = r#"{"format_version": "1.21.60", "minecraft:spawn_rules": {"description": {"identifier": "minecraft:pig"},
        "placement": {"type": "demo:registered_by_script", "spawn_predicate": "minecraft:animal", "heightmap": "motion_blocking"},
        "conditions": [{"weight": 1}]}}"#;
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::SimplifiedSpawnRules);
    let options = LoadOptions {
        experiments,
        ..LoadOptions::default()
    };
    let script = "import { system } from '@minecraft/server';\nsystem.beforeEvents.startup.subscribe((e) => { e.registerEntitySpawnCallback('demo:registered_by_script', () => true); });\n";
    let set = stacks(
        &[("spawn_rules/pig.json", rules), ("scripts/main.js", script)],
        &[],
        options,
    );
    let registry = set.behavior.spawn_rules();
    let mut load = references(registry.diagnostics());
    for (_, document) in registry {
        for finding in references(document.diagnostics().iter()) {
            if !load.contains(&finding) {
                load.push(finding);
            }
        }
    }
    assert_eq!(
        load,
        [
            (
                "UnresolvedReference",
                Severity::Warning,
                "minecraft:spawn_rules/placement/type".to_owned()
            ),
            (
                "UnresolvedReference",
                Severity::Warning,
                "minecraft:spawn_rules/placement/heightmap".to_owned()
            ),
        ]
    );
    assert_eq!(references(set.link().iter()), []);
}

/// SR5: the identifier of a spawn group is an entity name or a raid group, which the game
/// looks up when a raid asks for it.
#[cfg(all(feature = "spawn_rules", feature = "spawn_groups"))]
#[test]
fn sr5_the_identifier_of_a_spawn_group_stays_run() {
    use bedrock_addon::load::AppData;
    use bedrock_addon::storage::PackPath;
    let group = r#"{"format_version": "1.11.0", "minecraft:spawn_groups": {"description": {"identifier": "demo:no_such_entity_or_raid_group"}, "conditions": [{"minecraft:herd": {"min_size": 1, "max_size": 1}}]}}"#;
    let app_data = AppData::from_files([(
        PackPath::new("definitions/spawn_groups/x.json").expect("a path"),
        group.as_bytes().to_vec(),
    )]);
    let options = LoadOptions {
        app_data: Some(app_data),
        ..LoadOptions::default()
    };
    let set = stacks(&[], &[], options);
    let registry = set.behavior.spawn_groups();
    assert_eq!(registry.len(), 1);
    assert_eq!(references(registry.diagnostics()), []);
    assert_eq!(references(set.link().iter()), []);
}
