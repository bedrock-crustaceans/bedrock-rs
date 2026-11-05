//! `jump.static`, `jump.dynamic`, `preferred_path`, `flocking`, `game_event_movement_tracking`,
//! `grows_crop`, `physics`, `collision_box`, `block_movement_slowdown_immunity`.

use bedrock_addon_core::shared::BlockDescriptor;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::{
    BlockMovementSlowdownImmunity, CollisionBox, Flocking, GameEventMovementTracking, GrowsCrop,
    JumpDynamic, JumpStatic, Physics, PreferredPath, SkipData,
};

use super::{json, lenient, read_as, round_trip, strict, write};

#[test]
fn jump_static_and_dynamic_read_their_defaults() {
    let (jump, _) = strict::<JumpStatic>("{}");
    assert_eq!(jump.jump_power, 0.42);
    let (dynamic, _) = strict::<JumpDynamic>("{}");
    assert_eq!(dynamic, JumpDynamic::default());
    assert_eq!(dynamic.regular_skip_data, SkipData::default());
    assert_eq!(dynamic.fast_skip_data.jump_delay, 30);
    assert_eq!(dynamic.fast_skip_data.height, 0.1);
}

#[test]
fn e29b_jump_dynamic_skip_data_of_the_1_26_0_upgrade_round_trip() {
    // e29b;: the values written unconditionally.
    let value = round_trip::<JumpDynamic>(
        r#"{"regular_skip_data": {"distance_scale": 0.8, "height": 0.1, "jump_delay": 20, "animation_duration": 3},
            "fast_skip_data": {"distance_scale": 1.75, "height": 0.15, "jump_delay": 1, "animation_duration": 1}}"#,
    );
    assert_eq!(value.regular_skip_data, SkipData::regular_1_26_0());
    assert_eq!(value.fast_skip_data, SkipData::fast_1_26_0());
    assert_eq!(JumpDynamic::upgraded_1_26_0(), value);
}

#[test]
fn preferred_path_reads_block_sets() {
    let value = round_trip::<PreferredPath>(
        r#"{"max_fall_blocks": 1, "jump_cost": 5, "default_block_cost": 1.5,
            "preferred_path_blocks": [{"cost": 0, "blocks": ["minecraft:grass_path", {"name": "minecraft:stone_slab", "states": {"minecraft:vertical_half": "bottom"}}]},
                                      {"cost": 50, "blocks": [{"tags": "query.any_tag('fence')"}]}]}"#,
    );
    assert_eq!(value.preferred_path_blocks.len(), 2);
    assert_eq!(value.preferred_path_blocks[0].blocks.len(), 2);
    assert_eq!(value.preferred_path_blocks[1].cost, Some(50.0));
    assert!(matches!(
        value.preferred_path_blocks[1].blocks[0],
        BlockDescriptor::Tags { .. }
    ));
    assert_eq!(strict::<PreferredPath>("{}").0.max_fall_blocks, 3);
}

#[test]
fn e00d_salmon_flocking_round_trips() {
    // e00d "salmon `flocking`" (the 1.13.0 A-op's members); the vanilla spelling
    // `innner_cohesion_threshold`.
    let value = round_trip::<Flocking>(
        r#"{"in_water": true, "match_variants": false, "use_center_of_mass": true, "low_flock_limit": 4,
            "high_flock_limit": 8, "goal_weight": 2.0, "loner_chance": 0.1, "influence_radius": 3.0,
            "breach_influence": 7.0, "separation_weight": 1.75, "separation_threshold": 0.95,
            "cohesion_weight": 1.85, "cohesion_threshold": 1.95, "innner_cohesion_threshold": 1.25,
            "min_height": 1.5, "max_height": 6.0, "block_distance": 1.0, "block_weight": 0.75}"#,
    );
    assert!(value.in_water);
    assert_eq!(value.innner_cohesion_threshold, 1.25);
    let (default, _) = strict::<Flocking>("{}");
    assert_eq!(default, Flocking::default());
    assert_eq!(default.separation_threshold, 2.0);
}

#[test]
fn small_movement_components_read_their_defaults() {
    let (tracking, _) = strict::<GameEventMovementTracking>("{}");
    assert!(tracking.emit_move && tracking.emit_swim && !tracking.emit_flap);
    let (crop, _) = strict::<GrowsCrop>("{}");
    assert_eq!((crop.charges, crop.chance), (10, 0.0));
    let (physics, _) = strict::<Physics>("{}");
    assert!(physics.has_collision && physics.has_gravity && !physics.push_towards_closest_space);
    let (collision, _) = strict::<CollisionBox>("{}");
    assert_eq!((collision.width, collision.height), (0.6, 1.8));
}

#[test]
fn physics_of_the_area_effect_cloud_round_trips() {
    let value = round_trip::<Physics>(r#"{"has_gravity": true, "has_collision": false}"#);
    assert!(!value.has_collision);
    assert_eq!(
        write(&value),
        json(
            r#"{"has_collision": false, "has_gravity": true, "push_towards_closest_space": false}"#
        )
    );
}

#[test]
fn collision_box_outside_its_schema_range_is_an_error_and_a_lenient_default() {
    let (value, d) = read_as::<CollisionBox>(r#"{"width": 2e8}"#, ConvertMode::Strict);
    assert!(value.is_none() && d.has_errors());
    let (value, _) = lenient::<CollisionBox>(r#"{"width": 2e8, "height": 3}"#);
    assert_eq!((value.width, value.height), (0.6, 3.0));
}

#[test]
fn block_movement_slowdown_immunity_reads_blocks() {
    let value = round_trip::<BlockMovementSlowdownImmunity>(
        r#"{"blocks": ["minecraft:powder_snow", {"tags": "query.any_tag('cobweb')"}]}"#,
    );
    assert_eq!(value.blocks.len(), 2);
}
