//! Components about how the entity moves and collides: `jump.static`, `jump.dynamic`,
//! `preferred_path`, `flocking`, `game_event_movement_tracking`, `grows_crop`, `physics`,
//! `collision_box`, `block_movement_slowdown_immunity`; and the components 1.26.60 types first:
//! the move controls `movement.*`, `rail_movement`, `dash_action`, `buoyant`, `teleport`,
//! `trail`, `annotation.break_door`.
// Types and defaults follow the game's schemas (1.26.40 for `jump.*`, the rest 1.26.50).

use super::value::descriptor_tags;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, JsonPath, Severity};
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::{Filter, FloatRange};
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_shared_shapes::BlockDescriptor;
use facet::Facet;
use facet_value::VObject;

use super::parse::{ComponentCheck, Typed};
use super::value::tell_partial_vector;
use crate::goal::Vec3;

/// `minecraft:jump.static`: the entity jumps with a fixed power.
#[doc(alias = "minecraft:jump.static")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct JumpStatic {
    /// `jump_power`: the initial vertical velocity; default 0.42.
    #[facet(default = 0.42_f32)]
    pub jump_power: f32,
}

impl Typed for JumpStatic {}

/// `minecraft:jump.dynamic`: the entity's jump changes with its speed (the rabbit's hops).
#[doc(alias = "minecraft:jump.dynamic")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct JumpDynamic {
    /// `fast_skip_data`: the jump while moving fast.
    #[facet(default = SkipData::default())]
    pub fast_skip_data: SkipData,
    /// `regular_skip_data`: the jump at regular speed.
    #[facet(default = SkipData::default())]
    pub regular_skip_data: SkipData,
}

impl JumpDynamic {
    /// The skip data the 1.26.0 upgrade writes over any value the author wrote:
    /// [`SkipData::regular_1_26_0`] and [`SkipData::fast_1_26_0`].
    pub fn upgraded_1_26_0() -> Self {
        Self {
            fast_skip_data: SkipData::fast_1_26_0(),
            regular_skip_data: SkipData::regular_1_26_0(),
        }
    }
}

impl Typed for JumpDynamic {}

/// One jump of [`JumpDynamic`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SkipData {
    /// `animation_duration`: the jump animation's length in ticks; default 1.
    #[facet(default = 1_i32)]
    pub animation_duration: i32,
    /// `distance_scale`: the horizontal distance scale; default 1.
    #[facet(default = 1.0_f32)]
    pub distance_scale: f32,
    /// `height`: the jump height; default 0.1.
    #[facet(default = 0.1_f32)]
    pub height: f32,
    /// `jump_delay`: ticks between jumps; default 30.
    #[facet(default = 30_i32)]
    pub jump_delay: i32,
}

impl SkipData {
    /// Skip data with these values.
    pub fn new(animation_duration: i32, distance_scale: f32, height: f32, jump_delay: i32) -> Self {
        Self {
            animation_duration,
            distance_scale,
            height,
            jump_delay,
        }
    }

    /// The regular jump the 1.26.0 upgrade writes: distance 0.8, height 0.1, delay 20,
    /// animation 3.
    pub fn regular_1_26_0() -> Self {
        Self::new(3, 0.8, 0.1, 20)
    }

    /// The fast jump the 1.26.0 upgrade writes: distance 1.75, height 0.15, delay 1,
    /// animation 1.
    pub fn fast_1_26_0() -> Self {
        Self::new(1, 1.75, 0.15, 1)
    }
}

/// `minecraft:preferred_path`: path costs that make the entity prefer some blocks (the
/// villager's paths).
#[doc(alias = "minecraft:preferred_path")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct PreferredPath {
    /// `default_block_cost`: the cost of a block no set names; default 0.
    #[facet(default = 0.0_f32)]
    pub default_block_cost: f32,
    /// `jump_cost`: the cost of a step up; default 0.
    #[facet(default = 0.0_f32)]
    pub jump_cost: f32,
    /// `max_fall_blocks`: the highest drop the path may take; default 3.
    #[facet(default = 3_i32)]
    pub max_fall_blocks: i32,
    /// `preferred_path_blocks`: block sets with their costs.
    #[facet(default)]
    pub preferred_path_blocks: Vec<BlockSet>,
}

