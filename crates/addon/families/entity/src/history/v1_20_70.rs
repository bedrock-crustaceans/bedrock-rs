//! Format 1.20.70: the struct and the step to 1.20.80.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::ops::{Members, section_a, section_b, sections};

/// An entity document as written in format 1.20.70 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 20, 70>;

/// 1.20.70 → 1.20.80 (gate 1.20.80).
impl LegacyStep<1, 20, 70> for Payload {
    type Next = super::v1_20_80::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        // A, all types: `description.spawn_category` when
        // absent; nothing when `description` is not an object.
        if section_a(&payload, cx)
            && let Some(description) = payload.description.as_object_mut()
        {
            description.insert_absent("spawn_category", spawn_category(cx.actor_type).into());
        }
        if section_b(&payload, cx) {
            for section in sections(&mut payload) {
                if let Some(interact) = section.members.object("minecraft:interact")
                    && let Some(interactions) = interact.get_mut("interactions")
                {
                    slots_as_strings(interactions);
                }
            }
        }
        Legacy(payload)
    }
}

/// The category vanilla's switch gives a type; `misc` by default.
fn spawn_category(actor: ActorType) -> &'static str {
    use ActorType as T;
    match actor {
        T::Bee
        | T::Allay
        | T::WanderingTrader
        | T::Chicken
        | T::Cow
        | T::Pig
        | T::Sheep
        | T::Mooshroom
        | T::Rabbit
        | T::PolarBear
        | T::Llama
        | T::TraderLlama
        | T::Turtle
        | T::Panda
        | T::Fox
        | T::Strider
        | T::Goat
        | T::Frog
        | T::Camel
        | T::Sniffer
        | T::Tadpole
        | T::Wolf
        | T::Ocelot
        | T::Parrot
        | T::Cat
        | T::Horse
        | T::Donkey
        | T::Mule
        | T::SkeletonHorse
        | T::ZombieHorse => "creature",
        T::Piglin
        | T::PiglinBrute
        | T::Creeper
        | T::Slime
        | T::Enderman
        | T::Ghast
        | T::Blaze
        | T::Witch
        | T::Guardian
        | T::ElderGuardian
        | T::EnderDragon
        | T::Shulker
        | T::Vindicator
        | T::Ravager
        | T::EvocationIllager
        | T::Vex
        | T::Pillager
        | T::Warden
        | T::Hoglin
        | T::ZombiePigman
        | T::Wither
        | T::Phantom
        | T::Zoglin
        | T::Zombie
        | T::ZombieVillager
        | T::ZombieVillagerV2
        | T::Husk
        | T::Drowned
        | T::Spider
        | T::Silverfish
        | T::CaveSpider
        | T::Endermite
        | T::Skeleton
        | T::Stray
        | T::WitherSkeleton
        | T::WitherSkull => "monster",
        T::Axolotl => "axolotls",
        T::Squid | T::Dolphin => "water_creature",
        T::Pufferfish | T::Salmon | T::Tropicalfish | T::Cod => "water_ambient",
        T::GlowSquid => "underground_water_creature",
        T::Bat => "ambient",
        _ => "misc",
    }
}

/// `interactions` (an object or an array): an integer `equip_item_slot` / `drop_item_slot`
/// becomes its decimal string.
fn slots_as_strings(interactions: &mut Value) {
    let fix = |object: &mut facet_value::VObject| {
        for key in ["equip_item_slot", "drop_item_slot"] {
            if let Some(slot) = object.get_mut(key)
                && let Some(n) = slot
                    .as_number()
                    .filter(|n| n.is_integer())
                    .and_then(facet_value::VNumber::to_i64)
            {
                *slot = n.to_string().into();
            }
        }
    };
    if let Some(list) = interactions.as_array_mut() {
        for item in list.as_mut_slice() {
            if let Some(object) = item.as_object_mut() {
                fix(object);
            }
        }
    } else if let Some(object) = interactions.as_object_mut() {
        fix(object);
    }
}
