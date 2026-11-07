//! The family's validation rules: identifiers, aliases, root members, brain memories, generic
//! goals and component versions.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Severity};
use bedrock_addon_core::document::ComponentRef;
use bedrock_addon_core::document::JsonDocument;
use bedrock_addon_core::load::{Experiment, LoadContext};
use bedrock_addon_core::versioning::Loaded;
use bedrock_addon_family_entity::codes as entity_codes;
use bedrock_addon_family_entity::{Entity, PropertyDefault};

use super::common::{codes, cx, doc, json, outcome, reparse, stable, with_code};

/// A payload with `identifier` and the members in `rest` (JSON members, without braces).
fn payload(identifier: &str, rest: &str) -> String {
    let rest = if rest.is_empty() {
        String::new()
    } else {
        format!(", {rest}")
    };
    format!(r#"{{ "description": {{ "identifier": "{identifier}" }}, "components": {{}}{rest} }}"#)
}

fn rejected(version: &str, payload: &str, cx: &LoadContext) -> Vec<&'static str> {
    let outcome = outcome(&doc(version, payload), cx);
    assert!(outcome.value.is_none(), "expected a rejection");
    codes(&outcome.diagnostics, Severity::Error)
}

#[test]
fn a_custom_minecraft_identifier_is_rejected() {
    assert_eq!(
        rejected("1.26.50", &payload("minecraft:my_mob", ""), &cx()),
        ["core/namespace_rule"]
    );
    // The rule's text is ours, so none is attached.
    let outcome = outcome(&doc("1.26.50", &payload("minecraft:my_mob", "")), &cx());
    assert_eq!(
        with_code(&outcome.diagnostics, "core/namespace_rule")[0].code,
        Code::from(&core_codes::NAMESPACE_RULE)
    );
}

#[test]
fn a_vanilla_override_may_use_the_minecraft_namespace() {
    stable(&doc("1.26.50", &payload("minecraft:cow", "")), &cx());
}

#[test]
fn a_base_game_pack_may_use_any_minecraft_identifier() {
    let mut base = cx();
    base.is_base_game_pack = true;
    stable(&doc("1.26.50", &payload("minecraft:my_mob", "")), &base);
}

/// and: a payload without `description` is skipped. In a pack it is an
/// error; vanilla skips it silently, and its own base-game packs ship two such partial files
/// (`vanilla_1.16.100` and `vanilla_1.17.20` `entities/npc.json`), so a base-game pack gets a
/// warning.
#[test]
fn a_payload_without_description_is_an_information_in_a_base_game_pack() {
    // The game finds no identifier and returns without a log: in a base-game pack the
    // skip is told as an information.
    let text = r#"{ "components": { "minecraft:conditional_bandwidth_optimization": {} } }"#;
    assert_eq!(rejected("1.13.0", text, &cx()), ["core/missing_identifier"]);
    let mut base = cx();
    base.is_base_game_pack = true;
    let outcome = outcome(&doc("1.13.0", text), &base);
    assert!(outcome.value.is_none(), "the entity is skipped");
    assert!(
        codes(&outcome.diagnostics, Severity::Error).is_empty()
            && codes(&outcome.diagnostics, Severity::Warning).is_empty(),
        "{:#?}",
        outcome.diagnostics
    );
    assert_eq!(
        codes(&outcome.diagnostics, Severity::Info),
        ["core/missing_identifier"]
    );
}

#[test]
fn a_leading_digit_warns_below_1_20_30_and_is_rejected_from_it() {
    let (_, found) = stable(&doc("1.20.20", &payload("x:1abc", "")), &cx());
    assert_eq!(codes(&found, Severity::Info), ["core/invalid_value"]);
    assert_eq!(
        rejected("1.20.30", &payload("x:1abc", ""), &cx()),
        ["core/invalid_value"]
    );
}

#[test]
fn an_identifier_without_namespace_is_rejected() {
    assert_eq!(
        rejected("1.26.50", &payload("nonamespace", ""), &cx()),
        ["core/namespace_rule"]
    );
}

#[test]
fn aliases_below_1_21_10_move_to_the_side_car_with_a_warning() {
    let payload = r#"{ "description": { "identifier": "demo:x", "aliases": { "demo:y": {} } }, "components": {} }"#;
    let (entity, found) = stable(&doc("1.21.0", payload), &cx());
    assert_eq!(codes(&found, Severity::Info), ["core/deprecated_member"]);
    let side_car = entity.deprecated.as_ref().expect("the side-car");
    assert_eq!(side_car.aliases["demo:y"], json("{}"));
    assert_eq!(super::common::reparse(&entity), entity);
}

#[test]
fn aliases_from_1_21_10_are_dropped_with_vanillas_text() {
    let payload = r#"{ "description": { "identifier": "demo:x", "aliases": { "demo:y": {} } }, "components": {} }"#;
    let (entity, found) = stable(&doc("1.21.10", payload), &cx());
    assert!(entity.deprecated.is_none());
    let deprecated = with_code(&found, "entity/aliases_deprecated");
    assert_eq!(
        deprecated[0].code,
        Code::from(&entity_codes::ALIASES_DEPRECATED)
    );
}

fn filtered_event() -> String {
    payload(
        "demo:x",
        r#""events": { "demo:go": {
            "filters": { "test": "is_family", "value": "pig" },
            "sequence": [ { "filters": { "test": "is_family", "value": "cow" }, "add": { "component_groups": [] } } ]
        } }"#,
    )
}

#[test]
fn a_root_event_filter_below_1_19_20_is_dropped() {
    let (entity, found) = stable(&doc("1.19.10", &filtered_event()), &cx());
    let node = entity.events["demo:go"].node.as_object().unwrap();
    assert!(node.get("filters").is_none());
    // The game drops it without a line: an information of ours.
    assert!(codes(&found, Severity::Warning).is_empty(), "{found:#?}");
    assert_eq!(codes(&found, Severity::Info), ["core/dropped_field"]);
    // The text is ours: none is attached.
    assert_eq!(
        found.iter().next().unwrap().code,
        Code::from(&core_codes::DROPPED_FIELD)
    );
    let nested = node.get("sequence").unwrap().as_array().unwrap()[0]
        .as_object()
        .unwrap();
    assert!(nested.get("filters").is_some(), "nested filters are kept");
}

#[test]
fn a_root_event_filter_from_1_19_20_is_kept() {
    let (entity, found) = stable(&doc("1.19.20", &filtered_event()), &cx());
    let node = entity.events["demo:go"].node.as_object().unwrap();
    assert!(node.get("filters").is_some());
    assert!(codes(&found, Severity::Warning).is_empty(), "{found:#?}");
}

fn versioned(version_json: &str, upgrades: &str) -> String {
    format!(
        r#"{{ "description": {{ "identifier": "x:golem", "entity_version": {version_json} }}, "components": {{}}, "upgrades": {upgrades} }}"#
    )
}

const HANDLERS: &str = r#"{ "x:v1": { "add": { "component_groups": [] } }, "x:v2": {} }"#;

#[test]
fn entity_version_loads_at_1_26_30() {
    let text = doc(
        "1.26.30",
        &versioned(
            r#"{ "upgrades": [ { "upgrade_to": 1, "name": "x:v1" }, { "upgrade_to": 2, "name": "x:v2" } ] }"#,
            HANDLERS,
        ),
    );
    let (entity, found) = stable(&text, &cx());
    assert!(found.is_empty(), "{found:#?}");
    let version = entity.description.entity_version.as_ref().unwrap();
    assert_eq!(version.upgrades.len(), 2);
    assert_eq!(entity.upgrades.len(), 2);
    assert_eq!(super::common::reparse(&entity), entity);
}

