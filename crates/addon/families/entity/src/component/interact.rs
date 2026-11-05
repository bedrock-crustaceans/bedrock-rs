//! `interact`: what happens when a player interacts with the entity.
//!
//! Types and defaults follow the game's 1.26.50 `InteractionData.json` schema and its
//! sub-schemas. Below 1.26.50 the key is `Dynamic`, read by the game's untyped interact reader.

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::EventTrigger;
use bedrock_addon_core::versioning::ConvertMode;
use facet::Facet;
use facet_value::VObject;

use super::parse::{ComponentCheck, Typed};
use super::upgrade::one_or_many;

/// `minecraft:interact`: the interactions a player can have with the entity; the first whose
/// condition passes runs.
#[doc(alias = "minecraft:interact")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Interact {
    /// `interactions`: the interactions, in order.
    #[facet(default)]
    pub interactions: Vec<Interaction>,
}

/// One interaction of [`Interact`] (the game's interaction data).
#[allow(
    clippy::struct_excessive_bools,
    reason = "the game's flags, one member each"
)]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Interaction {
    /// `add_items`: items from a loot table added to the entity's inventory.
    pub add_items: Option<LootTable>,
    /// `admire`: the entity admires the item; default `false`.
    #[facet(default = false)]
    pub admire: bool,
    /// `barter`: the entity barters for the item; default `false`.
    #[facet(default = false)]
    pub barter: bool,
    /// `cooldown`: seconds before the entity can be interacted with again; default 0.
    #[facet(default = 0.0_f32)]
    pub cooldown: f32,
    /// `cooldown_after_being_attacked`: seconds after being attacked during which the
    /// interaction is unavailable; default 0.
    #[facet(default = 0.0_f32)]
    pub cooldown_after_being_attacked: f32,
    /// `drop_item_slot`: the slot whose item is dropped (a slot name or number as a string).
    pub drop_item_slot: Option<String>,
    /// `drop_item_y_offset`: the height above the entity the item drops from; default 0.
    #[facet(default = 0.0_f32)]
    pub drop_item_y_offset: f32,
    /// `equip_item_slot`: the slot the used item is equipped to (a slot name or number as a
    /// string).
    pub equip_item_slot: Option<String>,
    /// `give_item`: the item is given to the entity; default `false`.
    #[facet(default = false)]
    pub give_item: bool,
    /// `health_amount`: the health the entity gains; default 0.
    #[facet(default = 0_i32)]
    pub health_amount: i32,
    /// `hurt_item`: the durability the used item loses; default 0.
    #[facet(default = 0_i32)]
    pub hurt_item: i32,
    /// `interact_text`: the text of the touch-screen button; default empty.
    #[facet(default = String::new())]
    pub interact_text: String,
    /// `on_interact`: the event fired, and the condition (`filters`) for the interaction.
    pub on_interact: Option<EventTrigger>,
    /// `particle_on_start`: the particles shown when the interaction starts.
    pub particle_on_start: Option<InteractParticle>,
    /// `play_sounds`: the sounds played.
    #[facet(default)]
    pub play_sounds: Vec<String>,
    /// `repair_entity_item`: an item of the entity that is repaired.
    pub repair_entity_item: Option<RepairEntityItem>,
    /// `spawn_entities`: the entities spawned.
    #[facet(default)]
    pub spawn_entities: Vec<String>,
    /// `spawn_items`: items from a loot table spawned.
    pub spawn_items: Option<SpawnItems>,
    /// `swing`: the player's arm swings; default `true`.
    #[facet(default = true)]
    pub swing: bool,
    /// `take_item`: the entity takes the item; default `false`.
    #[facet(default = false)]
    pub take_item: bool,
    /// `transform_to_item`: the item the used item becomes (`name:aux`); default none (empty).
    #[facet(default = String::new())]
    pub transform_to_item: String,
    /// `use_item`: the used item is consumed; default `false`.
    #[facet(default = false)]
    pub use_item: bool,
    /// `vibration`: the vibration emitted; default `entity_interact`.
    #[facet(default = InteractVibration::EntityInteract)]
    pub vibration: InteractVibration,
}

/// `add_items` of an [`Interaction`]: a loot table.
// facet fills a member the text leaves out from the struct's `Default`: `table` reads as empty.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct LootTable {
    /// `table`: the loot table path; required.
    pub table: String,
}

/// `spawn_items` of an [`Interaction`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SpawnItems {
    /// `table`: the loot table path; default none (empty).
    #[facet(default = String::new())]
    pub table: String,
    /// `y_offset`: the height above the entity the items spawn at; default 1.
    #[facet(default = 1.0_f32)]
    pub y_offset: f32,
}

/// `repair_entity_item` of an [`Interaction`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RepairEntityItem {
    /// `amount`: the durability restored; default 1.
    #[facet(default = 1_i32)]
    pub amount: i32,
    /// `slot`: the slot of the item repaired.
    pub slot: Option<String>,
}

/// `particle_on_start` of an [`Interaction`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct InteractParticle {
    /// `copper_event`: the copper effect shown; default `none`.
    #[facet(default = CopperEvent::None)]
    pub copper_event: CopperEvent,
    /// `particle_offset_towards_interactor`: the particles move towards the player; default
    /// `false`.
    #[facet(default = false)]
    pub particle_offset_towards_interactor: bool,
    /// `particle_type`: vanilla's legacy particle name (`smoke`, `heart`, …); default `none`.
    #[facet(default = "none".to_owned())]
    pub particle_type: String,
    /// `particle_y_offset`: the height above the entity; default 0.
    #[facet(default = 0.0_f32)]
    pub particle_y_offset: f32,
}

/// The copper effect of an [`InteractParticle`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
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
    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
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

    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        let at = JsonPath::new().child("interactions");
        for (i, interaction) in self.interactions.iter().enumerate() {
            if let Some(trigger) = &interaction.on_interact {
                cx.extend(trigger.check(&at.index(i).child("on_interact")));
            }
        }
    }
}
