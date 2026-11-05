//! Components about how the entity moves and collides: `jump.static`, `jump.dynamic`,
//! `preferred_path`, `flocking`, `game_event_movement_tracking`, `grows_crop`, `physics`,
//! `collision_box`, `block_movement_slowdown_immunity`; and the components 1.26.60 types first:
//! the move controls `movement.*`, `rail_movement`, `dash_action`, `buoyant`, `teleport`,
//! `trail`, `annotation.break_door`.
// Types and defaults follow the game's schemas (1.26.40 for `jump.*`, the rest 1.26.50).

use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, Severity};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::shared::{BlockDescriptor, Filter, FloatRange};
use bedrock_addon_core::versioning::ConvertMode;
use facet::Facet;
use facet_value::VObject;

use super::parse::{Check, Typed};
use super::value::{tell_partial_vector, visit_block_tags};
use crate::goal::Vec3;

/// `minecraft:jump.static`: the entity jumps with a fixed power.
#[doc(alias = "minecraft:jump.static")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct JumpStatic {
    /// `jump_power`: the initial vertical velocity; default 0.42.
    #[facet(default = StrictFloat(0.42_f32))]
    pub jump_power: StrictFloat<f32>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for JumpStatic {
    fn default() -> Self {
        Self {
            jump_power: StrictFloat(0.42),
            extra: Extra::default(),
        }
    }
}

impl Typed for JumpStatic {}

/// `minecraft:jump.dynamic`: the entity's jump changes with its speed (the rabbit's hops).
#[doc(alias = "minecraft:jump.dynamic")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct JumpDynamic {
    /// `fast_skip_data`: the jump while moving fast.
    #[facet(default = SkipData::default())]
    pub fast_skip_data: SkipData,
    /// `regular_skip_data`: the jump at regular speed.
    #[facet(default = SkipData::default())]
    pub regular_skip_data: SkipData,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl JumpDynamic {
    /// The skip data the 1.26.0 upgrade writes over any value the author wrote:
    /// [`SkipData::regular_1_26_0`] and [`SkipData::fast_1_26_0`].
    pub fn upgraded_1_26_0() -> Self {
        Self {
            fast_skip_data: SkipData::fast_1_26_0(),
            regular_skip_data: SkipData::regular_1_26_0(),
            extra: Extra::default(),
        }
    }
}

impl Typed for JumpDynamic {}

/// One jump of [`JumpDynamic`].
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SkipData {
    /// `animation_duration`: the jump animation's length in ticks; default 1.
    #[facet(default = StrictInt(1_i32))]
    pub animation_duration: StrictInt<i32>,
    /// `distance_scale`: the horizontal distance scale; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub distance_scale: StrictFloat<f32>,
    /// `height`: the jump height; default 0.1.
    #[facet(default = StrictFloat(0.1_f32))]
    pub height: StrictFloat<f32>,
    /// `jump_delay`: ticks between jumps; default 30.
    #[facet(default = StrictInt(30_i32))]
    pub jump_delay: StrictInt<i32>,
    /// Members the entry does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for SkipData {
    fn default() -> Self {
        Self::new(1, 1.0, 0.1, 30)
    }
}