#[test]
fn entity_version_at_1_26_20_without_upcoming_creator_features_loads_with_a_note() {
    let text = doc(
        "1.26.20",
        &versioned(
            r#"{ "upgrades": [ { "upgrade_to": 1, "name": "x:v1" } ] }"#,
            HANDLERS,
        ),
    );
    let (entity, found) = stable(&text, &cx());
    // 1.26.60 has no such gate: the changelog's toggle is told as an
    // information of the crate's own, and the version and its handlers load.
    assert!(codes(&found, Severity::Warning).is_empty(), "{found:#?}");
    assert_eq!(
        codes(&found, Severity::Info),
        ["core/turn_on_toggle_before"]
    );
    assert_eq!(
        found.iter().next().unwrap().code,
        core_codes::TURN_ON_TOGGLE_BEFORE.with([
            "member".to_string(),
            "entity_version".to_string(),
            bedrock_addon_core::version::FormatVersion::new(1, 26, 30).to_string(),
            "upcoming_creator_features".to_string()
        ])
    );
    assert!(entity.description.entity_version.is_some());
    assert_eq!(entity.upgrades.len(), 2);
    let mut ucf = cx();
    ucf.upcoming_creator_features = true;
    let (entity, found) = stable(&text, &ucf);
    assert!(found.is_empty(), "{found:#?}");
    assert!(entity.description.entity_version.is_some());
}

#[test]
fn entity_version_below_1_26_20_is_an_unknown_member() {
    let text = |version: &str| {
        doc(
            version,
            &versioned(
                r#"{ "upgrades": [ { "upgrade_to": 1, "name": "x:v1" } ] }"#,
                HANDLERS,
            ),
        )
    };
    // From 1.21.89 a strict slice refuses both; below, they are told and not kept.
    let refused = outcome(&text("1.26.10"), &cx());
    assert!(refused.value.is_none());
    assert_eq!(
        codes(&refused.diagnostics, Severity::Error),
        ["core/unrecognized_field", "core/unrecognized_field"]
    );
    let (entity, found) = stable(&text("1.21.80"), &cx());
    assert_eq!(
        codes(&found, Severity::Warning),
        ["core/unrecognized_field", "core/unrecognized_field"]
    );
    assert!(entity.description.entity_version.is_none());
    let payload = bedrock_addon_core::diagnostic::JsonPath::of_keys(["minecraft:entity"]);
    assert!(
        found
            .unknown()
            .get(&payload.child("description"), "entity_version")
            .is_none()
    );
    assert!(found.unknown().get(&payload, "upgrades").is_none());
}

/// One structure check: `entity_version` as written gives `text`, and both members are
/// dropped. The game's checks are kept errors (its level-3 lines, the entity loads
/// versionless); the handler lookup's text is ours, an information.
fn structure(id: &str, version_json: &str, line: &str) {
    let (entity, found) = stable(&doc("1.26.30", &versioned(version_json, HANDLERS)), &cx());
    let invalid = with_code(&found, id);
    assert_eq!(invalid.len(), 1, "{version_json}: {found:#?}");
    if line.starts_with("upgrade handler ") {
        assert_eq!(invalid[0].severity, Severity::Info, "{version_json}");
    } else {
        assert_eq!(
            (invalid[0].severity, invalid[0].kept),
            (Severity::Error, true),
            "{version_json}"
        );
    }
    assert!(
        invalid[0].code.id().starts_with("entity/")
            && bedrock_addon_core::diagnostic::testing::same_line(&invalid[0].code, id, line),
        "{version_json}: {:?}",
        invalid[0].code
    );
    assert!(entity.description.entity_version.is_none());
    assert!(entity.upgrades.is_empty());
}

#[test]
fn every_vanilla_structure_message() {
    structure(
        "entity/version_not_object",
        "[]",
        "Error loading 'entity_version' as it is not a JSON object.",
    );
    structure(
        "entity/upgrades_not_array",
        r#"{ "upgrades": {} }"#,
        "Error loading 'entity_version' as 'upgrades' is not an array.",
    );
    structure(
        "entity/upgrade_entry_not_object",
        r#"{ "upgrades": [ 1 ] }"#,
        "Error loading 'entity_version' as 'upgrades' contains an entry that is not a JSON object.",
    );
    structure(
        "entity/no_integer_upgrade_to",
        r#"{ "upgrades": [ { "upgrade_to": "1", "name": "x:v1" } ] }"#,
        "Error loading 'entity_version' as an upgrade entry does not contain an integer 'upgrade_to'.",
    );
    structure(
        "entity/no_string_name",
        r#"{ "upgrades": [ { "upgrade_to": 1 } ] }"#,
        "Error loading 'entity_version' as an upgrade entry does not contain a string 'name'.",
    );
    structure(
        "entity/empty_name",
        r#"{ "upgrades": [ { "upgrade_to": 1, "name": "" } ] }"#,
        "Error loading 'entity_version' as an upgrade entry's 'name' is empty.",
    );
    structure(
        "entity/upgrade_to_zero",
        r#"{ "upgrades": [ { "upgrade_to": 0, "name": "x:v1" } ] }"#,
        "Error loading 'entity_version' as an upgrade entry 'upgrade_to' version is 0, which is reserved for versionless entities.",
    );
    structure(
        "entity/version_not_from_one",
        r#"{ "upgrades": [ { "upgrade_to": 2, "name": "x:v2" } ] }"#,
        "Error loading 'entity_version' as its version does not start at 1.",
    );
    structure(
        "entity/version_duplicate",
        r#"{ "upgrades": [ { "upgrade_to": 1, "name": "x:v1" }, { "upgrade_to": 1, "name": "x:v2" } ] }"#,
        "Error loading 'entity_version' as it contains multiple upgrade entries for version 1.",
    );
    structure(
        "entity/version_gap",
        r#"{ "upgrades": [ { "upgrade_to": 1, "name": "x:v1" }, { "upgrade_to": 3, "name": "x:v2" } ] }"#,
        "Error loading 'x:golem' as it contains a non-contiguous version between 1 and 3.",
    );
    structure(
        "entity/upgrade_handler_undefined",
        r#"{ "upgrades": [ { "upgrade_to": 1, "name": "x:missing" } ] }"#,
        "upgrade handler 'x:missing' is not defined in 'upgrades'",
    );
}

const BRAIN: &str = r#""brain": { "memories": [
    { "name": "x:target", "type": "entity_ref", "persists": true },
    { "name": "x:has_nectar", "type": "bool" },
    { "name": "x:target", "type": "int" }
], "internal_memory_access_mappings": { "beehive_has_nectar": "x:has_nectar" } }"#;

#[test]
fn brain_without_the_experiment_rejects_the_entity() {
    assert_eq!(
        rejected("1.26.50", &payload("x:bee", BRAIN), &cx()),
        ["core/turn_on_toggle"]
    );
    let outcome = outcome(&doc("1.26.50", &payload("x:bee", BRAIN)), &cx());
    let found = with_code(&outcome.diagnostics, "core/turn_on_toggle");
    assert_eq!(
        found[0].to_string().rsplit(" | ").next(),
        Some("To use member 'brain', turn on the 'brain_prototype' toggle in settings")
    );
}

