//! The newest struct: reading, writing, the inherent helpers.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::{Content, Placed};
use bedrock_addon_core::id::EntityId;
use bedrock_addon_family_entity::{Entity, Event, PropertyDefault, PropertyType, SpawnCategory};

use super::common::{codes, cx, doc, json, parse, reparse, stable};

const COW: &str = r#"{
  "description": {
    "identifier": "demo:cow",
    "is_spawnable": true,
    "is_summonable": true,
    "is_experimental": false,
    "spawn_category": "creature",
    "properties": {
      "demo:mood": { "type": "enum", "values": ["calm", "angry"], "default": "calm", "client_sync": true },
      "demo:size": { "type": "float", "range": [0.5, 2.0], "default": "query.had_component_group('demo:baby') ? 0.5 : 1.0" },
      "demo:count": { "type": "int", "range": [0, 9], "default": 3 }
    },
    "animations": { "look": "controller.animation.demo.look" },
    "scripts": { "animate": ["look", { "walk": "query.is_moving" }] }
  },
  "components": {
    "minecraft:health": { "value": 10, "max": 10 },
    "minecraft:type_family": { "family": ["cow", "mob"] }
  },
  "component_groups": {
    "demo:baby": { "minecraft:is_baby": {} }
  },
  "events": {
    "minecraft:entity_born": { "add": { "component_groups": ["demo:baby"] } }
  }
}"#;

#[test]
fn a_1_26_50_entity_reads_every_document_member() {
    let (entity, diagnostics) = stable(&doc("1.26.50", COW), &cx());
    assert_eq!(codes(&diagnostics, Severity::Warning), Vec::<&str>::new());
    let description = &entity.description;
    assert_eq!(description.identifier.as_str(), "demo:cow");
    assert!(entity.is_spawnable());
    assert!(entity.is_summonable());
    assert_eq!(description.spawn_category, Some(SpawnCategory::Creature));
    let description_at =
        bedrock_addon_core::diagnostic::JsonPath::of_keys(["minecraft:entity", "description"]);
    assert_eq!(
        diagnostics
            .unknown()
            .get(&description_at, "is_experimental"),
        Some(&json("false"))
    );
    let mood = &entity.properties()["demo:mood"];
    assert_eq!(mood.kind, PropertyType::Enum);
    assert_eq!(mood.default, Some(PropertyDefault::Enum("calm".to_owned())));
    assert!(matches!(
        entity.properties()["demo:size"].default,
        Some(PropertyDefault::Expression(_))
    ));
    assert_eq!(
        entity.properties()["demo:count"].default,
        Some(PropertyDefault::Int(3))
    );
    assert_eq!(entity.components.len(), 2);
    assert_eq!(
        entity
            .component_group("demo:baby")
            .unwrap()
            .components()
            .len(),
        1
    );
    let born = &entity.events["minecraft:entity_born"];
    assert_eq!(born.name(), "minecraft:entity_born");
    assert_eq!(
        born.node,
        json(r#"{ "add": { "component_groups": ["demo:baby"] } }"#)
    );
}

#[test]
fn the_newest_struct_round_trips_value_for_value() {
    let (entity, _) = stable(&doc("1.26.50", COW), &cx());
    assert_eq!(reparse(&entity), entity);
}

/// An unknown member is told at its level; from 1.21.89 (a strict slice) it
/// refuses the entity, below it is a warning and is not kept (the conversion to the newest
/// struct does not carry it), so the write-back reads back.
#[test]
fn unknown_members_are_told_at_their_level_and_not_kept() {
    let payload = r#"{
      "description": { "identifier": "demo:x", "demo_note": 1 },
      "components": {},
      "demo_root": true
    }"#;
    let refused = super::common::outcome(&doc("1.26.50", payload), &cx());
    assert!(refused.value.is_none());
    assert_eq!(
        codes(&refused.diagnostics, Severity::Error),
        ["UnrecognizedField", "UnrecognizedField"],
        "{:#?}",
        refused.diagnostics
    );
    let (entity, diagnostics) = stable(&doc("1.21.80", payload), &cx());
    assert_eq!(
        codes(&diagnostics, Severity::Warning),
        ["UnrecognizedField", "UnrecognizedField"],
        "{diagnostics:#?}"
    );
    let _ = entity;
    let payload = bedrock_addon_core::diagnostic::JsonPath::of_keys(["minecraft:entity"]);
    assert!(diagnostics.unknown().get(&payload, "demo_root").is_none());
    assert!(
        diagnostics
            .unknown()
            .get(&payload.child("description"), "demo_note")
            .is_none()
    );
    assert_eq!(reparse(&entity), entity);
}

