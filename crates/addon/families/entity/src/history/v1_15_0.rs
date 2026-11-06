//! Format 1.15.0: the struct and the step to 1.16.0, all section A.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};
use facet_value::VObject;

use super::Payload;
use super::fragments;
use super::ops::{Members, Told, components, has_bits, is, section_a, sections};

/// An entity document as written in format 1.15.0 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 15, 0>;

/// 1.15.0 → 1.16.0 (gate 1.16.0).
impl LegacyStep<1, 15, 0> for Payload {
    type Next = super::v1_16_0::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
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
                panic.insert_absent("damage_sources", fragments::panic_damage_sources());
            }
        }
        Legacy(payload)
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
        let to = section.at.child("minecraft:instant_despawn");
        if members
            .get("minecraft:instant_despawn")
            .is_some_and(|had| !had.is_null() && *had != instant)
        {
            told.dropped(to.clone());
        }
        members.set("minecraft:instant_despawn", instant);
        told.moved(section.at.child("minecraft:despawn"), to);
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
    let entries = if is(cx, &[T::Pufferfish, T::Salmon, T::Tropicalfish, T::Cod]) {
        vec![
            fragments::not_persistent(),
            fragments::fish_far_from_players(),
        ]
    } else if cx.actor_type == T::Drowned {
        vec![
            fragments::not_persistent(),
            fragments::is_daytime(),
            fragments::inactive(),
            fragments::random_chance_50(),
            fragments::far_from_players(),
        ]
    } else if has_bits(cx, 0xb00) {
        vec![
            fragments::not_persistent(),
            fragments::light_at_least_8(),
            fragments::is_daytime(),
            fragments::inactive(),
            fragments::random_chance_50(),
            fragments::far_from_players(),
            fragments::surface_mob(),
        ]
    } else {
        vec![
            fragments::not_persistent(),
            fragments::light_below_8(),
            fragments::inactive(),
            fragments::random_chance_50(),
            fragments::far_from_players(),
        ]
    };
    components.set("minecraft:despawn", fragments::despawn(entries));
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
