//! Item and block goals: picking up, giving, eating, placing and carrying.

use bedrock_addon_core::diagnostic::{Diagnostics, VanillaText};
use bedrock_addon_core::molang::{Molang, MolangFields, MolangVisitor, Step};
use bedrock_addon_core::schema::{self, GoalMeta, Schema};
use bedrock_addon_core::shared::{
    BlockDescriptor, EventTrigger, Filter, FloatRange, IntRange, ItemDescriptor,
};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use facet::Facet;
use facet_value::VObject;

use super::check::{Bound, Goal, GoalCheck, V1_26_30};
use super::common::{
    ControlFlag, EntityTypes, LegacyConditions, PlaceStrategy, SearchStrategy, SoundEvent, Vec2,
    Vec3,
};
use super::convert;
use super::molang::{
    SUCCESS_CHANCE, SUCCESS_CHANCE_CONVERTED, push_trigger, push_triggers, visit_block,
    visit_blocks, visit_conditions, visit_items,
};

/// Vanilla's message for a non-positive `transport_items.search_distance`.
const SEARCH_DISTANCE: &str =
    "behavior.transport_items: \"search_distance\" may only contain positive values";

/// `minecraft:behavior.admire_item`: enables the mob to admire items that have been configured as admirable. Must be used in combination with the `admire_item` component.
// `behavior.admire_item`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.admire_item")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct AdmireItem {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `admire_item_sound`: the sound event to play when admiring the item. Default `""`.
    pub admire_item_sound: Option<SoundEvent>,
    /// `on_admire_item_start`: event to run when the mob starts admiring an item.
    pub on_admire_item_start: Option<EventTrigger>,
    /// `on_admire_item_stop`: event to run when the mob stops admiring an item.
    pub on_admire_item_stop: Option<EventTrigger>,
    /// `sound_interval`: the range of time in seconds to randomly wait before playing the sound again. Default 0 to 0.
    pub sound_interval: Option<FloatRange>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl AdmireItem {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `admire_item_sound`, or its default `""`.
    pub fn admire_item_sound(&self) -> &str {
        self.admire_item_sound.as_ref().map_or("", SoundEvent::name)
    }

    /// `sound_interval`, or its default 0 to 0.
    pub fn sound_interval(&self) -> (f32, f32) {
        self.sound_interval
            .map_or((0.0, 0.0), |range| range.resolve(0.0, 0.0))
    }
}

impl Goal for AdmireItem {
    const TYPED: FormatVersion = known::V1_26_20;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.sound("admire_item_sound", self.admire_item_sound.as_ref());
        c.trigger("on_admire_item_start", self.on_admire_item_start.as_ref());
        c.trigger("on_admire_item_stop", self.on_admire_item_stop.as_ref());
        c.range("sound_interval", &mut self.sound_interval);
    }
}

impl MolangFields for AdmireItem {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(
            &mut triggers,
            "on_admire_item_start",
            self.on_admire_item_start.as_ref(),
        );
        push_trigger(
            &mut triggers,
            "on_admire_item_stop",
            self.on_admire_item_stop.as_ref(),
        );
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.barter`: enables the mob to barter for items that have been configured as barter currency. Must be used in combination with the barter component.
// `behavior.barter`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.barter")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Barter {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for Barter {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.charge_held_item`: allows an entity to charge and use their held item.
// `behavior.charge_held_item`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.charge_held_item")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct ChargeHeldItem {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `items`: the list of items that can be used to charge the held item. This list is required and must have at least one item in it.
    pub items: Option<Vec<ItemDescriptor>>,
}

impl ChargeHeldItem {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }
}

impl Goal for ChargeHeldItem {
    const TYPED: FormatVersion = known::V1_26_40;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.items("items", self.items.as_deref());
        c.required("items", self.items.is_some());
        if self.items.as_ref().is_some_and(Vec::is_empty) {
            // The list "must have at least one item in it" (`minItems` 1).
            c.reject("items", None);
            self.items = None;
        }
    }
}

impl MolangFields for ChargeHeldItem {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_items(visitor, "items", &mut self.items);
    }
}

/// `minecraft:behavior.drink_milk`: allows the mob to drink milk based on specified environment conditions.
// `behavior.drink_milk`: typed from 1.21.120.
#[doc(alias = "minecraft:behavior.drink_milk")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct DrinkMilk {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown_seconds`: time (in seconds) that the goal is on cooldown before it can be used again. Default 5.
    pub cooldown_seconds: Option<f32>,
    /// `filters`: conditions that need to be met for the behavior to start.
    pub filters: Option<Filter>,
}

impl DrinkMilk {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `cooldown_seconds`, or its default 5.
    pub fn cooldown_seconds(&self) -> f32 {
        self.cooldown_seconds.unwrap_or(5.0)
    }
}

impl Goal for DrinkMilk {
    const TYPED: FormatVersion = known::V1_21_120;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.filter("filters", self.filters.as_ref());
    }
}

bedrock_addon_core::impl_no_molang!(DrinkMilk);

/// `minecraft:behavior.drink_potion`: allows the mob to drink potions based on specified environment conditions.
// `behavior.drink_potion`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.drink_potion")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DrinkPotion {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `potions`: a list of potions that this entity can drink. Each potion entry has the following parameters.
    pub potions: Option<Vec<Potion>>,
    /// `speed_modifier`: the movement speed modifier to apply to the entity while it is drinking a potion. A value of 0 represents no change in speed. Default 0.
    #[facet(schema::fallback = 0.0)]
    pub speed_modifier: Option<f32>,
}

impl Goal for DrinkPotion {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.each("potions", &mut self.potions, Potion::check);
        if c.lenient()
            && let Some(potions) = &mut self.potions
        {
            // The legacy goal parser logs a potion whose `id` (default −1) is negative as an
            // error and does not append it. It is told at the entry's index as written, the
            // list keeping the others.
            let mut index = 0;
            potions.retain(|potion| {
                let keep = potion.id.is_some_and(|id| id >= 0);
                if !keep {
                    let mut at = c.path("potions");
                    at.push_index(index);
                    c.legacy_error_at(at, "Drinkable potions must specify a valid potion id");
                }
                index += 1;
                keep
            });
        }
    }
}

