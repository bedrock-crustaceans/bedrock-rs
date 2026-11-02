//! The Brain memory goals and memory sensors.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::goal;

use super::{assert_finding, fixture_member, goal, strict};

const MOVE_TO_POSITION: &str = "minecraft:memory_behavior.move_to_position";

/// `v15_brain_move_to_position_names`: `speed_modifier`, `goal_radius`,
/// `maximum_range`, `interval`, `memory`, `on_reach` and `chance_to_start` are read;
/// `speed_mod` / `interval_seconds` are `UnrecognizedField`.
#[test]
fn v15_move_to_position_reads_its_members_and_not_the_draft_spellings() {
    const NAME: &str = "v15_brain_move_to_position_names";
    let json = fixture_member(NAME, "minecraft:memory_behavior.move_to_position.x");
    let (g, d) = goal::<goal::MemoryMoveToPosition>(MOVE_TO_POSITION, &json, ConvertMode::Strict);
    let g = g.expect("kept");
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(g.speed_modifier.map(|v| v.0), Some(1.5));
    assert_eq!(g.goal_radius.map(|v| v.0), Some(3.0));
    assert_eq!(g.maximum_range.map(|v| v.0), Some(16.0));
    assert_eq!(g.interval.map(|r| (r.min, r.max)), Some((2.0, 4.0)));
    assert_eq!(
        g.memory.as_ref().map(|m| m.0.as_str()),
        Some("demo:nearest_gold_block")
    );
    assert_eq!(g.on_reach.as_ref().map(Vec::len), Some(1));
    assert_eq!(g.chance_to_start.map(|v| v.0), Some(0.5));

    let json = fixture_member(NAME, "draft_spellings");
    let (g, d) = goal::<goal::MemoryMoveToPosition>(MOVE_TO_POSITION, &json, ConvertMode::Strict);
    assert!(g.is_some(), "unknown members do not drop the goal");
    assert_finding(&d, "UnrecognizedField", "speed_mod", Severity::Warning);
    assert_finding(
        &d,
        "UnrecognizedField",
        "interval_seconds",
        Severity::Warning,
    );
}