#[test]
fn null_events_read_as_none() {
    let payload =
        r#"{ "description": { "identifier": "demo:x" }, "components": {}, "events": null }"#;
    let (entity, diagnostics) = stable(&doc("1.26.50", payload), &cx());
    assert!(entity.events.is_empty());
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

/// An unknown component refuses the entity from 1.21.89; below, it is a
/// warning and is not kept.
#[test]
fn an_unknown_component_is_reported_and_not_kept() {
    let payload = r#"{ "description": { "identifier": "demo:x" }, "components": { "demo:thing": { "a": 1 } } }"#;
    assert!(
        super::common::outcome(&doc("1.26.50", payload), &cx())
            .value
            .is_none()
    );
    let (entity, diagnostics) = stable(&doc("1.21.80", payload), &cx());
    assert_eq!(
        codes(&diagnostics, Severity::Warning),
        ["UnrecognizedComponent"]
    );
    assert!(entity.components.iter().all(|entry| !matches!(
        entry,
        bedrock_addon_core::document::ComponentRef::Unknown { .. }
    )));
    assert_eq!(reparse(&entity), entity);
}

#[test]
fn new_and_default_path() {
    let entity = Entity::new(EntityId::parse("demo:wolf"));
    assert_eq!(entity.default_path().as_str(), "entities/demo/wolf.json");
    assert!(!entity.is_spawnable());
    let event = Event::new("demo:go", json("{}"));
    assert_eq!(event.name(), "demo:go");
    let text = entity.to_json().unwrap();
    assert!(
        text.starts_with("{\n  \"format_version\": \"1.26.60\""),
        "{text}"
    );
    assert_eq!(reparse(&entity), entity);
}

#[test]
fn content_parse_returns_a_document() {
    let document = parse(&doc("1.26.60", COW)).expect("loads");
    assert_eq!(document.description.identifier.as_str(), "demo:cow");
    assert_eq!(document.version().parsed_as.to_string(), "1.26.60");
    assert!(!document.version().upgraded);
}

/// An empty member keeps the author's presence, and an absent one is not
/// written: `components`, `component_groups`, `events` and `scripts.animate`.
#[test]
fn empty_members_keep_the_authors_presence() {
    let payload_of = |text: &str| -> bedrock_addon_core::json::Value {
        let entity = super::common::stable(text, &super::common::cx()).0;
        let written = json(&entity.to_json().unwrap());
        written
            .as_object()
            .and_then(|d| d.get("minecraft:entity"))
            .cloned()
            .unwrap()
    };
    // (A member `scripts` does not read is an unknown member, refused from 1.21.89.)
    let bare = r#"{"format_version": "1.26.50", "minecraft:entity": {"description": {"identifier": "x:a", "scripts": {}}}}"#;
    assert_eq!(
        payload_of(bare),
        json(r#"{"description": {"identifier": "x:a", "scripts": {}}}"#)
    );
    let empty = r#"{"format_version": "1.26.50", "minecraft:entity": {"description": {"identifier": "x:a", "scripts": {"animate": []}}, "components": {}, "component_groups": {}, "events": {}}}"#;
    assert_eq!(
        payload_of(empty),
        json(
            r#"{"description": {"identifier": "x:a", "scripts": {"animate": []}}, "components": {}, "component_groups": {}, "events": {}}"#
        )
    );
    // A value built in code writes its `components`.
    let built =
        bedrock_addon_family_entity::Entity::new(bedrock_addon_core::id::EntityId::parse("x:a"));
    assert!(built.to_json().unwrap().contains("\"components\""));
}

// An object given to an identifier is a wrong type at the member, and no panic.
#[test]
fn an_identifier_that_is_an_object_is_a_type_error_and_no_panic() {
    let rejected = parse(&doc(
        "1.26.50",
        r#"{"description": {"identifier": {}}, "components": {}}"#,
    ))
    .expect_err("the document is rejected");
    let found: Vec<_> = rejected
        .diagnostics
        .iter()
        .filter(|f| f.code.name() == "InvalidType")
        .collect();
    assert_eq!(found.len(), 1, "{:#?}", rejected.diagnostics);
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:entity/description/identifier"
    );
}

/// The game reads an entity so that a member written as `null` is absent, also at the typed
/// version: a bare strict member keeps its default.
#[test]
fn a_null_member_of_a_typed_component_takes_its_default() {
    let payload = r#"{ "description": { "identifier": "demo:x" }, "components": { "minecraft:insomnia": { "days_until_insomnia": null } } }"#;
    let (entity, diagnostics) = stable(&doc("1.26.50", payload), &cx());
    // The game says nothing; the crate writes the default back where the file has `null`, so
    // the place carries an information.
    let told: Vec<String> = diagnostics
        .iter()
        .map(|f| {
            format!(
                "{:?} {} {}",
                f.severity,
                f.code.name(),
                f.location.json_path
            )
        })
        .collect();
    assert_eq!(
        told,
        ["Info ValueRewritten minecraft:entity/components/minecraft:insomnia/days_until_insomnia"],
        "{diagnostics:#?}"
    );
    let insomnia = entity
        .components
        .get::<bedrock_addon_family_entity::component::Insomnia>()
        .expect("the component is kept");
    assert_eq!(insomnia.days_until_insomnia, 3.0);
}

/// A member the type deprecates is `member "…" is deprecated`, an
/// error in both masks: the entity is not loaded.
#[test]
fn a_deprecated_member_rejects_the_entity() {
    let payload = r#"{ "description": { "identifier": "demo:x" }, "components": { "minecraft:wither_target_highest_damage": { "must_see": true } } }"#;
    let outcome = super::common::outcome(&doc("1.26.50", payload), &cx());
    assert!(outcome.value.is_none(), "{:#?}", outcome.diagnostics);
    assert!(
        outcome
            .diagnostics
            .iter()
            .any(|f| f.severity == Severity::Error
                && f.code.to_string() == "member \"must_see\" is deprecated"),
        "{:#?}",
        outcome.diagnostics
    );
}

/// For a component a schema reader reads, the schema looks at
/// every member as written, and a `null` is no value of a typed node: the type line at error
/// level and, strict, the component at its defaults; the entity loads. A `null` is absent where
/// the game reads a member by name.
#[test]
fn a_null_member_inside_a_schema_read_component_is_a_value_of_no_option() {
    let payload = r#"{ "description": { "identifier": "demo:x" }, "components": { "minecraft:interact": { "interactions": { "hurt_item": null, "interact_text": "x" } } } }"#;
    let (_, diagnostics) = stable(&doc("1.26.0", payload), &cx());
    assert!(
        diagnostics.iter().any(|f| f.kept
            && f.code.to_string() == "unknown child schema option type.  Allowed types:  'int'"
            && f.location.json_path.to_string()
                == "minecraft:entity/components/minecraft:interact/interactions/hurt_item"),
        "{diagnostics:#?}"
    );
}

// ---- strictness at the typed version ----

/// The game's component definition server tests (fixtures/entity): the
/// snippet `fixture` as component `key` of `test:mob` at `version`.
fn wrapped(version: &str, key: &str, fixture: &str) -> String {
    let snippet = super::common::fixture(&format!("engine/{fixture}"));
    format!(
        "{{\n\t\"format_version\": \"{version}\",\n\t\"minecraft:entity\": {{\n\t\t\"description\": {{\n\t\t\t\"identifier\": \"test:mob\"\n\t\t}},\n\t\t\"components\": {{\n\t\t\t\"{key}\": {snippet}\n\t\t}}\n\t}}\n}}"
    )
}

/// The file `text` through `Content::parse`: its findings as `<severity> <code> <path>`, the
/// texts of its errors, and whether it loaded.
fn parsed(text: &str) -> (Vec<String>, Vec<String>, bool) {
    let (found, loaded) = match parse(text) {
        Ok(document) => (document.diagnostics().clone(), true),
        Err(rejected) => (rejected.diagnostics, false),
    };
    let told = found
        .iter()
        .map(|f| {
            format!(
                "{:?} {} {}",
                f.severity,
                f.code.name(),
                f.location.json_path
            )
        })
        .collect();
    let texts = found
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| f.code.to_string())
        .collect();
    (told, texts, loaded)
}

