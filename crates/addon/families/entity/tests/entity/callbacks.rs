//! The steps whose output depends on the document: each test runs one step of the chain on an
//! input of its own (the step's base with the given `components`) and compares the output's
//! `components` with what gives.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::testing::{Checks, FailureKind, Run, StepOutput, step_output};
use bedrock_addon_core::versioning::{Upgrade, VersionStruct};
use bedrock_addon_family_entity::history::{
    v1_12_0, v1_13_0, v1_15_0, v1_16_0, v1_18_10, v1_19_0, v1_20_0, v1_20_60, v1_20_70, v1_21_20,
    v1_21_70, v1_21_89, v1_21_90, v1_21_100, v1_21_120, v1_21_130, v1_26_0, v1_26_20, v1_26_30,
    v1_26_40,
};

use super::common::{json, latest_components};

/// One step of the chain and the document its cases start from.
struct Case {
    /// The step's base document.
    base: Value,
    /// Runs the step on a document.
    step: fn(&Value, Run, &Checks<'_>) -> Result<StepOutput, FailureKind>,
    /// The identifier the base names.
    identifier: String,
}

impl Case {
    /// The step of `S` on `base` (JSON text).
    fn of<S>(base: &str) -> Self
    where
        S: Upgrade + VersionStruct,
        S::Next: VersionStruct,
    {
        let base = json(base);
        let identifier = base
            .as_object()
            .and_then(|b| b.get("description"))
            .and_then(Value::as_object)
            .and_then(|d| d.get("identifier"))
            .and_then(Value::as_string)
            .map(|s| s.as_str().to_owned())
            .unwrap_or_default();
        Self {
            base,
            step: |input, run, checks| step_output::<S>(input, run, checks),
            identifier,
        }
    }

    /// Runs the step on `input` with the default run settings.
    fn run(&self, input: &Value) -> Result<StepOutput, FailureKind> {
        self.run_with(input, Run::default())
    }

    /// Runs the step on `input` with `settings`.
    fn run_with(&self, input: &Value, settings: Run) -> Result<StepOutput, FailureKind> {
        let checks = Checks {
            identifier: &self.identifier,
            actor_type: ActorType::for_entity(&self.identifier, None),
            ..Checks::default()
        };
        (self.step)(input, settings, &checks)
    }
}

/// The base with its `components` replaced by `components` (JSON text) and, when given, a
/// group `g` holding `group`.
fn input(base: &Value, components: &str, group: Option<&str>) -> Value {
    let mut input = base.clone();
    let object = input.as_object_mut().expect("an object base");
    object.insert("components", json(components));
    if let Some(group) = group {
        let mut groups = facet_value::VObject::new();
        groups.insert("g", json(group));
        object.insert("component_groups", groups);
    }
    input
}

/// Runs the step over `components` (and `group`) and requires `expected_components` (and
/// `expected_group`) in the output; with `section_a`, the skipped run must leave the input as
/// it was.
fn expect(
    probe: &Case,
    components: &str,
    group: Option<&str>,
    expected_components: &str,
    expected_group: Option<&str>,
    section_a: bool,
) -> Result<(), &'static str> {
    let given = input(&probe.base, components, group);
    let out = probe.run(&given).map_err(|_| "the step does not run")?;
    let payload = out.payload.as_object().ok_or("no payload")?;
    if payload.get("components") != Some(&json(expected_components)) {
        eprintln!(
            "components: {}",
            bedrock_addon_core::json::to_json(&out.payload).unwrap_or_default()
        );
        return Err("components differ");
    }
    if let Some(expected) = expected_group {
        let group = payload
            .get("component_groups")
            .and_then(Value::as_object)
            .and_then(|g| g.get("g"));
        if group != Some(&json(expected)) {
            eprintln!(
                "group: {}",
                bedrock_addon_core::json::to_json(&out.payload).unwrap_or_default()
            );
            return Err("the group differs");
        }
    }
    if section_a {
        let skipped = probe
            .run_with(
                &given,
                Run {
                    original_version: None,
                    skip_section_a: true,
                },
            )
            .map_err(|_| "the step does not run")?;
        let payload = skipped.payload.as_object().ok_or("no payload")?;
        if payload.get("components") != Some(&json(components)) {
            return Err("a section-A op ran with do_not_upgrade");
        }
    }
    Ok(())
}