/// In 1.26.60 the wait goal binds `on_complete` and inherits the memory-goal base, whose
/// `duration` is optional ("If not provided, will default to an infinite duration").
#[test]
fn memory_wait_takes_an_optional_duration_and_its_on_complete() {
    let key = "minecraft:memory_behavior.wait";
    let (g, d) = goal::<goal::MemoryWait>(key, r#"{"duration": [1, 3]}"#, ConvertMode::Strict);
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(
        g.expect("kept").duration.map(|r| (r.min, r.max)),
        Some((1.0, 3.0))
    );
    let (g, d) = goal::<goal::MemoryWait>(
        key,
        r#"{"priority": 1, "on_complete": [{"event": "demo:done", "target": "self"}]}"#,
        ConvertMode::Strict,
    );
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(g.expect("kept").on_complete.as_ref().map(Vec::len), Some(1));
}

/// Every typed memory goal inherits the memory-goal base:
/// `conditional_filters`, `chance_to_start` (0 to 1),
/// `retry_attempt_cooldown`, `duration` (an optional number or range); `follow_target` binds
/// `min_range` (≥ 0); `look_at_entity`
/// binds no `look_time` (its members: `look_distance`, `look_angle`, `target_entity`).
#[test]
fn every_memory_goal_takes_the_memory_goal_base() {
    let base = r#""conditional_filters": {"test": "is_family", "subject": "self", "value": "demo"}, "chance_to_start": 0.5, "retry_attempt_cooldown": 2, "duration": [1, 2]"#;
    let rows = [
        (
            "minecraft:memory_behavior.look_at_entity",
            r#""look_distance": 6, "target_entity": "demo:player""#,
        ),
        (
            "minecraft:memory_behavior.follow_target",
            r#""target_memory": "demo:parent", "min_range": 1.5"#,
        ),
        (
            "minecraft:memory_behavior.move_to_position",
            r#""memory": "demo:spot""#,
        ),
        ("minecraft:memory_behavior.wait", r#""priority": 1"#),
    ];
    for (key, own) in rows {
        let json = format!("{{{base}, {own}}}");
        let (g, d) = strict(key, &json);
        assert!(g.is_some(), "{key}: {d:?}");
        assert!(d.is_empty(), "{key}: {d:?}");
        let (g, d) = strict(key, &format!("{{\"chance_to_start\": 2, {own}}}"));
        assert!(g.is_none(), "{key}");
        assert_finding(&d, "InvalidValue", "chance_to_start", Severity::Error);
    }
    let (g, d) = goal::<goal::MemoryFollowTarget>(
        "minecraft:memory_behavior.follow_target",
        r#"{"target_memory": "demo:parent", "min_range": 1.5}"#,
        ConvertMode::Strict,
    );
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(g.expect("kept").min_range.map(|v| v.0), Some(1.5));
    let (_, d) = goal::<goal::MemoryLookAtEntity>(
        "minecraft:memory_behavior.look_at_entity",
        r#"{"look_distance": 6, "target_entity": "demo:player", "look_time": 5}"#,
        ConvertMode::Strict,
    );
    assert_finding(&d, "UnrecognizedField", "look_time", Severity::Warning);
}

/// `look_angle` is a two-component vector,
/// each coordinate 0 to 360: the brain prototype's `[45, 45]` reads; a lone number is
/// no vector.
#[test]
fn memory_look_at_entity_requires_its_distance_and_target_and_bounds_its_angle() {
    let key = "minecraft:memory_behavior.look_at_entity";
    let json = r#"{"look_distance": 6, "look_angle": [45, 45], "duration": [1, 2], "target_entity": "demo:player"}"#;
    let (g, d) = goal::<goal::MemoryLookAtEntity>(key, json, ConvertMode::Strict);
    assert!(d.is_empty(), "{d:?}");
    let g = g.expect("kept");
    assert_eq!(
        g.look_angle.as_ref().map(|v| (v.x, v.y)),
        Some((45.0, 45.0))
    );
    let (g, d) = strict(
        key,
        r#"{"look_distance": 6, "look_angle": [400, 10], "target_entity": "demo:player"}"#,
    );
    assert!(g.is_none());
    assert_finding(&d, "InvalidValue", "look_angle", Severity::Error);
    let (g, _) = strict(
        key,
        r#"{"look_distance": 6, "look_angle": 90, "target_entity": "demo:player"}"#,
    );
    assert!(g.is_none(), "a lone number is no Vec2");
    let (g, d) = strict(key, r#"{"look_angle": [90, 90]}"#);
    assert!(g.is_none());
    assert_finding(&d, "MissingRequiredField", "look_distance", Severity::Error);
    assert_finding(&d, "MissingRequiredField", "target_entity", Severity::Error);
}

#[test]
fn memory_follow_target_reads_its_memory_and_speed() {
    let key = "minecraft:memory_behavior.follow_target";
    let json = r#"{"priority": 1, "target_memory": "demo:parent", "speed_modifier": 1.1}"#;
    let (g, d) = goal::<goal::MemoryFollowTarget>(key, json, ConvertMode::Strict);
    assert!(d.is_empty(), "{d:?}");
    let g = g.expect("kept");
    assert_eq!(g.priority(), 1);
    assert_eq!(g.target_memory.map(|m| m.0), Some("demo:parent".to_owned()));
}

/// The sensor entry of the experimental brain prototype pack (`demo_cube.json`), in the
/// typed form.
#[test]
fn memory_sensors_read_their_entries() {
    let key = "minecraft:memory_sensor.find_nearest_entity";
    let json = r#"{"sensor_entries": [{
        "name": "find_parent",
        "found_entity_memory": "minecraft:parent",
        "cooldown": 2,
        "search_range": [8.0, 4.0, 8.0],
        "conditional_filters": {"all_of": [{"test": "is_avoiding_mobs", "subject": "self", "value": false}]},
        "target_filters": {"all_of": [{"test": "is_family", "subject": "other", "value": "demo"}]}
    }]}"#;
    let (g, d) = goal::<goal::MemorySensorFindNearestEntity>(key, json, ConvertMode::Strict);
    assert!(d.is_empty(), "{d:?}");
    let entries = g.expect("kept").sensor_entries.expect("entries");
    assert_eq!(
        entries[0].search_range.as_ref().map(|v| (v.x, v.y, v.z)),
        Some((8.0, 4.0, 8.0))
    );

    let key = "minecraft:memory_sensor.find_nearest_block";
    let json = r#"{"sensor_entries": [{"cooldown": 1, "found_blockpos_memory": "demo:gold",
        "blocks": ["minecraft:gold_block", {"tags": "query.any_tag('stone')"}],
        "search_range": 16, "search_height": 4}]}"#;
    let (g, d) = goal::<goal::MemorySensorFindNearestBlock>(key, json, ConvertMode::Strict);
    assert!(d.is_empty(), "{d:?}");
    assert!(g.is_some());

    let key = "minecraft:memory_sensor.find_nearest_poi";
    let json = r#"{"sensor_entries": [{"name": "bed", "found_poi_memory": "demo:bed", "poi_name": "minecraft:bed", "search_radius": 16}]}"#;
    let (g, d) = goal::<goal::MemorySensorFindNearestPoi>(key, json, ConvertMode::Strict);
    assert!(d.is_empty() && g.is_some(), "{d:?}");

    let key = "minecraft:memory_sensor.game_event";
    let json = r#"{"sensor_entries": [{"radius": 16, "event_type": "GoatHorn",
        "source_entity_filters": {"test": "is_family", "subject": "other", "value": "player"},
        "response": {"event": "demo:heard", "setters": [{"memory": "demo:heard", "value": true}]}}]}"#;
    let (g, d) = goal::<goal::MemorySensorGameEvent>(key, json, ConvertMode::Strict);
    assert!(d.is_empty() && g.is_some(), "{d:?}");

    let key = "minecraft:memory_sensor.in_range_of_entity";
    let json = r#"{"sensor_entries": [{"cooldown": 0.5, "target_entity_memory": "demo:threat", "search_range": 8,
        "entered_range_response": {"event": "demo:near", "setters": [{"memory": "demo:near", "value": true}]},
        "exited_range_response": {"event": "demo:far"}}]}"#;
    let (g, d) = goal::<goal::MemorySensorInRangeOfEntity>(key, json, ConvertMode::Strict);
    assert!(d.is_empty(), "{d:?}");
    let entry = &g.expect("kept").sensor_entries.expect("entries")[0];
    let setters = entry
        .entered_range_response
        .as_ref()
        .and_then(|r| r.setters.as_ref());
    assert_eq!(setters.map(Vec::len), Some(1));

    let (g, d) = strict(key, "{}");
    assert!(g.is_none(), "a sensor without entries is dropped");
    assert_finding(
        &d,
        "MissingRequiredField",
        "sensor_entries",
        Severity::Error,
    );
    let (g, d) = strict(key, r#"{"sensor_entries": [{"cooldown": -1}]}"#);
    assert!(g.is_none());
    assert_finding(
        &d,
        "InvalidValue",
        "sensor_entries/0/cooldown",
        Severity::Error,
    );
}

