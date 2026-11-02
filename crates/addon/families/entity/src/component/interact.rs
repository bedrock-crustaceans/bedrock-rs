//! `interact`: what happens when a player interacts with the entity.
//!
//! Types and defaults follow the game's 1.26.50 `InteractionData.json` schema and its
//! sub-schemas. Below 1.26.50 the key is `Dynamic`, read by the game's untyped interact reader.

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, StrictString, Value};
use bedrock_addon_core::shared::EventTrigger;
use bedrock_addon_core::versioning::ConvertMode;
use facet::Facet;
use facet_value::VObject;

use super::parse::{Check, Typed, index, join, path};
use super::upgrade::one_or_many;

/// `minecraft:interact`: the interactions a player can have with the entity; the first whose
/// condition passes runs.
#[doc(alias = "minecraft:interact")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Interact {
    /// `interactions`: the interactions, in order.
    #[facet(default)]
    pub interactions: Vec<Interaction>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One interaction of [`Interact`] (the game's interaction data).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Interaction {
    /// `add_items`: items from a loot table added to the entity's inventory.
    #[facet(default)]
    pub add_items: Option<LootTable>,
    /// `admire`: the entity admires the item; default `false`.
    #[facet(default = StrictBool(false))]
    pub admire: StrictBool,
    /// `barter`: the entity barters for the item; default `false`.
    #[facet(default = StrictBool(false))]
    pub barter: StrictBool,
    /// `cooldown`: seconds before the entity can be interacted with again; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub cooldown: StrictFloat<f32>,
    /// `cooldown_after_being_attacked`: seconds after being attacked during which the
    /// interaction is unavailable; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub cooldown_after_being_attacked: StrictFloat<f32>,
    /// `drop_item_slot`: the slot whose item is dropped (a slot name or number as a string).
    #[facet(default)]
    pub drop_item_slot: Option<StrictString>,
    /// `drop_item_y_offset`: the height above the entity the item drops from; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub drop_item_y_offset: StrictFloat<f32>,
    /// `equip_item_slot`: the slot the used item is equipped to (a slot name or number as a
    /// string).
    #[facet(default)]
    pub equip_item_slot: Option<StrictString>,
    /// `give_item`: the item is given to the entity; default `false`.
    #[facet(default = StrictBool(false))]
    pub give_item: StrictBool,
    /// `health_amount`: the health the entity gains; default 0.
    #[facet(default = StrictInt(0_i32))]
    pub health_amount: StrictInt<i32>,
    /// `hurt_item`: the durability the used item loses; default 0.
    #[facet(default = StrictInt(0_i32))]
    pub hurt_item: StrictInt<i32>,
    /// `interact_text`: the text of the touch-screen button; default empty.
    #[facet(default = StrictString(String::new()))]
    pub interact_text: StrictString,
    /// `on_interact`: the event fired, and the condition (`filters`) for the interaction.
    #[facet(default)]
    pub on_interact: Option<EventTrigger>,
    /// `particle_on_start`: the particles shown when the interaction starts.
    #[facet(default)]
    pub particle_on_start: Option<InteractParticle>,
    /// `play_sounds`: the sounds played.
    #[facet(default)]
    pub play_sounds: Vec<StrictString>,
    /// `repair_entity_item`: an item of the entity that is repaired.
    #[facet(default)]
    pub repair_entity_item: Option<RepairEntityItem>,
    /// `spawn_entities`: the entities spawned.
    #[facet(default)]
    pub spawn_entities: Vec<StrictString>,
    /// `spawn_items`: items from a loot table spawned.
    #[facet(default)]
    pub spawn_items: Option<SpawnItems>,
    /// `swing`: the player's arm swings; default `true`.
    #[facet(default = StrictBool(true))]
    pub swing: StrictBool,
    /// `take_item`: the entity takes the item; default `false`.
    #[facet(default = StrictBool(false))]
    pub take_item: StrictBool,
    /// `transform_to_item`: the item the used item becomes (`name:aux`); default none (empty).
    #[facet(default = StrictString(String::new()))]
    pub transform_to_item: StrictString,
    /// `use_item`: the used item is consumed; default `false`.
    #[facet(default = StrictBool(false))]
    pub use_item: StrictBool,
    /// `vibration`: the vibration emitted; default `entity_interact`.
    #[facet(default = InteractVibration::EntityInteract)]
    pub vibration: InteractVibration,
    /// Members the interaction does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for Interaction {
    fn default() -> Self {
        Self {
            add_items: None,
            admire: StrictBool(false),
            barter: StrictBool(false),
            cooldown: StrictFloat(0.0),
            cooldown_after_being_attacked: StrictFloat(0.0),
            drop_item_slot: None,
            drop_item_y_offset: StrictFloat(0.0),
            equip_item_slot: None,
            give_item: StrictBool(false),
            health_amount: StrictInt(0),
            hurt_item: StrictInt(0),
            interact_text: StrictString::default(),
            on_interact: None,
            particle_on_start: None,
            play_sounds: Vec::new(),
            repair_entity_item: None,
            spawn_entities: Vec::new(),
            spawn_items: None,
            swing: StrictBool(true),
            take_item: StrictBool(false),
            transform_to_item: StrictString::default(),
            use_item: StrictBool(false),
            vibration: InteractVibration::EntityInteract,
            extra: Extra::default(),
        }
    }
}