/// The production line of a refused entity, as the game's tests assert its start.
const FAILED_TO_LOAD: &str = "ERROR: Entity 'test:mob' failed to load from JSON: ";

/// The second `{}` of the production line starts with the failed phase's prefix, inserted
/// before the first log entry's message: one prefix for parse, one for validate.
const PARSE_ERRORS: &str = "parse errors occurred: ";
const VALIDATE_ERRORS: &str = "validate errors occurred: ";

/// An entity refused in validation closes with the validate
/// prefix, then the first error's member and text (`validatePreferredPathComponent`).
#[test]
fn an_entity_refused_in_validation_closes_with_the_validate_prefix() {
    let text = r#"{"format_version": "1.26.50", "minecraft:entity": {"description": {"identifier": "test:mob"}, "components": {"minecraft:preferred_path": {"preferred_path_blocks": [{"cost": 0, "blocks": ["minecraft:stone"]}, {"cost": 3, "blocks": ["minecraft:stone"]}]}}}}"#;
    let (told, texts, loaded) = parsed(text);
    assert!(!loaded, "{told:#?}");
    let last = texts.last().expect("an error");
    assert!(
        last.starts_with(&format!(
            "{FAILED_TO_LOAD}{VALIDATE_ERRORS}components/minecraft:preferred_path"
        )) && last.ends_with("Duplicate block descriptors in two costing groups minecraft:stone"),
        "{texts:#?}"
    );
}

