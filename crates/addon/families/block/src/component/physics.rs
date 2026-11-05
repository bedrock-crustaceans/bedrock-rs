//! How a block behaves in the world: collision and selection, mining and explosions, light,
//! friction, fire, pistons, liquids, weather, placement and the remaining plain components.

use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, StrictString};
use facet::Facet;
use indexmap::IndexMap;

use super::types::{BoundingBox, Bounds, Direction, Facing, Vec3};
use super::{DESCRIPTOR_TAGS, ITEM_TAGS};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::shared::{BlockDescriptor, ItemDescriptor};

/// `minecraft:selection_box`: what the player's cursor hits.
///
/// JSON: `true` / `false` or one box.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum SelectionBox {
    /// The full block (`true`) or none (`false`).
    Enabled(bool),
    /// One box.
    Box(BoundingBox),
}

/// `minecraft:destructible_by_mining`: how long the block takes to mine.
///
/// JSON: `false` (cannot be mined), `true`, or `{"seconds_to_destroy", "item_specific_speeds"?}`.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum DestructibleByMining {
    /// Minable (`true`) at the default speed, or not at all (`false`).
    Enabled(bool),
    /// The object form.
    Params(MiningParams),
}

impl DestructibleByMining {
    /// Mined in `seconds` by hand, with no item-specific speeds.
    pub fn seconds(seconds: f32) -> Self {
        DestructibleByMining::Params(MiningParams {
            seconds_to_destroy: StrictFloat(seconds),
            item_specific_speeds: Vec::new(),
            extra: Extra::default(),
        })
    }
}

/// The object form of [`DestructibleByMining`].
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MiningParams {
    /// Seconds to mine by hand (JSON `seconds_to_destroy`, required, `>= 0`).
    pub seconds_to_destroy: StrictFloat<f32>,
    /// Faster or slower mining with particular items (JSON `item_specific_speeds`; from
    /// 1.21.30).
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub item_specific_speeds: Vec<ItemSpecificSpeed>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One entry of `item_specific_speeds`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ItemSpecificSpeed {
    /// The item (JSON `item`, required): a name or `{"tags": "<molang>"}`.
    pub item: ItemDescriptor,
    /// Seconds to mine with it (JSON `destroy_speed`, required; negative: cannot be mined).
    pub destroy_speed: StrictFloat<f32>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MolangFields for DestructibleByMining {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let DestructibleByMining::Params(params) = self else {
            return;
        };
        visitor.enter("item_specific_speeds".into());
        for (index, speed) in params.item_specific_speeds.iter_mut().enumerate() {
            if let ItemDescriptor::Tags { tags, .. } = &mut speed.item {
                visitor.enter(index.into());
                visitor.enter("item".into());
                visitor.field("tags", &ITEM_TAGS, tags);
                visitor.leave();
                visitor.leave();
            }
        }
        visitor.leave();
    }
}

/// `minecraft:destructible_by_explosion`: how much the block resists explosions.
///
/// JSON: `false` (immune), `true`, or `{"explosion_resistance"}`.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum DestructibleByExplosion {
    /// Destructible (`true`) at the default resistance, or immune (`false`).
    Enabled(bool),
    /// The object form.
    Params(ExplosionParams),
}

