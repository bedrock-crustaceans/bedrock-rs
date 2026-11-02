//! Components about the entity's life in the world: `instant_despawn`, `tick_world`,
//! `spawn_on_death`, `addrider`, `home`; and the components 1.26.60 types first:
//! `transformation`, `timer`, `scale_by_age`.
// Types and defaults follow the game's schemas: `instant_despawn` and `home` from 1.26.50,
// `tick_world` from 1.21.100, `spawn_on_death` from 1.26.20 and `addrider` from 1.21.130.

use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, Severity, VanillaText};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, StrictString, Value};
use bedrock_addon_core::shared::{EventTrigger, Filter, FloatRange, IntRange};
use bedrock_addon_core::versioning::ConvertMode;
use facet::Facet;
use facet_value::{VArray, VObject};

use super::parse::{Check, Typed, index, join, path};
use super::value::one_element;

/// `minecraft:instant_despawn`: the entity despawns at once.
#[doc(alias = "minecraft:instant_despawn")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct InstantDespawn {
    /// `remove_child_entities`: the entity's children despawn with it; default `false`.
    #[facet(default = StrictBool(false))]
    pub remove_child_entities: StrictBool,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Typed for InstantDespawn {}

/// `minecraft:tick_world`: the entity keeps the chunks around it ticking.
#[doc(alias = "minecraft:tick_world")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct TickWorld {
    /// `distance_to_players`: beyond this distance from every player the entity may despawn
    /// (unless `never_despawn`); at least 128, the default.
    #[facet(default = StrictFloat(128.0_f32))]
    pub distance_to_players: StrictFloat<f32>,
    /// `never_despawn`: the entity never despawns; default `true`.
    #[facet(default = StrictBool(true))]
    pub never_despawn: StrictBool,
    /// `radius`: the ticking area's radius in chunks, 2 to 6; default 2.
    #[facet(default = StrictInt(2_u32))]
    pub radius: StrictInt<u32>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for TickWorld {
    fn default() -> Self {
        Self {
            distance_to_players: StrictFloat(128.0),
            never_despawn: StrictBool(true),
            radius: StrictInt(2),
            extra: Extra::default(),
        }
    }
}

impl Typed for TickWorld {
    // The game's schema: `distance_to_players` minimum 128, `radius` 2 ..= 6.
    fn check(&mut self, cx: &mut Check<'_>) {
        cx.at_least(
            &path(&["distance_to_players"]),
            &mut self.distance_to_players.0,
            128.0,
            128.0,
        );
        cx.within(&path(&["radius"]), &mut self.radius.0, (2, 6), 2);
    }
}

/// `minecraft:spawn_on_death`: entities spawned where the entity dies.
#[doc(alias = "minecraft:spawn_on_death")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SpawnOnDeath {
    /// `additional_spawn_range`: a random number of extra entities in this range; default
    /// `{min: 0, max: 0}`.
    #[facet(default = IntRange::default())]
    pub additional_spawn_range: IntRange,
    /// `entity_to_spawn`: the entity identifier, or `self` for the dying entity's own type;
    /// default `self`.
    #[facet(default = StrictString("self".to_owned()))]
    pub entity_to_spawn: StrictString,
    /// `filters`: the condition for spawning; absent means always.
    #[facet(default)]
    pub filters: Option<Filter>,
    /// `inherit_parent_name`: the spawned entities take the dying entity's name; default `true`.
    #[facet(default = StrictBool(true))]
    pub inherit_parent_name: StrictBool,
    /// `spawn_amount`: how many entities spawn; default 1.
    #[facet(default = StrictInt(1_i32))]
    pub spawn_amount: StrictInt<i32>,
    /// `spawn_method`: the spawn reason the new entities get; default `spawned`.
    #[facet(default = StrictString("spawned".to_owned()))]
    pub spawn_method: StrictString,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for SpawnOnDeath {
    fn default() -> Self {
        Self {
            additional_spawn_range: IntRange::default(),
            entity_to_spawn: StrictString("self".to_owned()),
            filters: None,
            inherit_parent_name: StrictBool(true),
            spawn_amount: StrictInt(1),
            spawn_method: StrictString("spawned".to_owned()),
            extra: Extra::default(),
        }
    }
}

