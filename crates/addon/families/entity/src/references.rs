//! The references of an entity document to itself (the load half): a trigger on the entity itself
//! names one of its events, a `set_property` key one of its properties, an entry of `scripts.animate` one of its animations, and the filter
//! of a trigger in a component the game reads at run time uses registered tests.
//!
//! A reference of the first three kinds that names nothing is an information and is kept:
//! the game's own behaviour packs fire events their definitions lack (`minecraft:on_leash`,
//! `minecraft:on_open`, `minecraft:on_anger`, `minecraft:ageable_grow_up`, …: 285 triggers
//! in the game's own entity files) and the game runs those entities, so it tolerates the miss and says
//! nothing. An unknown filter test is the warning the game logs.

use std::collections::HashSet;
use std::sync::LazyLock;

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Phase, Severity};
use bedrock_addon_core::document::{ComponentMap, ComponentRef};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::shared::Filter;
use bedrock_addon_core::versioning::ValidateCx;
use bedrock_addon_shared_shapes::event;

use super::component::EntityComponent;
use super::{AnimateEntry, Entity};

/// The members of the components that are a trigger on the entity itself: the component's key
/// and the member (keys joined by `.`; `[]` after a key for "each element, or the member
/// itself when it is no array"; `*` for "each member that is an object", the values of a
/// map; the empty pattern is the component). A member holds the trigger as an object
/// (`{event, target, filters}`) or as a bare event name.
///
/// `breedable.breeds_with.*` is the newest format's map of one trigger per mate, which an
/// older file's list of `{mate_type, baby_type, breed_event}` is converted to before this
/// check runs; the game's files give it `target: baby` (the newborn's event, not judged),
/// `target: self` or no target.
///
/// Not here, because their event is another entity's whatever the trigger's target says:
/// `projectile.on_hit.spawn_chance.on_spawn[]` (the spawned entity),
/// `projectile.on_hit.definition_event.event_trigger` (the entities the hit affects),
/// `behavior.send_event.event_choices[].sequence[]` (the entities the spell reaches),
/// `spawn_egg_interaction.on_spawn` (the entity the egg spawns), and the plain event names
/// `addrider.spawn_event`, `addrider.riders[].spawn_event`,
/// `spawn_entity.entities[].spawn_event`, `behavior.summon_entity.…summon_event` (the spawned
/// or summoned entity).
// The nesting is the one the game's own entity files show. The list is the component rows':
// each `EventTrigger` member of a typed component that is not `schema::foreign_event`, and
// each row's `triggers = [..]` (the dynamic keys, and members that hold a bare event name).
static TRIGGERS: LazyLock<Vec<(&'static str, String)>> = LazyLock::new(EntityComponent::triggers);

/// The members of the older filter form: an object without a `test` whose members the game
/// maps to tests one by one. Sorted bytewise.
// The game maps these legacy entity-filter member forms to tests when it reads a filter group.
const OLDER_FILTER_MEMBERS: [&str; 79] = [
    "altitude_greater",
    "altitude_less",
    "brightness_greater",
    "brightness_less",
    "has_ability",
    "has_component",
    "has_damage",
    "has_equipment",
    "in_caravan",
    "is_altitude",
    "is_brightness",
    "is_daytime",
    "is_difficulty",
    "is_family",
    "is_game_rule",
    "is_game_rule_false",
    "is_game_rule_true",
    "is_in_caravan",
    "is_not_in_caravan",
    "is_not_world_difficulty",
    "is_snow_covered",
    "is_target",
    "is_underground",
    "is_variant",
    "is_world_difficulty",
    "other_is_in_caravan",
    "other_is_not_in_caravan",
    "other_is_owner",
    "other_is_target",
    "other_with_armor",
    "other_with_color",
    "other_with_components",
    "other_with_equipment",
    "other_with_families",
    "other_with_item",
    "other_with_variant",
    "other_without_armor",
    "other_without_color",
    "other_without_components",
    "other_without_equipment",
    "other_without_families",
    "other_without_item",
    "other_without_variant",
    "player_with_instabuild",
    "target_with_armor",
    "target_with_color",
    "target_with_components",
    "target_with_equipment",
    "target_with_families",
    "target_with_item",
    "target_with_variant",
    "target_without_armor",
    "target_without_color",
    "target_without_components",
    "target_without_equipment",
    "target_without_families",
    "target_without_item",
    "target_without_variant",
    "underground",
    "with_armor",
    "with_color",
    "with_components",
    "with_damage_cause",
    "with_damage_fatal",
    "with_environment_any",
    "with_environment_every",
    "with_equipment",
    "with_families",
    "with_item",
    "with_variant",
    "without_armor",
    "without_color",
    "without_components",
    "without_environment_any",
    "without_environment_every",
    "without_equipment",
    "without_families",
    "without_item",
    "without_variant",
];

/// The keys of a filter group, which hold filters.
const FILTER_GROUP_KEYS: [&str; 8] = [
    "all_of", "all", "AND", "any_of", "any", "OR", "none_of", "NOT",
];

/// Vanilla's text for a member of the older filter form that it maps to no test.
// The game logs this text for a legacy filter member it cannot map.
const INVALID_FILTER_MEMBER: &str = "Invalid EntityFilterGroup member found: {}";

/// The key a suffixed generic goal (`minecraft:memory_behavior.move_to_position.<name>`) has
/// in [`TRIGGERS`].
const GENERIC_GOAL: &str = "minecraft:memory_behavior.move_to_position";

/// The rows of [`TRIGGERS`] for the component `key`.
fn rows(key: &str) -> &'static [(&'static str, String)] {
    let key = if key.starts_with(GENERIC_GOAL) && key[GENERIC_GOAL.len()..].starts_with('.') {
        GENERIC_GOAL
    } else {
        key
    };
    let from = TRIGGERS.partition_point(|(component, _)| *component < key);
    let len = TRIGGERS[from..].partition_point(|(component, _)| *component == key);
    &TRIGGERS[from..from + len]
}