/// One block set of [`PreferredPath`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct BlockSet {
    /// `blocks`: the blocks.
    #[facet(schema::molang_with = descriptor_tags, default)]
    pub blocks: Vec<BlockDescriptor>,
    /// `cost`: the path cost of these blocks.
    pub cost: Option<f32>,
}

impl Typed for PreferredPath {
    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        let sets = JsonPath::new().child("preferred_path_blocks");
        for (i, set) in self.preferred_path_blocks.iter().enumerate() {
            let blocks = sets.index(i).child("blocks");
            for (j, block) in set.blocks.iter().enumerate() {
                cx.extend(block.check(&blocks.index(j), cx.unknown()));
            }
        }
    }
}

/// `minecraft:flocking`: the entity moves in a flock with others of its kind.
#[doc(alias = "minecraft:flocking")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Flocking {
    /// `block_distance`: the distance at which blocks are avoided; default 0.
    #[facet(default = 0.0_f32)]
    pub block_distance: f32,
    /// `block_weight`: the weight of block avoidance; default 0.
    #[facet(default = 0.0_f32)]
    pub block_weight: f32,
    /// `breach_influence`: the push out of the water when breaching; default 0.
    #[facet(default = 0.0_f32)]
    pub breach_influence: f32,
    /// `cohesion_threshold`: the distance at which members pull together; default 1.
    #[facet(default = 1.0_f32)]
    pub cohesion_threshold: f32,
    /// `cohesion_weight`: the weight of cohesion; default 1.
    #[facet(default = 1.0_f32)]
    pub cohesion_weight: f32,
    /// `goal_weight`: the weight of moving towards the flock's goal; default 0.
    #[facet(default = 0.0_f32)]
    pub goal_weight: f32,
    /// `high_flock_limit`: above this size members leave the flock; default 0.
    #[facet(default = 0_i32)]
    pub high_flock_limit: i32,
    /// `in_water`: the flock is in water; default `false`.
    #[facet(default = false)]
    pub in_water: bool,
    /// `influence_radius`: the radius within which members influence each other; default 0.
    #[facet(default = 0.0_f32)]
    pub influence_radius: f32,
    /// `innner_cohesion_threshold` (vanilla's spelling): the inner cohesion distance; default 0.
    #[facet(default = 0.0_f32)]
    pub innner_cohesion_threshold: f32,
    /// `loner_chance`: the chance to leave the flock; default 0.
    #[facet(default = 0.0_f32)]
    pub loner_chance: f32,
    /// `low_flock_limit`: below this size the flock looks for members; default 0.
    #[facet(default = 0_i32)]
    pub low_flock_limit: i32,
    /// `match_variants`: only entities of the same variant flock together; default `false`.
    #[facet(default = false)]
    pub match_variants: bool,
    /// `max_height`: the highest the flock swims or flies; default 0.
    #[facet(default = 0.0_f32)]
    pub max_height: f32,
    /// `min_height`: the lowest the flock swims or flies; default 0.
    #[facet(default = 0.0_f32)]
    pub min_height: f32,
    /// `separation_threshold`: the distance below which members push apart; default 2.
    #[facet(default = 2.0_f32)]
    pub separation_threshold: f32,
    /// `separation_weight`: the weight of separation; default 1.
    #[facet(default = 1.0_f32)]
    pub separation_weight: f32,
    /// `use_center_of_mass`: members steer towards the flock's centre; default `false`.
    #[facet(default = false)]
    pub use_center_of_mass: bool,
}

impl Typed for Flocking {}

/// `minecraft:game_event_movement_tracking`: the entity emits the `entityMove`, `swim` and
/// `flap` game events while it moves.
#[doc(alias = "minecraft:game_event_movement_tracking")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct GameEventMovementTracking {
    /// `emit_flap`: emits `flap` while flying; default `false`.
    #[facet(default = false)]
    pub emit_flap: bool,
    /// `emit_move`: emits `entityMove` while moving on the ground; default `true`.
    #[facet(default = true)]
    pub emit_move: bool,
    /// `emit_swim`: emits `swim` while swimming; default `true`.
    #[facet(default = true)]
    pub emit_swim: bool,
}

