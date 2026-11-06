//! The JSON fragments the steps insert, built as values: a step cannot fail, so no fragment is
//! text to parse. Each is pinned below against the text it replaced.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::ops::{array, empty_object, f32_value, object, strings};
use facet_value::VArray;

/// The snow golem's `minecraft:trail` (1.12.0): snow below the temperature 0.81 (an `f32`,
/// widened).
pub(super) fn snow_trail() -> Value {
    object([
        ("block_type", Value::from("snow_layer")),
        (
            "spawn_filter",
            object([
                ("test", Value::from("is_temperature_value")),
                ("operator", Value::from("<")),
                ("value", f32_value(0.81)),
            ]),
        ),
    ])
}

/// The pufferfish's `minecraft:movement` and `minecraft:underwater_movement` (1.12.0): 0.13 (an
/// `f32`, widened).
pub(super) fn slow_movement() -> Value {
    object([("value", f32_value(0.13))])
}

/// `minecraft:despawn` with `entries` as its one `all_of` (1.15.0).
pub(super) fn despawn(entries: Vec<Value>) -> Value {
    let all_of: VArray = entries.into_iter().collect();
    object([("filters", array([object([("all_of", all_of.into())])]))])
}

/// A slime's or a magma cube's `minecraft:area_attack` (1.19.0): `damage` per tick on the
/// `families`.
pub(super) fn slime_area_attack(damage: i32, families: &[&str]) -> Value {
    let filters: VArray = families
        .iter()
        .map(|family| {
            object([
                ("test", Value::from("is_family")),
                ("subject", Value::from("other")),
                ("value", Value::from(*family)),
            ])
        })
        .collect();
    object([
        ("damage_range", Value::from(0.15)),
        ("damage_per_tick", Value::from(damage)),
        ("damage_cooldown", Value::from(0.0)),
        ("cause", Value::from("entity_attack")),
        ("entity_filter", object([("any_of", filters.into())])),
    ])
}

/// `minecraft:apply_knockback_rules` with one preset (1.26.10).
pub(super) fn knockback_rules(horizontal: f64, vertical: f64, cap: f64, water: bool) -> Value {
    object([(
        "presets",
        array([object([
            ("horizontal_power", Value::from(horizontal)),
            ("vertical_power", Value::from(vertical)),
            ("vertical_velocity_cap", Value::from(cap)),
            ("check_if_target_is_immersed_in_water", Value::from(water)),
        ])]),
    )])
}

/// `hurt_on_condition` with no conditions yet (1.12.0).
pub(super) fn damage_conditions() -> Value {
    object([("damage_conditions", array([]))])
}

/// The lava damage entry (1.12.0).
pub(super) fn lava_condition() -> Value {
    object([
        (
            "filters",
            object([
                ("test", Value::from("in_lava")),
                ("subject", Value::from("self")),
                ("operator", Value::from("==")),
                ("value", Value::from(true)),
            ]),
        ),
        ("cause", Value::from("lava")),
        ("damage_per_tick", Value::from(4)),
    ])
}

/// The snow golem's temperature entry (1.12.0).
pub(super) fn temperature_condition() -> Value {
    object([
        (
            "filters",
            object([
                ("test", Value::from("is_temperature_value")),
                ("operator", Value::from(">")),
                ("value", Value::from(1.0)),
            ]),
        ),
        ("cause", Value::from("temperature")),
        ("damage_per_tick", Value::from(1)),
    ])
}

/// The rain entry: the snow golem's, and `hurt_when_wet`'s replacement (1.12.0).
pub(super) fn rain_condition() -> Value {
    object([
        (
            "filters",
            object([
                ("test", Value::from("in_water_or_rain")),
                ("operator", Value::from("==")),
                ("value", Value::from(true)),
            ]),
        ),
        ("cause", Value::from("drowning")),
        ("damage_per_tick", Value::from(1)),
    ])
}

/// The iron golem's `target_when_pushed` entity type (1.12.0).
pub(super) fn iron_golem_target() -> Value {
    object([(
        "minecraft:filters",
        object([(
            "minecraft:all_of",
            array([
                object([
                    ("test", Value::from("is_family")),
                    ("subject", Value::from("other")),
                    ("value", Value::from("monster")),
                ]),
                object([
                    ("test", Value::from("is_family")),
                    ("subject", Value::from("other")),
                    ("operator", Value::from("!=")),
                    ("value", Value::from("creeper")),
                ]),
            ]),
        )]),
    )])
}

/// The turtle's `damage_sensor` when it has none (1.12.0).
pub(super) fn lightning_sensor() -> Value {
    object([(
        "triggers",
        array([object([
            ("cause", Value::from("lightning")),
            ("deals_damage", Value::from(true)),
            ("damage_multiplier", Value::from(2000.0)),
        ])]),
    )])
}

/// The turtle's lightning trigger (1.12.0).
pub(super) fn lightning_trigger() -> Value {
    object([
        ("cause", Value::from("lightning")),
        ("deals_damage", Value::from(true)),
        ("damage_multiplier", Value::from(2000.0)),
    ])
}

