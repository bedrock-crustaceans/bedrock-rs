//! The 27 components 1.26.60 types first,
//! read strictly in a 1.26.60 document and through the legacy rules below it, with the typed
//! defaults written back. The game's own test inputs are the reference: the
//! actor component definition server-test fixtures under `component/engine/` (`AC-n`).

use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
use bedrock_addon_core::document::Content;
use bedrock_addon_core::json::Value;

use super::super::common::{cx, doc, json, outcome, stable};

/// The text of the game's test fixture `AC-n`.
fn ac(n: u32) -> String {
    super::fixture(&format!("engine/case-{n}.json"))
}

/// A document at `version` whose `components` hold `key` with `value` (JSON text): the
/// game's snippet wrapper (`_make<X>ActorJson`).
fn wrapped(version: &str, key: &str, value: &str) -> String {
    doc(
        version,
        &format!(
            r#"{{"description": {{"identifier": "test:mob"}}, "components": {{"{key}": {value}}}}}"#
        ),
    )
}

/// The `components` of `text` as the newest struct writes them back, with the findings.
fn written_components(text: &str) -> (Value, Diagnostics) {
    // A refused document (an error at the typed version) has nothing
    // written back; an absence asserted on it would hold for any member, so it is a failure.
    let outcome = super::super::common::outcome(text, &cx());
    let Some(bedrock_addon_core::versioning::Loaded::Stable(_)) = &outcome.value else {
        panic!("the entity is refused: {:#?}", outcome.diagnostics);
    };
    let (entity, found) = stable(text, &cx());
    let written = json(&entity.to_json().expect("the entity writes"));
    let components = written
        .as_object()
        .and_then(|o| o.get("minecraft:entity"))
        .and_then(Value::as_object)
        .and_then(|e| e.get("components"))
        .cloned()
        .expect("components are written");
    (components, found)
}

/// Component `key` of `components`.
fn member(components: &Value, key: &str) -> Value {
    components
        .as_object()
        .and_then(|o| o.get(key))
        .cloned()
        .unwrap_or_else(|| panic!("{key} is written: {components:?}"))
}

/// The written component `key` of a wrapped document, asserting no warning or error.
fn written(version: &str, key: &str, value: &str) -> Value {
    let (components, found) = written_components(&wrapped(version, key, value));
    assert!(quiet(&found), "{version} {key} {value}: {found:#?}");
    member(&components, key)
}

/// Whether `found` has no warning and no error (an information, such as an unresolved
/// reference outside a pack, is allowed).
fn quiet(found: &Diagnostics) -> bool {
    !found.iter().any(|f| f.severity >= Severity::Warning)
}

/// Whether `found` has a finding of `code` at `severity` at a path ending in `tail`.
fn told(found: &Diagnostics, code: &str, severity: Severity, tail: &str) -> bool {
    found.iter().any(|f| {
        f.code.name() == code
            && f.severity == severity
            && f.location.json_path.to_string().ends_with(tail)
    })
}

// ---- the move controls, rail_movement, dash_action, buoyant ----