fn a113_type_family_mob(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:type_family": {"family": ["a", "mob"]}, "minecraft:pushable": {"is_pushable": true, "is_pushable_by_piston": true}}"#,
        Some(r#"{"minecraft:type_family": {"family": "b"}}"#),
        r#"{"minecraft:type_family": {"family": ["a", "mob", "mob"]}, "minecraft:pushable": {"is_pushable": true, "is_pushable_by_piston": true}}"#,
        Some(r#"{"minecraft:type_family": {"family": ["b", "mob"]}}"#),
        true,
    )
}

fn b113_damage_sensor(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:damage_sensor": [{"cause": "fall", "filters": {"test": "a"}, "event": "x:e"}], "minecraft:pushable": {}}"#,
        Some(r#"{"minecraft:damage_sensor": {"cause": "fire"}}"#),
        r#"{"minecraft:damage_sensor": {"triggers": [{"on_damage": {"filters": {"test": "a"}, "event": "x:e"}, "cause": "fall"}]}, "minecraft:pushable": {"is_pushable": true, "is_pushable_by_piston": true}}"#,
        Some(r#"{"minecraft:damage_sensor": {"triggers": [{"cause": "fire"}]}}"#),
        false,
    )
}

fn b113_hurt_when_wet(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:hurt_when_wet": {"value": true}, "minecraft:pushable": {}}"#,
        None,
        r#"{"minecraft:pushable": {"is_pushable": true, "is_pushable_by_piston": true}, "minecraft:hurt_on_condition": {"damage_conditions": [{"filters": {"test": "in_water_or_rain", "operator": "==", "value": true}, "cause": "drowning", "damage_per_tick": 1}]}}"#,
        None,
        false,
    )
}

fn b113_fall_damage(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:fall_damage": {"value": 0.5}, "minecraft:pushable": {}}"#,
        Some(
            r#"{"minecraft:fall_damage": {"value": 0}, "minecraft:damage_sensor": {"cause": "fall", "deals_damage": true}}"#,
        ),
        r#"{"minecraft:pushable": {"is_pushable": true, "is_pushable_by_piston": true}, "minecraft:damage_sensor": {"triggers": [{"deals_damage": true, "damage_multiplier": 0.5, "cause": "fall"}]}}"#,
        Some(
            r#"{"minecraft:damage_sensor": {"triggers": [{"cause": "fall", "deals_damage": false}]}}"#,
        ),
        false,
    )
}

fn b113_environment_sensor(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:environment_sensor": {"on_environment": {"event": "x:a"}}, "minecraft:pushable": {}}"#,
        Some(
            r#"{"minecraft:environment_sensor": [{"on_environment": {"event": "x:b"}}, {"event": "x:c"}]}"#,
        ),
        r#"{"minecraft:environment_sensor": {"triggers": {"event": "x:a"}}, "minecraft:pushable": {"is_pushable": true, "is_pushable_by_piston": true}}"#,
        Some(
            r#"{"minecraft:environment_sensor": {"triggers": [{"event": "x:b"}, {"event": "x:c"}]}}"#,
        ),
        false,
    )
}

fn b113_giveable(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:giveable": {"items": ["x"]}, "minecraft:pushable": {}}"#,
        None,
        r#"{"minecraft:giveable": {"triggers": {"items": ["x"]}}, "minecraft:pushable": {"is_pushable": true, "is_pushable_by_piston": true}}"#,
        None,
        false,
    )
}

fn b113_interact(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:interact": {"event": "x:a", "target": "self", "use_item": true}, "minecraft:pushable": {}}"#,
        Some(
            r#"{"minecraft:interact": [{"on_interact": {"event": "x:keep"}, "filters": {"test": "x"}}]}"#,
        ),
        r#"{"minecraft:interact": {"interactions": {"use_item": true, "on_interact": {"event": "x:a", "target": "self"}}}, "minecraft:pushable": {"is_pushable": true, "is_pushable_by_piston": true}}"#,
        Some(r#"{"minecraft:interact": {"interactions": [{"on_interact": {"event": "x:keep"}}]}}"#),
        false,
    )
}

