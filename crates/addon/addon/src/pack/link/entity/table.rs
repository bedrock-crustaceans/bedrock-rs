//! Which members of an entity's components hold a reference: one table per kind of target,
//! each row a component key and a member pattern (`walk::members`: keys joined by `.`, `[]`
//! after a key for "each element, or the member itself when it is no array", `*` for every
//! member of an object).
//!
//! The rows are the members of the game's entity components whose value names a definition,
//! with the nesting the game's own entity files show for them; a member the game also reads
//! under a legacy camel-case spelling has a row for that spelling where the component is
//! still read at run time.

/// A table row: the component's key and the member pattern.
pub(super) type Row = (&'static str, &'static str);

/// Members that hold a list of block names: plain strings, which the game reads as
/// names (no `{name, states}` and no `{tags}` can stand there; the typed read refuses one).
#[cfg(feature = "block")]
pub(super) const BLOCK_NAMES: &[Row] = &[
    ("minecraft:behavior.raid_garden", "blocks[]"),
    ("minecraft:home", "home_block_list[]"),
];

/// Members that hold a block descriptor (a name, `{name, states}` or `{tags}`), one or a
/// list.
#[cfg(feature = "block")]
pub(super) const BLOCKS: &[Row] = &[
    ("minecraft:behavior.lay_egg", "egg_type"),
    ("minecraft:behavior.lay_egg", "target_blocks[]"),
    (
        "minecraft:behavior.place_block",
        "placeable_carried_blocks[]",
    ),
    (
        "minecraft:behavior.place_block",
        "randomly_placeable_blocks[].block",
    ),
    (
        "minecraft:behavior.random_search_and_dig",
        "target_blocks[]",
    ),
    ("minecraft:behavior.take_block", "blocks[]"),
    (
        "minecraft:behavior.transport_items",
        "destination_container_types[]",
    ),
    (
        "minecraft:behavior.transport_items",
        "source_container_types[]",
    ),
    ("minecraft:block_movement_slowdown_immunity", "blocks[]"),
    ("minecraft:block_sensor", "on_break[].block_list[]"),
    ("minecraft:break_blocks", "breakable_blocks[]"),
    ("minecraft:breathable", "breatheBlocks[]"),
    ("minecraft:breathable", "breathe_blocks[]"),
    ("minecraft:breathable", "nonBreatheBlocks[]"),
    ("minecraft:breathable", "non_breathe_blocks[]"),
    ("minecraft:breedable", "environment_requirements[].blocks[]"),
    ("minecraft:buoyant", "liquid_blocks[]"),
    ("minecraft:inside_block_notifier", "block_list[].block"),
    (
        "minecraft:memory_sensor.find_nearest_block",
        "entries[].blocks[]",
    ),
    (
        "minecraft:memory_sensor.find_nearest_block",
        "sensor_entries[].blocks[]",
    ),
    (
        "minecraft:memory_sensor.find_nearest_block_from",
        "sensor_entries[].blocks[]",
    ),
    ("minecraft:navigation.climb", "blocks_to_avoid[]"),
    ("minecraft:navigation.float", "blocks_to_avoid[]"),
    ("minecraft:navigation.fly", "blocks_to_avoid[]"),
    ("minecraft:navigation.generic", "blocks_to_avoid[]"),
    ("minecraft:navigation.hover", "blocks_to_avoid[]"),
    ("minecraft:navigation.swim", "blocks_to_avoid[]"),
    ("minecraft:navigation.walk", "blocks_to_avoid[]"),
    (
        "minecraft:preferred_path",
        "preferred_path_blocks[].blocks[]",
    ),
    ("minecraft:trail", "block_type"),
    ("minecraft:transformation", "delay.block_types[]"),
];

/// Goal members that name blocks and that the game reads as item descriptors (every
/// block has an item of its name): a name that is a block or an item resolves.
#[cfg(feature = "block")]
pub(super) const BLOCKS_AS_ITEMS: &[Row] = &[
    ("minecraft:behavior.avoid_block", "target_blocks[]"),
    ("minecraft:behavior.jump_to_block", "forbidden_blocks[]"),
    ("minecraft:behavior.jump_to_block", "preferred_blocks[]"),
    ("minecraft:behavior.move_to_block", "target_blocks[]"),
];

