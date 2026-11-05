//! Format 1.12.0: the struct and the step to 1.13.0: section A's
//! 1.13 entity rewrite and section B's trigger reshaping.

super::legacy_struct!(1, 12, 0);

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};
use facet_value::{VArray, VObject};

use super::ops::{
    Members, Removed, Told, components, f32_value, group, has_bits, is, json, number_of, overwrite,
    removed, section_a, section_b, sections,
};
use bedrock_addon_core::diagnostic::JsonPath;

/// The step's version, for its findings.
const V1_13_0: FormatVersion = FormatVersion::new(1, 13, 0);

/// 1.12.0 → 1.13.0 (gate 1.13.0).
impl Upgrade for Entity {
    type Next = super::v1_13_0::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        if section_a(&payload, cx) {
            section_a_113(&mut payload, cx);
        }
        if section_b(&payload, cx) {
            section_b_113(&mut payload, cx);
        }
        super::v1_13_0::Entity(payload)
    }
}

use ActorType as T;

/// `minecraft:persistent {}` (op 1).
const PERSISTENT: &[ActorType] = &[
    T::Agent,
    T::Npc,
    T::ArmorStand,
    T::IronGolem,
    T::SnowGolem,
    T::EnderDragon,
    T::EvocationIllager,
    T::Villager,
    T::VillagerV2,
    T::Wither,
];

/// `pushable {is_pushable: false, is_pushable_by_piston: true}` (op 2).
const NOT_PUSHABLE: &[ActorType] = &[
    T::Agent,
    T::ArmorStand,
    T::Player,
    T::Arrow,
    T::FishingHook,
    T::TntMinecart,
];

/// The types the `pushable` switch names without an op (op 2). The rest take the default branch.
const PUSHABLE_NO_OP: &[ActorType] = &[
    T::Bat,
    T::Panda,
    T::Cat,
    T::Fox,
    T::Bee,
    T::Shulker,
    T::Vex,
    T::Npc,
    T::EnderDragon,
];

/// The fire-immune mobs left out of the lava entry (op 4). The set is "most mobs": the mob
/// types that do not burn are taken out.
const NO_LAVA_ENTRY: &[ActorType] = &[
    T::Blaze,
    T::Ghast,
    T::MagmaCube,
    T::Strider,
    T::Wither,
    T::EnderDragon,
    T::ZombiePigman,
    T::WitherSkeleton,
];

/// The types without `"mob"` in `type_family` (op 8), besides the minecarts and projectiles.
const NO_MOB_FAMILY: &[ActorType] = &[
    T::Tnt,
    T::XpOrb,
    T::EyeOfEnderSignal,
    T::EnderCrystal,
    T::FireworksRocket,
    T::FishingHook,
    T::Boat,
    T::LightningBolt,
    T::AreaEffectCloud,
    T::Agent,
    T::Player,
    T::Cod,
    T::Pufferfish,
    T::Salmon,
    T::Tropicalfish,
];

/// The `Minecart` category bit of a type id.
const MINECART: u32 = 0x8_0000;
/// The `Projectile` category bit of a type id.
const PROJECTILE: u32 = 0x40_0000;
/// The `Mob` category bit of a type id.
const MOB: u32 = 0x100;

/// Section A of the step, in vanilla's order.
fn section_a_113(payload: &mut Payload, cx: &mut UpgradeCx<'_>) {
    let components = components(payload);
    // 1. `persistent`.
    if is(cx, PERSISTENT) {
        components.insert_absent("minecraft:persistent", json("{}"));
    }
    // 2. `pushable`, each member on its own.
    let pushable = if is(cx, NOT_PUSHABLE) {
        Some(false)
    } else if is(cx, PUSHABLE_NO_OP) {
        None
    } else {
        Some(true)
    };
    if let Some(is_pushable) = pushable {
        components.insert_absent("minecraft:pushable", json("{}"));
        if let Some(object) = components.object("minecraft:pushable") {
            object.insert_absent("is_pushable", is_pushable.into());
            object.insert_absent("is_pushable_by_piston", true.into());
        }
    }
    // 3. `target_when_pushed`, iron golem only (the `minecraft:` keys are vanilla's).
    if cx.actor_type == T::IronGolem && target_when_pushed(components) {
        // The crate's own note: an `entity_types` that is no array is replaced by the list of
        // vanilla's one entry; told at the member.
        let mut told = Told::default();
        told.wrong_type(JsonPath::new().with(&[
            "components",
            "minecraft:behavior.target_when_pushed",
            "entity_types",
        ]));
        told.report(cx);
    }
    // 4. The lava entry: vanilla types with the mob bit (not `Mob` itself) and a few others.
    let lava = cx.actor_type != T::Mob
        && !is(cx, NO_LAVA_ENTRY)
        && (has_bits(cx, MOB) || is(cx, &[T::Arrow, T::Boat]));
    if lava {
        // The entry is the fixer's own: nothing of the file's goes when it is not taken.
        let _ = append_condition(components, LAVA);
    }
    // 5. Per-type extras.
    flocking(components, cx.actor_type);
    if cx.actor_type == T::Turtle {
        let mut told = Told::default();
        turtle_lightning(components, &mut told);
        told.report(cx);
    }
    if cx.actor_type == T::SnowGolem {
        let _ = append_condition(components, TEMPERATURE);
        let _ = append_condition(components, RAIN);
        components.insert_absent(
            "minecraft:trail",
            json(&format!(
                r#"{{"block_type": "snow_layer", "spawn_filter": {{"test": "is_temperature_value", "operator": "<", "value": {}}}}}"#,
                f64::from(0.81_f32)
            )),
        );
    }
    // 6. `experience_reward`.
    experience_reward(payload, cx.actor_type);
    // 7. The pufferfish's puff state machine; the helper returns before op 8.
    if cx.actor_type == T::Pufferfish {
        let mut told = Told::default();
        pufferfish(payload, &mut told);
        told.report(cx);
        return;
    }
    // 8. `type_family` gets `"mob"`.
    let excluded = is(cx, NO_MOB_FAMILY) || has_bits(cx, MINECART) || has_bits(cx, PROJECTILE);
    if !excluded {
        for section in sections(payload) {
            if let Some(family) = section.members.object("minecraft:type_family") {
                add_mob_family(family);
            }
        }
    }
}