impl Typed for GameEventMovementTracking {}

/// `minecraft:grows_crop`: the entity may grow crops it walks over (the bee).
#[doc(alias = "minecraft:grows_crop")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct GrowsCrop {
    /// `chance`: the chance per tick to grow a crop; default 0.
    #[facet(default = 0.0_f32)]
    pub chance: f32,
    /// `charges`: how many crops it can grow; default 10.
    #[facet(default = 10_i32)]
    pub charges: i32,
}

impl Typed for GrowsCrop {}

/// `minecraft:physics`: gravity and collision.
#[doc(alias = "minecraft:physics")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Physics {
    /// `has_collision`: the entity collides with blocks; default `true`.
    #[facet(default = true)]
    pub has_collision: bool,
    /// `has_gravity`: the entity falls; default `true`.
    #[facet(default = true)]
    pub has_gravity: bool,
    /// `push_towards_closest_space`: the entity is pushed out of blocks towards the nearest free
    /// space; default `false`.
    #[facet(default = false)]
    pub push_towards_closest_space: bool,
}

impl Typed for Physics {}

/// `minecraft:collision_box`: the size of the entity's collision box.
#[doc(alias = "minecraft:collision_box")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct CollisionBox {
    /// `height`: the box's height in blocks, within ±100 000 000; default 1.8.
    #[facet(default = 1.8_f32, schema::min = -100_000_000.0, schema::max = 100_000_000.0)]
    pub height: f32,
    /// `width`: the box's width and depth in blocks, within ±100 000 000; default 0.6.
    #[facet(default = 0.6_f32, schema::min = -100_000_000.0, schema::max = 100_000_000.0)]
    pub width: f32,
}

impl CollisionBox {
    /// A box `width` wide and `height` high.
    pub fn new(width: f32, height: f32) -> Self {
        Self { height, width }
    }
}

impl Typed for CollisionBox {
    // The game's schema bounds both members to −100000000 ..= 100000000.
    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        cx.bounds(JsonPath::new(), self);
    }
}

/// `minecraft:block_movement_slowdown_immunity`: blocks that do not slow the entity down
/// (powder snow, cobwebs, sweet berry bushes).
#[doc(alias = "minecraft:block_movement_slowdown_immunity")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct BlockMovementSlowdownImmunity {
    /// `blocks`: the blocks.
    #[facet(schema::molang_with = descriptor_tags, default)]
    pub blocks: Vec<BlockDescriptor>,
}

impl Typed for BlockMovementSlowdownImmunity {
    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        let at = JsonPath::new().child("blocks");
        for (i, block) in self.blocks.iter().enumerate() {
            cx.extend(block.check(&at.index(i), cx.unknown()));
        }
    }
}

/// The members every move control has (the base of each `movement.*` type): the defaults the game's
/// legacy parsers and its typed readers share.
// The default `max_turn` is 30.0 in the typed reader and in every legacy reader.
const MAX_TURN: f32 = 30.0;