impl SkipData {
    /// Skip data with these values.
    pub fn new(animation_duration: i32, distance_scale: f32, height: f32, jump_delay: i32) -> Self {
        Self {
            animation_duration: StrictInt(animation_duration),
            distance_scale: StrictFloat(distance_scale),
            height: StrictFloat(height),
            jump_delay: StrictInt(jump_delay),
            extra: Extra::default(),
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
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct PreferredPath {
    /// `default_block_cost`: the cost of a block no set names; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub default_block_cost: StrictFloat<f32>,
    /// `jump_cost`: the cost of a step up; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub jump_cost: StrictFloat<f32>,
    /// `max_fall_blocks`: the highest drop the path may take; default 3.
    #[facet(default = StrictInt(3_i32))]
    pub max_fall_blocks: StrictInt<i32>,
    /// `preferred_path_blocks`: block sets with their costs.
    #[facet(default)]
    pub preferred_path_blocks: Vec<BlockSet>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for PreferredPath {
    fn default() -> Self {
        Self {
            default_block_cost: StrictFloat(0.0),
            jump_cost: StrictFloat(0.0),
            max_fall_blocks: StrictInt(3),
            preferred_path_blocks: Vec::new(),
            extra: Extra::default(),
        }
    }
}

/// One block set of [`PreferredPath`].
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct BlockSet {
    /// `blocks`: the blocks.
    #[facet(default)]
    pub blocks: Vec<BlockDescriptor>,
    /// `cost`: the path cost of these blocks.
    #[facet(default)]
    pub cost: Option<StrictFloat<f32>>,
    /// Members the entry does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Typed for PreferredPath {
    fn check(&mut self, cx: &mut Check<'_>) {
        let sets = JsonPath::new().child("preferred_path_blocks");
        for (i, set) in self.preferred_path_blocks.iter().enumerate() {
            let blocks = sets.index(i).child("blocks");
            for (j, block) in set.blocks.iter().enumerate() {
                cx.extend(block.check(&blocks.index(j)));
            }
        }
    }
}

impl MolangFields for PreferredPath {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter("preferred_path_blocks".into());
        for (i, set) in self.preferred_path_blocks.iter_mut().enumerate() {
            visitor.enter(i.into());
            visit_blocks(&mut set.blocks, visitor);
            visitor.leave();
        }
        visitor.leave();
    }
}

/// Hands the `tags` of every descriptor of `blocks` (member `blocks`) to the Molang walk.
fn visit_blocks(blocks: &mut [BlockDescriptor], visitor: &mut MolangVisitor<'_>) {
    visitor.enter("blocks".into());
    for (j, block) in blocks.iter_mut().enumerate() {
        visitor.enter(j.into());
        visit_block_tags(block, visitor);
        visitor.leave();
    }
    visitor.leave();
}

/// `minecraft:flocking`: the entity moves in a flock with others of its kind.
#[doc(alias = "minecraft:flocking")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Flocking {
    /// `block_distance`: the distance at which blocks are avoided; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub block_distance: StrictFloat<f32>,
    /// `block_weight`: the weight of block avoidance; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub block_weight: StrictFloat<f32>,
    /// `breach_influence`: the push out of the water when breaching; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub breach_influence: StrictFloat<f32>,
    /// `cohesion_threshold`: the distance at which members pull together; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub cohesion_threshold: StrictFloat<f32>,
    /// `cohesion_weight`: the weight of cohesion; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub cohesion_weight: StrictFloat<f32>,
    /// `goal_weight`: the weight of moving towards the flock's goal; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub goal_weight: StrictFloat<f32>,
    /// `high_flock_limit`: above this size members leave the flock; default 0.
    #[facet(default = StrictInt(0_i32))]
    pub high_flock_limit: StrictInt<i32>,
    /// `in_water`: the flock is in water; default `false`.
    #[facet(default = StrictBool(false))]
    pub in_water: StrictBool,
    /// `influence_radius`: the radius within which members influence each other; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub influence_radius: StrictFloat<f32>,
    /// `innner_cohesion_threshold` (vanilla's spelling): the inner cohesion distance; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub innner_cohesion_threshold: StrictFloat<f32>,
    /// `loner_chance`: the chance to leave the flock; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub loner_chance: StrictFloat<f32>,
    /// `low_flock_limit`: below this size the flock looks for members; default 0.
    #[facet(default = StrictInt(0_i32))]
    pub low_flock_limit: StrictInt<i32>,
    /// `match_variants`: only entities of the same variant flock together; default `false`.
    #[facet(default = StrictBool(false))]
    pub match_variants: StrictBool,
    /// `max_height`: the highest the flock swims or flies; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub max_height: StrictFloat<f32>,
    /// `min_height`: the lowest the flock swims or flies; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub min_height: StrictFloat<f32>,
    /// `separation_threshold`: the distance below which members push apart; default 2.
    #[facet(default = StrictFloat(2.0_f32))]
    pub separation_threshold: StrictFloat<f32>,
    /// `separation_weight`: the weight of separation; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub separation_weight: StrictFloat<f32>,
    /// `use_center_of_mass`: members steer towards the flock's centre; default `false`.
    #[facet(default = StrictBool(false))]
    pub use_center_of_mass: StrictBool,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for Flocking {
    fn default() -> Self {
        Self {
            block_distance: StrictFloat(0.0),
            block_weight: StrictFloat(0.0),
            breach_influence: StrictFloat(0.0),
            cohesion_threshold: StrictFloat(1.0),
            cohesion_weight: StrictFloat(1.0),
            goal_weight: StrictFloat(0.0),
            high_flock_limit: StrictInt(0),
            in_water: StrictBool(false),
            influence_radius: StrictFloat(0.0),
            innner_cohesion_threshold: StrictFloat(0.0),
            loner_chance: StrictFloat(0.0),
            low_flock_limit: StrictInt(0),
            match_variants: StrictBool(false),
            max_height: StrictFloat(0.0),
            min_height: StrictFloat(0.0),
            separation_threshold: StrictFloat(2.0),
            separation_weight: StrictFloat(1.0),
            use_center_of_mass: StrictBool(false),
            extra: Extra::default(),
        }
    }
}

impl Typed for Flocking {}

/// `minecraft:game_event_movement_tracking`: the entity emits the `entityMove`, `swim` and
/// `flap` game events while it moves.
#[doc(alias = "minecraft:game_event_movement_tracking")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct GameEventMovementTracking {
    /// `emit_flap`: emits `flap` while flying; default `false`.
    #[facet(default = StrictBool(false))]
    pub emit_flap: StrictBool,
    /// `emit_move`: emits `entityMove` while moving on the ground; default `true`.
    #[facet(default = StrictBool(true))]
    pub emit_move: StrictBool,
    /// `emit_swim`: emits `swim` while swimming; default `true`.
    #[facet(default = StrictBool(true))]
    pub emit_swim: StrictBool,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for GameEventMovementTracking {
    fn default() -> Self {
        Self {
            emit_flap: StrictBool(false),
            emit_move: StrictBool(true),
            emit_swim: StrictBool(true),
            extra: Extra::default(),
        }
    }
}

impl Typed for GameEventMovementTracking {}

/// `minecraft:grows_crop`: the entity may grow crops it walks over (the bee).
#[doc(alias = "minecraft:grows_crop")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct GrowsCrop {
    /// `chance`: the chance per tick to grow a crop; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub chance: StrictFloat<f32>,
    /// `charges`: how many crops it can grow; default 10.
    #[facet(default = StrictInt(10_i32))]
    pub charges: StrictInt<i32>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for GrowsCrop {
    fn default() -> Self {
        Self {
            chance: StrictFloat(0.0),
            charges: StrictInt(10),
            extra: Extra::default(),
        }
    }
}

impl Typed for GrowsCrop {}

/// `minecraft:physics`: gravity and collision.
#[doc(alias = "minecraft:physics")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Physics {
    /// `has_collision`: the entity collides with blocks; default `true`.
    #[facet(default = StrictBool(true))]
    pub has_collision: StrictBool,
    /// `has_gravity`: the entity falls; default `true`.
    #[facet(default = StrictBool(true))]
    pub has_gravity: StrictBool,
    /// `push_towards_closest_space`: the entity is pushed out of blocks towards the nearest free
    /// space; default `false`.
    #[facet(default = StrictBool(false))]
    pub push_towards_closest_space: StrictBool,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            has_collision: StrictBool(true),
            has_gravity: StrictBool(true),
            push_towards_closest_space: StrictBool(false),
            extra: Extra::default(),
        }
    }
}

