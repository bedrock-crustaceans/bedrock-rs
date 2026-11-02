//! Format 1.15.0: the struct and the step to 1.16.0, all section A.

super::legacy_struct!(1, 15, 0);

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};
use facet_value::VObject;

use super::ops::{Members, Told, below, components, has_bits, is, json, section_a, sections};

/// 1.15.0 → 1.16.0 (gate 1.16.0).
impl Upgrade for Entity {
    type Next = super::v1_16_0::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        if section_a(&payload, cx) {
            let mut told = Told::default();
            instant_despawn(&mut payload, &mut told);
            told.report(cx);
            if eligible(cx) {
                default_despawn(&mut payload, cx);
                set_persistent(&mut payload);
            }
            let components = components(&mut payload);
            if let Some(breathable) = components.object("minecraft:breathable") {
                breathable.insert_absent("breathes_lava", false.into());
            }
            if let Some(panic) = components.object("minecraft:behavior.panic") {
                panic.insert_absent("damage_sources", json(r#"["fire", "fire_tick", "magma"]"#));
            }
        }
        super::v1_16_0::Entity(payload)
    }
}

use ActorType as T;

/// Every section: a
/// `despawn` that is `{}`, `[]` or holds only `remove_child_entities` becomes
/// `instant_despawn {remove_child_entities: <copied, else false>}`.
fn instant_despawn(payload: &mut Payload, told: &mut Told) {
    for section in sections(payload) {
        let members = section.members;
        let Some(despawn) = members.get("minecraft:despawn") else {
            continue;
        };
        let child = match (despawn.as_object(), despawn.as_array()) {
            (Some(object), _) if object.is_empty() => None,
            (Some(object), _)
                if object.len() == 1 && object.contains_key("remove_child_entities") =>
            {
                object.get("remove_child_entities").cloned()
            }
            (None, Some(array)) if array.is_empty() => None,
            _ => continue,
        };
        members.take("minecraft:despawn");
        let mut instant = VObject::new();
        instant.insert(
            "remove_child_entities",
            child.unwrap_or_else(|| false.into()),
        );
        let instant = Value::from(instant);
        // An `instant_despawn` the file has beside the `despawn` is written over, and noted;
        // what is told of the new component is told of the old one.
        let to = below(&section.at, &["minecraft:instant_despawn"]);
        if members
            .get("minecraft:instant_despawn")
            .is_some_and(|had| !had.is_null() && *had != instant)
        {
            told.dropped(to.clone());
        }
        members.set("minecraft:instant_despawn", instant);
        told.moved(below(&section.at, &["minecraft:despawn"]), to);
    }
}

/// The types the two type ops apply to: monsters, animals and pathfinder mobs by their type
/// bits, and the bat; not the player, the NPC, the armor stand, the bee, the piglins and the
/// allay.
fn eligible(cx: &UpgradeCx<'_>) -> bool {
    let excluded = is(
        cx,
        &[
            T::Mob,
            T::Player,
            T::Npc,
            T::ArmorStand,
            T::Bee,
            T::Piglin,
            T::PiglinBrute,
            T::Allay,
        ],
    );
    !excluded
        && (has_bits(cx, 0xb00)
            || has_bits(cx, 0x1300)
            || has_bits(cx, 0x300)
            || cx.actor_type == T::Bat)
}

/// Entries of the default despawn filters.
const NOT_PERSISTENT: &str = r#"{"test": "is_persistent", "value": false}"#;
/// Entries of the default despawn filters.
const INACTIVE: &str = r#"{"test": "inactivity_timer", "subject": "self", "value": 600}"#;
/// Entries of the default despawn filters.
const RANDOM: &str = r#"{"test": "random_chance", "value": 50}"#;
/// Entries of the default despawn filters.
const FAR: &str = r#"{"test": "distance_to_nearest_player", "operator": ">", "value": 54}"#;
/// Entries of the default despawn filters.
const DAY: &str = r#"{"test": "is_daytime", "value": true}"#;
/// The default `despawn` of an eligible type without `persistent` or `despawn` in
/// `components`.
fn default_despawn(payload: &mut Payload, cx: &UpgradeCx<'_>) {
    const KEPT: &[ActorType] = &[
        T::IronGolem,
        T::SnowGolem,
        T::WanderingTrader,
        T::ElderGuardian,
        T::EnderDragon,
        T::Shulker,
        T::Vindicator,
        T::EvocationIllager,
        T::Wither,
        T::Villager,
        T::VillagerV2,
    ];
    if is(cx, KEPT) {
        return;
    }
    let components = components(payload);
    if components.has("minecraft:persistent") || components.has("minecraft:despawn") {
        return;
    }
    let entries: Vec<&str> = if is(cx, &[T::Pufferfish, T::Salmon, T::Tropicalfish, T::Cod]) {
        vec![
            NOT_PERSISTENT,
            r#"{"test": "distance_to_nearest_player", "operator": ">", "value": 40}"#,
        ]
    } else if cx.actor_type == T::Drowned {
        vec![NOT_PERSISTENT, DAY, INACTIVE, RANDOM, FAR]
    } else if has_bits(cx, 0xb00) {
        vec![
            NOT_PERSISTENT,
            r#"{"test": "light_level", "operator": ">=", "value": 8}"#,
            DAY,
            INACTIVE,
            RANDOM,
            FAR,
            r#"{"test": "surface_mob", "value": true}"#,
        ]
    } else {
        vec![
            NOT_PERSISTENT,
            r#"{"test": "light_level", "operator": "<", "value": 8}"#,
            INACTIVE,
            RANDOM,
            FAR,
        ]
    };
    let all_of = format!("[{}]", entries.join(", "));
    components.set(
        "minecraft:despawn",
        json(&format!(r#"{{"filters": [{{"all_of": {all_of}}}]}}"#)),
    );
}

/// `set_persistent: true` when absent in the attack goals of every section
/// (the fixer's 1.16 persistence rule).
fn set_persistent(payload: &mut Payload) {
    const GOALS: [&str; 7] = [
        "minecraft:behavior.leap_at_target",
        "minecraft:behavior.melee_attack",
        "minecraft:behavior.nearest_attackable_target",
        "minecraft:behavior.nearest_prioritized_attackable_target",
        "minecraft:behavior.ranged_attack",
        "minecraft:behavior.slime_attack",
        "minecraft:behavior.stalk_and_pounce_on_target",
    ];
    for section in sections(payload) {
        for goal in GOALS {
            if let Some(object) = section.members.object(goal) {
                object.insert_absent("set_persistent", true.into());
            }
        }
    }
}