fn b113_scheduler(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:scheduler": {"scheduled_events": [{"scheduled_event": {"event": "x:a"}}, {"weight": 1}]}, "minecraft:pushable": {}}"#,
        None,
        r#"{"minecraft:scheduler": {"scheduled_events": [{"event": "x:a"}]}, "minecraft:pushable": {"is_pushable": true, "is_pushable_by_piston": true}}"#,
        None,
        false,
    )
}

fn b113_spawn_entity(probe: &Case) -> Result<(), &'static str> {
    const R: &str = r#"{"test": "rider_count", "subject": "self", "operator": "==", "value": 0}"#;
    expect(
        probe,
        r#"{"minecraft:spawn_entity": {"spawn_item": "egg"}, "minecraft:pushable": {}}"#,
        Some(
            r#"{"minecraft:spawn_entity": [{"filters": {"test": "a"}}, {"filters": [{"test": "b"}]}]}"#,
        ),
        &format!(
            r#"{{"minecraft:spawn_entity": {{"spawn_item": "egg", "filters": {R}}}, "minecraft:pushable": {{"is_pushable": true, "is_pushable_by_piston": true}}}}"#
        ),
        Some(&format!(
            r#"{{"minecraft:spawn_entity": [{{"filters": {{"all_of": [{{"test": "a"}}, {R}]}}}}, {{"filters": [{{"test": "b"}}, {R}]}}]}}"#
        )),
        false,
    )
}

fn a115_swim_wander(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:behavior.swim_wander": {"interval": 4}}"#,
        None,
        r#"{"minecraft:behavior.swim_wander": {"interval": 0.25}}"#,
        None,
        true,
    )
}