impl Typed for SpawnOnDeath {
    fn check(&mut self, cx: &mut Check<'_>) {
        cx.range(
            &path(&["additional_spawn_range"]),
            &mut self.additional_spawn_range,
        );
        if let Some(filters) = &self.filters {
            cx.extend(filters.check(&path(&["filters"])));
        }
    }
}

/// `minecraft:addrider`: entities that ride this one when it spawns.
#[doc(alias = "minecraft:addrider")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Addrider {
    /// `riders`: the riders, in seat order.
    #[facet(default)]
    pub riders: Vec<Rider>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One rider of [`Addrider`].
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Rider {
    /// `entity_type`: the rider's entity identifier; required.
    pub entity_type: StrictString,
    /// `spawn_event`: the event the rider spawns with; default none (empty).
    #[facet(default = StrictString(String::new()))]
    pub spawn_event: StrictString,
    /// Members the rider does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Rider {
    /// A rider of type `entity_type` without a spawn event.
    pub fn new(entity_type: impl Into<String>) -> Self {
        Self {
            entity_type: StrictString(entity_type.into()),
            spawn_event: StrictString::default(),
            extra: Extra::default(),
        }
    }
}

impl Typed for Addrider {
    /// The lenient conversion: the legacy single rider at the component root
    /// (`entity_type`, `spawn_event`) becomes the one element of `riders`.
    // The crate's own reading, taken from the game's packs: the 1.13.0 `spider.json` writes
    // `{"entity_type": "minecraft:skeleton"}`, its 1.26.50 copy `riders: [{entity_type}]`.
    //
    // The game makes the rider only of an `entity_type` that is not empty: `riders` starts
    // empty, then a non-empty `entity_type` makes the rider, with `spawn_event` as its init
    // event. Without one the two members are read and never used: dropped, each with an
    // information.
    fn prepare(json: &mut VObject, cx: &mut Check<'_>) -> bool {
        if !matches!(cx.mode(), ConvertMode::Lenient) || json.contains_key("riders") {
            return false;
        }
        let no_type = json.get("entity_type").is_none_or(|value| {
            value
                .as_string()
                .is_some_and(facet_value::VString::is_empty)
        });
        if no_type {
            let mut changed = false;
            for key in ["entity_type", "spawn_event"] {
                if json.remove(key).is_some() {
                    cx.push(
                        DiagnosticCode::DroppedField { text: None },
                        Severity::Info,
                        &path(&[key]),
                    );
                    changed = true;
                }
            }
            return changed;
        }
        // The rider is the component's root in the file: what is told of `riders/0` is told
        // there.
        let element = index(&path(&["riders"]), 0);
        cx.moved(path(&[]), element.clone());
        let mut rider = VObject::new();
        for key in ["entity_type", "spawn_event"] {
            if let Some(value) = json.remove(key) {
                rider.insert(key, value);
                cx.moved(path(&[key]), join(&element, key));
            }
        }
        let mut riders = VArray::new();
        riders.push(Value::from(rider));
        json.insert("riders", Value::from(riders));
        true
    }
}

/// `minecraft:home`: the entity remembers where it spawned and stays near it.
#[doc(alias = "minecraft:home")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Home {
    /// `home_block_list`: the block identifiers that can be the entity's home.
    #[facet(default)]
    pub home_block_list: Vec<StrictString>,
    /// `restriction_radius`: how far from home the entity may go; 0 (the default) is no limit.
    #[facet(default = StrictInt(0_i32))]
    pub restriction_radius: StrictInt<i32>,
    /// `restriction_type`: which movement the radius limits; default `none`.
    #[facet(default = RestrictionType::None)]
    pub restriction_type: RestrictionType,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Typed for Home {}

/// What a [`Home`]'s `restriction_radius` limits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
#[repr(u8)]
#[non_exhaustive]
pub enum RestrictionType {
    /// Nothing (JSON `none`).
    #[default]
    #[facet(rename = "none")]
    None,
    /// Every movement (JSON `all_movement`).
    #[facet(rename = "all_movement")]
    AllMovement,
    /// Random wandering only (JSON `random_movement`).
    #[facet(rename = "random_movement")]
    RandomMovement,
}