/// The pufferfish's puff state machine: its groups and events (1.12.0).
#[allow(clippy::too_many_lines, reason = "one fragment, member by member")]
pub(super) fn pufferfish() -> Value {
    object([
        (
            "component_groups",
            object([
                (
                    "minecraft:normal_puff",
                    object([
                        ("minecraft:variant", object([("value", Value::from(0))])),
                        (
                            "minecraft:entity_sensor",
                            object([
                                ("sensor_range", Value::from(1.5)),
                                ("minimum_count", Value::from(1)),
                                ("event", Value::from("minecraft:to_full_puff")),
                                (
                                    "event_filters",
                                    object([(
                                        "any_of",
                                        array([
                                            object([
                                                ("test", Value::from("is_family")),
                                                ("subject", Value::from("other")),
                                                ("value", Value::from("mob")),
                                            ]),
                                            object([
                                                ("test", Value::from("is_family")),
                                                ("subject", Value::from("other")),
                                                ("value", Value::from("armor_stand")),
                                            ]),
                                            object([(
                                                "all_of",
                                                array([
                                                    object([
                                                        ("test", Value::from("is_family")),
                                                        ("subject", Value::from("other")),
                                                        ("value", Value::from("player")),
                                                    ]),
                                                    object([(
                                                        "none_of",
                                                        object([
                                                            ("test", Value::from("has_ability")),
                                                            ("subject", Value::from("other")),
                                                            ("value", Value::from("instabuild")),
                                                        ]),
                                                    )]),
                                                ]),
                                            )]),
                                        ]),
                                    )]),
                                ),
                            ]),
                        ),
                    ]),
                ),
                (
                    "minecraft:half_puff_inflate",
                    object([
                        ("minecraft:variant", object([("value", Value::from(1))])),
                        (
                            "minecraft:timer",
                            object([
                                ("looping", Value::from(false)),
                                ("time", f32_value(0.05)),
                                ("randomInterval", Value::from(false)),
                                (
                                    "time_down_event",
                                    object([("event", Value::from("minecraft:on_full_puff"))]),
                                ),
                            ]),
                        ),
                    ]),
                ),
                (
                    "minecraft:half_puff_deflate",
                    object([
                        ("minecraft:variant", object([("value", Value::from(1))])),
                        (
                            "minecraft:timer",
                            object([
                                ("looping", Value::from(false)),
                                ("time", Value::from(1)),
                                ("randomInterval", Value::from(false)),
                                (
                                    "time_down_event",
                                    object([("event", Value::from("minecraft:on_normal_puff"))]),
                                ),
                            ]),
                        ),
                    ]),
                ),
                (
                    "minecraft:start_deflate",
                    object([(
                        "minecraft:timer",
                        object([
                            ("looping", Value::from(false)),
                            ("time", Value::from(10)),
                            ("randomInterval", Value::from(false)),
                            (
                                "time_down_event",
                                object([("event", Value::from("minecraft:on_deflate"))]),
                            ),
                        ]),
                    )]),
                ),
                (
                    "minecraft:full_puff",
                    object([
                        ("minecraft:variant", object([("value", Value::from(2))])),
                        (
                            "minecraft:area_attack",
                            object([
                                ("damage_range", f32_value(0.2)),
                                ("damage_per_tick", Value::from(2)),
                                ("cause", Value::from("contact")),
                                (
                                    "entity_filter",
                                    object([(
                                        "any_of",
                                        array([
                                            object([
                                                ("test", Value::from("is_family")),
                                                ("subject", Value::from("other")),
                                                ("value", Value::from("monster")),
                                            ]),
                                            object([
                                                ("test", Value::from("is_family")),
                                                ("subject", Value::from("other")),
                                                ("value", Value::from("player")),
                                            ]),
                                        ]),
                                    )]),
                                ),
                            ]),
                        ),
                        (
                            "minecraft:mob_effect",
                            object([
                                ("effect_range", f32_value(0.2)),
                                ("mob_effect", Value::from("poison")),
                                ("effect_time", Value::from(10)),
                                (
                                    "entity_filter",
                                    object([(
                                        "any_of",
                                        array([
                                            object([
                                                ("test", Value::from("is_family")),
                                                ("subject", Value::from("other")),
                                                ("value", Value::from("monster")),
                                            ]),
                                            object([
                                                ("test", Value::from("is_family")),
                                                ("subject", Value::from("other")),
                                                ("value", Value::from("player")),
                                            ]),
                                        ]),
                                    )]),
                                ),
                            ]),
                        ),
                    ]),
                ),
                (
                    "minecraft:deflate_sensor",
                    object([(
                        "minecraft:entity_sensor",
                        object([
                            ("sensor_range", Value::from(10.0)),
                            ("require_all", Value::from(true)),
                            ("event", Value::from("minecraft:from_full_puff")),
                            (
                                "event_filters",
                                object([(
                                    "none_of",
                                    array([
                                        object([
                                            ("test", Value::from("is_family")),
                                            ("subject", Value::from("other")),
                                            ("value", Value::from("mob")),
                                        ]),
                                        object([
                                            ("test", Value::from("is_family")),
                                            ("subject", Value::from("other")),
                                            ("value", Value::from("armor_stand")),
                                        ]),
                                        object([(
                                            "all_of",
                                            array([
                                                object([
                                                    ("test", Value::from("is_family")),
                                                    ("subject", Value::from("other")),
                                                    ("value", Value::from("player")),
                                                ]),
                                                object([(
                                                    "none_of",
                                                    object([
                                                        ("test", Value::from("has_ability")),
                                                        ("subject", Value::from("other")),
                                                        ("value", Value::from("instabuild")),
                                                    ]),
                                                )]),
                                            ]),
                                        )]),
                                    ]),
                                )]),
                            ),
                        ]),
                    )]),
                ),
            ]),
        ),
        (
            "events",
            object([
                (
                    "minecraft:entity_spawned",
                    object([(
                        "add",
                        object([("component_groups", strings(&["minecraft:normal_puff"]))]),
                    )]),
                ),
                (
                    "minecraft:to_full_puff",
                    object([
                        (
                            "add",
                            object([(
                                "component_groups",
                                strings(&["minecraft:half_puff_inflate"]),
                            )]),
                        ),
                        (
                            "remove",
                            object([("component_groups", strings(&["minecraft:normal_puff"]))]),
                        ),
                    ]),
                ),
                (
                    "minecraft:on_full_puff",
                    object([
                        (
                            "add",
                            object([(
                                "component_groups",
                                strings(&["minecraft:full_puff", "minecraft:deflate_sensor"]),
                            )]),
                        ),
                        (
                            "remove",
                            object([(
                                "component_groups",
                                strings(&["minecraft:half_puff_inflate"]),
                            )]),
                        ),
                    ]),
                ),
                (
                    "minecraft:from_full_puff",
                    object([
                        (
                            "add",
                            object([("component_groups", strings(&["minecraft:start_deflate"]))]),
                        ),
                        (
                            "remove",
                            object([("component_groups", strings(&["minecraft:deflate_sensor"]))]),
                        ),
                    ]),
                ),
                (
                    "minecraft:on_deflate",
                    object([
                        (
                            "add",
                            object([(
                                "component_groups",
                                strings(&["minecraft:half_puff_deflate"]),
                            )]),
                        ),
                        (
                            "remove",
                            object([(
                                "component_groups",
                                strings(&["minecraft:full_puff", "minecraft:start_deflate"]),
                            )]),
                        ),
                    ]),
                ),
                (
                    "minecraft:on_normal_puff",
                    object([
                        (
                            "add",
                            object([("component_groups", strings(&["minecraft:normal_puff"]))]),
                        ),
                        (
                            "remove",
                            object([(
                                "component_groups",
                                strings(&["minecraft:half_puff_deflate"]),
                            )]),
                        ),
                    ]),
                ),
            ]),
        ),
    ])
}

/// A `damage_sensor` with no triggers (1.12.0).
pub(super) fn no_triggers() -> Value {
    object([("triggers", array([]))])
}