/// Declares a move control that has only `max_turn`.
macro_rules! move_controls {
    ($( $(#[$doc:meta])* $name:ident = $key:literal; )*) => {$(
        $(#[$doc])*
        #[doc(alias = $key)]
        #[derive(Clone, Debug, PartialEq, Facet, Schema)]
        #[facet(schema::unknown = "keep")]
        pub struct $name {
            /// `max_turn`: the most the mob turns per tick, in degrees; default 30.
            #[facet(default = MAX_TURN)]
            pub max_turn: f32,
        }

        impl Typed for $name {}
    )*};
}

// One section per key (typed at 1.26.60); each legacy reader reads the same member.
move_controls! {
    /// `minecraft:movement.amphibious`: the mob swims in water and walks on land.
    MovementAmphibious = "minecraft:movement.amphibious";
    /// `minecraft:movement.basic`: the mob's basic movement.
    MovementBasic = "minecraft:movement.basic";
    /// `minecraft:movement.fly`: the mob flies.
    MovementFly = "minecraft:movement.fly";
    /// `minecraft:movement.generic`: the mob flies, swims, climbs and so on.
    MovementGeneric = "minecraft:movement.generic";
    /// `minecraft:movement.hover`: the mob hovers.
    MovementHover = "minecraft:movement.hover";
    /// `minecraft:movement.skip`: the mob hops as it moves.
    MovementSkip = "minecraft:movement.skip";
}

/// `minecraft:movement.glide`: a flying mob that glides.
// `movement.glide`: the defaults are 0.1 and 0.2, in the typed and the legacy reader alike.
#[doc(alias = "minecraft:movement.glide")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MovementGlide {
    /// `max_turn`: the most the mob turns per tick, in degrees; default 30.
    #[facet(default = MAX_TURN)]
    pub max_turn: f32,
    /// `start_speed`: the speed when it starts gliding; default 0.1.
    #[facet(default = 0.1_f32)]
    pub start_speed: f32,
    /// `speed_when_turning`: the speed it slows to when it turns quickly; default 0.2.
    #[facet(default = 0.2_f32)]
    pub speed_when_turning: f32,
}

impl Typed for MovementGlide {}

/// `minecraft:movement.sway`: the mob sways from side to side as if it swims.
// `movement.sway`: the defaults are 0.5 and 0.05, in the typed and the legacy reader alike.
#[doc(alias = "minecraft:movement.sway")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MovementSway {
    /// `max_turn`: the most the mob turns per tick, in degrees; default 30.
    #[facet(default = MAX_TURN)]
    pub max_turn: f32,
    /// `sway_frequency`: the multiplier of the sway's frequency; default 0.5.
    #[facet(default = 0.5_f32)]
    pub sway_frequency: f32,
    /// `sway_amplitude`: the strength of the sway; default 0.05.
    #[facet(default = 0.05_f32)]
    pub sway_amplitude: f32,
}

impl Typed for MovementSway {}

/// `minecraft:movement.jump`: the mob jumps as it moves, with a delay between jumps (the
/// slime's move control).
// `movement.jump`: `jump_delay` is a float range, (0, 0) by default; the legacy parser takes a
// number, `[min, max]` or `{range_min, range_max}`.
#[doc(alias = "minecraft:movement.jump")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MovementJump {
    /// `max_turn`: the most the mob turns per tick, in degrees; default 30.
    #[facet(default = MAX_TURN)]
    pub max_turn: f32,
    /// `jump_delay`: the delay after landing before the next jump, a range; default 0 to 0.
    #[facet(default = FloatRange::default())]
    pub jump_delay: FloatRange,
}

impl Typed for MovementJump {
    const KEEP_NESTED: bool = true;

    /// Below 1.26.60 a `{range_min, range_max}` delay is the typed `{min, max}`.
    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
        if !matches!(cx.mode(), ConvertMode::Lenient) {
            return false;
        }
        legacy_range(json, "jump_delay", false, &JsonPath::new(), cx)
    }

    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        cx.range(&JsonPath::new().child("jump_delay"), &mut self.jump_delay);
    }
}