fn a115_circle_around_anchor(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:behavior.circle_around_anchor": {"radius_change_chance": 4, "height_change_chance": 0}}"#,
        Some(r#"{"minecraft:behavior.circle_around_anchor": {"radius_change_chance": 4}}"#),
        r#"{"minecraft:behavior.circle_around_anchor": {"radius_adjustment_chance": 0.25, "height_adjustment_chance": 1.0}}"#,
        Some(r#"{"minecraft:behavior.circle_around_anchor": {}}"#),
        true,
    )
}

fn b115_spawn_entity(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:spawn_entity": {"spawn_item": "egg"}}"#,
        Some(r#"{"minecraft:spawn_entity": [{"spawn_item": "egg"}]}"#),
        r#"{"minecraft:spawn_entity": {"entities": {"spawn_item": "egg"}}}"#,
        Some(r#"{"minecraft:spawn_entity": {"entities": [{"spawn_item": "egg"}]}}"#),
        false,
    )
}

fn a116_instant_despawn(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:despawn": {}}"#,
        Some(r#"{"minecraft:despawn": {"remove_child_entities": true}}"#),
        r#"{"minecraft:instant_despawn": {"remove_child_entities": false}}"#,
        Some(r#"{"minecraft:instant_despawn": {"remove_child_entities": true}}"#),
        true,
    )
}

fn a115_silverfish_attack_interval(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:behavior.nearest_attackable_target": {"entity_types": [{"attack_interval": 10}, {"attack_interval": 5}]}}"#,
        None,
        r#"{"minecraft:behavior.nearest_attackable_target": {"entity_types": [{}, {"attack_interval": 5}]}}"#,
        None,
        true,
    )
}

fn b116100_rewrites(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:dweller": {}, "minecraft:behavior.drop_item_for": {"max_dist": 6}, "minecraft:behavior.explore_outskirts": {"wait_time": 5}, "minecraft:economy_trade_table": {"table": "t"}}"#,
        Some(r#"{"minecraft:economy_trade_table": {"use_legacy_price_formula": false}}"#),
        r#"{"minecraft:dweller": {}, "minecraft:behavior.drop_item_for": {"target_range": [6, 6, 6]}, "minecraft:behavior.explore_outskirts": {"max_wait_time": 0.25}, "minecraft:economy_trade_table": {"table": "t", "use_legacy_price_formula": true}}"#,
        Some(r#"{"minecraft:economy_trade_table": {"use_legacy_price_formula": false}}"#),
        false,
    )
}

fn b11820_go_home(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:behavior.go_home": {"on_home": {"event": "a"}, "on_failed": [{"event": "b"}]}}"#,
        Some(r#"{"minecraft:behavior.go_home": {"on_failed": {"event": "c"}}}"#),
        r#"{"minecraft:behavior.go_home": {"on_home": [{"event": "a"}], "on_failed": [{"event": "b"}]}}"#,
        Some(r#"{"minecraft:behavior.go_home": {"on_failed": [{"event": "c"}]}}"#),
        false,
    )
}

fn b11950_daytime_filter(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:behavior.take_flower": {}}"#,
        Some(r#"{"minecraft:behavior.offer_flower": {"filters": {"all_of": [{"test": "x"}]}}}"#),
        r#"{"minecraft:behavior.take_flower": {"filters": {"all_of": [{"test": "is_daytime", "value": true}]}}}"#,
        Some(
            r#"{"minecraft:behavior.offer_flower": {"filters": {"all_of": [{"test": "x"}, {"test": "is_daytime", "value": true}]}}}"#,
        ),
        false,
    )
}

fn b12060_subsensors(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:entity_sensor": {"sensor_range": 2.5, "event": "x:near", "relative_range": false}}"#,
        None,
        r#"{"minecraft:entity_sensor": {"relative_range": false, "subsensors": [{"event": "x:near", "range": 2.5}]}}"#,
        None,
        false,
    )
}

fn b12070_subsensor_range(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:entity_sensor": {"subsensors": [{"range": 3}]}}"#,
        None,
        r#"{"minecraft:entity_sensor": {"subsensors": [{"range": [3, 3]}]}}"#,
        None,
        false,
    )
}

fn b12080_interact_slots(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:interact": {"interactions": [{"equip_item_slot": 2}]}}"#,
        Some(r#"{"minecraft:interact": {"interactions": {"drop_item_slot": 0}}}"#),
        r#"{"minecraft:interact": {"interactions": [{"equip_item_slot": "2"}]}}"#,
        Some(r#"{"minecraft:interact": {"interactions": {"drop_item_slot": "0"}}}"#),
        false,
    )
}

fn b12140_looked_at(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:lookat": {"set_target": true, "look_event": "x:e", "look_cooldown": 5}, "minecraft:damage_sensor": {"triggers": {"deals_damage": false}}}"#,
        Some(r#"{"minecraft:lookat": {"set_target": false, "allow_invulnerable": true}}"#),
        r#"{"minecraft:looked_at": {"set_target": "once_and_stop_scanning", "looked_at_event": "x:e", "looked_at_cooldown": 5}, "minecraft:damage_sensor": {"triggers": {"deals_damage": "no"}}, "minecraft:dimension_bound": {}, "minecraft:transient": {}}"#,
        Some(r#"{"minecraft:looked_at": {"set_target": "never"}}"#),
        false,
    )
}

fn b12180_leashable_preset(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:leashable": {"soft_distance": 4, "hard_distance": 6, "max_distance": 10, "presets": [{"filter": {}}]}}"#,
        None,
        r#"{"minecraft:leashable": {"on_unleash_interact_only": true, "presets": [{"filter": {}}, {"soft_distance": 4, "hard_distance": 6, "max_distance": 10, "spring_type": "bouncy"}]}}"#,
        None,
        false,
    )
}

fn b12190_preset_defaults(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:leashable": {"can_be_cut": true, "presets": [{}]}}"#,
        None,
        r#"{"minecraft:leashable": {"can_be_cut": true, "can_be_stolen": false, "presets": [{"hard_distance": 6.0, "max_distance": 10.0, "spring_type": "bouncy"}]}}"#,
        None,
        false,
    )
}

fn b121100_enderman_goals(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:behavior.enderman_leave_block": {"priority": 1, "chance": 3}}"#,
        Some(r#"{"minecraft:behavior.enderman_take_block": {"priority": 2}}"#),
        r#"{"minecraft:behavior.place_block": {"priority": 1, "chance": 0.0005, "xz_range": [-1, 1], "y_range": [0, 2]}}"#,
        None,
        false,
    )?;
    let out = probe
        .run(&input(
            &probe.base,
            r#"{"minecraft:behavior.enderman_take_block": {}}"#,
            None,
        ))
        .map_err(|_| "the step does not run")?;
    let goal = out
        .payload
        .as_object()
        .and_then(|p| p.get("components"))
        .and_then(Value::as_object)
        .and_then(|c| c.get("minecraft:behavior.take_block"))
        .and_then(Value::as_object)
        .ok_or("no take_block")?;
    let blocks = goal
        .get("blocks")
        .and_then(Value::as_array)
        .ok_or("no blocks")?;
    if blocks.len() != 26 || goal.get("chance") != Some(&json("0.05")) {
        return Err("take_block differs");
    }
    Ok(())
}

fn b121110_dash_vertical(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:input_air_controlled": {}, "minecraft:dash": {}}"#,
        None,
        r#"{"minecraft:free_camera_controlled": {}, "minecraft:vertical_movement_action": {}, "minecraft:dash_action": {"direction": "entity"}}"#,
        None,
        false,
    )
}

fn b121130_result_items_interval(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:ageable": {"feed_items": "wheat", "transform_to_item": "x"}, "minecraft:breedable": {"breed_items": "wheat", "transform_to_item": "y"}, "minecraft:behavior.nearest_attackable_target": {"attack_interval": 4, "attack_interval_min": 1}}"#,
        None,
        r#"{"minecraft:ageable": {"feed_items": "wheat"}, "minecraft:breedable": {"breed_items": [{"item": "wheat", "result_item": "y"}]}, "minecraft:behavior.nearest_attackable_target": {"attack_interval": {"range_min": 1, "range_max": 4}}}"#,
        None,
        false,
    )
}