/// At the target version 1.26.50 (`InsomniaTargetWrongThresholdType`,
/// `InstantDespawnTargetWrongValueType`, `InstantDespawnTargetUnknownField`, all
/// `…_IsRejected`): a wrong type or an unknown member of a typed component refuses the whole
/// entity, with the member's entry and then vanilla's closing line.
#[test]
fn a_bad_member_of_a_typed_component_at_its_typed_version_refuses_the_entity() {
    for (key, fixture, member) in [
        (
            "minecraft:insomnia",
            "component-13.json",
            "days_until_insomnia",
        ),
        (
            "minecraft:instant_despawn",
            "component-18.json",
            "remove_child_entities",
        ),
        (
            "minecraft:instant_despawn",
            "component-19.json",
            "unexpected_field",
        ),
    ] {
        let (told, texts, loaded) = parsed(&wrapped("1.26.50", key, fixture));
        assert!(!loaded, "{fixture}: {told:#?}");
        let at = format!("minecraft:entity/components/{key}/{member}");
        assert!(
            told.iter()
                .any(|t| t.starts_with("Error ") && t.ends_with(&at)),
            "{fixture}: {told:#?}"
        );
        let last = texts.last().expect("an error");
        assert!(
            last.starts_with(&format!(
                "{FAILED_TO_LOAD}{PARSE_ERRORS}components/{key}/{member}: "
            )),
            "{fixture}: {texts:#?}"
        );
    }
}

/// The same snippets in a legacy 1.26.40 document (`…Legacy…_UsesThreeDayDefault`,
/// `…_UsesFalseDefault`): the entity loads, the member keeps its default, and the entry is a
/// Warning; load without a finding at both versions.
#[test]
fn the_same_members_in_a_legacy_document_load_with_the_default() {
    // The legacy reader is a schema reader, strict from 1.15.0: the line is the
    // game's, at error level, and the entity loads with the component at its defaults (the
    // game tests' "uses the default"); kept errors.
    for (key, fixture, member, text) in [
        (
            "minecraft:insomnia",
            "component-13.json",
            "days_until_insomnia",
            "unknown child schema option type.  Allowed types:  'float'",
        ),
        (
            "minecraft:instant_despawn",
            "component-18.json",
            "remove_child_entities",
            "unknown child schema option type.  Allowed types:  'bool'",
        ),
        (
            "minecraft:instant_despawn",
            "component-19.json",
            "unexpected_field",
            "child 'unexpected_field' not valid here.",
        ),
    ] {
        let (told, texts, loaded) = parsed(&wrapped("1.26.40", key, fixture));
        assert!(loaded, "{fixture}: {told:#?}");
        let at = format!("minecraft:entity/components/{key}/{member}");
        assert!(
            told.iter()
                .any(|t| t.starts_with("Error ") && t.ends_with(&at)),
            "{fixture}: {told:#?}"
        );
        assert!(texts.iter().any(|t| t == text), "{fixture}: {texts:#?}");
    }
    for version in ["1.26.40", "1.26.50"] {
        for (key, fixture) in [
            ("minecraft:insomnia", "component-12.json"),
            ("minecraft:insomnia", "component-9.json"),
            ("minecraft:instant_despawn", "component-16.json"),
            ("minecraft:instant_despawn", "component-17.json"),
            ("minecraft:instant_despawn", "component-9.json"),
        ] {
            let (told, _, loaded) = parsed(&wrapped(version, key, fixture));
            assert!(loaded, "{version} {fixture}: {told:#?}");
            assert!(
                told.iter()
                    .all(|t| !t.starts_with("Error") && !t.starts_with("Warning")),
                "{version} {fixture}: {told:#?}"
            );
        }
    }
}