/// Every self-reference of `entity`.
pub(crate) fn check(entity: &Entity, cx: &mut ValidateCx<'_>) {
    let events: HashSet<&str> = entity.events.keys().map(String::as_str).collect();
    animate(entity, cx);
    let events_at = cx.at.child("events");
    for (name, event) in &entity.events {
        nodes(&event.node, events_at.child(name), entity, &events, cx);
    }
    let upgrades_at = cx.at.child("upgrades");
    for (name, node) in &entity.upgrades {
        nodes(node, upgrades_at.child(name), entity, &events, cx);
    }
    components(&entity.components, &cx.at.child("components"), &events, cx);
    let groups_at = cx.at.child("component_groups");
    for (name, group) in &entity.component_groups {
        components(&group.components, &groups_at.child(name), &events, cx);
    }
}

/// A reference of the entity to itself that names nothing, at `at`.
fn unresolved(cx: &mut ValidateCx<'_>, at: JsonPath) {
    cx.diagnostics.push(
        Diagnostic::info(DiagnosticCode::UnresolvedReference { text: None })
            .at(at)
            .phase(Phase::Validate),
    );
}

/// E8: each entry of `description.scripts.animate` names a key of `description.animations`;
/// the game looks a plain name up in lower case and the key of a conditional entry as
/// written.
fn animate(entity: &Entity, cx: &mut ValidateCx<'_>) {
    let Some(scripts) = &entity.description.scripts else {
        return;
    };
    let animations = &entity.description.animations;
    let at = cx.at.child("description").child("scripts").child("animate");
    for (index, entry) in scripts.animate.iter().enumerate() {
        let entry_at = at.index(scripts.place(index));
        match entry {
            AnimateEntry::Always(name) => {
                if !animations.contains_key(&name.to_lowercase()) {
                    unresolved(cx, entry_at);
                }
            }
            AnimateEntry::Conditional(conditions) => {
                for name in conditions.keys() {
                    if !animations.contains_key(name) {
                        unresolved(cx, entry_at.child(name));
                    }
                }
            }
        }
    }
}