fn b1260_interact_look_breedable(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:interact": {"interactions": {}}, "minecraft:behavior.look_at_entity": {"max_look_time": 9, "target_distance": 1}, "minecraft:breedable": {"mutation_factor": {}, "breeds_with": {"mate_type": "a", "baby_type": "b"}}}"#,
        None,
        r#"{"minecraft:interact": {"interactions": {"swing": false}}, "minecraft:behavior.look_at_entity": {"look_time": {"min": 2, "max": 9}}, "minecraft:breedable": {"breeds_with": {"a": {}}}, "minecraft:offspring": {"mutation_factor": {}, "offspring_pairs": {"a": "b"}}}"#,
        None,
        false,
    )
}

fn c1260_experience_fold(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:experience_reward": {"on_death_array": ["5"], "on_bred": "", "on_bred_array": ["1", "2"]}}"#,
        None,
        r#"{"minecraft:experience_reward": {"on_death": "5", "on_bred": "", "on_bred_array": ["1", "2"]}}"#,
        None,
        false,
    )
}

fn b12610_pushable_split(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:pushable": {"is_pushable": true, "is_pushable_by_piston": false}}"#,
        Some(r#"{"minecraft:pushable": {}}"#),
        r#"{"minecraft:pushable_by_entity": {}}"#,
        Some(r#"{"minecraft:pushable_by_entity": {}, "minecraft:pushable_by_block": {}}"#),
        false,
    )
}

fn c12630_knockback_presets(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:apply_knockback_rules": {"presets": [{"horizontal_power": 1}, {"knockback_mode": "hit_direction"}]}}"#,
        None,
        r#"{"minecraft:apply_knockback_rules": {"presets": [
            {"horizontal_power": 1, "extra_knockback_approach": "reapply_default", "knockback_mode": "relative_horizontal", "scale_with_damage": false, "slowdown_scale": 0.5},
            {"knockback_mode": "hit_direction", "extra_knockback_approach": "reapply_default", "scale_with_damage": false, "slowdown_scale": 0.5}]}}"#,
        None,
        false,
    )
}