/// The lava damage entry.
const LAVA: &str = r#"{"filters": {"test": "in_lava", "subject": "self", "operator": "==", "value": true}, "cause": "lava", "damage_per_tick": 4}"#;
/// The snow golem's temperature entry.
const TEMPERATURE: &str = r#"{"filters": {"test": "is_temperature_value", "operator": ">", "value": 1.0}, "cause": "temperature", "damage_per_tick": 1}"#;
/// The rain entry (the snow golem's, and `hurt_when_wet`'s replacement).
const RAIN: &str = r#"{"filters": {"test": "in_water_or_rain", "operator": "==", "value": true}, "cause": "drowning", "damage_per_tick": 1}"#;

/// Appends `entry` to `minecraft:hurt_on_condition.damage_conditions`: the container is
/// created when absent; an existing `hurt_on_condition` without a `damage_conditions` list
/// gets nothing. Returns whether the entry was appended.
fn append_condition(section: &mut dyn Members, entry: &str) -> bool {
    if !section.has("minecraft:hurt_on_condition") {
        section.set(
            "minecraft:hurt_on_condition",
            json(r#"{"damage_conditions": []}"#),
        );
    }
    let Some(conditions) = section
        .object("minecraft:hurt_on_condition")
        .and_then(|h| h.get_mut("damage_conditions"))
        .and_then(|c| c.as_array_mut())
    else {
        return false;
    };
    conditions.push(json(entry));
    true
}

/// `minecraft:behavior.target_when_pushed` for the iron golem. Returns whether an
/// `entity_types` that is no array (and not `null`) was replaced.
fn target_when_pushed(components: &mut dyn Members) -> bool {
    components.insert_absent("minecraft:behavior.target_when_pushed", json("{}"));
    let Some(goal) = components.object("minecraft:behavior.target_when_pushed") else {
        return false;
    };
    goal.insert_absent("priority", 1.into());
    goal.insert_absent("percent_chance", 5.0.into());
    let mut replaced = false;
    if !goal.get("entity_types").is_some_and(Value::is_array) {
        replaced = goal
            .remove("entity_types")
            .is_some_and(|old| !old.is_null());
        goal.insert("entity_types", VArray::new());
    }
    if let Some(types) = goal.get_mut("entity_types").and_then(|t| t.as_array_mut()) {
        types.push(json(
            r#"{"minecraft:filters": {"minecraft:all_of": [
                {"test": "is_family", "subject": "other", "value": "monster"},
                {"test": "is_family", "subject": "other", "operator": "!=", "value": "creeper"}
            ]}}"#,
        ));
    }
    replaced
}