/// The `rider_count` filter of `spawn_entity` (1.12.0).
pub(super) fn no_riders() -> Value {
    object([
        ("test", Value::from("rider_count")),
        ("subject", Value::from("self")),
        ("operator", Value::from("==")),
        ("value", Value::from(0)),
    ])
}

/// `behavior.panic`'s damage sources (1.15.0).
pub(super) fn panic_damage_sources() -> Value {
    strings(&["fire", "fire_tick", "magma"])
}

/// A despawn filter: not persistent (1.15.0).
pub(super) fn not_persistent() -> Value {
    object([
        ("test", Value::from("is_persistent")),
        ("value", Value::from(false)),
    ])
}

/// A despawn filter: inactive for 600 ticks (1.15.0).
pub(super) fn inactive() -> Value {
    object([
        ("test", Value::from("inactivity_timer")),
        ("subject", Value::from("self")),
        ("value", Value::from(600)),
    ])
}

/// A despawn filter: one chance in 50 (1.15.0).
pub(super) fn random_chance_50() -> Value {
    object([
        ("test", Value::from("random_chance")),
        ("value", Value::from(50)),
    ])
}

/// A despawn filter: no player within 54 blocks (1.15.0).
pub(super) fn far_from_players() -> Value {
    object([
        ("test", Value::from("distance_to_nearest_player")),
        ("operator", Value::from(">")),
        ("value", Value::from(54)),
    ])
}

/// A filter: daytime (1.15.0's despawn, 1.19.0's flower goals).
pub(super) fn is_daytime() -> Value {
    object([
        ("test", Value::from("is_daytime")),
        ("value", Value::from(true)),
    ])
}

/// A despawn filter of the fish: no player within 40 blocks (1.15.0).
pub(super) fn fish_far_from_players() -> Value {
    object([
        ("test", Value::from("distance_to_nearest_player")),
        ("operator", Value::from(">")),
        ("value", Value::from(40)),
    ])
}

/// A despawn filter: light level 8 or more (1.15.0).
pub(super) fn light_at_least_8() -> Value {
    object([
        ("test", Value::from("light_level")),
        ("operator", Value::from(">=")),
        ("value", Value::from(8)),
    ])
}

/// A despawn filter: a surface mob (1.15.0).
pub(super) fn surface_mob() -> Value {
    object([
        ("test", Value::from("surface_mob")),
        ("value", Value::from(true)),
    ])
}

/// A despawn filter: light level below 8 (1.15.0).
pub(super) fn light_below_8() -> Value {
    object([
        ("test", Value::from("light_level")),
        ("operator", Value::from("<")),
        ("value", Value::from(8)),
    ])
}

/// `minecraft:home` (1.16.0).
pub(super) fn home() -> Value {
    object([("restriction_radius", Value::from(16))])
}

/// The boat's groups and components (1.16.0).
#[allow(clippy::too_many_lines, reason = "one fragment, member by member")]
pub(super) fn boat() -> Value {
    object([
        (
            "component_groups",
            object([
                (
                    "minecraft:floating",
                    object([(
                        "minecraft:buoyant",
                        object([
                            (
                                "liquid_blocks",
                                strings(&["minecraft:water", "minecraft:flowing_water"]),
                            ),
                            ("simulate_waves", Value::from(true)),
                        ]),
                    )]),
                ),
                ("minecraft:can_ride", empty_object()),
                (
                    "minecraft:above_bubble_column_down",
                    object([(
                        "minecraft:buoyant",
                        object([
                            (
                                "liquid_blocks",
                                strings(&["minecraft:water", "minecraft:flowing_water"]),
                            ),
                            ("simulate_waves", Value::from(false)),
                            ("drag_down_on_buoyancy_removed", Value::from(0.7)),
                        ]),
                    )]),
                ),
                (
                    "minecraft:above_bubble_column_up",
                    object([(
                        "minecraft:buoyant",
                        object([
                            (
                                "liquid_blocks",
                                strings(&["minecraft:water", "minecraft:flowing_water"]),
                            ),
                            ("simulate_waves", Value::from(false)),
                            ("drag_down_on_buoyancy_removed", Value::from(0.7)),
                        ]),
                    )]),
                ),
            ]),
        ),
        (
            "components",
            object([
                (
                    "minecraft:buoyant",
                    object([
                        (
                            "liquid_blocks",
                            strings(&["minecraft:water", "minecraft:flowing_water"]),
                        ),
                        ("simulate_waves", Value::from(false)),
                        ("drag_down_on_buoyancy_removed", Value::from(0.7)),
                    ]),
                ),
                (
                    "minecraft:timer",
                    object([
                        ("looping", Value::from(false)),
                        ("time", Value::from(3)),
                        (
                            "time_down_event",
                            object([
                                ("event", Value::from("minecraft:sink")),
                                ("target", Value::from("self")),
                            ]),
                        ),
                    ]),
                ),
                ("minecraft:out_of_control", empty_object()),
                (
                    "minecraft:inside_block_notifier",
                    object([(
                        "block_list",
                        array([
                            object([
                                (
                                    "block",
                                    object([
                                        ("name", Value::from("minecraft:bubble_column")),
                                        ("states", object([("drag_down", Value::from(true))])),
                                    ]),
                                ),
                                (
                                    "entered_block_event",
                                    object([
                                        (
                                            "event",
                                            Value::from("minecraft:entered_bubble_column_down"),
                                        ),
                                        ("target", Value::from("self")),
                                    ]),
                                ),
                                (
                                    "exited_block_event",
                                    object([
                                        ("event", Value::from("minecraft:exited_bubble_column")),
                                        ("target", Value::from("self")),
                                    ]),
                                ),
                            ]),
                            object([
                                (
                                    "block",
                                    object([
                                        ("name", Value::from("minecraft:bubble_column")),
                                        ("states", object([("drag_down", Value::from(false))])),
                                    ]),
                                ),
                                (
                                    "entered_block_event",
                                    object([
                                        (
                                            "event",
                                            Value::from("minecraft:entered_bubble_column_up"),
                                        ),
                                        ("target", Value::from("self")),
                                    ]),
                                ),
                                (
                                    "exited_block_event",
                                    object([
                                        ("event", Value::from("minecraft:exited_bubble_column")),
                                        ("target", Value::from("self")),
                                    ]),
                                ),
                            ]),
                        ]),
                    )]),
                ),
            ]),
        ),
        (
            "events",
            object([
                (
                    "minecraft:entered_bubble_column_down",
                    object([
                        (
                            "add",
                            object([(
                                "component_groups",
                                strings(&["minecraft:above_bubble_column_down"]),
                            )]),
                        ),
                        (
                            "remove",
                            object([("component_groups", strings(&["minecraft:floating"]))]),
                        ),
                    ]),
                ),
                (
                    "minecraft:entered_bubble_column_up",
                    object([
                        (
                            "add",
                            object([(
                                "component_groups",
                                strings(&["minecraft:above_bubble_column_up"]),
                            )]),
                        ),
                        (
                            "remove",
                            object([("component_groups", strings(&["minecraft:floating"]))]),
                        ),
                    ]),
                ),
                (
                    "minecraft:exited_bubble_column",
                    object([
                        (
                            "add",
                            object([(
                                "component_groups",
                                strings(&["minecraft:floating", "minecraft:can_ride"]),
                            )]),
                        ),
                        (
                            "remove",
                            object([(
                                "component_groups",
                                strings(&[
                                    "minecraft:above_bubble_column_down",
                                    "minecraft:above_bubble_column_up",
                                ]),
                            )]),
                        ),
                    ]),
                ),
                (
                    "minecraft:sink",
                    object([(
                        "remove",
                        object([(
                            "component_groups",
                            strings(&[
                                "minecraft:floating",
                                "minecraft:can_ride",
                                "minecraft:above_bubble_column_down",
                                "minecraft:above_bubble_column_up",
                            ]),
                        )]),
                    )]),
                ),
            ]),
        ),
    ])
}

