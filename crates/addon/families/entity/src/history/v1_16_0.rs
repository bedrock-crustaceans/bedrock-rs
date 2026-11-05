//! Format 1.16.0: the struct and the step to 1.16.100.

super::legacy_struct!(1, 16, 0);

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};
use facet_value::VArray;

use super::ops::{
    Members, Removed, Told, components, f32_value, float_of, json, overwrite, removed, rename,
    section_a, section_b, sections,
};

/// The step's version, for its findings.
const V1_16_100: FormatVersion = known::V1_16_100;

/// 1.16.0 → 1.16.100 (gate 1.16.100).
impl Upgrade for Entity {
    type Next = super::v1_16_100::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        if section_a(&payload, cx) {
            restriction_split(&mut payload, cx);
            if cx.actor_type == ActorType::Boat {
                let mut told = Told::default();
                boat(&mut payload, &mut told);
                told.report(cx);
            }
        }
        if section_b(&payload, cx) {
            rewrites(&mut payload, cx);
        }
        super::v1_16_100::Entity(payload)
    }
}

/// A, all identifiers, every section: `behavior.move_towards_restriction` becomes the dwelling
/// goal next to `dweller`, the home goal (with `home {restriction_radius: 16}`) for the
/// guardians, and is removed otherwise.
fn restriction_split(payload: &mut Payload, cx: &mut UpgradeCx<'_>) {
    const OLD: &str = "minecraft:behavior.move_towards_restriction";
    let guardian = matches!(
        cx.actor_type,
        ActorType::Guardian | ActorType::ElderGuardian
    );
    let mut gone = Vec::new();
    let mut told = Told::default();
    for section in sections(payload) {
        let members = section.members;
        if !members.has(OLD) {
            continue;
        }
        if members.has("minecraft:dweller") {
            const NEW: &str = "minecraft:behavior.move_towards_dwelling_restriction";
            rename(members, &section.at, OLD, NEW, &mut told);
        } else if guardian {
            const NEW: &str = "minecraft:behavior.move_towards_home_restriction";
            rename(members, &section.at, OLD, NEW, &mut told);
            members.insert_absent("minecraft:home", json(r#"{"restriction_radius": 16}"#));
        } else {
            members.take(OLD);
            gone.push(section.at.child(OLD));
        }
    }
    for at in gone {
        removed(cx, Removed::Component, &at, OLD, V1_16_100);
    }
    told.report(cx);
}

/// A, vanilla `boat`: the bubble-column groups, components and
/// events, one constant fragment. Their members and the events follow vanilla's own boat
/// file.
fn boat(payload: &mut Payload, told: &mut Told) {
    let rideable = payload
        .components
        .as_ref()
        .and_then(|c| c.get("minecraft:rideable").cloned())
        .unwrap_or_else(|| json("{}"));
    let fragment = json(BOAT);
    let Some(fragment) = fragment.as_object() else {
        return;
    };
    let groups = payload
        .component_groups
        .get_or_insert_with(Default::default);
    // What the fixer writes over a group, an event or a component the file has with another
    // value is noted where the file has it (an information: the game's own fixer does it).
    if let Some(new) = fragment.get("component_groups").and_then(|g| g.as_object()) {
        for (name, group) in new {
            // `can_ride` is the one group the fixer adds to: the file's own stays.
            if name.as_str() == "minecraft:can_ride" && groups.contains_key(name.as_str()) {
                continue;
            }
            if groups
                .get(name.as_str())
                .is_some_and(|had| !had.is_null() && had != group)
            {
                told.dropped(JsonPath::new().with(&["component_groups", name.as_str()]));
            }
            groups.insert(name.as_str().to_owned(), group.clone());
        }
    }
    if let Some(can_ride) = groups
        .get_mut("minecraft:can_ride")
        .and_then(|g| g.as_object_mut())
    {
        can_ride.insert_absent("minecraft:rideable", rideable);
    }
    let events = payload.events.get_or_insert_with(Default::default);
    if let Some(new) = fragment.get("events").and_then(|g| g.as_object()) {
        for (name, event) in new {
            if events
                .get(name.as_str())
                .is_some_and(|had| !had.is_null() && had != event)
            {
                told.dropped(JsonPath::new().with(&["events", name.as_str()]));
            }
            events.insert(name.as_str().to_owned(), event.clone());
        }
    }
    let components = components(payload);
    if let Some(new) = fragment.get("components").and_then(|g| g.as_object()) {
        for (name, component) in new {
            if Members::get(components, name.as_str())
                .is_some_and(|had| !had.is_null() && had != component)
            {
                told.dropped(JsonPath::new().with(&["components", name.as_str()]));
            }
            components.set(name.as_str(), component.clone());
        }
    }
}

/// The boat fragment.
const BOAT: &str = r#"{
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

/// B, every section: `drop_item_for.max_dist` → `target_range`, `explore_outskirts.wait_time`
/// (ticks) → `max_wait_time` (seconds), `economy_trade_table.use_legacy_price_formula`, and
/// `foot_size` removed.
fn rewrites(payload: &mut Payload, cx: &mut UpgradeCx<'_>) {
    let mut gone = Vec::new();
    let mut told = Told::default();
    for section in sections(payload) {
        let members = section.members;
        // What the step writes into a member the file already has is noted there.
        if let Some(goal) = members.object("minecraft:behavior.drop_item_for")
            && let Some(distance) = goal.remove("max_dist")
        {
            const GOAL: &str = "minecraft:behavior.drop_item_for";
            let at = section.at.child(GOAL);
            let mut range = VArray::new();
            for _ in 0..3 {
                range.push(distance.clone());
            }
            overwrite(goal, "target_range", range.into(), &at, &mut told);
            told.moved(at.child("max_dist"), at.child("target_range"));
        }
        // A `null` member is an absent one. The crate's own note: a `wait_time` that is no number
        // is read as 0, one beyond a float as the largest float, and told at the member.
        if let Some(goal) = members.object("minecraft:behavior.explore_outskirts")
            && let Some(wait) = goal.remove("wait_time")
            && !wait.is_null()
        {
            const GOAL: &str = "minecraft:behavior.explore_outskirts";
            let at = section.at.child(GOAL);
            let ticks = told.read(float_of(&wait), || at.child("wait_time"));
            overwrite(
                goal,
                "max_wait_time",
                f32_value(ticks * 0.05),
                &at,
                &mut told,
            );
            told.moved(at.child("wait_time"), at.child("max_wait_time"));
        }
        if let Some(table) = members.object("minecraft:economy_trade_table") {
            table.insert_absent("use_legacy_price_formula", true.into());
        }
        if members.take("minecraft:foot_size").is_some() {
            gone.push(section.at.child("minecraft:foot_size"));
        }
    }
    for at in gone {
        removed(
            cx,
            Removed::Component,
            &at,
            "minecraft:foot_size",
            V1_16_100,
        );
    }
    told.report(cx);
}