/// `minecraft:behavior.drop_item_for`: allows the entity to move toward a target, and drop an item near the target. This goal requires a `minecraft:navigation` to execute.
// `behavior.drop_item_for`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.drop_item_for")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct DropItemFor {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `cooldown`: total time that the goal is on cooldown before it can be used again. Default 0.25; at least 0.
    pub cooldown: Option<f32>,
    /// `drop_item_chance`: the percent chance the entity will drop an item when using this goal. Default 1; 0 to 1.
    pub drop_item_chance: Option<f32>,
    /// `entity_types`: the list of conditions another entity must meet to be a valid target to drop an item for.
    pub entity_types: Option<EntityTypes>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5; greater than 0.
    pub goal_radius: Option<f32>,
    /// `loot_table`: the loot table that contains the possible loot the entity can drop with this goal. Default `""`.
    pub loot_table: Option<String>,
    /// `max_head_look_at_height`: the maximum height the entities head will look at when dropping the item. The entity will always be looking at its target. Default 1e+01; at least 0.
    pub max_head_look_at_height: Option<f32>,
    /// `minimum_teleport_distance`: if the target position is farther away than this distance on any tick, the entity will teleport to the target position. Default 2; at least 0.
    pub minimum_teleport_distance: Option<f32>,
    /// `offering_distance`: the preferred distance the entity tries to be from the target it is dropping an item for. Default 1; at least 0.
    pub offering_distance: Option<f32>,
    /// `on_drop_attempt`: the event to trigger when the entity attempts to drop an item.
    pub on_drop_attempt: Option<EventTrigger>,
    /// `search_count`: the number of randomly selected blocks each tick that the mob will check within its search range and height for a valid block to move to. A value of 0 will have the mob check every block within range in one tick. Default 0; at least 0.
    pub search_count: Option<i32>,
    /// `search_height`: the height in blocks the entity will search within to find a valid target position. Default 1; greater than 0.
    pub search_height: Option<i32>,
    /// `search_range`: the distance in blocks the entity will search within to find a valid target position. Default 0; greater than 0.
    pub search_range: Option<i32>,
    /// `seconds_before_pickup`: the numbers of seconds that will pass before the dropped entity can be picked up from the ground. Default 0; at least 0.
    pub seconds_before_pickup: Option<f32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this goal. Default 1; at least 0.
    pub speed_multiplier: Option<f32>,
    /// `target_range`: the range in blocks within which the entity searches to find a target to drop an item for. Default `[1, 1, 1]`.
    pub target_range: Option<Vec3>,
    /// `teleport_offset`: when the entity teleports, offset the teleport position by this many blocks in the X, Y, and Z coordinate. Default `[0, 1, 0]`.
    pub teleport_offset: Option<Vec3>,
    /// `time_of_day_range`: the valid times of day that this goal can be used. For reference: noon is 0.0, sunset is 0.25, midnight is 0.5, and sunrise is 0.75, and back to noon for 1.0. Default 0 to 1.
    pub time_of_day_range: Option<FloatRange>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl DropItemFor {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `cooldown`, or its default 0.25.
    pub fn cooldown(&self) -> f32 {
        self.cooldown.unwrap_or(0.25)
    }

    /// `drop_item_chance`, or its default 1.
    pub fn drop_item_chance(&self) -> f32 {
        self.drop_item_chance.unwrap_or(1.0)
    }

    /// `goal_radius`, or its default 0.5.
    pub fn goal_radius(&self) -> f32 {
        self.goal_radius.unwrap_or(0.5)
    }

    /// `loot_table`, or its default `""`.
    pub fn loot_table(&self) -> &str {
        self.loot_table.as_ref().map_or("", |v| v.as_str())
    }

    /// `max_head_look_at_height`, or its default 1e+01.
    pub fn max_head_look_at_height(&self) -> f32 {
        self.max_head_look_at_height.unwrap_or(1.0e1)
    }

    /// `minimum_teleport_distance`, or its default 2.
    pub fn minimum_teleport_distance(&self) -> f32 {
        self.minimum_teleport_distance.unwrap_or(2.0)
    }

    /// `offering_distance`, or its default 1.
    pub fn offering_distance(&self) -> f32 {
        self.offering_distance.unwrap_or(1.0)
    }

    /// `search_count`, or its default 0.
    // The goal's own default (the game's 1.26.10 schema), not the base shape's.
    pub fn search_count(&self) -> i32 {
        self.search_count.unwrap_or(0)
    }

    /// `search_height`, or its default 1.
    pub fn search_height(&self) -> i32 {
        self.search_height.unwrap_or(1)
    }

    /// `search_range`, or its default 0.
    pub fn search_range(&self) -> i32 {
        self.search_range.unwrap_or(0)
    }

    /// `seconds_before_pickup`, or its default 0.
    pub fn seconds_before_pickup(&self) -> f32 {
        self.seconds_before_pickup.unwrap_or(0.0)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.unwrap_or(1.0)
    }

    /// `target_range`, or its default `[1, 1, 1]`.
    pub fn target_range(&self) -> [f32; 3] {
        self.target_range
            .as_ref()
            .map_or([1.0, 1.0, 1.0], |v| [v.x, v.y, v.z])
    }

    /// `teleport_offset`, or its default `[0, 1, 0]`.
    pub fn teleport_offset(&self) -> [f32; 3] {
        self.teleport_offset
            .as_ref()
            .map_or([0.0, 1.0, 0.0], |v| [v.x, v.y, v.z])
    }

    /// `time_of_day_range`, or its default 0 to 1.
    pub fn time_of_day_range(&self) -> (f32, f32) {
        self.time_of_day_range
            .map_or((0.0, 1.0), |range| range.resolve(0.0, 1.0))
    }
}

impl Goal for DropItemFor {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.float("cooldown", &mut self.cooldown, Bound::Min(0.0));
        c.float(
            "drop_item_chance",
            &mut self.drop_item_chance,
            Bound::MinMax(0.0, 1.0),
        );
        c.entity_types("entity_types", self.entity_types.as_ref());
        c.float("goal_radius", &mut self.goal_radius, Bound::Above(0.0));
        c.float(
            "max_head_look_at_height",
            &mut self.max_head_look_at_height,
            Bound::Min(0.0),
        );
        c.float(
            "minimum_teleport_distance",
            &mut self.minimum_teleport_distance,
            Bound::Min(0.0),
        );
        c.float(
            "offering_distance",
            &mut self.offering_distance,
            Bound::Min(0.0),
        );
        c.trigger("on_drop_attempt", self.on_drop_attempt.as_ref());
        c.int("search_count", &mut self.search_count, Bound::Min(0));
        c.int("search_height", &mut self.search_height, Bound::Above(0));
        c.int("search_range", &mut self.search_range, Bound::Above(0));
        c.float(
            "seconds_before_pickup",
            &mut self.seconds_before_pickup,
            Bound::Min(0.0),
        );
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
        c.vector("target_range", self.target_range.as_ref().map(|v| &v.extra));
        c.vector(
            "teleport_offset",
            self.teleport_offset.as_ref().map(|v| &v.extra),
        );
        c.range("time_of_day_range", &mut self.time_of_day_range);
    }
}

