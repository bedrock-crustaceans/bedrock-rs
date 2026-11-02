//! The format version at which vanilla first types each component: a document at or above it
//! reads the component strictly, an older one leniently (vanilla's legacy upgrade callbacks).
//! Keys that stay dynamic in 1.26.60 have no row.

use bedrock_addon_core::version::FormatVersion;

/// `(key, first typed)`, sorted by key.
const FIRST_TYPED: &[(&str, FormatVersion)] = &[
    ("minecraft:addrider", FormatVersion::new(1, 21, 130)),
    ("minecraft:admire_item", FormatVersion::new(1, 26, 40)),
    ("minecraft:anger_level", FormatVersion::new(1, 26, 50)),
    (
        "minecraft:annotation.open_door",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:apply_knockback_rules",
        FormatVersion::new(1, 8, 0),
    ),
    ("minecraft:barter", FormatVersion::new(1, 26, 50)),
    (
        "minecraft:behavior.admire_item",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.aquatic_charge_attack",
        FormatVersion::new(1, 21, 120),
    ),
    (
        "minecraft:behavior.avoid_block",
        FormatVersion::new(1, 21, 120),
    ),
    (
        "minecraft:behavior.avoid_mob_type",
        FormatVersion::new(1, 21, 120),
    ),
    ("minecraft:behavior.barter", FormatVersion::new(1, 26, 20)),
    ("minecraft:behavior.beg", FormatVersion::new(1, 26, 40)),
    (
        "minecraft:behavior.break_door",
        FormatVersion::new(1, 26, 30),
    ),
    ("minecraft:behavior.breed", FormatVersion::new(1, 21, 90)),
    (
        "minecraft:behavior.celebrate",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.celebrate_survive",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.charge_attack",
        FormatVersion::new(1, 21, 100),
    ),
    (
        "minecraft:behavior.charge_held_item",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.circle_around_anchor",
        FormatVersion::new(1, 21, 100),
    ),
    (
        "minecraft:behavior.controlled_by_player",
        FormatVersion::new(1, 21, 100),
    ),
    ("minecraft:behavior.croak", FormatVersion::new(1, 26, 30)),
    (
        "minecraft:behavior.defend_trusted_target",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.defend_village_target",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.delayed_attack",
        FormatVersion::new(1, 26, 0),
    ),
    ("minecraft:behavior.dig", FormatVersion::new(1, 21, 120)),
    (
        "minecraft:behavior.door_interact",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.dragonchargeplayer",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.dragondeath",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.dragonflaming",
        FormatVersion::new(1, 21, 100),
    ),
    (
        "minecraft:behavior.dragonholdingpattern",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.dragonlanding",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.dragonscanning",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.dragonstrafeplayer",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.dragontakeoff",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.drink_milk",
        FormatVersion::new(1, 21, 120),
    ),
    (
        "minecraft:behavior.drink_potion",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.drop_item_for",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.eat_block",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.eat_carried_item",
        FormatVersion::new(1, 26, 20),
    ),
    ("minecraft:behavior.eat_mob", FormatVersion::new(1, 21, 100)),
    ("minecraft:behavior.emerge", FormatVersion::new(1, 26, 30)),
    (
        "minecraft:behavior.equip_item",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.explore_outskirts",
        FormatVersion::new(1, 21, 130),
    ),
    (
        "minecraft:behavior.fertilize_farm_block",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.find_cover",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.find_mount",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.find_underwater_treasure",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.fire_at_target",
        FormatVersion::new(1, 26, 30),
    ),
    ("minecraft:behavior.flee_sun", FormatVersion::new(1, 26, 20)),
    ("minecraft:behavior.float", FormatVersion::new(1, 26, 20)),
    (
        "minecraft:behavior.float_wander",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.follow_caravan",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.follow_mob",
        FormatVersion::new(1, 26, 50),
    ),
    (
        "minecraft:behavior.follow_owner",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.follow_parent",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.follow_target_leader",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.go_and_give_items_to_noteblock",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.go_and_give_items_to_owner",
        FormatVersion::new(1, 26, 20),
    ),
    ("minecraft:behavior.go_home", FormatVersion::new(1, 26, 20)),
    (
        "minecraft:behavior.guardian_attack",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.harvest_farm_block",
        FormatVersion::new(1, 26, 10),
    ),
    ("minecraft:behavior.hide", FormatVersion::new(1, 26, 20)),
    (
        "minecraft:behavior.hold_ground",
        FormatVersion::new(1, 26, 20),
    ),
    ("minecraft:behavior.hover", FormatVersion::new(1, 26, 20)),
    (
        "minecraft:behavior.hurt_by_target",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.inspect_bookshelf",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.investigate_suspicious_location",
        FormatVersion::new(1, 21, 130),
    ),
    (
        "minecraft:behavior.jump_around_target",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.jump_to_block",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.knockback_roar",
        FormatVersion::new(1, 26, 40),
    ),
    ("minecraft:behavior.lay_down", FormatVersion::new(1, 26, 20)),
    ("minecraft:behavior.lay_egg", FormatVersion::new(1, 26, 10)),
    (
        "minecraft:behavior.leap_at_target",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.look_at_entity",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.look_at_player",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.look_at_target",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.look_at_trading_player",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.make_love",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.melee_attack",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.melee_box_attack",
        FormatVersion::new(1, 26, 0),
    ),
    ("minecraft:behavior.mingle", FormatVersion::new(1, 26, 20)),
    (
        "minecraft:behavior.mount_pathing",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.move_around_target",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.move_indoors",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.move_outdoors",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.move_through_village",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.move_to_block",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.move_to_land",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.move_to_lava",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.move_to_liquid",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.move_to_poi",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.move_to_random_block",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.move_to_village",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.move_to_water",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.move_towards_dwelling_restriction",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.move_towards_home_restriction",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.move_towards_target",
        FormatVersion::new(1, 26, 20),
    ),
    ("minecraft:behavior.nap", FormatVersion::new(1, 26, 40)),
    (
        "minecraft:behavior.nearest_attackable_target",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.nearest_prioritized_attackable_target",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.ocelot_sit_on_block",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.ocelotattack",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.offer_flower",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.open_door",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.owner_hurt_by_target",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.owner_hurt_target",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.pet_sleep_with_owner",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.pickup_items",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.place_block",
        FormatVersion::new(1, 26, 30),
    ),
    ("minecraft:behavior.play", FormatVersion::new(1, 26, 10)),
    (
        "minecraft:behavior.play_dead",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.player_ride_tamed",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.raid_garden",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.ram_attack",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.random_breach",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.random_fly",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.random_hover",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.random_look_around",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.random_look_around_and_sit",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.random_search_and_dig",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.random_sitting",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.random_stroll",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.random_swim",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.ranged_attack",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.receive_love",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.restrict_open_door",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.restrict_sun",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.rise_to_liquid_level",
        FormatVersion::new(1, 26, 30),
    ),
    ("minecraft:behavior.roar", FormatVersion::new(1, 21, 110)),
    ("minecraft:behavior.roll", FormatVersion::new(1, 26, 30)),
    (
        "minecraft:behavior.run_around_like_crazy",
        FormatVersion::new(1, 26, 30),
    ),
    ("minecraft:behavior.scared", FormatVersion::new(1, 26, 30)),
    (
        "minecraft:behavior.share_items",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.silverfish_merge_with_stone",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.silverfish_wake_up_friends",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.skeleton_horse_trap",
        FormatVersion::new(1, 26, 30),
    ),
    ("minecraft:behavior.sleep", FormatVersion::new(1, 26, 20)),
    (
        "minecraft:behavior.slime_attack",
        FormatVersion::new(1, 21, 110),
    ),
    (
        "minecraft:behavior.slime_float",
        FormatVersion::new(1, 21, 110),
    ),
    (
        "minecraft:behavior.slime_keep_on_jumping",
        FormatVersion::new(1, 21, 110),
    ),
    (
        "minecraft:behavior.slime_random_direction",
        FormatVersion::new(1, 21, 110),
    ),
    ("minecraft:behavior.snacking", FormatVersion::new(1, 26, 40)),
    ("minecraft:behavior.sneeze", FormatVersion::new(1, 26, 30)),
    ("minecraft:behavior.sniff", FormatVersion::new(1, 21, 110)),
    (
        "minecraft:behavior.sonic_boom",
        FormatVersion::new(1, 21, 110),
    ),
    (
        "minecraft:behavior.squid_dive",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.squid_flee",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.squid_idle",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.squid_move_away_from_ground",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.squid_out_of_water",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.stalk_and_pounce_on_target",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:behavior.stay_near_noteblock",
        FormatVersion::new(1, 21, 110),
    ),
    (
        "minecraft:behavior.stay_while_sitting",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.stomp_attack",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.stomp_turtle_egg",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.stroll_towards_village",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:behavior.summon_entity",
        FormatVersion::new(1, 26, 40),
    ),
    ("minecraft:behavior.swell", FormatVersion::new(1, 26, 30)),
    (
        "minecraft:behavior.swim_idle",
        FormatVersion::new(1, 21, 110),
    ),
    (
        "minecraft:behavior.swim_up_for_breath",
        FormatVersion::new(1, 21, 110),
    ),
    (
        "minecraft:behavior.swim_wander",
        FormatVersion::new(1, 21, 110),
    ),
    (
        "minecraft:behavior.swim_with_entity",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.swoop_attack",
        FormatVersion::new(1, 21, 110),
    ),
    (
        "minecraft:behavior.take_block",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.take_flower",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.target_when_pushed",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.teleport_to_owner",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.timer_flag_1",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.timer_flag_2",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.timer_flag_3",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:behavior.trade_interest",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.trade_with_player",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:behavior.transport_items",
        FormatVersion::new(1, 21, 100),
    ),
    (
        "minecraft:behavior.use_kinetic_weapon",
        FormatVersion::new(1, 21, 120),
    ),
    (
        "minecraft:behavior.wither_random_attack_pos_goal",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:behavior.wither_target_highest_damage",
        FormatVersion::new(1, 26, 30),
    ),
    ("minecraft:behavior.work", FormatVersion::new(1, 26, 10)),
    (
        "minecraft:behavior.work_composter",
        FormatVersion::new(1, 26, 10),
    ),
    ("minecraft:block_climber", FormatVersion::new(1, 26, 20)),
    (
        "minecraft:block_movement_slowdown_immunity",
        FormatVersion::new(1, 26, 50),
    ),
    (
        "minecraft:body_rotation_always_follows_head",
        FormatVersion::new(1, 21, 120),
    ),
    (
        "minecraft:body_rotation_blocked",
        FormatVersion::new(1, 21, 120),
    ),
    ("minecraft:boss", FormatVersion::new(1, 26, 50)),
    ("minecraft:bucketable", FormatVersion::new(1, 26, 40)),
    (
        "minecraft:burns_in_daylight",
        FormatVersion::new(1, 21, 130),
    ),
    ("minecraft:can_climb", FormatVersion::new(1, 21, 120)),
    ("minecraft:can_fly", FormatVersion::new(1, 21, 120)),
    ("minecraft:can_join_raid", FormatVersion::new(1, 26, 20)),
    ("minecraft:can_power_jump", FormatVersion::new(1, 21, 120)),
    (
        "minecraft:can_stand_on_powder_snow",
        FormatVersion::new(1, 26, 50),
    ),
    (
        "minecraft:cannot_be_attacked",
        FormatVersion::new(1, 26, 40),
    ),
    ("minecraft:collision_box", FormatVersion::new(1, 26, 50)),
    ("minecraft:damage_over_time", FormatVersion::new(1, 26, 50)),
    ("minecraft:dimension_bound", FormatVersion::new(1, 26, 20)),
    ("minecraft:exhaustion_values", FormatVersion::new(1, 26, 50)),
    (
        "minecraft:experience_reward",
        FormatVersion::new(1, 21, 110),
    ),
    ("minecraft:fire_immune", FormatVersion::new(1, 21, 120)),
    ("minecraft:floats_in_liquid", FormatVersion::new(1, 26, 40)),
    ("minecraft:flocking", FormatVersion::new(1, 26, 50)),
    ("minecraft:freezing_immune", FormatVersion::new(1, 26, 50)),
    (
        "minecraft:freezing_vulnerable",
        FormatVersion::new(1, 26, 50),
    ),
    (
        "minecraft:game_event_movement_tracking",
        FormatVersion::new(1, 26, 50),
    ),
    ("minecraft:grows_crop", FormatVersion::new(1, 26, 50)),
    ("minecraft:hide", FormatVersion::new(1, 26, 40)),
    ("minecraft:home", FormatVersion::new(1, 26, 50)),
    (
        "minecraft:ignore_cannot_be_attacked",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:input_ground_controlled",
        FormatVersion::new(1, 21, 120),
    ),
    ("minecraft:insomnia", FormatVersion::new(1, 26, 50)),
    ("minecraft:instant_despawn", FormatVersion::new(1, 26, 50)),
    ("minecraft:interact", FormatVersion::new(1, 26, 50)),
    ("minecraft:is_baby", FormatVersion::new(1, 21, 120)),
    ("minecraft:is_charged", FormatVersion::new(1, 21, 120)),
    ("minecraft:is_chested", FormatVersion::new(1, 21, 120)),
    (
        "minecraft:is_hidden_when_invisible",
        FormatVersion::new(1, 21, 120),
    ),
    ("minecraft:is_ignited", FormatVersion::new(1, 21, 120)),
    (
        "minecraft:is_illager_captain",
        FormatVersion::new(1, 21, 120),
    ),
    ("minecraft:is_pregnant", FormatVersion::new(1, 21, 120)),
    ("minecraft:is_saddled", FormatVersion::new(1, 21, 120)),
    ("minecraft:is_shaking", FormatVersion::new(1, 21, 120)),
    ("minecraft:is_sheared", FormatVersion::new(1, 21, 120)),
    ("minecraft:is_stackable", FormatVersion::new(1, 21, 120)),
    ("minecraft:is_stunned", FormatVersion::new(1, 21, 120)),
    ("minecraft:is_tamed", FormatVersion::new(1, 21, 120)),
    ("minecraft:item_hopper", FormatVersion::new(1, 26, 20)),
    ("minecraft:jump.dynamic", FormatVersion::new(1, 26, 40)),
    ("minecraft:jump.static", FormatVersion::new(1, 26, 40)),
    (
        "minecraft:managed_wandering_trader",
        FormatVersion::new(1, 26, 40),
    ),
    (
        "minecraft:memory_behavior.follow_target",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:memory_behavior.look_at_entity",
        FormatVersion::new(1, 26, 10),
    ),
    (
        "minecraft:memory_behavior.move_to_position",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:memory_behavior.wait",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:memory_sensor.find_nearest_block",
        FormatVersion::new(1, 26, 30),
    ),
    (
        "minecraft:memory_sensor.find_nearest_entity",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:memory_sensor.find_nearest_poi",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:memory_sensor.game_event",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:memory_sensor.in_range_of_entity",
        FormatVersion::new(1, 26, 0),
    ),
    (
        "minecraft:not_pickable_from_inside",
        FormatVersion::new(1, 26, 40),
    ),
    ("minecraft:on_death", FormatVersion::new(1, 26, 40)),
    (
        "minecraft:on_equipment_changed",
        FormatVersion::new(1, 26, 20),
    ),
    ("minecraft:on_friendly_anger", FormatVersion::new(1, 26, 40)),
    ("minecraft:on_hurt", FormatVersion::new(1, 26, 40)),
    ("minecraft:on_hurt_by_player", FormatVersion::new(1, 26, 40)),
    ("minecraft:on_ignite", FormatVersion::new(1, 26, 40)),
    ("minecraft:on_start_landing", FormatVersion::new(1, 26, 40)),
    ("minecraft:on_start_takeoff", FormatVersion::new(1, 26, 40)),
    (
        "minecraft:on_target_acquired",
        FormatVersion::new(1, 26, 40),
    ),
    ("minecraft:on_target_escape", FormatVersion::new(1, 26, 40)),
    (
        "minecraft:on_wake_with_owner",
        FormatVersion::new(1, 26, 40),
    ),
    ("minecraft:out_of_control", FormatVersion::new(1, 26, 20)),
    ("minecraft:persistent", FormatVersion::new(1, 26, 40)),
    ("minecraft:physics", FormatVersion::new(1, 26, 50)),
    ("minecraft:preferred_path", FormatVersion::new(1, 26, 50)),
    ("minecraft:projectile", FormatVersion::new(1, 26, 50)),
    (
        "minecraft:pushable_by_entity",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:renders_when_invisible",
        FormatVersion::new(1, 21, 120),
    ),
    (
        "minecraft:rotation_axis_aligned",
        FormatVersion::new(1, 21, 120),
    ),
    (
        "minecraft:rotation_locked_to_vehicle",
        FormatVersion::new(1, 21, 120),
    ),
    ("minecraft:spawn_on_death", FormatVersion::new(1, 26, 20)),
    ("minecraft:suspect_tracking", FormatVersion::new(1, 26, 20)),
    ("minecraft:tamemount", FormatVersion::new(1, 26, 50)),
    ("minecraft:tick_world", FormatVersion::new(1, 21, 100)),
    ("minecraft:trade_resupply", FormatVersion::new(1, 26, 40)),
    ("minecraft:trade_table", FormatVersion::new(1, 26, 50)),
    ("minecraft:transient", FormatVersion::new(1, 26, 20)),
    ("minecraft:trust", FormatVersion::new(1, 26, 40)),
    (
        "minecraft:uses_legacy_ambient_sounds",
        FormatVersion::new(1, 26, 50),
    ),
    (
        "minecraft:uses_legacy_friction",
        FormatVersion::new(1, 26, 20),
    ),
    (
        "minecraft:uses_uniform_air_drag",
        FormatVersion::new(1, 26, 20),
    ),
    ("minecraft:vibration_damper", FormatVersion::new(1, 26, 20)),
    (
        "minecraft:vibration_listener",
        FormatVersion::new(1, 26, 20),
    ),
    ("minecraft:wants_jockey", FormatVersion::new(1, 26, 40)),
    (
        "minecraft:wither_target_highest_damage",
        FormatVersion::new(1, 26, 30),
    ),
];