/// E5 and E6 in an event node and the nodes nested in it: the node's `trigger` and the keys
/// of its `set_property`.
fn nodes(
    root: &Value,
    at: JsonPath,
    entity: &Entity,
    events: &HashSet<&str>,
    cx: &mut ValidateCx<'_>,
) {
    let mut pending = vec![(at, root)];
    while let Some((at, node)) = pending.pop() {
        let Some(object) = node.as_object() else {
            continue;
        };
        if let Some(trigger) = object.get("trigger") {
            own_event(trigger, at.child("trigger"), events, cx);
        }
        if let Some(properties) = object.get("set_property").and_then(Value::as_object) {
            let set_at = at.child("set_property");
            for (name, _) in properties {
                if !entity.description.properties.contains_key(name.as_str()) {
                    unresolved(cx, set_at.child(name.as_str()));
                }
            }
        }
        let mut nested = Vec::new();
        for (key, list) in event::children(node).lists {
            let list_at = at.child(key);
            for (index, node) in list.iter().enumerate() {
                let node_at = list_at.index(index);
                nested.push((node_at, node));
            }
        }
        pending.extend(nested.into_iter().rev());
    }
}

/// E5 on one trigger at `at`: a bare event name, or an object whose target is the entity
/// itself (`self`, or no target), must name one of the entity's events. A trigger on another
/// target fires an event of whatever entity that is.
fn own_event(trigger: &Value, at: JsonPath, events: &HashSet<&str>, cx: &mut ValidateCx<'_>) {
    if let Some(name) = trigger.as_string() {
        if !name.as_str().is_empty() && !events.contains(name.as_str()) {
            unresolved(cx, at);
        }
        return;
    }
    let Some(object) = trigger.as_object() else {
        return;
    };
    let Some(event) = object.get("event").and_then(Value::as_string) else {
        return;
    };
    let on_self = match object.get("target") {
        None => true,
        Some(target) => target
            .as_string()
            .is_some_and(|target| target.as_str() == "self"),
    };
    if on_self && !event.as_str().is_empty() && !events.contains(event.as_str()) {
        unresolved(cx, at.child("event"));
    }
}

/// E5 and E9 in the components of one section: the members [`TRIGGERS`] names. In a
/// component the game reads at run time the filter of such a trigger is checked here too,
/// whether or not the trigger names an event (a typed component checks its own).
fn components(
    map: &ComponentMap<EntityComponent>,
    section: &JsonPath,
    events: &HashSet<&str>,
    cx: &mut ValidateCx<'_>,
) {
    for entry in map.iter() {
        let (key, typed, json) = match entry {
            ComponentRef::Typed { name, value, .. } => (name, Some(value), None),
            ComponentRef::Dynamic { name, json } => (name, None, Some(json)),
            _ => continue,
        };
        let rows = rows(key);
        if rows.is_empty() {
            continue;
        }
        // A typed component is read as it is written; a value the format cannot express was
        // told where it was read.
        let written;
        let (value, dynamic) = match (typed, json) {
            (Some(component), _) => match super::write::component(component) {
                Ok(json) => {
                    written = json;
                    (&written, false)
                }
                Err(_) => continue,
            },
            (None, Some(json)) => (json, true),
            (None, None) => continue,
        };
        let at = section.child(key);
        for (_, pattern) in rows {
            members(value, pattern, &at, &mut |trigger_at, trigger| {
                if dynamic
                    && let Some(filters) = trigger.as_object().and_then(|o| o.get("filters"))
                    && let Ok(filter) = Filter::try_from(filters.clone())
                {
                    older_members(filters, &trigger_at.child("filters"), cx);
                    let found = filter.check(&trigger_at.child("filters"), Severity::Warning);
                    // Of the shared check the unknown test names alone: the game's own
                    // files write members of the older filter form here, which it maps to
                    // tests and which `older_members` holds to the game's list. Told in the
                    // phase a typed component's filter is told in.
                    let unknown_tests = found.into_iter().filter(|finding| {
                        // By legacy name: the variant and the definitions that were it.
                        matches!(finding.code.name(), "UnresolvedReference")
                    });
                    cx.diagnostics.extend(unknown_tests.map(|mut finding| {
                        finding.phase = Phase::Parse;
                        finding
                    }));
                }
                own_event(trigger, trigger_at, events, cx);
            });
        }
    }
}