impl MolangFields for DropItemFor {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(
            &mut triggers,
            "on_drop_attempt",
            self.on_drop_attempt.as_ref(),
        );
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.eat_block`: allows the entity to consume a block, replace the eaten block with another block, and trigger an event as a result.
// `behavior.eat_block`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.eat_block")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct EatBlock {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `eat_and_replace_block_pairs`: a collection of pairs of blocks; the first (`eat_block`) is the block the entity should eat, the second (`replace_block`) is the block that should replace the eaten block.
    pub eat_and_replace_block_pairs: Option<Vec<EatAndReplaceBlockPair>>,
    /// `on_eat`: the event to trigger when the block eating animation has completed.
    pub on_eat: Option<EventTrigger>,
    /// `success_chance`: a molang expression defining the success chance the entity has to consume a block.
    pub success_chance: Option<Molang>,
    /// `time_until_eat`: the amount of time (in seconds) it takes for the block to be eaten upon a successful eat attempt. Default 1.8.
    pub time_until_eat: Option<f32>,
    /// Read by the lenient conversion of a document older than the goal's typed version:
    /// a rejected `success_chance` is then a warning and the member takes its default.
    /// Not written.
    #[facet(skip, rename = "\u{0}converted")]
    pub converted: bool,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl EatBlock {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `time_until_eat`, or its default 1.8.
    pub fn time_until_eat(&self) -> f32 {
        self.time_until_eat.unwrap_or(1.8)
    }
}

impl Goal for EatBlock {
    const TYPED: FormatVersion = known::V1_26_40;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.each(
            "eat_and_replace_block_pairs",
            &mut self.eat_and_replace_block_pairs,
            EatAndReplaceBlockPair::check,
        );
        c.trigger("on_eat", self.on_eat.as_ref());
    }

    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        convert::success_chance(json, d);
    }

    fn lenient_value(&mut self) {
        self.converted = true;
    }
}

impl MolangFields for EatBlock {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let spec = if self.converted {
            &SUCCESS_CHANCE_CONVERTED
        } else {
            &SUCCESS_CHANCE
        };
        visitor.field("success_chance", spec, &mut self.success_chance);
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_eat", self.on_eat.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.eat_carried_item`: if the mob is carrying a food item, the mob will eat it and the effects will be applied to the mob.
// `behavior.eat_carried_item`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.eat_carried_item")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct EatCarriedItem {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `delay_before_eating`: time in seconds the mob should wait before eating the item. Default 0; at least 0.
    #[facet(schema::fallback = 0.0, schema::min = 0.0)]
    pub delay_before_eating: Option<f32>,
}

impl Goal for EatCarriedItem {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.equip_item`: the entity puts on the desired equipment.
// `behavior.equip_item`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.equip_item")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct EquipItem {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
}

impl Goal for EquipItem {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
    }
}

/// `minecraft:behavior.go_and_give_items_to_noteblock`: the entity will attempt to toss the items from its inventory to a nearby recently played noteblock.
// `behavior.go_and_give_items_to_noteblock`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.go_and_give_items_to_noteblock")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct GoAndGiveItemsToNoteblock {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `listen_time`: sets the time an entity should continue delivering items to a noteblock after hearing it. Default 30.
    pub listen_time: Option<i32>,
    /// `on_item_throw`: event(s) to run when this mob throws items.
    pub on_item_throw: Option<Vec<EventTrigger>>,
    /// `reach_block_distance`: sets the desired distance to be reached before throwing the items towards the block. Default 3.
    pub reach_block_distance: Option<f32>,
    /// `run_speed`: sets the entity's speed when running toward the block. Default 1.
    pub run_speed: Option<f32>,
    /// `throw_force`: sets the throw force. Default 0.2.
    pub throw_force: Option<f32>,
    /// `throw_sound`: sound to play when this mob throws an item. Default `charge`.
    pub throw_sound: Option<SoundEvent>,
    /// `vertical_throw_mul`: sets the vertical throw multiplier that is applied on top of the throw force in the vertical direction. Default 1.5.
    pub vertical_throw_mul: Option<f32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl GoAndGiveItemsToNoteblock {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `listen_time`, or its default 30.
    pub fn listen_time(&self) -> i32 {
        self.listen_time.unwrap_or(30)
    }

    /// `reach_block_distance`, or its default 3.
    pub fn reach_block_distance(&self) -> f32 {
        self.reach_block_distance.unwrap_or(3.0)
    }

    /// `run_speed`, or its default 1.
    pub fn run_speed(&self) -> f32 {
        self.run_speed.unwrap_or(1.0)
    }

    /// `throw_force`, or its default 0.2.
    pub fn throw_force(&self) -> f32 {
        self.throw_force.unwrap_or(0.2)
    }

    /// `throw_sound`, or its default `charge`.
    pub fn throw_sound(&self) -> &str {
        self.throw_sound.as_ref().map_or("charge", SoundEvent::name)
    }

    /// `vertical_throw_mul`, or its default 1.5.
    pub fn vertical_throw_mul(&self) -> f32 {
        self.vertical_throw_mul.unwrap_or(1.5)
    }
}

impl Goal for GoAndGiveItemsToNoteblock {
    const TYPED: FormatVersion = known::V1_26_20;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.triggers("on_item_throw", self.on_item_throw.as_deref());
        c.sound("throw_sound", self.throw_sound.as_ref());
    }
}

impl MolangFields for GoAndGiveItemsToNoteblock {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_triggers(
            &mut triggers,
            "on_item_throw",
            self.on_item_throw.as_deref(),
        );
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.go_and_give_items_to_owner`: the entity will attempt to toss the items from its inventory to its owner.
// `behavior.go_and_give_items_to_owner`: typed from 1.26.20.
#[doc(alias = "minecraft:behavior.go_and_give_items_to_owner")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct GoAndGiveItemsToOwner {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `on_item_throw`: event(s) to run when this mob throws items.
    pub on_item_throw: Option<Vec<EventTrigger>>,
    /// `reach_mob_distance`: sets the desired distance to be reached before giving items to owner. Default 3.
    pub reach_mob_distance: Option<f32>,
    /// `run_speed`: sets the entity's speed when running toward the owner. Default 1.
    pub run_speed: Option<f32>,
    /// `throw_force`: sets the throw force. Default 0.2.
    pub throw_force: Option<f32>,
    /// `throw_sound`: sound to play when this mob throws an item. Default `charge`.
    pub throw_sound: Option<SoundEvent>,
    /// `vertical_throw_mul`: sets the vertical throw multiplier that is applied on top of the throw force in the vertical direction. Default 1.5.
    pub vertical_throw_mul: Option<f32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl GoAndGiveItemsToOwner {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `reach_mob_distance`, or its default 3.
    pub fn reach_mob_distance(&self) -> f32 {
        self.reach_mob_distance.unwrap_or(3.0)
    }

    /// `run_speed`, or its default 1.
    pub fn run_speed(&self) -> f32 {
        self.run_speed.unwrap_or(1.0)
    }

    /// `throw_force`, or its default 0.2.
    pub fn throw_force(&self) -> f32 {
        self.throw_force.unwrap_or(0.2)
    }

    /// `throw_sound`, or its default `charge`.
    pub fn throw_sound(&self) -> &str {
        self.throw_sound.as_ref().map_or("charge", SoundEvent::name)
    }

    /// `vertical_throw_mul`, or its default 1.5.
    pub fn vertical_throw_mul(&self) -> f32 {
        self.vertical_throw_mul.unwrap_or(1.5)
    }
}

impl Goal for GoAndGiveItemsToOwner {
    const TYPED: FormatVersion = known::V1_26_20;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.triggers("on_item_throw", self.on_item_throw.as_deref());
        c.sound("throw_sound", self.throw_sound.as_ref());
    }
}