/// The V12660 template at 1.26.60: every
/// move control and the movement components read with the template's values, the members it
/// leaves out at their typed defaults.
#[test]
fn the_v12660_template_reads_the_move_controls_and_movement_components() {
    let (components, found) = written_components(&ac(1));
    assert!(quiet(&found), "{found:#?}");
    let expect =
        |key: &str, value: &str| assert_eq!(member(&components, key), json(value), "{key}");
    expect("minecraft:movement.basic", r#"{"max_turn": 45.0}"#);
    expect(
        "minecraft:movement.sway",
        r#"{"max_turn": 30.0, "sway_frequency": 0.1, "sway_amplitude": 0.2}"#,
    );
    for key in [
        "minecraft:movement.skip",
        "minecraft:movement.generic",
        "minecraft:movement.amphibious",
        "minecraft:movement.fly",
        "minecraft:movement.hover",
    ] {
        expect(key, r#"{"max_turn": 45.0}"#);
    }
    expect(
        "minecraft:movement.jump",
        r#"{"max_turn": 30.0, "jump_delay": {"min": 0.1, "max": 0.2}}"#,
    );
    expect(
        "minecraft:movement.glide",
        r#"{"max_turn": 30.0, "start_speed": 0.5, "speed_when_turning": 0.6}"#,
    );
    expect(
        "minecraft:dash_action",
        r#"{"cooldown_time": 2, "horizontal_momentum": 154.0, "vertical_momentum": 0.1,
            "can_dash_underwater": true, "direction": "passenger"}"#,
    );
    expect("minecraft:rail_movement", r#"{"max_speed": 0.5}"#);
}

/// The same template at 1.26.50: the
/// legacy read gives the same definitions; `jump_delay`'s `range_min` / `range_max` is the
/// typed `min` / `max` (the 1.26.60 builder writes `min` / `max`, Revision 1).
#[test]
fn the_v12660_template_at_1_26_50_gives_the_same_move_controls() {
    let (cereal, _) = written_components(&ac(1));
    let (legacy, found) = written_components(&ac(2));
    assert!(quiet(&found), "{found:#?}");
    for key in [
        "minecraft:movement.basic",
        "minecraft:movement.sway",
        "minecraft:movement.skip",
        "minecraft:movement.generic",
        "minecraft:movement.amphibious",
        "minecraft:movement.jump",
        "minecraft:movement.fly",
        "minecraft:movement.hover",
        "minecraft:movement.glide",
        "minecraft:dash_action",
        "minecraft:rail_movement",
    ] {
        assert_eq!(member(&legacy, key), member(&cereal, key), "{key}");
    }
}

/// The DEFAULT lines of the move controls, `rail_movement`, `dash_action` and `buoyant`
/// `{}` at 1.26.60 and at 1.26.50.
#[test]
fn an_empty_movement_component_takes_the_typed_defaults() {
    let defaults = [
        ("minecraft:movement.basic", r#"{"max_turn": 30.0}"#),
        ("minecraft:movement.amphibious", r#"{"max_turn": 30.0}"#),
        ("minecraft:movement.generic", r#"{"max_turn": 30.0}"#),
        ("minecraft:movement.fly", r#"{"max_turn": 30.0}"#),
        ("minecraft:movement.hover", r#"{"max_turn": 30.0}"#),
        ("minecraft:movement.skip", r#"{"max_turn": 30.0}"#),
        (
            "minecraft:movement.glide",
            r#"{"max_turn": 30.0, "start_speed": 0.1, "speed_when_turning": 0.2}"#,
        ),
        (
            "minecraft:movement.sway",
            r#"{"max_turn": 30.0, "sway_frequency": 0.5, "sway_amplitude": 0.05}"#,
        ),
        (
            "minecraft:movement.jump",
            r#"{"max_turn": 30.0, "jump_delay": {"min": 0.0, "max": 0.0}}"#,
        ),
        ("minecraft:rail_movement", r#"{"max_speed": 0.4}"#),
        (
            "minecraft:dash_action",
            r#"{"cooldown_time": 1.0, "horizontal_momentum": 1.0, "vertical_momentum": 1.0,
                "can_dash_underwater": false, "direction": "entity"}"#,
        ),
        (
            "minecraft:buoyant",
            r#"{"base_buoyancy": 1.0, "apply_gravity": true, "movement_type": "waves",
                "big_wave_probability": 0.03, "big_wave_speed": 10.0, "liquid_blocks": [],
                "drag_down_on_buoyancy_removed": 0.0, "can_auto_step_from_liquid": false}"#,
        ),
    ];
    for version in ["1.26.60", "1.26.50"] {
        for (key, expected) in defaults {
            assert_eq!(
                written(version, key, &ac(9)),
                json(expected),
                "{version} {key}"
            );
        }
    }
}

/// At 1.26.60 and its legacy twin at 1.26.50 (`BuoyancyTargetVersionDefinition_…`,
/// `BuoyancyLegacyVersionDefinition_…`): the typed values, `bobbing` the enum's 2.
#[test]
fn buoyant_reads_its_typed_values_at_the_target_and_below() {
    let expected = json(
        r#"{"base_buoyancy": 0.5, "apply_gravity": false, "movement_type": "bobbing",
            "big_wave_probability": 0.8, "big_wave_speed": 5.2, "liquid_blocks": ["minecraft:water"],
            "drag_down_on_buoyancy_removed": 0.7, "can_auto_step_from_liquid": true}"#,
    );
    for version in ["1.26.60", "1.26.50"] {
        assert_eq!(
            written(version, "minecraft:buoyant", &ac(10)),
            expected,
            "{version}"
        );
    }
}

/// Below the target: the
/// definition loads, `movement_type` keeps `waves`, and the member is told; at the target
/// the member is an error.
#[test]
fn an_invalid_buoyant_movement_type_is_waves_below_the_target_and_an_error_at_it() {
    let (components, found) = written_components(&wrapped("1.26.50", "minecraft:buoyant", &ac(11)));
    assert_eq!(
        member(&components, "minecraft:buoyant")
            .as_object()
            .and_then(|o| o.get("movement_type"))
            .cloned(),
        Some(json(r#""waves""#))
    );
    // The legacy reader's string node takes the name and the game logs no text for it
    // here: the crate's own finding is an information.
    assert!(
        told(&found, "InvalidValue", Severity::Info, "movement_type"),
        "{found:#?}"
    );
    let found = outcome(&wrapped("1.26.60", "minecraft:buoyant", &ac(11)), &cx()).diagnostics;
    assert!(
        found.iter().any(|f| f.severity == Severity::Error
            && f.location.json_path.to_string().ends_with("movement_type")),
        "{found:#?}"
    );
}

/// Whether every member of `expected` (an object) is in `actual` with the same value, at
/// every level of nested objects; lists are compared whole.
fn has_members(actual: &Value, expected: &Value) -> bool {
    match (actual.as_object(), expected.as_object()) {
        (Some(actual), Some(expected)) => expected.iter().all(|(key, value)| {
            actual
                .get(key.as_str())
                .is_some_and(|found| has_members(found, value))
        }),
        _ => actual == expected,
    }
}

/// Asserts that the written `key` of a wrapped document at `version` holds the members of
/// `expected` (JSON text), with no warning or error.
fn assert_members(version: &str, key: &str, value: &str, expected: &str) {
    let found = written(version, key, value);
    assert!(
        has_members(&found, &json(expected)),
        "{version} {key}: {found:?} lacks {expected}"
    );
}

/// The member at `path` (keys joined by `/`) of the written component `key`.
fn at(component: &Value, path: &str) -> Option<Value> {
    let mut node = component.clone();
    for key in path.split('/') {
        node = if let Ok(i) = key.parse::<usize>() {
            node.as_array()?.get(i)?.clone()
        } else {
            node.as_object()?.get(key)?.clone()
        };
    }
    Some(node)
}

/// An error of code `code` whose path ends in `tail` is told for the document at `version`
/// (`InvalidType` is a value of the wrong JSON type; `InvalidValue` a value
/// outside the typed reader's constraint; `InvalidEnumValue` a name outside the enum).
fn assert_error_at(version: &str, key: &str, value: &str, tail: &str, code: &str) {
    let found = outcome(&wrapped(version, key, value), &cx()).diagnostics;
    assert!(
        found.iter().any(|f| f.severity == Severity::Error
            && f.code.name() == code
            && f.location.json_path.to_string().ends_with(tail)),
        "{version} {key} {value}: no {code} error at {tail}: {found:#?}"
    );
}

// ---- attack, spell_effects, strength, healable, teleport, trail, transformation, timer ----

/// At 1.26.60 and the
/// defaults of (`…_UsesDefaults`): damage 0..0, no effect, duration 0, amplifier 0.
#[test]
fn attack_reads_its_typed_values_and_defaults() {
    assert_members(
        "1.26.60",
        "minecraft:attack",
        &ac(23),
        r#"{"damage": {"min": 4, "max": 4}, "effect_name": "poison", "effect_duration": 2.5,
            "effect_amplifier": 3}"#,
    );
    for version in ["1.26.60", "1.26.50"] {
        assert_members(
            version,
            "minecraft:attack",
            &ac(9),
            r#"{"damage": {"min": 0, "max": 0}, "effect_name": "", "effect_duration": 0,
                "effect_amplifier": 0}"#,
        );
    }
}

/// At 1.26.50: a scalar
/// `damage` is min = max; `effect_amplifier` is read by the legacy parser too. at the
/// target (`…_RejectsWrongValueType`): `damage: "strong"` is an error at `damage`.
#[test]
fn attack_takes_a_scalar_damage_below_the_target_and_refuses_a_string_at_it() {
    let found = written("1.26.50", "minecraft:attack", &ac(25));
    assert!(
        has_members(
            &found,
            &json(r#"{"effect_name": "hunger", "effect_duration": 3.5, "effect_amplifier": 2}"#)
        ),
        "{found:?}"
    );
    let damage = at(&found, "damage").expect("damage");
    assert!(
        damage == json("5") || damage == json(r#"{"min": 5, "max": 5}"#),
        "{damage:?}"
    );
    assert_error_at(
        "1.26.60",
        "minecraft:attack",
        &ac(24),
        "damage",
        "InvalidType",
    );
}

/// At 1.26.60 and at 1.26.50 (`MobEffect…`): the six effect members; the legacy
/// `"duration": "infinite"` is kept. at the target: `add_effects: "speed"` is an error.
#[test]
fn spell_effects_read_their_effects_and_an_infinite_duration() {
    assert_members(
        "1.26.60",
        "minecraft:spell_effects",
        &ac(26),
        r#"{"add_effects": [{"effect": "speed", "duration": 2.5, "amplifier": 3, "ambient": true,
            "visible": false, "display_on_screen_animation": true}], "remove_effects": ["slowness"]}"#,
    );
    assert_members(
        "1.26.50",
        "minecraft:spell_effects",
        &ac(28),
        r#"{"add_effects": [{"effect": "regeneration", "duration": "infinite", "amplifier": 2,
            "ambient": true, "visible": false, "display_on_screen_animation": true}],
            "remove_effects": ["poison"]}"#,
    );
    for version in ["1.26.60", "1.26.50"] {
        assert_members(
            version,
            "minecraft:spell_effects",
            &ac(9),
            r#"{"add_effects": [], "remove_effects": []}"#,
        );
    }
    assert_error_at(
        "1.26.60",
        "minecraft:spell_effects",
        &ac(27),
        "add_effects",
        "InvalidType",
    );
}

/// An effect of `add_effects` takes its defaults (amplifier 0, ambient false, visible true, no
/// animation, duration 0), and a single string `remove_effects` is its one element below the
/// target (vanilla's 1.15 `zombie_villager`), refused at it.
#[test]
fn a_spell_effect_takes_its_defaults_and_a_lone_removed_effect_is_a_list() {
    assert_members(
        "1.26.60",
        "minecraft:spell_effects",
        r#"{"add_effects": [{"effect": "speed"}]}"#,
        r#"{"add_effects": [{"effect": "speed", "duration": 0, "amplifier": 0, "ambient": false,
            "visible": true, "display_on_screen_animation": false}]}"#,
    );
    assert_members(
        "1.26.50",
        "minecraft:spell_effects",
        r#"{"add_effects": [{"duration": 100, "effect": "strength"}], "remove_effects": "weakness"}"#,
        r#"{"remove_effects": ["weakness"]}"#,
    );
    // The typed reader refuses it.
    assert_error_at(
        "1.26.60",
        "minecraft:spell_effects",
        r#"{"remove_effects": "bad_omen"}"#,
        "remove_effects",
        "InvalidType",
    );
}

/// A transformation sound written as one name is its list below 1.26.60 and an error at the
/// typed version (`expected an array`).
#[test]
fn a_lone_transformation_sound_is_a_list_only_below_the_target() {
    assert_members(
        "1.26.50",
        "minecraft:transformation",
        r#"{"transformation_sound": "unfect"}"#,
        r#"{"transformation_sound": ["unfect"]}"#,
    );
    assert_error_at(
        "1.26.60",
        "minecraft:transformation",
        r#"{"transformation_sound": "unfect"}"#,
        "transformation_sound",
        "InvalidType",
    );
}