fn b12640_projectile(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:projectile": {"should_bounce": true, "on_hit": {"impact_damage": {"semi_random_diff_damage": true}}}}"#,
        Some(r#"{"minecraft:projectile": {"should_bounce": false}}"#),
        r#"{"minecraft:projectile": {"should_bounce": "if_invulnerable", "on_hit": {"impact_damage": {"difficulty_randomization": "additive", "power_multiplier": 2.0}}}}"#,
        Some(r#"{"minecraft:projectile": {"should_bounce": "no"}}"#),
        false,
    )
}

fn c12640_shapes(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:behavior.ranged_attack": {"attack_interval_min": 1, "attack_interval_max": 3}, "minecraft:behavior.beg": {"look_time": [2, 4]}, "minecraft:behavior.float_wander": {"float_duration": [0.1, 0.35]}, "minecraft:behavior.stomp_attack": {"on_kill": {"event": "k"}}, "minecraft:pushable_by_entity": {"presets": [{}]}}"#,
        None,
        r#"{"minecraft:behavior.ranged_attack": {"attack_interval": {"min": 1, "max": 3}}, "minecraft:behavior.beg": {"look_time": {"min": 2, "max": 4}}, "minecraft:behavior.float_wander": {"float_duration": {"min": 0.1, "max": 0.35}}, "minecraft:behavior.stomp_attack": {"on_attack": {"event": "k"}}, "minecraft:pushable_by_entity": {"presets": [{"require_collision_overlap": false}]}}"#,
        None,
        false,
    )
}

/// The last step converts to the typed structs, so its output is compared with the expected
/// members as the newest struct writes them (typed defaults included).
fn c12650_attack_range_anchor(probe: &Case) -> Result<(), &'static str> {
    let given = input(
        &probe.base,
        r#"{"minecraft:behavior.ranged_attack": {"attack_radius": 15, "attack_radius_min": 4}, "minecraft:projectile": {"anchor": 2}}"#,
        None,
    );
    let out = probe.run(&given).map_err(|_| "the step does not run")?;
    let payload = out.payload.as_object().ok_or("no payload")?;
    let expected = latest_components(
        r#"{"minecraft:behavior.ranged_attack": {"attack_range": {"min": 4, "max": 15}, "in_range_movement_mode": "hold_position"}, "minecraft:projectile": {"anchor": "middle", "multiple_targets": true}, "minecraft:uses_legacy_ambient_sounds": {}}"#,
    );
    if payload.get("components") != Some(&expected) {
        eprintln!(
            "components: {}",
            bedrock_addon_core::json::to_json(&out.payload).unwrap_or_default()
        );
        return Err("components differ");
    }
    Ok(())
}