/// `minecraft:flocking` for the fish and the dolphin, every member when absent.
#[allow(clippy::too_many_lines, reason = "a table with one row per type")]
fn flocking(components: &mut dyn Members, actor: ActorType) {
    // in_water, match_variants, use_center_of_mass, influence_radius, breach_influence,
    // separation_weight, separation_threshold, cohesion_weight, cohesion_threshold,
    // innner_cohesion_threshold, min_height, max_height, block_distance, block_weight
    #[allow(clippy::type_complexity, reason = "one row of the table")]
    let row: (
        bool,
        bool,
        bool,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
    ) = match actor {
        T::Salmon => (
            true,
            false,
            false,
            3.0,
            7.0,
            f64::from(0.65_f32),
            f64::from(0.15_f32),
            2.25,
            1.5,
            1.5,
            4.0,
            4.0,
            1.0,
            0.75,
        ),
        T::Dolphin => (
            false,
            false,
            false,
            6.0,
            6.0,
            1.75,
            3.0,
            f64::from(1.85_f32),
            6.5,
            3.5,
            4.0,
            4.0,
            1.0,
            1.0,
        ),
        T::Cod | T::Pufferfish => (
            true,
            false,
            true,
            3.0,
            7.0,
            1.75,
            f64::from(0.95_f32),
            2.0,
            f64::from(1.95_f32),
            1.25,
            1.5,
            6.0,
            2.0,
            f64::from(0.85_f32),
        ),
        T::Tropicalfish => (
            true,
            true,
            false,
            3.0,
            7.0,
            f64::from(0.65_f32),
            f64::from(0.15_f32),
            2.75,
            1.5,
            1.5,
            1.5,
            6.0,
            2.0,
            f64::from(0.85_f32),
        ),
        _ => return,
    };
    components.insert_absent("minecraft:flocking", json("{}"));
    let Some(flocking) = components.object("minecraft:flocking") else {
        return;
    };
    let members: [(&str, Value); 17] = [
        ("in_water", row.0.into()),
        ("match_variants", row.1.into()),
        ("use_center_of_mass", row.2.into()),
        ("low_flock_limit", 4.into()),
        ("high_flock_limit", 8.into()),
        ("goal_weight", 2.0.into()),
        ("loner_chance", f32_value(0.1)),
        ("influence_radius", row.3.into()),
        ("breach_influence", row.4.into()),
        ("separation_weight", row.5.into()),
        ("separation_threshold", row.6.into()),
        ("cohesion_weight", row.7.into()),
        ("cohesion_threshold", row.8.into()),
        ("innner_cohesion_threshold", row.9.into()),
        ("min_height", row.10.into()),
        ("max_height", row.11.into()),
        ("block_distance", row.12.into()),
    ];
    for (key, value) in members {
        flocking.insert_absent(key, value);
    }
    flocking.insert_absent("block_weight", row.13.into());
}