/// E9, the older filter form: in the filter `filters` at `at`, every object without a `test`
/// is a group whose members are group keys (which hold filters) or members the game maps to
/// tests ([`OLDER_FILTER_MEMBERS`]). Any other member is the game's "Invalid
/// `EntityFilterGroup` member found", a warning at the member. The members of a test object
/// are the shared check's.
fn older_members(filters: &Value, at: &JsonPath, cx: &mut ValidateCx<'_>) {
    use bedrock_addon_core::diagnostic::VanillaText;
    let mut pending = vec![(at.clone(), filters)];
    while let Some((at, filter)) = pending.pop() {
        if let Some(list) = filter.as_array() {
            let mut elements = Vec::with_capacity(list.len());
            for (index, element) in list.iter().enumerate() {
                let element_at = at.index(index);
                elements.push((element_at, element));
            }
            pending.extend(elements.into_iter().rev());
            continue;
        }
        let Some(object) = filter.as_object() else {
            continue;
        };
        if object.get("test").is_some() {
            continue;
        }
        let mut groups = Vec::new();
        for (key, member) in object {
            let key = key.as_str();
            if FILTER_GROUP_KEYS.contains(&key) {
                groups.push((at.child(key), member));
            } else if OLDER_FILTER_MEMBERS.binary_search(&key).is_err() {
                // The game logs this at the error level and the entity still loads: a kept
                // error. Whether the trigger itself survives is not observable, so the
                // document keeps it.
                cx.diagnostics.push_kept_error(
                    Diagnostic::error(DiagnosticCode::UnrecognizedField {
                        text: Some(VanillaText::Formatted {
                            template: INVALID_FILTER_MEMBER,
                            args: vec![key.to_owned()],
                        }),
                    })
                    .at(at.child(key)),
                );
            }
        }
        pending.extend(groups.into_iter().rev());
    }
}