/// The wild ocelot child's `spawn_entity` (1.18.0).
pub(super) fn ocelot_spawn() -> Value {
    object([(
        "entities",
        object([
            ("min_wait_time", Value::from(0)),
            ("max_wait_time", Value::from(0)),
            ("num_to_spawn", Value::from(2)),
            ("single_use", Value::from(true)),
            ("spawn_entity", Value::from("minecraft:ocelot")),
            ("spawn_event", Value::from("minecraft:entity_born")),
            ("spawn_method", Value::from("born")),
            ("spawn_sound", Value::from("")),
            (
                "filters",
                array([object([
                    ("test", Value::from("random_chance")),
                    ("value", Value::from(7)),
                ])]),
            ),
        ]),
    )])
}

/// `variable_max_auto_step` as the step writes it (1.19.0).
pub(super) fn auto_step() -> Value {
    object([
        ("base_value", Value::from(1.0625)),
        ("jump_prevented_value", Value::from(0.5625)),
    ])
}

/// The witch's shooter; the filter JSON follows vanilla's own witch file (1.19.50).
pub(super) fn witch_shooter() -> Value {
    object([
        ("def", Value::from("minecraft:splash_potion")),
        ("power", Value::from(0.75)),
        ("aux_val", Value::from(23)),
        ("sound", Value::from("throw")),
        ("magic", Value::from(true)),
        (
            "projectiles",
            array([
                object([
                    ("aux_val", Value::from(17)),
                    (
                        "filters",
                        object([(
                            "all_of",
                            array([
                                object([
                                    ("test", Value::from("target_distance")),
                                    ("subject", Value::from("self")),
                                    ("operator", Value::from(">=")),
                                    ("value", Value::from(8)),
                                ]),
                                object([(
                                    "none_of",
                                    array([object([
                                        ("test", Value::from("has_mob_effect")),
                                        ("subject", Value::from("other")),
                                        ("value", Value::from("slowness")),
                                    ])]),
                                )]),
                            ]),
                        )]),
                    ),
                ]),
                object([
                    ("aux_val", Value::from(25)),
                    (
                        "filters",
                        object([(
                            "all_of",
                            array([
                                object([
                                    ("test", Value::from("actor_health")),
                                    ("subject", Value::from("other")),
                                    ("operator", Value::from(">=")),
                                    ("value", Value::from(8)),
                                ]),
                                object([(
                                    "none_of",
                                    array([object([
                                        ("test", Value::from("has_mob_effect")),
                                        ("subject", Value::from("other")),
                                        ("value", Value::from("poison")),
                                    ])]),
                                )]),
                            ]),
                        )]),
                    ),
                ]),
                object([
                    ("aux_val", Value::from(34)),
                    ("chance", Value::from(0.25)),
                    (
                        "filters",
                        object([(
                            "all_of",
                            array([
                                object([
                                    ("test", Value::from("target_distance")),
                                    ("subject", Value::from("self")),
                                    ("operator", Value::from("<=")),
                                    ("value", Value::from(3)),
                                ]),
                                object([(
                                    "none_of",
                                    array([object([
                                        ("test", Value::from("has_mob_effect")),
                                        ("subject", Value::from("other")),
                                        ("value", Value::from("weakness")),
                                    ])]),
                                )]),
                            ]),
                        )]),
                    ),
                ]),
            ]),
        ),
    ])
}

/// The witch's potion goal (1.19.50).
pub(super) fn witch_drink_potion() -> Value {
    object([
        ("priority", Value::from(1)),
        ("speed_modifier", Value::from(-0.25)),
        (
            "potions",
            array([
                object([
                    ("id", Value::from(19)),
                    ("chance", Value::from(0.15)),
                    (
                        "filters",
                        object([
                            ("test", Value::from("is_underwater")),
                            ("subject", Value::from("self")),
                            ("value", Value::from(true)),
                        ]),
                    ),
                ]),
                object([
                    ("id", Value::from(12)),
                    ("chance", Value::from(0.15)),
                    (
                        "filters",
                        object([(
                            "any_of",
                            array([
                                object([
                                    ("test", Value::from("on_fire")),
                                    ("subject", Value::from("self")),
                                    ("value", Value::from(true)),
                                ]),
                                object([
                                    ("test", Value::from("on_hot_block")),
                                    ("subject", Value::from("self")),
                                    ("value", Value::from(true)),
                                ]),
                            ]),
                        )]),
                    ),
                ]),
                object([
                    ("id", Value::from(21)),
                    ("chance", Value::from(0.05)),
                    (
                        "filters",
                        object([
                            ("test", Value::from("is_missing_health")),
                            ("subject", Value::from("self")),
                            ("value", Value::from(true)),
                        ]),
                    ),
                ]),
                object([
                    ("id", Value::from(14)),
                    ("chance", Value::from(0.25)),
                    (
                        "filters",
                        object([(
                            "all_of",
                            array([
                                object([
                                    ("test", Value::from("has_target")),
                                    ("subject", Value::from("self")),
                                    ("value", Value::from(true)),
                                ]),
                                object([
                                    ("test", Value::from("target_distance")),
                                    ("subject", Value::from("self")),
                                    ("operator", Value::from(">=")),
                                    ("value", Value::from(11)),
                                ]),
                            ]),
                        )]),
                    ),
                ]),
            ]),
        ),
    ])
}