impl Typed for Physics {}

/// `minecraft:collision_box`: the size of the entity's collision box.
#[doc(alias = "minecraft:collision_box")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct CollisionBox {
    /// `height`: the box's height in blocks, within ±100 000 000; default 1.8.
    #[facet(default = StrictFloat(1.8_f32))]
    pub height: StrictFloat<f32>,
    /// `width`: the box's width and depth in blocks, within ±100 000 000; default 0.6.
    #[facet(default = StrictFloat(0.6_f32))]
    pub width: StrictFloat<f32>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for CollisionBox {
    fn default() -> Self {
        Self::new(0.6, 1.8)
    }
}

impl CollisionBox {
    /// A box `width` wide and `height` high.
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            height: StrictFloat(height),
            width: StrictFloat(width),
            extra: Extra::default(),
        }
    }
}

impl Typed for CollisionBox {
    // The game's schema bounds both members to −100000000 ..= 100000000.
    fn check(&mut self, cx: &mut Check<'_>) {
        const BOUND: (f32, f32) = (-100_000_000.0, 100_000_000.0);
        cx.within(
            &JsonPath::new().child("height"),
            &mut self.height.0,
            BOUND,
            1.8,
        );
        cx.within(
            &JsonPath::new().child("width"),
            &mut self.width.0,
            BOUND,
            0.6,
        );
    }
}