/// Members that hold a loot table path.
#[cfg(feature = "loot")]
pub(super) const LOOT_TABLES: &[Row] = &[
    ("minecraft:barter", "barter_table"),
    ("minecraft:behavior.drop_item_for", "loot_table"),
    ("minecraft:behavior.eat_mob", "loot_table"),
    ("minecraft:behavior.random_search_and_dig", "item_table"),
    ("minecraft:behavior.sneeze", "loot_table"),
    ("minecraft:equipment", "table"),
    ("minecraft:interact", "interactions[].add_items.table"),
    ("minecraft:interact", "interactions[].spawn_items.table"),
    ("minecraft:loot", "table"),
];

/// Members that hold a trade table path.
#[cfg(feature = "trade")]
pub(super) const TRADE_TABLES: &[Row] = &[
    ("minecraft:economy_trade_table", "table"),
    ("minecraft:trade_table", "table"),
];

/// Members that hold an entity identifier, with or without an `<event>`. The member of
/// [`OFFSPRING`] is an object of pairs, a mate's identifier → the child's: both are checked.
pub(super) const ENTITIES: &[Row] = &[
    ("minecraft:addrider", "entity_type"),
    ("minecraft:addrider", "riders[].entity_type"),
    ("minecraft:behavior.fire_at_target", "projectile_def"),
    ("minecraft:behavior.follow_mob", "preferred_actor_type"),
    ("minecraft:behavior.mingle", "mingle_partner_type"),
    (
        "minecraft:behavior.summon_entity",
        "summon_choices[].sequence[].entity_type",
    ),
    BREEDS_WITH,
    ("minecraft:interact", "interactions[].spawn_entities[]"),
    OFFSPRING,
    ("minecraft:projectile", "ignored_entities[]"),
    ("minecraft:reflect_projectiles", "reflected_projectiles[]"),
    ("minecraft:shooter", "def"),
    ("minecraft:shooter", "projectiles[].def"),
    ("minecraft:spawn_entity", "entities[].spawn_entity"),
    ("minecraft:spawn_on_death", "entity_to_spawn"),
    ("minecraft:transformation", "into"),
];

/// The row of [`ENTITIES`] whose member maps a mate's identifier to the child's.
pub(super) const OFFSPRING: Row = ("minecraft:offspring", "offspring_pairs");

/// The row of [`ENTITIES`] whose member maps each mate's identifier to its breed event: the
/// 1.21.130+ shape (an upgrade step makes it of an older file's list, whose babies go to
/// [`OFFSPRING`]). The keys are checked.
pub(super) const BREEDS_WITH: Row = ("minecraft:breedable", "breeds_with");

/// Members that hold an item: a name (with the legacy `:aux`), or a descriptor object
/// (`{item}`, `{name}`, `{item_tag}`, `{tags}`), one or a list. An entry object of a
/// component the game reads at run time (`{"item": …, "result_item": …}`) reads as the
/// descriptor of its `item`; its other item members have rows of their own.
#[cfg(feature = "item")]
pub(super) const ITEMS: &[Row] = &[
    ("minecraft:ageable", "drop_items[]"),
    ("minecraft:ageable", "feedItems[]"),
    ("minecraft:ageable", "feedItems[].result_item"),
    ("minecraft:ageable", "feed_items[]"),
    ("minecraft:ageable", "feed_items[].result_item"),
    ("minecraft:ageable", "pause_growth_items[]"),
    ("minecraft:ageable", "reset_growth_items[]"),
    ("minecraft:behavior.beg", "items[]"),
    ("minecraft:behavior.charge_held_item", "items[]"),
    ("minecraft:behavior.float_tempt", "items[]"),
    ("minecraft:behavior.pickup_items", "excluded_items[]"),
    ("minecraft:behavior.snacking", "items[]"),
    ("minecraft:behavior.tempt", "items[]"),
    ("minecraft:behavior.transport_items", "allowed_items[]"),
    ("minecraft:behavior.transport_items", "disallowed_items[]"),
    ("minecraft:boostable", "boost_items[]"),
    ("minecraft:boostable", "boost_items[].replaceItem"),
    ("minecraft:boostable", "boost_items[].replace_item"),
    ("minecraft:breedable", "breedItems[]"),
    ("minecraft:breedable", "breedItems[].result_item"),
    ("minecraft:breedable", "breed_items[]"),
    ("minecraft:breedable", "breed_items[].result_item"),
    ("minecraft:bribeable", "bribe_items[]"),
    ("minecraft:equip_item", "excluded_items[]"),
    ("minecraft:equippable", "slots[].accepted_items[]"),
    ("minecraft:equippable", "slots[].item"),
    ("minecraft:giveable", "triggers[].items[]"),
    ("minecraft:healable", "items[]"),
    ("minecraft:healable", "items[].result_item"),
    ("minecraft:interact", "interactions[].transform_to_item"),
    ("minecraft:item_controllable", "control_items[]"),
    ("minecraft:shareables", "items[]"),
    ("minecraft:shareables", "items[].craft_into"),
    ("minecraft:spawn_entity", "entities[].spawn_item"),
    ("minecraft:tameable", "tameItems[]"),
    ("minecraft:tameable", "tameItems[].result_item"),
    ("minecraft:tameable", "tame_items[]"),
    ("minecraft:tameable", "tame_items[].result_item"),
    ("minecraft:tamemount", "auto_reject_items[]"),
    ("minecraft:tamemount", "feed_items[]"),
    ("minecraft:trusting", "trust_items[]"),
];