/// `add_items` of an [`Interaction`]: a loot table.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LootTable {
    /// `table`: the loot table path; required.
    pub table: StrictString,
    /// Members the object does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `spawn_items` of an [`Interaction`].
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SpawnItems {
    /// `table`: the loot table path; default none (empty).
    #[facet(default = StrictString(String::new()))]
    pub table: StrictString,
    /// `y_offset`: the height above the entity the items spawn at; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub y_offset: StrictFloat<f32>,
    /// Members the object does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for SpawnItems {
    fn default() -> Self {
        Self {
            table: StrictString::default(),
            y_offset: StrictFloat(1.0),
            extra: Extra::default(),
        }
    }
}

/// `repair_entity_item` of an [`Interaction`].
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct RepairEntityItem {
    /// `amount`: the durability restored; default 1.
    #[facet(default = StrictInt(1_i32))]
    pub amount: StrictInt<i32>,
    /// `slot`: the slot of the item repaired.
    #[facet(default)]
    pub slot: Option<StrictString>,
    /// Members the object does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for RepairEntityItem {
    fn default() -> Self {
        Self {
            amount: StrictInt(1),
            slot: None,
            extra: Extra::default(),
        }
    }
}

/// `particle_on_start` of an [`Interaction`].
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct InteractParticle {
    /// `copper_event`: the copper effect shown; default `none`.
    #[facet(default = CopperEvent::None)]
    pub copper_event: CopperEvent,
    /// `particle_offset_towards_interactor`: the particles move towards the player; default
    /// `false`.
    #[facet(default = StrictBool(false))]
    pub particle_offset_towards_interactor: StrictBool,
    /// `particle_type`: vanilla's legacy particle name (`smoke`, `heart`, …); default `none`.
    #[facet(default = StrictString("none".to_owned()))]
    pub particle_type: StrictString,
    /// `particle_y_offset`: the height above the entity; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub particle_y_offset: StrictFloat<f32>,
    /// Members the object does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for InteractParticle {
    fn default() -> Self {
        Self {
            copper_event: CopperEvent::None,
            particle_offset_towards_interactor: StrictBool(false),
            particle_type: StrictString("none".to_owned()),
            particle_y_offset: StrictFloat(0.0),
            extra: Extra::default(),
        }
    }
}

/// The copper effect of an [`InteractParticle`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
#[repr(u8)]
#[non_exhaustive]
pub enum CopperEvent {
    /// None (JSON `none`).
    #[default]
    #[facet(rename = "none")]
    None,
    /// Waxing (JSON `wax_on`).
    #[facet(rename = "wax_on")]
    WaxOn,
    /// Removing wax (JSON `wax_off`).
    #[facet(rename = "wax_off")]
    WaxOff,
    /// Scraping oxidation (JSON `scrape`).
    #[facet(rename = "scrape")]
    Scrape,
}

/// The vibration an [`Interaction`] emits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
#[repr(u8)]
#[non_exhaustive]
pub enum InteractVibration {
    /// None (JSON `none`).
    #[facet(rename = "none")]
    None,
    /// The entity acted (JSON `entity_act`).
    #[facet(rename = "entity_act")]
    EntityAct,
    /// The entity died (JSON `entity_die`).
    #[facet(rename = "entity_die")]
    EntityDie,
    /// The entity was interacted with (JSON `entity_interact`).
    #[default]
    #[facet(rename = "entity_interact")]
    EntityInteract,
    /// The entity was sheared (JSON `shear`).
    #[facet(rename = "shear")]
    Shear,
}

impl Typed for Interact {
    /// The lenient conversion: the legacy parser's single interaction object, and its single
    /// string for `play_sounds` and `spawn_entities`, become one-element arrays, as vanilla's own
    /// 1.26.50 files write them.
    // The crate's own reading, taken from the game's packs (`"play_sounds": "eat"` below
    // 1.26.50, arrays from it).
    fn prepare(json: &mut VObject, cx: &mut Check<'_>) -> bool {
        if !matches!(cx.mode(), ConvertMode::Lenient) {
            return false;
        }
        let mut changed = false;
        if let Some(interactions) = json.get_mut("interactions") {
            // The legacy single interaction object.
            if interactions.is_object() {
                changed |= one_or_many(interactions);
            }
            if let Some(list) = interactions.as_array_mut() {
                for object in list.iter_mut().filter_map(Value::as_object_mut) {
                    for key in ["play_sounds", "spawn_entities"] {
                        if let Some(value) = object.get_mut(key)
                            && value.is_string()
                        {
                            changed |= one_or_many(value);
                        }
                    }
                }
            }
        }
        changed
    }

    fn check(&mut self, cx: &mut Check<'_>) {
        let at = path(&["interactions"]);
        for (i, interaction) in self.interactions.iter().enumerate() {
            if let Some(trigger) = &interaction.on_interact {
                cx.extend(trigger.check(&join(&index(&at, i), "on_interact")));
            }
        }
    }
}