/// `minecraft:block_movement_slowdown_immunity`: blocks that do not slow the entity down
/// (powder snow, cobwebs, sweet berry bushes).
#[doc(alias = "minecraft:block_movement_slowdown_immunity")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct BlockMovementSlowdownImmunity {
    /// `blocks`: the blocks.
    #[facet(default)]
    pub blocks: Vec<BlockDescriptor>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Typed for BlockMovementSlowdownImmunity {
    fn check(&mut self, cx: &mut Check<'_>) {
        let at = JsonPath::new().child("blocks");
        for (i, block) in self.blocks.iter().enumerate() {
            cx.extend(block.check(&at.index(i)));
        }
    }
}

impl MolangFields for BlockMovementSlowdownImmunity {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_blocks(&mut self.blocks, visitor);
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
        #[derive(Clone, Debug, PartialEq, Facet)]
        pub struct $name {
            /// `max_turn`: the most the mob turns per tick, in degrees; default 30.
            #[facet(default = StrictFloat(MAX_TURN))]
            pub max_turn: StrictFloat<f32>,
            /// Members the component does not define, kept for write-back.
            #[facet(skip, rename = "\u{0}extra")]
            pub extra: Extra,
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    max_turn: StrictFloat(MAX_TURN),
                    extra: Extra::default(),
                }
            }
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
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MovementGlide {
    /// `max_turn`: the most the mob turns per tick, in degrees; default 30.
    #[facet(default = StrictFloat(MAX_TURN))]
    pub max_turn: StrictFloat<f32>,
    /// `start_speed`: the speed when it starts gliding; default 0.1.
    #[facet(default = StrictFloat(0.1_f32))]
    pub start_speed: StrictFloat<f32>,
    /// `speed_when_turning`: the speed it slows to when it turns quickly; default 0.2.
    #[facet(default = StrictFloat(0.2_f32))]
    pub speed_when_turning: StrictFloat<f32>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for MovementGlide {
    fn default() -> Self {
        Self {
            max_turn: StrictFloat(MAX_TURN),
            start_speed: StrictFloat(0.1),
            speed_when_turning: StrictFloat(0.2),
            extra: Extra::default(),
        }
    }
}

impl Typed for MovementGlide {}

/// `minecraft:movement.sway`: the mob sways from side to side as if it swims.
// `movement.sway`: the defaults are 0.5 and 0.05, in the typed and the legacy reader alike.
#[doc(alias = "minecraft:movement.sway")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MovementSway {
    /// `max_turn`: the most the mob turns per tick, in degrees; default 30.
    #[facet(default = StrictFloat(MAX_TURN))]
    pub max_turn: StrictFloat<f32>,
    /// `sway_frequency`: the multiplier of the sway's frequency; default 0.5.
    #[facet(default = StrictFloat(0.5_f32))]
    pub sway_frequency: StrictFloat<f32>,
    /// `sway_amplitude`: the strength of the sway; default 0.05.
    #[facet(default = StrictFloat(0.05_f32))]
    pub sway_amplitude: StrictFloat<f32>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for MovementSway {
    fn default() -> Self {
        Self {
            max_turn: StrictFloat(MAX_TURN),
            sway_frequency: StrictFloat(0.5),
            sway_amplitude: StrictFloat(0.05),
            extra: Extra::default(),
        }
    }
}

impl Typed for MovementSway {}

/// `minecraft:movement.jump`: the mob jumps as it moves, with a delay between jumps (the
/// slime's move control).
// `movement.jump`: `jump_delay` is a float range, (0, 0) by default; the legacy parser takes a
// number, `[min, max]` or `{range_min, range_max}`.
#[doc(alias = "minecraft:movement.jump")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MovementJump {
    /// `max_turn`: the most the mob turns per tick, in degrees; default 30.
    #[facet(default = StrictFloat(MAX_TURN))]
    pub max_turn: StrictFloat<f32>,
    /// `jump_delay`: the delay after landing before the next jump, a range; default 0 to 0.
    #[facet(default = FloatRange::default())]
    pub jump_delay: FloatRange,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for MovementJump {
    fn default() -> Self {
        Self {
            max_turn: StrictFloat(MAX_TURN),
            jump_delay: FloatRange::default(),
            extra: Extra::default(),
        }
    }
}

impl Typed for MovementJump {
    const KEEP_NESTED: bool = true;

    /// Below 1.26.60 a `{range_min, range_max}` delay is the typed `{min, max}`.
    fn prepare(json: &mut VObject, cx: &mut Check<'_>) -> bool {
        if !matches!(cx.mode(), ConvertMode::Lenient) {
            return false;
        }
        legacy_range(json, "jump_delay", false, &JsonPath::new(), cx)
    }

    fn check(&mut self, cx: &mut Check<'_>) {
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
    cx: &mut Check<'_>,
) -> bool {
    let Some(range) = json.get_mut(key).and_then(Value::as_object_mut) else {
        return false;
    };
    let member = at.child(key);
    let mut changed = false;
    for typed in ["min", "max"] {
        if range.remove(typed).is_some() {
            cx.push(
                DiagnosticCode::DroppedField { text: None },
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
                DiagnosticCode::ValueRewritten { text: None },
            )
        } else {
            let read = i64::from(value.as_bool() == Some(true));
            (
                Value::from(read),
                DiagnosticCode::InvalidValue { text: None },
            )
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
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct RailMovement {
    /// `max_speed`: the most speed on a rail; default 0.4.
    #[facet(default = StrictFloat(0.4_f32))]
    pub max_speed: StrictFloat<f32>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for RailMovement {
    fn default() -> Self {
        Self {
            max_speed: StrictFloat(0.4),
            extra: Extra::default(),
        }
    }
}

impl Typed for RailMovement {}

/// `minecraft:dash_action`: a rideable entity can dash.
// `dash_action`: the defaults are 1, 1, 1 and false; `entity` is 0.
#[doc(alias = "minecraft:dash_action")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct DashAction {
    /// `cooldown_time`: the dash's cooldown in seconds; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub cooldown_time: StrictFloat<f32>,
    /// `horizontal_momentum`: the dash's horizontal momentum; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub horizontal_momentum: StrictFloat<f32>,
    /// `vertical_momentum`: the dash's vertical momentum; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub vertical_momentum: StrictFloat<f32>,
    /// `can_dash_underwater`: the entity can dash under water; default `false`.
    #[facet(default = StrictBool(false))]
    pub can_dash_underwater: StrictBool,
    /// `direction`: whose look the momentum follows; default `entity`.
    #[facet(default = DashDirection::Entity)]
    pub direction: DashDirection,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for DashAction {
    fn default() -> Self {
        Self {
            cooldown_time: StrictFloat(1.0),
            horizontal_momentum: StrictFloat(1.0),
            vertical_momentum: StrictFloat(1.0),
            can_dash_underwater: StrictBool(false),
            direction: DashDirection::Entity,
            extra: Extra::default(),
        }
    }
}

impl Typed for DashAction {}

/// Whose look a [`DashAction`]'s momentum follows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
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
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Buoyant {
    /// `base_buoyancy`: how much the entity floats; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub base_buoyancy: StrictFloat<f32>,
    /// `apply_gravity`: gravity applies each tick; default `true`.
    #[facet(default = StrictBool(true))]
    pub apply_gravity: StrictBool,
    /// `movement_type`: the vertical movement applied; default `waves`.
    #[facet(default = BuoyancyMovement::Waves)]
    pub movement_type: BuoyancyMovement,
    /// `big_wave_probability`: the chance of a big wave (with `waves`); default 0.03.
    #[facet(default = StrictFloat(0.03_f32))]
    pub big_wave_probability: StrictFloat<f32>,
    /// `big_wave_speed`: the speed multiplier of a big wave; default 10.
    #[facet(default = StrictFloat(10.0_f32))]
    pub big_wave_speed: StrictFloat<f32>,
    /// `liquid_blocks`: the liquid blocks the entity floats on; default none.
    #[facet(default)]
    pub liquid_blocks: Vec<BlockDescriptor>,
    /// `drag_down_on_buoyancy_removed`: how much the entity is dragged down when the
    /// component goes; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub drag_down_on_buoyancy_removed: StrictFloat<f32>,
    /// `can_auto_step_from_liquid`: the entity steps out of a liquid onto a solid neighbour it
    /// is pushed against; default `false`.
    #[facet(default = StrictBool(false))]
    pub can_auto_step_from_liquid: StrictBool,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for Buoyant {
    fn default() -> Self {
        Self {
            base_buoyancy: StrictFloat(1.0),
            apply_gravity: StrictBool(true),
            movement_type: BuoyancyMovement::Waves,
            big_wave_probability: StrictFloat(0.03),
            big_wave_speed: StrictFloat(10.0),
            liquid_blocks: Vec::new(),
            drag_down_on_buoyancy_removed: StrictFloat(0.0),
            can_auto_step_from_liquid: StrictBool(false),
            extra: Extra::default(),
        }
    }
}

impl Typed for Buoyant {
    fn check(&mut self, cx: &mut Check<'_>) {
        let blocks = JsonPath::new().child("liquid_blocks");
        for (i, block) in self.liquid_blocks.iter().enumerate() {
            cx.extend(block.check(&blocks.index(i)));
        }
    }
}

impl MolangFields for Buoyant {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_block_list("liquid_blocks", &mut self.liquid_blocks, visitor);
    }
}

/// The vertical movement of a [`Buoyant`] entity (`buoyancy_movement_type`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
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

/// Hands the `tags` of every descriptor of `blocks` (member `key`) to the Molang walk.
fn visit_block_list(key: &str, blocks: &mut [BlockDescriptor], visitor: &mut MolangVisitor<'_>) {
    visitor.enter(key.into());
    for (i, block) in blocks.iter_mut().enumerate() {
        visitor.enter(i.into());
        visit_block_tags(block, visitor);
        visitor.leave();
    }
    visitor.leave();
}

/// `minecraft:teleport`: how the entity teleports.
// `teleport`: `projectile_filter` is absent by default; the typed type binds no camel-case
// spelling.
#[doc(alias = "minecraft:teleport")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Teleport {
    /// `random_teleports`: it teleports at random; default `true`.
    #[facet(default = StrictBool(true))]
    pub random_teleports: StrictBool,
    /// `teleports_on_projectile_hit`: it teleports away from a projectile about to hit it;
    /// default `false`.
    #[facet(default = StrictBool(false))]
    pub teleports_on_projectile_hit: StrictBool,
    /// `projectile_filter`: which projectiles make it teleport; absent means all.
    #[facet(default)]
    pub projectile_filter: Option<Filter>,
    /// `min_random_teleport_time`: the least seconds between random teleports; default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub min_random_teleport_time: StrictFloat<f32>,
    /// `max_random_teleport_time`: the most seconds between random teleports; default 20.
    #[facet(default = StrictFloat(20.0_f32))]
    pub max_random_teleport_time: StrictFloat<f32>,
    /// `random_teleport_cube`: the area a random teleport lands in; default 32 × 16 × 32.
    #[facet(default = Vec3::new(32.0, 16.0, 32.0))]
    pub random_teleport_cube: Vec3,
    /// `target_distance`: the farthest it teleports while chasing a target; default 16.
    #[facet(default = StrictFloat(16.0_f32))]
    pub target_distance: StrictFloat<f32>,
    /// `target_teleport_chance`: the chance it teleports, 0 to 1; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub target_teleport_chance: StrictFloat<f32>,
    /// `light_teleport_chance`: the chance modifier in daylight; default 0.01.
    #[facet(default = StrictFloat(0.01_f32))]
    pub light_teleport_chance: StrictFloat<f32>,
    /// `dark_teleport_chance`: the chance modifier in darkness; default 0.01.
    #[facet(default = StrictFloat(0.01_f32))]
    pub dark_teleport_chance: StrictFloat<f32>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for Teleport {
    fn default() -> Self {
        Self {
            random_teleports: StrictBool(true),
            teleports_on_projectile_hit: StrictBool(false),
            projectile_filter: None,
            min_random_teleport_time: StrictFloat(0.0),
            max_random_teleport_time: StrictFloat(20.0),
            random_teleport_cube: Vec3::new(32.0, 16.0, 32.0),
            target_distance: StrictFloat(16.0),
            target_teleport_chance: StrictFloat(1.0),
            light_teleport_chance: StrictFloat(0.01),
            dark_teleport_chance: StrictFloat(0.01),
            extra: Extra::default(),
        }
    }
}

impl Typed for Teleport {
    /// The legacy reader binds no camel-case spelling: the schema reader's
    /// validation before the read refuses one, so nothing is renamed here.
    fn prepare(json: &mut VObject, cx: &mut Check<'_>) -> bool {
        tell_partial_vector(json, "random_teleport_cube", &JsonPath::new(), cx);
        false
    }

    fn check(&mut self, cx: &mut Check<'_>) {
        if let Some(filter) = &self.projectile_filter {
            cx.extend(filter.check(&JsonPath::new().child("projectile_filter")));
        }
    }
}

/// `minecraft:trail`: the entity leaves a trail of blocks (mob griefing permitting).
// `trail`: `block_type` defaults to "air", `spawn_offset` to 0 and `radius` to −1 (within
// −1 ..= 16); the new `replaceable_blocks` (empty: air only) and `force_replace_matched_blocks`
// are ignored by the legacy reader.
#[doc(alias = "minecraft:trail")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Trail {
    /// `spawn_filter`: when the block spawns; absent means always.
    #[facet(default)]
    pub spawn_filter: Option<Filter>,
    /// `block_type`: the block spawned; default `air`.
    #[facet(default = BlockDescriptor::block(BlockId::parse("air")))]
    pub block_type: BlockDescriptor,
    /// `spawn_offset`: where the block spawns relative to the entity; default 0, 0, 0.
    #[facet(default = Vec3::new(0.0, 0.0, 0.0))]
    pub spawn_offset: Vec3,
    /// `replaceable_blocks`: the blocks the trail may replace; empty means air only.
    #[facet(default)]
    pub replaceable_blocks: Vec<BlockDescriptor>,
    /// `radius`: the trail covers a square of `2 × radius + 1` blocks; −1 (the default) is the
    /// legacy four-corner sampling. −1 to 16.
    #[facet(default = StrictInt(-1_i32))]
    pub radius: StrictInt<i32>,
    /// `force_replace_matched_blocks`: replaceable blocks are replaced even where placement
    /// rules forbid it; default `false`.
    #[facet(default = StrictBool(false))]
    pub force_replace_matched_blocks: StrictBool,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for Trail {
    fn default() -> Self {
        Self {
            spawn_filter: None,
            block_type: BlockDescriptor::block(BlockId::parse("air")),
            spawn_offset: Vec3::new(0.0, 0.0, 0.0),
            replaceable_blocks: Vec::new(),
            radius: StrictInt(-1),
            force_replace_matched_blocks: StrictBool(false),
            extra: Extra::default(),
        }
    }
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
    fn prepare(json: &mut VObject, cx: &mut Check<'_>) -> bool {
        tell_partial_vector(json, "spawn_offset", &JsonPath::new(), cx);
        if !matches!(cx.mode(), ConvertMode::Lenient) {
            return false;
        }
        let mut changed = false;
        for member in TRAIL_NEW_MEMBERS {
            if json.remove(member).is_some() {
                cx.push(
                    DiagnosticCode::DroppedField { text: None },
                    Severity::Info,
                    &JsonPath::new().child(member),
                );
                changed = true;
            }
        }
        changed
    }

    fn check(&mut self, cx: &mut Check<'_>) {
        if let Some(filter) = &self.spawn_filter {
            cx.extend(filter.check(&JsonPath::new().child("spawn_filter")));
        }
        cx.extend(self.block_type.check(&JsonPath::new().child("block_type")));
        let blocks = JsonPath::new().child("replaceable_blocks");
        for (i, block) in self.replaceable_blocks.iter().enumerate() {
            cx.extend(block.check(&blocks.index(i)));
        }
        // The typed reader bounds `radius` to -1 ..= 16.
        cx.within(
            &JsonPath::new().child("radius"),
            &mut self.radius.0,
            (-1, 16),
            -1,
        );
    }
}

impl MolangFields for Trail {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter("block_type".into());
        visit_block_tags(&mut self.block_type, visitor);
        visitor.leave();
        visit_block_list("replaceable_blocks", &mut self.replaceable_blocks, visitor);
    }
}

/// `minecraft:annotation.break_door`: the entity breaks doors, as its navigation allows.
// `annotation.break_door`: the break time defaults to 12 and the minimum difficulty to 3
// (`hard`).
#[doc(alias = "minecraft:annotation.break_door")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct AnnotationBreakDoor {
    /// `break_time`: the seconds needed to break a door; default 12.
    #[facet(default = StrictFloat(12.0_f32))]
    pub break_time: StrictFloat<f32>,
    /// `min_difficulty`: the lowest difficulty at which it breaks doors; default `hard`.
    #[facet(default = Difficulty::Hard)]
    pub min_difficulty: Difficulty,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for AnnotationBreakDoor {
    fn default() -> Self {
        Self {
            break_time: StrictFloat(12.0),
            min_difficulty: Difficulty::Hard,
            extra: Extra::default(),
        }
    }
}

impl Typed for AnnotationBreakDoor {}

/// A world difficulty.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
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