/// The object form of [`DestructibleByExplosion`].
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ExplosionParams {
    /// JSON `explosion_resistance` (required, `>= 0`). Read by the slice's scalar rules: a
    /// number under the strict reader, a number or a bool under the legacy one.
    pub explosion_resistance: f32,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:friction`: `0.0..=0.9`; 0.4 is vanilla's default.
#[derive(Clone, Copy, Debug, PartialEq, Facet)]
#[facet(transparent)]
pub struct Friction(pub StrictFloat<f32>);

/// `minecraft:light_dampening`: how much light the block absorbs, `0..=15`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct LightDampening(pub StrictInt<u8>);

/// `minecraft:light_emission`: the light level the block gives off, `0..=15`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct LightEmission(pub StrictInt<u8>);

/// `minecraft:display_name`: a literal name or a localisation key.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct DisplayName(pub StrictString);

/// `minecraft:loot`: the loot table path.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct Loot(pub StrictString);

/// `minecraft:tags`: tags of the block (`namespace:tag_name`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct Tags(pub Vec<StrictString>);

/// `minecraft:sound`: the block's sound type (engine 1.26.50).
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct Sound(pub StrictString);

/// `minecraft:flammable`: how fire spreads to and destroys the block.
///
/// JSON: `true` / `false`, or `{"catch_chance_modifier"?, "destroy_chance_modifier"?,
/// "lava_flammable"?}`.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum Flammable {
    /// Flammable at the default odds (`true`) or not (`false`).
    Enabled(bool),
    /// The object form.
    Params(FlammableParams),
}

/// The object form of [`Flammable`].
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct FlammableParams {
    /// JSON `catch_chance_modifier`; 5 when absent.
    #[facet(default)]
    pub catch_chance_modifier: Option<StrictInt<u16>>,
    /// JSON `destroy_chance_modifier`; 20 when absent.
    #[facet(default)]
    pub destroy_chance_modifier: Option<StrictInt<u16>>,
    /// Whether lava sets it alight (JSON `lava_flammable`; engine 1.26.30).
    #[facet(default)]
    pub lava_flammable: Option<LavaFlammable>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl FlammableParams {
    /// The chance to catch fire: `catch_chance_modifier`, 5 when absent.
    pub fn catch_chance_modifier(&self) -> u16 {
        self.catch_chance_modifier.map_or(5, |chance| chance.0)
    }

    /// The chance to burn away: `destroy_chance_modifier`, 20 when absent.
    pub fn destroy_chance_modifier(&self) -> u16 {
        self.destroy_chance_modifier.map_or(20, |chance| chance.0)
    }
}

/// `flammable.lava_flammable`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum LavaFlammable {
    /// `never`.
    Never,
    /// `always`.
    Always,
}

/// `minecraft:movable`: how pistons move the block.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Movable {
    /// JSON `movement_type` (required).
    pub movement_type: MovementType,
    /// JSON `sticky`; `none` when absent.
    #[facet(default)]
    pub sticky: Option<Sticky>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `movable.movement_type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum MovementType {
    /// `push_pull`.
    PushPull,
    /// `push`.
    Push,
    /// `popped`.
    Popped,
    /// `immovable`.
    Immovable,
}

/// `movable.sticky`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum Sticky {
    /// `none`.
    None,
    /// `same`.
    Same,
}

/// `minecraft:random_offset`: a random shift of the model and boxes per position.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct RandomOffset {
    /// JSON `x`.
    #[facet(default)]
    pub x: Option<OffsetAxis>,
    /// JSON `y`.
    #[facet(default)]
    pub y: Option<OffsetAxis>,
    /// JSON `z`.
    #[facet(default)]
    pub z: Option<OffsetAxis>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One axis of [`RandomOffset`].
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct OffsetAxis {
    /// The offset range in sixteenths (JSON `range`, required).
    pub range: Bounds,
    /// How many steps the range is cut into (JSON `steps`); 0 (continuous) when absent.
    #[facet(default)]
    pub steps: Option<StrictInt<u32>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl OffsetAxis {
    /// The number of steps the range is divided into: `steps`, 0 (continuous) when absent.
    pub fn steps(&self) -> u32 {
        self.steps.map_or(0, |steps| steps.0)
    }
}

/// `minecraft:support`: what the block's top can hold.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Support {
    /// JSON `shape` (required).
    pub shape: SupportShape,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `support.shape`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum SupportShape {
    /// `fence`.
    Fence,
    /// `stair`.
    Stair,
}

/// `minecraft:tick`: a scheduled tick that fires the block's custom components.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Tick {
    /// The delay in ticks, `[min, max]` with `min <= max` (JSON `interval_range`, required).
    pub interval_range: [StrictInt<u32>; 2],
    /// Tick again after each tick (JSON `looping`); `true` when absent.
    #[facet(default)]
    pub looping: Option<StrictBool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Tick {
    /// Whether it ticks again after each tick: `looping`, `true` when absent.
    pub fn looping(&self) -> bool {
        self.looping.is_none_or(|b| b.0)
    }
}

/// `minecraft:entity_fall_on`: report falls onto the block to custom components.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct EntityFallOn {
    /// JSON `min_fall_distance`; 0 when absent.
    #[facet(default)]
    pub min_fall_distance: Option<StrictFloat<f32>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl EntityFallOn {
    /// The shortest fall that is reported: `min_fall_distance`, 0 when absent.
    pub fn min_fall_distance(&self) -> f32 {
        self.min_fall_distance.map_or(0.0, |distance| distance.0)
    }
}

/// `minecraft:leashable`: leads can be tied to the block.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Leashable {
    /// Where the knot sits (JSON `offset`); `[0, 0.25, 0]` when absent.
    #[facet(default)]
    pub offset: Option<Vec3>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Leashable {
    /// Where the knot sits: `offset`, `[0, 0.25, 0]` when absent.
    pub fn offset(&self) -> Vec3 {
        self.offset.unwrap_or(super::types::vec3(0.0, 0.25, 0.0))
    }
}

/// `minecraft:chest_obstruction`: whether the block stops a chest below it opening.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct ChestObstruction {
    /// JSON `obstruction_rule`.
    #[facet(default)]
    pub obstruction_rule: Option<ObstructionRule>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl ChestObstruction {
    /// The rule: `obstruction_rule`, `shape` when absent (the game's constructor default).
    pub fn obstruction_rule(&self) -> ObstructionRule {
        self.obstruction_rule.unwrap_or(ObstructionRule::Shape)
    }
}

/// `chest_obstruction.obstruction_rule`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum ObstructionRule {
    /// `always`.
    Always,
    /// `never`.
    Never,
    /// `shape`: by the collision shape.
    Shape,
}

/// `minecraft:connection_rule`: which blocks connect to this one (fences, walls, panes).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct ConnectionRule {
    /// JSON `accepts_connections_from`.
    #[facet(default)]
    pub accepts_connections_from: Option<AcceptsConnections>,
    /// JSON `enabled_directions`; all four when absent.
    #[facet(default)]
    pub enabled_directions: Option<Vec<Direction>>,
    /// JSON `vanilla_connections_from`: base-game packs only.
    #[facet(default)]
    pub vanilla_connections_from: Option<Vec<ConnectionId>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl ConnectionRule {
    /// Who connects: `accepts_connections_from`, `all` when absent (the game's constructor default).
    pub fn accepts_connections_from(&self) -> AcceptsConnections {
        self.accepts_connections_from
            .unwrap_or(AcceptsConnections::All)
    }

    /// The directions: `enabled_directions`, all four when absent, in the game's default order
    /// south, north, east, west.
    pub fn enabled_directions(&self) -> Vec<Direction> {
        self.enabled_directions.clone().unwrap_or_else(|| {
            vec![
                Direction::South,
                Direction::North,
                Direction::East,
                Direction::West,
            ]
        })
    }
}

/// `connection_rule.accepts_connections_from`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum AcceptsConnections {
    /// `none`.
    None,
    /// `only_fences`.
    OnlyFences,
    /// `all`.
    All,
}

/// `connection_rule.vanilla_connections_from`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum ConnectionId {
    /// `fence`.
    Fence,
    /// `nether_brick_fence`.
    NetherBrickFence,
    /// `wall`.
    Wall,
    /// `thin_fence`.
    ThinFence,
}

/// `minecraft:crafting_table`: the block opens a crafting grid.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct CraftingTable {
    /// Recipe tags the table crafts (JSON `crafting_tags`, required; at most 64, each at most
    /// 64 characters).
    pub crafting_tags: Vec<StrictString>,
    /// The screen title, a localisation key (JSON `table_name`); `""` when absent.
    #[facet(default)]
    pub table_name: Option<StrictString>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl CraftingTable {
    /// The screen title: `table_name`, `""` when absent.
    pub fn table_name(&self) -> &str {
        self.table_name.as_ref().map_or("", |name| name.0.as_str())
    }
}

/// `minecraft:placement_filter`: where the block may be placed or survive.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct PlacementFilter {
    /// JSON `conditions` (at most 64); any one passing allows the placement.
    #[facet(default)]
    pub conditions: Option<Vec<PlacementCondition>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl PlacementFilter {
    /// The conditions: `conditions`, none when absent.
    pub fn conditions(&self) -> &[PlacementCondition] {
        self.conditions.as_deref().unwrap_or_default()
    }
}

/// One entry of `placement_filter.conditions`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct PlacementCondition {
    /// The faces it may be placed against (JSON `allowed_faces`).
    #[facet(default)]
    pub allowed_faces: Option<Vec<PlacementFace>>,
    /// The blocks it may be placed on (JSON `block_filter`; at most 64).
    #[facet(default)]
    pub block_filter: Option<Vec<BlockDescriptor>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl PlacementCondition {
    /// The faces the block may be placed against: `allowed_faces`, none listed when absent.
    pub fn allowed_faces(&self) -> &[PlacementFace] {
        self.allowed_faces.as_deref().unwrap_or_default()
    }

    /// The blocks it may be placed on: `block_filter`, none listed when absent.
    pub fn block_filter(&self) -> &[BlockDescriptor] {
        self.block_filter.as_deref().unwrap_or_default()
    }
}

/// `placement_filter.conditions[].allowed_faces`: a face, `side` or `all`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum PlacementFace {
    /// `up`.
    Up,
    /// `down`.
    Down,
    /// `north`.
    North,
    /// `south`.
    South,
    /// `east`.
    East,
    /// `west`.
    West,
    /// `side`: the four horizontal faces.
    Side,
    /// `all`.
    All,
}

impl MolangFields for PlacementFilter {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter("conditions".into());
        for (index, condition) in self.conditions.iter_mut().flatten().enumerate() {
            visitor.enter(index.into());
            visitor.enter("block_filter".into());
            for (i, descriptor) in condition.block_filter.iter_mut().flatten().enumerate() {
                if let BlockDescriptor::Tags { tags, .. } | BlockDescriptor::Mixed { tags, .. } =
                    descriptor
                {
                    visitor.enter(i.into());
                    visitor.field("tags", &DESCRIPTOR_TAGS, tags);
                    visitor.leave();
                }
            }
            visitor.leave();
            visitor.leave();
        }
        visitor.leave();
    }
}

/// `minecraft:precipitation_interactions`: what rain and snow do on the block.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct PrecipitationInteractions {
    /// JSON `precipitation_behavior` (required).
    pub precipitation_behavior: PrecipitationBehavior,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `precipitation_interactions.precipitation_behavior`.
///
/// `snow_log_no_collision`, the 1.26.20 spelling of `snowlogging`, is read and normalised to
/// [`PrecipitationBehavior::Snowlogging`] when the block loads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum PrecipitationBehavior {
    /// `none`.
    None,
    /// `obstruct_rain`.
    ObstructRain,
    /// `obstruct_rain_accumulate_snow`.
    ObstructRainAccumulateSnow,
    /// `snowlogging`: snow layers can share the block's space.
    Snowlogging,
    /// `snow_log_no_collision`: the 1.26.20 spelling of `snowlogging`.
    SnowLogNoCollision,
}

/// `minecraft:liquid_detection`: how the block reacts to liquids.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct LiquidDetection {
    /// JSON `detection_rules`: exactly one rule.
    pub detection_rules: Vec<DetectionRule>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One entry of `liquid_detection.detection_rules`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DetectionRule {
    /// JSON `liquid_type`; `water` is the only liquid.
    #[facet(default)]
    pub liquid_type: Option<LiquidType>,
    /// JSON `can_contain_liquid`.
    #[facet(default)]
    pub can_contain_liquid: Option<StrictBool>,
    /// JSON `on_liquid_touches`.
    #[facet(default)]
    pub on_liquid_touches: Option<LiquidReaction>,
    /// JSON `stops_liquid_flowing_from_direction`.
    #[facet(default)]
    pub stops_liquid_flowing_from_direction: Option<Vec<Facing>>,
    /// JSON `use_liquid_clipping` (from 1.26.0; older documents get `true`).
    #[facet(default)]
    pub use_liquid_clipping: Option<StrictBool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DetectionRule {
    /// Whether the block can hold the liquid: `can_contain_liquid`, `false` when absent.
    pub fn can_contain_liquid(&self) -> bool {
        self.can_contain_liquid.is_some_and(|b| b.0)
    }

    /// The directions the block stops the liquid flowing from:
    /// `stops_liquid_flowing_from_direction`, none when absent.
    pub fn stops_liquid_flowing_from_direction(&self) -> &[Facing] {
        self.stops_liquid_flowing_from_direction
            .as_deref()
            .unwrap_or_default()
    }

    /// Whether the liquid is clipped to the block's shape: `use_liquid_clipping`, `false` when
    /// absent.
    pub fn use_liquid_clipping(&self) -> bool {
        self.use_liquid_clipping.is_some_and(|b| b.0)
    }
}

/// `detection_rules[].liquid_type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum LiquidType {
    /// `water`.
    Water,
}

/// `detection_rules[].on_liquid_touches`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum LiquidReaction {
    /// `broken`.
    Broken,
    /// `popped`.
    Popped,
    /// `blocking`.
    Blocking,
    /// `no_reaction`.
    NoReaction,
}