impl MolangFields for GoAndGiveItemsToOwner {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_triggers(
            &mut triggers,
            "on_item_throw",
            self.on_item_throw.as_deref(),
        );
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.offer_flower`: allows the mob to offer a flower to another mob with the `minecraft:take_flower` behavior.
// `behavior.offer_flower`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.offer_flower")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct OfferFlower {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `chance_to_start`: percent chance that the mob will start this goal from 0.0 to 1.0 (where 1.0 = 100%). Default 0.000125; 0 to 1.
    pub chance_to_start: Option<f32>,
    /// `filters`: conditions that need to be met for the behavior to start.
    pub filters: Option<Filter>,
    /// `max_head_rotation_y`: maximum rotation (in degrees), on the Y-axis, this entity can rotate its head while trying to look at the target. Default 3e+01; at least 0.
    pub max_head_rotation_y: Option<f32>,
    /// `max_offer_flower_duration`: the max amount of time (in seconds) that the mob will offer the flower for before exiting the Goal. Default 2e+01; at least 0.
    pub max_offer_flower_duration: Option<f32>,
    /// `max_rotation_x`: maximum rotation (in degrees), on the X-axis, this entity can rotate while trying to look at the target. Default 3e+01; at least 0.
    pub max_rotation_x: Option<f32>,
    /// `search_area`: the dimensions of the AABB used to search for a potential mob to offer flower to. Default `[6, 2, 6]`.
    pub search_area: Option<Vec3>,
}

impl OfferFlower {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `chance_to_start`, or its default 0.000125.
    pub fn chance_to_start(&self) -> f32 {
        self.chance_to_start.unwrap_or(0.000_125)
    }

    /// `max_head_rotation_y`, or its default 3e+01.
    pub fn max_head_rotation_y(&self) -> f32 {
        self.max_head_rotation_y.unwrap_or(3.0e1)
    }

    /// `max_offer_flower_duration`, or its default 2e+01.
    pub fn max_offer_flower_duration(&self) -> f32 {
        self.max_offer_flower_duration.unwrap_or(2.0e1)
    }

    /// `max_rotation_x`, or its default 3e+01.
    pub fn max_rotation_x(&self) -> f32 {
        self.max_rotation_x.unwrap_or(3.0e1)
    }

    /// `search_area`, or its default `[6, 2, 6]`.
    pub fn search_area(&self) -> [f32; 3] {
        self.search_area
            .as_ref()
            .map_or([6.0, 2.0, 6.0], |v| [v.x, v.y, v.z])
    }
}

impl Goal for OfferFlower {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.float(
            "chance_to_start",
            &mut self.chance_to_start,
            Bound::MinMax(0.0, 1.0),
        );
        c.filter("filters", self.filters.as_ref());
        c.float(
            "max_head_rotation_y",
            &mut self.max_head_rotation_y,
            Bound::Min(0.0),
        );
        c.float(
            "max_offer_flower_duration",
            &mut self.max_offer_flower_duration,
            Bound::Min(0.0),
        );
        c.float("max_rotation_x", &mut self.max_rotation_x, Bound::Min(0.0));
        c.vector("search_area", self.search_area.as_ref().map(|v| &v.extra));
    }
}

bedrock_addon_core::impl_no_molang!(OfferFlower);

/// `minecraft:behavior.pickup_items`: allows the mob to pick up items on the ground.
// `behavior.pickup_items`: typed from 1.26.20, redeclared 1.26.30.
#[doc(alias = "minecraft:behavior.pickup_items")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct PickupItems {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `can_pickup_any_item`: if true, the mob can pick up any item. Default `false`.
    pub can_pickup_any_item: Option<bool>,
    /// `can_pickup_to_hand_or_equipment`: if true, the mob can pick up items to its hand or armor slots. Default `true`.
    pub can_pickup_to_hand_or_equipment: Option<bool>,
    /// `cooldown_after_being_attacked`: time in seconds the mob cannot pickup items after being attacked. Default 0.
    pub cooldown_after_being_attacked: Option<i32>,
    /// `excluded_items`: list of items this mob will not pick up.
    pub excluded_items: Option<Vec<ItemDescriptor>>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5.
    pub goal_radius: Option<f32>,
    /// `max_dist`: maximum distance this mob will look for items to pick up. Default 1.
    pub max_dist: Option<i32>,
    /// `on_pickup_item_end`: event to run when this mob either finishes or loses interest in picking up an item.
    pub on_pickup_item_end: Option<EventTrigger>,
    /// `on_pickup_item_start`: event to run when this mob moves to pick up an item.
    pub on_pickup_item_start: Option<EventTrigger>,
    /// `pickup_based_on_chance`: if true, depending on the difficulty, there is a random chance that the mob may not be able to pickup items. Default `false`.
    pub pickup_based_on_chance: Option<bool>,
    /// `pickup_same_items_as_in_hand`: if true, the mob will only pick up items that match what's in their hand. Default `false`.
    pub pickup_same_items_as_in_hand: Option<bool>,
    /// `search_height`: height in blocks the mob will search for items. Default 1.
    pub search_height: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    pub speed_multiplier: Option<f32>,
    /// `stop_if_holding_item`: if true, the mob will not pick up another item if the item's preferred slot matches. Default `false`.
    pub stop_if_holding_item: Option<bool>,
    /// `track_target`: if true, this mob will chase after the target as long as it's a valid target. Default `false`.
    pub track_target: Option<bool>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl PickupItems {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `can_pickup_any_item`, or its default `false`.
    pub fn can_pickup_any_item(&self) -> bool {
        self.can_pickup_any_item.is_some_and(|v| v)
    }

    /// `can_pickup_to_hand_or_equipment`, or its default `true`.
    pub fn can_pickup_to_hand_or_equipment(&self) -> bool {
        self.can_pickup_to_hand_or_equipment.is_none_or(|v| v)
    }

    /// `cooldown_after_being_attacked`, or its default 0.
    pub fn cooldown_after_being_attacked(&self) -> i32 {
        self.cooldown_after_being_attacked.unwrap_or(0)
    }

    /// `goal_radius`, or its default 0.5.
    pub fn goal_radius(&self) -> f32 {
        self.goal_radius.unwrap_or(0.5)
    }

    /// `max_dist`, or its default 1.
    pub fn max_dist(&self) -> i32 {
        self.max_dist.unwrap_or(1)
    }

    /// `pickup_based_on_chance`, or its default `false`.
    pub fn pickup_based_on_chance(&self) -> bool {
        self.pickup_based_on_chance.is_some_and(|v| v)
    }

    /// `pickup_same_items_as_in_hand`, or its default `false`.
    pub fn pickup_same_items_as_in_hand(&self) -> bool {
        self.pickup_same_items_as_in_hand.is_some_and(|v| v)
    }

    /// `search_height`, or its default 1.
    pub fn search_height(&self) -> i32 {
        self.search_height.unwrap_or(1)
    }

    /// `speed_multiplier`, or its default 1.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.unwrap_or(1.0)
    }

    /// `stop_if_holding_item`, or its default `false`.
    pub fn stop_if_holding_item(&self) -> bool {
        self.stop_if_holding_item.is_some_and(|v| v)
    }

    /// `track_target`, or its default `false`.
    pub fn track_target(&self) -> bool {
        self.track_target.is_some_and(|v| v)
    }
}

impl Goal for PickupItems {
    const TYPED: FormatVersion = known::V1_26_20;

    const GATED: &'static [(&'static str, FormatVersion)] = &[("stop_if_holding_item", V1_26_30)];

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.items("excluded_items", self.excluded_items.as_deref());
        c.trigger("on_pickup_item_end", self.on_pickup_item_end.as_ref());
        c.trigger("on_pickup_item_start", self.on_pickup_item_start.as_ref());
    }

    fn forget(&mut self, name: &str) {
        if name == "stop_if_holding_item" {
            self.stop_if_holding_item = None;
        }
    }
}

impl MolangFields for PickupItems {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_items(visitor, "excluded_items", &mut self.excluded_items);
        let mut triggers = Vec::new();
        push_trigger(
            &mut triggers,
            "on_pickup_item_end",
            self.on_pickup_item_end.as_ref(),
        );
        push_trigger(
            &mut triggers,
            "on_pickup_item_start",
            self.on_pickup_item_start.as_ref(),
        );
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.place_block`: allows the entity to place a block.
// `behavior.place_block`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.place_block")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct PlaceBlock {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `affected_by_griefing_rule`: if true, whether the goal is affected by the mob griefing game rule. Default `true`.
    pub affected_by_griefing_rule: Option<bool>,
    /// `can_place`: filters for if the entity should try to place its block. Self and Target are set.
    pub can_place: Option<Filter>,
    /// `chance`: chance each tick for the entity to try and place a block. Default 1; 0 to 1.
    pub chance: Option<f32>,
    /// `on_place`: trigger ran if the entity does place its block. Self, Target, and Block are set.
    pub on_place: Option<EventTrigger>,
    /// `placeable_carried_blocks`: block descriptors for which blocks are valid to be placed from the entity's carried item, if empty all blocks are valid.
    pub placeable_carried_blocks: Option<Vec<BlockDescriptor>>,
    /// `randomly_placeable_blocks`: weighted block descriptors for which blocks should be randomly placed, if empty the entity will try to place its carried block from `placeable_carried_blocks`.
    pub randomly_placeable_blocks: Option<Vec<WeightedBlock>>,
    /// `xz_range`: xZ range from which the entity will try and place blocks in. Default -2 to 2.
    pub xz_range: Option<IntRange>,
    /// `y_range`: y range from which the entity will try and place blocks in. Default 0 to 3.
    pub y_range: Option<IntRange>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl PlaceBlock {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `affected_by_griefing_rule`, or its default `true`.
    pub fn affected_by_griefing_rule(&self) -> bool {
        self.affected_by_griefing_rule.is_none_or(|v| v)
    }

    /// `chance`, or its default 1.
    pub fn chance(&self) -> f32 {
        self.chance.unwrap_or(1.0)
    }

    /// `xz_range`, or its default -2 to 2.
    pub fn xz_range(&self) -> (i32, i32) {
        self.xz_range.map_or((-2, 2), |range| range.resolve(-2, 2))
    }

    /// `y_range`, or its default 0 to 3.
    pub fn y_range(&self) -> (i32, i32) {
        self.y_range.map_or((0, 3), |range| range.resolve(0, 3))
    }
}

impl Goal for PlaceBlock {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.filter("can_place", self.can_place.as_ref());
        c.float("chance", &mut self.chance, Bound::MinMax(0.0, 1.0));
        c.trigger("on_place", self.on_place.as_ref());
        c.blocks(
            "placeable_carried_blocks",
            self.placeable_carried_blocks.as_deref(),
        );
        c.each(
            "randomly_placeable_blocks",
            &mut self.randomly_placeable_blocks,
            WeightedBlock::check,
        );
        c.range("xz_range", &mut self.xz_range);
        c.range("y_range", &mut self.y_range);
    }
}