/// The legacy reading of a range written as an object at member `key`: every legacy reader of these
/// ranges (the float and int range parsers, the `timer` range node and the `movement.jump` reader)
/// reads an object's `range_min` / `range_max` only, each with jsoncpp's `asFloat(float)` /
/// `asInt(int)`, and logs nothing for a bound. So: a `min` / `max` member is not read and is
/// dropped, with an information (the range keeps its defaults where nothing else gives it); a bound
/// that is no number reads as a bool's 0 or 1 and as 0 otherwise, with an information
/// `InvalidValue`; with `integer`, a real is truncated toward zero, with an information
/// `ValueRewritten` (the write-back writes the integer); an absent or `null` bound keeps its
/// default. Then `range_min` / `range_max` become the typed `min` / `max`, each move told. Returns
/// whether anything changed.
pub(crate) fn legacy_range(
    json: &mut VObject,
    key: &str,
    integer: bool,
    at: &JsonPath,
    cx: &mut ComponentCheck<'_>,
) -> bool {
    let Some(range) = json.get_mut(key).and_then(Value::as_object_mut) else {
        return false;
    };
    let member = at.child(key);
    let mut changed = false;
    for typed in ["min", "max"] {
        if range.remove(typed).is_some() {
            cx.push(
                Code::from(&core_codes::DROPPED_FIELD),
                Severity::Info,
                &member.child(typed),
            );
            changed = true;
        }
    }
    for bound in ["range_min", "range_max"] {
        let Some(value) = range.get(bound).filter(|value| !value.is_null()) else {
            continue;
        };
        let (read, code) = if let Some(number) = value.as_number() {
            if !integer || number.is_integer() {
                continue;
            }
            #[allow(
                clippy::cast_possible_truncation,
                reason = "a real is truncated toward zero, as the game converts"
            )]
            let truncated = number
                .to_f64_lossy()
                .trunc()
                .clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i64;
            (
                Value::from(truncated),
                Code::from(&core_codes::VALUE_REWRITTEN),
            )
        } else {
            let read = i64::from(value.as_bool() == Some(true));
            (Value::from(read), Code::from(&core_codes::INVALID_VALUE))
        };
        range.insert(bound, read);
        cx.push(code, Severity::Info, &member.child(bound));
        changed = true;
    }
    for (legacy, typed) in [("range_min", "min"), ("range_max", "max")] {
        if let Some(value) = range.remove(legacy) {
            range.insert(typed, value);
            cx.moved(member.child(legacy), member.child(typed));
            changed = true;
        }
    }
    changed
}

/// `minecraft:rail_movement`: the entity moves only on rails.
// `rail_movement`: `max_speed` defaults to 0.4.
#[doc(alias = "minecraft:rail_movement")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RailMovement {
    /// `max_speed`: the most speed on a rail; default 0.4.
    #[facet(default = 0.4_f32)]
    pub max_speed: f32,
}

impl Typed for RailMovement {}

/// `minecraft:dash_action`: a rideable entity can dash.
// `dash_action`: the defaults are 1, 1, 1 and false; `entity` is 0.
#[doc(alias = "minecraft:dash_action")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DashAction {
    /// `cooldown_time`: the dash's cooldown in seconds; default 1.
    #[facet(default = 1.0_f32)]
    pub cooldown_time: f32,
    /// `horizontal_momentum`: the dash's horizontal momentum; default 1.
    #[facet(default = 1.0_f32)]
    pub horizontal_momentum: f32,
    /// `vertical_momentum`: the dash's vertical momentum; default 1.
    #[facet(default = 1.0_f32)]
    pub vertical_momentum: f32,
    /// `can_dash_underwater`: the entity can dash under water; default `false`.
    #[facet(default = false)]
    pub can_dash_underwater: bool,
    /// `direction`: whose look the momentum follows; default `entity`.
    #[facet(default = DashDirection::Entity)]
    pub direction: DashDirection,
}

impl Typed for DashAction {}

/// Whose look a [`DashAction`]'s momentum follows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum DashDirection {
    /// The entity's yaw, horizontally (JSON `entity`).
    #[default]
    #[facet(rename = "entity")]
    Entity,
    /// The controlling passenger's pitch and yaw (JSON `passenger`).
    #[facet(rename = "passenger")]
    Passenger,
}