/// From 1.21.89 every slice is registered with the
/// strict mask, so an unknown member of a component read at its typed version, and an unknown
/// component, refuse the entity also in a document the steps carry to the newest struct;
/// below 1.21.89 both are warnings.
#[test]
fn unknown_members_and_components_refuse_from_1_21_89() {
    let goal = |version: &str| {
        doc(
            version,
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.aquatic_charge_attack": {"priority": 1, "made_up": 1}}}"#,
        )
    };
    let component = |version: &str| {
        doc(
            version,
            r#"{"description": {"identifier": "demo:x"}, "components": {"demo:made_up": {}}}"#,
        )
    };
    for version in ["1.26.0", "1.26.50"] {
        let (told, _, loaded) = parsed(&goal(version));
        assert!(!loaded, "{version}: {told:#?}");
        assert!(told.contains(&
            "Error UnrecognizedField minecraft:entity/components/minecraft:behavior.aquatic_charge_attack/made_up".to_owned()), "{version}: {told:#?}");
    }
    for version in ["1.21.90", "1.26.50"] {
        let (told, _, loaded) = parsed(&component(version));
        assert!(!loaded, "{version}: {told:#?}");
        assert!(
            told.contains(
                &"Error UnrecognizedComponent minecraft:entity/components/demo:made_up".to_owned()
            ),
            "{version}: {told:#?}"
        );
    }
    let (told, _, loaded) = parsed(&component("1.21.80"));
    assert!(loaded, "{told:#?}");
    assert!(
        told.contains(
            &"Warning UnrecognizedComponent minecraft:entity/components/demo:made_up".to_owned()
        ),
        "{told:#?}"
    );
}

/// The adjacent versions of the strictness bound: 1.21.88 is the last unstrict slice, 1.21.89 the
/// first strict one, for an unknown component and an unknown member of the payload.
#[test]
fn strictness_starts_at_1_21_89_not_1_21_88() {
    for (body, at, code) in [
        (
            r#"{"description": {"identifier": "demo:x"}, "components": {"demo:made_up": {}}}"#,
            "minecraft:entity/components/demo:made_up",
            "UnrecognizedComponent",
        ),
        (
            r#"{"description": {"identifier": "demo:x"}, "components": {}, "made_up": {}}"#,
            "minecraft:entity/made_up",
            "UnrecognizedField",
        ),
    ] {
        let text = |version: &str| doc(version, body);
        let (told, _, loaded) = parsed(&text("1.21.88"));
        assert!(loaded, "1.21.88 {body}: {told:#?}");
        assert!(
            told.contains(&format!("Warning {code} {at}")),
            "1.21.88: {told:#?}"
        );
        let (told, _, loaded) = parsed(&text("1.21.89"));
        assert!(!loaded, "1.21.89 {body}: {told:#?}");
        assert!(
            told.contains(&format!("Error {code} {at}")),
            "1.21.89: {told:#?}"
        );
    }
}