impl MolangFields for PlaceBlock {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_blocks(
            visitor,
            "placeable_carried_blocks",
            &mut self.placeable_carried_blocks,
        );
        visitor.nested(
            "randomly_placeable_blocks",
            &mut self.randomly_placeable_blocks,
        );
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_place", self.on_place.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.share_items`: allows the mob to give items it has to others.
// `behavior.share_items`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.share_items")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ShareItems {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    #[facet(schema::fallback = 0, schema::min = 0)]
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `entity_types`: list of entities this mob will share items with.
    pub entity_types: Option<EntityTypes>,
    /// `goal_radius`: distance in blocks within the mob considers it has reached the goal. This is the "wiggle room" to stop the AI from bouncing back and forth trying to reach a specific spot. Default 0.5.
    #[facet(schema::fallback = 0.5)]
    pub goal_radius: Option<f32>,
    /// `max_dist`: maximum distance in blocks this mob will look for entities to share items with. Default 1.
    #[facet(schema::fallback = 1)]
    pub max_dist: Option<i32>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 1.
    #[facet(schema::fallback = 1.0)]
    pub speed_multiplier: Option<f32>,
}

impl Goal for ShareItems {
    const TYPED: FormatVersion = <Self as GoalMeta>::TYPED;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.bounds(self);
        c.entity_types("entity_types", self.entity_types.as_ref());
    }
}

/// `minecraft:behavior.snacking`: allows the mob to take a load off and snack on food that it found nearby.
// `behavior.snacking`: typed from 1.26.40.
#[doc(alias = "minecraft:behavior.snacking")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct Snacking {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `items`: items that we are interested in snacking on.
    pub items: Option<Vec<ItemDescriptor>>,
    /// `snacking_cooldown`: the cooldown time in seconds before the mob is able to snack again. Default 7.5.
    pub snacking_cooldown: Option<f32>,
    /// `snacking_cooldown_min`: the minimum time in seconds before the mob is able to snack again. Default 0.5.
    pub snacking_cooldown_min: Option<f32>,
    /// `snacking_stop_chance`: this is the chance that the mob will stop snacking, from 0 to 1. Default 0.0016666667.
    pub snacking_stop_chance: Option<f32>,
}

impl Snacking {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `snacking_cooldown`, or its default 7.5.
    pub fn snacking_cooldown(&self) -> f32 {
        self.snacking_cooldown.unwrap_or(7.5)
    }

    /// `snacking_cooldown_min`, or its default 0.5.
    pub fn snacking_cooldown_min(&self) -> f32 {
        self.snacking_cooldown_min.unwrap_or(0.5)
    }

    /// `snacking_stop_chance`, or its default 0.0016666667.
    pub fn snacking_stop_chance(&self) -> f32 {
        self.snacking_stop_chance.unwrap_or(0.001_666_666_7)
    }
}

impl Goal for Snacking {
    const TYPED: FormatVersion = known::V1_26_40;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.items("items", self.items.as_deref());
    }
}

impl MolangFields for Snacking {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_items(visitor, "items", &mut self.items);
    }
}