/// `minecraft:flower_pottable`: the block can go in a flower pot.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct FlowerPottable {
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:replaceable`: placing another block replaces this one.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Replaceable {
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:block_entity`: the block has a block entity (engine 1.26.30, Upcoming Creator
/// Features; base components only).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct BlockEntity {
    /// JSON `dynamic_properties`.
    #[facet(default)]
    pub dynamic_properties: Option<StrictBool>,
    /// Entities stored in the block through scripting (JSON `entity_storage`, 1.26.60, Upcoming
    /// Creator Features); `false` when absent.
    #[facet(default)]
    pub entity_storage: Option<StrictBool>,
    /// A container of the block (JSON `container`, Upcoming Creator Features; not with a
    /// `crafting_table`).
    #[facet(default)]
    pub container: Option<ContainerData>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `block_entity.container`: the slot count and title of the block's container.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ContainerData {
    /// The number of slots, `1..=54` (JSON `slot_count`, required).
    pub slot_count: StrictInt<u8>,
    /// The container's title, a language key or raw text, non-empty, at most 256 characters
    /// (JSON `title`).
    #[facet(default)]
    pub title: Option<StrictString>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl BlockEntity {
    /// Whether it stores entities: `entity_storage`, `false` when absent.
    pub fn entity_storage(&self) -> bool {
        self.entity_storage.is_some_and(|b| b.0)
    }

    /// Whether scripts may store dynamic properties on it: `dynamic_properties`, `false` when
    /// absent.
    pub fn dynamic_properties(&self) -> bool {
        self.dynamic_properties.is_some_and(|b| b.0)
    }
}

/// `minecraft:instrument_sound`: the note block instrument above (`up`) and below (`down`) the
/// block (engine 1.26.30, Upcoming Creator Features).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(transparent)]
pub struct InstrumentSound(pub IndexMap<String, NoteBlockInstrument>);

/// A note block instrument.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[repr(u8)]
#[non_exhaustive]
#[allow(missing_docs, reason = "each variant is its JSON spelling")]
pub enum NoteBlockInstrument {
    #[facet(rename = "note.harp")]
    Harp,
    #[facet(rename = "note.bd")]
    BassDrum,
    #[facet(rename = "note.snare")]
    Snare,
    #[facet(rename = "note.hat")]
    Hat,
    #[facet(rename = "note.bassattack")]
    BassAttack,
    #[facet(rename = "note.flute")]
    Flute,
    #[facet(rename = "note.bell")]
    Bell,
    #[facet(rename = "note.guitar")]
    Guitar,
    #[facet(rename = "note.chime")]
    Chime,
    #[facet(rename = "note.xylophone")]
    Xylophone,
    #[facet(rename = "note.iron_xylophone")]
    IronXylophone,
    #[facet(rename = "note.cow_bell")]
    CowBell,
    #[facet(rename = "note.didgeridoo")]
    Didgeridoo,
    #[facet(rename = "note.bit")]
    Bit,
    #[facet(rename = "note.banjo")]
    Banjo,
    #[facet(rename = "note.pling")]
    Pling,
    #[facet(rename = "note.trumpet")]
    Trumpet,
    #[facet(rename = "note.trumpet_exposed")]
    TrumpetExposed,
    #[facet(rename = "note.trumpet_weathered")]
    TrumpetWeathered,
    #[facet(rename = "note.trumpet_oxidized")]
    TrumpetOxidized,
    #[facet(rename = "note.zombie")]
    Zombie,
    #[facet(rename = "note.skeleton")]
    Skeleton,
    #[facet(rename = "note.creeper")]
    Creeper,
    #[facet(rename = "note.enderdragon")]
    EnderDragon,
    #[facet(rename = "note.witherskeleton")]
    WitherSkeleton,
    #[facet(rename = "note.piglin")]
    Piglin,
    #[facet(rename = "note.none")]
    None,
}

/// `minecraft:enchantment_power_transmitter` (1.26.60, Upcoming Creator Features): placed in
/// the gap between an enchanting table and a bookshelf, the table stays powered (`true`).
///
/// JSON: a bare bool; `false` is the component's absence at run time.
#[derive(Clone, Copy, Debug, PartialEq, Facet)]
#[facet(transparent)]
pub struct EnchantmentPowerTransmitter(pub StrictBool);

/// `minecraft:vibration_properties` (1.26.60, Upcoming Creator Features): how the block
/// treats vibrations.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct VibrationProperties {
    /// JSON `dampen_vibrations`; `false` when absent.
    #[facet(default)]
    pub dampen_vibrations: Option<StrictBool>,
    /// JSON `occlude_vibrations`; `false` when absent.
    #[facet(default)]
    pub occlude_vibrations: Option<StrictBool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl VibrationProperties {
    /// `dampen_vibrations`, `false` when absent.
    pub fn dampen_vibrations(&self) -> bool {
        self.dampen_vibrations.is_some_and(|b| b.0)
    }

    /// `occlude_vibrations`, `false` when absent.
    pub fn occlude_vibrations(&self) -> bool {
        self.occlude_vibrations.is_some_and(|b| b.0)
    }
}

/// `minecraft:neighbor_change` (1.26.60, Upcoming Creator Features): which neighbouring
/// changes reach the block's custom components.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct NeighborChange {
    /// JSON `neighbor_directions`: 1 to 6 directions, `all` alone; `[all]` when absent.
    #[facet(default)]
    pub neighbor_directions: Option<Vec<NeighborChangeDirection>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl NeighborChange {
    /// The directions observed: `neighbor_directions`, `[all]` when absent (the game's constructor
    /// default).
    pub fn neighbor_directions(&self) -> Vec<NeighborChangeDirection> {
        self.neighbor_directions
            .clone()
            .unwrap_or_else(|| vec![NeighborChangeDirection::All])
    }
}

/// A direction of `neighbor_change`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
#[allow(missing_docs, reason = "each variant is its JSON spelling")]
pub enum NeighborChangeDirection {
    All,
    Up,
    Down,
    North,
    South,
    West,
    East,
}