/// The turtle's lightning trigger: `damage_sensor` normalised, then the first `lightning`
/// trigger gets `damage_multiplier: 2000.0` (unless it deals no damage), or one is appended.
fn turtle_lightning(components: &mut dyn Members, told: &mut Told) {
    let at = JsonPath::new().with(&["components", "minecraft:damage_sensor"]);
    let Some(sensor) = components.get_mut("minecraft:damage_sensor") else {
        components.set(
            "minecraft:damage_sensor",
            json(r#"{"triggers": [{"cause": "lightning", "deals_damage": true, "damage_multiplier": 2000.0}]}"#),
        );
        return;
    };
    normalise_damage_sensor(sensor, &at, told);
    let Some(triggers) = sensor
        .as_object_mut()
        .and_then(|s| s.get_mut("triggers"))
        .and_then(|t| t.as_array_mut())
    else {
        return;
    };
    let lightning = triggers.as_mut_slice().iter_mut().position(|t| {
        t.as_object()
            .and_then(|t| t.get("cause"))
            .and_then(|c| c.as_string())
            .is_some_and(|c| c.as_str() == "lightning")
    });
    let trigger =
        lightning.and_then(|index| Some((index, triggers.get_mut(index)?.as_object_mut()?)));
    match trigger {
        Some((index, trigger)) => {
            let deals = trigger.get("deals_damage").and_then(Value::as_bool);
            if deals != Some(false) {
                // The multiplier the turtle's file has is the fixer's to replace;
                // another value of the file's is noted.
                let at = at.child("triggers").index(index);
                overwrite(trigger, "damage_multiplier", 2000.0.into(), &at, told);
            }
        }
        None => triggers.push(json(
            r#"{"cause": "lightning", "deals_damage": true, "damage_multiplier": 2000.0}"#,
        )),
    }
}

/// `damage_sensor` → `{triggers: [...]}`; every trigger with both
/// `filters` and `event` gets them under `on_damage`.
fn normalise_damage_sensor(sensor: &mut Value, at: &JsonPath, told: &mut Told) {
    let triggers_at = at.child("triggers");
    let old = std::mem::replace(sensor, Value::NULL);
    let triggers = match old.as_object() {
        _ if old.is_array() => {
            told.moved(at.clone(), triggers_at);
            old
        }
        Some(object) => match object.get("triggers") {
            Some(triggers) if triggers.is_object() => {
                let mut list = VArray::new();
                list.push(triggers.clone());
                let mut out = object.clone();
                out.insert("triggers", list);
                *sensor = out.into();
                told.moved(triggers_at.clone(), triggers_at.index(0));
                wrap_on_damage(sensor, at, told);
                return;
            }
            Some(_) => {
                *sensor = old;
                wrap_on_damage(sensor, at, told);
                return;
            }
            None => {
                let mut list = VArray::new();
                list.push(old.clone());
                told.moved(at.clone(), triggers_at.index(0));
                list.into()
            }
        },
        None => {
            told.moved(at.clone(), triggers_at);
            old
        }
    };
    let mut out = VObject::new();
    out.insert("triggers", triggers);
    *sensor = out.into();
    wrap_on_damage(sensor, at, told);
}
/// Each trigger with both `filters` and `event` gets them under `on_damage`: in the
/// `on_damage` the trigger has, when it has one (the crate's own: the folder gives the move for
/// a trigger without one; a member of the file's `on_damage` the move writes over is noted).
fn wrap_on_damage(sensor: &mut Value, at: &JsonPath, told: &mut Told) {
    let Some(triggers) = sensor
        .as_object_mut()
        .and_then(|s| s.get_mut("triggers"))
        .and_then(|t| t.as_array_mut())
    else {
        return;
    };
    let list_at = at.child("triggers");
    for (index, trigger) in triggers.as_mut_slice().iter_mut().enumerate() {
        let Some(object) = trigger.as_object_mut() else {
            continue;
        };
        if object.contains_key("filters") && object.contains_key("event") {
            let trigger_at = list_at.index(index);
            let on_damage_at = trigger_at.child("on_damage");
            let mut on_damage = match object.remove("on_damage") {
                Some(own) if own.is_object() => own.as_object().cloned().unwrap_or_default(),
                Some(own) => {
                    // No object: replaced by the one the move makes.
                    if !own.is_null() {
                        told.wrong_type(on_damage_at.clone());
                    }
                    VObject::new()
                }
                None => VObject::new(),
            };
            for key in ["filters", "event"] {
                if let Some(value) = object.remove(key) {
                    overwrite(&mut on_damage, key, value, &on_damage_at, told);
                    told.moved(trigger_at.child(key), on_damage_at.child(key));
                }
            }
            let rest = object.clone();
            let mut rebuilt = VObject::new();
            rebuilt.insert("on_damage", on_damage);
            for (key, value) in &rest {
                rebuilt.insert(key.as_str(), value.clone());
            }
            *object = rebuilt;
        }
    }
}
/// The `experience_reward` table of op 6: where it goes and its members.
fn experience_reward(payload: &mut Payload, actor: ActorType) {
    const ON_DEATH_3: &str = "query.last_hit_by_player ? Math.Random(1,3) : 0";
    const ON_DEATH_4: &str = "query.last_hit_by_player ? Math.Random(1,4) : 0";
    const BRED: &str = "Math.Random(1,7)";
    const BABY: &str =
        "query.last_hit_by_player ? 12 + (query.equipment_count * Math.Random(1,3)) : 0";
    const ADULT: &str =
        "query.last_hit_by_player ? 5 + (query.equipment_count * Math.Random(1,3)) : 0";
    let in_components = match actor {
        T::Creeper
        | T::Enderman
        | T::Phantom
        | T::Shulker
        | T::Silverfish
        | T::Spider
        | T::CaveSpider => Some("query.last_hit_by_player ? 5 : 0"),
        T::Skeleton | T::Stray | T::WitherSkeleton | T::Vex | T::Ghast => {
            Some("query.last_hit_by_player ? 5 + (query.equipment_count * Math.Random(1,3)) : 0")
        }
        T::Cod | T::Pufferfish | T::Salmon | T::Tropicalfish => Some(ON_DEATH_3),
        T::Blaze | T::Guardian | T::ElderGuardian => Some("query.last_hit_by_player ? 10 : 0"),
        T::Pillager | T::Vindicator | T::Witch => Some(
            "query.last_hit_by_player ? (query.is_baby ? 12 : 5) + (Math.die_roll(query.equipment_count,1,3)) : 0",
        ),
        T::Slime | T::MagmaCube => Some("query.last_hit_by_player ? query.variant : 0"),
        T::Ravager => Some("query.last_hit_by_player ? 20 : 0"),
        T::EvocationIllager => Some("10"),
        T::Wither => Some("50"),
        T::Endermite => Some("query.last_hit_by_player ? 3 : 0"),
        T::Squid => Some("!query.is_baby && query.last_hit_by_player ? Math.Random(1,3) : 0"),
        T::Player => Some("Math.Min(query.player_level * 7, 100)"),
        _ => None,
    };
    if let Some(on_death) = in_components {
        reward(components(payload), None, on_death);
        return;
    }
    let groups: &[(&str, Option<&str>, &str)] = match actor {
        T::Zombie => &[
            ("minecraft:zombie_baby", None, BABY),
            ("minecraft:zombie_adult", None, ADULT),
        ],
        T::Husk => &[
            ("minecraft:zombie_husk_baby", None, BABY),
            ("minecraft:zombie_husk_adult", None, ADULT),
        ],
        T::Drowned => &[
            ("minecraft:baby_drowned", None, BABY),
            ("minecraft:adult_drowned", None, ADULT),
        ],
        T::ZombiePigman => &[
            ("minecraft:pig_zombie_baby", None, BABY),
            ("minecraft:pig_zombie_adult", None, ADULT),
        ],
        T::ZombieVillager | T::ZombieVillagerV2 => &[
            ("minecraft:baby", None, BABY),
            ("minecraft:adult", None, ADULT),
        ],
        T::Parrot => &[("minecraft:parrot_adult", None, ON_DEATH_3)],
        T::Mule => &[("minecraft:mule_adult", None, ON_DEATH_3)],
        T::SkeletonHorse => &[("minecraft:skeleton_horse_adult", None, ON_DEATH_3)],
        T::ZombieHorse => &[("minecraft:horse_adult", None, ON_DEATH_3)],
        T::PolarBear => &[("minecraft:adult", None, ON_DEATH_4)],
        T::Dolphin => &[("minecraft:dolphin_adult", None, ON_DEATH_4)],
        T::Turtle => &[("minecraft:adult", Some(BRED), ON_DEATH_4)],
        T::Panda => &[("minecraft:panda_adult", Some(BRED), ON_DEATH_4)],
        T::Cow | T::Mooshroom => &[("minecraft:cow_adult", Some(BRED), ON_DEATH_3)],
        T::Chicken => &[("minecraft:chicken_adult", Some(BRED), ON_DEATH_3)],
        T::Pig => &[("minecraft:pig_adult", Some(BRED), ON_DEATH_3)],
        T::Llama => &[("minecraft:llama_adult", Some(BRED), ON_DEATH_3)],
        T::Fox => &[("minecraft:fox_adult", Some(BRED), ON_DEATH_3)],
        T::Ocelot => &[("minecraft:ocelot_adult", Some(BRED), ON_DEATH_3)],
        T::Cat => &[("minecraft:cat_adult", Some(BRED), ON_DEATH_3)],
        T::Horse => &[("minecraft:horse_adult", Some(BRED), ON_DEATH_3)],
        T::Donkey => &[("minecraft:donkey_adult", Some(BRED), ON_DEATH_3)],
        T::Sheep => &[("sheep_adult", Some(BRED), ON_DEATH_3)],
        T::Wolf => &[("wolf_adult", Some(BRED), ON_DEATH_3)],
        T::Rabbit => &[("adult", Some(BRED), ON_DEATH_3)],
        _ => &[],
    };
    for (name, on_bred, on_death) in groups {
        if let Some(group) = group(payload, name) {
            reward(group, *on_bred, on_death);
        }
    }
}

/// `minecraft:experience_reward` in `section` when absent.
fn reward(section: &mut dyn Members, on_bred: Option<&str>, on_death: &str) {
    let mut object = VObject::new();
    if let Some(on_bred) = on_bred {
        object.insert("on_bred", on_bred);
    }
    object.insert("on_death", on_death);
    section.insert_absent("minecraft:experience_reward", object.into());
}

/// The pufferfish's puff state machine, one constant fragment.
fn pufferfish(payload: &mut Payload, told: &mut Told) {
    // The groups and events the fixer takes out, and those of its own it writes over the
    // file's: each one the file has is noted where it is (an information, the game's fixer).
    if let Some(groups) = &mut payload.component_groups {
        for name in [
            "minecraft:pufferfish_puffed",
            "minecraft:pufferfish_deflated",
        ] {
            if groups
                .shift_remove(name)
                .is_some_and(|had| holds_something(&had))
            {
                told.dropped(JsonPath::new().with(&["component_groups", name]));
            }
        }
    }
    if let Some(events) = &mut payload.events {
        for name in [
            "minecraft:entity_spawned",
            "minecraft:on_puff",
            "minecraft:on_deflate",
        ] {
            if events
                .shift_remove(name)
                .is_some_and(|had| holds_something(&had))
            {
                told.dropped(JsonPath::new().with(&["events", name]));
            }
        }
    }
    let fragment = json(PUFFERFISH);
    let Some(fragment) = fragment.as_object() else {
        return;
    };
    let groups = payload
        .component_groups
        .get_or_insert_with(Default::default);
    if let Some(new) = fragment.get("component_groups").and_then(|g| g.as_object()) {
        for (name, group) in new {
            if groups
                .get(name.as_str())
                .is_some_and(|had| !had.is_null() && had != group)
            {
                told.dropped(JsonPath::new().with(&["component_groups", name.as_str()]));
            }
            groups.insert(name.as_str().to_owned(), group.clone());
        }
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
    // Inserted when absent (op 7).
    let components = components(payload);
    components.insert_absent(
        "minecraft:movement",
        json(&format!(r#"{{"value": {}}}"#, f64::from(0.13_f32))),
    );
    components.insert_absent(
        "minecraft:underwater_movement",
        json(&format!(r#"{{"value": {}}}"#, f64::from(0.13_f32))),
    );
}

/// Whether a group or an event the fixer takes out holds anything of the author's: `null`,
/// `{}` and `[]` hold nothing.
fn holds_something(value: &Value) -> bool {
    !(value.is_null()
        || value.as_object().is_some_and(VObject::is_empty)
        || value.as_array().is_some_and(VArray::is_empty))
}

/// The pufferfish fragment; `A` and `M` are the filters it uses.
const PUFFERFISH: &str = r#"{
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

/// `type_family.family` gets `"mob"`: appended to an array (even when present); a string `s`
/// becomes `[s, "mob"]` (vanilla nests a second `type_family`, a bug this crate does not copy).
fn add_mob_family(family: &mut VObject) {
    match family.get_mut("family") {
        Some(list) if list.is_array() => {
            if let Some(list) = list.as_array_mut() {
                list.push("mob");
            }
        }
        Some(single) if single.is_string() => {
            let mut list = VArray::new();
            list.push(single.clone());
            list.push("mob");
            *single = list.into();
        }
        _ => {}
    }
}

/// Section B of the step (original version below 1.13.0), in vanilla's order, in `components`
/// and every group.
fn section_b_113(payload: &mut Payload, cx: &mut UpgradeCx<'_>) {
    let mut findings = Vec::new();
    // The crate's own note: what the step reads as another value, or replaces, is told at the
    // member.
    let mut told = Told::default();
    // The moves of `interact` and `scheduler`, recorded after their findings.
    let mut later = Told::default();
    for section in sections(payload) {
        let members = section.members;
        // The sensor's reshaping is recorded, so what is told of a trigger from here on is
        // told at the trigger as the file has it.
        let sensor_at = section.at.child("minecraft:damage_sensor");
        if let Some(sensor) = members.get_mut("minecraft:damage_sensor") {
            normalise_damage_sensor(sensor, &sensor_at, &mut told);
        }
        if let Some(wet) = members.take("minecraft:hurt_when_wet") {
            let wet_true = wet.as_bool() == Some(true)
                || wet
                    .as_object()
                    .and_then(|o| o.get("value"))
                    .and_then(Value::as_bool)
                    == Some(true);
            let at = section.at.child("minecraft:hurt_when_wet");
            // The entry is made of `value` alone, and only a `damage_conditions` list of
            // the section takes it: a component that makes no entry is told as removed,
            // whatever it held, and a member beside the `value` of one that does is told
            // at the component the file has.
            if wet_true && append_condition(members, RAIN) {
                for key in wet.as_object().into_iter().flat_map(VObject::keys) {
                    if key.as_str() != "value" {
                        told.unknown(at.child(key.as_str()));
                    }
                }
            } else {
                findings.push((Removed::Component, at, "minecraft:hurt_when_wet"));
            }
        }
        // A `null` component is an absent one.
        if let Some(fall) = members.take("minecraft:fall_damage")
            && !fall.is_null()
        {
            let at = section.at.child("minecraft:fall_damage");
            fall_damage(members, &fall, &at, &sensor_at, &mut told);
        }
        if let Some(sensor) = members.get_mut("minecraft:environment_sensor") {
            let at = section.at.child("minecraft:environment_sensor");
            environment_sensor(sensor, &at, &mut told);
        }
        if let Some(giveable) = members.get_mut("minecraft:giveable") {
            let at = section.at.child("minecraft:giveable");
            let old = giveable.clone();
            let mut wrapped = VObject::new();
            wrapped.insert("triggers", old);
            *giveable = wrapped.into();
            told.moved(at.clone(), at.child("triggers"));
        }
        if let Some(interact) = members.get_mut("minecraft:interact") {
            let at = section.at.child("minecraft:interact");
            for (index, lost) in interact_113(interact, &at, &mut later) {
                // At the member as the file has it: in the element of a list.
                let at = index.map_or_else(|| at.clone(), |index| at.index(index));
                findings.push((Removed::Field, at.child(lost), lost));
            }
        }
        if let Some(scheduler) = members.object("minecraft:scheduler") {
            let at = section
                .at
                .with(&["minecraft:scheduler", "scheduled_events"]);
            for index in scheduler_113(scheduler, &at, &mut told, &mut later) {
                findings.push((Removed::Field, at.index(index), "scheduled_events"));
            }
        }
        if let Some(spawn) = members.get_mut("minecraft:spawn_entity") {
            let at = section.at.child("minecraft:spawn_entity");
            spawn_entity_113(spawn, &at, &mut told);
        }
    }
    told.report(cx);
    // These findings name the member as the file has it: told before the moves of the
    // members they are about are recorded.
    for (what, at, member) in findings {
        removed(cx, what, &at, member, V1_13_0);
    }
    later.report(cx);
}

/// `fall_damage` → the section's `damage_sensor` `fall` trigger. A value that is no number is
/// read as 0 and told at `at`; a `deals_damage` or `damage_multiplier` the `fall` trigger had
/// is replaced, and told when it held another value.
fn fall_damage(
    members: &mut dyn Members,
    fall: &Value,
    at: &JsonPath,
    sensor_at: &JsonPath,
    told: &mut Told,
) {
    let (member, member_at) = match fall.as_object() {
        Some(object) => {
            // The step reads `value` alone: any other member goes with the component, and
            // is told.
            for key in object.keys().map(facet_value::VString::as_str) {
                if key != "value" {
                    told.unknown(at.child(key));
                }
            }
            (object.get("value"), at.child("value"))
        }
        None => (Some(fall), at.clone()),
    };
    let value = member.map_or(0.0, |member| {
        told.read(number_of(member), || member_at.clone())
    });
    if !members.has("minecraft:damage_sensor") {
        members.set("minecraft:damage_sensor", json(r#"{"triggers": []}"#));
    }
    let Some(sensor) = members.get_mut("minecraft:damage_sensor") else {
        return;
    };
    normalise_damage_sensor(sensor, sensor_at, told);
    let Some(sensor) = sensor.as_object_mut() else {
        return;
    };
    // A `triggers` that is no list takes no trigger: it is replaced by the list of the
    // `fall` trigger, and told where the file has it (as for `entity_sensor/subsensors`).
    // A `null` one is an absent one.
    if !sensor.get("triggers").is_some_and(Value::is_array) {
        if sensor.remove("triggers").is_some_and(|old| !old.is_null()) {
            told.wrong_type(sensor_at.child("triggers"));
        }
        sensor.insert("triggers", VArray::new());
    }
    let Some(triggers) = sensor.get_mut("triggers").and_then(|t| t.as_array_mut()) else {
        return;
    };
    let position = triggers.iter().position(|t| {
        t.as_object()
            .and_then(|t| t.get("cause"))
            .and_then(|c| c.as_string())
            .is_some_and(|c| c.as_str() == "fall")
    });
    let index = position.unwrap_or_else(|| {
        triggers.push(VObject::new());
        triggers.len() - 1
    });
    let Some(trigger) = triggers.get_mut(index).and_then(|t| t.as_object_mut()) else {
        return;
    };
    // A member the `fall` trigger of the file has is written over, and noted when it held
    // another value (the trigger's path is the one it has now; the sensor's reshaping is
    // recorded, so the finding sits at the trigger of the file).
    let trigger_at = sensor_at.child("triggers").index(index);
    if value == 0.0 {
        overwrite(trigger, "deals_damage", false.into(), &trigger_at, told);
    } else {
        overwrite(trigger, "deals_damage", true.into(), &trigger_at, told);
        overwrite(
            trigger,
            "damage_multiplier",
            value.into(),
            &trigger_at,
            told,
        );
        told.moved(member_at, trigger_at.child("damage_multiplier"));
    }
    trigger.insert("cause", "fall");
}
/// `environment_sensor`: `on_environment` → `triggers`, anything else wrapped.
fn environment_sensor(sensor: &mut Value, at: &JsonPath, told: &mut Told) {
    let triggers_at = at.child("triggers");
    if let Some(object) = sensor.as_object_mut()
        && object.contains_key("on_environment")
    {
        if let Some(on) = object.remove("on_environment") {
            if object.contains_key("triggers") {
                // The `triggers` the file has beside it stays; the old member goes.
                if !on.is_null() {
                    told.dropped(at.child("on_environment"));
                }
            } else {
                object.insert("triggers", on);
                told.moved(at.child("on_environment"), triggers_at);
            }
        }
        return;
    }
    let old = std::mem::replace(sensor, Value::NULL);
    told.moved(at.clone(), triggers_at.clone());
    let old = if let Some(items) = old.as_array() {
        let mut list = VArray::new();
        for (index, item) in items.iter().enumerate() {
            let on = item.as_object().and_then(|o| o.get("on_environment"));
            if on.is_some() {
                // The element becomes its `on_environment`: what else it holds goes, and
                // is told at the element the file has.
                for key in item.as_object().into_iter().flat_map(VObject::keys) {
                    if key.as_str() != "on_environment" {
                        told.unknown(at.index(index).child(key.as_str()));
                    }
                }
                let from = triggers_at.index(index).child("on_environment");
                told.moved(from, triggers_at.index(index));
            }
            list.push(on.cloned().unwrap_or_else(|| item.clone()));
        }
        list.into()
    } else {
        old
    };
    let mut wrapped = VObject::new();
    wrapped.insert("triggers", old);
    *sensor = wrapped.into();
}
/// `interact`: `event` / `target` / `filters` into `on_interact` (lost when it exists), then
/// wrapped as `{interactions: …}`. Returns the members lost, each with the index of its
/// element when `interact` is a list. An `on_interact` that is `null` is an absent one.
fn interact_113(
    interact: &mut Value,
    at: &JsonPath,
    told: &mut Told,
) -> Vec<(Option<usize>, &'static str)> {
    let mut lost = Vec::new();
    let mut move_members = |object: &mut VObject, index: Option<usize>| {
        if object.get("on_interact").is_some_and(Value::is_null) {
            object.remove("on_interact");
        }
        let entry_at = index.map_or_else(|| at.clone(), |index| at.index(index));
        let has_on = object.contains_key("on_interact");
        let mut on = VObject::new();
        for key in ["event", "target", "filters"] {
            if let Some(value) = object.remove(key) {
                if has_on {
                    lost.push((index, key));
                } else {
                    on.insert(key, value);
                    told.moved(entry_at.child(key), entry_at.with(&["on_interact", key]));
                }
            }
        }
        if !has_on && !on.is_empty() {
            object.insert("on_interact", on);
        }
    };
    if let Some(items) = interact.as_array_mut() {
        for (index, item) in items.as_mut_slice().iter_mut().enumerate() {
            if let Some(object) = item.as_object_mut() {
                move_members(object, Some(index));
            }
        }
    } else if let Some(object) = interact.as_object_mut() {
        move_members(object, None);
    }
    let old = std::mem::replace(interact, Value::NULL);
    let mut wrapped = VObject::new();
    wrapped.insert("interactions", old);
    *interact = wrapped.into();
    told.moved(at.clone(), at.child("interactions"));
    lost
}
/// `scheduler.scheduled_events[]`: each element replaced by its `scheduled_event`; an element
/// without one is dropped (vanilla writes `null`). Returns the indices of the dropped ones.
fn scheduler_113(
    scheduler: &mut VObject,
    at: &JsonPath,
    unread: &mut Told,
    told: &mut Told,
) -> Vec<usize> {
    let Some(events) = scheduler
        .get_mut("scheduled_events")
        .and_then(|e| e.as_array_mut())
    else {
        return Vec::new();
    };
    let mut dropped = Vec::new();
    let mut out = VArray::new();
    // The moves in the order of the elements: the move to an index is recorded before the
    // move FROM that index, so undoing the one does not undo the other.
    let mut moves = Vec::new();
    for (index, item) in events.iter().enumerate() {
        match item.as_object().and_then(|o| o.get("scheduled_event")) {
            Some(event) => {
                // The element becomes its `scheduled_event`: what else it holds goes, and
                // is told at the element the file has.
                for key in item.as_object().into_iter().flat_map(VObject::keys) {
                    if key.as_str() != "scheduled_event" {
                        unread.unknown(at.index(index).child(key.as_str()));
                    }
                }
                moves.push((
                    at.index(index).child("scheduled_event"),
                    at.index(out.len()),
                ));
                out.push(event.clone());
            }
            None => dropped.push(index),
        }
    }
    for (from, to) in moves {
        told.moved(from, to);
    }
    *events = out;
    dropped
}
/// `spawn_entity`: every entry gets the `rider_count` filter.
fn spawn_entity_113(spawn: &mut Value, at: &JsonPath, told: &mut Told) {
    const RIDERS: &str =
        r#"{"test": "rider_count", "subject": "self", "operator": "==", "value": 0}"#;
    let mut add = |entry: &mut VObject, entry_at: JsonPath| match entry.get_mut("filters") {
        None => {
            entry.insert("filters", json(RIDERS));
        }
        Some(filters) if filters.is_array() => {
            if let Some(list) = filters.as_array_mut() {
                list.push(json(RIDERS));
            }
        }
        Some(filters) => {
            let mut all = VArray::new();
            all.push(filters.clone());
            all.push(json(RIDERS));
            let mut group = VObject::new();
            group.insert("all_of", all);
            *filters = group.into();
            let filters_at = entry_at.child("filters");
            told.moved(filters_at.clone(), filters_at.child("all_of").index(0));
        }
    };
    if let Some(items) = spawn.as_array_mut() {
        for (index, item) in items.as_mut_slice().iter_mut().enumerate() {
            if let Some(object) = item.as_object_mut() {
                add(object, at.index(index));
            }
        }
    } else if let Some(object) = spawn.as_object_mut() {
        add(object, at.clone());
    }
}