/// The V12660 template's `strength` (value 7, max 12) and the defaults 1 and 5.
#[test]
fn strength_reads_its_values_and_defaults() {
    let (components, _) = written_components(&ac(1));
    assert_eq!(
        member(&components, "minecraft:strength"),
        json(r#"{"value": 7, "max": 12}"#)
    );
    for version in ["1.26.60", "1.26.50"] {
        assert_eq!(
            written(version, "minecraft:strength", &ac(9)),
            json(r#"{"value": 1, "max": 5}"#),
            "{version}"
        );
    }
}

/// At 1.26.60: an
/// item as a string or `{name}`, its `result_item`, `heal_amount`, `effects` (duration
/// `"infinite"` kept as text), `force_use`, `filters`; the defaults.
#[test]
fn healable_reads_its_items_and_defaults() {
    let found = written("1.26.60", "minecraft:healable", &ac(32));
    assert_eq!(
        at(&found, "items/0/item"),
        Some(json(r#""minecraft:apple""#))
    );
    assert_eq!(at(&found, "items/0/heal_amount"), Some(json("1")));
    assert_eq!(at(&found, "items/1/heal_amount"), Some(json("4")));
    assert_eq!(
        at(&found, "items/1/result_item"),
        Some(json(r#""minecraft:stick""#))
    );
    assert_eq!(
        at(&found, "items/1/effects"),
        Some(json(
            r#"[{"name": "regeneration", "duration": "infinite", "amplifier": 2, "chance": 0.25}]"#
        ))
    );
    assert_eq!(at(&found, "force_use"), Some(json("true")));
    assert_eq!(
        at(&found, "filters"),
        Some(json(
            r#"{"test": "is_family", "subject": "self", "value": "player"}"#
        ))
    );
    for version in ["1.26.60", "1.26.50"] {
        assert_members(
            version,
            "minecraft:healable",
            &ac(9),
            r#"{"items": [], "force_use": false}"#,
        );
    }
    // An effect's defaults: amplifier 1, chance 1.0, duration 0.
    assert_members(
        "1.26.60",
        "minecraft:healable",
        r#"{"items": [{"item": "minecraft:apple", "effects": [{"name": "speed"}]}]}"#,
        r#"{"items": [{"item": "minecraft:apple", "heal_amount": 1, "effects": [{"name": "speed", "duration": 0, "amplifier": 1, "chance": 1.0}]}]}"#,
    );
}

/// The singleton `effects` object is an error at the target
/// and one effect below it; a string `items` entry is the item below the target;
/// the ordinary form is read below the target.
#[test]
fn healable_takes_a_lone_effect_and_string_items_only_below_the_target() {
    assert_error_at(
        "1.26.60",
        "minecraft:healable",
        &ac(33),
        "effects",
        "InvalidType",
    );
    let lone = written("1.26.50", "minecraft:healable", &ac(33));
    assert_eq!(
        at(&lone, "items/0/effects/0/name"),
        Some(json(r#""regeneration""#))
    );
    assert_eq!(
        at(&lone, "items/0/effects").and_then(|e| e.as_array().map(|a| a.len())),
        Some(1)
    );
    let strings = written("1.26.50", "minecraft:healable", &ac(34));
    assert_eq!(
        at(&strings, "items/0/item"),
        Some(json(r#""minecraft:apple""#))
    );
    let legacy = written("1.26.50", "minecraft:healable", &ac(35));
    assert_eq!(at(&legacy, "items/0/heal_amount"), Some(json("3")));
    assert_eq!(
        at(&legacy, "items/0/effects/0/duration"),
        Some(json(r#""infinite""#))
    );
    assert_eq!(at(&legacy, "force_use"), Some(json("true")));
}

/// The V12660 template's `teleport` with `teleports_on_projectile_hit` ("TeleportComponentDefinition
/// should parse teleports_on_projectile_hit"), and the DEFAULT lines (light and dark 0.01).
#[test]
fn teleport_reads_its_values_and_defaults() {
    let (components, _) = written_components(&ac(1));
    let teleport = member(&components, "minecraft:teleport");
    assert!(
        has_members(
            &teleport,
            &json(
                r#"{"random_teleports": false, "teleports_on_projectile_hit": true,
                    "min_random_teleport_time": 2.5, "max_random_teleport_time": 8.0,
                    "random_teleport_cube": [12, 8, 10], "target_distance": 20.0,
                    "target_teleport_chance": 0.75, "light_teleport_chance": 0.2,
                    "dark_teleport_chance": 0.4}"#
            )
        ),
        "{teleport:?}"
    );
    for version in ["1.26.60", "1.26.50"] {
        assert_members(
            version,
            "minecraft:teleport",
            &ac(9),
            r#"{"random_teleports": true, "teleports_on_projectile_hit": false,
                "min_random_teleport_time": 0, "max_random_teleport_time": 20,
                "random_teleport_cube": [32, 16, 32], "target_distance": 16,
                "target_teleport_chance": 1, "light_teleport_chance": 0.01,
                "dark_teleport_chance": 0.01}"#,
        );
    }
}

/// The 1.26.60 legacy reader of teleport binds the snake_case names only, and the game knows
/// none of `randomTeleports` … `darkTeleportChance` but `lightTeleportChance`, which only the
/// 1.2 enderman fixer reads. Below 1.26.60 each camel-case spelling is an unknown
/// member (a kept error, the component at its defaults); at 1.26.60 the typed read refuses it.
#[test]
fn teleport_s_camel_case_spellings_are_no_members() {
    let value = r#"{"randomTeleports": false, "minRandomTeleportTime": 1, "maxRandomTeleportTime": 2,
            "randomTeleportCube": [1, 2, 3], "targetDistance": 4, "lightTeleportChance": 0.5,
            "darkTeleportChance": 0.6}"#;
    let (components, found) = written_components(&wrapped("1.26.50", "minecraft:teleport", value));
    let (defaults, _) = written_components(&wrapped("1.26.50", "minecraft:teleport", "{}"));
    assert_eq!(
        member(&components, "minecraft:teleport"),
        member(&defaults, "minecraft:teleport")
    );
    let lines: Vec<String> = found
        .iter()
        .filter(|f| f.kept)
        .map(|f| f.code.to_string())
        .collect();
    assert_eq!(
        lines,
        [
            "randomTeleports",
            "minRandomTeleportTime",
            "maxRandomTeleportTime",
            "randomTeleportCube",
            "targetDistance",
            "lightTeleportChance",
            "darkTeleportChance"
        ]
        .map(|name| format!("child '{name}' not valid here."))
    );
    let value = r#"{"lightTeleportChance": 0.5}"#;
    let refused = outcome(&wrapped("1.26.60", "minecraft:teleport", value), &cx());
    assert!(refused.value.is_none());
    assert_error_at(
        "1.26.60",
        "minecraft:teleport",
        value,
        "minecraft:teleport/lightTeleportChance",
        "UnrecognizedField",
    );
}

/// At 1.26.60: `radius` 2 and
/// two replaceable blocks; at 1.26.40 the legacy reader ignores `radius` and `replaceable_blocks` (radius −1, no
/// list), told as an information since the write-back leaves them out. The defaults.
#[test]
fn trail_reads_its_new_members_at_the_target_and_ignores_them_below() {
    assert_members(
        "1.26.60",
        "minecraft:trail",
        &ac(14),
        r#"{"radius": 2, "replaceable_blocks": ["minecraft:water", "minecraft:air"]}"#,
    );
    let (components, found) = written_components(&wrapped("1.26.40", "minecraft:trail", &ac(15)));
    let trail = member(&components, "minecraft:trail");
    assert_eq!(at(&trail, "radius"), Some(json("-1")));
    assert_eq!(at(&trail, "replaceable_blocks"), Some(json("[]")));
    assert!(quiet(&found), "{found:#?}");
    assert!(
        told(&found, "DroppedField", Severity::Info, "radius"),
        "{found:#?}"
    );
    assert!(
        told(&found, "DroppedField", Severity::Info, "replaceable_blocks"),
        "{found:#?}"
    );
    for version in ["1.26.60", "1.26.50"] {
        assert_members(
            version,
            "minecraft:trail",
            &ac(9),
            r#"{"block_type": "air", "spawn_offset": [0, 0, 0], "replaceable_blocks": [],
                "radius": -1, "force_replace_matched_blocks": false}"#,
        );
    }
}

/// `trail.radius` is within −1 ..= 16 (`BasicNumericConstraint<int>::min(-1).max(16)`).
#[test]
fn a_trail_radius_outside_its_bounds_is_an_error_at_the_target() {
    assert_error_at(
        "1.26.60",
        "minecraft:trail",
        r#"{"radius": 17}"#,
        "radius",
        "InvalidValue",
    );
    assert_error_at(
        "1.26.60",
        "minecraft:trail",
        r#"{"radius": -2}"#,
        "radius",
        "InvalidValue",
    );
}

/// The V12650 template's `transformation` ( at 1.26.50, at 1.26.40: delay 15.0): the
/// legacy reader takes a number `delay` as its `value`; the structured delay and `add` at the
/// target, and the defaults.
#[test]
fn transformation_reads_a_number_delay_below_the_target_and_its_structure_at_it() {
    for n in [3, 4] {
        let (components, found) = written_components(&ac(n));
        assert!(quiet(&found), "AC-{n}: {found:#?}");
        let transformation = member(&components, "minecraft:transformation");
        assert_eq!(
            at(&transformation, "delay/value"),
            Some(json("15.0")),
            "AC-{n}"
        );
    }
    assert_members(
        "1.26.60",
        "minecraft:transformation",
        r#"{"into": "minecraft:villager<minecraft:spawn>", "begin_transform_sound": ["remedy"],
            "transformation_sound": ["unfect"], "keep_level": true,
            "add": {"component_groups": ["demo:a"]},
            "delay": {"value": 100, "block_assist_chance": 0.01, "block_radius": 4,
                      "block_chance": 0.3, "block_types": ["minecraft:bed"]}}"#,
        r#"{"into": "minecraft:villager<minecraft:spawn>", "keep_level": true,
            "add": {"component_groups": ["demo:a"]}, "delay": {"value": 100, "block_radius": 4}}"#,
    );
    for version in ["1.26.60", "1.26.50"] {
        assert_members(
            version,
            "minecraft:transformation",
            &ac(9),
            r#"{"drop_equipment": false, "drop_inventory": false, "preserve_equipment": false,
                "keep_owner": false, "keep_level": false,
                "delay": {"value": 0, "range_min": 0, "range_max": 0, "block_assist_chance": 0,
                          "block_radius": 0, "block_max": 0, "block_chance": 0, "block_types": []}}"#,
        );
    }
}

/// `transformation.delay`'s constraints: `value`, `block_radius`, `block_max` at least 0, the two
/// chances within 0 ..= 1.
#[test]
fn a_transformation_delay_outside_its_bounds_is_an_error_at_the_target() {
    assert_error_at(
        "1.26.60",
        "minecraft:transformation",
        r#"{"delay": {"block_chance": 2}}"#,
        "block_chance",
        "InvalidValue",
    );
    assert_error_at(
        "1.26.60",
        "minecraft:transformation",
        r#"{"delay": {"value": -1}}"#,
        "value",
        "InvalidValue",
    );
}

/// The V12660 template's `timer` at 1.26.60 and at 1.26.50 (`time` written `range_min` /
/// `range_max` there): the same definition; the defaults (`looping` and `randomInterval` true,
/// no `time`, no choices).
#[test]
fn timer_reads_its_values_at_the_target_and_below() {
    let (cereal, _) = written_components(&ac(1));
    let (legacy, _) = written_components(&ac(2));
    let timer = member(&cereal, "minecraft:timer");
    assert!(
        has_members(
            &timer,
            &json(
                r#"{"looping": false, "randomInterval": true, "time": {"min": 1, "max": 3},
                    "time_down_event": {"event": "minecraft:target_far_enough", "target": "self"},
                    "random_time_choices": []}"#
            )
        ),
        "{timer:?}"
    );
    assert_eq!(member(&legacy, "minecraft:timer"), timer);
    for version in ["1.26.60", "1.26.50"] {
        let found = written(version, "minecraft:timer", &ac(9));
        assert!(
            has_members(
                &found,
                &json(r#"{"looping": true, "randomInterval": true, "random_time_choices": []}"#)
            ),
            "{version}: {found:?}"
        );
        assert_eq!(at(&found, "time"), None, "{version}");
    }
}

/// `timer.random_time_choices`: each a `value` and a `weight`.
#[test]
fn timer_reads_weighted_time_choices() {
    assert_members(
        "1.26.60",
        "minecraft:timer",
        r#"{"random_time_choices": [{"value": 1, "weight": 3}, {"value": 2.5, "weight": 1}]}"#,
        r#"{"random_time_choices": [{"value": 1, "weight": 3}, {"value": 2.5, "weight": 1}]}"#,
    );
}

// ---- A null legacy member at the component ----

/// A legacy spelling written as `null` that the lenient read renames is absent, and its
/// information is told where the file has it (the projectile's legacy aliases; teleport's
/// camel-case spellings are no members of its 1.26.60 legacy reader).
#[test]
fn a_renamed_legacy_member_written_null_is_told_where_the_file_has_it() {
    for (version, key, value, written_at) in [
        (
            "1.26.40",
            "minecraft:projectile",
            r#"{"onFireTime": null}"#,
            "minecraft:projectile/onFireTime",
        ),
        // Renamed by the 1.26.30 → 1.26.40 step before the typed read.
        (
            "1.26.30",
            "minecraft:projectile",
            r#"{"onFireTime": null}"#,
            "minecraft:projectile/onFireTime",
        ),
    ] {
        let (components, found) = written_components(&wrapped(version, key, value));
        assert!(quiet(&found), "{key}: {found:#?}");
        let notes: Vec<String> = found
            .iter()
            .filter(|f| f.code.name() == "ValueRewritten")
            .map(|f| f.location.json_path.to_string())
            .collect();
        assert_eq!(notes.len(), 1, "{key}: {found:#?}");
        assert!(notes[0].ends_with(written_at), "{key}: {notes:?}");
        assert!(member(&components, key).as_object().is_some(), "{key}");
    }
}

/// A typed component whose value is itself `null` is absent: a member written `null` is read as not found, so the
/// entity loads without it, at the target version and below, with an information where the file has it.
#[test]
fn a_typed_component_written_null_is_absent() {
    for (version, key) in [
        ("1.26.60", "minecraft:buoyant"),
        ("1.26.50", "minecraft:buoyant"),
        ("1.26.50", "minecraft:insomnia"),
        ("1.26.40", "minecraft:insomnia"),
    ] {
        let (components, found) = written_components(&wrapped(version, key, "null"));
        assert!(quiet(&found), "{version} {key}: {found:#?}");
        assert!(
            components.as_object().is_some_and(|c| c.get(key).is_none()),
            "{version} {key}: {components:?}"
        );
        assert!(
            found.iter().any(|f| f.code.name() == "ValueRewritten"
                && f.location.json_path.to_string().ends_with(key)),
            "{version} {key}: {found:#?}"
        );
    }
}

// ---- inventory, shareables, economy_trade_table, rideable, scale_by_age,
// pushable_by_block, annotation.break_door ----

/// At 1.26.60 and at 1.26.50 (`ContainerTargetVersionDefinition_…`,
/// `ContainerLegacyVersionDefinition_…`); the defaults (`container_type` "none", 5
/// slots); at the target: `inventory_size: "large"` is an error.
#[test]
fn inventory_reads_its_values_and_defaults() {
    assert_eq!(
        written("1.26.60", "minecraft:inventory", &ac(29)),
        json(&ac(29))
    );
    assert_eq!(
        written("1.26.50", "minecraft:inventory", &ac(31)),
        json(&ac(31))
    );
    for version in ["1.26.60", "1.26.50"] {
        assert_eq!(
            written(version, "minecraft:inventory", &ac(9)),
            json(
                r#"{"container_type": "none", "inventory_size": 5, "can_be_siphoned_from": false,
                    "private": false, "restrict_to_owner": false, "additional_slots_per_strength": 0}"#
            ),
            "{version}"
        );
    }
    assert_error_at(
        "1.26.60",
        "minecraft:inventory",
        &ac(30),
        "inventory_size",
        "InvalidType",
    );
}

/// The V12660 template's `shareables` (two items) and the defaults: −1 amounts, priority 0,
/// every flag false.
#[test]
fn shareables_read_their_items_and_defaults() {
    let (components, _) = written_components(&ac(1));
    let shareables = member(&components, "minecraft:shareables");
    assert!(
        has_members(
            &shareables,
            &json(
                r#"{"singular_pickup": true, "all_items": true, "all_items_max_amount": 1,
                      "all_items_want_amount": -1, "all_items_surplus_amount": -1}"#
            )
        ),
        "{shareables:?}"
    );
    assert_eq!(
        at(&shareables, "items/0/item"),
        Some(json(r#""minecraft:is_food""#))
    );
    assert_eq!(at(&shareables, "items/1/max_amount"), Some(json("1")));
    assert_members(
        "1.26.60",
        "minecraft:shareables",
        r#"{"items": [{"item": "minecraft:apple"}]}"#,
        r#"{"items": [{"item": "minecraft:apple", "want_amount": -1, "surplus_amount": -1,
            "max_amount": -1, "pickup_limit": -1, "craft_into": "", "priority": 0,
            "admire": false, "barter": false, "stored_in_inventory": false,
            "consume_item": false, "pickup_only": false}]}"#,
    );
    for version in ["1.26.60", "1.26.50"] {
        assert_eq!(
            written(version, "minecraft:shareables", &ac(9)),
            json(
                r#"{"singular_pickup": false, "all_items": false, "all_items_want_amount": -1,
                    "all_items_surplus_amount": -1, "all_items_max_amount": -1, "items": []}"#
            ),
            "{version}"
        );
    }
}

/// The game's 1.26.50 twin of the V12660 template loads it with an empty content log: no kept error, and nothing else above an information.
#[test]
fn the_v12660_template_at_1_26_50_loads_with_no_kept_error() {
    let (_, found) = written_components(&ac(2));
    assert!(!found.iter().any(|f| f.kept), "{found:#?}");
    assert!(
        !found.iter().any(|f| f.severity >= Severity::Warning),
        "{found:#?}"
    );
}

/// The V12660 template's `economy_trade_table` at 1.26.60 and at 1.26.50 (every
/// value asserted by the game's test), and the DEFAULT lines.
#[test]
fn economy_trade_table_reads_its_values_and_defaults() {
    let (cereal, _) = written_components(&ac(1));
    let (legacy, _) = written_components(&ac(2));
    let table = member(&cereal, "minecraft:economy_trade_table");
    assert!(
        has_members(
            &table,
            &json(
                r#"{"display_name": "entity.villager.toolsmith", "table": "trading/tool_smith_trades.json",
                    "new_screen": true, "hero_demand_discount": -6,
                    "cured_discount": {"min": -30, "max": -24}, "nearby_cured_discount": -18,
                    "persist_trades": true, "convert_trades_economy": true,
                    "show_trade_screen": false, "use_legacy_price_formula": true}"#
            )
        ),
        "{table:?}"
    );
    // At 1.26.50 the legacy range reader reads an
    // object's `range_min` / `range_max` only, so the template's `{min, max}` ranges keep their
    // defaults there; every other member is the 1.26.60 read's.
    let mut legacy_table = member(&legacy, "minecraft:economy_trade_table");
    let mut typed_table = table.clone();
    for (one, other) in [(&mut legacy_table, &mut typed_table)] {
        for key in ["cured_discount", "max_cured_discount"] {
            assert!(one.as_object_mut().and_then(|o| o.remove(key)).is_some());
            assert!(other.as_object_mut().and_then(|o| o.remove(key)).is_some());
        }
    }
    assert_eq!(legacy_table, typed_table);
    assert_members(
        "1.26.60",
        "minecraft:economy_trade_table",
        &ac(9),
        r#"{"display_name": "", "table": "", "new_screen": false, "hero_demand_discount": -4,
            "nearby_cured_discount": -20, "max_nearby_cured_discount": -200,
            "persist_trades": false, "convert_trades_economy": false, "show_trade_screen": true,
            "use_legacy_price_formula": false}"#,
    );
    let defaults = written("1.26.60", "minecraft:economy_trade_table", &ac(9));
    for key in ["cured_discount", "max_cured_discount"] {
        let range = at(&defaults, key).expect(key);
        assert!(
            range == json("[-25, -20]") || range == json(r#"{"min": -25, "max": -20}"#),
            "{key}: {range:?}"
        );
    }
}