/// A trade filter: not in water (1.19.50).
pub(super) fn not_in_water() -> Value {
    object([
        ("test", Value::from("in_water")),
        ("value", Value::from(false)),
    ])
}

/// A trade filter: on the ground or sleeping (1.19.50).
pub(super) fn on_ground_or_sleeping() -> Value {
    object([(
        "any_of",
        array([
            object([
                ("test", Value::from("on_ground")),
                ("value", Value::from(true)),
            ]),
            object([
                ("test", Value::from("is_sleeping")),
                ("value", Value::from(true)),
            ]),
        ]),
    )])
}

/// `behavior.panic`'s damage sources (1.20.80).
pub(super) fn all_damage_sources() -> Value {
    strings(&["all"])
}

/// The armor slot mapping of an equippable entity (1.21.0).
pub(super) fn chest_armor_slot() -> Value {
    object([("armor_slot", Value::from("slot.armor.chest"))])
}

/// The dolphin's `behavior.swim_up_for_breath` (1.21.10).
pub(super) fn priority_one() -> Value {
    object([("priority", Value::from(1))])
}

/// The strider's `movement_sound_distance_offset` (1.21.40).
pub(super) fn strider_sound_offset() -> Value {
    object([("value", Value::from(0.6))])
}

/// The legacy goals' `xz_range` (1.21.90).
pub(super) fn xz_range() -> Value {
    array([Value::from(-1), Value::from(1)])
}

/// `y_range` of the first legacy goal (1.21.90).
pub(super) fn y_range_2() -> Value {
    array([Value::from(0), Value::from(2)])
}

/// `y_range` of the second legacy goal (1.21.90).
pub(super) fn y_range_3() -> Value {
    array([Value::from(0), Value::from(3)])
}

/// The legacy enderman-holdable blocks (1.21.90).
pub(super) fn take_blocks() -> Value {
    strings(&[
        "minecraft:grass_block",
        "minecraft:dirt",
        "minecraft:coarse_dirt",
        "minecraft:podzol",
        "minecraft:sand",
        "minecraft:red_sand",
        "minecraft:gravel",
        "minecraft:crimson_roots",
        "minecraft:warped_roots",
        "minecraft:dandelion",
        "minecraft:poppy",
        "minecraft:brown_mushroom",
        "minecraft:red_mushroom",
        "minecraft:crimson_fungus",
        "minecraft:warped_fungus",
        "minecraft:tnt",
        "minecraft:cactus",
        "minecraft:clay",
        "minecraft:pumpkin",
        "minecraft:carved_pumpkin",
        "minecraft:melon_block",
        "minecraft:mycelium",
        "minecraft:warped_nylium",
        "minecraft:crimson_nylium",
        "minecraft:dirt_with_roots",
        "minecraft:cactus_flower",
    ])
}

/// The slime's `regular_skip_data` (1.21.130).
pub(super) fn regular_skip_data() -> Value {
    object([
        ("distance_scale", Value::from(0.8)),
        ("height", Value::from(0.1)),
        ("jump_delay", Value::from(20)),
        ("animation_duration", Value::from(3)),
    ])
}

/// The slime's `fast_skip_data` (1.21.130).
pub(super) fn fast_skip_data() -> Value {
    object([
        ("distance_scale", Value::from(1.75)),
        ("height", Value::from(0.15)),
        ("jump_delay", Value::from(1)),
        ("animation_duration", Value::from(1)),
    ])
}

/// `behavior.follow_target_leader`'s `leader_filters` (1.26.10).
pub(super) fn captain_leader_filter() -> Value {
    object([
        ("test", Value::from("has_component")),
        ("subject", Value::from("other")),
        ("value", Value::from("minecraft:is_illager_captain")),
    ])
}