// The components first typed at 1.26.60: the keys the step to 1.26.60 only converts. `(key, first typed)`, sorted by key.
const FIRST_TYPED_1_26_60_COMPONENTS: &[(&str, FormatVersion)] = &[
    ("minecraft:annotation.break_door", V1_26_60),
    ("minecraft:attack", V1_26_60),
    ("minecraft:buoyant", V1_26_60),
    ("minecraft:dash_action", V1_26_60),
    ("minecraft:economy_trade_table", V1_26_60),
    ("minecraft:healable", V1_26_60),
    ("minecraft:inventory", V1_26_60),
    ("minecraft:movement.amphibious", V1_26_60),
    ("minecraft:movement.basic", V1_26_60),
    ("minecraft:movement.fly", V1_26_60),
    ("minecraft:movement.generic", V1_26_60),
    ("minecraft:movement.glide", V1_26_60),
    ("minecraft:movement.hover", V1_26_60),
    ("minecraft:movement.jump", V1_26_60),
    ("minecraft:movement.skip", V1_26_60),
    ("minecraft:movement.sway", V1_26_60),
    ("minecraft:pushable_by_block", V1_26_60),
    ("minecraft:rail_movement", V1_26_60),
    ("minecraft:rideable", V1_26_60),
    ("minecraft:scale_by_age", V1_26_60),
    ("minecraft:shareables", V1_26_60),
    ("minecraft:spell_effects", V1_26_60),
    ("minecraft:strength", V1_26_60),
    ("minecraft:teleport", V1_26_60),
    ("minecraft:timer", V1_26_60),
    ("minecraft:trail", V1_26_60),
    ("minecraft:transformation", V1_26_60),
];

