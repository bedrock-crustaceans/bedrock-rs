//! What a step of an entity replaces, drops or moves is told at the member the FILE has
//! A legacy member
//! beside the member a step writes it to, the turtle's and the pufferfish's own values the
//! fixer replaces, and the findings of the typed read of a member a step moved.

use bedrock_addon_core::document::Content;

use super::common::{cx, doc, stable};

/// Every finding of the entity `identifier` with the payload members `rest` at `version`, as
/// `<code> <severity> <path below the payload>`, and the written entity on one line.
#[track_caller]
fn told(version: &str, identifier: &str, rest: &str) -> (Vec<String>, String) {
    let text = doc(
        version,
        &format!(r#"{{"description": {{"identifier": "{identifier}"}}, {rest}}}"#),
    );
    let (entity, found) = stable(&text, &cx());
    let said = found
        .iter()
        .map(|f| {
            let path = f.location.json_path.to_string();
            let path = path.strip_prefix("minecraft:entity/").unwrap_or(&path);
            format!("{} {:?} {path}", f.code.name(), f.severity)
        })
        .collect();
    let written = entity.to_json().expect("the entity writes");
    (
        said,
        written.split_whitespace().collect::<Vec<_>>().join(" "),
    )
}

#[track_caller]
fn has(found: &[String], finding: &str) {
    assert!(
        found.iter().any(|f| f == finding),
        "no `{finding}` in {found:#?}"
    );
}

#[test]
fn case_1_a_target_range_beside_max_dist_is_noted_as_replaced() {
    let (found, written) = told(
        "1.16.0",
        "demo:x",
        r#""components": {"minecraft:behavior.drop_item_for": {"priority": 1, "max_dist": 3, "target_range": [9, 9, 9]}}"#,
    );
    has(
        &found,
        "DroppedField Info components/minecraft:behavior.drop_item_for/target_range",
    );
    assert!(
        written.contains(r#""target_range": [ 3.0, 3.0, 3.0 ]"#),
        "{written}"
    );
    // The same range: nothing is replaced.
    let (found, _) = told(
        "1.16.0",
        "demo:x",
        r#""components": {"minecraft:behavior.drop_item_for": {"priority": 1, "max_dist": 3, "target_range": [3, 3, 3]}}"#,
    );
    assert!(
        !found.iter().any(|f| f.starts_with("DroppedField")),
        "{found:#?}"
    );
}

#[test]
fn case_2_a_max_wait_time_beside_wait_time_is_noted_as_replaced() {
    let (found, written) = told(
        "1.16.0",
        "demo:x",
        r#""components": {"minecraft:behavior.explore_outskirts": {"priority": 1, "wait_time": 600, "max_wait_time": 99}}"#,
    );
    has(
        &found,
        "DroppedField Info components/minecraft:behavior.explore_outskirts/max_wait_time",
    );
    assert!(written.contains(r#""max_wait_time": 30"#), "{written}");
    assert!(!written.contains("99"), "{written}");
}

#[test]
fn case_3_a_trigger_s_filters_and_event_go_into_the_on_damage_it_has() {
    for sensor in [
        r#"{"triggers": [{"cause": "fire", "filters": {"test": "has_tag", "value": "a"}, "event": "x:e", "on_damage": {"event": "x:mine", "target": "self"}}]}"#,
        r#"{"cause": "fire", "filters": {"test": "has_tag", "value": "a"}, "event": "x:e", "on_damage": {"event": "x:mine", "target": "self"}}"#,
    ] {
        let (found, written) = told(
            "1.12.0",
            "demo:x",
            &format!(r#""components": {{"minecraft:damage_sensor": {sensor}}}"#),
        );
        // Both members are under `on_damage`, as the folder says; the event the file's
        // `on_damage` had is written over and noted there; what the move does not write
        // (`target`) stays.
        assert!(written.contains(r#""event": "x:e""#), "{sensor}: {written}");
        assert!(written.contains(r#""value": "a""#), "{sensor}: {written}");
        assert!(
            written.contains(r#""target": "self""#),
            "{sensor}: {written}"
        );
        assert!(!written.contains("x:mine"), "{sensor}: {written}");
        let at = if sensor.starts_with(r#"{"triggers""#) {
            "components/minecraft:damage_sensor/triggers/0/on_damage/event"
        } else {
            "components/minecraft:damage_sensor/on_damage/event"
        };
        has(&found, &format!("DroppedField Info {at}"));
    }
}

#[test]
fn case_4_an_entry_of_breeds_with_a_later_entry_of_its_mate_replaces_is_noted() {
    let (found, written) = told(
        "1.21.120",
        "demo:x",
        r#""components": {"minecraft:breedable": {"breeds_with": [{"mate_type": "demo:a", "baby_type": "demo:b", "breed_event": {"event": "x:first"}}, {"mate_type": "demo:a", "baby_type": "demo:c"}]}}"#,
    );
    has(
        &found,
        "DroppedField Info components/minecraft:breedable/breeds_with/0/breed_event",
    );
    has(
        &found,
        "DroppedField Info components/minecraft:breedable/breeds_with/0/baby_type",
    );
    assert!(written.contains(r#""demo:a": "demo:c""#), "{written}");
    assert!(!written.contains("x:first"), "{written}");
    // Two mates: nothing is replaced.
    let (found, _) = told(
        "1.21.120",
        "demo:x",
        r#""components": {"minecraft:breedable": {"breeds_with": [{"mate_type": "demo:a", "baby_type": "demo:b"}, {"mate_type": "demo:d", "baby_type": "demo:c"}]}}"#,
    );
    assert!(
        !found.iter().any(|f| f.starts_with("DroppedField")),
        "{found:#?}"
    );
}

#[test]
fn case_5_an_offspring_of_the_file_the_breedable_split_replaces_is_noted() {
    let (found, written) = told(
        "1.21.120",
        "demo:x",
        r#""components": {"minecraft:breedable": {"breeds_with": {"mate_type": "demo:a", "baby_type": "demo:b"}}, "minecraft:offspring": {"offspring_pairs": {"demo:z": "demo:y"}, "blend_attributes": false}}"#,
    );
    has(&found, "DroppedField Info components/minecraft:offspring");
    assert!(written.contains(r#""demo:a": "demo:b""#), "{written}");
    assert!(!written.contains("demo:z"), "{written}");
    // A `breedable` that gives no offspring leaves the file's own where it is.
    let (found, written) = told(
        "1.21.120",
        "demo:x",
        r#""components": {"minecraft:breedable": {"require_tame": false}, "minecraft:offspring": {"offspring_pairs": {"demo:z": "demo:y"}}}"#,
    );
    assert!(
        !found.iter().any(|f| f.starts_with("DroppedField")),
        "{found:#?}"
    );
    assert!(written.contains(r#""demo:z": "demo:y""#), "{written}");
}

#[test]
fn case_6_the_turtle_s_own_lightning_multiplier_the_fixer_replaces_is_noted() {
    for (sensor, at) in [
        (
            r#"[{"cause": "lightning", "deals_damage": true, "damage_multiplier": 3}]"#,
            "components/minecraft:damage_sensor/0/damage_multiplier",
        ),
        (
            r#"{"cause": "lightning", "deals_damage": true, "damage_multiplier": 3}"#,
            "components/minecraft:damage_sensor/damage_multiplier",
        ),
        (
            r#"{"triggers": {"cause": "lightning", "damage_multiplier": 3}}"#,
            "components/minecraft:damage_sensor/triggers/damage_multiplier",
        ),
    ] {
        let (found, written) = told(
            "1.12.0",
            "minecraft:turtle",
            &format!(r#""components": {{"minecraft:damage_sensor": {sensor}}}"#),
        );
        has(&found, &format!("DroppedField Info {at}"));
        assert!(
            written.contains(r#""damage_multiplier": 2000"#),
            "{sensor}: {written}"
        );
    }
    // The fixer's own number: nothing to note.
    let (found, _) = told(
        "1.12.0",
        "minecraft:turtle",
        r#""components": {"minecraft:damage_sensor": [{"cause": "lightning", "deals_damage": true, "damage_multiplier": 2000}]}"#,
    );
    assert!(
        !found.iter().any(|f| f.contains("damage_multiplier")),
        "{found:#?}"
    );
}

#[test]
fn case_7_the_pufferfish_s_own_groups_and_events_the_fixer_replaces_are_noted() {
    let (found, written) = told(
        "1.12.0",
        "minecraft:pufferfish",
        r#""component_groups": {"minecraft:pufferfish_puffed": {"minecraft:scale": {"value": 7.5}}, "minecraft:full_puff": {"minecraft:variant": {"value": 9}}, "keep": {"minecraft:scale": {"value": 2.5}}}, "components": {"minecraft:health": {"value": 3}}, "events": {"minecraft:on_puff": {"add": {"component_groups": ["minecraft:pufferfish_puffed"]}}, "minecraft:on_deflate": {}}"#,
    );
    // The group and the event the fixer takes out, and the group of its own it writes over
    // the file's; the empty event holds nothing and is not noted.
    has(
        &found,
        "DroppedField Info component_groups/minecraft:pufferfish_puffed",
    );
    has(
        &found,
        "DroppedField Info component_groups/minecraft:full_puff",
    );
    has(&found, "DroppedField Info events/minecraft:on_puff");
    assert!(
        !found
            .iter()
            .any(|f| f.ends_with("events/minecraft:on_deflate")),
        "{found:#?}"
    );
    assert!(
        !found.iter().any(|f| f.ends_with("component_groups/keep")),
        "{found:#?}"
    );
    assert!(!written.contains("7.5"), "{written}");
    assert!(written.contains("2.5"), "{written}");
}

#[test]
fn a_component_a_step_renames_over_one_the_file_has_is_noted() {
    for (version, old, new) in [
        (
            "1.16.200",
            "minecraft:scaffolding_climber",
            "minecraft:block_climber",
        ),
        (
            "1.21.90",
            "minecraft:body_rotation_axis_aligned",
            "minecraft:rotation_axis_aligned",
        ),
        (
            "1.26.10",
            "minecraft:behavior.follow_target_captain",
            "minecraft:behavior.follow_target_leader",
        ),
    ] {
        let (found, _) = told(
            version,
            "demo:x",
            &format!(r#""components": {{"{old}": {{"priority": 1}}, "{new}": {{"priority": 2}}}}"#),
        );
        has(&found, &format!("DroppedField Info components/{new}"));
        // The same value in both: nothing is replaced.
        let (found, _) = told(
            version,
            "demo:x",
            &format!(r#""components": {{"{old}": {{"priority": 1}}, "{new}": {{"priority": 1}}}}"#),
        );
        // (A marker's legacy reader tells `priority` itself: not the step's note.)
        has_not(&found, &format!("DroppedField Info components/{new}"));
    }
}

#[test]
fn a_finding_of_a_member_a_step_moved_names_the_member_of_the_file() {
    // (rows: version, components, the path told, the path
    // the upgraded document has, which is not told).
    let rows = [
        (
            "1.20.70",
            r#"{"minecraft:behavior.follow_target_captain": {"priority": "x"}}"#,
            "components/minecraft:behavior.follow_target_captain/priority",
            "follow_target_leader",
        ),
        (
            "1.16.100",
            r#"{"minecraft:scaffolding_climber": 7}"#,
            "components/minecraft:scaffolding_climber",
            "block_climber",
        ),
        (
            "1.21.130",
            r#"{"minecraft:behavior.look_at_player": {"min_look_time": 1e39, "max_look_time": 4}}"#,
            "components/minecraft:behavior.look_at_player/min_look_time",
            "look_at_player/look_time",
        ),
        (
            "1.12.0",
            r#"{"minecraft:interact": [{"on_interact": {"event": 7}}]}"#,
            "components/minecraft:interact/0/on_interact",
            "interactions",
        ),
        (
            "1.16.0",
            r#"{"minecraft:behavior.drop_item_for": {"priority": 1, "max_dist": "x"}}"#,
            "components/minecraft:behavior.drop_item_for/max_dist",
            "target_range",
        ),
    ];
    let mut wrong = Vec::new();
    for (version, components, at, not_at) in rows {
        let (found, _) = told(version, "demo:x", &format!(r#""components": {components}"#));
        if !found
            .iter()
            .any(|f| f.ends_with(&format!(" {at}")) || f.contains(&format!(" {at}/")))
        {
            wrong.push(format!(
                "{version} {components}: nothing at {at} in {found:?}"
            ));
        }
        if found.iter().any(|f| f.contains(not_at)) {
            wrong.push(format!(
                "{version} {components}: a finding names {not_at}: {found:?}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

// --- more cases ---------

#[track_caller]
fn has_not(found: &[String], finding: &str) {
    assert!(
        !found.iter().any(|f| f == finding),
        "`{finding}` in {found:#?}"
    );
}

#[test]
fn case_r2_1_a_movement_type_beside_simulate_waves_is_noted_as_replaced() {
    let at = "DroppedField Info components/minecraft:buoyant/movement_type";
    let (found, written) = told(
        "1.26.0",
        "demo:x",
        r#""components": {"minecraft:buoyant": {"simulate_waves": false, "movement_type": "waves"}}"#,
    );
    has(&found, at);
    assert!(written.contains(r#""movement_type": "none""#), "{written}");
    let (found, written) = told(
        "1.26.0",
        "demo:x",
        r#""components": {"minecraft:buoyant": {"simulate_waves": true, "movement_type": "bobbing"}}"#,
    );
    has(&found, at);
    assert!(written.contains(r#""movement_type": "waves""#), "{written}");
    // The same value: nothing is replaced.
    let (found, _) = told(
        "1.26.0",
        "demo:x",
        r#""components": {"minecraft:buoyant": {"simulate_waves": true, "movement_type": "waves"}}"#,
    );
    has_not(&found, at);
}

#[test]
fn case_r2_2_a_pushable_by_component_the_pushable_split_replaces_is_noted() {
    let entity = "DroppedField Info components/minecraft:pushable_by_entity";
    let block = "DroppedField Info components/minecraft:pushable_by_block";
    let (found, written) = told(
        "1.26.0",
        "demo:x",
        r#""components": {"minecraft:pushable": {"is_pushable": true}, "minecraft:pushable_by_entity": {"use_default_with_overlap_check": true, "presets": [{"pushable_by": "all"}]}, "minecraft:pushable_by_block": {"b": 2}}"#,
    );
    has(&found, entity);
    has(&found, block);
    assert!(
        written.contains(r#""minecraft:pushable_by_block": {}"#),
        "{written}"
    );
    assert!(!written.contains("pushable_by\""), "{written}");
    // A member that does not hold leaves the file's component alone; an empty one of the
    // file is what the step writes.
    let (found, written) = told(
        "1.26.0",
        "demo:x",
        r#""components": {"minecraft:pushable": {"is_pushable": false}, "minecraft:pushable_by_entity": {"presets": [{"pushable_by": "all"}]}, "minecraft:pushable_by_block": {}}"#,
    );
    has_not(&found, entity);
    has_not(&found, block);
    // `pushable_by` is no member of the newest preset: told, not kept.
    // The legacy reader of `pushable_by_entity` (typed first at 1.26.20) has no child
    // (it binds nothing): the file's component is the game's unknown-member line.
    has(
        &found,
        "UnrecognizedField Error components/minecraft:pushable_by_entity/presets",
    );
    assert!(!written.contains(r#""pushable_by": "all""#), "{written}");
    // In a group, at the group.
    let (found, _) = told(
        "1.26.0",
        "demo:x",
        r#""component_groups": {"g": {"minecraft:pushable": {}, "minecraft:pushable_by_block": {"b": 2}}}"#,
    );
    has(
        &found,
        "DroppedField Info component_groups/g/minecraft:pushable_by_block",
    );
}

#[test]
fn case_r2_3_an_impact_damage_knockback_flag_the_step_sets_is_noted() {
    let (found, written) = told(
        "1.21.100",
        "demo:x",
        r#""components": {"minecraft:projectile": {"on_hit": {"impact_damage": {"damage": 1, "apply_knockback_to_blocking_targets": false}}}}"#,
    );
    has(
        &found,
        "DroppedField Info components/minecraft:projectile/on_hit/impact_damage/apply_knockback_to_blocking_targets",
    );
    assert!(
        written.contains(r#""apply_knockback_to_blocking_targets": true"#),
        "{written}"
    );
    let (found, _) = told(
        "1.21.100",
        "demo:x",
        r#""component_groups": {"g": {"minecraft:projectile": {"on_hit": {"impact_damage": {"damage": 1, "apply_knockback_to_blocking_targets": "x"}}}}}"#,
    );
    has(
        &found,
        "DroppedField Info component_groups/g/minecraft:projectile/on_hit/impact_damage/apply_knockback_to_blocking_targets",
    );
    let (found, _) = told(
        "1.21.100",
        "demo:x",
        r#""components": {"minecraft:projectile": {"on_hit": {"impact_damage": {"damage": 1, "apply_knockback_to_blocking_targets": true}}}}"#,
    );
    assert!(
        !found.iter().any(|f| f.starts_with("DroppedField")),
        "{found:#?}"
    );
}

#[test]
fn case_r2_4_a_subsensors_that_is_no_list_is_noted_as_replaced() {
    let (found, written) = told(
        "1.19.50",
        "demo:x",
        r#""components": {"minecraft:entity_sensor": {"sensor_range": 5, "subsensors": {"range": 9, "event": "x:f"}}}"#,
    );
    has(
        &found,
        "InvalidType Info components/minecraft:entity_sensor/subsensors",
    );
    assert!(!written.contains("x:f"), "{written}");
    let (found, _) = told(
        "1.19.50",
        "demo:x",
        r#""component_groups": {"g": {"minecraft:entity_sensor": {"sensor_range": 5, "subsensors": "zz"}}}"#,
    );
    has(
        &found,
        "InvalidType Info component_groups/g/minecraft:entity_sensor/subsensors",
    );
    // A list of the file is kept behind the one the root members make.
    let (found, written) = told(
        "1.19.50",
        "demo:x",
        // information, told at `subsensors/0/event`, is a finding about `subsensors`)
        r#""components": {"minecraft:entity_sensor": {"sensor_range": 5, "subsensors": [{"range": 9, "event": "x:f"}]}}, "events": {"x:f": {}}"#,
    );
    assert!(
        !found.iter().any(|f| f.contains("subsensors")),
        "{found:#?}"
    );
    assert!(written.contains("x:f"), "{written}");
}

#[test]
fn a_finding_of_a_subsensor_the_1_20_0_step_shifted_names_the_member_of_the_file() {
    // the root members become `subsensors[0]` and the
    // file's own subsensors move one place on. Nothing tells of a subsensor today (the
    // sensor is held as written), so the step is run alone and a finding of a later check
    // is told at each place of its output.
    use bedrock_addon_core::diagnostic::{
        Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
    };
    use bedrock_addon_core::json::JsonSlice;
    use bedrock_addon_core::vanilla::ActorType;
    use bedrock_addon_core::version::FormatVersionInput;
    use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};
    use bedrock_addon_family_entity::history::v1_20_0;

    let payload = r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:entity_sensor": {"sensor_range": 5, "event": "e", "subsensors": [{"range": 1}, {"range": 2}]}}}"#;
    let load = cx();
    let version = <v1_20_0::Entity as Versioned>::VERSION;
    let parse = ParseCx {
        load: &load,
        written: FormatVersionInput {
            version: Some(version),
            beta_literal: false,
            prerelease: false,
        },
        original_version: version,
        at: JsonPath::new(),
    };
    let mut found = Diagnostics::default();
    let entity = v1_20_0::Entity::parse(JsonSlice::new(payload), &parse, &mut found)
        .expect("the 1.20.0 struct takes it");
    let _ = entity.upgrade(&mut UpgradeCx {
        load: &load,
        original_version: version,
        identifier: "demo:x",
        actor_type: ActorType::Mob,
        skip_section_a: false,
        diagnostics: &mut found,
    });
    let sensor = "minecraft:entity/components/minecraft:entity_sensor";
    let rows = [
        ("subsensors/0/range", "sensor_range"),
        ("subsensors/0/event", "event"),
        ("subsensors/1/range", "subsensors/0/range"),
        ("subsensors/2/range", "subsensors/1/range"),
        ("subsensors/2", "subsensors/1"),
    ];
    for (after, _) in rows {
        let mut at = JsonPath::new();
        for segment in format!("{sensor}/{after}").split('/') {
            match segment.parse::<usize>() {
                Ok(index) => at.push_index(index),
                Err(_) => at.push(segment),
            }
        }
        found.push(Diagnostic::new(
            DiagnosticCode::InvalidValue { text: None },
            Severity::Warning,
            Phase::Validate,
            Location::new(None, None, at, None),
        ));
    }
    found.name_as_the_file_has_it();
    let told: Vec<String> = found
        .iter()
        .map(|f| f.location.json_path.to_string())
        .collect();
    let expected: Vec<String> = rows
        .iter()
        .map(|(_, before)| format!("{sensor}/{before}"))
        .collect();
    assert_eq!(told, expected);
}

#[test]
fn a_legacy_goal_member_that_is_an_empty_list_goes_alone() {
    // The lenient goal read gives up the member a failure names; a failure named BELOW a
    // member that has no such element (`priority: []` fails at `priority/0`) took the whole
    // goal's members with it, told at an element the file does not have.
    // `random_stroll`'s legacy reader is hand-written; the
    // schema reader of `melee_attack` (typed first at 1.26.0) fails on the list from 1.15.0
    // The game's type line, the goal at its defaults.
    for (goal, finding, kept) in [
        ("random_stroll", "InvalidValue Info", true),
        ("melee_attack", "InvalidType Error", false),
    ] {
        let (found, written) = told(
            "1.16.0",
            "demo:x",
            &format!(
                r#""components": {{"minecraft:behavior.{goal}": {{"priority": [], "speed_multiplier": 1.5}}}}"#
            ),
        );
        // `random_stroll`'s hand-written reader says nothing: an information.
        has(
            &found,
            &format!("{finding} components/minecraft:behavior.{goal}/priority"),
        );
        assert_eq!(
            written.contains(r#""speed_multiplier": 1.5"#),
            kept,
            "{goal}: {written}"
        );
    }
}

#[test]
fn an_entry_of_a_legacy_interact_list_that_does_not_read_goes_alone_and_is_told_where_it_is() {
    // the list went whole for its first entry, told at
    // `minecraft:interact` (as if the component were dropped), and the second entry's own
    // fault had no finding.
    let (found, written) = told(
        "1.12.0",
        "demo:x",
        r#""components": {"minecraft:interact": [{"swing": "x", "cooldown": 1}, {"cooldown": 2.5, "use_item": true}, {"cooldown": "c"}]}"#,
    );
    // Below 1.15.0 the legacy schema reader takes out each member no option takes, with the
    // game's type line, and keeps the entry.
    has(
        &found,
        "InvalidType Error components/minecraft:interact/0/swing",
    );
    has(
        &found,
        "InvalidType Error components/minecraft:interact/2/cooldown",
    );
    has_not(&found, "DroppedField Info components/minecraft:interact/0");
    has_not(&found, "DroppedField Info components/minecraft:interact");
    assert!(written.contains(r#""cooldown": 1"#), "{written}");
    assert!(written.contains(r#""cooldown": 2.5"#), "{written}");
    assert!(written.contains(r#""use_item": true"#), "{written}");
}

#[test]
fn what_a_step_takes_apart_and_does_not_read_is_told_where_the_file_has_it() {
    // The judge's inputs with an unknown member, one more element, and a member a step adds
    // Rows of version, identifier, payload members, the finding.
    let rows = [
        (
            "1.12.0",
            "demo:x",
            r#""components": {"minecraft:fall_damage": {"value": 2, "zz": 3}}"#,
            "UnrecognizedField Info components/minecraft:fall_damage/zz",
        ),
        (
            "1.12.0",
            "demo:x",
            r#""components": {"minecraft:scheduler": {"scheduled_events": [{"zz": 3, "scheduled_event": {"event": "e"}}]}}"#,
            "UnrecognizedField Info components/minecraft:scheduler/scheduled_events/0/zz",
        ),
        (
            "1.12.0",
            "demo:x",
            r#""component_groups": {"g": {"minecraft:environment_sensor": [{"zz": 3, "on_environment": {"event": "e"}}]}}"#,
            "UnrecognizedField Info component_groups/g/minecraft:environment_sensor/0/zz",
        ),
        (
            "1.21.120",
            "demo:x",
            r#""components": {"minecraft:breedable": {"breeds_with": [{"mate_type": "demo:a", "baby_type": "demo:b"}, {"mate_type": 7, "baby_type": "demo:c"}]}}"#,
            "DroppedField Info components/minecraft:breedable/breeds_with/1",
        ),
        (
            "1.21.120",
            "demo:x",
            r#""components": {"minecraft:breedable": {"breeds_with": {"mate_type": 7, "baby_type": "demo:c"}}}"#,
            "DroppedField Info components/minecraft:breedable/breeds_with",
        ),
        (
            "1.18.0",
            "minecraft:ocelot",
            r#""components": {}, "component_groups": {"minecraft:wild_child_ocelot_spawn": {"minecraft:spawn_entity": {"entities": {"spawn_entity": "demo:mine"}}}}, "events": {"minecraft:entity_spawned": {}}"#,
            "DroppedField Info component_groups/minecraft:wild_child_ocelot_spawn/minecraft:spawn_entity",
        ),
        (
            "1.21.100",
            "demo:x",
            r#""components": {"minecraft:behavior.take_block": {"priority": 1, "xz_range": [1, 2, 9]}}"#,
            "UnrecognizedField Info components/minecraft:behavior.take_block/xz_range/2",
        ),
        // (From 1.21.89 a strict slice refuses it: told below.) An element beyond
        // the two the legacy range reader reads is the crate's word: an information.
        (
            "1.21.80",
            "demo:x",
            r#""components": {"minecraft:behavior.place_block": {"priority": 1, "y_range": [1, 2, 9]}}"#,
            "UnrecognizedField Info components/minecraft:behavior.place_block/y_range/2",
        ),
    ];
    let mut wrong = Vec::new();
    for (version, identifier, rest, finding) in rows {
        let (found, _) = told(version, identifier, rest);
        if !found.iter().any(|f| f == finding) {
            wrong.push(format!("{version} {rest}: no `{finding}` in {found:?}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// `ranged_attack`'s legacy reader (a schema reader, typed first at 1.26.40)
/// validates before the conversion: a half that is no number is the game's type line; strict
/// (1.15.0 on) the goal keeps its defaults, lenient the half alone is taken out and the other
/// half converts.
#[test]
fn a_half_of_a_1_26_30_ranged_interval_that_is_no_number_is_the_game_s_type_line() {
    let rest = r#""components": {"minecraft:behavior.ranged_attack": {"attack_interval_min": "x", "attack_interval_max": 3}}"#;
    for (version, half_kept) in [("1.26.30", false), ("1.14.0", true)] {
        let (found, written) = told(version, "demo:x", rest);
        has(
            &found,
            "InvalidType Error components/minecraft:behavior.ranged_attack/attack_interval_min",
        );
        assert!(
            !found
                .iter()
                .any(|f| f.ends_with("ranged_attack/attack_interval")),
            "{version}: {found:#?}"
        );
        assert_eq!(
            written.contains(r#""max": 3"#),
            half_kept,
            "{version}: {written}"
        );
    }
}

#[test]
fn b2_a_hurt_when_wet_that_makes_no_rain_entry_is_told_at_the_component() {
    // a `hurt_on_condition` of the file without a
    // `damage_conditions` list gets no entry, and the component went
    // without a word.
    for (rest, at) in [
        (
            r#""components": {"minecraft:hurt_on_condition": {"zz": 1}, "minecraft:hurt_when_wet": {"value": true}}"#,
            "components",
        ),
        (
            r#""components": {"minecraft:hurt_on_condition": {"damage_conditions": {"cause": "lava"}}, "minecraft:hurt_when_wet": true}"#,
            "components",
        ),
        (
            r#""components": {}, "component_groups": {"g": {"minecraft:hurt_on_condition": 5, "minecraft:hurt_when_wet": true}}"#,
            "component_groups/g",
        ),
    ] {
        let (found, written) = told("1.12.0", "demo:x", rest);
        has(
            &found,
            &format!("DroppedComponent Info {at}/minecraft:hurt_when_wet"),
        );
        assert!(!written.contains("in_water_or_rain"), "{written}");
    }
    // A list takes the entry, and nothing is to tell.
    let (found, written) = told(
        "1.12.0",
        "demo:x",
        r#""components": {"minecraft:hurt_on_condition": {"damage_conditions": []}, "minecraft:hurt_when_wet": {"value": true}}"#,
    );
    assert!(written.contains("in_water_or_rain"), "{written}");
    assert!(
        !found.iter().any(|f| f.contains("hurt_when_wet")),
        "{found:#?}"
    );
}

#[test]
fn b2_a_member_of_hurt_when_wet_beside_its_value_is_told() {
    let (found, written) = told(
        "1.12.0",
        "demo:x",
        r#""components": {"minecraft:hurt_when_wet": {"value": true, "zz": 3}}"#,
    );
    has(
        &found,
        "UnrecognizedField Info components/minecraft:hurt_when_wet/zz",
    );
    assert!(written.contains("in_water_or_rain"), "{written}");
}

#[test]
fn b3a_a_fall_damage_beside_a_damage_sensor_without_a_trigger_list_is_carried() {
    // the step took `fall_damage` and returned at a
    // `triggers` that is no list, so the component and its 2.5 went without a word. The
    // value that is no list is replaced by the list of the `fall` trigger, and told.
    for (section, sensor, at) in [
        ("components", r#"{"triggers": "none"}"#, "/triggers"),
        ("components", r#"{"triggers": true}"#, "/triggers"),
        ("component_groups/g", "7", ""),
    ] {
        let members = format!(
            r#"{{"minecraft:damage_sensor": {sensor}, "minecraft:fall_damage": {{"value": 2.5}}}}"#
        );
        let rest = if section == "components" {
            format!(r#""components": {members}"#)
        } else {
            format!(r#""components": {{}}, "component_groups": {{"g": {members}}}"#)
        };
        let (found, written) = told("1.12.0", "demo:x", &rest);
        assert!(
            written.contains(r#""damage_multiplier": 2.5"#),
            "{sensor}: {written}"
        );
        has(
            &found,
            &format!("InvalidType Info {section}/minecraft:damage_sensor{at}"),
        );
    }
    // A `null` sensor is an absent one: the trigger is made and nothing is told.
    let (found, written) = told(
        "1.12.0",
        "demo:x",
        r#""components": {"minecraft:damage_sensor": null, "minecraft:fall_damage": {"value": 2.5}}"#,
    );
    assert!(written.contains(r#""damage_multiplier": 2.5"#), "{written}");
    assert!(
        !found.iter().any(|f| f.contains("damage_sensor")),
        "{found:#?}"
    );
}

#[test]
fn b3b_a_stop_if_holding_item_of_the_file_below_1_26_30_is_told() {
    // the member gate forgets the member below 1.26.30 and
    // its finding was held back as if the 1.26.20 step had written the `false`.
    let at = "components/minecraft:behavior.pickup_items/stop_if_holding_item";
    // At 1.26.20 the goal is read by its typed struct, which lacks the member: the strict
    // slice refuses the document.
    let refused = super::common::outcome(
        &doc(
            "1.26.20",
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.pickup_items": {"priority": 1, "stop_if_holding_item": true}}}"#,
        ),
        &cx(),
    );
    assert!(refused.value.is_none());
    for version in ["1.20.0"] {
        for value in ["false", "true"] {
            let (found, _) = told(
                version,
                "demo:x",
                &format!(
                    r#""components": {{"minecraft:behavior.pickup_items": {{"priority": 1, "stop_if_holding_item": {value}}}}}"#
                ),
            );
            let here: Vec<_> = found.iter().filter(|f| f.ends_with(at)).collect();
            assert_eq!(
                here,
                [&format!("UnrecognizedField Info {at}")],
                "{version} {value}: {found:#?}"
            );
        }
        // The `false` the step writes is not the file's: nothing is told.
        let (found, _) = told(
            version,
            "demo:x",
            r#""components": {"minecraft:behavior.pickup_items": {"priority": 1}}"#,
        );
        assert!(!found.iter().any(|f| f.ends_with(at)), "{found:#?}");
    }
}

#[test]
fn b3c_the_upgrades_that_go_with_an_entity_version_are_told() {
    // an `entity_version` that is refused, or gated, takes
    // the payload's `upgrades` with it, and the one finding
    // was at `description/entity_version`.
    // The 1.26.20 toggle gate is no refusal: it takes nothing with it.
    for (version, entity_version) in [
        ("1.26.30", r#""x""#),
        ("1.26.30", r#"{"upgrades": [{"upgrade_to": 1}]}"#),
    ] {
        let (found, written) = told(
            version,
            "demo:x",
            &format!(
                r#""description": {{"identifier": "demo:x", "entity_version": {entity_version}}}, "components": {{}}, "upgrades": {{"h": {{}}}}"#
            ),
        );
        // An information of the crate's own (no game line for the member).
        has(&found, "DroppedField Info upgrades");
        assert!(!written.contains("upgrades"), "{written}");
    }
    // A failure inside `entity_version` takes the whole member: told at the member too.
    let (found, _) = told(
        "1.26.30",
        "demo:x",
        r#""description": {"identifier": "demo:x", "entity_version": {"upgrades": [{"upgrade_to": 1}]}}, "components": {}"#,
    );
    has(
        &found,
        // The game's level-3 structure line, a kept error.
        "InvalidValue Error description/entity_version/upgrades/0/name",
    );
    has(&found, "DroppedField Info description/entity_version");
    has_not(&found, "DroppedField Info upgrades");
    // The handlers of a version that reads stay, and nothing is told.
    let (found, written) = told(
        "1.26.30",
        "demo:x",
        r#""description": {"identifier": "demo:x", "entity_version": {"upgrades": [{"upgrade_to": 1, "name": "h"}]}}, "components": {}, "upgrades": {"h": {}}"#,
    );
    assert!(written.contains(r#""upgrades": { "h": {} }"#), "{written}");
    assert!(
        !found.iter().any(|f| f.starts_with("DroppedField")),
        "{found:#?}"
    );
}

#[test]
fn b3d_a_member_of_entity_version_the_reader_does_not_read_is_told() {
    // the version is rebuilt from `upgrades`, and an entry
    // from `upgrade_to` and `name`.
    let (found, written) = told(
        "1.26.30",
        "demo:x",
        r#""description": {"identifier": "demo:x", "entity_version": {"upgrades": [{"upgrade_to": 1, "name": "h", "zz": 3}], "yy": 4}}, "components": {}, "upgrades": {"h": {}}"#,
    );
    has(
        &found,
        "UnrecognizedField Info description/entity_version/yy",
    );
    has(
        &found,
        "UnrecognizedField Info description/entity_version/upgrades/0/zz",
    );
    assert!(written.contains(r#""name": "h""#), "{written}");
}

#[test]
fn b3e_the_skip_key_and_a_payload_version_that_is_not_the_document_s_are_told() {
    // `do_not_upgrade` is read as a flag and
    // `format_version` in the payload by the version selection; neither is written.
    for version in ["1.16.0", "1.26.50"] {
        let (found, written) = told(
            version,
            "demo:x",
            r#""components": {}, "do_not_upgrade": true"#,
        );
        has(&found, "DroppedField Info do_not_upgrade");
        assert!(!written.contains("do_not_upgrade"), "{written}");
        let (found, _) = told(
            version,
            "demo:x",
            r#""components": {}, "format_version": "1.2.0""#,
        );
        has(&found, "UnrecognizedField Info format_version");
        // The payload's version that IS the document's says nothing.
        let (found, _) = told(
            version,
            "demo:x",
            &format!(r#""components": {{}}, "format_version": "{version}""#),
        );
        assert!(
            !found.iter().any(|f| f.ends_with("format_version")),
            "{found:#?}"
        );
    }
}

/// The written entity of `told` loads again without an error and is written as the same
/// text.
#[track_caller]
fn rereads(written: &str) {
    let (again, found) = stable(written, &cx());
    assert!(
        !found
            .iter()
            .any(|f| f.severity == bedrock_addon_core::diagnostic::Severity::Error),
        "{found:#?}"
    );
    let rewritten = again.to_json().expect("the entity writes");
    assert_eq!(
        rewritten.split_whitespace().collect::<Vec<_>>().join(" "),
        written
    );
}

#[test]
fn a_ranged_radius_pair_whose_min_is_above_its_max_is_the_range_as_written() {
    // The game's 1.26.40 → 1.26.50 upgrade makes `attack_range`
    // {min = `attack_radius_min`, max = `attack_radius`} with no check, and
    // the typed range binds `min` / `max` with no order
    // constraint: the pair is the range in the order written, no member is unknown, and the
    // written text reads again to itself.
    for version in ["1.26.40", "1.20.0"] {
        let (found, written) = told(
            version,
            "demo:x",
            r#""components": {"minecraft:behavior.ranged_attack": {"priority": 1, "attack_radius_min": 10, "attack_radius": 2}}"#,
        );
        assert!(
            !found
                .iter()
                .any(|f| f.contains("Warning") || f.contains("Error")),
            "{version}: {found:#?}"
        );
        assert!(
            // (The 1.26.40 struct holds `f32`s, written `4.0`.)
            written
                .replace(".0", "")
                .contains(r#""attack_range": { "min": 10, "max": 2 }"#),
            "{version}: {written}"
        );
        assert!(
            !written.contains("attack_radius_min") && !written.contains(r#""attack_radius""#),
            "{version}: {written}"
        );
        rereads(&written);
        // A pair in order is the range, and nothing is told of it.
        let (found, written) = told(
            version,
            "demo:x",
            r#""components": {"minecraft:behavior.ranged_attack": {"priority": 1, "attack_radius_min": 4, "attack_radius": 15}}"#,
        );
        assert!(
            !found.iter().any(|f| f.contains("attack_ra")),
            "{version}: {found:#?}"
        );
        assert!(
            written.contains(r#""attack_range": { "min": 4"#),
            "{written}"
        );
        rereads(&written);
    }
}

#[test]
fn m2_a_half_of_a_ranged_radius_is_told_at_the_member_the_file_has() {
    // a half that is no number took the other half with it,
    // told at `attack_range`, a member the file does not have.
    let at = "components/minecraft:behavior.ranged_attack";
    let (found, written) = told(
        "1.20.0",
        "demo:x",
        r#""components": {"minecraft:behavior.ranged_attack": {"priority": 1, "attack_radius_min": "near", "attack_radius": 9}}"#,
    );
    // The legacy schema reader's type line; strict at 1.20.0, the goal at its
    // defaults.
    has(&found, &format!("InvalidType Error {at}/attack_radius_min"));
    assert!(
        !found.iter().any(|f| f.contains("attack_range")),
        "{found:#?}"
    );
    assert!(!written.contains(r#""max": 9"#), "{written}");
}

#[test]
fn a_lower_radius_alone_is_the_range_up_to_the_default_upper_end() {
    // The legacy `attack_radius` defaults to 0, so a lower end alone is
    // `attack_range {min: 4, max: 0}`, as the game's upgrade makes it; nothing is unknown.
    for version in ["1.26.40", "1.20.0"] {
        let (found, written) = told(
            version,
            "demo:x",
            r#""components": {"minecraft:behavior.ranged_attack": {"priority": 1, "attack_radius_min": 4}}"#,
        );
        assert!(
            !found
                .iter()
                .any(|f| f.contains("Warning") || f.contains("Error")),
            "{version}: {found:#?}"
        );
        assert!(
            // (The 1.26.40 struct holds `f32`s, written `4.0`.)
            written
                .replace(".0", "")
                .contains(r#""attack_range": { "min": 4, "max": 0 }"#),
            "{version}: {written}"
        );
        assert!(!written.contains("attack_radius_min"), "{written}");
        rereads(&written);
    }
}

/// The legacy `addrider` makes its one rider only of an `entity_type` that is not
/// empty; a `spawn_event` without one is read and never used.
/// The crate drops it with an information, and no rider is written.
#[test]
fn a_legacy_addrider_without_an_entity_type_makes_no_rider() {
    let at = "components/minecraft:addrider";
    for (component, dropped) in [
        (
            r#""minecraft:addrider": {"spawn_event": "minecraft:on_spawn"}"#,
            vec!["spawn_event"],
        ),
        (
            r#""minecraft:addrider": {"entity_type": "", "spawn_event": "minecraft:on_spawn"}"#,
            vec!["entity_type", "spawn_event"],
        ),
    ] {
        let (found, written) = told(
            "1.20.0",
            "demo:x",
            &format!(r#""components": {{{component}}}"#),
        );
        assert!(
            !found
                .iter()
                .any(|f| f.contains("Warning") || f.contains("Error")),
            "{component}: {found:#?}"
        );
        for member in dropped {
            has(&found, &format!("DroppedField Info {at}/{member}"));
        }
        assert!(
            written.contains(r#""riders": []"#)
                && !written.contains("spawn_event")
                && !written.contains("entity_type"),
            "{component}: {written}"
        );
        rereads(&written);
    }
}

#[test]
fn m2_a_finding_of_a_member_a_lenient_component_read_moved_names_the_member_of_the_file() {
    // the legacy single rider (`riders/0`), the legacy single
    // effect of a projectile (`mob_effect/effects/0`) and the projectile's aliases were
    // moved without a record, so the typed read told of members the file does not have.
    let rows = [
        // `addrider`'s legacy reader is a schema reader (typed first at 1.21.130): the game's
        // type line at the member the file has.
        (
            r#""minecraft:addrider": {"entity_type": 5}"#,
            "InvalidType Error components/minecraft:addrider/entity_type",
            "riders",
        ),
        (
            r#""minecraft:addrider": {"entity_type": "minecraft:skeleton", "spawn_event": 7}"#,
            "InvalidType Error components/minecraft:addrider/spawn_event",
            "riders",
        ),
        (
            r#""minecraft:projectile": {"on_hit": {"mob_effect": {"effect": "poison", "amplifier": "strong", "durationeasy": 5}}}"#,
            "InvalidValue Info components/minecraft:projectile/on_hit/mob_effect/amplifier",
            "effects",
        ),
        (
            // A number, not a bool: the 1.26.30 step makes a bool the name it stands for
            // under the alias too.
            r#""minecraft:projectile": {"shouldbounce": 7}"#,
            "InvalidValue Info components/minecraft:projectile/shouldbounce",
            "projectile/should_bounce",
        ),
        (
            r#""minecraft:projectile": {"hitWater": "yes"}"#,
            "InvalidValue Info components/minecraft:projectile/hitWater",
            "projectile/hit_water",
        ),
        (
            r#""minecraft:projectile": {"on_hit": {"impact_damage": {"catchFire": "yes"}}}"#,
            "InvalidValue Info components/minecraft:projectile/on_hit/impact_damage/catchFire",
            "impact_damage/catch_fire",
        ),
    ];
    let mut wrong = Vec::new();
    for (component, finding, not_at) in rows {
        let (found, _) = told(
            "1.16.0",
            "demo:x",
            &format!(r#""components": {{{component}}}"#),
        );
        if !found.iter().any(|f| f == finding) {
            wrong.push(format!("{component}: no `{finding}` in {found:?}"));
        }
        if let Some(elsewhere) = found.iter().find(|f| f.contains(not_at)) {
            wrong.push(format!("{component}: `{elsewhere}`"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn a_bool_under_the_alias_shouldbounce_is_converted_as_should_bounce_is() {
    // A bool under the legacy reader's alias `shouldbounce` is converted as `should_bounce`
    // is: it is neither told `InvalidValue` nor read as the default.
    let at = "components/minecraft:projectile";
    for version in ["1.16.0", "1.26.30"] {
        for (flag, name) in [("true", "if_invulnerable"), ("false", "no")] {
            let projectile = |member: &str| {
                told(
                    version,
                    "demo:x",
                    &format!(r#""components": {{"minecraft:projectile": {{"{member}": {flag}}}}}"#),
                )
            };
            let (found, written) = projectile("shouldbounce");
            assert!(found.is_empty(), "{version} {flag}: {found:#?}");
            assert!(
                written.contains(&format!(r#""should_bounce": "{name}""#)),
                "{version} {flag}: {written}"
            );
            assert_eq!(
                (found, written),
                projectile("should_bounce"),
                "{version} {flag}"
            );
        }
        // A string outside the set is the step's to tell, at the member the file has.
        let (found, written) = told(
            version,
            "demo:x",
            r#""components": {"minecraft:projectile": {"shouldbounce": "sometimes"}}"#,
        );
        assert_eq!(
            found,
            [format!("InvalidValue Error {at}/shouldbounce")],
            "{version}"
        );
        assert!(written.contains(r#""should_bounce": "no""#), "{written}");
        // The member beside its alias is the one read; the alias stays an unknown member.
        let (found, written) = told(
            version,
            "demo:x",
            r#""components": {"minecraft:projectile": {"shouldbounce": true, "should_bounce": "if_no_damage_dealt"}}"#,
        );
        assert_eq!(
            found,
            [format!("UnrecognizedField Info {at}/shouldbounce")],
            "{version}"
        );
        assert!(
            written.contains(r#""should_bounce": "if_no_damage_dealt""#),
            "{written}"
        );
        // Told, not kept.
        assert!(!written.contains(r#""shouldbounce""#), "{written}");
    }
    // From 1.26.40 the member is a name: no step converts a bool, under either spelling.
    for member in ["shouldbounce", "should_bounce"] {
        let (found, written) = told(
            "1.26.40",
            "demo:x",
            &format!(r#""components": {{"minecraft:projectile": {{"{member}": true}}}}"#),
        );
        assert_eq!(found, [format!("InvalidValue Info {at}/{member}")]);
        assert!(written.contains(r#""should_bounce": "no""#), "{written}");
    }
}

#[test]
fn k92_the_impact_damage_of_a_projectile_under_its_aliases_is_converted_by_the_1_26_30_step() {
    // The same step's other migration (`impact_damage`) read the member
    // names alone: the old default it writes into `power_multiplier` took the place of the
    // file's `powermultiplier`, and nothing below `onHit` was converted.
    for version in ["1.21.120", "1.26.30"] {
        let projectile = |members: &str| {
            told(
                version,
                "demo:x",
                &format!(r#""components": {{"minecraft:projectile": {{{members}}}}}"#),
            )
        };
        // The file's multiplier under its alias is the multiplier.
        let (found, written) = projectile(r#""on_hit": {"impact_damage": {"powermultiplier": 5}}"#);
        assert!(found.is_empty(), "{version}: {found:#?}");
        assert!(written.contains(r#""power_multiplier": 5"#), "{written}");
        assert_eq!(
            (found, written),
            projectile(r#""on_hit": {"impact_damage": {"power_multiplier": 5}}"#),
            "{version}"
        );
        // `onHit` is `on_hit`: the old default and the old flag below it are converted.
        let (found, written) =
            projectile(r#""onHit": {"impact_damage": {"semi_random_diff_damage": true}}"#);
        assert!(found.is_empty(), "{version}: {found:#?}");
        assert!(written.contains(r#""power_multiplier": 2"#), "{written}");
        assert!(
            written.contains(r#""difficulty_randomization": "additive""#),
            "{written}"
        );
        assert_eq!(
            (found, written),
            projectile(r#""on_hit": {"impact_damage": {"semi_random_diff_damage": true}}"#),
            "{version}"
        );
        // What the typed read tells below a renamed member names the member of the file.
        let (found, _) = projectile(r#""onHit": {"impact_damage": {"powermultiplier": "much"}}"#);
        // The findings are those of the members, each at the member the file has.
        let (of_the_members, _) =
            projectile(r#""on_hit": {"impact_damage": {"power_multiplier": "much"}}"#);
        let as_written: Vec<String> = of_the_members
            .iter()
            .map(|f| {
                f.replace("on_hit", "onHit")
                    .replace("power_multiplier", "powermultiplier")
            })
            .collect();
        assert_eq!(found, as_written, "{version}");
        assert_eq!(
            found[0],
            "InvalidValue Info components/minecraft:projectile/onHit/impact_damage/powermultiplier",
            "{version}"
        );
    }
}

#[test]
fn the_old_flag_of_impact_damage_under_its_alias_is_converted_by_the_1_26_30_step() {
    // `semirandomdiffdamage` (the
    // `impact_damage` aliases) was the one alias of the step's two migrations left out: told
    // `UnrecognizedField`, the randomization `none`.
    for version in ["1.21.120", "1.26.30"] {
        let projectile = |members: &str| {
            told(
                version,
                "demo:x",
                &format!(
                    r#""components": {{"minecraft:projectile": {{"on_hit": {{"impact_damage": {{{members}}}}}}}}}"#
                ),
            )
        };
        for (flag, name) in [("true", "additive"), ("false", "none")] {
            let (found, written) = projectile(&format!(r#""semirandomdiffdamage": {flag}"#));
            assert!(found.is_empty(), "{version} {flag}: {found:#?}");
            assert!(
                written.contains(&format!(r#""difficulty_randomization": "{name}""#)),
                "{version} {flag}: {written}"
            );
            assert_eq!(
                (found, written),
                projectile(&format!(r#""semi_random_diff_damage": {flag}"#)),
                "{version} {flag}"
            );
        }
        // What the step tells of the member it tells at the alias the file has.
        let (found, _) = projectile(r#""semirandomdiffdamage": "yes""#);
        assert_eq!(
            found,
            [
                "InvalidType Info components/minecraft:projectile/on_hit/impact_damage/semirandomdiffdamage"
            ],
            "{version}"
        );
        let (found, written) = projectile(
            r#""semirandomdiffdamage": true, "difficulty_randomization": "multiplicative""#,
        );
        assert_eq!(
            found,
            [
                "DroppedField Info components/minecraft:projectile/on_hit/impact_damage/semirandomdiffdamage"
            ],
            "{version}"
        );
        assert!(
            written.contains(r#""difficulty_randomization": "multiplicative""#),
            "{written}"
        );
    }
    // From 1.26.40 no step reads the flag: an unknown member under either spelling, kept.
    for member in ["semirandomdiffdamage", "semi_random_diff_damage"] {
        let (found, written) = told(
            "1.26.40",
            "demo:x",
            &format!(
                r#""components": {{"minecraft:projectile": {{"on_hit": {{"impact_damage": {{"{member}": true}}}}}}}}"#
            ),
        );
        assert_eq!(
            found,
            [format!(
                "UnrecognizedField Info components/minecraft:projectile/on_hit/impact_damage/{member}"
            )]
        );
        // Told, not kept.
        assert!(
            !written.contains(&format!(r#""{member}": true"#)),
            "{written}"
        );
    }
}

#[test]
fn the_1_21_110_step_sets_the_knockback_flag_below_on_hit_and_not_below_its_alias() {
    // The game's fixer sets, on `on_hit.impact_damage`
    // (`apply_knockback_to_blocking_targets: true`, written unconditionally). The fixer does
    // not read the legacy reader's alias `onHit`, so below it the flag is the file's, or the
    // default. Nothing the file holds goes without a word.
    let flag = "apply_knockback_to_blocking_targets";
    let at = "components/minecraft:projectile";
    for version in ["1.16.0", "1.21.100"] {
        let projectile = |member: &str, damage: &str| {
            told(
                version,
                "demo:x",
                &format!(
                    r#""components": {{"minecraft:projectile": {{"{member}": {{"impact_damage": {{{damage}}}}}}}}}"#
                ),
            )
        };
        let wrote = |written: &str, value: bool| {
            assert!(
                written.contains(&format!(r#""{flag}": {value}"#)),
                "{version}: {written}"
            );
        };
        // Below `on_hit` the step writes `true`, over a `false` of the file, which is noted.
        let (found, written) = projectile("on_hit", r#""damage": 1"#);
        assert!(found.is_empty(), "{version}: {found:#?}");
        wrote(&written, true);
        let (found, written) = projectile("on_hit", &format!(r#""damage": 1, "{flag}": false"#));
        assert_eq!(
            found,
            [format!(
                "DroppedField Info {at}/on_hit/impact_damage/{flag}"
            )],
            "{version}"
        );
        wrote(&written, true);
        // Below `onHit` the step writes nothing: the default, or what the file says.
        let (found, written) = projectile("onHit", r#""damage": 1"#);
        assert!(found.is_empty(), "{version}: {found:#?}");
        wrote(&written, false);
        for value in [false, true] {
            let (found, written) =
                projectile("onHit", &format!(r#""damage": 1, "{flag}": {value}"#));
            assert!(found.is_empty(), "{version} {value}: {found:#?}");
            wrote(&written, value);
        }
    }
}

/// The four melee goals' legacy schema readers take `path_fail_time_increase` (a member added
/// after `max_path_time`), so a legacy file's member is read, not told as unknown.
#[test]
fn a_legacy_melee_goal_reads_its_path_fail_time_increase() {
    for key in [
        "minecraft:behavior.delayed_attack",
        "minecraft:behavior.melee_attack",
        "minecraft:behavior.melee_box_attack",
        "minecraft:behavior.stomp_attack",
    ] {
        let (found, written) = told(
            "1.20.0",
            "demo:x",
            &format!(
                r#""components": {{"{key}": {{"priority": 1, "path_fail_time_increase": 1.5}}}}"#
            ),
        );
        assert!(
            !found
                .iter()
                .any(|f| f.contains("Warning") || f.contains("Error")),
            "{key}: {found:#?}"
        );
        assert!(
            written.contains(r#""path_fail_time_increase": 1.5"#),
            "{key}: {written}"
        );
    }
}

/// A legacy goal's range written `{min, max}` is not read where its reader reads
/// `range_min` / `range_max` only — the schema-read goals' float range node (croak) and three of
/// the four range reads of the hand-written goal reader (`hover_height`, `float_duration`, `sound_interval`): the member
/// keeps its default, `min` / `max` dropped with an information each; `range_*` still reads.
#[test]
fn a_legacy_goal_range_written_min_max_keeps_the_member_default() {
    let at = "components/minecraft:";
    for (version, key, member) in [
        ("1.26.20", "behavior.croak", "interval"),
        ("1.21.0", "behavior.float_wander", "float_duration"),
    ] {
        let (found, written) = told(
            version,
            "demo:x",
            &format!(
                r#""components": {{"minecraft:{key}": {{"priority": 1, "{member}": {{"min": 1, "max": 3}}}}}}"#
            ),
        );
        for end in ["min", "max"] {
            has(
                &found,
                &format!("DroppedField Info {at}{key}/{member}/{end}"),
            );
        }
        assert!(
            !written.contains(&format!("\"{member}\"")),
            "{key}: {written}"
        );
        let (_, written) = told(
            version,
            "demo:x",
            &format!(
                r#""components": {{"minecraft:{key}": {{"priority": 1, "{member}": {{"range_min": 1, "range_max": 3}}}}}}"#
            ),
        );
        assert!(
            written.contains(&format!(r#""{member}": {{ "min": 1"#)),
            "{key}: {written}"
        );
    }
}

/// `look_time` of the hand-written look goals is read by
/// the game's integer range reader, which reads `min`
/// and `max` and never `range_*`: `{min, max}` is read; `{range_min, range_max}`
/// is not, the member keeping its default with an information at each.
#[test]
fn a_legacy_look_time_reads_min_max_and_not_the_range_spelling() {
    let at = "components/minecraft:behavior.look_at_player/look_time";
    let (found, written) = told(
        "1.21.0",
        "demo:x",
        r#""components": {"minecraft:behavior.look_at_player": {"priority": 1, "look_time": {"min": 1, "max": 3}}}"#,
    );
    assert!(written.contains(r#""look_time": { "min": 1"#), "{written}");
    assert!(
        !found.iter().any(|f| f.contains("DroppedField")),
        "{found:?}"
    );
    let (found, written) = told(
        "1.21.0",
        "demo:x",
        r#""components": {"minecraft:behavior.look_at_player": {"priority": 1, "look_time": {"range_min": 1, "range_max": 3}}}"#,
    );
    for end in ["range_min", "range_max"] {
        has(&found, &format!("DroppedField Info {at}/{end}"));
    }
    assert!(!written.contains(r#""look_time""#), "{written}");
}

/// The integer range reader reads a string `look_time` as the text between `[` and `]`, split
/// at `,` without empty parts, the first and second parts read as integers through a
/// string stream, swapped when reversed; without a `[`
/// the whole string is one integer for both ends; a `[` without a `]` fails, and
/// the goal then takes `look_time` (2, 4). Nothing is logged.
#[test]
fn a_legacy_look_time_string_is_read_as_parse_range_reads_it() {
    let at = "components/minecraft:behavior.look_at_player/look_time";
    let goal = |look_time: &str| {
        told(
            "1.21.0",
            "demo:x",
            &format!(
                r#""components": {{"minecraft:behavior.look_at_player": {{"priority": 1, "look_time": {look_time}}}}}"#
            ),
        )
    };
    for (written, expected) in [
        (r#""[1,3]""#, r#""look_time": { "min": 1, "max": 3 }"#),
        (r#""[ 4 , 2 ]""#, r#""look_time": { "min": 2, "max": 4 }"#),
        (r#""5""#, r#""look_time": { "min": 5, "max": 5 }"#),
        // No `[`: the stream reads `1` and stops at `-`; both ends 1 (not a "1-3" range).
        (r#""1-3""#, r#""look_time": { "min": 1, "max": 1 }"#),
        // The game's substring length is `]`'s absolute offset
        // minus one, over bytes: in `ab[1]é` it takes `1]` and the first byte of `é`, reads 1,
        // then fails at `]` (0) and swaps: (0, 1); in `[1,3]é` it takes `1,3`: (1, 3).
        (r#""ab[1]é""#, r#""look_time": { "min": 0, "max": 1 }"#),
        (r#""[1,3]é""#, r#""look_time": { "min": 1, "max": 3 }"#),
    ] {
        let (found, text) = goal(written);
        // (The typed `look_time` holds floats, written `1.0`.)
        let text = text.replace(".0", "");
        assert!(text.contains(expected), "{written}: {text}");
        assert!(
            !found
                .iter()
                .any(|f| f.contains("Warning") || f.contains("Error")),
            "{written}: {found:?}"
        );
    }
    // Malformed: a `[` without a `]`: the goal's (2, 4).
    let (found, text) = goal(r#""[1,3""#);
    let text = text.replace(".0", "");
    assert!(
        text.contains(r#""look_time": { "min": 2, "max": 4 }"#),
        "{text}"
    );
    has(&found, &format!("ValueRewritten Info {at}"));
}

/// Where in the file a member a step moved is, as the loaded document's findings answer it
/// after the loader gave each finding its file (phase 1 step 4 pin): the `look_time` range the
/// 1.21.130 step builds is at the `min_look_time` the file has; a member nothing moved is
/// where it is.
#[test]
fn the_document_answers_where_a_moved_member_is_in_the_file() {
    use bedrock_addon_core::diagnostic::JsonPath;
    let text = doc(
        "1.21.120",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.look_at_player": {"priority": 1, "min_look_time": 3}}}"#,
    );
    let document = super::common::parse(&text).expect("the entity loads");
    let goal = [
        "minecraft:entity",
        "components",
        "minecraft:behavior.look_at_player",
    ];
    let at = |member: &[&str]| JsonPath::new().with(&goal).with(member);
    let found = document.diagnostics();
    assert_eq!(
        found.place_of(&at(&["look_time", "min"])),
        at(&["min_look_time"])
    );
    assert_eq!(found.place_of(&at(&["look_time"])), at(&["min_look_time"]));
    assert_eq!(found.place_of(&at(&["priority"])), at(&["priority"]));
}

/// The span of a finding in a file with a byte-order mark is an offset into the file as
/// written, the mark's three bytes included (phase 1 step 6 pin: the loader blanks the mark).
#[test]
fn a_finding_in_a_file_with_a_byte_order_mark_is_at_the_file_s_offset() {
    let text = "\u{feff}{\"format_version\": \"1.26.50\", \"minecraft:entity\": @}";
    let rejected = super::common::parse(text).expect_err("not JSON");
    let spans: Vec<(String, u32)> = rejected
        .diagnostics
        .iter()
        .filter_map(|f| {
            f.location
                .span
                .map(|s| (f.code.name().to_owned(), s.offset))
        })
        .collect();
    let at = u32::try_from(text.find('@').unwrap()).unwrap();
    assert!(
        spans.iter().any(|(_, offset)| *offset == at),
        "{spans:?}, `@` at {at}"
    );
}