/// The teleport filter the 1.26.50 fixer builds: one `is_family` test of `other` per
/// projectile family, under `none_of` (1.26.40).
pub(super) fn enderman_projectile_filter() -> Value {
    object([(
        "none_of",
        array([
            object([
                ("test", Value::from("is_family")),
                ("subject", Value::from("other")),
                ("value", Value::from("splash_potion")),
            ]),
            object([
                ("test", Value::from("is_family")),
                ("subject", Value::from("other")),
                ("value", Value::from("lingering_potion")),
            ]),
            object([
                ("test", Value::from("is_family")),
                ("subject", Value::from("other")),
                ("value", Value::from("snowball")),
            ]),
            object([
                ("test", Value::from("is_family")),
                ("subject", Value::from("other")),
                ("value", Value::from("xp_bottle")),
            ]),
        ]),
    )])
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::json::{from_str, to_json};

    /// The same value and the same text: `VNumber`'s `==` takes `1` for `1.0`, the text does
    /// not.
    fn assert_is(built: &Value, text: &str) {
        let parsed: Value = from_str(text).expect("the text the fragment replaced");
        assert_eq!(built, &parsed, "{text}");
        assert_eq!(to_json(built).ok(), to_json(&parsed).ok(), "{text}");
    }

    const DAMAGE_CONDITIONS_TEXT: &str = r#"{"damage_conditions": []}"#;
    const LAVA_CONDITION_TEXT: &str = r#"{"filters": {"test": "in_lava", "subject": "self", "operator": "==", "value": true}, "cause": "lava", "damage_per_tick": 4}"#;
    const TEMPERATURE_CONDITION_TEXT: &str = r#"{"filters": {"test": "is_temperature_value", "operator": ">", "value": 1.0}, "cause": "temperature", "damage_per_tick": 1}"#;
    const RAIN_CONDITION_TEXT: &str = r#"{"filters": {"test": "in_water_or_rain", "operator": "==", "value": true}, "cause": "drowning", "damage_per_tick": 1}"#;
    const IRON_GOLEM_TARGET_TEXT: &str = r#"{"minecraft:filters": {"minecraft:all_of": [
                {"test": "is_family", "subject": "other", "value": "monster"},
                {"test": "is_family", "subject": "other", "operator": "!=", "value": "creeper"}
            ]}}"#;
    const LIGHTNING_SENSOR_TEXT: &str = r#"{"triggers": [{"cause": "lightning", "deals_damage": true, "damage_multiplier": 2000.0}]}"#;
    const LIGHTNING_TRIGGER_TEXT: &str =
        r#"{"cause": "lightning", "deals_damage": true, "damage_multiplier": 2000.0}"#;
    const PUFFERFISH_TEXT: &str = r#"{
  "component_groups": {
    "minecraft:normal_puff": {
      "minecraft:variant": {"value": 0},
      "minecraft:entity_sensor": {"sensor_range": 1.5, "minimum_count": 1, "event": "minecraft:to_full_puff", "event_filters": {"any_of": [
        {"test": "is_family", "subject": "other", "value": "mob"},
        {"test": "is_family", "subject": "other", "value": "armor_stand"},
        {"all_of": [{"test": "is_family", "subject": "other", "value": "player"}, {"none_of": {"test": "has_ability", "subject": "other", "value": "instabuild"}}]}
      ]}}
    },
    "minecraft:half_puff_inflate": {
      "minecraft:variant": {"value": 1},
      "minecraft:timer": {"looping": false, "time": 0.05000000074505806, "randomInterval": false, "time_down_event": {"event": "minecraft:on_full_puff"}}
    },
    "minecraft:half_puff_deflate": {
      "minecraft:variant": {"value": 1},
      "minecraft:timer": {"looping": false, "time": 1, "randomInterval": false, "time_down_event": {"event": "minecraft:on_normal_puff"}}
    },
    "minecraft:start_deflate": {
      "minecraft:timer": {"looping": false, "time": 10, "randomInterval": false, "time_down_event": {"event": "minecraft:on_deflate"}}
    },
    "minecraft:full_puff": {
      "minecraft:variant": {"value": 2},
      "minecraft:area_attack": {"damage_range": 0.20000000298023224, "damage_per_tick": 2, "cause": "contact", "entity_filter": {"any_of": [
        {"test": "is_family", "subject": "other", "value": "monster"},
        {"test": "is_family", "subject": "other", "value": "player"}
      ]}},
      "minecraft:mob_effect": {"effect_range": 0.20000000298023224, "mob_effect": "poison", "effect_time": 10, "entity_filter": {"any_of": [
        {"test": "is_family", "subject": "other", "value": "monster"},
        {"test": "is_family", "subject": "other", "value": "player"}
      ]}}
    },
    "minecraft:deflate_sensor": {
      "minecraft:entity_sensor": {"sensor_range": 10.0, "require_all": true, "event": "minecraft:from_full_puff", "event_filters": {"none_of": [
        {"test": "is_family", "subject": "other", "value": "mob"},
        {"test": "is_family", "subject": "other", "value": "armor_stand"},
        {"all_of": [{"test": "is_family", "subject": "other", "value": "player"}, {"none_of": {"test": "has_ability", "subject": "other", "value": "instabuild"}}]}
      ]}}
    }
  },
  "events": {
    "minecraft:entity_spawned": {"add": {"component_groups": ["minecraft:normal_puff"]}},
    "minecraft:to_full_puff": {"add": {"component_groups": ["minecraft:half_puff_inflate"]}, "remove": {"component_groups": ["minecraft:normal_puff"]}},
    "minecraft:on_full_puff": {"add": {"component_groups": ["minecraft:full_puff", "minecraft:deflate_sensor"]}, "remove": {"component_groups": ["minecraft:half_puff_inflate"]}},
    "minecraft:from_full_puff": {"add": {"component_groups": ["minecraft:start_deflate"]}, "remove": {"component_groups": ["minecraft:deflate_sensor"]}},
    "minecraft:on_deflate": {"add": {"component_groups": ["minecraft:half_puff_deflate"]}, "remove": {"component_groups": ["minecraft:full_puff", "minecraft:start_deflate"]}},
    "minecraft:on_normal_puff": {"add": {"component_groups": ["minecraft:normal_puff"]}, "remove": {"component_groups": ["minecraft:half_puff_deflate"]}}
  }
}"#;
    const NO_TRIGGERS_TEXT: &str = r#"{"triggers": []}"#;
    const NO_RIDERS_TEXT: &str =
        r#"{"test": "rider_count", "subject": "self", "operator": "==", "value": 0}"#;
    const PANIC_DAMAGE_SOURCES_TEXT: &str = r#"["fire", "fire_tick", "magma"]"#;
    const NOT_PERSISTENT_TEXT: &str = r#"{"test": "is_persistent", "value": false}"#;
    const INACTIVE_TEXT: &str = r#"{"test": "inactivity_timer", "subject": "self", "value": 600}"#;
    const RANDOM_CHANCE_50_TEXT: &str = r#"{"test": "random_chance", "value": 50}"#;
    const FAR_FROM_PLAYERS_TEXT: &str =
        r#"{"test": "distance_to_nearest_player", "operator": ">", "value": 54}"#;
    const IS_DAYTIME_TEXT: &str = r#"{"test": "is_daytime", "value": true}"#;
    const FISH_FAR_FROM_PLAYERS_TEXT: &str =
        r#"{"test": "distance_to_nearest_player", "operator": ">", "value": 40}"#;
    const LIGHT_AT_LEAST_8_TEXT: &str = r#"{"test": "light_level", "operator": ">=", "value": 8}"#;
    const SURFACE_MOB_TEXT: &str = r#"{"test": "surface_mob", "value": true}"#;
    const LIGHT_BELOW_8_TEXT: &str = r#"{"test": "light_level", "operator": "<", "value": 8}"#;
    const HOME_TEXT: &str = r#"{"restriction_radius": 16}"#;
    const BOAT_TEXT: &str = r#"{
  "component_groups": {
    "minecraft:floating": {
      "minecraft:buoyant": {"liquid_blocks": ["minecraft:water", "minecraft:flowing_water"], "simulate_waves": true}
    },
    "minecraft:can_ride": {},
    "minecraft:above_bubble_column_down": {
      "minecraft:buoyant": {"liquid_blocks": ["minecraft:water", "minecraft:flowing_water"], "simulate_waves": false, "drag_down_on_buoyancy_removed": 0.7}
    },
    "minecraft:above_bubble_column_up": {
      "minecraft:buoyant": {"liquid_blocks": ["minecraft:water", "minecraft:flowing_water"], "simulate_waves": false, "drag_down_on_buoyancy_removed": 0.7}
    }
  },
  "components": {
    "minecraft:buoyant": {"liquid_blocks": ["minecraft:water", "minecraft:flowing_water"], "simulate_waves": false, "drag_down_on_buoyancy_removed": 0.7},
    "minecraft:timer": {"looping": false, "time": 3, "time_down_event": {"event": "minecraft:sink", "target": "self"}},
    "minecraft:out_of_control": {},
    "minecraft:inside_block_notifier": {"block_list": [
      {"block": {"name": "minecraft:bubble_column", "states": {"drag_down": true}},
       "entered_block_event": {"event": "minecraft:entered_bubble_column_down", "target": "self"},
       "exited_block_event": {"event": "minecraft:exited_bubble_column", "target": "self"}},
      {"block": {"name": "minecraft:bubble_column", "states": {"drag_down": false}},
       "entered_block_event": {"event": "minecraft:entered_bubble_column_up", "target": "self"},
       "exited_block_event": {"event": "minecraft:exited_bubble_column", "target": "self"}}
    ]}
  },
  "events": {
    "minecraft:entered_bubble_column_down": {"add": {"component_groups": ["minecraft:above_bubble_column_down"]}, "remove": {"component_groups": ["minecraft:floating"]}},
    "minecraft:entered_bubble_column_up": {"add": {"component_groups": ["minecraft:above_bubble_column_up"]}, "remove": {"component_groups": ["minecraft:floating"]}},
    "minecraft:exited_bubble_column": {"add": {"component_groups": ["minecraft:floating", "minecraft:can_ride"]}, "remove": {"component_groups": ["minecraft:above_bubble_column_down", "minecraft:above_bubble_column_up"]}},
    "minecraft:sink": {"remove": {"component_groups": ["minecraft:floating", "minecraft:can_ride", "minecraft:above_bubble_column_down", "minecraft:above_bubble_column_up"]}}
  }
}"#;
    const OCELOT_SPAWN_TEXT: &str = r#"{"entities": {"min_wait_time": 0, "max_wait_time": 0, "num_to_spawn": 2, "single_use": true,
                "spawn_entity": "minecraft:ocelot", "spawn_event": "minecraft:entity_born", "spawn_method": "born",
                "spawn_sound": "", "filters": [{"test": "random_chance", "value": 7}]}}"#;
    const AUTO_STEP_TEXT: &str = r#"{"base_value": 1.0625, "jump_prevented_value": 0.5625}"#;
    const WITCH_SHOOTER_TEXT: &str = r#"{"def": "minecraft:splash_potion", "power": 0.75, "aux_val": 23, "sound": "throw", "magic": true, "projectiles": [
  {"aux_val": 17, "filters": {"all_of": [{"test": "target_distance", "subject": "self", "operator": ">=", "value": 8}, {"none_of": [{"test": "has_mob_effect", "subject": "other", "value": "slowness"}]}]}},
  {"aux_val": 25, "filters": {"all_of": [{"test": "actor_health", "subject": "other", "operator": ">=", "value": 8}, {"none_of": [{"test": "has_mob_effect", "subject": "other", "value": "poison"}]}]}},
  {"aux_val": 34, "chance": 0.25, "filters": {"all_of": [{"test": "target_distance", "subject": "self", "operator": "<=", "value": 3}, {"none_of": [{"test": "has_mob_effect", "subject": "other", "value": "weakness"}]}]}}
]}"#;
    const WITCH_DRINK_POTION_TEXT: &str = r#"{"priority": 1, "speed_modifier": -0.25, "potions": [
  {"id": 19, "chance": 0.15, "filters": {"test": "is_underwater", "subject": "self", "value": true}},
  {"id": 12, "chance": 0.15, "filters": {"any_of": [{"test": "on_fire", "subject": "self", "value": true}, {"test": "on_hot_block", "subject": "self", "value": true}]}},
  {"id": 21, "chance": 0.05, "filters": {"test": "is_missing_health", "subject": "self", "value": true}},
  {"id": 14, "chance": 0.25, "filters": {"all_of": [{"test": "has_target", "subject": "self", "value": true}, {"test": "target_distance", "subject": "self", "operator": ">=", "value": 11}]}}
]}"#;
    const NOT_IN_WATER_TEXT: &str = r#"{"test": "in_water", "value": false}"#;
    const ON_GROUND_OR_SLEEPING_TEXT: &str = r#"{"any_of": [{"test": "on_ground", "value": true}, {"test": "is_sleeping", "value": true}]}"#;
    const ALL_DAMAGE_SOURCES_TEXT: &str = r#"["all"]"#;
    const CHEST_ARMOR_SLOT_TEXT: &str = r#"{"armor_slot": "slot.armor.chest"}"#;
    const PRIORITY_ONE_TEXT: &str = r#"{"priority": 1}"#;
    const STRIDER_SOUND_OFFSET_TEXT: &str = r#"{"value": 0.6}"#;
    const XZ_RANGE_TEXT: &str = "[-1, 1]";
    const Y_RANGE_2_TEXT: &str = "[0, 2]";
    const Y_RANGE_3_TEXT: &str = "[0, 3]";
    const TAKE_BLOCKS_TEXT: &str = r#"["minecraft:grass_block", "minecraft:dirt", "minecraft:coarse_dirt", "minecraft:podzol",
  "minecraft:sand", "minecraft:red_sand", "minecraft:gravel", "minecraft:crimson_roots", "minecraft:warped_roots",
  "minecraft:dandelion", "minecraft:poppy", "minecraft:brown_mushroom", "minecraft:red_mushroom",
  "minecraft:crimson_fungus", "minecraft:warped_fungus", "minecraft:tnt", "minecraft:cactus", "minecraft:clay",
  "minecraft:pumpkin", "minecraft:carved_pumpkin", "minecraft:melon_block", "minecraft:mycelium",
  "minecraft:warped_nylium", "minecraft:crimson_nylium", "minecraft:dirt_with_roots", "minecraft:cactus_flower"]"#;
    const REGULAR_SKIP_DATA_TEXT: &str =
        r#"{"distance_scale": 0.8, "height": 0.1, "jump_delay": 20, "animation_duration": 3}"#;
    const FAST_SKIP_DATA_TEXT: &str =
        r#"{"distance_scale": 1.75, "height": 0.15, "jump_delay": 1, "animation_duration": 1}"#;
    const CAPTAIN_LEADER_FILTER_TEXT: &str =
        r#"{"test": "has_component", "subject": "other", "value": "minecraft:is_illager_captain"}"#;
    const ENDERMAN_PROJECTILE_FILTER_TEXT: &str = r#"{"none_of": [{"test": "is_family", "subject": "other", "value": "splash_potion"}, {"test": "is_family", "subject": "other", "value": "lingering_potion"}, {"test": "is_family", "subject": "other", "value": "snowball"}, {"test": "is_family", "subject": "other", "value": "xp_bottle"}]}"#;

    #[test]
    fn each_fragment_is_the_text_it_replaced() {
        for (built, text) in [
            (damage_conditions(), DAMAGE_CONDITIONS_TEXT),
            (lava_condition(), LAVA_CONDITION_TEXT),
            (temperature_condition(), TEMPERATURE_CONDITION_TEXT),
            (rain_condition(), RAIN_CONDITION_TEXT),
            (iron_golem_target(), IRON_GOLEM_TARGET_TEXT),
            (lightning_sensor(), LIGHTNING_SENSOR_TEXT),
            (lightning_trigger(), LIGHTNING_TRIGGER_TEXT),
            (pufferfish(), PUFFERFISH_TEXT),
            (no_triggers(), NO_TRIGGERS_TEXT),
            (no_riders(), NO_RIDERS_TEXT),
            (panic_damage_sources(), PANIC_DAMAGE_SOURCES_TEXT),
            (not_persistent(), NOT_PERSISTENT_TEXT),
            (inactive(), INACTIVE_TEXT),
            (random_chance_50(), RANDOM_CHANCE_50_TEXT),
            (far_from_players(), FAR_FROM_PLAYERS_TEXT),
            (is_daytime(), IS_DAYTIME_TEXT),
            (fish_far_from_players(), FISH_FAR_FROM_PLAYERS_TEXT),
            (light_at_least_8(), LIGHT_AT_LEAST_8_TEXT),
            (surface_mob(), SURFACE_MOB_TEXT),
            (light_below_8(), LIGHT_BELOW_8_TEXT),
            (home(), HOME_TEXT),
            (boat(), BOAT_TEXT),
            (ocelot_spawn(), OCELOT_SPAWN_TEXT),
            (auto_step(), AUTO_STEP_TEXT),
            (witch_shooter(), WITCH_SHOOTER_TEXT),
            (witch_drink_potion(), WITCH_DRINK_POTION_TEXT),
            (not_in_water(), NOT_IN_WATER_TEXT),
            (on_ground_or_sleeping(), ON_GROUND_OR_SLEEPING_TEXT),
            (all_damage_sources(), ALL_DAMAGE_SOURCES_TEXT),
            (chest_armor_slot(), CHEST_ARMOR_SLOT_TEXT),
            (priority_one(), PRIORITY_ONE_TEXT),
            (strider_sound_offset(), STRIDER_SOUND_OFFSET_TEXT),
            (xz_range(), XZ_RANGE_TEXT),
            (y_range_2(), Y_RANGE_2_TEXT),
            (y_range_3(), Y_RANGE_3_TEXT),
            (take_blocks(), TAKE_BLOCKS_TEXT),
            (regular_skip_data(), REGULAR_SKIP_DATA_TEXT),
            (fast_skip_data(), FAST_SKIP_DATA_TEXT),
            (captain_leader_filter(), CAPTAIN_LEADER_FILTER_TEXT),
            (
                enderman_projectile_filter(),
                ENDERMAN_PROJECTILE_FILTER_TEXT,
            ),
        ] {
            assert_is(&built, text);
        }
    }

    #[test]
    fn the_fragments_of_a_value_are_the_texts_they_replaced() {
        assert_is(
            &snow_trail(),
            &format!(
                r#"{{"block_type": "snow_layer", "spawn_filter": {{"test": "is_temperature_value", "operator": "<", "value": {}}}}}"#,
                f64::from(0.81_f32)
            ),
        );
        assert_is(
            &slow_movement(),
            &format!(r#"{{"value": {}}}"#, f64::from(0.13_f32)),
        );
        for (damage, families) in [
            (6, &["player", "irongolem"][..]),
            (3, &["player", "irongolem"][..]),
            (4, &["player", "irongolem", "snowgolem"][..]),
        ] {
            let filters: Vec<String> = families
                .iter()
                .map(|family| {
                    format!(r#"{{"test": "is_family", "subject": "other", "value": "{family}"}}"#)
                })
                .collect();
            assert_is(
                &slime_area_attack(damage, families),
                &format!(
                    r#"{{"damage_range": 0.15, "damage_per_tick": {damage}, "damage_cooldown": 0.0, "cause": "entity_attack", "entity_filter": {{"any_of": [{}]}}}}"#,
                    filters.join(", ")
                ),
            );
        }
        for (h, v, cap, water) in [
            (0.8, 1.6, 0.8, false),
            (0.52, 0.39, 0.8, false),
            (0.6, -0.6, -0.4, true),
            (0.4, 0.12, 0.12, false),
        ] {
            assert_is(
                &knockback_rules(h, v, cap, water),
                &format!(
                    r#"{{"presets": [{{"horizontal_power": {h}, "vertical_power": {v}, "vertical_velocity_cap": {cap}, "check_if_target_is_immersed_in_water": {water}}}]}}"#
                ),
            );
        }
        let lists: [Vec<(Value, &str)>; 4] = [
            vec![
                (not_persistent(), NOT_PERSISTENT_TEXT),
                (fish_far_from_players(), FISH_FAR_FROM_PLAYERS_TEXT),
            ],
            vec![
                (not_persistent(), NOT_PERSISTENT_TEXT),
                (is_daytime(), IS_DAYTIME_TEXT),
                (inactive(), INACTIVE_TEXT),
                (random_chance_50(), RANDOM_CHANCE_50_TEXT),
                (far_from_players(), FAR_FROM_PLAYERS_TEXT),
            ],
            vec![
                (not_persistent(), NOT_PERSISTENT_TEXT),
                (light_at_least_8(), LIGHT_AT_LEAST_8_TEXT),
                (is_daytime(), IS_DAYTIME_TEXT),
                (inactive(), INACTIVE_TEXT),
                (random_chance_50(), RANDOM_CHANCE_50_TEXT),
                (far_from_players(), FAR_FROM_PLAYERS_TEXT),
                (surface_mob(), SURFACE_MOB_TEXT),
            ],
            vec![
                (not_persistent(), NOT_PERSISTENT_TEXT),
                (light_below_8(), LIGHT_BELOW_8_TEXT),
                (inactive(), INACTIVE_TEXT),
                (random_chance_50(), RANDOM_CHANCE_50_TEXT),
                (far_from_players(), FAR_FROM_PLAYERS_TEXT),
            ],
        ];
        for list in lists {
            let texts: Vec<&str> = list.iter().map(|(_, text)| *text).collect();
            let all_of = format!("[{}]", texts.join(", "));
            let built = despawn(list.into_iter().map(|(value, _)| value).collect());
            assert_is(
                &built,
                &format!(r#"{{"filters": [{{"all_of": {all_of}}}]}}"#),
            );
        }
        assert_is(&empty_object(), "{}");
    }
}