/// `minecraft:buoyant`: the entity floats on the given liquid blocks.
#[doc(alias = "minecraft:buoyant")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Buoyant {
    /// `base_buoyancy`: how much the entity floats; default 1.
    #[facet(default = 1.0_f32)]
    pub base_buoyancy: f32,
    /// `apply_gravity`: gravity applies each tick; default `true`.
    #[facet(default = true)]
    pub apply_gravity: bool,
    /// `movement_type`: the vertical movement applied; default `waves`.
    #[facet(default = BuoyancyMovement::Waves)]
    pub movement_type: BuoyancyMovement,
    /// `big_wave_probability`: the chance of a big wave (with `waves`); default 0.03.
    #[facet(default = 0.03_f32)]
    pub big_wave_probability: f32,
    /// `big_wave_speed`: the speed multiplier of a big wave; default 10.
    #[facet(default = 10.0_f32)]
    pub big_wave_speed: f32,
    /// `liquid_blocks`: the liquid blocks the entity floats on; default none.
    #[facet(schema::molang_with = descriptor_tags, default)]
    pub liquid_blocks: Vec<BlockDescriptor>,
    /// `drag_down_on_buoyancy_removed`: how much the entity is dragged down when the
    /// component goes; default 0.
    #[facet(default = 0.0_f32)]
    pub drag_down_on_buoyancy_removed: f32,
    /// `can_auto_step_from_liquid`: the entity steps out of a liquid onto a solid neighbour it
    /// is pushed against; default `false`.
    #[facet(default = false)]
    pub can_auto_step_from_liquid: bool,
}

impl Typed for Buoyant {
    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        let blocks = JsonPath::new().child("liquid_blocks");
        for (i, block) in self.liquid_blocks.iter().enumerate() {
            cx.extend(block.check(&blocks.index(i), cx.unknown()));
        }
    }
}

/// The vertical movement of a [`Buoyant`] entity (`buoyancy_movement_type`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum BuoyancyMovement {
    /// None (JSON `none`).
    #[facet(rename = "none")]
    None,
    /// Waves by the entity's speed (JSON `waves`).
    #[default]
    #[facet(rename = "waves")]
    Waves,
    /// Waves going through (JSON `bobbing`).
    #[facet(rename = "bobbing")]
    Bobbing,
}

/// `minecraft:teleport`: how the entity teleports.
// `teleport`: `projectile_filter` is absent by default; the typed type binds no camel-case
// spelling.
#[doc(alias = "minecraft:teleport")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Teleport {
    /// `random_teleports`: it teleports at random; default `true`.
    #[facet(default = true)]
    pub random_teleports: bool,
    /// `teleports_on_projectile_hit`: it teleports away from a projectile about to hit it;
    /// default `false`.
    #[facet(default = false)]
    pub teleports_on_projectile_hit: bool,
    /// `projectile_filter`: which projectiles make it teleport; absent means all.
    pub projectile_filter: Option<Filter>,
    /// `min_random_teleport_time`: the least seconds between random teleports; default 0.
    #[facet(default = 0.0_f32)]
    pub min_random_teleport_time: f32,
    /// `max_random_teleport_time`: the most seconds between random teleports; default 20.
    #[facet(default = 20.0_f32)]
    pub max_random_teleport_time: f32,
    /// `random_teleport_cube`: the area a random teleport lands in; default 32 × 16 × 32.
    #[facet(default = Vec3::new(32.0, 16.0, 32.0))]
    pub random_teleport_cube: Vec3,
    /// `target_distance`: the farthest it teleports while chasing a target; default 16.
    #[facet(default = 16.0_f32)]
    pub target_distance: f32,
    /// `target_teleport_chance`: the chance it teleports, 0 to 1; default 1.
    #[facet(default = 1.0_f32)]
    pub target_teleport_chance: f32,
    /// `light_teleport_chance`: the chance modifier in daylight; default 0.01.
    #[facet(default = 0.01_f32)]
    pub light_teleport_chance: f32,
    /// `dark_teleport_chance`: the chance modifier in darkness; default 0.01.
    #[facet(default = 0.01_f32)]
    pub dark_teleport_chance: f32,
}

impl Typed for Teleport {
    /// The legacy reader binds no camel-case spelling: the schema reader's
    /// validation before the read refuses one, so nothing is renamed here.
    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
        tell_partial_vector(json, "random_teleport_cube", &JsonPath::new(), cx);
        false
    }

    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        if let Some(filter) = &self.projectile_filter {
            cx.extend(filter.check(&JsonPath::new().child("projectile_filter"), cx.unknown()));
        }
    }
}