/// Calls `found` with every value `pattern` names below `value` (which is at `at`), with its
/// path.
fn members(value: &Value, pattern: &str, at: &JsonPath, found: &mut dyn FnMut(JsonPath, &Value)) {
    if pattern.is_empty() {
        found(at.clone(), value);
        return;
    }
    let (step, rest) = pattern.split_once('.').unwrap_or((pattern, ""));
    if step == "*" {
        // Each member that is an object: the values of a map.
        for (key, member) in value.as_object().into_iter().flatten() {
            if member.as_object().is_some() {
                members(member, rest, &at.child(key.as_str()), found);
            }
        }
        return;
    }
    let (key, list) = match step.strip_suffix("[]") {
        Some(key) => (key, true),
        None => (step, false),
    };
    let Some(member) = value.as_object().and_then(|object| object.get(key)) else {
        return;
    };
    let member_at = at.child(key);
    match member.as_array() {
        Some(elements) if list => {
            for (index, element) in elements.iter().enumerate() {
                let element_at = member_at.index(index);
                members(element, rest, &element_at, found);
            }
        }
        _ => members(member, rest, &member_at, found),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table is sorted by key (a component's rows are found by bisection) and unique, and
    /// every key is one the component table has.
    #[test]
    fn the_trigger_table_is_sorted_unique_and_of_known_components() {
        assert!(TRIGGERS.windows(2).all(|pair| pair[0] < pair[1]));
        // The table lists 116 rows.
        assert_eq!(TRIGGERS.len(), 116);
        assert!(
            OLDER_FILTER_MEMBERS
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        );
        for (key, _) in TRIGGERS.iter() {
            assert!(EntityComponent::TABLE.get(key).is_some(), "{key}");
        }
        assert_eq!(rows("minecraft:peek").len(), 3);
        assert_eq!(
            rows("minecraft:memory_behavior.move_to_position.gold").len(),
            1
        );
        assert_eq!(rows("minecraft:memory_behavior.move_to_positions").len(), 0);
        assert_eq!(rows("minecraft:health").len(), 0);
    }

    /// Every `(component, trigger member)` pair of the table.
    const PINNED_TRIGGERS: [(&str, &str); 116] = [
        ("minecraft:ageable", "grow_up"),
        ("minecraft:ageable", "on_pause_growth"),
        ("minecraft:ageable", "on_reset_growth"),
        ("minecraft:angry", "calm_event"),
        (
            "minecraft:attack_cooldown",
            "attack_cooldown_complete_event",
        ),
        ("minecraft:balloonable", "on_balloon"),
        ("minecraft:balloonable", "on_unballoon"),
        ("minecraft:behavior.admire_item", "on_admire_item_start"),
        ("minecraft:behavior.admire_item", "on_admire_item_stop"),
        ("minecraft:behavior.avoid_block", "on_escape[]"),
        ("minecraft:behavior.avoid_mob_type", "on_escape_event"),
        ("minecraft:behavior.celebrate", "on_celebration_end_event"),
        (
            "minecraft:behavior.celebrate_survive",
            "on_celebration_end_event",
        ),
        (
            "minecraft:behavior.defend_trusted_target",
            "on_defend_start",
        ),
        ("minecraft:behavior.delayed_attack", "on_attack"),
        ("minecraft:behavior.delayed_attack", "on_kill"),
        ("minecraft:behavior.dig", "on_start"),
        ("minecraft:behavior.drop_item_for", "on_drop_attempt"),
        ("minecraft:behavior.eat_block", "on_eat"),
        ("minecraft:behavior.emerge", "on_done"),
        ("minecraft:behavior.float_tempt", "on_tempt_end"),
        ("minecraft:behavior.float_tempt", "on_tempt_start"),
        (
            "minecraft:behavior.go_and_give_items_to_noteblock",
            "on_item_throw[]",
        ),
        (
            "minecraft:behavior.go_and_give_items_to_owner",
            "on_item_throw[]",
        ),
        ("minecraft:behavior.go_home", "on_failed[]"),
        ("minecraft:behavior.go_home", "on_home[]"),
        ("minecraft:behavior.hold_ground", "within_radius_event"),
        ("minecraft:behavior.knockback_roar", "on_roar_end"),
        ("minecraft:behavior.lay_egg", "on_lay"),
        ("minecraft:behavior.melee_attack", "on_attack"),
        ("minecraft:behavior.melee_attack", "on_kill"),
        ("minecraft:behavior.melee_box_attack", "on_attack"),
        ("minecraft:behavior.melee_box_attack", "on_kill"),
        ("minecraft:behavior.move_to_block", "on_reach[]"),
        ("minecraft:behavior.move_to_block", "on_stay_completed[]"),
        ("minecraft:behavior.pickup_items", "on_pickup_item_end"),
        ("minecraft:behavior.pickup_items", "on_pickup_item_start"),
        ("minecraft:behavior.place_block", "on_place"),
        ("minecraft:behavior.ram_attack", "on_start[]"),
        (
            "minecraft:behavior.random_search_and_dig",
            "on_digging_start",
        ),
        (
            "minecraft:behavior.random_search_and_dig",
            "on_fail_during_digging",
        ),
        (
            "minecraft:behavior.random_search_and_dig",
            "on_fail_during_searching",
        ),
        ("minecraft:behavior.random_search_and_dig", "on_item_found"),
        (
            "minecraft:behavior.random_search_and_dig",
            "on_searching_start",
        ),
        ("minecraft:behavior.random_search_and_dig", "on_success"),
        ("minecraft:behavior.stomp_attack", "on_attack"),
        ("minecraft:behavior.stomp_attack", "on_kill"),
        ("minecraft:behavior.take_block", "on_take"),
        ("minecraft:behavior.take_flower", "on_take_flower"),
        ("minecraft:behavior.tempt", "on_tempt_end"),
        ("minecraft:behavior.tempt", "on_tempt_start"),
        ("minecraft:behavior.timer_flag_1", "on_end"),
        ("minecraft:behavior.timer_flag_1", "on_start"),
        ("minecraft:behavior.timer_flag_2", "on_end"),
        ("minecraft:behavior.timer_flag_2", "on_start"),
        ("minecraft:behavior.timer_flag_3", "on_end"),
        ("minecraft:behavior.timer_flag_3", "on_start"),
        ("minecraft:behavior.use_kinetic_weapon", "on_attack"),
        ("minecraft:behavior.use_kinetic_weapon", "on_kill"),
        ("minecraft:behavior.work", "on_arrival"),
        ("minecraft:behavior.work_composter", "on_arrival"),
        ("minecraft:block_sensor", "on_break[].on_block_broken"),
        ("minecraft:breedable", "breeds_with.*"),
        ("minecraft:damage_sensor", "triggers[].on_damage"),
        ("minecraft:drying_out_timer", "dried_out_event"),
        (
            "minecraft:drying_out_timer",
            "recover_after_dried_out_event",
        ),
        ("minecraft:drying_out_timer", "stopped_drying_out_event"),
        ("minecraft:entity_sensor", "event"),
        ("minecraft:entity_sensor", "subsensors[].event"),
        ("minecraft:environment_sensor", "triggers[]"),
        ("minecraft:equippable", "slots[].on_equip"),
        ("minecraft:equippable", "slots[].on_unequip"),
        (
            "minecraft:genetics",
            "genes[].genetic_variants[].birth_event",
        ),
        ("minecraft:giveable", "triggers[].on_give"),
        (
            "minecraft:inside_block_notifier",
            "block_list[].entered_block_event",
        ),
        (
            "minecraft:inside_block_notifier",
            "block_list[].exited_block_event",
        ),
        ("minecraft:interact", "interactions[].on_interact"),
        ("minecraft:leashable", "on_leash"),
        ("minecraft:leashable", "on_unleash"),
        ("minecraft:looked_at", "looked_at_event"),
        ("minecraft:looked_at", "not_looked_at_event"),
        ("minecraft:memory_behavior.move_to_position", "on_reach[]"),
        ("minecraft:nameable", "default_trigger"),
        ("minecraft:nameable", "name_actions[].on_named"),
        ("minecraft:on_death", ""),
        ("minecraft:on_equipment_changed", "slots[].on_equip"),
        ("minecraft:on_equipment_changed", "slots[].on_unequip"),
        ("minecraft:on_friendly_anger", ""),
        ("minecraft:on_hurt", ""),
        ("minecraft:on_hurt_by_player", ""),
        ("minecraft:on_ignite", ""),
        ("minecraft:on_start_landing", ""),
        ("minecraft:on_start_takeoff", ""),
        ("minecraft:on_target_acquired", ""),
        ("minecraft:on_target_escape", ""),
        ("minecraft:on_wake_with_owner", ""),
        ("minecraft:peek", "on_close"),
        ("minecraft:peek", "on_open"),
        ("minecraft:peek", "on_target_open"),
        ("minecraft:raid_trigger", "triggered_event"),
        ("minecraft:rail_sensor", "on_activate"),
        ("minecraft:rail_sensor", "on_deactivate"),
        ("minecraft:ravager_blocked", "reaction_choices[].value"),
        ("minecraft:rideable", "on_rider_enter_event"),
        ("minecraft:rideable", "on_rider_exit_event"),
        ("minecraft:scheduler", "scheduled_events[]"),
        ("minecraft:sittable", "sit_event"),
        ("minecraft:sittable", "stand_event"),
        ("minecraft:spawn_entity", "entities[].spawn_item_event"),
        ("minecraft:tameable", "tame_event"),
        ("minecraft:tamemount", "tame_event"),
        ("minecraft:target_nearby_sensor", "on_inside_range"),
        ("minecraft:target_nearby_sensor", "on_outside_range"),
        (
            "minecraft:target_nearby_sensor",
            "on_vision_lost_inside_range",
        ),
        ("minecraft:timer", "time_down_event"),
        ("minecraft:trusting", "trust_event"),
    ];

    /// Every `(component, trigger member)` pair of the table, pinned: the table is read from
    /// the payloads' `Schema` and the rows' `triggers = [..]`, so a payload that stops listing a
    /// trigger (or starts listing one) shows here by name, not only in the count.
    #[test]
    fn the_trigger_table_lists_each_pinned_pair() {
        let table: Vec<(&str, &str)> = TRIGGERS
            .iter()
            .map(|(component, member)| (*component, member.as_str()))
            .collect();
        assert_eq!(table, PINNED_TRIGGERS);
    }

    /// An entity with `n` events, each triggering the next and setting a property, `n`
    /// animate entries and a sensor with `n` triggers; one reference in four names nothing.
    fn entity(n: usize) -> Entity {
        use bedrock_addon_core::id::EntityId;
        let json = |text: &str| bedrock_addon_core::json::from_str::<Value>(text).expect("json");
        let mut entity = Entity::new(EntityId::parse("demo:pet"));
        let name = |i: usize| {
            if i % 4 == 3 {
                format!("demo:none_{i}")
            } else {
                format!("demo:e_{}", (i + 1) % n)
            }
        };
        for i in 0..n {
            let node = json(&format!(
                r#"{{"sequence": [{{"trigger": "{}"}}, {{"set_property": {{"demo:p_{i}": 1}}}}]}}"#,
                name(i)
            ));
            let key = format!("demo:e_{i}");
            entity
                .events
                .insert(key.clone(), super::super::Event { name: key, node });
            entity
                .description
                .animations
                .insert(format!("a_{i}"), format!("animation.a_{i}"));
        }
        let animate = (0..n).map(|i| AnimateEntry::Always(format!("a_{}", i + n / 2)));
        entity.description.scripts = Some(super::super::Scripts {
            animate: animate.collect(),
            ..Default::default()
        });
        let triggers: Vec<String> = (0..n)
            .map(|i| {
                format!(
                    r#"{{"event": "{}", "filters": {{"test": "is_daytime", "value": true}}}}"#,
                    name(i)
                )
            })
            .collect();
        entity.components.insert_dynamic(
            "minecraft:environment_sensor",
            json(&format!(r#"{{"triggers": [{}]}}"#, triggers.join(","))),
        );
        entity
    }

    /// The findings of the check on `entity`.
    fn checked(entity: &Entity) -> usize {
        use bedrock_addon_core::diagnostic::Diagnostics;
        use bedrock_addon_core::load::LoadContext;
        use bedrock_addon_core::version::FormatVersion;
        let load = LoadContext::detached();
        let mut diagnostics = Diagnostics::default();
        let mut cx = ValidateCx {
            load: &load,
            original_version: FormatVersion::new(1, 26, 50),
            at: JsonPath::new(),
            skipped: Vec::new(),
            #[cfg(feature = "molang")]
            molang_failures: Vec::new(),
            reading: load.reading,
            diagnostics: &mut diagnostics,
        };
        check(entity, &mut cx);
        diagnostics.len()
    }

    #[test]
    fn the_check_tells_each_reference_that_names_nothing_once() {
        // Of 8: the triggers of events 3 and 7 and of sensor entries 3 and 7; every
        // `set_property` key (no property is declared); the animate entries 4 to 7 (`a_8` …).
        assert_eq!(checked(&entity(8)), 2 + 2 + 8 + 4);
    }

    #[test]
    fn the_check_of_n_references_among_n_events_costs_n() {
        bedrock_addon_core::test_cost::assert_linear_at(
            "entity references: n triggers, n events",
            5_000,
            entity,
            |entity: &Entity| {
                let found = checked(entity);
                assert!(found > entity.events.len());
                found
            },
        );
    }
}