fn b12610_buoyant(probe: &Case) -> Result<(), &'static str> {
    expect(
        probe,
        r#"{"minecraft:buoyant": {"simulate_waves": true}}"#,
        Some(r#"{"minecraft:buoyant": {"simulate_waves": false, "base_buoyancy": 1}}"#),
        r#"{"minecraft:buoyant": {"movement_type": "waves"}}"#,
        Some(r#"{"minecraft:buoyant": {"movement_type": "none", "base_buoyancy": 1}}"#),
        false,
    )
}

/// Part 2: the identifier- and family-bound ops of the
/// 1.26.50 step's `collectUpgraders`, each on an input without the component.
fn c12650_after_ops(probe: &Case) -> Result<(), &'static str> {
    let cases: [(&str, &str, Option<&str>, &str, &str); 5] = [
        (
            "minecraft:spider",
            "{}",
            None,
            "components",
            "minecraft:block_movement_slowdown_immunity",
        ),
        (
            "minecraft:stray",
            "{}",
            None,
            "components",
            "minecraft:freezing_immune",
        ),
        (
            "minecraft:blaze",
            "{}",
            None,
            "components",
            "minecraft:freezing_vulnerable",
        ),
        (
            "minecraft:sulfur_cube",
            "{}",
            Some(r#"{"minecraft:sulfur_cube_medium_with_block": {}}"#),
            "minecraft:sulfur_cube_medium_with_block",
            "minecraft:freezing_immune",
        ),
        (
            "demo:x",
            r#"{"minecraft:type_family": {"family": ["lightweight"]}}"#,
            None,
            "components",
            "minecraft:can_stand_on_powder_snow",
        ),
    ];
    for (identifier, components, groups, section, key) in cases {
        let mut given = probe.base.clone();
        let object = given.as_object_mut().ok_or("no base")?;
        let mut description = facet_value::VObject::new();
        description.insert("identifier", Value::from(identifier));
        object.insert("description", description);
        object.insert("components", json(components));
        if let Some(groups) = groups {
            object.insert("component_groups", json(groups));
        }
        let out = probe.run(&given).map_err(|_| "the step does not run")?;
        let payload = out.payload.as_object().ok_or("no payload")?;
        let holder = if section == "components" {
            payload.get("components")
        } else {
            payload
                .get("component_groups")
                .and_then(Value::as_object)
                .and_then(|groups| groups.get(section))
        };
        if holder
            .and_then(Value::as_object)
            .and_then(|holder| holder.get(key))
            .is_none()
        {
            eprintln!(
                "{identifier}: {}",
                bedrock_addon_core::json::to_json(&out.payload).unwrap_or_default()
            );
            return Err("a component the step inserts is missing");
        }
    }
    Ok(())
}

/// Runs the check of a step on its base.
#[track_caller]
fn check(case: Case, check: fn(&Case) -> Result<(), &'static str>) {
    check(&case).unwrap_or_else(|reason| panic!("{reason}"));
}

#[test]
fn a113_type_family_mob_step() {
    check(
        Case::of::<v1_12_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        a113_type_family_mob,
    );
}

#[test]
fn b113_damage_sensor_step() {
    check(
        Case::of::<v1_12_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b113_damage_sensor,
    );
}

#[test]
fn b113_hurt_when_wet_step() {
    check(
        Case::of::<v1_12_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b113_hurt_when_wet,
    );
}

#[test]
fn b113_fall_damage_step() {
    check(
        Case::of::<v1_12_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b113_fall_damage,
    );
}

#[test]
fn b113_environment_sensor_step() {
    check(
        Case::of::<v1_12_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b113_environment_sensor,
    );
}

#[test]
fn b113_giveable_step() {
    check(
        Case::of::<v1_12_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b113_giveable,
    );
}

#[test]
fn b113_interact_step() {
    check(
        Case::of::<v1_12_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b113_interact,
    );
}

#[test]
fn b113_scheduler_step() {
    check(
        Case::of::<v1_12_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b113_scheduler,
    );
}

#[test]
fn b113_spawn_entity_step() {
    check(
        Case::of::<v1_12_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b113_spawn_entity,
    );
}

#[test]
fn a115_swim_wander_step() {
    check(
        Case::of::<v1_13_0::Entity>(
            r#"{"description": {"identifier": "minecraft:cod"}, "components": {}}"#,
        ),
        a115_swim_wander,
    );
}

#[test]
fn a115_circle_around_anchor_step() {
    check(
        Case::of::<v1_13_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.lay_egg": {}}, "component_groups": {"g": {"minecraft:behavior.stomp_turtle_egg": {}}}}"#,
        ),
        a115_circle_around_anchor,
    );
}

#[test]
fn b115_spawn_entity_step() {
    check(
        Case::of::<v1_13_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.lay_egg": {}}, "component_groups": {"g": {"minecraft:behavior.stomp_turtle_egg": {}}}}"#,
        ),
        b115_spawn_entity,
    );
}

#[test]
fn a116_instant_despawn_step() {
    check(
        Case::of::<v1_15_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:breathable": {}, "minecraft:behavior.panic": {}}}"#,
        ),
        a116_instant_despawn,
    );
}

#[test]
fn a115_silverfish_attack_interval_step() {
    check(
        Case::of::<v1_13_0::Entity>(
            r#"{"description": {"identifier": "minecraft:silverfish"}, "components": {"minecraft:behavior.nearest_attackable_target": {"entity_types": [{}]}}}"#,
        ),
        a115_silverfish_attack_interval,
    );
}

#[test]
fn b116100_rewrites_step() {
    check(
        Case::of::<v1_16_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:dweller": {}}}"#,
        ),
        b116100_rewrites,
    );
}

#[test]
fn b11820_go_home_step() {
    check(
        Case::of::<v1_18_10::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b11820_go_home,
    );
}

#[test]
fn b11950_daytime_filter_step() {
    check(
        Case::of::<v1_19_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:input_ground_controlled": {}}}"#,
        ),
        b11950_daytime_filter,
    );
}