/// 1.26.60, the version at which the step to it types its 31 keys.
const V1_26_60: FormatVersion = FormatVersion::new(1, 26, 60);

// The four goals the step to 1.26.60 only converts, which type them at 1.26.60,
// sorted by key.
const GOALS_1_26_60: &[(&str, FormatVersion)] = &[
    ("minecraft:behavior.float_tempt", V1_26_60),
    ("minecraft:behavior.panic", V1_26_60),
    ("minecraft:behavior.send_event", V1_26_60),
    ("minecraft:behavior.tempt", V1_26_60),
];

// The four Brain keys the game first binds in 1.26.50, sorted by key.
const BRAIN_1_26_50: &[(&str, FormatVersion)] = &[
    (
        "minecraft:memory_behavior.jump_to_block",
        FormatVersion::new(1, 26, 50),
    ),
    (
        "minecraft:memory_behavior.prepare_ram_attack",
        FormatVersion::new(1, 26, 50),
    ),
    (
        "minecraft:memory_behavior.ram_attack",
        FormatVersion::new(1, 26, 50),
    ),
    (
        "minecraft:memory_sensor.find_ram_charge_position",
        FormatVersion::new(1, 26, 50),
    ),
];

/// Every table of `(key, first typed)`, each sorted by key.
const TABLES: &[&[(&str, FormatVersion)]] = &[
    FIRST_TYPED,
    FIRST_TYPED_1_26_60_COMPONENTS,
    GOALS_1_26_60,
    BRAIN_1_26_50,
];

/// The version at which vanilla first types `key`; `None` for a key that stays dynamic.
pub(crate) fn first_typed(key: &str) -> Option<FormatVersion> {
    TABLES.iter().find_map(|table| {
        let i = table.binary_search_by(|(name, _)| (*name).cmp(key)).ok()?;
        Some(table[i].1)
    })
}

#[cfg(test)]
mod tests {
    use super::{TABLES, first_typed};

    /// Each table is sorted (the lookup is a binary search) and no key has two rows.
    #[test]
    fn the_tables_are_sorted_and_name_each_key_once() {
        let mut keys = Vec::new();
        for table in TABLES {
            assert!(table.is_sorted_by_key(|(key, _)| *key));
            keys.extend(table.iter().map(|(key, _)| *key));
        }
        let all = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), all, "a key has two rows");
        for key in keys {
            assert!(first_typed(key).is_some(), "{key}");
        }
    }
}