/// A component converted from a document below its typed version is read by the legacy
/// reader, outside the strict slice (`behavior.panic`, typed at 1.26.60, in a 1.26.50
/// document): the legacy goal reader reads the keys it knows and nothing
/// else, so the unknown member is an information.
#[test]
fn a_converted_goal_s_unknown_member_is_an_information() {
    let (told, _, loaded) = parsed(&doc(
        "1.26.50",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.panic": {"speed_multiplier": 1.25, "made_up": 1}}}"#,
    ));
    assert!(loaded, "{told:#?}");
    assert!(
        told.contains(
            &"Info UnrecognizedField minecraft:entity/components/minecraft:behavior.panic/made_up"
                .to_owned()
        ),
        "{told:#?}"
    );
}

/// At the 1.26.60 target (`BreakDoor…RejectsUnknownField`,
/// `Attack…RejectsUnknownField`, `Container…RejectsUnknownField` and the wrong types): the
/// components typed at 1.26.60 refuse the entity, the member's entry an error, then the
/// closing line; at 1.26.50 the legacy reader only tells it.
#[test]
fn the_1_26_60_components_refuse_a_bad_member_at_their_typed_version() {
    for (key, fixture, member) in [
        (
            "minecraft:annotation.break_door",
            "component-19.json",
            "unexpected_field",
        ),
        ("minecraft:attack", "component-19.json", "unexpected_field"),
        (
            "minecraft:inventory",
            "component-19.json",
            "unexpected_field",
        ),
        (
            "minecraft:annotation.break_door",
            "component-21.json",
            "break_time",
        ),
        ("minecraft:attack", "component-24.json", "damage"),
        ("minecraft:inventory", "component-30.json", "inventory_size"),
    ] {
        let (told, texts, loaded) = parsed(&wrapped("1.26.60", key, fixture));
        assert!(!loaded, "{key} {fixture}: {told:#?}");
        let at = format!("minecraft:entity/components/{key}/{member}");
        assert!(
            told.iter()
                .any(|t| t.starts_with("Error ") && t.ends_with(&at)),
            "{key} {fixture}: {told:#?}"
        );
        assert!(
            texts.last().is_some_and(|t| t.starts_with(FAILED_TO_LOAD)),
            "{key} {fixture}: {texts:#?}"
        );
    }
    let (told, _, loaded) = parsed(&wrapped("1.26.50", "minecraft:attack", "component-19.json"));
    assert!(loaded, "{told:#?}");
    assert!(
        // The hand-written legacy `attack` reader reads four keys and ignores the rest: an information.
        told.contains(
            &"Info UnrecognizedField minecraft:entity/components/minecraft:attack/unexpected_field"
                .to_owned()
        ),
        "{told:#?}"
    );
}

/// The beta slice is registered with the strict error codes: an unknown member refuses a beta document, as it refuses a stable one.
#[test]
fn a_beta_document_s_unknown_member_refuses_it() {
    let mut context = cx();
    context.is_base_game_pack = true;
    let text = doc(
        "beta",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:apply_knockback_rules": {"presets": [{"horizontal_power": 0.4, "scale_previous_velocity": 1.0}]}}}"#,
    );
    let refused = super::common::outcome(&text, &context);
    assert!(refused.value.is_none(), "{:#?}", refused.diagnostics);
    assert_eq!(
        codes(&refused.diagnostics, Severity::Error),
        ["UnrecognizedField"]
    );
}