#[test]
fn b12060_subsensors_step() {
    check(
        Case::of::<v1_20_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b12060_subsensors,
    );
}

#[test]
fn b12070_subsensor_range_step() {
    check(
        Case::of::<v1_20_60::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b12070_subsensor_range,
    );
}

#[test]
fn b12080_interact_slots_step() {
    check(
        Case::of::<v1_20_70::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b12080_interact_slots,
    );
}

#[test]
fn b12140_looked_at_step() {
    check(
        Case::of::<v1_21_20::Entity>(
            r#"{"description": {"identifier": "minecraft:fishing_hook"}, "components": {"minecraft:home": {"restriction_radius": 16}}}"#,
        ),
        b12140_looked_at,
    );
}

#[test]
fn b12180_leashable_preset_step() {
    check(
        Case::of::<v1_21_70::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:leashable": {}}}"#,
        ),
        b12180_leashable_preset,
    );
}

#[test]
fn b12190_preset_defaults_step() {
    check(
        Case::of::<v1_21_89::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:leashable": {}}}"#,
        ),
        b12190_preset_defaults,
    );
}

#[test]
fn b121100_enderman_goals_step() {
    check(
        Case::of::<v1_21_90::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b121100_enderman_goals,
    );
}

#[test]
fn b121110_dash_vertical_step() {
    check(
        Case::of::<v1_21_100::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b121110_dash_vertical,
    );
}

#[test]
fn b121130_result_items_interval_step() {
    check(
        Case::of::<v1_21_120::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {}}"#,
        ),
        b121130_result_items_interval,
    );
}

#[test]
fn b1260_interact_look_breedable_step() {
    check(
        Case::of::<v1_21_130::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:jump.dynamic": {}, "minecraft:behavior.aquatic_charge_attack": {}}}"#,
        ),
        b1260_interact_look_breedable,
    );
}

#[test]
fn c1260_experience_fold_step() {
    check(
        Case::of::<v1_21_130::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:jump.dynamic": {}, "minecraft:behavior.aquatic_charge_attack": {}}}"#,
        ),
        c1260_experience_fold,
    );
}

#[test]
fn b12610_pushable_split_step() {
    check(
        Case::of::<v1_26_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:buoyant": {}}}"#,
        ),
        b12610_pushable_split,
    );
}

#[test]
fn c12630_knockback_presets_step() {
    check(
        Case::of::<v1_26_20::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.pickup_items": {}, "minecraft:pushable_by_entity": {}, "minecraft:leashable": {}, "minecraft:leashable_to": {}, "minecraft:apply_knockback_rules": {"presets": [{}]}}}"#,
        ),
        c12630_knockback_presets,
    );
}

#[test]
fn b12640_projectile_step() {
    check(
        Case::of::<v1_26_30::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.melee_attack": {}, "minecraft:pushable_by_entity": {}, "minecraft:projectile": {"on_hit": {"impact_damage": {}}}}}"#,
        ),
        b12640_projectile,
    );
}

#[test]
fn c12640_shapes_step() {
    check(
        Case::of::<v1_26_30::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.melee_attack": {}, "minecraft:pushable_by_entity": {}, "minecraft:projectile": {"on_hit": {"impact_damage": {}}}}}"#,
        ),
        c12640_shapes,
    );
}

#[test]
fn c12650_attack_range_anchor_step() {
    check(
        Case::of::<v1_26_40::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.ranged_attack": {}, "minecraft:navigation.walk": {}, "minecraft:projectile": {}, "minecraft:behavior.follow_mob": {}}}"#,
        ),
        c12650_attack_range_anchor,
    );
}

#[test]
fn b12610_buoyant_step() {
    check(
        Case::of::<v1_26_0::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:buoyant": {}}}"#,
        ),
        b12610_buoyant,
    );
}

#[test]
fn c12650_after_ops_step() {
    check(
        Case::of::<v1_26_40::Entity>(
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.ranged_attack": {}, "minecraft:navigation.walk": {}, "minecraft:projectile": {}, "minecraft:behavior.follow_mob": {}}}"#,
        ),
        c12650_after_ops,
    );
}