/// Members that hold the name of a level sound event as a plain string: those of the
/// components the game reads at run time and of the typed components that are no goal. A
/// typed goal's sound is a `LevelSoundEvent` of its typed parse, checked at load.
pub(super) const SOUNDS: &[Row] = &[
    ("minecraft:ambient_sound_interval", "event_name"),
    (
        "minecraft:ambient_sound_interval",
        "event_names[].event_name",
    ),
    ("minecraft:anger_level", "on_increase_sounds[].sound"),
    ("minecraft:angry", "angrySound"),
    ("minecraft:angry", "angry_sound"),
    ("minecraft:behavior.float_tempt", "tempt_sound"),
    ("minecraft:behavior.panic", "panic_sound"),
    (
        "minecraft:behavior.send_event",
        "event_choices[].sequence[].sound_event",
    ),
    (
        "minecraft:behavior.send_event",
        "event_choices[].start_sound_event",
    ),
    ("minecraft:behavior.tempt", "tempt_sound"),
    ("minecraft:celebrate_hunt", "celebrate_sound"),
    (
        "minecraft:damage_sensor",
        "triggers[].on_damage_sound_event",
    ),
    ("minecraft:explode", "sound_effect"),
    ("minecraft:heartbeat", "sound_event"),
    ("minecraft:interact", "interactions[].play_sounds[]"),
    ("minecraft:projectile", "hit_ground_sound"),
    ("minecraft:projectile", "hit_sound"),
    ("minecraft:projectile", "shoot_sound"),
    ("minecraft:reflect_projectiles", "reflection_sound"),
    ("minecraft:shooter", "sound"),
    ("minecraft:spawn_entity", "entities[].spawn_sound"),
    ("minecraft:transformation", "begin_transform_sound[]"),
    ("minecraft:transformation", "transformation_sound[]"),
];

/// Members that hold a particle: a particle type of the game or a particle effect.
#[cfg(feature = "particle")]
pub(super) const PARTICLES: &[Row] = &[
    ("minecraft:explode", "particle_effect"),
    (
        "minecraft:interact",
        "interactions[].particle_on_start.particle_type",
    ),
    (
        "minecraft:projectile",
        "on_hit.particle_on_hit.particle_type",
    ),
    ("minecraft:projectile", "on_hit.spawn_aoe_cloud.particle"),
    ("minecraft:projectile", "particle"),
];

/// Members that hold the name of a point of interest (a type or a tag).
#[cfg(feature = "poi")]
pub(super) const POIS: &[Row] = &[
    (
        "minecraft:memory_sensor.find_nearest_poi",
        "entries[].poi_name",
    ),
    (
        "minecraft:memory_sensor.find_nearest_poi",
        "sensor_entries[].poi_name",
    ),
];