/// `minecraft:behavior.take_block`: allows the entity to take a block from the world and carry it around.
// `behavior.take_block`: typed from 1.26.30.
#[doc(alias = "minecraft:behavior.take_block")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct TakeBlock {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `affected_by_griefing_rule`: if true, whether the goal is affected by the mob griefing game rule. Default `true`.
    pub affected_by_griefing_rule: Option<bool>,
    /// `blocks`: block descriptors for which blocks are valid to be taken by the entity, if empty all blocks are valid.
    pub blocks: Option<Vec<BlockDescriptor>>,
    /// `can_take`: filters for if the entity should try to take a block. Self and Target are set.
    pub can_take: Option<Filter>,
    /// `chance`: chance each tick for the entity to try and take a block. Default 1; 0 to 1.
    pub chance: Option<f32>,
    /// `on_take`: trigger ran if the entity does take a block. Self, Target, and Block are set.
    pub on_take: Option<EventTrigger>,
    /// `requires_line_of_sight`: if true, whether the entity needs line of sight to the block they are trying to take. Default `true`.
    pub requires_line_of_sight: Option<bool>,
    /// `xz_range`: xZ range from which the entity will try and take blocks from. Default -2 to 2.
    pub xz_range: Option<IntRange>,
    /// `y_range`: y range from which the entity will try and take blocks from. Default 0 to 3.
    pub y_range: Option<IntRange>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl TakeBlock {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `affected_by_griefing_rule`, or its default `true`.
    pub fn affected_by_griefing_rule(&self) -> bool {
        self.affected_by_griefing_rule.is_none_or(|v| v)
    }

    /// `chance`, or its default 1.
    pub fn chance(&self) -> f32 {
        self.chance.unwrap_or(1.0)
    }

    /// `requires_line_of_sight`, or its default `true`.
    pub fn requires_line_of_sight(&self) -> bool {
        self.requires_line_of_sight.is_none_or(|v| v)
    }

    /// `xz_range`, or its default -2 to 2.
    pub fn xz_range(&self) -> (i32, i32) {
        self.xz_range.map_or((-2, 2), |range| range.resolve(-2, 2))
    }

    /// `y_range`, or its default 0 to 3.
    pub fn y_range(&self) -> (i32, i32) {
        self.y_range.map_or((0, 3), |range| range.resolve(0, 3))
    }
}

impl Goal for TakeBlock {
    const TYPED: FormatVersion = known::V1_26_30;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.blocks("blocks", self.blocks.as_deref());
        c.filter("can_take", self.can_take.as_ref());
        c.float("chance", &mut self.chance, Bound::MinMax(0.0, 1.0));
        c.trigger("on_take", self.on_take.as_ref());
        c.range("xz_range", &mut self.xz_range);
        c.range("y_range", &mut self.y_range);
    }
}

