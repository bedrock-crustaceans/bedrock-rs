//! Wear and repair: `damage_absorption`, `durability`, `durability_sensor`, `dyeable`,
//! `enchantable`, `repairable`, `wearable`.

use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::molang::{Molang, MolangFields, MolangVisitor};
use bedrock_addon_core::shared::{IntRange, ItemDescriptor};
use facet::Facet;

use super::{FORMAT_VERSION_MOLANG, visit_item_descriptor};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;

/// `minecraft:damage_absorption`: worn armour absorbs damage of these causes, spending
/// durability.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct DamageAbsorption {
    /// The damage causes (JSON `absorbable_causes`, required, at least one).
    pub absorbable_causes: Vec<String>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:durability`: how much use the item takes before it breaks.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Durability {
    /// The durability, at least 0 (JSON `max_durability`, required).
    pub max_durability: i32,
    /// The chance in percent, 0–100, that a use spends durability (JSON `damage_chance`;
    /// default 100).
    #[facet(default)]
    pub damage_chance: Option<IntRange>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Durability {
    /// A durability of `max_durability`.
    pub fn new(max_durability: i32) -> Self {
        Self {
            max_durability,
            damage_chance: None,
            extra: Extra::default(),
        }
    }
}

/// `minecraft:durability_sensor`: particles and sounds when the durability falls to a
/// threshold.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct DurabilitySensor {
    /// The thresholds (JSON `durability_thresholds`, required, at least one).
    pub durability_thresholds: Vec<DurabilityThreshold>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One threshold of a [`DurabilitySensor`].
///
/// A member the document leaves out stays unset (and is not written); the game gives it the
/// value the constants and accessors below state.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DurabilityThreshold {
    /// The durability at which it fires (JSON `durability`).
    #[facet(default)]
    pub durability: i32,
    /// The particle shown (JSON `particle_type`; unset: the particle type value
    /// [`DEFAULT_PARTICLE_TYPE`](Self::DEFAULT_PARTICLE_TYPE)).
    #[facet(default)]
    pub particle_type: Option<String>,
    /// The sound played (JSON `sound_event`): a level sound event's name below 1.26.50, any
    /// sound name from 1.26.50 (unset: the empty name).
    #[facet(default)]
    pub sound_event: Option<SoundEventName>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DurabilityThreshold {
    /// The particle type value the game gives an unset `particle_type`
    /// when it builds a threshold; the name of that value is not known, so it stays a number.
    pub const DEFAULT_PARTICLE_TYPE: u32 = 0;

    /// The sound name the game uses: the written one, else the empty name.
    pub fn effective_sound_event(&self) -> &str {
        self.sound_event.as_ref().map_or("", |name| name.as_str())
    }
}

/// `minecraft:dyeable`: the item can be dyed in a cauldron.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Dyeable {
    /// The colour before dyeing (JSON `default_color`: a `#RRGGBB` string or `[r, g, b]`), as
    /// written.
    #[facet(default)]
    pub default_color: Option<Value>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:enchantable`: which enchantments the item takes and how well.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Enchantable {
    /// The enchantability, at least 0 (JSON `value`, required).
    pub value: i32,
    /// The enchant slot (JSON `slot`, required); an unknown name is read as `none`.
    pub slot: String,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The enchant slot names vanilla's `slot` regex accepts (`EnchantSlot`).
// `enchantable` constrains `slot` with a regular expression; these are its alternatives, in
// the order the game's schema spells the pattern.
pub(crate) const ENCHANT_SLOTS: &[&str] = &[
    "none",
    "all",
    "g_armor",
    "armor_head",
    "armor_torso",
    "armor_feet",
    "armor_legs",
    "sword",
    "bow",
    "spear",
    "crossbow",
    "melee_spear",
    "g_tool",
    "hoe",
    "shears",
    "flintsteel",
    "shield",
    "g_digging",
    "axe",
    "pickaxe",
    "shovel",
    "fishing_rod",
    "carrot_stick",
    "elytra",
    "cosmetic_head",
];

/// `minecraft:repairable`: the items that repair this one, and by how much.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Repairable {
    /// The repair entries (JSON `repair_items`).
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub repair_items: Vec<RepairItem>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One entry of a [`Repairable`].
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct RepairItem {
    /// The items that repair (JSON `items`, required).
    pub items: Vec<ItemDescriptor>,
    /// The durability restored (JSON `repair_amount`): a number, a Molang string or
    /// `{"expression", "version"}`, compiled at the item's version.
    #[facet(default)]
    pub repair_amount: Option<Molang>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MolangFields for Repairable {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter("repair_items".into());
        for (i, entry) in self.repair_items.iter_mut().enumerate() {
            visitor.enter(i.into());
            visitor.enter("items".into());
            for (j, item) in entry.items.iter_mut().enumerate() {
                visitor.enter(j.into());
                visit_item_descriptor(item, visitor);
                visitor.leave();
            }
            visitor.leave();
            visitor.field(
                "repair_amount",
                &FORMAT_VERSION_MOLANG,
                &mut entry.repair_amount,
            );
            visitor.leave();
        }
        visitor.leave();
    }
}

/// `minecraft:wearable`: the item is worn in `slot`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Wearable {
    /// The slot (JSON `slot`, required).
    pub slot: WearableSlot,
    /// The armour points, at least 0 (JSON `protection`; default 0).
    #[facet(default)]
    pub protection: i32,
    /// The wearer is hidden from the locator bar (JSON `hides_player_location`, from 1.21.90).
    #[facet(default)]
    pub hides_player_location: Option<bool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Wearable {
    /// Worn in `slot`, no protection.
    pub fn new(slot: WearableSlot) -> Self {
        Self {
            slot,
            protection: 0,
            hides_player_location: None,
            extra: Extra::default(),
        }
    }
}

/// The slot a [`Wearable`] goes in.
// The five-value "Wearable Slot" enum
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
#[repr(u8)]
#[non_exhaustive]
pub enum WearableSlot {
    /// The off hand (JSON `slot.weapon.offhand`).
    #[facet(rename = "slot.weapon.offhand")]
    Offhand,
    /// The head (JSON `slot.armor.head`).
    #[default]
    #[facet(rename = "slot.armor.head")]
    Head,
    /// The chest (JSON `slot.armor.chest`).
    #[facet(rename = "slot.armor.chest")]
    Chest,
    /// The legs (JSON `slot.armor.legs`).
    #[facet(rename = "slot.armor.legs")]
    Legs,
    /// The feet (JSON `slot.armor.feet`).
    #[facet(rename = "slot.armor.feet")]
    Feet,
}

impl WearableSlot {
    /// The JSON spelling.
    pub fn name(self) -> &'static str {
        match self {
            Self::Offhand => "slot.weapon.offhand",
            Self::Head => "slot.armor.head",
            Self::Chest => "slot.armor.chest",
            Self::Legs => "slot.armor.legs",
            Self::Feet => "slot.armor.feet",
        }
    }
}