#[test]
fn brain_with_the_experiment_keeps_two_memories_of_one_name_and_two_types() {
    let mut brain = cx();
    brain.experiments.insert(Experiment::BrainPrototype);
    let (entity, found) = stable(&doc("1.26.50", &payload("x:bee", BRAIN)), &brain);
    let memories = &entity.brain.as_ref().unwrap().memories;
    // The game's memory set takes the name and the type as the key: an
    // `entity_ref` and an `int` of one name are two memories, both kept, nothing said.
    assert_eq!(memories.len(), 3);
    let kinds: Vec<_> = memories
        .iter()
        .filter(|m| m.name == "x:target")
        .map(|m| m.kind)
        .collect();
    assert_eq!(
        kinds,
        [
            bedrock_addon_family_entity::MemoryType::EntityRef,
            bedrock_addon_family_entity::MemoryType::Int
        ]
    );
    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn brain_keeps_the_first_of_two_memories_of_one_name_and_type() {
    let mut brain = cx();
    brain.experiments.insert(Experiment::BrainPrototype);
    let text = payload(
        "x:bee",
        r#""brain": { "memories": [ { "name": "x:n", "type": "int", "persists": true }, { "name": "x:n", "type": "int" } ] }"#,
    );
    let (entity, found) = stable(&doc("1.26.50", &text), &brain);
    let memories = &entity.brain.as_ref().unwrap().memories;
    // The game keeps the first, in silence: the
    // later entry is dropped with an information of the crate's own.
    assert_eq!(memories.len(), 1);
    assert_eq!(memories[0].persists, Some(true));
    assert_eq!(codes(&found, Severity::Info), ["core/invalid_value"]);
    assert_eq!(
        found.iter().next().unwrap().location.json_path.to_string(),
        "minecraft:entity/brain/memories/1"
    );
}

#[test]
fn brain_beehive_mapping_must_name_a_bool_memory() {
    let mut brain = cx();
    brain.experiments.insert(Experiment::BrainPrototype);
    let text = payload(
        "x:bee",
        r#""brain": { "memories": [ { "name": "x:n", "type": "int" } ], "internal_memory_access_mappings": { "beehive_has_nectar": "x:n" } }"#,
    );
    let (_, found) = stable(&doc("1.26.50", &text), &brain);
    // No line of the game: an information of the crate's own.
    assert_eq!(codes(&found, Severity::Info), ["core/unresolved_reference"]);
}

#[test]
fn brain_below_1_21_130_is_an_unknown_member() {
    // From 1.21.89 a strict slice refuses it; below, it is told and not kept.
    let refused = outcome(&doc("1.21.120", &payload("x:bee", BRAIN)), &cx());
    assert!(refused.value.is_none());
    assert_eq!(
        codes(&refused.diagnostics, Severity::Error),
        ["core/unrecognized_field"]
    );
    let (entity, found) = stable(&doc("1.21.80", &payload("x:bee", BRAIN)), &cx());
    assert!(entity.brain.is_none());
    assert_eq!(
        codes(&found, Severity::Warning),
        ["core/unrecognized_field"]
    );
    let payload = bedrock_addon_core::diagnostic::JsonPath::of_keys(["minecraft:entity"]);
    assert!(found.unknown().get(&payload, "brain").is_none());
}

#[test]
fn an_event_naming_an_undefined_group_is_reported_and_kept() {
    let text = payload(
        "demo:x",
        r#""component_groups": { "demo:a": {} }, "events": { "demo:go": { "add": { "component_groups": ["demo:a", "demo:b"] } } }"#,
    );
    let (entity, found) = stable(&doc("1.26.50", &text), &cx());
    let unresolved = with_code(&found, "core/unresolved_reference");
    assert_eq!(unresolved.len(), 1);
    // The game stores the names and says nothing at load: the crate's own, an information.
    assert_eq!(unresolved[0].severity, Severity::Info);
    assert_eq!(
        unresolved[0].location.json_path.to_string(),
        "minecraft:entity/events/demo:go/add/component_groups/1"
    );
    assert_eq!(
        entity.events["demo:go"].node,
        json(r#"{ "add": { "component_groups": ["demo:a", "demo:b"] } }"#)
    );
}

#[test]
fn an_event_with_randomize_and_sequence_warns() {
    let text = payload(
        "demo:x",
        r#""events": { "demo:go": { "randomize": [ { "weight": 1 } ], "sequence": [ {} ] } }"#,
    );
    let (_, found) = stable(&doc("1.26.50", &text), &cx());
    assert_eq!(
        codes(&found, Severity::Warning),
        ["shapes/event_both_randomize_and_sequence"]
    );
}

/// Today's behaviour, not capture-checked: the game's text says the sequence is ignored, yet
/// both lists are validated (no content log of an entity event with both lists exists).
#[test]
fn an_event_with_randomize_and_sequence_still_validates_both_lists() {
    let text = payload(
        "demo:x",
        r#""events": { "demo:go": { "randomize": [ { "weight": 1, "add": { "component_groups": ["demo:r"] } } ], "sequence": [ { "add": { "component_groups": ["demo:s"] } } ] } }"#,
    );
    let (_, found) = stable(&doc("1.26.50", &text), &cx());
    let ignored = with_code(&found, "shapes/event_both_randomize_and_sequence");
    assert_eq!(ignored.len(), 1);
    assert_eq!(ignored[0].severity, Severity::Warning);
    assert_eq!(
        ignored[0].phase,
        bedrock_addon_core::diagnostic::Phase::Validate
    );
    assert!(
        bedrock_addon_core::diagnostic::testing::same_line(
            &ignored[0].code,
            "shapes/event_both_randomize_and_sequence",
            bedrock_addon_core::diagnostic::EVENT_BOTH_RANDOMIZE_AND_SEQUENCE
        ),
        "{:?}",
        ignored[0].code
    );
    assert_eq!(
        ignored[0].location.json_path.to_string(),
        "minecraft:entity/events/demo:go/sequence"
    );
    // The ignored sequence's group reference is still checked, after the randomize's (the
    // document's order).
    let unresolved: Vec<String> = with_code(&found, "core/unresolved_reference")
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        unresolved,
        [
            "minecraft:entity/events/demo:go/randomize/0/add/component_groups/0",
            "minecraft:entity/events/demo:go/sequence/0/add/component_groups/0",
        ]
    );
}

#[test]
fn an_unknown_or_item_response_is_an_unknown_member() {
    let text = payload(
        "demo:x",
        r#""events": { "demo:go": { "damage": { "amount": 1 }, "demo_thing": 1, "queue_command": { "command": "say hi" } } }"#,
    );
    let (_, found) = stable(&doc("1.26.50", &text), &cx());
    assert_eq!(
        codes(&found, Severity::Warning),
        ["core/child_not_valid", "core/child_not_valid"]
    );
}

#[test]
fn event_response_gates() {
    let event = |response: &str| {
        payload(
            "demo:x",
            &format!(r#""events": {{ "demo:go": {{ {response} }} }}"#),
        )
    };
    // `run_command` needs the legacy creator features; `set_memory` the Brain Prototype.
    let (_, found) = stable(
        &doc(
            "1.26.50",
            &event(r#""run_command": { "command": "say hi" }"#),
        ),
        &cx(),
    );
    assert_eq!(codes(&found, Severity::Info), ["core/missing_toggles"]);
    let (_, found) = stable(&doc("1.26.50", &event(r#""set_memory": {}"#)), &cx());
    assert_eq!(codes(&found, Severity::Info), ["core/turn_on_toggle"]);
    // `unleash` arrived in 1.26.30.
    let (_, found) = stable(
        &doc("1.26.20", &event(r#""unleash": { "unleash_self": true }"#)),
        &cx(),
    );
    assert_eq!(codes(&found, Severity::Warning), ["core/child_not_valid"]);
    let (_, found) = stable(
        &doc("1.26.30", &event(r#""unleash": { "unleash_self": true }"#)),
        &cx(),
    );
    assert!(found.is_empty(), "{found:#?}");
    // `execute_event_on_home_block` is experimental below 1.26.30: Upcoming Creator
    // Features, which a base-game pack does not need (vanilla's own `creaking.json` uses it).
    let home = event(r#""execute_event_on_home_block": { "event": "demo:e" }"#);
    let (_, found) = stable(&doc("1.26.20", &home), &cx());
    // The crate's own: an information.
    assert!(codes(&found, Severity::Warning).is_empty(), "{found:#?}");
    assert_eq!(codes(&found, Severity::Info), ["core/turn_on_toggle"]);
    let mut ucf = cx();
    ucf.upcoming_creator_features = true;
    let mut base = cx();
    base.is_base_game_pack = true;
    for (version, cx) in [("1.26.20", &ucf), ("1.26.20", &base), ("1.26.30", &cx())] {
        let (_, found) = stable(&doc(version, &home), cx);
        assert!(found.is_empty(), "{version}: {found:#?}");
    }
}

#[test]
fn an_event_filter_with_numbered_subject_is_written_with_names() {
    let text = payload(
        "demo:x",
        r#""events": { "demo:go": { "sequence": [ { "filters": { "test": "has_component", "subject": 1, "operator": 1, "value": "minecraft:is_baby" } } ] } }"#,
    );
    let (entity, found) = stable(&doc("1.26.50", &text), &cx());
    assert!(found.is_empty(), "{found:#?}");
    assert_eq!(
        entity.events["demo:go"].node,
        json(
            r#"{ "sequence": [ { "filters": { "test": "has_component", "subject": "other", "operator": "!=", "value": "minecraft:is_baby" } } ] }"#
        )
    );
}

#[test]
fn an_unknown_filter_test_in_an_event_warns() {
    let text = payload(
        "demo:x",
        r#""events": { "demo:go": { "trigger": { "event": "demo:y", "filters": { "test": "no_such_test" } } } }"#,
    );
    let (_, found) = stable(&doc("1.26.50", &text), &cx());
    assert_eq!(
        codes(&found, Severity::Warning),
        ["core/filter_test_not_found"]
    );
}

#[test]
fn spawn_egg_interaction_without_offspring_is_kept_with_the_game_line() {
    let text = r#"{ "description": { "identifier": "demo:x" }, "components": { "minecraft:spawn_egg_interaction": {} } }"#;
    let (entity, found) = stable(&doc("1.26.50", text), &cx());
    // The game logs at `initialize` and keeps the definition: so does the crate.
    assert_eq!(entity.components.len(), 1, "{found:#?}");
    assert!(
        with_code(&found, "DroppedComponent").is_empty(),
        "{found:#?}"
    );
    let told = with_code(&found, "entity/spawn_egg_needs_offspring");
    assert_eq!(told.len(), 1, "{found:#?}");
    assert_eq!(
        told[0].code,
        Code::from(&entity_codes::SPAWN_EGG_NEEDS_OFFSPRING)
    );
    assert_eq!(
        told[0].location.json_path.to_string(),
        "minecraft:entity/components/minecraft:spawn_egg_interaction"
    );
    // The game's level-3 line, and the entity loads: a kept error.
    assert_eq!((told[0].severity, told[0].kept), (Severity::Error, true));
    let with_offspring = r#"{ "description": { "identifier": "demo:x" }, "components": { "minecraft:spawn_egg_interaction": {} },
        "component_groups": { "g": { "minecraft:offspring": {} } } }"#;
    let (entity, found) = stable(&doc("1.26.50", with_offspring), &cx());
    assert_eq!(entity.components.len(), 1);
    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn a_null_group_from_1_21_90_is_dropped() {
    let text = r#"{ "description": { "identifier": "demo:x" }, "components": {}, "component_groups": { "g": null, "h": {} } }"#;
    let (entity, found) = stable(&doc("1.26.50", text), &cx());
    // A `null` member is absent, so the group
    // is not there, without a line of the game's: an information of the crate's own.
    assert!(codes(&found, Severity::Warning).is_empty(), "{found:#?}");
    assert_eq!(codes(&found, Severity::Info), ["core/invalid_value"]);
    assert_eq!(entity.component_groups.len(), 1);
}

#[test]
fn property_default_expressions() {
    let text = payload(
        "demo:x",
        "",
    )
    .replace(
        r#""identifier": "demo:x" }"#,
        r#""identifier": "demo:x", "properties": {
            "demo:size": { "type": "float", "range": [0, 1], "default": "query.had_component_group('baby') ? 1 : 0" },
            "demo:r": { "type": "float", "range": [0, 1], "default": "math.random(0, 1)" },
            "demo:mood": { "type": "enum", "values": ["a", "b"], "default": "a" }
        } }"#,
    );
    let (entity, found) = stable(&doc("1.26.50", &text), &cx());
    assert!(codes(&found, Severity::Warning).is_empty(), "{found:#?}");
    assert_eq!(
        entity.properties()["demo:mood"].default,
        Some(PropertyDefault::Enum("a".to_owned()))
    );
    assert!(matches!(
        entity.properties()["demo:size"].default,
        Some(PropertyDefault::Expression(_))
    ));
    let _ = Loaded::<Entity>::Stable(entity);
}

/// The component keys of `entity`, in order.
fn component_keys(entity: &Entity) -> Vec<String> {
    entity
        .components
        .iter()
        .map(|entry| match entry {
            ComponentRef::Typed { name, .. }
            | ComponentRef::Dynamic { name, .. }
            | ComponentRef::Unknown { name, .. } => name.to_owned(),
            ComponentRef::Custom(custom) => custom.name.clone(),
            _ => String::new(),
        })
        .collect()
}

/// A generic memory goal under each key of `keys`, in a document of `version`.
fn generic_goals(version: &str, keys: &[&str]) -> String {
    let goals: Vec<String> = keys
        .iter()
        .enumerate()
        .map(|(i, key)| format!(r#""{key}": {{"priority": {i}}}"#))
        .collect();
    doc(
        version,
        &format!(
            r#"{{"description": {{"identifier": "x:a"}}, "components": {{{}}}}}"#,
            goals.join(", ")
        ),
    )
}

const MOVE_TO: &str = "minecraft:memory_behavior.move_to_position";

/// From 1.26.20 a generic memory goal without a suffix, or with
/// one outside `[A-Za-z0-9_]{1,32}`, is `GenericGoalSuffixMissing` (an information of the crate's own: the game logs no line, the component is dropped and the
/// entity loads) and dropped; each named goal
/// is its own entry under its written key, and writes back under it.
#[test]
fn each_suffixed_generic_goal_is_its_own_entry() {
    let long = format!("{MOVE_TO}.{}", "a".repeat(33));
    let gold = format!("{MOVE_TO}.gold");
    let iron = format!("{MOVE_TO}.iron_2");
    let outcome = outcome(
        &generic_goals("1.26.20", &[MOVE_TO, &long, &gold, &iron]),
        &cx(),
    );
    let missing: Vec<String> = outcome
        .diagnostics
        .iter()
        .filter(|d| d.code == Code::from(&core_codes::GENERIC_GOAL_SUFFIX_MISSING))
        .inspect(|d| assert_eq!(d.severity, Severity::Info))
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        missing,
        [
            format!("minecraft:entity/components/{MOVE_TO}"),
            format!("minecraft:entity/components/{long}"),
        ]
    );
    let Some(Loaded::Stable(entity)) = outcome.value else {
        panic!("the entity loads: {:#?}", outcome.diagnostics);
    };
    // The 1.26.50 step adds `uses_legacy_ambient_sounds` behind them.
    let ambient = "minecraft:uses_legacy_ambient_sounds".to_owned();
    assert_eq!(
        component_keys(&entity),
        [gold.clone(), iron.clone(), ambient.clone()]
    );
    assert_eq!(component_keys(&reparse(&entity)), [gold, iron, ambient]);
}

/// below 1.26.20 the bare key is a legacy key and gets the suffix `.legacy` on
/// upgrade (ours).
#[test]
fn a_bare_generic_goal_below_1_26_20_gets_the_legacy_suffix_deviation() {
    let (entity, found) = stable(&generic_goals("1.26.10", &[MOVE_TO]), &cx());
    assert!(codes(&found, Severity::Error).is_empty(), "{found:#?}");
    // Section A of the 1.26.20 step adds `uses_legacy_friction` after it.
    assert_eq!(
        component_keys(&entity),
        [
            format!("{MOVE_TO}.legacy"),
            "minecraft:uses_legacy_friction".to_owned(),
            // The 1.26.50 step's.
            "minecraft:uses_legacy_ambient_sounds".to_owned(),
        ]
    );
}

/// A document of `version` whose `components` are `components` (JSON members, without braces).
fn with_components(version: &str, components: &str) -> String {
    doc(
        version,
        &format!(r#"{{"description": {{"identifier": "x:a"}}, "components": {{{components}}}}}"#),
    )
}

/// two versions of one component in a section (`looked_at` supersedes `lookat`):
/// `DuplicateComponentVersion`, and the superseded one is dropped.
#[test]
fn two_versions_of_one_component_keep_the_newer() {
    // in the legacy slice; from 1.21.89 the old key alone rejects the entity
    // (strict).
    let (entity, found) = stable(
        &with_components(
            "1.21.50",
            r#""minecraft:lookat": {}, "minecraft:looked_at": {}"#,
        ),
        &cx(),
    );
    assert!(
        with_code(&found, "core/duplicate_component_version").len() == 1,
        "{found:#?}"
    );
    let keys = component_keys(&entity);
    assert!(keys.contains(&"minecraft:looked_at".to_owned()), "{keys:?}");
    assert!(!keys.contains(&"minecraft:lookat".to_owned()), "{keys:?}");
}

/// (`requires`, `deprecated_since`): `bounciness` before its 1.26.30 release
/// needs the Beta APIs toggle (`MissingToggles`); `enderman_take_block` from 1.21.100 is
/// `DeprecatedMember` and dropped.
#[test]
fn generic_component_metadata() {
    let text = with_components("1.26.20", r#""minecraft:bounciness": {}"#);
    let gated = outcome(&text, &cx());
    assert_eq!(
        with_code(&gated.diagnostics, "core/turn_on_toggle_before").len(),
        1,
        "{:#?}",
        gated.diagnostics
    );
    let mut beta = cx();
    beta.beta_apis = true;
    let (_, found) = stable(&text, &beta);
    assert!(
        with_code(&found, "core/turn_on_toggle_before").is_empty(),
        "{found:#?}"
    );

    // from its deprecation the component is no member of the schema: an
    // error in the strict slice of 1.21.100 that rejects the entity.
    let refused = outcome(
        &with_components(
            "1.21.100",
            r#""minecraft:behavior.enderman_take_block": {"priority": 1}"#,
        ),
        &cx(),
    );
    assert!(refused.value.is_none(), "{:#?}", refused.diagnostics);
    let unknown = with_code(&refused.diagnostics, "core/component_not_in_schema");
    assert_eq!(unknown.len(), 1, "{:#?}", refused.diagnostics);
    assert_eq!(unknown[0].severity, Severity::Error);
    assert!(with_code(&refused.diagnostics, "DeprecatedMember").is_empty());
}

/// with the goal's member gate: the step's `stop_if_holding_item: false` in a
/// document below 1.26.30 is the default the gate would leave, so it raises no finding; a value
/// the author wrote there is `UnrecognizedField`.
#[test]
fn e32_the_stop_if_holding_item_insert_is_not_reported_by_the_gate() {
    let goal = |members: &str| {
        with_components(
            "1.26.20",
            &format!(r#""minecraft:behavior.pickup_items": {{"priority": 1{members}}}"#),
        )
    };
    let (_, found) = stable(&goal(""), &cx());
    assert!(found.is_empty(), "{found:#?}");
    // The file's own `true` is a member the 1.26.20 struct lacks: the strict slice refuses
    // it.
    let refused = outcome(&goal(r#", "stop_if_holding_item": true"#), &cx());
    assert!(refused.value.is_none());
    assert_eq!(
        codes(&refused.diagnostics, Severity::Error),
        ["core/unrecognized_field"]
    );
}

/// The Brain memory goals and sensors need the Brain Prototype
/// experiment; without it each is `MissingToggles` and dropped, the entity loads. The generic
/// `memory_behavior.move_to_position.<name>` is not gated.
#[test]
fn brain_components_need_the_brain_prototype() {
    let text = with_components(
        "1.26.50",
        r#""minecraft:memory_behavior.wait": {"duration": {"min": 1, "max": 2}},
           "minecraft:memory_behavior.move_to_position.home": {"priority": 1}"#,
    );
    let (entity, found) = stable(&text, &cx());
    assert_eq!(
        codes(&found, Severity::Info),
        ["core/turn_on_toggle"],
        "{found:#?}"
    );
    assert_eq!(
        component_keys(&entity),
        ["minecraft:memory_behavior.move_to_position.home"]
    );
    let mut brain = cx();
    brain.experiments.insert(Experiment::BrainPrototype);
    let (entity, found) = stable(&text, &brain);
    assert!(found.is_empty(), "{found:#?}");
    assert_eq!(component_keys(&entity).len(), 2);
}

/// `deprecated_since`: each superseded legacy key, alone in a
/// document at its deprecation version, is `DeprecatedMember` and dropped.
#[test]
fn each_superseded_legacy_key_is_deprecated_from_its_version() {
    for (key, version) in [
        ("minecraft:scaffolding_climber", "1.17.10"),
        ("minecraft:foot_size", "1.16.100"),
        ("minecraft:lookat", "1.21.40"),
        ("minecraft:body_rotation_axis_aligned", "1.21.100"),
        ("minecraft:input_air_controlled", "1.21.110"),
        ("minecraft:dash", "1.21.110"),
        ("minecraft:pushable", "1.26.10"),
        ("minecraft:behavior.follow_target_captain", "1.26.20"),
    ] {
        let mut no_upgrade = cx();
        no_upgrade.is_base_game_pack = false;
        let text = doc(
            version,
            &format!(
                r#"{{"description": {{"identifier": "x:a"}}, "do_not_upgrade": true, "components": {{"{key}": {{}}}}}}"#
            ),
        );
        // the key is no member of the schema at its version: at the
        // component, a warning that drops it in the legacy slice (below 1.21.89), an error that
        // rejects the entity in a strict one.
        let loaded = outcome(&text, &no_upgrade);
        let unknown = with_code(&loaded.diagnostics, "core/component_not_in_schema");
        assert_eq!(unknown.len(), 1, "{key}: {:#?}", loaded.diagnostics);
        assert_eq!(
            unknown[0].location.json_path.to_string(),
            format!("minecraft:entity/components/{key}")
        );
        assert_eq!(
            unknown[0].code.to_string(),
            "this component was found in the input, but is not present in the Schema"
        );
        assert!(
            with_code(&loaded.diagnostics, "DeprecatedMember").is_empty(),
            "{key}: {:#?}",
            loaded.diagnostics
        );
        let strict = !matches!(version, "1.17.10" | "1.16.100" | "1.21.40");
        if strict {
            assert_eq!(unknown[0].severity, Severity::Error, "{key}");
            assert!(loaded.value.is_none(), "{key} rejects the entity");
        } else {
            assert_eq!(unknown[0].severity, Severity::Warning, "{key}");
            let (entity, _) = stable(&text, &no_upgrade);
            assert!(
                !component_keys(&entity).contains(&key.to_owned()),
                "{key} is dropped"
            );
        }
    }
}

/// one bad property (a bad `type`, a bad `range`, a non-scalar `default` or
/// an `int` default outside `i32`) drops the whole property group with "Error loading Actor
/// Properties"; the entity loads, so every finding is a warning.
#[test]
fn a_bad_property_drops_the_whole_property_group() {
    for bad in [
        r#"{"type": "color", "default": 1}"#,
        r#"{"type": "int", "default": 1, "range": [0]}"#,
        r#"{"type": "int", "default": [1]}"#,
        r#"{"type": "int", "default": 3000000000}"#,
    ] {
        let text = doc(
            "1.26.50",
            &format!(
                r#"{{"description": {{"identifier": "x:a", "properties": {{"x:good": {{"type": "bool", "default": true}}, "x:bad": {bad}}}}}, "components": {{}}}}"#
            ),
        );
        let (entity, found) = stable(&text, &cx());
        // Nothing refuses the entity: the group's line is a kept error.
        assert!(!found.has_errors(), "{bad}: {found:#?}");
        assert!(
            entity.properties().is_empty(),
            "{bad}: the group is dropped"
        );
        let dropped = with_code(&found, "entity/actor_properties_not_loaded");
        assert_eq!(dropped.len(), 1, "{bad}: {found:#?}");
        assert_eq!(
            dropped[0].code,
            Code::from(&entity_codes::ACTOR_PROPERTIES_NOT_LOADED)
        );
        assert_eq!(
            (dropped[0].severity, dropped[0].kept),
            (Severity::Error, true)
        );
    }
}

/// `property_memory_mapping` needs the Brain Prototype experiment
/// Without it `MissingToggles` and the member is dropped, the entity loads.
#[test]
fn property_memory_mapping_needs_the_brain_prototype() {
    let text = doc(
        "1.26.50",
        r#"{"description": {"identifier": "x:a", "property_memory_mapping": {"x:p": "x:m"}}, "components": {}}"#,
    );
    let (entity, found) = stable(&text, &cx());
    assert_eq!(
        found.iter().map(|d| d.code.clone()).collect::<Vec<_>>(),
        [
            core_codes::TURN_ON_TOGGLE.with([
                "member",
                "property_memory_mapping",
                "brain_prototype"
            ])
        ]
    );
    assert!(entity.description.property_memory_mapping.is_none());
    let mut brain = cx();
    brain.experiments.insert(Experiment::BrainPrototype);
    let (entity, found) = stable(&text, &brain);
    assert!(found.is_empty(), "{found:#?}");
    assert!(entity.description.property_memory_mapping.is_some());
}

/// On the newest-version path, at 1.26.50 a payload without `description` fails the
/// required member (`MissingRequiredField`); in a base-game pack it is a warning
/// and the file is skipped.
#[test]
fn a_newest_payload_without_description_only_warns_in_a_base_game_pack() {
    let text = r#"{ "components": {} }"#;
    assert_eq!(
        rejected("1.26.50", text, &cx()),
        ["core/missing_required_field"]
    );
    let mut base = cx();
    base.is_base_game_pack = true;
    let outcome = outcome(&doc("1.26.50", text), &base);
    assert!(outcome.value.is_none(), "the entity is skipped");
    assert!(
        codes(&outcome.diagnostics, Severity::Error).is_empty(),
        "{:#?}",
        outcome.diagnostics
    );
    assert_eq!(
        codes(&outcome.diagnostics, Severity::Info),
        ["core/missing_required_field"]
    );
}

/// grammar, `randomize:= [node & { weight: number }]`: a weight of
/// another JSON type is `InvalidType`.
#[test]
fn a_randomize_weight_must_be_a_number() {
    let text = |weight: &str| {
        doc(
            "1.26.50",
            &payload(
                "x:a",
                &format!(
                    r#""events": {{ "x:go": {{ "randomize": [ {{ "weight": {weight} }} ] }} }}"#
                ),
            ),
        )
    };
    let (_, found) = stable(&text("2"), &cx());
    assert!(found.is_empty(), "{found:#?}");
    let (_, found) = stable(&text(r#""2""#), &cx());
    assert_eq!(codes(&found, Severity::Info), ["core/invalid_type"]);
    assert_eq!(
        found.iter().next().unwrap().location.json_path.to_string(),
        "minecraft:entity/events/x:go/randomize/0/weight"
    );
}

/// The lenient conversion wraps the legacy single interaction object (and a single
/// `play_sounds` string) into a one-element array before the typed read; a finding names the
/// member as the file has it, without the element `0` the file does not have.
#[test]
fn a_finding_in_a_legacy_single_interaction_is_at_the_member_as_written() {
    let at = |interactions: &str| -> Vec<String> {
        let text = super::common::doc(
            "1.21.0",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:interact": {{"interactions": {interactions}}}}}}}"#
            ),
        );
        let (_, found) = super::common::stable(&text, &super::common::cx());
        // The legacy schema reader's type line, strict at 1.21.0.
        found
            .iter()
            .filter(|f| f.kept && f.code.id() == "core/unknown_child_option")
            .map(|f| f.location.json_path.to_string())
            .filter(|at| at.contains("minecraft:interact"))
            .collect()
    };
    let base = "minecraft:entity/components/minecraft:interact/interactions";
    // The single object: no element in the file.
    assert_eq!(
        at(r#"{"hurt_item": "x", "swing": true}"#),
        [format!("{base}/hurt_item")]
    );
    // A list: the element the file has, the first and another.
    assert_eq!(
        at(r#"[{"hurt_item": "x", "swing": true}]"#),
        [format!("{base}/0/hurt_item")]
    );
    assert_eq!(
        at(r#"[{"swing": true}, {"hurt_item": "x"}]"#),
        [format!("{base}/1/hurt_item")]
    );
}

// An unknown member of a goal is told whatever its key holds. The
// finding's path writes `/` and `~` in the key escaped; the member is the key as written.
#[test]
fn an_unknown_goal_member_is_told_whatever_characters_its_key_holds() {
    for (key, written) in [("plain", "plain"), ("a/b", "a~1b"), ("a~b", "a~0b")] {
        let text = doc(
            "1.26.50",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:behavior.float": {{"priority": 0, "{key}": 1}}}}}}"#
            ),
        );
        // A strict slice refuses it.
        let refused = outcome(&text, &cx());
        assert!(refused.value.is_none());
        let found = refused.diagnostics;
        let told: Vec<(Severity, String)> = with_code(&found, "core/unrecognized_field")
            .iter()
            .map(|f| (f.severity, f.location.json_path.to_string()))
            .collect();
        assert_eq!(
            told,
            [(
                Severity::Error,
                format!("minecraft:entity/components/minecraft:behavior.float/{written}")
            )],
            "{key:?}"
        );
        // The path's last step is the key as the file has it.
        let unknown = with_code(&found, "core/unrecognized_field")[0];
        assert_eq!(
            unknown.location.json_path.segments().last(),
            Some(bedrock_addon_core::diagnostic::JsonPathSegment::Key(key))
        );
    }
}

/// The creaking of the development build (`behavior_packs/vanilla_1.21.50` and
/// `vanilla_1.21.60`): a root `use_beta_features` beside the version and the payload.
fn creaking(flag: &str) -> String {
    format!(
        r#"{{
  "format_version": "1.26.0",
  "use_beta_features": {flag},
  "minecraft:entity": {{
    "description": {{"identifier": "minecraft:creaking", "is_summonable": true, "is_spawnable": true}},
    "components": {{"minecraft:health": {{"value": 1, "max": 1}}}}
  }}
}}"#
    )
}

/// The findings of a load of `text`, as `<code> <severity> <path>`, and whether it loaded.
fn told_at_the_root(text: &str, cx: &LoadContext) -> (Vec<String>, bool) {
    use bedrock_addon_core::document::Content;
    let outcome = Entity::parse(
        &bedrock_addon_core::pack_identity::PackPath::new("entities/creaking.json").unwrap(),
        text.as_bytes(),
        cx,
    );
    let (found, loaded) = match &outcome {
        Ok(document) => (document.diagnostics().clone(), true),
        Err(rejected) => (rejected.diagnostics.clone(), false),
    };
    let said = found
        .iter()
        .map(|f| format!("{} {:?} {}", f.code.id(), f.severity, f.location.json_path))
        .collect();
    (said, loaded)
}

/// 1.26.60 has no `use_beta_features` and no root
/// member count for an entity: the loader takes `format_version` and
/// `minecraft:entity` out of the root and hands
/// the payload alone on, reading no other root member. So a root `use_beta_features`, or any
/// third root member, is passed over and the entity loads, whatever the pack and the toggles.
/// The game says nothing; the crate tells the member as it tells every root member a family
/// never counts (`UnrecognizedField`, an information of the crate's own) and writes the document back without it.
#[test]
fn a_root_member_beside_the_version_and_the_payload_is_passed_over() {
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::json::JsonSlice;
    use bedrock_addon_core::versioning::parse_as_written;
    let base_game = {
        let mut cx = cx();
        cx.is_base_game_pack = true;
        cx
    };
    let beta_apis = {
        let mut cx = cx();
        cx.beta_apis = true;
        cx
    };
    let other = creaking("true").replace("use_beta_features", "use_alpha_features");
    for context in [&base_game, &beta_apis, &cx()] {
        for (text, member) in [
            (creaking("true"), "use_beta_features"),
            (creaking("false"), "use_beta_features"),
            (creaking(r#""yes""#), "use_beta_features"),
            (other.clone(), "use_alpha_features"),
        ] {
            let (found, loaded) = told_at_the_root(&text, context);
            assert!(loaded, "{text}\n{found:#?}");
            assert_eq!(
                found,
                [format!("core/unrecognized_field Info {member}")],
                "{text}"
            );
            let parsed = parse_as_written::<Entity>(JsonSlice::new(&text), context);
            assert!(parsed.value.is_some(), "{text}: {:#?}", parsed.diagnostics);
            let document = Entity::parse(
                &bedrock_addon_core::pack_identity::PackPath::new("entities/creaking.json")
                    .unwrap(),
                text.as_bytes(),
                context,
            )
            .unwrap();
            let written = document.to_json().unwrap();
            assert!(!written.contains(member), "{written}");
        }
    }
}

/// The other side: without a member count the two members the loader takes still
/// decide. The plain creaking loads with nothing told at the root, and a root whose third
/// member stands where the payload should is refused for the missing payload, not the count.
#[test]
fn the_uncounted_root_still_needs_its_payload() {
    let plain = creaking("true").replace("  \"use_beta_features\": true,\n", "");
    assert!(!plain.contains("use_beta_features"), "{plain}");
    let (found, loaded) = told_at_the_root(&plain, &cx());
    assert!(loaded, "{found:#?}");
    assert!(found.is_empty(), "{found:#?}");
    let no_payload = creaking("true").replace("\"minecraft:entity\"", "\"minecraft:actor\"");
    let (found, loaded) = told_at_the_root(&no_payload, &cx());
    assert!(!loaded, "{found:#?}");
    assert!(
        found
            .iter()
            .any(|f| f.starts_with("core/missing_payload Error")),
        "{found:#?}"
    );
    assert!(
        !found.iter().any(|f| f.starts_with("MalformedInput")),
        "{found:#?}"
    );
}

/// A file refused before it is read (no UTF-8, nested past the limit) names its file as a
/// file refused by the read does.
#[test]
fn a_file_refused_before_the_read_is_placed_in_its_file() {
    use bedrock_addon_core::document::Content;
    let path = bedrock_addon_core::pack_identity::PackPath::new("entities/creaking.json").unwrap();
    let nested = format!("{}{}", "[".repeat(200), "]".repeat(200));
    for bytes in [&b"{\"a\": \"\xff\"}"[..], nested.as_bytes()] {
        for rejected in [
            Entity::parse(&path, bytes, &cx()).unwrap_err(),
            Entity::parse_as_written(&path, bytes, &cx()).unwrap_err(),
        ] {
            let refusal = with_code(&rejected.diagnostics, "core/syntax")[0];
            assert!(refusal.location.pack.is_some(), "{refusal:#?}");
            assert_eq!(refusal.location.path.as_ref(), Some(&path));
        }
    }
}

/// What the lenient conversion tells of a list element is at the index the file has the
/// element at: a goal's second failing element behind the first it took out, and a rule of
/// a typed component behind an element left out.
#[test]
fn a_finding_of_the_lenient_conversion_behind_an_element_left_out_is_at_the_index_of_the_file() {
    let text = doc(
        "1.16.0",
        r#"{ "description": { "identifier": "demo:x" }, "components": {
            "minecraft:behavior.raid_garden": { "priority": 1, "blocks": [ 7, "carrots", 8, "wheat", 9 ] },
            "minecraft:home": { "home_block_list": [ 7, "bee_nest", {}, "beehive", true ] },
            "minecraft:anger_level": { "on_increase_sounds": [ 7, { "sound": "a", "condition": { "expression": "1" } }, { "sound": "b", "condition": "1" }, { "sound": "c", "condition": { "expression": "1" } } ] }
        } }"#,
    );
    let (_, found) = stable(&text, &cx());
    let at: Vec<String> = found
        .iter()
        .filter(|finding| finding.severity >= Severity::Warning)
        .map(|finding| finding.location.json_path.to_string())
        .filter(|path| path.contains("/components/"))
        .collect();
    let c = "minecraft:entity/components/minecraft:";
    assert_eq!(
        at,
        [
            // `raid_garden`'s hand-written legacy reader says nothing: its three are
            // informations; `home` and `anger_level` are schema readers.
            format!("{c}home/home_block_list/0"),
            format!("{c}home/home_block_list/2"),
            format!("{c}home/home_block_list/4"),
            format!("{c}anger_level/on_increase_sounds/0"),
            format!("{c}anger_level/on_increase_sounds/1/condition"),
            format!("{c}anger_level/on_increase_sounds/3/condition"),
        ],
        "{found:#?}"
    );
}

/// A list element the lenient conversion leaves out for what it is (an empty list where a
/// string is asked) is told at the element: the file has nothing below it.
#[test]
fn an_element_left_out_for_what_it_is_is_told_at_the_element_the_file_has() {
    let text = doc(
        "1.16.0",
        r#"{ "description": { "identifier": "demo:x" }, "components": {
            "minecraft:home": { "home_block_list": [ "bee_nest", [], "beehive" ] }
        } }"#,
    );
    let (_, found) = stable(&text, &cx());
    let at: Vec<String> = found
        .iter()
        .map(|finding| format!("{} {}", finding.code.id(), finding.location.json_path))
        .filter(|told| told.contains("/components/"))
        .collect();
    // `home`'s legacy reader is a schema reader (typed first at 1.26.50): the element no
    // option takes is its type line, at the element the file has; strict at 1.16.0, the
    // component at its defaults.
    assert_eq!(
        at,
        [
            "core/unknown_child_option minecraft:entity/components/minecraft:home/home_block_list/1",
            "core/value_rewritten minecraft:entity/components/minecraft:home",
            "core/dropped_field minecraft:entity/components/minecraft:home/home_block_list"
        ],
        "{found:#?}"
    );
}

/// The other door, `parse_as_written` and then `upgrade`, tells what the load tells of a
/// document whose lenient conversion left elements out, at the same places.
#[test]
fn the_upgrade_of_a_document_as_written_tells_what_the_load_tells_behind_an_element_left_out() {
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::pack_identity::PackPath;
    let text = doc(
        "1.16.0",
        r#"{ "description": { "identifier": "demo:x" }, "components": {
            "minecraft:behavior.raid_garden": { "priority": 1, "blocks": [ 7, "carrots", 8, "wheat" ] },
            "minecraft:anger_level": { "on_increase_sounds": [ 7, { "sound": "a", "condition": { "expression": "1" } }, { "sound": "b", "condition": "1" } ] }
        } }"#,
    );
    let path = PackPath::new("entities/test.json").unwrap();
    let told = |d: &bedrock_addon_core::diagnostic::Diagnostics| -> Vec<String> {
        d.iter()
            .map(|f| format!("{} {}", f.code.id(), f.location.json_path))
            .collect()
    };
    let loaded = Entity::parse(&path, text.as_bytes(), &cx()).expect("it loads");
    let written = Entity::parse_as_written(&path, text.as_bytes(), &cx()).expect("it is read");
    let upgraded = Entity::upgrade(written, &cx()).expect("it upgrades");
    assert_eq!(told(upgraded.diagnostics()), told(loaded.diagnostics()));
    assert!(
        told(loaded.diagnostics())
            .iter()
            .any(|told| told.ends_with("behavior.raid_garden/blocks/2")),
        "{:#?}",
        loaded.diagnostics()
    );
}

// ---- the entity validation functions ----

/// The timer validation: a `timer` with both `time` and
/// `random_time_choices` is told (`ValidationResultCode` 1, a warning) and keeps `time`, in
/// `components` and in each group; either alone says nothing.
#[test]
fn a_timer_with_time_and_random_time_choices_is_a_warning() {
    let text = "Both \"time\" and \"random_time_choices\" have been specified for a timer. Defaulting to \"time\".";
    let both = r#"{"time": [1, 2], "random_time_choices": [{"value": 3, "weight": 1}]}"#;
    let (_, found) = stable(
        &with_components("1.26.50", &format!(r#""minecraft:timer": {both}"#)),
        &cx(),
    );
    let warned: Vec<String> = found
        .iter()
        .filter(|f| f.severity == Severity::Warning)
        .map(|f| format!("{} {}", f.location.json_path, f.code))
        .collect();
    assert_eq!(
        warned,
        [format!(
            "minecraft:entity/components/minecraft:timer {text}"
        )]
    );
    let group = doc(
        "1.26.50",
        &format!(
            r#"{{"description": {{"identifier": "x:a"}}, "components": {{}}, "component_groups": {{"g": {{"minecraft:timer": {both}}}}}}}"#
        ),
    );
    let (_, found) = stable(&group, &cx());
    assert_eq!(
        codes(&found, Severity::Warning),
        ["entity/timer_both"],
        "{found:#?}"
    );
    for alone in [
        r#"{"time": [1, 2]}"#,
        r#"{"random_time_choices": [{"value": 3}]}"#,
    ] {
        let (_, found) = stable(
            &with_components("1.26.50", &format!(r#""minecraft:timer": {alone}"#)),
            &cx(),
        );
        assert!(
            codes(&found, Severity::Warning).is_empty(),
            "{alone}: {found:#?}"
        );
    }
}

/// The preferred path validation: a block in two cost groups of
/// `preferred_path` is an error (`ValidationResultCode` 2) at the second group that refuses the entity, with the
/// game's text and the block's name; the same block twice in one group says nothing.
#[test]
fn a_preferred_path_block_in_two_cost_groups_is_an_error() {
    let path = |sets: &str| {
        with_components(
            "1.26.50",
            &format!(r#""minecraft:preferred_path": {{"preferred_path_blocks": {sets}}}"#),
        )
    };
    let refused = outcome(
        &path(
            r#"[{"cost": 0, "blocks": ["minecraft:grass_path", "minecraft:stone"]}, {"cost": 3, "blocks": ["minecraft:dirt", {"name": "minecraft:stone"}]}]"#,
        ),
        &cx(),
    );
    assert!(refused.value.is_none());
    let errors: Vec<String> = refused
        .diagnostics
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| {
            format!(
                "{} {:?} {} {}",
                f.code.id(),
                f.phase,
                f.location.json_path,
                f.code
            )
        })
        .collect();
    assert_eq!(
        errors,
        [
            "entity/duplicate_path_block Validate minecraft:entity/components/minecraft:preferred_path/preferred_path_blocks/1 Duplicate block descriptors in two costing groups minecraft:stone"
        ],
        "{:#?}",
        refused.diagnostics
    );
    let (_, found) = stable(
        &path(
            r#"[{"cost": 0, "blocks": ["minecraft:stone", "minecraft:stone"]}, {"cost": 3, "blocks": ["minecraft:dirt"]}]"#,
        ),
        &cx(),
    );
    assert!(codes(&found, Severity::Warning).is_empty(), "{found:#?}");
}

// The game's test for a deprecated component after its cut-off version: at 1.16.100 `minecraft:foot_size` is no component of the schema, and the log
// holds `minecraft:foot_size: this component was found in the input, but is not present in
// the Schema` (the path, then the text). The legacy slice is unstrict: a warning, kept.
#[test]
fn an_unknown_component_has_the_game_text_at_the_component() {
    let text = super::common::fixture("engine/upgrade-asm-2.json");
    let after = outcome(&text, &cx());
    assert!(after.value.is_some(), "{:#?}", after.diagnostics);
    let found: Vec<_> = after
        .diagnostics
        .iter()
        // By legacy name: the variant and the definitions that were it.
        .filter(|f| matches!(f.code.id(), "core/component_not_in_schema"))
        .map(|f| {
            (
                f.severity,
                f.location.json_path.to_string(),
                f.code.to_string(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [(
            Severity::Warning,
            "minecraft:entity/components/minecraft:foot_size".to_owned(),
            "this component was found in the input, but is not present in the Schema".to_owned()
        )]
    );
    // Before the cut-off (`..._BeforeCutOffVersion_NoContentErrors`, df12a54): no such line.
    let before = super::common::fixture("engine/upgrade-asm-1.json");
    let earlier = outcome(&before, &cx());
    assert!(
        earlier
            .diagnostics
            .iter()
            // By legacy name: the variant and the definitions that were it.
            .all(|f| !matches!(f.code.id(), "core/component_not_in_schema")),
        "{:#?}",
        earlier.diagnostics
    );
}

// The split at the strict slices' edge itself.
#[test]
fn a_deprecated_component_is_a_warning_at_1_21_88_and_rejects_at_1_21_89() {
    let text = |version: &str| {
        doc(
            version,
            r#"{"description": {"identifier": "x:a"}, "components": {"minecraft:lookat": {}}}"#,
        )
    };
    let legacy = outcome(&text("1.21.88"), &cx());
    assert!(legacy.value.is_some(), "{:#?}", legacy.diagnostics);
    let found = with_code(&legacy.diagnostics, "core/component_not_in_schema");
    assert_eq!(found.len(), 1, "{:#?}", legacy.diagnostics);
    assert_eq!(found[0].severity, Severity::Warning);
    let strict = outcome(&text("1.21.89"), &cx());
    assert!(strict.value.is_none(), "{:#?}", strict.diagnostics);
    let found = with_code(&strict.diagnostics, "core/component_not_in_schema");
    assert_eq!(found.len(), 1, "{:#?}", strict.diagnostics);
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(
        found[0].code.to_string(),
        "this component was found in the input, but is not present in the Schema"
    );
}