/// `minecraft:transformation`: the entity turns into another definition.
// `transformation`: `into` is a definition reference, the two sound members are lists of
// level sound events (empty by default), five flags default to false, and `add` and `delay`
// are nested types whose members are zero (typed and legacy agree); the legacy parser takes a
// number `delay` as its value. The game's own template writes `transformation_sound` as a
// list, vanilla's files as one name: both are read.
#[doc(alias = "minecraft:transformation")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Transformation {
    /// `into`: the definition it turns into (with an optional `<event>`); default none.
    #[facet(default = StrictString(String::new()))]
    pub into: StrictString,
    /// `begin_transform_sound`: the sounds when the transformation starts; default none.
    #[facet(default)]
    pub begin_transform_sound: Vec<StrictString>,
    /// `transformation_sound`: the sounds when it is done; default none.
    #[facet(default)]
    pub transformation_sound: Vec<StrictString>,
    /// `drop_equipment`: it drops its equipment; default `false`.
    #[facet(default = StrictBool(false))]
    pub drop_equipment: StrictBool,
    /// `drop_inventory`: it drops its inventory; default `false`.
    #[facet(default = StrictBool(false))]
    pub drop_inventory: StrictBool,
    /// `preserve_equipment`: it keeps its equipment; default `false`.
    #[facet(default = StrictBool(false))]
    pub preserve_equipment: StrictBool,
    /// `keep_owner`: an owned entity stays owned; default `false`.
    #[facet(default = StrictBool(false))]
    pub keep_owner: StrictBool,
    /// `keep_level`: a trader keeps its level; default `false`.
    #[facet(default = StrictBool(false))]
    pub keep_level: StrictBool,
    /// `add`: what the new entity gets.
    #[facet(default)]
    pub add: TransformationAdd,
    /// `delay`: how long the transformation takes.
    #[facet(default)]
    pub delay: TransformationDelay,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The `add` of a [`Transformation`].
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct TransformationAdd {
    /// `component_groups`: the component groups added.
    #[facet(default)]
    pub component_groups: Vec<StrictString>,
    /// Members the object does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The `delay` of a [`Transformation`]; every member defaults to 0.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct TransformationDelay {
    /// `value`: seconds before it transforms; at least 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub value: StrictFloat<f32>,
    /// `range_min`: seconds added to `value` for the low end of a random time.
    #[facet(default = StrictFloat(0.0_f32))]
    pub range_min: StrictFloat<f32>,
    /// `range_max`: seconds added to `value` for the high end of a random time.
    #[facet(default = StrictFloat(0.0_f32))]
    pub range_max: StrictFloat<f32>,
    /// `block_assist_chance`: the chance it looks for blocks that speed it up; 0 to 1.
    #[facet(default = StrictFloat(0.0_f32))]
    pub block_assist_chance: StrictFloat<f32>,
    /// `block_radius`: how far it looks for those blocks; at least 0.
    #[facet(default = StrictInt(0_i32))]
    pub block_radius: StrictInt<i32>,
    /// `block_max`: the most blocks it looks for (0: the radius); at least 0.
    #[facet(default = StrictInt(0_i32))]
    pub block_max: StrictInt<i32>,
    /// `block_chance`: the chance a found block helps; 0 to 1.
    #[facet(default = StrictFloat(0.0_f32))]
    pub block_chance: StrictFloat<f32>,
    /// `block_types`: the blocks that help.
    #[facet(default)]
    pub block_types: Vec<StrictString>,
    /// Members the object does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Typed for Transformation {
    const KEEP_NESTED: bool = true;

    /// Below 1.26.60 a sound written as one name is the list of it (vanilla's own files write
    /// one name) and a number `delay` is the delay's `value`. The typed reader refuses a lone
    /// name: the game logs `expected an array` for a value that is no array. A `delay` that is
    /// neither a number nor an object gets the game's `Expected Number or Object.` text (for
    /// example `delay: 15.0` is a number and stays).
    fn prepare(json: &mut VObject, cx: &mut Check<'_>) -> bool {
        if !matches!(cx.mode(), ConvertMode::Lenient) {
            return false;
        }
        let mut changed = false;
        for key in ["begin_transform_sound", "transformation_sound"] {
            if json.get(key).is_some_and(Value::is_string) {
                changed |= one_element(json, key, &JsonPath::new(), cx);
            }
        }
        // The legacy reader's text for a `delay` that is neither; the delay keeps its default.
        if json
            .get("delay")
            .is_some_and(|delay| !delay.is_null() && !delay.is_number() && !delay.is_object())
        {
            json.remove("delay");
            // The game logs this at the error level and keeps the document with the default
            // delay: a kept error.
            cx.kept(
                DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::Fixed("Expected Number or Object.")),
                },
                &path(&["delay"]),
            );
            return true;
        }
        let Some(number) = json
            .get("delay")
            .filter(|delay| delay.as_number().is_some())
        else {
            return changed;
        };
        let mut delay = VObject::new();
        delay.insert("value", number.clone());
        json.insert("delay", Value::from(delay));
        cx.moved(path(&["delay"]), path(&["delay", "value"]));
        true
    }

    /// The typed reader's numeric constraints: `value`, `block_radius`,
    /// `block_max` at least 0, the two chances 0 ..= 1. The legacy parser has none, so they
    /// bind only at the typed version.
    fn check(&mut self, cx: &mut Check<'_>) {
        if !matches!(cx.mode(), ConvertMode::Strict) {
            return;
        }
        let delay = &mut self.delay;
        let at = |key: &str| path(&["delay", key]);
        cx.at_least(&at("value"), &mut delay.value.0, 0.0, 0.0);
        cx.at_least(&at("block_radius"), &mut delay.block_radius.0, 0, 0);
        cx.at_least(&at("block_max"), &mut delay.block_max.0, 0, 0);
        cx.within(
            &at("block_assist_chance"),
            &mut delay.block_assist_chance.0,
            (0.0, 1.0),
            0.0,
        );
        cx.within(
            &at("block_chance"),
            &mut delay.block_chance.0,
            (0.0, 1.0),
            0.0,
        );
    }
}