/// the typed default of `use_legacy_price_formula` is `false`, while the crate's
/// 1.16.0 step keeps writing `true` into an older document that has none:
/// both hold, each for its documents.
#[test]
fn use_legacy_price_formula_is_false_typed_and_true_for_an_old_document() {
    assert_eq!(
        at(
            &written("1.26.60", "minecraft:economy_trade_table", "{}"),
            "use_legacy_price_formula"
        ),
        Some(json("false"))
    );
    let (components, _) = written_components(&wrapped(
        "1.15.0",
        "minecraft:economy_trade_table",
        r#"{"table": "trading/economy_trades/farmer_trades.json"}"#,
    ));
    assert_eq!(
        at(
            &member(&components, "minecraft:economy_trade_table"),
            "use_legacy_price_formula"
        ),
        Some(json("true"))
    );
}

/// The V12660 template's `rideable` and vanilla 1.26.60 `cushion.json`'s (`dismount_mode:
/// "on_top_center"`, `rotate_rider_by` −90), the defaults, and a seat's defaults.
#[test]
fn rideable_reads_its_values_and_defaults() {
    let (components, found) = written_components(&ac(1));
    assert!(quiet(&found), "{found:#?}");
    let rideable = member(&components, "minecraft:rideable");
    assert!(
        has_members(
            &rideable,
            &json(
                r#"{"seat_count": 1, "crouching_skip_interact": true, "family_types": ["player"],
                      "interact_text": "action.interact.ride"}"#
            )
        ),
        "{rideable:?}"
    );
    assert_eq!(
        at(&rideable, "seats/0/position"),
        Some(json("[0.0, 1.7, -0.2]"))
    );
    assert_members(
        "1.26.60",
        "minecraft:rideable",
        r#"{"seat_count": 1, "interact_text": "action.interact.ride.cushion",
            "dismount_mode": "on_top_center",
            "seats": [{"position": [0.0, 0.0625, 0.0], "rotate_rider_by": -90}]}"#,
        r#"{"dismount_mode": "on_top_center",
            "seats": [{"position": [0.0, 0.0625, 0.0], "rotate_rider_by": -90,
                       "min_rider_count": 0, "max_rider_count": 0, "lock_rider_rotation": 181}]}"#,
    );
    for version in ["1.26.60", "1.26.50"] {
        assert_members(
            version,
            "minecraft:rideable",
            &ac(9),
            r#"{"seat_count": 1, "controlling_seat": 0, "crouching_skip_interact": true,
                "pull_in_entities": false, "rider_can_interact": false, "passenger_max_width": 0,
                "family_types": [], "interact_text": "", "dismount_mode": "default",
                "on_rider_enter_event": "", "on_rider_exit_event": "", "seats": []}"#,
        );
    }
    assert_error_at(
        "1.26.60",
        "minecraft:rideable",
        r#"{"dismount_mode": "sideways"}"#,
        "dismount_mode",
        "InvalidEnumValue",
    );
}