/// `minecraft:trail`: the entity leaves a trail of blocks (mob griefing permitting).
// `trail`: `block_type` defaults to "air", `spawn_offset` to 0 and `radius` to −1 (within
// −1 ..= 16); the new `replaceable_blocks` (empty: air only) and `force_replace_matched_blocks`
// are ignored by the legacy reader.
#[doc(alias = "minecraft:trail")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Trail {
    /// `spawn_filter`: when the block spawns; absent means always.
    pub spawn_filter: Option<Filter>,
    /// `block_type`: the block spawned; default `air`.
    #[facet(schema::molang_with = descriptor_tags, default = BlockDescriptor::block(BlockId::parse("air")))]
    pub block_type: BlockDescriptor,
    /// `spawn_offset`: where the block spawns relative to the entity; default 0, 0, 0.
    #[facet(default = Vec3::new(0.0, 0.0, 0.0))]
    pub spawn_offset: Vec3,
    /// `replaceable_blocks`: the blocks the trail may replace; empty means air only.
    #[facet(schema::molang_with = descriptor_tags, default)]
    pub replaceable_blocks: Vec<BlockDescriptor>,
    /// `radius`: the trail covers a square of `2 × radius + 1` blocks; −1 (the default) is the
    /// legacy four-corner sampling. −1 to 16.
    #[facet(default = -1_i32)]
    pub radius: i32,
    /// `force_replace_matched_blocks`: replaceable blocks are replaced even where placement
    /// rules forbid it; default `false`.
    #[facet(default = false)]
    pub force_replace_matched_blocks: bool,
}

/// The members 1.26.60 adds to `trail`, which the legacy reader does not read.
const TRAIL_NEW_MEMBERS: [&str; 3] = [
    "replaceable_blocks",
    "radius",
    "force_replace_matched_blocks",
];

impl Typed for Trail {
    /// Below 1.26.60 the legacy reader ignores the new members: they keep their defaults.
    /// The game says nothing; the crate tells each with an information, since the write-back
    /// writes the default where the file has a value.
    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
        tell_partial_vector(json, "spawn_offset", &JsonPath::new(), cx);
        if !matches!(cx.mode(), ConvertMode::Lenient) {
            return false;
        }
        let mut changed = false;
        for member in TRAIL_NEW_MEMBERS {
            if json.remove(member).is_some() {
                cx.push(
                    Code::from(&core_codes::DROPPED_FIELD),
                    Severity::Info,
                    &JsonPath::new().child(member),
                );
                changed = true;
            }
        }
        changed
    }

    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        if let Some(filter) = &self.spawn_filter {
            cx.extend(filter.check(&JsonPath::new().child("spawn_filter"), cx.unknown()));
        }
        cx.extend(
            self.block_type
                .check(&JsonPath::new().child("block_type"), cx.unknown()),
        );
        let blocks = JsonPath::new().child("replaceable_blocks");
        for (i, block) in self.replaceable_blocks.iter().enumerate() {
            cx.extend(block.check(&blocks.index(i), cx.unknown()));
        }
        // The typed reader bounds `radius` to -1 ..= 16.
        cx.within(
            &JsonPath::new().child("radius"),
            &mut self.radius,
            (-1, 16),
            -1,
        );
    }
}

/// `minecraft:annotation.break_door`: the entity breaks doors, as its navigation allows.
// `annotation.break_door`: the break time defaults to 12 and the minimum difficulty to 3
// (`hard`).
#[doc(alias = "minecraft:annotation.break_door")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct AnnotationBreakDoor {
    /// `break_time`: the seconds needed to break a door; default 12.
    #[facet(default = 12.0_f32)]
    pub break_time: f32,
    /// `min_difficulty`: the lowest difficulty at which it breaks doors; default `hard`.
    #[facet(default = Difficulty::Hard)]
    pub min_difficulty: Difficulty,
}

impl Typed for AnnotationBreakDoor {}

/// A world difficulty.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum Difficulty {
    /// Peaceful (JSON `peaceful`).
    #[facet(rename = "peaceful")]
    Peaceful,
    /// Easy (JSON `easy`).
    #[facet(rename = "easy")]
    Easy,
    /// Normal (JSON `normal`).
    #[facet(rename = "normal")]
    Normal,
    /// Hard (JSON `hard`).
    #[default]
    #[facet(rename = "hard")]
    Hard,
}