/// Goal part: the three ram/jump memory goals (a `.<name>` family
/// each) and the ram charge sensor, typed from 1.26.50 with the Brain Prototype, with their
/// defaults; a bare family key is `GenericGoalSuffixMissing`; below 1.26.50 they are unknown.
#[test]
fn the_1_26_50_ram_and_jump_brain_keys_are_typed() {
    use bedrock_addon_core::document::ComponentRef;
    use bedrock_addon_core::load::{Experiment, LoadContext};
    let mut cx = LoadContext::detached();
    cx.experiments.insert(Experiment::BrainPrototype);
    let doc = |version: &str, components: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:entity": {{"description": {{"identifier": "demo:goat"}}, "components": {{{components}}}}}}}"#
        )
    };
    let all = r#""minecraft:memory_behavior.jump_to_block.hop": {"priority": 1, "target_blockpos_memory": "jump_target", "jump_velocity_memory": "jump_velocity", "on_complete": [{"event": "demo:landed"}]},
        "minecraft:memory_behavior.prepare_ram_attack.ram": {"priority": 2, "target_entity_memory": "target", "target_position_memory": "target_pos", "on_entity_moved": [{"event": "demo:moved"}]},
        "minecraft:memory_behavior.ram_attack.ram": {"priority": 3, "ram_position_memory": "ram_pos", "knockback_force": 2.5, "on_collide": [{"event": "demo:hit"}]},
        "minecraft:memory_sensor.find_ram_charge_position": {"sensor_entries": [{"target_entity_memory": "target", "charge_position_memory": "charge", "min_ram_distance": 3}]}"#;
    let outcome = bedrock_addon_core::versioning::load::<bedrock_addon_family_entity::Entity>(
        bedrock_addon_core::json::JsonSlice::new(&doc("1.26.50", all)),
        &cx,
    );
    let Some(bedrock_addon_core::versioning::Loaded::Stable(entity)) = outcome.value else {
        panic!("{:#?}", outcome.diagnostics);
    };
    let warned: Vec<_> = outcome
        .diagnostics
        .iter()
        .filter(|f| f.severity >= Severity::Warning)
        .collect();
    assert!(warned.is_empty(), "{warned:#?}");
    let typed: Vec<&str> = entity
        .components
        .iter()
        .filter_map(|entry| match entry {
            ComponentRef::Typed { name, .. } => Some(name),
            _ => None,
        })
        .collect();
    for key in [
        "minecraft:memory_behavior.jump_to_block.hop",
        "minecraft:memory_behavior.prepare_ram_attack.ram",
        "minecraft:memory_behavior.ram_attack.ram",
        "minecraft:memory_sensor.find_ram_charge_position",
    ] {
        assert!(typed.contains(&key), "{key} is not typed: {typed:?}");
    }
    use bedrock_addon_family_entity::component::EntityComponent as C;
    let typed_value = |key: &str| {
        entity.components.iter().find_map(|entry| match entry {
            ComponentRef::Typed { name, value, .. } if name == key => Some(value.clone()),
            _ => None,
        })
    };
    let Some(C::MemoryBehaviorRamAttack(ram)) =
        typed_value("minecraft:memory_behavior.ram_attack.ram")
    else {
        panic!("a ram attack");
    };
    assert!((ram.knockback_force() - 2.5).abs() < f32::EPSILON);
    assert!((ram.knockback_height() - 0.1).abs() < f32::EPSILON);
    assert!((ram.baby_knockback_modifier() - 1.0 / 3.0).abs() < f32::EPSILON);
    assert_eq!(ram.pre_ram_sound(), "undefined");
    let Some(C::MemoryBehaviorJumpToBlock(jump)) =
        typed_value("minecraft:memory_behavior.jump_to_block.hop")
    else {
        panic!("a jump");
    };
    assert!((jump.scale_factor() - 0.7).abs() < f32::EPSILON);
    let sensor = entity
        .components
        .get::<goal::MemorySensorFindRamChargePosition>()
        .expect("a sensor");
    let entry = &sensor.sensor_entries.as_ref().unwrap()[0];
    assert_eq!((entry.min_ram_distance(), entry.max_ram_distance()), (3, 7));
    // A bare family key is a missing suffix.
    let bare = bedrock_addon_core::versioning::load::<bedrock_addon_family_entity::Entity>(
        bedrock_addon_core::json::JsonSlice::new(&doc(
            "1.26.50",
            r#""minecraft:memory_behavior.ram_attack": {"priority": 1}"#,
        )),
        &cx,
    );
    let said = |found: &bedrock_addon_core::diagnostic::Diagnostics, code: &str| -> Vec<String> {
        found
            .iter()
            .filter(|f| f.code.name() == code)
            .map(|f| format!("{:?} {} {}", f.severity, f.kept, f.location.json_path))
            .collect()
    };
    // The generic goal rule's information ; the entity loads without the goal.
    assert!(bare.value.is_some(), "{:#?}", bare.diagnostics);
    assert_eq!(
        said(&bare.diagnostics, "GenericGoalSuffixMissing"),
        ["Info false minecraft:entity/components/minecraft:memory_behavior.ram_attack"],
        "{:#?}",
        bare.diagnostics
    );
    // Below 1.26.50 no slice binds them: an unknown component, refused from 1.21.89. The
    // adjacent versions: 1.26.49 refuses each key, 1.26.50 (above) types it.
    for (key, json) in [
        (
            "minecraft:memory_behavior.jump_to_block.hop",
            r#"{"priority": 1}"#,
        ),
        (
            "minecraft:memory_behavior.prepare_ram_attack.ram",
            r#"{"priority": 1}"#,
        ),
        (
            "minecraft:memory_behavior.ram_attack.ram",
            r#"{"priority": 1}"#,
        ),
        (
            "minecraft:memory_sensor.find_ram_charge_position",
            r#"{"sensor_entries": []}"#,
        ),
    ] {
        let older = bedrock_addon_core::versioning::load::<bedrock_addon_family_entity::Entity>(
            bedrock_addon_core::json::JsonSlice::new(&doc(
                "1.26.49",
                &format!(r#""{key}": {json}"#),
            )),
            &cx,
        );
        assert!(older.value.is_none(), "{key}: {:#?}", older.diagnostics);
        assert_eq!(
            said(&older.diagnostics, "UnrecognizedComponent"),
            [format!("Error false minecraft:entity/components/{key}")],
            "{key}: {:#?}",
            older.diagnostics
        );
    }
}