/// Below 1.26.60 a single `seats` object is the one seat (vanilla's older files write it so,
/// 708 times in the corpus); a `rotate_rider_by` expression is kept.
#[test]
fn a_lone_seat_object_is_one_seat_below_the_target() {
    let found = written(
        "1.26.50",
        "minecraft:rideable",
        r#"{"seat_count": 1, "family_types": ["zombie"],
            "seats": {"position": [0, 1.1, -0.35], "rotate_rider_by": "query.is_baby ? 90 : 0"}}"#,
    );
    assert_eq!(
        at(&found, "seats/0/position"),
        Some(json("[0, 1.1, -0.35]"))
    );
    assert_eq!(
        at(&found, "seats/0/rotate_rider_by"),
        Some(json(r#""query.is_baby ? 90 : 0""#))
    );
}

/// The two seat camera members are optional and bound by `range(1, 64)` and `range(1, 32)` at
/// the typed version (`SeatLimits`).
#[test]
fn a_seat_camera_member_outside_its_range_is_an_error_at_the_target() {
    let found = written(
        "1.26.60",
        "minecraft:rideable",
        r#"{"seats": [{"third_person_camera_radius": 7}]}"#,
    );
    assert_eq!(
        at(&found, "seats/0/third_person_camera_radius"),
        Some(json("7"))
    );
    assert_eq!(at(&found, "seats/0/camera_relax_distance_smoothing"), None);
    assert_error_at(
        "1.26.60",
        "minecraft:rideable",
        r#"{"seats": [{"third_person_camera_radius": 65}]}"#,
        "third_person_camera_radius",
        "InvalidValue",
    );
    assert_error_at(
        "1.26.60",
        "minecraft:rideable",
        r#"{"seats": [{"camera_relax_distance_smoothing": 0.5}]}"#,
        "camera_relax_distance_smoothing",
        "InvalidValue",
    );
}

/// The V12660 template's `scale_by_age` (start 0.5, end 1.0) and the defaults 1 and 1.
#[test]
fn scale_by_age_reads_its_values_and_defaults() {
    let (components, _) = written_components(&ac(1));
    assert_eq!(
        member(&components, "minecraft:scale_by_age"),
        json(r#"{"start_scale": 0.5, "end_scale": 1.0}"#)
    );
    for version in ["1.26.60", "1.26.50"] {
        assert_eq!(
            written(version, "minecraft:scale_by_age", &ac(9)),
            json(r#"{"start_scale": 1.0, "end_scale": 1.0}"#),
            "{version}"
        );
    }
}

/// `pushable_by_block` stays memberless: a member written into it is not one of it.
#[test]
fn pushable_by_block_is_memberless() {
    for version in ["1.26.60", "1.26.50"] {
        assert_eq!(
            written(version, "minecraft:pushable_by_block", "{}"),
            json("{}")
        );
    }
    // At the typed version the member refuses the entity; below it the legacy reader's
    // schema has no child (from 1.15.0): the game logs "child 'unexpected_field' not
    // valid here." at error level and loads the entity, the component at its defaults.
    let value = r#"{"unexpected_field": true}"#;
    let refused = outcome(
        &wrapped("1.26.60", "minecraft:pushable_by_block", value),
        &cx(),
    );
    assert!(refused.value.is_none());
    assert_error_at(
        "1.26.60",
        "minecraft:pushable_by_block",
        value,
        "minecraft:pushable_by_block/unexpected_field",
        "UnrecognizedField",
    );
    let (components, found) =
        written_components(&wrapped("1.26.50", "minecraft:pushable_by_block", value));
    assert!(
        found.iter().any(|f| f.kept
            && f.code.to_string() == "child 'unexpected_field' not valid here."
            && f.location
                .json_path
                .to_string()
                .ends_with("unexpected_field")),
        "{found:#?}"
    );
    assert!(
        found
            .iter()
            .all(|f| f.severity != Severity::Error || f.kept),
        "{found:#?}"
    );
    assert_eq!(
        member(&components, "minecraft:pushable_by_block"),
        json("{}")
    );
}

/// At 1.26.60 (3.5 seconds, `normal` = 2) and at 1.26.50 (5.0, `easy`); the
/// defaults 12 and `hard` (`DEFAULT_BREAK_TIME`, `DEFAULT_MIN_DIFFICULTY`); at the
/// target: `break_time: "fast"` is an error.
#[test]
fn annotation_break_door_reads_its_values_and_defaults() {
    assert_eq!(
        written("1.26.60", "minecraft:annotation.break_door", &ac(20)),
        json(r#"{"break_time": 3.5, "min_difficulty": "normal"}"#)
    );
    assert_eq!(
        written("1.26.50", "minecraft:annotation.break_door", &ac(22)),
        json(r#"{"break_time": 5.0, "min_difficulty": "easy"}"#)
    );
    for version in ["1.26.60", "1.26.50"] {
        assert_eq!(
            written(version, "minecraft:annotation.break_door", &ac(9)),
            json(r#"{"break_time": 12.0, "min_difficulty": "hard"}"#),
            "{version}"
        );
    }
    assert_error_at(
        "1.26.60",
        "minecraft:annotation.break_door",
        &ac(21),
        "break_time",
        "InvalidType",
    );
}

// ---- the upgrade-time conflicts ----

/// `teleport.light_teleport_chance`: the typed default 0.01, and an older enderman's
/// `lightTeleportChance` of 0.01 stays 0.01. The game's rewrite of it to 0.05 is a fixer op of
/// the 1.2.0 gate, below the crate's floor (documented, not implemented), so
/// no document the crate reads meets it.
#[test]
fn an_older_enderman_s_light_teleport_chance_is_kept() {
    let text = doc(
        "1.16.0",
        r#"{"description": {"identifier": "minecraft:enderman"},
            "components": {"minecraft:teleport": {"lightTeleportChance": 0.01, "randomTeleports": true}}}"#,
    );
    let (components, _) = written_components(&text);
    let teleport = member(&components, "minecraft:teleport");
    assert_eq!(at(&teleport, "light_teleport_chance"), Some(json("0.01")));
    assert_eq!(at(&teleport, "random_teleports"), Some(json("true")));
}

/// `behavior.panic.damage_sources`: the typed default is the empty list, and the game picks a
/// list by the document's version at run time (`PRE_SUMMER_24_UPDATE_DEFAULT_PANIC_DAMAGE_CAUSES`
/// = [`all`]); the crate's steps write `["all"]` into a document older than 1.21.0 that has none,
/// so the upgraded document, now at the newest version, keeps the older list.
#[test]
fn an_older_panic_keeps_all_damage_sources_after_the_upgrade() {
    let text = doc(
        "1.20.0",
        r#"{"description": {"identifier": "demo:x"},
            "components": {"minecraft:behavior.panic": {"speed_multiplier": 1.25}}}"#,
    );
    let (components, _) = written_components(&text);
    assert_eq!(
        at(
            &member(&components, "minecraft:behavior.panic"),
            "damage_sources"
        ),
        Some(json(r#"["all"]"#))
    );
}

// ---- the vanilla 1.26.60 entities (every file at format_version 1.26.60) ----

/// The eleven entity files of vanilla's `vanilla_1.26.60` pack (copied from the 1.26.60
/// game files) load at the newest struct with no warning and no error, read the 1.26.60 components
/// typed, and round-trip: written and read again, the same entity.
#[test]
fn the_vanilla_1_26_60_entities_load_typed_and_round_trip() {
    use bedrock_addon_family_entity::component::{
        Attack, Buoyant, DashAction, DismountMode, Healable, Inventory, MovementBasic,
        MovementSway, PushableByBlock, Rideable, Shareables, Strength,
    };
    let names = [
        "cushion",
        "fireball",
        "fox",
        "horse",
        "llama",
        "ocelot",
        "sheep",
        "shulker",
        "strider",
        "trader_llama",
        "zombie_nautilus",
    ];
    for name in names {
        let text = super::super::common::fixture(&format!("vanilla/{name}.json"));
        assert!(text.contains(r#""format_version":"1.26.60""#), "{name}");
        let (entity, found) = stable(&text, &cx());
        // Vanilla's own `ocelot` and `zombie_nautilus` name a component group in an event that
        // the file does not define (`minecraft:ocelot_leashed`,
        // `minecraft:zombie_nautilus_tame_unmounted`): the event rule's warning, not a
        // component's.
        let component_findings: Vec<_> = found
            .iter()
            .filter(|f| f.severity >= Severity::Warning)
            .filter(|f| {
                !(f.code.name() == "UnresolvedReference"
                    && f.location.json_path.to_string().contains("/events/"))
            })
            .collect();
        assert!(
            component_findings.is_empty(),
            "{name}: {component_findings:#?}"
        );
        assert_eq!(super::super::common::reparse(&entity), entity, "{name}");
    }
    let load = |name: &str| {
        stable(
            &super::super::common::fixture(&format!("vanilla/{name}.json")),
            &cx(),
        )
        .0
    };
    let cushion = load("cushion");
    let rideable = cushion
        .components
        .get::<Rideable>()
        .expect("cushion rideable");
    assert_eq!(rideable.dismount_mode, DismountMode::OnTopCenter);
    assert_eq!(
        rideable.seats[0].rotate_rider_by,
        bedrock_addon_core::molang::Molang::Const(-90.0)
    );
    assert!(somewhere::<PushableByBlock>(&load("fireball")));
    let fox = load("fox");
    assert!(somewhere::<MovementBasic>(&fox) && somewhere::<Shareables>(&fox));
    assert!(somewhere::<Attack>(&fox));
    let horse = load("horse");
    assert!(somewhere::<Buoyant>(&horse) && somewhere::<Healable>(&horse));
    assert!(somewhere::<Inventory>(&horse));
    assert!(somewhere::<Strength>(&load("llama")));
    let nautilus = load("zombie_nautilus");
    assert!(somewhere::<MovementSway>(&nautilus) && somewhere::<Attack>(&nautilus));
    assert!(somewhere::<DashAction>(&nautilus) && somewhere::<Inventory>(&nautilus));
}

/// Whether `entity` holds component `T`, in `components` or in a component group.
fn somewhere<T>(entity: &bedrock_addon_family_entity::Entity) -> bool
where
    T: bedrock_addon_core::document::Component<
            Set = bedrock_addon_family_entity::component::EntityComponent,
        >,
{
    entity.components.get::<T>().is_some()
        || entity
            .component_groups
            .values()
            .any(|group| group.components.get::<T>().is_some())
}

/// The DEFAULT lines the tests above do not reach: `transformation.into` empty, its sounds
/// and `add.component_groups` empty lists; a timer choice's `value` and `weight` 0 and no
/// `time_down_event`; no `teleport.projectile_filter`, no `trail.spawn_filter`; a seat's
/// `position` 0, rider counts 0, `rotate_rider_by` 0, `lock_rider_rotation` 181 and no camera
/// members.
#[test]
fn the_remaining_default_lines_hold() {
    let transformation = written("1.26.60", "minecraft:transformation", "{}");
    for (key, expected) in [
        ("into", r#""""#),
        ("begin_transform_sound", "[]"),
        ("transformation_sound", "[]"),
        ("add/component_groups", "[]"),
    ] {
        assert_eq!(at(&transformation, key), Some(json(expected)), "{key}");
    }
    let timer = written(
        "1.26.60",
        "minecraft:timer",
        r#"{"random_time_choices": [{}]}"#,
    );
    assert_eq!(
        at(&timer, "random_time_choices/0"),
        Some(json(r#"{"value": 0, "weight": 0}"#))
    );
    assert_eq!(at(&timer, "time_down_event"), None);
    assert_eq!(
        at(
            &written("1.26.60", "minecraft:teleport", "{}"),
            "projectile_filter"
        ),
        None
    );
    assert_eq!(
        at(&written("1.26.60", "minecraft:trail", "{}"), "spawn_filter"),
        None
    );
    let rideable = written("1.26.60", "minecraft:rideable", r#"{"seats": [{}]}"#);
    assert_eq!(
        at(&rideable, "seats/0"),
        Some(json(
            r#"{"position": [0, 0, 0], "min_rider_count": 0, "max_rider_count": 0,
                "rotate_rider_by": 0, "lock_rider_rotation": 181}"#
        ))
    );
}

/// A vector written as an object that leaves a coordinate out reads it as 0 and is written back
/// with all three: the place carries an information (`ValueRewritten`), as every place the
/// write-back changes, for a seat's `position`, `teleport.random_teleport_cube` and
/// `trail.spawn_offset`, in both reads.
#[test]
fn a_vector_object_missing_a_coordinate_is_told() {
    for (version, key, value, member_at, written_at) in [
        (
            "1.26.60",
            "minecraft:rideable",
            r#"{"seats": [{"position": {"y": 1}}]}"#,
            "minecraft:rideable/seats/0/position",
            "seats/0/position",
        ),
        (
            "1.16.0",
            "minecraft:rideable",
            r#"{"seats": {"position": {}}}"#,
            "minecraft:rideable/seats/position",
            "seats/0/position",
        ),
        (
            "1.26.60",
            "minecraft:teleport",
            r#"{"random_teleport_cube": {"x": 4, "z": 4}}"#,
            "minecraft:teleport/random_teleport_cube",
            "random_teleport_cube",
        ),
        (
            "1.26.50",
            "minecraft:trail",
            r#"{"spawn_offset": {}}"#,
            "minecraft:trail/spawn_offset",
            "spawn_offset",
        ),
    ] {
        let (components, found) = written_components(&wrapped(version, key, value));
        assert!(quiet(&found), "{version} {key}: {found:#?}");
        assert!(
            found.iter().any(|f| f.code.name() == "ValueRewritten"
                && f.severity == Severity::Info
                && f.location.json_path.to_string().ends_with(member_at)),
            "{version} {key}: {found:#?}"
        );
        let vector = at(&member(&components, key), written_at).expect("written");
        assert_eq!(vector.as_object().map(|o| o.len()), Some(3), "{vector:?}");
    }
}

/// The five opted-in components whose legacy reader is a schema reader, each with a value of
/// a type its reader does not take, nested where the schema nests it: the member, its path below
/// the component, and the allowed types the game lists.
const SCHEMA_READER_NESTED: [(&str, &str, &str, &str); 5] = [
    (
        "minecraft:healable",
        r#"{"items": [{"item": "minecraft:apple"},
                      {"item": "minecraft:golden_apple", "heal_amount": 4,
                       "effects": [{"name": "regeneration", "amplifier": "bad", "chance": 0.5}]}]}"#,
        "items/1/effects/0/amplifier",
        " 'int'",
    ),
    (
        "minecraft:rideable",
        r#"{"seat_count": 2, "seats": [{"position": [0, 1, 0], "min_rider_count": "two", "max_rider_count": 2}]}"#,
        "seats/0/min_rider_count",
        " 'int'",
    ),
    (
        "minecraft:shareables",
        r#"{"items": [{"item": "minecraft:apple", "priority": "high", "max_amount": 3}]}"#,
        "items/0/priority",
        " 'int'",
    ),
    (
        "minecraft:economy_trade_table",
        r#"{"display_name": "x", "table": 5}"#,
        "table",
        " 'string'",
    ),
    (
        "minecraft:timer",
        r#"{"looping": false, "random_time_choices": [{"value": 2, "weight": "heavy"}]}"#,
        "random_time_choices/0/weight",
        " 'int'",
    ),
];

/// From 1.15.0 the schema reader of a component validates the
/// whole tree first and a value no option takes fails it: the game logs the type line at error level, parses nothing and
/// loads the entity. The component is written as its defaults.
#[test]
fn a_bad_nested_value_of_a_schema_reader_defaults_the_whole_component_from_1_15_0() {
    for (key, value, tail, allowed) in SCHEMA_READER_NESTED {
        let (components, found) = written_components(&wrapped("1.26.50", key, value));
        let (defaults, _) = written_components(&wrapped("1.26.50", key, "{}"));
        assert_eq!(
            member(&components, key),
            member(&defaults, key),
            "{key}: the component keeps its defaults"
        );
        let line = found
            .iter()
            .find(|f| {
                f.kept
                    && f.location
                        .json_path
                        .to_string()
                        .ends_with(&format!("{key}/{tail}"))
            })
            .unwrap_or_else(|| panic!("{key}: no kept error at {tail}: {found:#?}"));
        assert_eq!(line.severity, Severity::Error, "{key}");
        assert_eq!(line.code.name(), "InvalidType", "{key}");
        assert_eq!(
            line.code.to_string(),
            format!("unknown child schema option type.  Allowed types: {allowed}"),
            "{key}"
        );
        assert!(
            !found
                .iter()
                .any(|f| f.severity >= Severity::Warning && !f.kept),
            "{key}: {found:#?}"
        );
    }
}

/// Below 1.15.0 the reader is lenient: the value no option takes is still logged
/// (both modes), and taken out alone (the member is removed): its element stays and the member takes its default.
#[test]
fn below_1_15_0_a_bad_nested_value_of_a_schema_reader_is_taken_out_alone() {
    let (components, found) = written_components(&wrapped(
        "1.14.0",
        SCHEMA_READER_NESTED[0].0,
        SCHEMA_READER_NESTED[0].1,
    ));
    let healable = member(&components, "minecraft:healable");
    assert_eq!(
        at(&healable, "items").and_then(|i| i.as_array().map(|a| a.len())),
        Some(2)
    );
    assert_eq!(at(&healable, "items/1/heal_amount"), Some(json("4")));
    assert_eq!(
        at(&healable, "items/1/effects/0/amplifier"),
        Some(json("1"))
    );
    assert_eq!(at(&healable, "items/1/effects/0/chance"), Some(json("0.5")));
    assert!(
        found.iter().any(|f| f.kept
            && f.code.name() == "InvalidType"
            && f.location
                .json_path
                .to_string()
                .ends_with("items/1/effects/0/amplifier")),
        "{found:#?}"
    );
    assert!(
        !found
            .iter()
            .any(|f| f.severity >= Severity::Warning && !f.kept),
        "{found:#?}"
    );
    // Each of the five below 1.15.0 — the type line told, the member taken out
    // alone, the rest of the component read (it is not its defaults).
    for (key, value, tail, allowed) in SCHEMA_READER_NESTED {
        let (components, found) = written_components(&wrapped("1.14.0", key, value));
        let (defaults, _) = written_components(&wrapped("1.14.0", key, "{}"));
        assert_ne!(
            member(&components, key),
            member(&defaults, key),
            "{key}: the rest of the component is read"
        );
        assert!(
            found.iter().any(|f| f.kept
                && f.code.to_string()
                    == format!("unknown child schema option type.  Allowed types: {allowed}")
                && f.location
                    .json_path
                    .to_string()
                    .ends_with(&format!("{key}/{tail}"))),
            "{key}: {found:#?}"
        );
        assert!(
            !found
                .iter()
                .any(|f| f.severity >= Severity::Warning && !f.kept),
            "{key}: {found:#?}"
        );
    }
}

/// Every legacy reader of these ranges reads an object's `range_min` / `range_max`
/// only — `timer.time`, `movement.jump.jump_delay`, `attack.damage` and
/// `economy_trade_table.cured_discount`: a `{min, max}` object keeps
/// the range's defaults, its two members dropped with an information each (the game is silent).
#[test]
fn a_legacy_range_object_reads_only_range_min_and_range_max() {
    for (key, member_key, value, beside) in [
        (
            "minecraft:timer",
            "time",
            r#"{"min": 1, "max": 3}"#,
            r#", "time_down_event": {"event": "demo:go"}"#,
        ),
        (
            "minecraft:movement.jump",
            "jump_delay",
            r#"{"min": 1, "max": 3}"#,
            "",
        ),
        ("minecraft:attack", "damage", r#"{"min": 1, "max": 3}"#, ""),
        (
            "minecraft:economy_trade_table",
            "cured_discount",
            r#"{"min": 1, "max": 3}"#,
            "",
        ),
    ] {
        let with = |range: &str| {
            written_components(&wrapped(
                "1.26.50",
                key,
                &format!(r#"{{"{member_key}": {range}{beside}}}"#),
            ))
        };
        let (components, found) = with(value);
        let (plain, _) = with("{}");
        let held = at(&member(&components, key), member_key);
        let defaults = at(&member(&plain, key), member_key);
        assert_eq!(held, defaults, "{key}: {components:#?}");
        for bound in ["min", "max"] {
            assert!(
                told(
                    &found,
                    "DroppedField",
                    Severity::Info,
                    &format!("{member_key}/{bound}")
                ),
                "{key} {bound}: {found:#?}"
            );
        }
        assert!(
            !found.iter().any(|f| f.severity >= Severity::Warning),
            "{key}: {found:#?}"
        );
    }
}

/// A legacy range bound that is no number reads as the game's
/// float / integer conversion reads it (a string 0), and the game logs nothing for it (the node's only
/// content-log line is the array-size text): an information where the file has it.
#[test]
fn a_legacy_range_bound_that_is_no_number_reads_as_zero_with_an_information() {
    let (components, found) = written_components(&wrapped(
        "1.26.50",
        "minecraft:timer",
        r#"{"time": {"range_min": "bad", "range_max": 3}, "time_down_event": {"event": "demo:go"}}"#,
    ));
    let timer = member(&components, "minecraft:timer");
    assert_eq!(at(&timer, "time/max"), Some(json("3")));
    assert_eq!(at(&timer, "time/min"), Some(json("0")));
    assert!(!found.iter().any(|f| f.kept), "{found:#?}");
    assert!(
        told(&found, "InvalidValue", Severity::Info, "time/range_min"),
        "{found:#?}"
    );
    assert!(
        !found.iter().any(|f| f.severity >= Severity::Warning),
        "{found:#?}"
    );
}

/// The three opted-in components whose legacy reader is hand-written
/// read member by member with defaults and keep the element (spell_effects pushes each
/// effect after its members are read; so do transformation and movement.jump).
#[test]
fn the_hand_written_legacy_readers_keep_the_element() {
    for (key, value, kept_at, kept, warned_at) in [
        (
            "minecraft:transformation",
            r#"{"into": "minecraft:villager", "delay": {"value": 5, "block_radius": "far"}}"#,
            "delay/value",
            "5",
            "minecraft:transformation/delay/block_radius",
        ),
        (
            "minecraft:spell_effects",
            r#"{"add_effects": [{"effect": "speed", "duration": 5, "amplifier": "bad"}]}"#,
            "add_effects/0/duration",
            "5",
            "minecraft:spell_effects/add_effects/0/amplifier",
        ),
    ] {
        let (components, found) = written_components(&wrapped("1.26.50", key, value));
        let component = member(&components, key);
        assert_eq!(
            at(&component, kept_at),
            Some(json(kept)),
            "{key}: {component:?}"
        );
        // A hand-written reader says nothing of a value of another type: the crate's finding is
        // an information.
        assert!(
            told(&found, "InvalidValue", Severity::Info, warned_at),
            "{key}: {found:#?}"
        );
        assert!(!found.iter().any(|f| f.kept), "{key}: {found:#?}");
    }
}

/// The 1.26.60 counterpart of the test above: the typed read refuses the
/// same nested values, each an `InvalidType` error at the member (the game's type error).
#[test]
fn a_bad_nested_value_is_an_error_at_the_target() {
    for (key, value, tail) in [
        (
            "minecraft:healable",
            r#"{"items": [{"item": "minecraft:apple", "effects": [{"name": "regeneration", "amplifier": "bad"}]}]}"#,
            "items/0/effects/0/amplifier",
        ),
        (
            "minecraft:transformation",
            r#"{"delay": {"value": 5, "block_radius": "far"}}"#,
            "delay/block_radius",
        ),
        (
            "minecraft:rideable",
            r#"{"seats": [{"min_rider_count": "two"}]}"#,
            "seats/0/min_rider_count",
        ),
        (
            "minecraft:shareables",
            r#"{"items": [{"item": "minecraft:apple", "priority": "high"}]}"#,
            "items/0/priority",
        ),
    ] {
        assert_error_at("1.26.60", key, value, tail, "InvalidType");
    }
}

/// `legacy_range` with integers (`IntRange`): the legacy reader reads
/// each bound with the game's integer conversion, which truncates a real toward zero;
/// the write-back writes the integer, told with an information.
#[test]
fn a_real_bound_of_a_legacy_integer_range_is_truncated() {
    let (components, found) = written_components(&wrapped(
        "1.26.50",
        "minecraft:economy_trade_table",
        r#"{"table": "trading/x.json", "cured_discount": {"range_min": -25.7, "range_max": -20}}"#,
    ));
    assert!(quiet(&found), "{found:#?}");
    let table = member(&components, "minecraft:economy_trade_table");
    assert_eq!(at(&table, "cured_discount/min"), Some(json("-25")));
    assert_eq!(at(&table, "cured_discount/max"), Some(json("-20")));
    assert!(
        told(
            &found,
            "ValueRewritten",
            Severity::Info,
            "cured_discount/range_min"
        ),
        "{found:#?}"
    );
}

/// The legacy `transformation` reader (log level 3) logs `Expected Number or Object.` for a `delay` of another type, and the delay
/// keeps its default; the entity loads below 1.26.60. At 1.26.60 the typed reader binds the
/// member (`Expected Number or Object.` is the legacy reader's alone): the wrong type is an
/// error that refuses the entity.
#[test]
fn a_legacy_transformation_delay_of_another_type_is_told_with_the_legacy_text() {
    let value = r#"{"into": "minecraft:pig", "delay": "soon"}"#;
    let (components, found) =
        written_components(&wrapped("1.26.50", "minecraft:transformation", value));
    assert!(components.is_object(), "{found:#?}");
    let texts: Vec<String> = found
        .iter()
        .filter(|f| f.severity >= Severity::Warning)
        .map(|f| {
            format!(
                "{} {:?} {} {}",
                f.code.name(),
                f.severity,
                f.location.json_path,
                f.code
            )
        })
        .collect();
    assert_eq!(
        texts,
        [
            "InvalidValue Error minecraft:entity/components/minecraft:transformation/delay Expected Number or Object."
        ],
        "{found:#?}"
    );
    let refused = outcome(
        &wrapped("1.26.60", "minecraft:transformation", value),
        &cx(),
    );
    assert!(refused.value.is_none(), "{:#?}", refused.diagnostics);
    let at_delay: Vec<String> = refused
        .diagnostics
        .iter()
        .filter(|f| {
            f.location
                .json_path
                .to_string()
                .ends_with("transformation/delay")
        })
        .map(|f| format!("{} {:?}", f.code.name(), f.severity))
        .collect();
    assert_eq!(
        at_delay,
        ["InvalidType Error"],
        "{:#?}",
        refused.diagnostics
    );
    assert!(
        !refused
            .diagnostics
            .iter()
            .any(|f| f.code.to_string() == "Expected Number or Object."),
        "{:#?}",
        refused.diagnostics
    );
}

/// At its typed version a component's range binds only `min` / `max`
/// (the integer range has the same shape): a
/// `{range_min, range_max}` range is two unknown members, an error at each.
#[test]
fn a_typed_component_range_refuses_the_legacy_range_spelling() {
    for (key, value, at) in [
        (
            "minecraft:timer",
            r#"{"time": {"range_min": 1, "range_max": 3}}"#,
            "time",
        ),
        (
            "minecraft:attack",
            r#"{"damage": {"range_min": 1, "range_max": 3}}"#,
            "damage",
        ),
        (
            "minecraft:movement.jump",
            r#"{"jump_delay": {"range_min": 1, "range_max": 3}}"#,
            "jump_delay",
        ),
        (
            "minecraft:economy_trade_table",
            r#"{"table": "x.json", "cured_discount": {"range_min": -3, "range_max": -1}}"#,
            "cured_discount",
        ),
    ] {
        for end in ["range_min", "range_max"] {
            assert_error_at(
                "1.26.60",
                key,
                value,
                &format!("{at}/{end}"),
                "UnrecognizedField",
            );
        }
    }
}