impl MolangFields for TakeBlock {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_blocks(visitor, "blocks", &mut self.blocks);
        let mut triggers = Vec::new();
        push_trigger(&mut triggers, "on_take", self.on_take.as_ref());
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.take_flower`: allows the mob to accept flowers from another mob with the `minecraft:offer_flower` behavior.
// `behavior.take_flower`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.take_flower")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct TakeFlower {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `filters`: conditions that need to be met for the behavior to start.
    pub filters: Option<Filter>,
    /// `max_head_rotation_y`: maximum rotation (in degrees), on the Y-axis, this entity can rotate its head while trying to look at the target. Default 3e+01; greater than 0.
    pub max_head_rotation_y: Option<f32>,
    /// `max_rotation_x`: maximum rotation (in degrees), on the X-axis, this entity can rotate while trying to look at the target. Default 3e+01; greater than 0.
    pub max_rotation_x: Option<f32>,
    /// `max_wait_time`: the maximum amount of time (in seconds) for the mob to randomly wait for before taking the flower. Default 2e+01; at least 0.
    pub max_wait_time: Option<f32>,
    /// `min_distance_to_target`: minimum distance (in blocks) for the entity to be considered having reached its target. Default 2; greater than 0.
    pub min_distance_to_target: Option<f32>,
    /// `min_wait_time`: the minimum amount of time (in seconds) for the mob to randomly wait for before taking the flower. Default 4; at least 0.
    pub min_wait_time: Option<f32>,
    /// `on_take_flower`: event triggered when the entity takes a flower from another entity.
    pub on_take_flower: Option<EventTrigger>,
    /// `search_area`: the dimensions of the AABB used to search for a potential mob to take a flower from. Default `[6, 2, 6]`.
    pub search_area: Option<Vec3>,
    /// `speed_multiplier`: movement speed multiplier of the mob when using this AI Goal. Default 0.5; at least 0.
    pub speed_multiplier: Option<f32>,
    /// The legacy `condition`s of the goal's triggers, read for the Molang check;
    /// written back from each trigger's `extra`.
    #[doc(hidden)]
    #[facet(skip, rename = "\u{0}legacy_conditions")]
    pub legacy_conditions: LegacyConditions,
}

impl TakeFlower {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `max_head_rotation_y`, or its default 3e+01.
    pub fn max_head_rotation_y(&self) -> f32 {
        self.max_head_rotation_y.unwrap_or(3.0e1)
    }

    /// `max_rotation_x`, or its default 3e+01.
    pub fn max_rotation_x(&self) -> f32 {
        self.max_rotation_x.unwrap_or(3.0e1)
    }

    /// `max_wait_time`, or its default 2e+01.
    pub fn max_wait_time(&self) -> f32 {
        self.max_wait_time.unwrap_or(2.0e1)
    }

    /// `min_distance_to_target`, or its default 2.
    pub fn min_distance_to_target(&self) -> f32 {
        self.min_distance_to_target.unwrap_or(2.0)
    }

    /// `min_wait_time`, or its default 4.
    pub fn min_wait_time(&self) -> f32 {
        self.min_wait_time.unwrap_or(4.0)
    }

    /// `search_area`, or its default `[6, 2, 6]`.
    pub fn search_area(&self) -> [f32; 3] {
        self.search_area
            .as_ref()
            .map_or([6.0, 2.0, 6.0], |v| [v.x, v.y, v.z])
    }

    /// `speed_multiplier`, or its default 0.5.
    pub fn speed_multiplier(&self) -> f32 {
        self.speed_multiplier.unwrap_or(0.5)
    }
}

impl Goal for TakeFlower {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.filter("filters", self.filters.as_ref());
        c.float(
            "max_head_rotation_y",
            &mut self.max_head_rotation_y,
            Bound::Above(0.0),
        );
        c.float(
            "max_rotation_x",
            &mut self.max_rotation_x,
            Bound::Above(0.0),
        );
        c.float("max_wait_time", &mut self.max_wait_time, Bound::Min(0.0));
        c.float(
            "min_distance_to_target",
            &mut self.min_distance_to_target,
            Bound::Above(0.0),
        );
        c.float("min_wait_time", &mut self.min_wait_time, Bound::Min(0.0));
        c.trigger("on_take_flower", self.on_take_flower.as_ref());
        c.vector("search_area", self.search_area.as_ref().map(|v| &v.extra));
        c.float(
            "speed_multiplier",
            &mut self.speed_multiplier,
            Bound::Min(0.0),
        );
    }
}

impl MolangFields for TakeFlower {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let mut triggers = Vec::new();
        push_trigger(
            &mut triggers,
            "on_take_flower",
            self.on_take_flower.as_ref(),
        );
        visit_conditions(visitor, &mut self.legacy_conditions, &triggers);
    }
}

/// `minecraft:behavior.trade_with_player`: allows the player to trade with this mob. When the goal starts, it will stop the mob's navigation.
// `behavior.trade_with_player`: typed from 1.26.10.
#[doc(alias = "minecraft:behavior.trade_with_player")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct TradeWithPlayer {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `filters`: conditions that need to be met for the behavior to start.
    pub filters: Option<Filter>,
    /// `max_distance_from_player`: the max distance that the mob can be from the player before exiting the goal. Default 8; greater than 0.
    pub max_distance_from_player: Option<f32>,
}

impl TradeWithPlayer {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `max_distance_from_player`, or its default 8.
    pub fn max_distance_from_player(&self) -> f32 {
        self.max_distance_from_player.unwrap_or(8.0)
    }
}

impl Goal for TradeWithPlayer {
    const TYPED: FormatVersion = known::V1_26_10;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.filter("filters", self.filters.as_ref());
        c.float(
            "max_distance_from_player",
            &mut self.max_distance_from_player,
            Bound::Above(0.0),
        );
    }
}

bedrock_addon_core::impl_no_molang!(TradeWithPlayer);

/// `minecraft:behavior.transport_items`: a behavior that enables a mob to transport items from and to containers.
// `behavior.transport_items`: typed from 1.21.100.
#[doc(alias = "minecraft:behavior.transport_items")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct TransportItems {
    /// `priority`: the goal's priority; lower runs first. Default 0; at least 0.
    pub priority: Option<i32>,
    /// `control_flags`: what the goal claims while it runs (`move`, `look`, `jump`).
    pub control_flags: Option<Vec<ControlFlag>>,
    /// `allow_simultaneous_interaction`: when true the mob will wait until a container is not used by other entities before starting to interact with it. Default `false`.
    pub allow_simultaneous_interaction: Option<bool>,
    /// `allowed_items`: a list of item descriptors that are the only items the mob is allowed to transport. If this and `disallowed_items` are both empty, then all items are allowed. If non-empty `disallowed_items` must be empty.
    pub allowed_items: Option<Vec<ItemDescriptor>>,
    /// `destination_container_types`: a list of block descriptors that should be a container type to put items in.
    pub destination_container_types: Option<Vec<BlockDescriptor>>,
    /// `disallowed_items`: a list of item descriptors that are the mob is not allowed to transport. If non-empty `allowed_items` must be empty.
    pub disallowed_items: Option<Vec<ItemDescriptor>>,
    /// `idle_cooldown`: when the mob cannot find a container in which to get or put items, the goal will stop being active for this amount of time in seconds. Default 20.
    pub idle_cooldown: Option<u32>,
    /// `initial_cooldown`: how long the mob will wait after spawning or getting the goal added before the goal can start. Default 0.
    pub initial_cooldown: Option<u32>,
    /// `interaction_time`: the amount of time in seconds spent interacting with the containers. Default 3; at least 0.
    pub interaction_time: Option<f32>,
    /// `max_stack_size`: the maximum stack size that the mob will try to take from a container. Default 64; 1 to 64.
    pub max_stack_size: Option<u32>,
    /// `max_visited_containers`: the maximum number of containers that the mob will try to take/place items from before going on cooldown and starting over. Default 16.
    pub max_visited_containers: Option<u32>,
    /// `place_strategy`: the strategy to use for placing the transported item. Any - always place if there is room, With matching - place if there is a matching item in the container, With matching or empty - like With matching but will also place in empty containers. Default `any`.
    pub place_strategy: Option<PlaceStrategy>,
    /// `search_distance`: the maximum search distance horizontally and vertically at which to find containers for taking or placing items. Default `[64, 32]`.
    pub search_distance: Option<Vec2>,
    /// `search_strategy`: the strategy to use for finding source or destination containers. The nearest valid container or a random valid container in range. Default `random`.
    pub search_strategy: Option<SearchStrategy>,
    /// `source_container_types`: a list of block descriptors that should be a container type to get items from.
    pub source_container_types: Option<Vec<BlockDescriptor>>,
}

impl TransportItems {
    /// `priority`, or its default 0.
    pub fn priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// `allow_simultaneous_interaction`, or its default `false`.
    pub fn allow_simultaneous_interaction(&self) -> bool {
        self.allow_simultaneous_interaction.is_some_and(|v| v)
    }

    /// `idle_cooldown`, or its default 20.
    pub fn idle_cooldown(&self) -> u32 {
        self.idle_cooldown.unwrap_or(20)
    }

    /// `initial_cooldown`, or its default 0.
    pub fn initial_cooldown(&self) -> u32 {
        self.initial_cooldown.unwrap_or(0)
    }

    /// `interaction_time`, or its default 3.
    pub fn interaction_time(&self) -> f32 {
        self.interaction_time.unwrap_or(3.0)
    }

    /// `max_stack_size`, or its default 64.
    pub fn max_stack_size(&self) -> u32 {
        self.max_stack_size.unwrap_or(64)
    }

    /// `max_visited_containers`, or its default 16.
    pub fn max_visited_containers(&self) -> u32 {
        self.max_visited_containers.unwrap_or(16)
    }

    /// `place_strategy`, or its default `any`.
    pub fn place_strategy(&self) -> PlaceStrategy {
        self.place_strategy.unwrap_or(PlaceStrategy::Any)
    }

    /// `search_distance`, or its default `[64, 32]`.
    pub fn search_distance(&self) -> [f32; 2] {
        self.search_distance
            .as_ref()
            .map_or([64.0, 32.0], |v| [v.x, v.y])
    }

    /// `search_strategy`, or its default `random`.
    pub fn search_strategy(&self) -> SearchStrategy {
        self.search_strategy.unwrap_or(SearchStrategy::Random)
    }
}

impl Goal for TransportItems {
    const TYPED: FormatVersion = known::V1_21_100;

    fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.base(&mut self.priority);
        c.items("allowed_items", self.allowed_items.as_deref());
        c.blocks(
            "destination_container_types",
            self.destination_container_types.as_deref(),
        );
        c.items("disallowed_items", self.disallowed_items.as_deref());
        c.float(
            "interaction_time",
            &mut self.interaction_time,
            Bound::Min(0.0),
        );
        c.int(
            "max_stack_size",
            &mut self.max_stack_size,
            Bound::MinMax(1, 64),
        );
        c.vector(
            "search_distance",
            self.search_distance.as_ref().map(|v| &v.extra),
        );
        c.blocks(
            "source_container_types",
            self.source_container_types.as_deref(),
        );
        // A non-positive `search_distance` is reported and the default is used: the game logs
        // it at the error level at the goal's set-up and keeps loading the entity, in either
        // reader, so it is a kept error.
        if self
            .search_distance
            .as_ref()
            .is_some_and(|v| v.x <= 0.0 || v.y <= 0.0)
        {
            c.kept("search_distance", VanillaText::Fixed(SEARCH_DISTANCE));
            self.search_distance = None;
        }
    }
}

impl MolangFields for TransportItems {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_items(visitor, "allowed_items", &mut self.allowed_items);
        visit_blocks(
            visitor,
            "destination_container_types",
            &mut self.destination_container_types,
        );
        visit_items(visitor, "disallowed_items", &mut self.disallowed_items);
        visit_blocks(
            visitor,
            "source_container_types",
            &mut self.source_container_types,
        );
    }
}

/// One potion `drink_potion` may drink, and when.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct Potion {
    /// `chance`: the percent chance (from 0.0 to 1.0) of this potion being selected when searching for a potion to use. Default 1.
    pub chance: Option<f32>,
    /// `filters`: the filters to use when determining if this potion can be selected.
    pub filters: Option<Filter>,
    /// `id`: the registry ID of the potion to use. Default -1.
    pub id: Option<i32>,
}

impl Potion {
    /// `chance`, or its default 1.
    pub fn chance(&self) -> f32 {
        self.chance.unwrap_or(1.0)
    }

    /// `id`, or its default -1.
    pub fn id(&self) -> i32 {
        self.id.unwrap_or(-1)
    }
}

impl Potion {
    /// Checks the entry's members.
    pub(crate) fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.filter("filters", self.filters.as_ref());
    }
}

/// A block `eat_block` eats and the block it leaves behind.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct EatAndReplaceBlockPair {
    /// `eat_block`: the block the entity should eat. Default `""`.
    pub eat_block: Option<String>,
    /// `replace_block`: the block that should replace the eaten block. Default `""`.
    pub replace_block: Option<String>,
}

impl EatAndReplaceBlockPair {
    /// `eat_block`, or its default `""`.
    pub fn eat_block(&self) -> &str {
        self.eat_block.as_ref().map_or("", |v| v.as_str())
    }

    /// `replace_block`, or its default `""`.
    pub fn replace_block(&self) -> &str {
        self.replace_block.as_ref().map_or("", |v| v.as_str())
    }
}

impl EatAndReplaceBlockPair {
    /// Checks the entry's members.
    #[allow(clippy::unused_self, reason = "every entry check has this shape")]
    pub(crate) fn check(&mut self, _: &mut GoalCheck<'_>) {}
}

/// A block `place_block` may place at random, with its weight and condition.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct WeightedBlock {
    /// `block`: block descriptor for this entry.
    pub block: Option<BlockDescriptor>,
    /// `filter`: filter for if this entry should be randomly selected from. Self, Target, and Block are set.
    pub filter: Option<Filter>,
    /// `weight`: weight for this entry. Default 1.
    pub weight: Option<i32>,
}

impl WeightedBlock {
    /// `weight`, or its default 1.
    pub fn weight(&self) -> i32 {
        self.weight.unwrap_or(1)
    }
}

impl WeightedBlock {
    /// Checks the entry's members.
    pub(crate) fn check(&mut self, c: &mut GoalCheck<'_>) {
        c.block("block", self.block.as_ref());
        c.filter("filter", self.filter.as_ref());
    }
}

impl MolangFields for WeightedBlock {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Some(block) = &mut self.block {
            visitor.enter(Step::from("block"));
            visit_block(visitor, block);
            visitor.leave();
        }
    }
}

crate::hand_fields!(AdmireItem {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "admire_item_sound": Option<SoundEvent>,
    "on_admire_item_start": Option<EventTrigger>,
    "on_admire_item_stop": Option<EventTrigger>,
});
crate::hand_fields!(ChargeHeldItem {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
});
crate::hand_fields!(DrinkMilk {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "cooldown_seconds": Option<f32>,
});
crate::hand_fields!(DropItemFor {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "cooldown": Option<f32>,
    "drop_item_chance": Option<f32>,
    "entity_types": Option<EntityTypes>,
    "goal_radius": Option<f32>,
    "loot_table": Option<String>,
    "max_head_look_at_height": Option<f32>,
    "minimum_teleport_distance": Option<f32>,
    "offering_distance": Option<f32>,
    "on_drop_attempt": Option<EventTrigger>,
    "search_count": Option<i32>,
    "search_height": Option<i32>,
    "search_range": Option<i32>,
    "seconds_before_pickup": Option<f32>,
    "speed_multiplier": Option<f32>,
    "target_range": Option<Vec3>,
    "teleport_offset": Option<Vec3>,
});
crate::hand_fields!(EatBlock {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "eat_and_replace_block_pairs": Option<Vec<EatAndReplaceBlockPair>>,
    "on_eat": Option<EventTrigger>,
    "success_chance": Option<Molang>,
    "time_until_eat": Option<f32>,
});
crate::hand_fields!(GoAndGiveItemsToNoteblock {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "listen_time": Option<i32>,
    "on_item_throw": Option<Vec<EventTrigger>>,
    "reach_block_distance": Option<f32>,
    "run_speed": Option<f32>,
    "throw_force": Option<f32>,
    "throw_sound": Option<SoundEvent>,
    "vertical_throw_mul": Option<f32>,
});
crate::hand_fields!(GoAndGiveItemsToOwner {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "on_item_throw": Option<Vec<EventTrigger>>,
    "reach_mob_distance": Option<f32>,
    "run_speed": Option<f32>,
    "throw_force": Option<f32>,
    "throw_sound": Option<SoundEvent>,
    "vertical_throw_mul": Option<f32>,
});
crate::hand_fields!(OfferFlower {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "chance_to_start": Option<f32>,
    "max_head_rotation_y": Option<f32>,
    "max_offer_flower_duration": Option<f32>,
    "max_rotation_x": Option<f32>,
    "search_area": Option<Vec3>,
});
crate::hand_fields!(PickupItems {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "can_pickup_any_item": Option<bool>,
    "can_pickup_to_hand_or_equipment": Option<bool>,
    "cooldown_after_being_attacked": Option<i32>,
    "goal_radius": Option<f32>,
    "max_dist": Option<i32>,
    "on_pickup_item_end": Option<EventTrigger>,
    "on_pickup_item_start": Option<EventTrigger>,
    "pickup_based_on_chance": Option<bool>,
    "pickup_same_items_as_in_hand": Option<bool>,
    "search_height": Option<i32>,
    "speed_multiplier": Option<f32>,
    "stop_if_holding_item": Option<bool>,
    "track_target": Option<bool>,
});
crate::hand_fields!(PlaceBlock {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "affected_by_griefing_rule": Option<bool>,
    "chance": Option<f32>,
    "on_place": Option<EventTrigger>,
    "randomly_placeable_blocks": Option<Vec<WeightedBlock>>,
});
crate::hand_fields!(Snacking {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "snacking_cooldown": Option<f32>,
    "snacking_cooldown_min": Option<f32>,
    "snacking_stop_chance": Option<f32>,
});
crate::hand_fields!(TakeBlock {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "affected_by_griefing_rule": Option<bool>,
    "chance": Option<f32>,
    "on_take": Option<EventTrigger>,
    "requires_line_of_sight": Option<bool>,
});
crate::hand_fields!(TakeFlower {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "max_head_rotation_y": Option<f32>,
    "max_rotation_x": Option<f32>,
    "max_wait_time": Option<f32>,
    "min_distance_to_target": Option<f32>,
    "min_wait_time": Option<f32>,
    "on_take_flower": Option<EventTrigger>,
    "search_area": Option<Vec3>,
    "speed_multiplier": Option<f32>,
});
crate::hand_fields!(TradeWithPlayer {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "max_distance_from_player": Option<f32>,
});
crate::hand_fields!(TransportItems {
    "priority": Option<i32>,
    "control_flags": Option<Vec<ControlFlag>>,
    "allow_simultaneous_interaction": Option<bool>,
    "idle_cooldown": Option<u32>,
    "initial_cooldown": Option<u32>,
    "interaction_time": Option<f32>,
    "max_stack_size": Option<u32>,
    "max_visited_containers": Option<u32>,
    "place_strategy": Option<PlaceStrategy>,
    "search_distance": Option<Vec2>,
    "search_strategy": Option<SearchStrategy>,
});
crate::hand_fields!(Potion {
    "chance": Option<f32>,
    "id": Option<i32>,
});
bedrock_addon_core::impl_no_molang!(Potion);
crate::hand_fields!(WeightedBlock {
    "weight": Option<i32>,
});