/// Which legacy readers tell an unknown member. The
/// hand-written ones (every goal but `melee_attack`, `attack`, `projectile` and its `on_hit`
/// members) read the keys they know and say nothing; the schema readers (`rideable`,
/// `buoyant`, `melee_attack`, `instant_despawn`, …) say nothing below 1.15.0
/// (the schema is lenient for a document below 1.15.0) and log "child '<name>' not valid
/// here." from it. The silent ones are
/// informations; the logged ones stay warnings.
#[test]
fn a_legacy_reader_s_unknown_member_is_an_information_where_the_game_is_silent() {
    let entity = |version: &str, components: &str| {
        doc(
            version,
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}"#
            ),
        )
    };
    for (version, components, at) in [
        (
            "1.16.0",
            r#""minecraft:behavior.leap_at_target": {"priority": 4, "target_dist": 0.4}"#,
            "minecraft:behavior.leap_at_target/target_dist",
        ),
        (
            "1.16.0",
            r#""minecraft:projectile": {"semi_random_diff_damage": true}"#,
            "minecraft:projectile/semi_random_diff_damage",
        ),
        (
            "1.14.0",
            r#""minecraft:rideable": {"seat_count": 1, "priority": 0}"#,
            "minecraft:rideable/priority",
        ),
        (
            "1.13.0",
            r#""minecraft:behavior.melee_attack": {"priority": 3, "set_persistent": true}"#,
            "minecraft:behavior.melee_attack/set_persistent",
        ),
    ] {
        let (told, _, loaded) = parsed(&entity(version, components));
        assert!(loaded, "{version} {components}: {told:#?}");
        let line = format!("Info UnrecognizedField minecraft:entity/components/{at}");
        assert!(told.contains(&line), "{version} {components}: {told:#?}");
    }
    // From 1.15.0 the schema reader logs the member at error level and keeps the
    // component's defaults: a kept error with the game's text
    let (told, texts, loaded) = parsed(&entity(
        "1.16.0",
        r#""minecraft:rideable": {"seat_count": 1, "priority": 0}"#,
    ));
    assert!(loaded, "{told:#?}");
    assert!(
        told.contains(
            &"Error UnrecognizedField minecraft:entity/components/minecraft:rideable/priority"
                .to_owned()
        ),
        "{told:#?}"
    );
    assert!(
        texts.contains(&"child 'priority' not valid here.".to_owned()),
        "{texts:#?}"
    );
}

/// The hand-written legacy readers take a value outside the typed
/// bounds without a word (`probability` is read as a plain float): the crate's lenient bound
/// check is its own, an information. The value still takes its
/// default as before.
#[test]
fn a_legacy_reader_s_out_of_bounds_value_is_an_information() {
    for (components, at) in [
        (
            r#""minecraft:behavior.look_at_player": {"priority": 7, "probability": 8.0}"#,
            "minecraft:behavior.look_at_player/probability",
        ),
        // (The projectile's legacy spawn chances are read, not bounded.)
        // A schema reader (`ranged_attack`) has no bound either: its numeric nodes
        // carry no range text; vanilla's blaze writes 0.0.
        (
            r#""minecraft:behavior.ranged_attack": {"priority": 2, "charge_charged_trigger": 0.0}"#,
            "minecraft:behavior.ranged_attack/charge_charged_trigger",
        ),
    ] {
        let text = doc(
            "1.16.0",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}"#
            ),
        );
        let (told, _, loaded) = parsed(&text);
        assert!(loaded, "{components}: {told:#?}");
        let line = format!("Info InvalidValue minecraft:entity/components/{at}");
        assert!(told.contains(&line), "{components}: {told:#?}");
    }
}