/// The memory-goal base's `duration` carries a constraint: its number form at least 0,
/// its range form a range whose ends are each at least 0 and
/// whose max is not below its min ("max value cannot be lower than min value").
/// A value outside is an error of the strict read with the constraint's text.
#[test]
fn a_memory_goal_duration_outside_its_constraint_is_refused() {
    let key = "minecraft:memory_behavior.wait";
    let rows: [(&str, &str, &str); 4] = [
        (
            r#"{"duration": -1}"#,
            "duration",
            "the input must be a number >= 0",
        ),
        (
            r#"{"duration": [-1, 2]}"#,
            "duration/0",
            "the input must be a number >= 0",
        ),
        (
            r#"{"duration": {"min": 1, "max": -2}}"#,
            "duration/max",
            "the input must be a number >= 0",
        ),
        (
            r#"{"duration": {"min": 3, "max": 2}}"#,
            "duration",
            "max value cannot be lower than min value",
        ),
    ];
    for (json, at, text) in rows {
        let (g, d) = strict(key, json);
        assert!(g.is_none(), "{json}: {d:?}");
        assert!(
            d.iter().any(|f| f.code.name() == "InvalidValue"
                && f.severity == Severity::Error
                && f.location.json_path.to_string() == at
                && f.code.to_string() == text),
            "{json}: {d:?}"
        );
    }
    let (g, d) = strict(key, r#"{"duration": [0, 2]}"#);
    assert!(g.is_some() && d.is_empty(), "{d:?}");
}

/// `move_to_position.interval` carries the same constraint (float min 0,
/// range ends min 0), and
/// `speed_modifier`, `goal_radius`, `maximum_range` are each at least 0.
#[test]
fn move_to_position_values_outside_their_constraints_are_refused() {
    let rows: [(&str, &str, &str); 5] = [
        (
            r#""interval": -1"#,
            "interval",
            "the input must be a number >= 0",
        ),
        (
            r#""interval": {"min": 4, "max": 2}"#,
            "interval",
            "max value cannot be lower than min value",
        ),
        (
            r#""speed_modifier": -1"#,
            "speed_modifier",
            "the input must be a number >= 0",
        ),
        (
            r#""goal_radius": -1"#,
            "goal_radius",
            "the input must be a number >= 0",
        ),
        (
            r#""maximum_range": -1"#,
            "maximum_range",
            "the input must be a number >= 0",
        ),
    ];
    for (member, at, text) in rows {
        let json = format!(r#"{{"memory": "demo:spot", {member}}}"#);
        let (g, d) = strict(MOVE_TO_POSITION, &json);
        assert!(g.is_none(), "{json}: {d:?}");
        assert!(
            d.iter().any(|f| f.code.name() == "InvalidValue"
                && f.severity == Severity::Error
                && f.location.json_path.to_string() == at
                && f.code.to_string() == text),
            "{json}: {d:?}"
        );
    }
}

/// The game validates the stored range, where an end the
/// object leaves out holds the type's default 0: `{"min": 1}` is max 0 below min 1 and fails; `{"max": 2}` passes.
#[test]
fn a_memory_goal_range_with_one_end_is_checked_against_the_default_end() {
    let key = "minecraft:memory_behavior.wait";
    let (g, d) = strict(key, r#"{"duration": {"min": 1}}"#);
    assert!(g.is_none(), "{d:?}");
    assert!(
        d.iter()
            .any(|f| f.location.json_path.to_string() == "duration"
                && f.code.to_string() == "max value cannot be lower than min value"),
        "{d:?}"
    );
    let (g, d) = strict(key, r#"{"duration": {"max": 2}}"#);
    assert!(g.is_some() && d.is_empty(), "{d:?}");
}

/// A value on the bound (0) is accepted for `interval` and the
/// three `move_to_position` members; `follow_target`'s `speed_modifier` and `min_range` carry
/// the "must be a number >= 0" text; an end below 0 is told at `/min`, `/max` and `/1` as the
/// file writes it (the typed range has no `range_*` spelling).
#[test]
fn the_memory_goal_bounds_on_their_edge_and_at_each_end_path() {
    for member in [
        r#""interval": 0"#,
        r#""interval": [0, 0]"#,
        r#""speed_modifier": 0"#,
        r#""goal_radius": 0"#,
        r#""maximum_range": 0"#,
    ] {
        let json = format!(r#"{{"memory": "demo:spot", {member}}}"#);
        let (g, d) = strict(MOVE_TO_POSITION, &json);
        assert!(g.is_some() && d.is_empty(), "{json}: {d:?}");
    }
    let follow = "minecraft:memory_behavior.follow_target";
    for member in ["speed_modifier", "min_range"] {
        let json = format!(r#"{{"target_memory": "demo:parent", "{member}": -0.5}}"#);
        let (g, d) = strict(follow, &json);
        assert!(g.is_none(), "{json}");
        assert!(
            d.iter().any(|f| f.severity == Severity::Error
                && f.location.json_path.to_string() == member
                && f.code.to_string() == "the input must be a number >= 0"),
            "{json}: {d:?}"
        );
    }
    let key = "minecraft:memory_behavior.wait";
    for (json, at) in [
        (r#"{"duration": {"min": -1, "max": 2}}"#, "duration/min"),
        (r#"{"duration": {"min": 1, "max": -2}}"#, "duration/max"),
        (r#"{"duration": [1, -2]}"#, "duration/1"),
    ] {
        let (g, d) = strict(key, json);
        assert!(g.is_none(), "{json}");
        assert!(
            d.iter().any(|f| f.location.json_path.to_string() == at
                && f.code.to_string() == "the input must be a number >= 0"),
            "{json}: {d:?}"
        );
    }
}

/// The typed range binds only
/// `min` and `max` and has no other spelling: at a goal's typed version `{range_min, range_max}` are unknown members
/// of the range, refused under the Document mask; the legacy schema readers' spelling
/// is no part of the typed read.
#[test]
fn a_typed_goal_range_refuses_the_legacy_range_spelling() {
    for (key, json, at) in [
        (
            "minecraft:memory_behavior.wait",
            r#"{"duration": {"range_min": 1, "range_max": 2}}"#,
            "duration",
        ),
        (
            MOVE_TO_POSITION,
            r#"{"memory": "demo:spot", "interval": {"range_min": 1, "range_max": 2}}"#,
            "interval",
        ),
        (
            "minecraft:behavior.beg",
            r#"{"look_time": {"range_min": 2, "range_max": 4}}"#,
            "look_time",
        ),
    ] {
        let (g, d) = strict(key, json);
        assert!(g.is_none(), "{key}: {d:?}");
        for end in ["range_min", "range_max"] {
            assert!(
                d.iter().any(|f| f.code.name() == "UnrecognizedField"
                    && f.severity == Severity::Error
                    && f.location.json_path.to_string() == format!("{at}/{end}")),
                "{key} {end}: {d:?}"
            );
        }
    }
}