/// `minecraft:timer`: an event fires when a timer runs out.
// `timer`: `looping` and `randomInterval` default to true (the one camel-case member the typed
// type binds), `time` is an optional range, and the trigger and the weighted choices follow;
// the legacy `{range_min, range_max}` time is the typed `{min, max}`.
#[doc(alias = "minecraft:timer")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Timer {
    /// `looping`: the timer restarts when it fires; default `true`.
    #[facet(default = StrictBool(true))]
    pub looping: StrictBool,
    /// `randomInterval`: the time is random between `time`'s ends; default `true`.
    #[facet(rename = "randomInterval", default = StrictBool(true))]
    pub random_interval: StrictBool,
    /// `time`: the seconds on the timer, a range; absent when `random_time_choices` is used.
    #[facet(default)]
    pub time: Option<FloatRange>,
    /// `time_down_event`: the event fired when the time runs out.
    #[facet(default)]
    pub time_down_event: Option<EventTrigger>,
    /// `random_time_choices`: weighted times, one picked each time.
    #[facet(default)]
    pub random_time_choices: Vec<TimeChoice>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            looping: StrictBool(true),
            random_interval: StrictBool(true),
            time: None,
            time_down_event: None,
            random_time_choices: Vec::new(),
            extra: Extra::default(),
        }
    }
}

/// One of a [`Timer`]'s `random_time_choices`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct TimeChoice {
    /// `value`: the seconds; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub value: StrictFloat<f32>,
    /// `weight`: how likely this choice is; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub weight: StrictFloat<f32>,
    /// Members the object does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Typed for Timer {
    // Not KEEP_NESTED: the legacy reader is one of the game's schema readers, which validates
    // the whole tree before it parses: a nested value of another type never reaches this
    // read; from 1.15.0 it defaults the whole component.

    fn prepare(json: &mut VObject, cx: &mut Check<'_>) -> bool {
        if !matches!(cx.mode(), ConvertMode::Lenient) {
            return false;
        }
        super::movement::legacy_range(json, "time", false, &JsonPath::new(), cx)
    }

    fn check(&mut self, cx: &mut Check<'_>) {
        if let Some(time) = &mut self.time {
            cx.range(&path(&["time"]), time);
        }
        if let Some(trigger) = &self.time_down_event {
            cx.extend(trigger.check(&path(&["time_down_event"])));
        }
    }
}

/// `minecraft:scale_by_age`: the entity's size follows its age.
// `scale_by_age`: the start and end scale default to 1.
#[doc(alias = "minecraft:scale_by_age")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ScaleByAge {
    /// `start_scale`: the scale of the newborn; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub start_scale: StrictFloat<f32>,
    /// `end_scale`: the scale when fully grown; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub end_scale: StrictFloat<f32>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for ScaleByAge {
    fn default() -> Self {
        Self {
            start_scale: StrictFloat(1.0),
            end_scale: StrictFloat(1.0),
            extra: Extra::default(),
        }
    }
}

impl Typed for ScaleByAge {}