/// The value texts the legacy goal reader writes to
/// the content log for a goal converted below its typed version; the caller
/// ignores the reader's result, so the
/// goal is kept every time. Level 3 lines are kept errors, level 2 lines warnings.
#[test]
fn the_legacy_goal_reader_s_value_texts() {
    let found = |components: &str| {
        let text = doc(
            "1.16.0",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}"#
            ),
        );
        super::common::stable(&text, &super::common::cx()).1
    };
    let told = |d: &bedrock_addon_core::diagnostic::Diagnostics, end: &str, text: &str| {
        d.iter()
            .find(|f| f.location.json_path.to_string().ends_with(end) && f.code.to_string() == text)
            .map(|f| (f.severity, f.kept))
    };
    // Level 3: a negative chance is 0.
    let d =
        found(r#""minecraft:behavior.float": {"priority": 0, "chance_per_tick_to_float": -0.5}"#);
    assert_eq!(
        told(
            &d,
            "minecraft:behavior.float/chance_per_tick_to_float",
            "behavior.float - chance_per_tick_to_float value should be `0.0 <= value <= 1.0`"
        ),
        Some((Severity::Error, true)),
        "{d:#?}"
    );
    // Level 3.
    let d = found(
        r#""minecraft:behavior.float": {"priority": 0, "time_under_water_to_dismount_passengers": -1}"#,
    );
    assert_eq!(
        told(
            &d,
            "minecraft:behavior.float/time_under_water_to_dismount_passengers",
            "behavior.float - time_under_water_to_dismount_passengers should be positive"
        ),
        Some((Severity::Error, true)),
        "{d:#?}"
    );
    // Level 3: the potion entry is dropped.
    let d = found(
        r#""minecraft:behavior.drink_potion": {"priority": 1, "potions": [{"chance": 1.0}]}"#,
    );
    assert_eq!(
        told(
            &d,
            "minecraft:behavior.drink_potion/potions/0",
            "Drinkable potions must specify a valid potion id"
        ),
        Some((Severity::Error, true)),
        "{d:#?}"
    );
    // `behavior.work`'s legacy reader is a
    // schema reader, whose numeric check logs at level 3 and takes the default.
    let d = found(r#""minecraft:behavior.work": {"priority": 1, "active_time": -5}"#);
    assert_eq!(
        told(
            &d,
            "minecraft:behavior.work/active_time",
            "\"active_time\" on minecraft:behavior.work cannot be less than 0, value was -5, defaulting to DefinitionType::DEFAULT_ACTIVE_TIME"
        ),
        Some((Severity::Error, true)),
        "{d:#?}"
    );
    // Level 2: a name `MATERIAL_TYPE_MAP` lacks (case-sensitive) is Air.
    let d = found(
        r#""minecraft:behavior.lay_egg": {"priority": 1, "target_materials_above_block": ["Air", "water"]}"#,
    );
    assert_eq!(
        told(
            &d,
            "minecraft:behavior.lay_egg/target_materials_above_block/1",
            "Could not find specified material. Using Air."
        ),
        Some((Severity::Warning, false)),
        "{d:#?}"
    );
    // Level 2: `leader_filters` that does not read as a filter group.
    let d = found(
        r#""minecraft:behavior.follow_target_leader": {"priority": 1, "leader_filters": {"test": 5}}"#,
    );
    assert_eq!(
        told(
            &d,
            "minecraft:behavior.follow_target_leader/leader_filters",
            "'minecraft:follow_target_leader' | 'leader_filters' must specify filters to identify leader to follow."
        ),
        Some((Severity::Warning, false)),
        "{d:#?}"
    );
}

/// A beta file read on its own (`Content::parse`, which keeps only a stable entity) is refused
/// with "no parser available" for `beta`, told in its file, as every single-type read
/// refuses one.
#[test]
fn a_beta_file_read_on_its_own_is_refused_in_its_file() {
    use bedrock_addon_core::diagnostic::{DiagnosticCode, VanillaText};
    use bedrock_addon_core::pack_identity::{PackPath, PackRef};

    let mut context = cx();
    context.is_base_game_pack = true;
    let path = PackPath::new("entities/test.json").unwrap();
    let text = doc(
        "beta",
        r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
    );
    let refused = <Entity as Content>::parse(&path, text.as_bytes(), &context).unwrap_err();
    let refusal = refused
        .diagnostics
        .iter()
        .find(|d| matches!(d.code, DiagnosticCode::UnsupportedVersion { .. }))
        .unwrap_or_else(|| panic!("{:#?}", refused.diagnostics));
    assert!(
        matches!(
            &refusal.code,
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion { version })
            } if version == "beta"
        ),
        "{refusal:?}"
    );
    assert_eq!(refusal.location.pack, Some(PackRef::app_data()));
    assert_eq!(refusal.location.path, Some(path));
}

/// Vanilla's `is_experimental` is kept as written, whatever it holds (vanilla reads nothing
/// from it), and the document writes it back: a number where vanilla writes a bool too, and
/// through the upgrade of an older file. It lives in the document's unknown members, so the
/// document writes it and the bare value does not.
#[test]
fn is_experimental_holding_a_number_is_written_back_as_read() {
    let payload = r#"{ "description": { "identifier": "demo:x", "is_experimental": 7 },
        "components": {} }"#;
    for version in ["1.26.30", "1.26.60"] {
        let document = parse(&doc(version, payload)).unwrap();
        let written = json(&document.to_json().unwrap());
        let held = written
            .as_object()
            .and_then(|root| root.get("minecraft:entity"))
            .and_then(bedrock_addon_core::json::Value::as_object)
            .and_then(|payload| payload.get("description"))
            .and_then(bedrock_addon_core::json::Value::as_object)
            .and_then(|description| description.get("is_experimental"))
            .cloned();
        assert_eq!(held, Some(json("7")), "{version}: {written:?}");
    }
}
